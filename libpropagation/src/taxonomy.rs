use std::convert::Infallible;

use serde::Serialize;
use toasty::Deferred;

pub mod dto;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum TaxonomicAuthority {
    Itis,
}

use crate::{
    ImportProgressReporter,
    citation::{Citation, TaxonPropagationProcedureCitation},
    collecting::{CollectingData, TaxonCleaningProcedure},
    dto::ObjectReference,
    error::ImportExportError,
    propagation::PropagationProcedure,
    region::RegionalTaxonStatus,
};

#[derive(Debug, Clone, Copy, toasty::Embed, strum::Display, PartialEq, Serialize)]
pub enum Rank {
    #[column(variant = 0)]
    Unknown,
    #[column(variant = 10)]
    Kingdom,
    #[column(variant = 20)]
    Subkingdom,
    #[column(variant = 25)]
    Infrakingdom,
    #[column(variant = 27)]
    Superdivision,
    #[column(variant = 30)]
    Division,
    #[column(variant = 40)]
    Subdivision,
    #[column(variant = 45)]
    Infradivision,
    #[column(variant = 50)]
    Superclass,
    #[column(variant = 60)]
    Class,
    #[column(variant = 70)]
    Subclass,
    #[column(variant = 80)]
    Infraclass,
    #[column(variant = 90)]
    Superorder,
    #[column(variant = 100)]
    Order,
    #[column(variant = 110)]
    Suborder,
    #[column(variant = 140)]
    Family,
    #[column(variant = 150)]
    Subfamily,
    #[column(variant = 160)]
    Tribe,
    #[column(variant = 170)]
    Subtribe,
    #[column(variant = 180)]
    Genus,
    #[column(variant = 190)]
    Subgenus,
    #[column(variant = 200)]
    Section,
    #[column(variant = 210)]
    Subsection,
    #[column(variant = 220)]
    Species,
    #[column(variant = 230)]
    Subspecies,
    #[column(variant = 240)]
    Variety,
    #[column(variant = 250)]
    Subvariety,
    #[column(variant = 260)]
    Form,
    #[column(variant = 270)]
    Subform,
}

impl std::str::FromStr for Rank {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().trim() {
            "subspecies" | "ssp" => Rank::Subspecies,
            "variety" | "var" => Rank::Variety,
            "species" => Rank::Species,
            "genus" => Rank::Genus,
            "family" => Rank::Family,
            _ => Rank::Unknown,
        })
    }
}

impl From<itis::Rank> for Rank {
    fn from(value: itis::Rank) -> Self {
        match value {
            itis::Rank::Unknown => Rank::Unknown,
            itis::Rank::Kingdom => Rank::Kingdom,
            itis::Rank::Subkingdom => Rank::Subkingdom,
            itis::Rank::Infrakingdom => Rank::Infrakingdom,
            itis::Rank::Superdivision => Rank::Superdivision,
            itis::Rank::Division => Rank::Division,
            itis::Rank::Subdivision => Rank::Subdivision,
            itis::Rank::Infradivision => Rank::Infradivision,
            itis::Rank::Superclass => Rank::Superclass,
            itis::Rank::Class => Rank::Class,
            itis::Rank::Subclass => Rank::Subclass,
            itis::Rank::Infraclass => Rank::Infraclass,
            itis::Rank::Superorder => Rank::Superorder,
            itis::Rank::Order => Rank::Order,
            itis::Rank::Suborder => Rank::Suborder,
            itis::Rank::Family => Rank::Family,
            itis::Rank::Subfamily => Rank::Subfamily,
            itis::Rank::Tribe => Rank::Tribe,
            itis::Rank::Subtribe => Rank::Subtribe,
            itis::Rank::Genus => Rank::Genus,
            itis::Rank::Subgenus => Rank::Subgenus,
            itis::Rank::Section => Rank::Section,
            itis::Rank::Subsection => Rank::Subsection,
            itis::Rank::Species => Rank::Species,
            itis::Rank::Subspecies => Rank::Subspecies,
            itis::Rank::Variety => Rank::Variety,
            itis::Rank::Subvariety => Rank::Subvariety,
            itis::Rank::Form => Rank::Form,
            itis::Rank::Subform => Rank::Subform,
        }
    }
}

#[derive(Debug, Clone, Copy, toasty::Embed, Serialize)]
pub enum LifeForm {
    #[column(variant = 1)]
    Tree,
    #[column(variant = 2)]
    Shrub,
    #[column(variant = 3)]
    Forb,
    #[column(variant = 4)]
    Graminoid,
    #[column(variant = 5)]
    Fern,
    #[column(variant = 99)]
    Other,
}

#[derive(Debug, Clone, Copy, toasty::Embed, Serialize)]
pub enum LifeCycle {
    #[column(variant = 1)]
    Annual,
    #[column(variant = 2)]
    Biennial,
    #[column(variant = 3)]
    Perennial,
    #[column(variant = 99)]
    Other,
}

impl From<&Taxon> for crate::dto::ObjectReference {
    fn from(taxon: &Taxon) -> Self {
        Self {
            id: taxon.id,
            name: Some(taxon.complete_name.clone()),
        }
    }
}

impl From<Taxon> for crate::dto::ObjectReference {
    fn from(taxon: Taxon) -> Self {
        Self {
            id: taxon.id,
            name: Some(taxon.complete_name),
        }
    }
}

#[derive(Debug, Clone, toasty::Model)]
#[table = "taxa"]
pub struct Taxon {
    #[auto]
    #[key]
    pub id: u64,
    #[index]
    pub itis_id: u64,
    #[index]
    pub inaturalist_id: Option<u64>,
    #[index]
    pub name1: String,
    #[index]
    pub name2: Option<String>,
    #[index]
    pub name3: Option<String>,
    #[index]
    pub complete_name: String,

    #[index]
    pub parent_id: Option<u64>,
    #[belongs_to(key=parent_id, references=id)]
    pub parent: Deferred<Option<Taxon>>,

    #[unique]
    pub sequence: u64,

    // #[index]
    pub rank: Rank,

    pub life_form: Option<LifeForm>,
    pub life_cycle: Option<LifeCycle>,

    #[has_many(pair=parent)]
    pub children: Deferred<Vec<Taxon>>,
    #[has_many]
    pub vernaculars: Deferred<Vec<VernacularName>>,
    #[has_many]
    pub synonyms: Deferred<Vec<Synonym>>,
    #[has_many]
    pub regional_statuses: Deferred<Vec<RegionalTaxonStatus>>,
    #[has_one]
    pub collecting_data: Deferred<Option<CollectingData>>,
    #[has_many]
    pub cleaning_procedures: Deferred<Vec<TaxonCleaningProcedure>>,
    #[has_many]
    pub propagation_procedures: Deferred<Vec<TaxonPropagationProcedure>>,
    #[has_many]
    pub notes: Deferred<Vec<TaxonNote>>,
}

#[derive(Debug, Clone)]
pub enum TaxonIdentifier {
    Id(u64),
    Name(String),
}

impl std::str::FromStr for TaxonIdentifier {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u64>() {
            Ok(id) => Ok(Self::Id(id)),
            Err(_) => Ok(Self::Name(s.to_string())),
        }
    }
}

impl Taxon {
    pub fn reference(&self) -> ObjectReference {
        self.into()
    }

    pub fn names(&self) -> String {
        [Some(&self.name1), self.name2.as_ref(), self.name3.as_ref()]
            .into_iter()
            .filter_map(|val| val.cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub async fn get_by_name_or_synonym(
        db: &mut dyn toasty::Executor,
        name: &str,
    ) -> Result<Taxon, toasty::Error> {
        match Taxon::get_by_complete_name(db, name).await {
            Ok(taxon) => Ok(taxon),
            Err(_e) => {
                Taxon::filter(
                    Taxon::fields()
                        .synonyms()
                        .any(Synonym::fields().complete_name().like(name)),
                )
                .one()
                .exec(db)
                .await
            }
        }
    }
    pub async fn get_by_complete_name_ignore_case(
        db: &mut dyn toasty::Executor,
        name: &str,
    ) -> Result<Taxon, toasty::Error> {
        Self::filter(Self::fields().complete_name().like(name))
            .one()
            .exec(db)
            .await
    }

    pub fn matches(&self, inat_taxon: &inaturalist::Taxon) -> bool {
        let rank = inat_taxon.rank.parse::<Rank>();
        rank == Ok(self.rank)
            && (self.complete_name.eq_ignore_ascii_case(&inat_taxon.name)
                || self.names().eq_ignore_ascii_case(&inat_taxon.name))
    }
}

#[derive(Debug, Clone, toasty::Model)]
pub struct VernacularName {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    pub name: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Synonym {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    #[index]
    pub name1: String,
    #[index]
    pub name2: Option<String>,
    #[index]
    pub name3: Option<String>,
    #[index]
    pub complete_name: String,
    // is_accepted: bool,
}

// TODO: if procedures become parametrized, we'd need to add the parameters to
// this model...
#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonPropagationProcedure {
    #[key]
    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    #[key]
    #[index]
    pub propagation_id: u64,
    #[belongs_to(key=propagation_id, references=id)]
    pub propagation: Deferred<PropagationProcedure>,

    pub confidence: Option<u8>,
    pub notes: Option<String>,
    #[has_many(pair=taxon_propagation)]
    pub citation_links: Deferred<Vec<TaxonPropagationProcedureCitation>>,
    #[has_many(via=citation_links.citation)]
    pub citations: Deferred<Vec<Citation>>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonNote {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    pub text: String,

    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub async fn import(
    db: &mut toasty::Db,
    taxonomy_db_uri: &str,
    authority: TaxonomicAuthority,
    reporter: &mut dyn ImportProgressReporter,
) -> Result<(), ImportExportError> {
    let ntaxon = Taxon::all().count().exec(db).await?;
    if ntaxon > 0 {
        return Err(ImportExportError::TaxonomyPresent(ntaxon));
    }
    let itisdb = toasty::Db::builder()
        .models(toasty::models!(itis::*))
        .connect(taxonomy_db_uri)
        .await?;

    let mut txn = db.transaction().await?;

    match authority {
        TaxonomicAuthority::Itis => itisdb::import(itisdb, &mut txn, reporter).await?,
    }

    txn.commit().await?;

    Ok(())
}

mod itisdb {
    use std::collections::HashMap;

    use itertools::Itertools;

    use super::*;

    const CHUNK_SIZE: usize = 500;

    pub async fn import(
        mut itisdb: toasty::Db,
        ourtxn: &mut toasty::Transaction<'_>,
        reporter: &mut dyn ImportProgressReporter,
    ) -> Result<(), ImportExportError> {
        // find plant kingdom
        let plant_kingdom = itis::Kingdom::get_by_kingdom_name(&mut itisdb, "Plantae").await?;
        let mut tsn_to_id: HashMap<u64, u64> = HashMap::default();
        let mut tsn_to_seq: HashMap<u64, _> = HashMap::default();
        let records = itis::Hierarchy::all()
            .order_by(itis::Hierarchy::fields().hierarchy_string().asc())
            .exec(&mut itisdb)
            .await?;
        reporter.begin_step("Building hierarchy sequence...", records.len());
        for (seq, record) in records.into_iter().enumerate() {
            reporter.increment();
            tsn_to_seq.insert(record.tsn, seq);
        }
        reporter.finish_step();

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
                let rank: Rank = theirs.rank_id.into();
                Taxon::create()
                    .itis_id(theirs.tsn)
                    .name1(&theirs.unit_name1)
                    .name2(&theirs.unit_name2)
                    .name3(&theirs.unit_name3)
                    .complete_name(&theirs.complete_name)
                    .rank(rank)
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
}
