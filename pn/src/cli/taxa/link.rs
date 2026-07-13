use crate::cli::OutputFormat;

use toasty::Db;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ExternalTaxonomy {
    Inaturalist,
}

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
        taxonomy: ExternalTaxonomy,
        _format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonLinkCommands::Clear => match taxonomy {
                ExternalTaxonomy::Inaturalist => {
                    Taxon::update_by_id(taxon_id)
                        .inaturalist_id(None)
                        .exec(db)
                        .await?
                }
            },
            TaxonLinkCommands::Set { id: external_id } => match taxonomy {
                ExternalTaxonomy::Inaturalist => {
                    Taxon::update_by_id(taxon_id)
                        .inaturalist_id(external_id)
                        .exec(db)
                        .await?
                }
            },
            TaxonLinkCommands::Search => {
                let taxon = Taxon::filter_by_id(taxon_id)
                    .include(Taxon::fields().vernaculars())
                    .one()
                    .exec(db)
                    .await?;
                match taxonomy {
                    ExternalTaxonomy::Inaturalist => {
                        let client = inaturalist::Client::new()?;
                        let mut options = vec![InaturalistSearchTerm::LatinName(taxon.names())];
                        for vn in taxon.vernaculars.get() {
                            options.push(InaturalistSearchTerm::CommonName(vn.name.clone()));
                        }
                        options.push(InaturalistSearchTerm::Custom);
                        if let Some(resp) = inquire::Select::new("Choose a search term:", options)
                            .prompt_skippable()?
                        {
                            let term = match resp {
                                InaturalistSearchTerm::LatinName(s)
                                | InaturalistSearchTerm::CommonName(s) => Some(s),
                                InaturalistSearchTerm::Custom => {
                                    inquire::Text::new("Custom search term:").prompt_skippable()?
                                }
                            };
                            if let Some(term) = term {
                                let taxa = client.taxon_search(&term).await?;
                                if taxa.is_empty() {
                                    anyhow::bail!("No results found");
                                } else {
                                    if let Some(chosen) = inquire::Select::new(
                                        "Choose from the following options:",
                                        taxa,
                                    )
                                    .prompt_skippable()?
                                    {
                                        Taxon::update_by_id(taxon_id)
                                            .inaturalist_id(chosen.id)
                                            .exec(db)
                                            .await?;
                                        println!("Updated iNaturalist ID")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
