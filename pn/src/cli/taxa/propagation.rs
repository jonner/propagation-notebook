use libpropagation::{
    citation::{
        Citation, TaxonPropagationProcedureCitation,
        dto::{CitationCompact, CitationDetails},
    },
    taxonomy::{
        TaxonPropagationProcedure,
        dto::{TaxonPropagationProcedureCompact, TaxonPropagationProcedureDetails},
    },
};

use toasty::Db;

use crate::{
    cli::{OutputFormat, citation::CitationCommands},
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::{CitationDetailsView, CitationListView},
        propagation::{TaxonPropagationProcedureDetailView, TaxonPropagationProcedureListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonPropagationCommands {
    #[command(about = "List seed propagation procedure for the taxon", alias = "ls")]
    List,
    #[command(about = "Show seed propagation information for the taxon")]
    Show {
        #[arg(help = "An ID of a propagation procedure ID assigned to this taxon")]
        propagation_id: u64,
    },
    #[command(
        about = "Assign a new seed propagation procedure to a taxon",
        alias = "new"
    )]
    Add {
        #[arg(help = "An ID of a propagation procedure ID")]
        propagation_id: u64,
        #[arg(
            long,
            help = "Confidence level in this propagation procedure (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation procedure"
        )]
        notes: Option<String>,
    },
    #[command(about = "Modify propagation information for a taxon", group(clap::ArgGroup::new("modify_props").args(["confidence", "notes"]).required(true).multiple(true)), alias="edit")]
    Modify {
        #[arg(help = "A propagation procedure ID assigned to this taxon")]
        propagation_id: u64,
        #[arg(
            long,
            help = "Confidence level in this propagation procedure (0-10)",
            value_parser = clap::value_parser!(u8).range(0..=10)
        )]
        confidence: Option<u8>,
        #[arg(
            short,
            long,
            help = "Taxon-specific notes for this propagation procedure"
        )]
        notes: Option<String>,
    },
    #[command(about = "Remove propagation information from the taxon")]
    Remove {
        #[arg(help = "A propagation procedure ID assigned to this taxon")]
        propagation_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Manage citations for taxon propagation procedures")]
    Citations {
        #[arg(help = "An ID of a propagation procedure")]
        propagation_id: u64,
        #[command(subcommand)]
        command: CitationCommands,
    },
}

impl TaxonPropagationCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
        TaxonPropagationCommands::List => {
            let tps: Vec<TaxonPropagationProcedureCompact> = TaxonPropagationProcedure::filter_by_taxon_id(taxon_id)
                .include(TaxonPropagationProcedure::fields().taxon())
                .include(TaxonPropagationProcedure::fields().propagation())
                .exec(db)
                .await?.into_iter().map(Into::into).collect();
            let output = match format {
                OutputFormat::Text => TaxonPropagationProcedureListView::new(&tps).render()?,
                OutputFormat::Json => JsonView::new(&tps).render()?,
                OutputFormat::Yaml => YamlView::new(&tps).render()?,
            };
            println!("{output}");
        }
        TaxonPropagationCommands::Show { propagation_id } => {
            load_and_display_taxon_propagation_details(db, taxon_id, propagation_id, format).await?;
        }
        TaxonPropagationCommands::Add {
            propagation_id,
            confidence,
            notes,
        } => {
            let tp: TaxonPropagationProcedureDetails = TaxonPropagationProcedure::create()
                .propagation_id(propagation_id)
                .taxon_id(taxon_id)
                .confidence(confidence)
                .notes(notes)
                .exec(db)
                .await?.into();
            let output = match format {
                OutputFormat::Text => TaxonPropagationProcedureDetailView::new(&tp).render()?,
                OutputFormat::Json => JsonView::new(&tp).render()?,
                OutputFormat::Yaml => YamlView::new(&tp).render()?,
            };
            println!("{output}");
        }
        TaxonPropagationCommands::Modify {
            propagation_id,
            confidence,
            notes,
        } => {
            let mut query =
                TaxonPropagationProcedure::update_by_taxon_id_and_propagation_id(taxon_id, propagation_id);
            if let Some(confidence) = confidence {
                query = query.confidence(confidence);
            }
            if let Some(notes) = notes {
                query = query.notes(notes);
            }
            query.exec(db).await?;
            load_and_display_taxon_propagation_details(db, taxon_id, propagation_id, format).await?;
        }
        TaxonPropagationCommands::Remove {
            propagation_id,
            assumeyes,
        } => {
            if *assumeyes
                        || confirm(
                            &format!("Are you sure you wish to remove this propagation procedure from taxon {taxon_id}?"),
                        )
                        .selected(false)
                        .run()?
                    {
                        TaxonPropagationProcedure::delete_by_taxon_id_and_propagation_id(
                            db,
                            taxon_id,
                            propagation_id,
                        )
                        .await?;
                        println!("Removed propagation procedure {propagation_id} for taxon {taxon_id}");
                    }
        }
        TaxonPropagationCommands::Citations { propagation_id, command } => {
            command.run_taxon_propagation(db, *propagation_id, taxon_id, format).await?;
            }
    }
        Ok(())
    }
}

async fn load_and_display_taxon_propagation_details(
    db: &mut Db,
    taxon_id: u64,
    propagation_id: &u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let tp: TaxonPropagationProcedureDetails =
        TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(taxon_id, propagation_id)
            .include(TaxonPropagationProcedure::fields().taxon())
            .include(TaxonPropagationProcedure::fields().propagation())
            .include(
                TaxonPropagationProcedure::fields()
                    .citation_links()
                    .citation(),
            )
            .one()
            .exec(db)
            .await?
            .into();
    let output = match format {
        OutputFormat::Text => TaxonPropagationProcedureDetailView::new(&tp).render()?,
        OutputFormat::Json => JsonView::new(&tp).render()?,
        OutputFormat::Yaml => YamlView::new(&tp).render()?,
    };
    println!("{output}");
    Ok(())
}

impl CitationCommands {
    async fn run_taxon_propagation(
        &self,
        db: &mut toasty::Db,
        propagation_id: u64,
        taxon_id: u64,
        format: super::OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            CitationCommands::List => {
                let procedure_citations: Vec<CitationCompact> =
                    TaxonPropagationProcedure::filter_by_taxon_id_and_propagation_id(
                        taxon_id,
                        propagation_id,
                    )
                    .include(
                        TaxonPropagationProcedure::fields()
                            .citation_links()
                            .citation(),
                    )
                    .one()
                    .exec(db)
                    .await?
                    .citation_links
                    .get()
                    .iter()
                    .map(|link| link.citation.get().into())
                    .collect();
                let output = match format {
                    OutputFormat::Text => CitationListView::new(&procedure_citations).render()?,
                    OutputFormat::Json => JsonView::new(&procedure_citations).render()?,
                    OutputFormat::Yaml => YamlView::new(&procedure_citations).render()?,
                };
                println!("{output}");
            }
            CitationCommands::Show { id } => {
                load_and_display_citation_details(db, taxon_id, propagation_id, id, format).await?
            }
            CitationCommands::Link { id } => {
                TaxonPropagationProcedureCitation::create()
                    .citation_id(id)
                    .taxon_id(taxon_id)
                    .propagation_id(propagation_id)
                    .exec(db)
                    .await?;
                load_and_display_citation_details(db, taxon_id, propagation_id, id, format).await?
            }
            CitationCommands::Add {
                title,
                url,
                author,
                date,
            } => {
                TaxonPropagationProcedureCitation::create()
                    .citation(
                        Citation::create()
                            .title(title)
                            .url(url)
                            .author(author)
                            .date(date)
                            .exec(db)
                            .await?,
                    )
                    .taxon_id(taxon_id)
                    .propagation_id(propagation_id)
                    .exec(db)
                    .await?;
                load_and_display_taxon_propagation_details(db, taxon_id, &propagation_id, format)
                    .await?;
            }
            CitationCommands::Remove {
                citation_id,
                assumeyes,
            } => {
                if *assumeyes || {
                    load_and_display_citation_details(
                        db,
                        taxon_id,
                        propagation_id,
                        citation_id,
                        OutputFormat::Text,
                    )
                    .await?;
                    confirm("Do you want to remove this citation?")
                        .selected(false)
                        .run()?
                } {
                    TaxonPropagationProcedureCitation::delete_by_citation_id_and_propagation_id_and_taxon_id(
                        db,
                        citation_id,
                        propagation_id,
                        taxon_id,
                    )
                    .await?;
                    Citation::delete_if_unused(db, citation_id).await?;
                    load_and_display_taxon_propagation_details(
                        db,
                        taxon_id,
                        &propagation_id,
                        format,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_citation_details(
    db: &mut Db,
    taxon_id: u64,
    propagation_id: u64,
    citation_id: &u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let pc: CitationDetails =
        TaxonPropagationProcedureCitation::filter_by_citation_id_and_propagation_id_and_taxon_id(
            citation_id,
            propagation_id,
            taxon_id,
        )
        .include(TaxonPropagationProcedureCitation::fields().citation())
        .one()
        .exec(db)
        .await?
        .citation
        .get()
        .into();
    let output = match format {
        OutputFormat::Text => CitationDetailsView::new(&pc, false).render()?,
        OutputFormat::Json => JsonView::new(&pc).render()?,
        OutputFormat::Yaml => YamlView::new(&pc).render()?,
    };
    println!("{output}");
    Ok(())
}
