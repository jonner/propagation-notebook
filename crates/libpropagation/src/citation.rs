use toasty::{Db, Deferred};

use crate::{
    cleaning::CleaningProcedure,
    propagation::PropagationProcedure,
    taxonomy::{TaxonNote, TaxonPropagationProcedure},
};

pub mod dto;

#[derive(Debug, Clone, toasty::Model)]
pub struct Citation {
    #[key]
    #[auto]
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    #[default(Some(jiff::Zoned::now().date()))]
    pub access_date: Option<jiff::civil::Date>,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
    pub publication_year: Option<i16>,
    pub container_title: Option<String>,
    pub doi: Option<String>,

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

    pub fn format_cse(&self) -> String {
        [
            Some(format!("{}. {}.", self.author, self.title)),
            self.container_title
                .as_ref()
                .map(|container| format!(" {container}.")),
            self.publication_year.map(|y| format!(" {y}.")),
            self.access_date
                .map(|date| format!(" [accessed {}]", date.strftime("%Y %b %d"))),
            self.url.as_ref().map(|val| {
                format!(
                    " Available from: {}",
                    if let Ok(parsed) = url::Url::parse(val) {
                        if let Some(host) = parsed.host_str() {
                            format!("{}://{}", parsed.scheme(), host)
                        } else {
                            val.to_string()
                        }
                    } else {
                        val.to_string()
                    }
                )
            }),
        ]
        .into_iter()
        .flatten()
        .collect()
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
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
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
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
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
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
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
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}
