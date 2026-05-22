use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use indicatif::ProgressIterator;
use propagation_notebook::{
    region::{ConservationStatus, Region, RegionalTaxonStatus, WetlandIndicator},
    taxonomy::Synonym,
};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
enum Origin {
    Native,
    Introduced,
    Unknown,
}

impl From<Origin> for propagation_notebook::region::Origin {
    fn from(value: Origin) -> Self {
        match value {
            Origin::Native => propagation_notebook::region::Origin::Native,
            Origin::Introduced => propagation_notebook::region::Origin::Introduced,
            Origin::Unknown => propagation_notebook::region::Origin::Unknown,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TaxonInfo {
    name: String,
    c_value: Option<u64>,
    origin: Option<Origin>,
    status: Option<ConservationStatus>,
    wetland_indicator: Option<WetlandIndicator>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RegionInfo {
    name: String,
    bounds: Option<String>,
    taxa: Vec<TaxonInfo>,
    notes: Option<String>,
    // npcs: Vec<NativePlantCommunityInfo>,
}

#[derive(Debug, clap::Parser)]
struct Args {
    region_file: PathBuf,
}

pub async fn import_region<P>(db: &mut toasty::Db, path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let mut txn = db.transaction().await?;

    let mut f = tokio::fs::OpenOptions::new().read(true).open(path).await?;
    let mut info_string = String::new();
    f.read_to_string(&mut info_string).await?;

    let info: RegionInfo = serde_yaml::from_str(&info_string)?;

    // loop through the input list and search for names from our taxonomy that
    // match the given name. Some of these input names may map to the same name in our
    // taxonomy, so we need to eliminate duplicates at the end. We do this by storing
    // the result in a hashmap by result taxon id
    let mut lookups: HashMap<u64, TaxonInfo> = HashMap::default();
    println!("Validating taxa...");
    for taxon_info in info.taxa.into_iter().progress() {
        let t = find_taxon_for_name(&mut txn, &taxon_info.name).await?;
        lookups
            .entry(t.id)
            .and_modify(|existing| {
                // if any of the lumped taxa is native, consider the whole thing native
                if taxon_info.origin == Some(Origin::Native) {
                    existing.origin = taxon_info.origin;
                } else if existing.origin.is_none_or(|x| x == Origin::Unknown) {
                    // any new status overrides unknown
                    existing.origin = taxon_info.origin;
                }
            })
            .or_insert(taxon_info);
    }

    // now insert all unique taxa into the region table
    let mut taxa_create = Vec::new();
    for (id, taxon_info) in lookups.into_iter() {
        taxa_create.push(
            RegionalTaxonStatus::create()
                .taxon_id(id)
                .origin(taxon_info.origin.map(|x| x.into()))
                .conservation_status(taxon_info.status)
                .c_value(taxon_info.c_value),
        );
    }

    let n_taxa = taxa_create.len();
    let region = Region::create()
        .name(info.name)
        .bounds(info.bounds)
        .notes(info.notes)
        .taxon_statuses(taxa_create)
        .exec(&mut txn)
        .await?;

    txn.commit().await?;

    println!(
        "Created region '{}' with {} taxa",
        region.reference(),
        n_taxa
    );

    Ok(())
}

async fn find_taxon_for_name(
    db: &mut dyn toasty::Executor,
    name: &str,
) -> anyhow::Result<propagation_notebook::taxonomy::Taxon> {
    use propagation_notebook::taxonomy::Taxon;
    Ok(match name.parse::<u64>() {
        Ok(val) => Taxon::get_by_id(db, val).await?,
        Err(_) => match Taxon::get_by_complete_name(db, name).await {
            Ok(taxon) => taxon,
            Err(_e) => {
                // tracing::warn!(?e);
                Synonym::filter_by_complete_name(name)
                    .include(Synonym::fields().taxon())
                    .one()
                    .exec(db)
                    .await
                    .map_err(|_| anyhow::anyhow!("Couldn't find a taxon matching {name}"))
                    .map(|synonym| synonym.taxon.get().clone())?
            }
        },
    })
}
