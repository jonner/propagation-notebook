use toasty::{BelongsTo, HasMany, HasOne};

use crate::{
    collection::{CleaningProcedure, CollectionData, Phenology},
    protocol::Protocol,
    region::RegionStatus,
};

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Embed)]
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

#[derive(Debug, toasty::Model)]
#[table = "taxa"]
pub struct Taxon {
    #[auto]
    #[key]
    pub id: u64,
    #[index]
    name1: String,
    #[index]
    name2: Option<String>,
    #[index]
    name3: Option<String>,
    #[index]
    complete_name: String,

    #[index]
    parent_id: Option<u64>,
    #[belongs_to(key=parent_id, references=id)]
    parent: BelongsTo<Option<Taxon>>,

    // #[index]
    rank: Rank,

    life_form: Option<LifeForm>,
    life_cycle: Option<LifeCycle>,

    #[has_many(pair=parent)]
    children: HasMany<Taxon>,
    #[has_many]
    vernaculars: HasMany<VernacularName>,
    #[has_many]
    synonyms: HasMany<Synonym>,
    #[has_many]
    region_statuses: HasMany<RegionStatus>,
    #[has_one]
    collection_data: HasOne<Option<CollectionData>>,
    #[has_one]
    cleaning_procedure: HasOne<Option<CleaningProcedure>>,
    #[has_many]
    phenologies: HasMany<Phenology>,
    #[has_many]
    protocols: HasMany<Protocol>,
}

#[derive(Debug, toasty::Model)]
pub struct VernacularName {
    #[auto]
    #[key]
    id: u64,

    #[index]
    taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    name: String,
}

#[derive(Debug, toasty::Model)]
pub struct Synonym {
    #[auto]
    #[key]
    id: u64,

    #[index]
    taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    taxon: BelongsTo<Taxon>,

    #[index]
    name1: String,
    #[index]
    name2: Option<String>,
    #[index]
    name3: Option<String>,
    #[index]
    pub complete_name: String,
    // is_accepted: bool,
}
