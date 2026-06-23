use std::collections::HashMap;

use itertools::Itertools;
use toasty::Deferred;

use crate::{
    ImportProgressReporter,
    taxonomy::{ImportError, Rank, Synonym, Taxon, VernacularName},
};
const CHUNK_SIZE: usize = 500;

#[derive(Debug, toasty::Model)]
pub struct TaxonomicUnit {
    #[key]
    pub tsn: u64,
    pub unit_ind1: Option<String>,
    pub unit_name1: String,
    pub unit_ind2: Option<String>,
    pub unit_name2: Option<String>,
    pub unit_ind3: Option<String>,
    pub unit_name3: Option<String>,
    pub unit_ind4: Option<String>,
    pub unit_name4: Option<String>,
    // unnamed_taxon_ind: char(1) DEFAULT NULL,
    #[index]
    pub name_usage: String,
    pub unaccept_reason: Option<String>,
    // credibility_rtng: varchar(40) NOT NULL,
    // completeness_rtng: char(10) DEFAULT NULL,
    // currency_rating: char(7) DEFAULT NULL,
    pub phylo_sort_seq: u64,
    // initial_time_stamp: datetime NOT NULL,
    #[index]
    pub parent_tsn: Option<u64>,
    #[belongs_to(key=parent_tsn, references=tsn)]
    pub parent: Deferred<TaxonomicUnit>,
    // taxon_author_id: int(11) DEFAULT NULL,
    // hybrid_author_id: int(11) DEFAULT NULL,
    pub kingdom_id: u64,
    pub rank_id: Rank,
    // update_date: date NOT NULL,
    // uncertain_prnt_ind: char(3) DEFAULT NULL,
    // n_usage: text,
    pub complete_name: String,

    #[has_many(pair=parent)]
    pub children: Deferred<Vec<TaxonomicUnit>>,
    #[has_many(pair=taxon)]
    pub vernaculars: Deferred<Vec<Vernacular>>,
}

#[derive(Debug, toasty::Model)]
#[table = "hierarchy"]
pub struct Hierarchy {
    #[key]
    pub hierarchy_string: String,
    #[index]
    pub tsn: u64,
    pub level: u64,
}

#[derive(Debug, toasty::Model)]
pub struct SynonymLink {
    #[key]
    pub tsn: u64,
    #[key]
    pub tsn_accepted: u64,
}

#[derive(Debug, toasty::Model)]
pub struct Kingdom {
    #[key]
    pub kingdom_id: u64,
    #[index]
    pub kingdom_name: String,
}

#[derive(Debug, toasty::Model)]
pub struct Vernacular {
    #[key]
    pub vern_id: u64,
    #[index]
    pub tsn: u64,
    #[belongs_to(key=tsn, references= tsn)]
    pub taxon: Deferred<TaxonomicUnit>,
    pub language: String,
    pub vernacular_name: String,
}

pub async fn import(
    mut itisdb: toasty::Db,
    ourtxn: &mut toasty::Transaction<'_>,
    reporter: &mut dyn ImportProgressReporter,
) -> Result<(), ImportError> {
    // find plant kingdom
    let plant_kingdom = Kingdom::get_by_kingdom_name(&mut itisdb, "Plantae").await?;
    let mut tsn_to_id: HashMap<u64, u64> = HashMap::default();
    let mut tsn_to_seq: HashMap<u64, _> = HashMap::default();
    let records = Hierarchy::all()
        .order_by(Hierarchy::fields().hierarchy_string().asc())
        .exec(&mut itisdb)
        .await?;
    reporter.begin_step("Building hierarchy sequence...", records.len());
    for (seq, record) in records.into_iter().enumerate() {
        reporter.increment();
        tsn_to_seq.insert(record.tsn, seq);
    }
    reporter.finish_step();

    let taxa = TaxonomicUnit::all()
        .filter(
            TaxonomicUnit::fields().name_usage().eq("accepted").and(
                TaxonomicUnit::fields()
                    .kingdom_id()
                    .eq(plant_kingdom.kingdom_id),
            ),
        )
        .order_by(TaxonomicUnit::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    reporter.begin_step("Importing accepted taxa...", taxa.len());
    for chunk in &taxa
        .iter()
        .map(|theirs| {
            reporter.increment();
            let sequence = tsn_to_seq.get(&theirs.tsn).copied().unwrap();
            Taxon::create()
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
            Taxon::filter_by_itis_id(theirs.tsn)
                .update()
                .parent_id(our_parent_id)
        })
        .chunks(CHUNK_SIZE)
    {
        let chunk: Vec<_> = chunk.into_iter().collect();
        toasty::batch(chunk).exec(ourtxn).await?;
    }
    reporter.finish_step();

    let records = Vernacular::all()
        .order_by(Vernacular::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    reporter.begin_step("Importing vernacular names...", records.len());
    for chunk in &records
        .into_iter()
        .filter_map(|record| {
            reporter.increment();
            tsn_to_id.get(&record.tsn).map(|ourid| {
                VernacularName::create()
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
    let synonym_links: HashMap<u64, u64> = SynonymLink::all()
        .exec(&mut itisdb)
        .await?
        .into_iter()
        .map(|link| (link.tsn, link.tsn_accepted))
        .collect();

    let records = TaxonomicUnit::filter(
        TaxonomicUnit::fields().name_usage().eq("not accepted").and(
            TaxonomicUnit::fields()
                .kingdom_id()
                .eq(plant_kingdom.kingdom_id),
        ),
    )
    .order_by(TaxonomicUnit::fields().tsn().asc())
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
                    Synonym::create()
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
