use libpropagation::{
    collecting::TaxonCleaningProcedure,
    dto::ObjectReference,
    region::{RegionalTaxonStatus, dto::RegionalTaxonStatusDetailsNoRegion},
    taxonomy::{
        Synonym, Taxon, TaxonIdentifier, TaxonNote, TaxonPropagationProcedure, TaxonomicAuthority,
        VernacularName, dto::TaxonDetails,
    },
};
use serde::Serialize;
use toasty::Db;

use crate::{
    cli::OutputFormat,
    util::IndicatifImportProgress,
    views::{
        JsonView, YamlView,
        taxa::{RegionalTaxaListView, TaxaListView, TaxaSearchResultsView, TaxonView},
    },
};

pub mod cleaning;
pub mod collecting;
pub mod note;
pub mod propagation;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Print a list of all taxa")]
    List {
        #[arg(short, long, help = "Show only taxa in the specified region")]
        region_id: Option<u64>,
        #[arg(long, help = "Show only taxa with custom data", hide = true)]
        has_data: bool,
    },
    #[command(about = "Show detailed information about a Taxon")]
    Show { name_or_id: TaxonIdentifier },
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
    #[command(about = "Import a new taxonomy for use with this tool")]
    Import {
        #[arg(help = "A URI to the external taxonomy database")]
        db_uri: String,
        #[arg(
            short,
            long,
            help = "The creator of the database",
            value_enum,
            default_value_t = TaxonomicAuthority::Itis
        )]
        authority: TaxonomicAuthority,
    },
    #[command(about = "Manage collecting information for a taxon")]
    Collecting {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: collecting::TaxonCollectingCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Cleaning {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: cleaning::TaxonCleaningCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Propagation {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: propagation::TaxonPropagationCommands,
    },
    #[command(about = "Manage notes for a taxon")]
    Notes {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: note::TaxonNoteCommands,
    },
}

#[derive(Debug, Serialize)]
pub struct TaxonSearchResult {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub common_names: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,
}

impl TaxonCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            TaxonCommands::Search { search_string } => {
                let wildcard = format!("%{search_string}%");

                if let Ok(found) = Taxon::filter(
                    Taxon::fields()
                        .complete_name()
                        .like(&wildcard)
                        .or(Taxon::fields()
                            .vernaculars()
                            .any(VernacularName::fields().name().like(&wildcard)))
                        .or(Taxon::fields()
                            .synonyms()
                            .any(Synonym::fields().complete_name().like(&wildcard))),
                )
                .order_by(Taxon::fields().sequence().asc())
                .include(Taxon::fields().vernaculars())
                .include(Taxon::fields().synonyms())
                .exec(db)
                .await
                {
                    let results = found
                        .into_iter()
                        .map(|t| TaxonSearchResult {
                            id: t.id,
                            name: t.complete_name,
                            common_names: t
                                .vernaculars
                                .get()
                                .iter()
                                .map(|v| v.name.clone())
                                .collect::<Vec<_>>(),
                            synonyms: t
                                .synonyms
                                .get()
                                .iter()
                                .filter_map(|s| {
                                    match s
                                        .complete_name
                                        .to_lowercase()
                                        .contains(&search_string.to_lowercase())
                                    {
                                        true => Some(s.complete_name.clone()),
                                        false => None,
                                    }
                                })
                                .collect::<Vec<_>>(),
                        })
                        .collect::<Vec<_>>();
                    let output = match format {
                        OutputFormat::Text => TaxaSearchResultsView::new(&results).render()?,
                        OutputFormat::Json => JsonView::new(&results).render()?,
                        OutputFormat::Yaml => YamlView::new(&results).render()?,
                    };

                    println!("{output}");
                }
            }
            TaxonCommands::Show { name_or_id } => {
                let taxon: TaxonDetails = Taxon::filter(Taxon::filter_by_identifier(name_or_id))
                    .include(Taxon::fields().parent())
                    .include(Taxon::fields().children())
                    .include(Taxon::fields().vernaculars())
                    .include(Taxon::fields().synonyms())
                    .include(Taxon::fields().regional_statuses().region())
                    .include(Taxon::fields().collecting_data())
                    .include(Taxon::fields().cleaning_procedures().procedure())
                    .include(Taxon::fields().propagation_procedures().propagation())
                    .include(Taxon::fields().notes())
                    .one()
                    .exec(db)
                    .await?
                    .into();

                let output = match format {
                    OutputFormat::Text => TaxonView::new(&taxon).render()?,
                    OutputFormat::Json => JsonView::new(&taxon).render()?,
                    OutputFormat::Yaml => YamlView::new(&taxon).render()?,
                };
                println!("{output}");
                println!();
            }
            TaxonCommands::List {
                region_id,
                has_data,
            } => match region_id {
                Some(id) => {
                    let region_id = *id;
                    let taxa = Taxon::filter(
                        Taxon::fields()
                            .regional_statuses()
                            .any(RegionalTaxonStatus::fields().region_id().eq(region_id)),
                    )
                    .include(Taxon::fields().regional_statuses())
                    .order_by(Taxon::fields().sequence().asc())
                    .exec(db)
                    .await?;
                    let statuses = RegionalTaxonStatusDetailsNoRegion::from_taxa(taxa, region_id);
                    let output = match format {
                        OutputFormat::Text => RegionalTaxaListView::new(&statuses).render()?,
                        OutputFormat::Json => JsonView::new(&statuses).render()?,
                        OutputFormat::Yaml => YamlView::new(&statuses).render()?,
                    };
                    println!("{output}");
                }
                None => {
                    let taxa: Vec<ObjectReference> = if *has_data {
                        Taxon::filter(
                            Taxon::fields()
                                .collecting_data()
                                .id()
                                .gt(0)
                                .or(Taxon::fields().regional_statuses().any(
                                    RegionalTaxonStatus::fields()
                                        .harvest_window()
                                        .start_doy()
                                        .is_some()
                                        .or(RegionalTaxonStatus::fields()
                                            .harvest_window()
                                            .end_doy()
                                            .is_some()),
                                ))
                                .or(Taxon::fields()
                                    .cleaning_procedures()
                                    .any(TaxonCleaningProcedure::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .propagation_procedures()
                                    .any(TaxonPropagationProcedure::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .notes()
                                    .any(TaxonNote::fields().taxon_id().gt(0))),
                        )
                        .order_by(Taxon::fields().sequence().asc())
                        .exec(db)
                        .await?
                    } else {
                        let taxa = Taxon::all()
                            .order_by(Taxon::fields().sequence().asc())
                            .exec(db)
                            .await?;
                        if taxa.is_empty() {
                            println!(
                                "The taxonomy has not been imported. Please download the ITIS taxonomy database from https://www.itis.gov/downloads/index.html and import it with `pn taxa import`"
                            )
                        }
                        taxa
                    }.into_iter().map(Into::into).collect();
                    let output = match format {
                        OutputFormat::Text => TaxaListView::new(&taxa).render()?,
                        OutputFormat::Json => JsonView::new(&taxa).render()?,
                        OutputFormat::Yaml => YamlView::new(&taxa).render()?,
                    };
                    println!("{output}",);
                }
            },
            TaxonCommands::Import { db_uri, authority } => {
                libpropagation::taxonomy::import(
                    db,
                    db_uri,
                    *authority,
                    &mut IndicatifImportProgress::default(),
                )
                .await?
            }
            TaxonCommands::Cleaning { taxon_id, command } => {
                command.run(db, *taxon_id, format).await?
            }
            TaxonCommands::Collecting { taxon_id, command } => {
                command.run(db, *taxon_id, format).await?
            }
            TaxonCommands::Propagation { taxon_id, command } => {
                command.run(db, *taxon_id, format).await?
            }
            TaxonCommands::Notes { taxon_id, command } => {
                command.run(db, *taxon_id, format).await?
            }
        }
        Ok(())
    }
}
