use std::collections::HashMap;

use anyhow::Context;
use propagation_notebook::taxonomy::Rank;
use toasty::{BelongsTo, HasMany};

#[derive(Debug, toasty::Model)]
pub struct TaxonomicUnit {
    #[key]
    tsn: u64,
    unit_ind1: Option<String>,
    unit_name1: String,
    unit_ind2: Option<String>,
    unit_name2: Option<String>,
    unit_ind3: Option<String>,
    unit_name3: Option<String>,
    unit_ind4: Option<String>,
    unit_name4: Option<String>,
    // unnamed_taxon_ind: char(1) DEFAULT NULL,
    #[index]
    name_usage: String,
    unaccept_reason: Option<String>,
    // credibility_rtng: varchar(40) NOT NULL,
    // completeness_rtng: char(10) DEFAULT NULL,
    // currency_rating: char(7) DEFAULT NULL,
    phylo_sort_seq: u64,
    // initial_time_stamp: datetime NOT NULL,
    #[index]
    parent_tsn: Option<u64>,
    #[belongs_to(key=parent_tsn, references=tsn)]
    parent: BelongsTo<TaxonomicUnit>,
    // taxon_author_id: int(11) DEFAULT NULL,
    // hybrid_author_id: int(11) DEFAULT NULL,
    kingdom_id: u64,
    rank_id: Rank,
    // update_date: date NOT NULL,
    // uncertain_prnt_ind: char(3) DEFAULT NULL,
    // n_usage: text,
    complete_name: String,

    #[has_many(pair=parent)]
    children: HasMany<TaxonomicUnit>,
    #[has_many(pair=taxon)]
    vernaculars: HasMany<Vernacular>,
}

#[derive(Debug, toasty::Model)]
pub struct SynonymLink {
    #[key]
    tsn: u64,
    #[key]
    tsn_accepted: u64,
}

#[derive(Debug, toasty::Model)]
pub struct Vernacular {
    #[key]
    vern_id: u64,
    #[index]
    tsn: u64,
    #[belongs_to(key=tsn, references= tsn)]
    taxon: BelongsTo<TaxonomicUnit>,
    language: String,
    vernacular_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let mut itisdb = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect("sqlite:///home/jjongsma/Projects/seedcollection/db/itis/seedcollection.sqlite")
        .await?;

    let mut ourdb = toasty::Db::builder()
        .models(propagation_notebook::models())
        .connect(
            &std::env::var("DB_URI").with_context(|| "Please set DB_URI environment variable")?,
        )
        .await?;

    let mut itis_to_ours: HashMap<u64, u64> = HashMap::default();
    let mut page = TaxonomicUnit::filter_by_name_usage("accepted")
        // need to sort by taxonomic sequence to guarantee that the parent will
        // be added to the database before the child that refers to it.
        .order_by(TaxonomicUnit::fields().phylo_sort_seq().asc())
        .paginate(100)
        .exec(&mut itisdb)
        .await?;

    loop {
        for theirs in page.iter() {
            let errmsg = format!(
                "Failed to find parent of {} (id={}, parent={:?})",
                theirs.complete_name, theirs.tsn, theirs.parent_tsn
            );
            let our_parent_id = theirs
                .parent_tsn
                .filter(|id| id != &0)
                .map(|id| *itis_to_ours.get(&id).expect(&errmsg));
            let ourtaxon = propagation_notebook::taxonomy::Taxon::create()
                .name1(&theirs.unit_name1)
                .name2(&theirs.unit_name2)
                .name3(&theirs.unit_name3)
                .complete_name(&theirs.complete_name)
                .rank(&theirs.rank_id)
                .parent_id(our_parent_id)
                .exec(&mut ourdb)
                .await?;
            println!("Inserted '{}'", theirs.complete_name);
            itis_to_ours.insert(theirs.tsn, ourtaxon.id);
            let vernaculars = theirs.vernaculars().exec(&mut itisdb).await?;
            if !vernaculars.is_empty() {
                println!("Inserting {} vernacular names...", vernaculars.len());
                toasty::batch(
                    vernaculars
                        .into_iter()
                        .map(|v| {
                            propagation_notebook::taxonomy::VernacularName::create()
                                .name(v.vernacular_name)
                                .taxon_id(ourtaxon.id)
                        })
                        .collect::<Vec<_>>(),
                )
                .exec(&mut ourdb)
                .await?;
            }
        }

        match page.next(&mut itisdb).await? {
            Some(next) => page = next,
            None => break,
        }
    }

    let mut page = TaxonomicUnit::filter_by_name_usage("not accepted")
        .order_by(TaxonomicUnit::fields().tsn().asc())
        .paginate(100)
        .exec(&mut itisdb)
        .await?;
    loop {
        let mut creates = Vec::new();
        for theirs in page.iter() {
            match SynonymLink::get_by_tsn(&mut itisdb, theirs.tsn).await {
                Ok(link) => {
                    let ourid = itis_to_ours
                        .get(&link.tsn_accepted)
                        .expect("Failed to find id of accepted taxon");
                    let synonym = propagation_notebook::taxonomy::Synonym::create()
                        .name1(&theirs.unit_name1)
                        .name2(&theirs.unit_name2)
                        .name3(&theirs.unit_name3)
                        .complete_name(&theirs.complete_name)
                        .taxon_id(ourid);
                    creates.push(synonym);
                }
                Err(e) => tracing::warn!(?e),
            };
        }

        toasty::batch(creates).exec(&mut ourdb).await?;
        match page.next(&mut itisdb).await? {
            Some(next) => page = next,
            None => break,
        }
    }

    Ok(())
}
