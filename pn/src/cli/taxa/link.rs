use crate::{
    cli::OutputFormat,
    util::dialog::{input, select},
};

use anyhow::anyhow;
use demand::DemandOption;
use libpropagation::taxonomy::{Taxon, TaxonomicAuthority};
use toasty::Db;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonLinkCommands {
    #[command(about = "Clear the link to the external taxonomy")]
    Clear,
    #[command(about = "Set a new id for this taxon in the external taxonomy")]
    Set { id: u64 },
    #[command(about = "Search for the taxon in the external taxonomy")]
    Search,
}

pub(crate) enum InaturalistSearchTerm {
    LatinName(String),
    CommonName(String),
    Custom,
}

impl std::fmt::Display for InaturalistSearchTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InaturalistSearchTerm::LatinName(s) => write!(f, "Latin Name: '{s}'"),
            InaturalistSearchTerm::CommonName(s) => write!(f, "Common name: '{s}'"),
            InaturalistSearchTerm::Custom => write!(f, "Specify a custom search string"),
        }
    }
}

impl TaxonLinkCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        taxonomy: TaxonomicAuthority,
        _format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonLinkCommands::Clear => match taxonomy {
                TaxonomicAuthority::Inaturalist => {
                    Taxon::update_by_id(taxon_id)
                        .inaturalist_id(None)
                        .exec(db)
                        .await?
                }
                TaxonomicAuthority::Itis => {
                    return Err(anyhow!(
                        "ITIS is the internal taxonomic authority for this database"
                    ));
                }
            },
            TaxonLinkCommands::Set { id: external_id } => match taxonomy {
                TaxonomicAuthority::Inaturalist => {
                    Taxon::update_by_id(taxon_id)
                        .inaturalist_id(external_id)
                        .exec(db)
                        .await?
                }
                TaxonomicAuthority::Itis => {
                    return Err(anyhow!(
                        "ITIS is the internal taxonomic authority for this database"
                    ));
                }
            },
            TaxonLinkCommands::Search => {
                let taxon = Taxon::filter_by_id(taxon_id)
                    .include(Taxon::fields().vernaculars())
                    .one()
                    .exec(db)
                    .await?;
                match taxonomy {
                    TaxonomicAuthority::Inaturalist => {
                        let client = inaturalist::Client::new()?;
                        let mut options = vec![InaturalistSearchTerm::LatinName(taxon.names())];
                        for vn in taxon.vernaculars.get() {
                            options.push(InaturalistSearchTerm::CommonName(vn.name.clone()));
                        }
                        options.push(InaturalistSearchTerm::Custom);
                        if let Ok(resp) = select("Choose a search term:")
                            .filterable(true)
                            .options(options.into_iter().map(DemandOption::new).collect())
                            .run()
                        {
                            let term = match resp {
                                InaturalistSearchTerm::LatinName(s)
                                | InaturalistSearchTerm::CommonName(s) => Some(s.clone()),
                                InaturalistSearchTerm::Custom => {
                                    input("Custom search term:").run().ok()
                                }
                            };
                            if let Some(term) = term {
                                let taxa = client.taxon_search(&term).await?;
                                if taxa.is_empty() {
                                    anyhow::bail!("No results found");
                                } else {
                                    if let Ok(inat_taxon) =
                                        select("Choose from the following options:")
                                            .options(
                                                taxa.into_iter().map(DemandOption::new).collect(),
                                            )
                                            .run()
                                    {
                                        Taxon::update_by_id(taxon_id)
                                            .inaturalist_id(inat_taxon.id)
                                            .exec(db)
                                            .await?;
                                        println!("Updated iNaturalist ID")
                                    }
                                }
                            }
                        }
                    }
                    TaxonomicAuthority::Itis => {
                        return Err(anyhow!(
                            "ITIS is the internal taxonomic authority for this database"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
