use toasty::Deferred;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum TaxonomicAuthority {
    Itis,
}

use crate::{
    ImportProgressReporter,
    collecting::{CollectingData, HarvestEvent, TaxonCleaningProcedure},
    propagation::TaxonProtocol,
    region::RegionalTaxonStatus,
};

mod itis;

#[derive(Debug, Clone, Copy, toasty::Embed, strum::Display)]
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

#[derive(Debug, Clone, Copy, toasty::Embed)]
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

#[derive(Debug, Clone, Copy, toasty::Embed)]
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
    pub propagation_protocols: Deferred<Vec<TaxonProtocol>>,
    #[has_many]
    pub notes: Deferred<Vec<TaxonNote>>,
    #[has_many]
    pub harvest_events: Deferred<Vec<HarvestEvent>>,
}

impl Taxon {
    pub fn reference(&self) -> String {
        format!("{}: {}", self.id, self.complete_name)
    }

    pub fn names(&self) -> String {
        [Some(&self.name1), self.name2.as_ref(), self.name3.as_ref()]
            .into_iter()
            .filter_map(|val| val.cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub async fn find_by_name_or_synonym(
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

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Database already contains {0} taxa. Refusing to import.")]
    DatabaseContainsTaxa(u64),
    #[error(transparent)]
    ToastyError(#[from] toasty::Error),
}

pub async fn import(
    db: &mut toasty::Db,
    taxonomy_db_uri: &str,
    authority: TaxonomicAuthority,
    reporter: &mut dyn ImportProgressReporter,
) -> Result<(), ImportError> {
    let ntaxon = Taxon::all().count().exec(db).await?;
    if ntaxon > 0 {
        return Err(ImportError::DatabaseContainsTaxa(ntaxon));
    }
    let itisdb = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(taxonomy_db_uri)
        .await?;

    let mut txn = db.transaction().await?;

    match authority {
        TaxonomicAuthority::Itis => itis::import(itisdb, &mut txn, reporter).await?,
    }

    txn.commit().await?;

    Ok(())
}
