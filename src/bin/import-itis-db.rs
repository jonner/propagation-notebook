use std::{
    collections::HashMap,
    io::{Write, stdout},
};

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

    let mut itis_to_ours: HashMap<u64, u64> = HashMap::default();
    let mut page = TaxonomicUnit::filter_by_name_usage("accepted")
        // need to sort by taxonomic sequence to guarantee that the parent will
        // be added to the database before the child that refers to it.
        .order_by(TaxonomicUnit::fields().phylo_sort_seq().asc())
        .paginate(100)
        .exec(&mut itisdb)
        .await?;

    println!("Importing accepted taxa...");
    loop {
        let mut creates = Vec::new();
        for theirs in page.iter() {
            let query = propagation_notebook::taxonomy::Taxon::create()
                .itis_id(theirs.tsn)
                .name1(&theirs.unit_name1)
                .name2(&theirs.unit_name2)
                .name3(&theirs.unit_name3)
                .complete_name(&theirs.complete_name)
                .rank(&theirs.rank_id);
            creates.push(query);
        }
        let objs = toasty::batch(creates).exec(&mut ourdb).await?;
        itis_to_ours.extend(objs.into_iter().map(|obj| (obj.itis_id, obj.id)));
        print!(".");
        stdout().flush().unwrap();

        match page.next(&mut itisdb).await? {
            Some(next) => page = next,
            None => break,
        }
    }
    println!();

    println!("Setting parent taxa...");
    let mut page = TaxonomicUnit::filter_by_name_usage("accepted")
        // need to sort by taxonomic sequence to guarantee that the parent will
        // be added to the database before the child that refers to it.
        .order_by(TaxonomicUnit::fields().phylo_sort_seq().asc())
        .paginate(100)
        .exec(&mut itisdb)
        .await?;
    loop {
        let mut updates = Vec::new();
        for theirs in page.iter() {
            let errmsg = format!(
                "Failed to find parent of {} (id={}, parent={:?})",
                theirs.complete_name, theirs.tsn, theirs.parent_tsn
            );
            let our_parent_id = theirs
                .parent_tsn
                .filter(|id| id != &0)
                .map(|id| *itis_to_ours.get(&id).expect(&errmsg));
            updates.push(
                propagation_notebook::taxonomy::Taxon::filter_by_itis_id(theirs.tsn)
                    .update()
                    .parent_id(our_parent_id),
            );
        }
        toasty::batch(updates).exec(&mut ourdb).await?;
        print!(".");
        stdout().flush().unwrap();

        match page.next(&mut itisdb).await? {
            Some(next) => page = next,
            None => break,
        }
    }
    println!();

    println!("Importing vernacular names...");
    let mut page = Vernacular::all()
        .order_by(Vernacular::fields().tsn().asc())
        .paginate(100)
        .exec(&mut itisdb)
        .await?;
    loop {
        let mut creates = Vec::new();
        for v in page.iter() {
            if let Some(ourid) = itis_to_ours.get(&v.tsn) {
                creates.push(
                    propagation_notebook::taxonomy::VernacularName::create()
                        .name(&v.vernacular_name)
                        .taxon_id(ourid),
                )
            }
        }
        toasty::batch(creates).exec(&mut ourdb).await?;
        print!(".");
        stdout().flush().unwrap();

        match page.next(&mut itisdb).await? {
            Some(next) => page = next,
            None => break,
        }
    }
    println!();

    println!("Importing synonyms...");
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
        print!(".");
        stdout().flush().unwrap();
        match page.next(&mut itisdb).await? {
            Some(next) => page = next,
            None => break,
        }
    }
    println!();

    Ok(())
}
