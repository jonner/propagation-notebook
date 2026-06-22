use std::collections::HashMap;

use itertools::Itertools;

use propagation_notebook::taxonomy::{TaxonomicAuthority, itis};

pub trait ImportProgressReporter {
    fn begin_step(&mut self, name: &str, total: usize);
    fn increment(&mut self);
    fn finish_step(&mut self);
}

const CHUNK_SIZE: usize = 500;
pub async fn import_taxa(
    db: &mut toasty::Db,
    taxonomy_db_uri: &str,
    authority: TaxonomicAuthority,
    reporter: &mut dyn ImportProgressReporter,
) -> anyhow::Result<()> {
    let itisdb = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(taxonomy_db_uri)
        .await?;

    let mut txn = db.transaction().await?;

    match authority {
        TaxonomicAuthority::Itis => import_taxa_itis(itisdb, &mut txn, reporter).await?,
    }

    txn.commit().await?;

    Ok(())
}

async fn import_taxa_itis(
    mut itisdb: toasty::Db,
    ourtxn: &mut toasty::Transaction<'_>,
    reporter: &mut dyn ImportProgressReporter,
) -> Result<(), anyhow::Error> {
    // find plant kingdom
    let plant_kingdom = itis::Kingdom::get_by_kingdom_name(&mut itisdb, "Plantae").await?;
    let mut tsn_to_id: HashMap<u64, u64> = HashMap::default();
    println!("Building hierarchy sequence...");
    let mut tsn_to_seq: HashMap<u64, _> = HashMap::default();
    let records = itis::Hierarchy::all()
        .order_by(itis::Hierarchy::fields().hierarchy_string().asc())
        .exec(&mut itisdb)
        .await?;
    for (seq, record) in records.into_iter().enumerate() {
        tsn_to_seq.insert(record.tsn, seq);
    }
    let taxa = itis::TaxonomicUnit::all()
        .filter(
            itis::TaxonomicUnit::fields()
                .name_usage()
                .eq("accepted")
                .and(
                    itis::TaxonomicUnit::fields()
                        .kingdom_id()
                        .eq(plant_kingdom.kingdom_id),
                ),
        )
        .order_by(itis::TaxonomicUnit::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    reporter.begin_step("Importing accepted taxa...", taxa.len());
    for chunk in &taxa
        .iter()
        .map(|theirs| {
            reporter.increment();
            let sequence = tsn_to_seq.get(&theirs.tsn).copied().unwrap();
            propagation_notebook::taxonomy::Taxon::create()
                .itis_id(theirs.tsn)
                .name1(&theirs.unit_name1)
                .name2(&theirs.unit_name2)
                .name3(&theirs.unit_name3)
                .complete_name(&theirs.complete_name)
                .rank(theirs.rank_id)
                .sequence(sequence as u64)
        })
        .chunks(CHUNK_SIZE)
    {
        let chunk: Vec<_> = chunk.into_iter().collect();
        let objs = toasty::batch(chunk).exec(ourtxn).await?;
        tsn_to_id.extend(objs.into_iter().map(|obj| (obj.itis_id, obj.id)));
    }
    reporter.finish_step();

    reporter.begin_step("Setting parent taxa...", taxa.len());
    for chunk in &taxa
        .into_iter()
        .map(|theirs| {
            reporter.increment();
            let errmsg = format!(
                "Failed to find parent of {} (id={}, parent={:?})",
                theirs.complete_name, theirs.tsn, theirs.parent_tsn
            );
            let our_parent_id = theirs
                .parent_tsn
                .filter(|id| id != &0)
                .map(|id| *tsn_to_id.get(&id).expect(&errmsg));
            propagation_notebook::taxonomy::Taxon::filter_by_itis_id(theirs.tsn)
                .update()
                .parent_id(our_parent_id)
        })
        .chunks(CHUNK_SIZE)
    {
        let chunk: Vec<_> = chunk.into_iter().collect();
        toasty::batch(chunk).exec(ourtxn).await?;
    }
    reporter.finish_step();

    let records = itis::Vernacular::all()
        .order_by(itis::Vernacular::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    reporter.begin_step("Importing vernacular names...", records.len());
    for chunk in &records
        .into_iter()
        .filter_map(|record| {
            reporter.increment();
            tsn_to_id.get(&record.tsn).map(|ourid| {
                propagation_notebook::taxonomy::VernacularName::create()
                    .name(&record.vernacular_name)
                    .taxon_id(ourid)
            })
        })
        .chunks(CHUNK_SIZE)
    {
        toasty::batch(chunk.into_iter().collect::<Vec<_>>())
            .exec(ourtxn)
            .await?;
    }
    reporter.finish_step();

    tracing::debug!("Loading synonym links...");
    let synonym_links: HashMap<u64, u64> = itis::SynonymLink::all()
        .exec(&mut itisdb)
        .await?
        .into_iter()
        .map(|link| (link.tsn, link.tsn_accepted))
        .collect();

    let records = itis::TaxonomicUnit::filter(
        itis::TaxonomicUnit::fields()
            .name_usage()
            .eq("not accepted")
            .and(
                itis::TaxonomicUnit::fields()
                    .kingdom_id()
                    .eq(plant_kingdom.kingdom_id),
            ),
    )
    .order_by(itis::TaxonomicUnit::fields().tsn().asc())
    .exec(&mut itisdb)
    .await?;

    reporter.begin_step("Importing synonyms...", records.len());
    for chunk in &records.into_iter().chunks(CHUNK_SIZE) {
        let creates: Vec<_> = chunk
            .filter_map(|theirs| {
                reporter.increment();
                let tsn_accepted = match synonym_links.get(&theirs.tsn) {
                    Some(id) => id,
                    None => {
                        tracing::warn!(tsn = theirs.tsn, "No synonym link found");
                        return None;
                    }
                };
                tsn_to_id.get(tsn_accepted).map(|ourid| {
                    propagation_notebook::taxonomy::Synonym::create()
                        .name1(&theirs.unit_name1)
                        .name2(&theirs.unit_name2)
                        .name3(&theirs.unit_name3)
                        .complete_name(&theirs.complete_name)
                        .taxon_id(ourid)
                })
            })
            .collect();
        if !creates.is_empty() {
            toasty::batch(creates).exec(ourtxn).await?;
        }
    }
    reporter.finish_step();
    Ok(())
}
