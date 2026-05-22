use std::collections::HashMap;

use anyhow::Context;
use indicatif::ProgressIterator;
use itertools::Itertools;
use propagation_notebook::taxonomy::Rank;
use toasty::{BelongsTo, HasMany};

const CHUNK_SIZE: usize = 500;
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
#[table = "hierarchy"]
pub struct Hierarchy {
    #[key]
    hierarchy_string: String,
    #[index]
    tsn: u64,
    level: u64,
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
    let itis_db_path = std::env::var("ITIS_DB_URI").with_context(|| " Please set the ITIS_DB_URI environment variable to the uri to connect to the ITIS database (download from https://www.itis.gov/downloads/index.html)")?;
    let our_db_path = &std::env::var("DB_URI").with_context(|| "Please set the DB_URI environment variable to the uri to connect to the propagation notebook database")?;

    let mut itisdb = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&itis_db_path)
        .await?;

    let mut ourdb = toasty::Db::builder()
        .models(propagation_notebook::models())
        .connect(our_db_path)
        .await?;
    let mut ourtxn = ourdb.transaction().await?;

    let mut tsn_to_id: HashMap<u64, u64> = HashMap::default();

    // Build a sequence map from the hierarchy table. hierarchy_string is the
    // primary key and encodes the ancestry path (e.g. "202422-846491-660046"),
    // so lexicographic order guarantees parent-before-child. phylo_sort_seq
    // has too many ties (89k taxa share seq=0) and causes rows to be skipped
    // at page boundaries when paginating.
    println!("Building hierarchy sequence...");
    let mut tsn_to_seq: HashMap<u64, _> = HashMap::default();
    let records = Hierarchy::all()
        .order_by(Hierarchy::fields().hierarchy_string().asc())
        .exec(&mut itisdb)
        .await?;
    for (seq, record) in records.into_iter().enumerate().progress() {
        tsn_to_seq.insert(record.tsn, seq);
    }

    println!("Importing accepted taxa...");
    let taxa = TaxonomicUnit::filter_by_name_usage("accepted")
        .order_by(TaxonomicUnit::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;

    for chunk in &taxa
        .into_iter()
        .progress()
        .map(|theirs| {
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
        let objs = toasty::batch(chunk).exec(&mut ourtxn).await?;
        tsn_to_id.extend(objs.into_iter().map(|obj| (obj.itis_id, obj.id)));
    }

    println!("Setting parent taxa...");
    let taxa = TaxonomicUnit::filter_by_name_usage("accepted")
        .order_by(TaxonomicUnit::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    for chunk in &taxa
        .into_iter()
        .progress()
        .map(|theirs| {
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
        toasty::batch(chunk).exec(&mut ourtxn).await?;
    }

    println!("Importing vernacular names...");
    let records = Vernacular::all()
        .order_by(Vernacular::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    for chunk in &records
        .into_iter()
        .progress()
        .filter_map(|record| {
            tsn_to_id.get(&record.tsn).map(|ourid| {
                propagation_notebook::taxonomy::VernacularName::create()
                    .name(&record.vernacular_name)
                    .taxon_id(ourid)
            })
        })
        .chunks(CHUNK_SIZE)
    {
        toasty::batch(chunk.into_iter().collect::<Vec<_>>())
            .exec(&mut ourtxn)
            .await?;
    }

    println!("Importing synonyms...");
    let records = TaxonomicUnit::filter_by_name_usage("not accepted")
        .order_by(TaxonomicUnit::fields().tsn().asc())
        .exec(&mut itisdb)
        .await?;
    for chunk in &records.into_iter().progress().chunks(CHUNK_SIZE) {
        let mut creates = Vec::new();

        for theirs in chunk {
            match SynonymLink::get_by_tsn(&mut itisdb, theirs.tsn).await {
                Ok(link) => {
                    let ourid = tsn_to_id
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
        if !creates.is_empty() {
            toasty::batch(creates).exec(&mut ourtxn).await?;
        }
    }

    ourtxn.commit().await?;

    Ok(())
}
