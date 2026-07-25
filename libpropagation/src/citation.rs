use toasty::{Db, Deferred};

use crate::{
    collecting::CleaningProcedure,
    dto::ObjectReference,
    propagation::PropagationProcedure,
    taxonomy::{TaxonNote, TaxonPropagationProcedure},
};

pub mod dto {
    use serde::Serialize;
    use serde_with::skip_serializing_none;

    #[skip_serializing_none]
    #[derive(Serialize, Clone, Debug)]
    pub struct CitationDetails {
        pub id: u64,
        pub subject: String,
        pub url: Option<String>,
        pub author: Option<String>,
        pub date: Option<jiff::civil::Date>,
    }

    impl From<super::Citation> for CitationDetails {
        fn from(value: super::Citation) -> Self {
            Self {
                id: value.id,
                subject: value.title,
                url: value.url,
                author: value.author,
                date: value.date,
            }
        }
    }

    impl From<&super::Citation> for CitationDetails {
        fn from(value: &super::Citation) -> Self {
            Self {
                id: value.id,
                subject: value.title.clone(),
                url: value.url.clone(),
                author: value.author.clone(),
                date: value.date,
            }
        }
    }
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Citation {
    #[key]
    #[auto]
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub date: Option<jiff::civil::Date>,

    #[has_many]
    pub cleaning_procedures: Deferred<Vec<CleaningProcedureCitation>>,
    #[has_many]
    pub propagation_procedures: Deferred<Vec<PropagationProcedureCitation>>,
    #[has_many]
    pub taxon_propagation_procedures: Deferred<Vec<TaxonPropagationProcedureCitation>>,
    #[has_many]
    pub taxon_notes: Deferred<Vec<TaxonNoteCitation>>,
}

impl Citation {
    pub async fn delete_if_unused(db: &mut Db, citation_id: &u64) -> Result<(), toasty::Error> {
        let citation = Self::filter_by_id(citation_id)
            .include(Self::fields().propagation_procedures())
            .include(Self::fields().taxon_propagation_procedures())
            .include(Self::fields().cleaning_procedures())
            .include(Self::fields().taxon_notes())
            .one()
            .exec(db)
            .await?;
        if citation.propagation_procedures.get().is_empty()
            && citation.taxon_propagation_procedures.get().is_empty()
            && citation.cleaning_procedures.get().is_empty()
            && citation.taxon_notes.get().is_empty()
        {
            Citation::delete_by_id(db, citation_id).await?;
        }
        Ok(())
    }
}

impl From<&Citation> for ObjectReference {
    fn from(value: &Citation) -> Self {
        Self {
            id: value.id,
            name: Some(value.title.clone()),
        }
    }
}

#[derive(Debug, Clone, toasty::Model)]
pub struct CleaningProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    #[index]
    pub cleaning_id: u64,
    #[belongs_to(key=cleaning_id, references=id)]
    pub cleaning: Deferred<CleaningProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct PropagationProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    #[index]
    pub propagation_id: u64,
    #[belongs_to(key=propagation_id, references=id)]
    pub propagation: Deferred<PropagationProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
#[index(taxon_id, propagation_id)]
pub struct TaxonPropagationProcedureCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    pub propagation_id: u64,
    #[key]
    pub taxon_id: u64,
    #[belongs_to(key=[taxon_id, propagation_id], references=[taxon_id, propagation_id])]
    pub taxon_propagation: Deferred<TaxonPropagationProcedure>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct TaxonNoteCitation {
    #[key]
    #[index]
    pub citation_id: u64,
    #[belongs_to(key=citation_id, references=id)]
    pub citation: Deferred<Citation>,
    #[key]
    #[index]
    pub note_id: u64,
    #[belongs_to(key=note_id, references=id)]
    pub note: Deferred<TaxonNote>,
}
