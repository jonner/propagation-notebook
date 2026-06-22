use toasty::Deferred;

use crate::taxonomy::Rank;

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
