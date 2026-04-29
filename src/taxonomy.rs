use toasty::{BelongsTo, HasMany, HasOne};

use crate::{
    collection::{CleaningProcedure, CollectionData},
    protocol::Protocol,
    region::RegionalTaxonStatus,
};

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
    pub parent: BelongsTo<Option<Taxon>>,

    // #[index]
    pub rank: Rank,

    pub life_form: Option<LifeForm>,
    pub life_cycle: Option<LifeCycle>,

    #[has_many(pair=parent)]
    pub children: HasMany<Taxon>,
    #[has_many]
    pub vernaculars: HasMany<VernacularName>,
    #[has_many]
    pub synonyms: HasMany<Synonym>,
    #[has_many]
    pub regional_statuses: HasMany<RegionalTaxonStatus>,
    #[has_one]
    pub collection_data: HasOne<Option<CollectionData>>,
    #[has_one]
    pub cleaning_procedure: HasOne<Option<CleaningProcedure>>,
    #[has_many]
    pub protocols: HasMany<Protocol>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct VernacularName {
    #[auto]
    #[key]
    pub id: u64,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: BelongsTo<Taxon>,

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
    pub taxon: BelongsTo<Taxon>,

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
