use libpropagation::{
    citation::{
        Citation, TaxonNoteCitation,
        dto::{CitationCompact, CitationDetails},
    },
    taxonomy::{
        TaxonNote, TaxonNoteCategory,
        dto::{TaxonNoteDetails, TaxonNoteNoTaxon},
    },
};
use toasty::Db;

use crate::{
    cli::{OutputFormat, citation::CitationCommands},
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::{CitationDetailsView, CitationListView},
        taxa::{TaxonNoteDetailsView, TaxonNotesListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonNoteCommands {
    #[command(about = "List notes for the taxon", alias = "ls")]
    List,
    #[command(about = "Show a note for the taxon")]
    Show {
        #[arg(help = "A note ID")]
        note_id: u64,
    },
    #[command(about = "Add a new note to a taxon", alias = "new")]
    Add {
        #[arg(long, value_enum, help = "A note category")]
        category: Option<TaxonNoteCategory>,
        #[arg(long, help = "A note title")]
        title: String,
        #[arg(long, help = "A note body")]
        text: String,
    },
    #[command(about = "Modify a note for a taxon", group(clap::ArgGroup::new("modify_fields").args(["category", "title", "text"]).required(true).multiple(true)), alias = "edit")]
    Modify {
        #[arg(help = "A note ID assigned to this taxon")]
        note_id: u64,
        #[arg(long, value_enum, help = "A note category")]
        category: Option<TaxonNoteCategory>,
        #[arg(long, help = "A note title")]
        title: Option<String>,
        #[arg(long, help = "A note body")]
        text: Option<String>,
    },
    #[command(about = "Remove a note from the taxon")]
    Remove {
        #[arg(help = "A note ID")]
        note_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Manage citations for notes")]
    Citations {
        #[arg(help = "A taxon note ID")]
        id: u64,
        #[command(subcommand)]
        command: CitationCommands,
    },
}

impl TaxonNoteCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonNoteCommands::List => {
                let notes: Vec<TaxonNoteNoTaxon> = TaxonNote::filter_by_taxon_id(taxon_id)
                    .include(TaxonNote::fields().taxon())
                    .exec(db)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
                let output = match format {
                    OutputFormat::Text => TaxonNotesListView::new(&notes).render()?,
                    OutputFormat::Json => JsonView::new(&notes).render()?,
                    OutputFormat::Yaml => YamlView::new(&notes).render()?,
                };
                println!("{output}");
            }
            TaxonNoteCommands::Show { note_id } => {
                load_and_display_note_details(db, note_id, format).await?;
            }
            TaxonNoteCommands::Add {
                category,
                title,
                text,
            } => {
                let mut query = TaxonNote::create()
                    .taxon_id(taxon_id)
                    .title(title)
                    .text(text);
                if let Some(category) = category {
                    query = query.category(category);
                }
                let note: TaxonNoteDetails = query.exec(db).await?.into();
                let output = match format {
                    OutputFormat::Text => TaxonNoteDetailsView::new(&note).render()?,
                    OutputFormat::Json => JsonView::new(&note).render()?,
                    OutputFormat::Yaml => YamlView::new(&note).render()?,
                };
                println!("{output}");
            }
            TaxonNoteCommands::Modify {
                note_id,
                category,
                title,
                text,
            } => {
                let mut query = TaxonNote::update_by_id(note_id);
                if let Some(category) = category {
                    query = query.category(category);
                }
                if let Some(title) = title {
                    query = query.title(title);
                }
                if let Some(text) = text {
                    query = query.text(text);
                }
                query.exec(db).await?;
                println!("Updated note {note_id}")
            }
            TaxonNoteCommands::Remove { note_id, assumeyes } => {
                let note: TaxonNoteDetails = TaxonNote::get_by_id(db, note_id).await?.into();
                if *assumeyes || {
                    println!("{}", TaxonNoteDetailsView::new(&note).render()?);
                    confirm(&format!("Are you sure you wish to remove note {note_id}?"))
                        .selected(false)
                        .run()?
                } {
                    {
                        TaxonNote::delete_by_id(db, note_id).await?;
                        println!("Removed note {note_id}");
                    }
                }
            }
            TaxonNoteCommands::Citations {
                id: note_id,
                command,
            } => match command {
                CitationCommands::List => {
                    let note = TaxonNote::filter_by_id(note_id)
                        .include(TaxonNote::fields().citation_links().citation())
                        .one()
                        .exec(db)
                        .await?;
                    let citations: Vec<CitationCompact> = note
                        .citation_links
                        .get()
                        .iter()
                        .map(|cl| cl.citation.get().into())
                        .collect();
                    let output = match format {
                        OutputFormat::Text => CitationListView::new(&citations).render()?,
                        OutputFormat::Json => JsonView::new(&citations).render()?,
                        OutputFormat::Yaml => YamlView::new(&citations).render()?,
                    };
                    println!("{output}");
                }
                CitationCommands::Show { id } => {
                    load_and_display_citation_details(db, id, note_id, format).await?;
                }
                CitationCommands::Link { id } => {
                    TaxonNoteCitation::create()
                        .citation_id(id)
                        .note_id(note_id)
                        .exec(db)
                        .await?;
                    load_and_display_citation_details(db, id, note_id, format).await?;
                }
                CitationCommands::Add {
                    title,
                    url,
                    author,
                    year,
                    access_date,
                    container_title,
                    doi,
                } => {
                    TaxonNoteCitation::create()
                        .citation(
                            Citation::create()
                                .title(title)
                                .url(url)
                                .author(author)
                                .access_date(access_date)
                                .publication_year(year)
                                .container_title(container_title)
                                .doi(doi)
                                .exec(db)
                                .await?,
                        )
                        .note_id(note_id)
                        .exec(db)
                        .await?;
                }
                CitationCommands::Remove {
                    citation_id,
                    assumeyes,
                } => {
                    if *assumeyes || {
                        load_and_display_citation_details(
                            db,
                            citation_id,
                            note_id,
                            OutputFormat::Text,
                        )
                        .await?;
                        confirm("Do you want to remove this citation?")
                            .selected(false)
                            .run()?
                    } {
                        TaxonNoteCitation::delete_by_citation_id_and_note_id(
                            db,
                            citation_id,
                            note_id,
                        )
                        .await?;
                        Citation::delete_if_unused(db, citation_id).await?;
                        load_and_display_note_details(db, note_id, format).await?;
                    }
                }
            },
        }
        Ok(())
    }
}

async fn load_and_display_note_details(
    db: &mut Db,
    note_id: &u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let note: TaxonNoteDetails = TaxonNote::filter_by_id(note_id)
        .include(TaxonNote::fields().taxon())
        .one()
        .exec(db)
        .await?
        .into();
    let output = match format {
        OutputFormat::Text => TaxonNoteDetailsView::new(&note).render()?,
        OutputFormat::Json => JsonView::new(&note).render()?,
        OutputFormat::Yaml => YamlView::new(&note).render()?,
    };
    println!("{output}");
    Ok(())
}

async fn load_and_display_citation_details(
    db: &mut Db,
    id: &u64,
    note_id: &u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let tc = TaxonNoteCitation::filter_by_citation_id_and_note_id(id, note_id)
        .include(TaxonNoteCitation::fields().citation())
        .include(TaxonNoteCitation::fields().note())
        .one()
        .exec(db)
        .await?;
    let citation: CitationDetails = tc.citation.get().into();
    let output = match format {
        OutputFormat::Text => CitationDetailsView::new(&citation, false).render()?,
        OutputFormat::Json => JsonView::new(&citation).render()?,
        OutputFormat::Yaml => YamlView::new(&citation).render()?,
    };
    println!("{output}");
    Ok(())
}
