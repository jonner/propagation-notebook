use libpropagation::citation::{
    Citation,
    dto::{CitationCompact, CitationDetails},
};
use toasty::Db;

use crate::{
    cli::OutputFormat,
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::{CitationDetailsView, CitationListView},
    },
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum CitationCommands {
    #[command(about = "List all citations")]
    List,
    #[command(about = "Show citation details")]
    Show {
        #[arg(help = "A Citation ID")]
        id: u64,
    },
    #[command(about = "Link a citation to another object")]
    Link {
        #[arg(help = "A Citation ID")]
        id: u64,
    },
    #[command(about = "Add a new citation")]
    Add {
        #[arg(help = "Citation title")]
        title: String,
        #[arg(long, help = "A canonical URL for the citation")]
        url: Option<String>,
        #[arg(long, help = "The author being cited")]
        author: Option<String>,
        #[arg(long, help = "The date of the citation")]
        date: Option<jiff::civil::Date>,
    },
    #[command(about = "Remove a citation")]
    Remove {
        #[arg(help = "A citation ID")]
        citation_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl CitationCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            CitationCommands::List => {
                let citations: Vec<CitationCompact> = Citation::all()
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let output = match format {
                    OutputFormat::Text => CitationListView::new(&citations).render()?,
                    OutputFormat::Json => JsonView::new(&citations).render()?,
                    OutputFormat::Yaml => YamlView::new(&citations).render()?,
                };
                println!("{output}");
            }
            CitationCommands::Show { id } => {
                let citation: CitationDetails = Citation::filter_by_id(id)
                    .include(Citation::fields().cleaning_procedures())
                    .include(Citation::fields().propagation_procedures())
                    .include(Citation::fields().taxon_propagation_procedures())
                    .include(Citation::fields().taxon_notes())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => CitationDetailsView::new(&citation, true).render()?,
                    OutputFormat::Json => JsonView::new(&citation).render()?,
                    OutputFormat::Yaml => YamlView::new(&citation).render()?,
                };
                println!("{output}");
            }
            CitationCommands::Link { id: _id } => todo!(),
            CitationCommands::Add {
                title,
                url,
                author,
                date,
            } => {
                let citation: CitationDetails = toasty::create!(Citation {
                    title,
                    url,
                    author,
                    date: date.or(Some(jiff::Zoned::now().date())),
                })
                .exec(db)
                .await?
                .into();
                let output = match format {
                    OutputFormat::Text => CitationDetailsView::new(&citation, false).render()?,
                    OutputFormat::Json => JsonView::new(&citation).render()?,
                    OutputFormat::Yaml => YamlView::new(&citation).render()?,
                };
                println!("{output}");
            }
            CitationCommands::Remove {
                citation_id,
                assumeyes,
            } => {
                if *assumeyes || {
                    let citation: CitationDetails = Citation::filter_by_id(citation_id)
                        .include(Citation::fields().cleaning_procedures())
                        .include(Citation::fields().propagation_procedures())
                        .include(Citation::fields().taxon_propagation_procedures())
                        .include(Citation::fields().taxon_notes())
                        .one()
                        .exec(db)
                        .await?
                        .into();

                    println!("{}", CitationDetailsView::new(&citation, true).render()?);
                    confirm("Are you sure you wish to remove this Propagation procedure?")
                        .selected(false)
                        .run()?
                } {
                    Citation::delete_by_id(db, citation_id).await?;
                    println!("Removed citation {citation_id}");
                }
            }
        }
        Ok(())
    }
}
