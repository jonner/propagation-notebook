use toasty::Deferred;

use crate::{
    ImportProgressReporter,
    citation::{Citation, CleaningProcedureCitation},
    dto::ObjectReference,
    error::ImportExportError,
    taxonomy::Taxon,
};

pub mod dto;

#[derive(Debug, Clone, toasty::Model)]
pub struct CleaningProcedure {
    #[auto]
    #[key]
    pub id: u64,
    pub name: String,
    pub notes: Option<String>,
    pub instructions: String,
    #[has_many(pair=cleaning)]
    pub citation_links: Deferred<Vec<CleaningProcedureCitation>>,
    #[has_many(via=citation_links.citation)]
    pub citations: Deferred<Vec<Citation>>,

    #[index]
    pub taxon_id: u64,
    #[belongs_to(key=taxon_id, references=id)]
    pub taxon: Deferred<Taxon>,

    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

impl From<CleaningProcedure> for ObjectReference {
    fn from(value: CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: Some(value.name),
        }
    }
}

impl From<&CleaningProcedure> for ObjectReference {
    fn from(value: &CleaningProcedure) -> Self {
        Self {
            id: value.id,
            name: Some(value.name.clone()),
        }
    }
}

impl CleaningProcedure {
    pub fn reference(&self) -> ObjectReference {
        self.into()
    }

    pub async fn import<R>(
        db: &mut toasty::Db,
        reader: R,
        reporter: &mut dyn ImportProgressReporter,
    ) -> Result<Vec<Self>, ImportExportError>
    where
        R: std::io::Read,
    {
        let cleaning_defs: Vec<import::CleaningInput> = serde_yaml::from_reader(reader)?;

        let mut cleanings: Vec<import::Cleaning> = Vec::default();
        reporter.begin_step("Validating taxa...", cleaning_defs.len());
        for cleaning in cleaning_defs.into_iter() {
            reporter.increment();
            tracing::debug!(?cleaning);
            let t = match cleaning.taxon.parse::<u64>() {
                Ok(val) => Taxon::get_by_id(db, val).await?,
                Err(_) => Taxon::get_by_name_or_synonym(db, &cleaning.taxon)
                    .await
                    .inspect(|t| tracing::debug!(?t))
                    .map_err(|_e| ImportExportError::NoMatchingTaxon(cleaning.taxon.clone()))?,
            };
            cleanings.push(import::Cleaning {
                taxon_id: t.id,
                name: cleaning.name,
                instructions: cleaning.instructions,
                notes: cleaning.notes,
                citations: cleaning.citations,
            })
        }
        reporter.finish_step();

        let mut procs = Vec::default();
        let mut txn = db.transaction().await?;
        reporter.begin_step("Importing cleaning procedures...", cleanings.len());
        for cleaning in cleanings.into_iter() {
            reporter.increment();
            let mut citation_ids = Vec::default();
            for citation_def in cleaning.citations {
                let id = match citation_def {
                    import::CitationDef::Existing(def) => def.id,
                    import::CitationDef::New(def) => {
                        Citation::create()
                            .title(def.title)
                            .url(def.url)
                            .author(def.author)
                            .exec(&mut txn)
                            .await
                            .inspect(|c| tracing::debug!(?c))?
                            .id
                    }
                };
                citation_ids.push(id);
                tracing::debug!(id, "Using citation");
            }

            let proc = CleaningProcedure::create()
                .taxon_id(cleaning.taxon_id)
                .name(cleaning.name)
                .instructions(cleaning.instructions)
                .notes(cleaning.notes)
                .exec(&mut txn)
                .await?;
            tracing::debug!(?proc);
            for id in citation_ids {
                let cpc = CleaningProcedureCitation::create()
                    .cleaning_id(proc.id)
                    .citation_id(id)
                    .exec(&mut txn)
                    .await?;
                tracing::debug!(
                    cpc.cleaning_id,
                    cpc.citation_id,
                    "New CleaningProcedureCitation"
                );
            }
            procs.push(proc);
        }

        txn.commit().await?;
        reporter.finish_step();

        Ok(procs)
    }
}

mod import {
    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct NewCitationDef {
        pub url: String,
        pub title: String,
        pub author: Option<String>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ExistingCitationDef {
        pub id: u64,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(untagged)]
    pub enum CitationDef {
        Existing(ExistingCitationDef),
        New(NewCitationDef),
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CleaningInput {
        pub taxon: String,
        pub name: String,
        pub instructions: String,
        pub notes: Option<String>,
        pub citations: Vec<CitationDef>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Cleaning {
        pub taxon_id: u64,
        pub name: String,
        pub instructions: String,
        pub notes: Option<String>,
        pub citations: Vec<CitationDef>,
    }
}
