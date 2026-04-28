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
    name1: String,
    name2: Option<String>,
    name3: Option<String>,
    complete_name: String,
    parent_id: Option<u64>,
    rank: Rank,
    life_form: Option<LifeForm>,
    life_cycle: Option<LifeCycle>,
}

#[derive(Debug, toasty::Model)]
pub struct VernacularNames {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    name: String,
}

#[derive(Debug, toasty::Model)]
pub struct Synonym {
    #[auto]
    #[key]
    id: u64,
    taxon_id: u64,
    name1: String,
    name2: Option<String>,
    name3: Option<String>,
    pub complete_name: String,
    // is_accepted: bool,
}
