use toasty::Deferred;

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

#[derive(Debug, toasty::Model)]
#[table = "taxonomic_units"]
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
#[table = "synonym_links"]
pub struct SynonymLink {
    #[key]
    pub tsn: u64,
    #[key]
    pub tsn_accepted: u64,
}

#[derive(Debug, toasty::Model)]
#[table = "kingdoms"]
pub struct Kingdom {
    #[key]
    pub kingdom_id: u64,
    #[index]
    pub kingdom_name: String,
}

#[derive(Debug, toasty::Model)]
#[table = "vernaculars"]
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
