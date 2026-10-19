use libpropagation::{
    citation::{Citation, dto::CitationDetails},
    cleaning::{CollectingData, CollectingDataCategory},
    taxonomy::dto::CollectingDataDetails,
};

use toasty::Db;

use crate::{
    cli::{OutputFormat, citation::CitationCommands},
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        citation::{CitationDetailsView, CitationListView},
        collecting::{CollectingDataListView, CollectingDataView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCollectingCommands {
    #[command(about = "Show seed collecting information")]
    List,
    #[command(about = "Show seed collecting information")]
    Show {
        #[arg(help = "A collecting information ID")]
        id: u64,
    },
    #[command(about = "Add seed collecting information for a taxon")]
    Add {
        #[arg(short, long, value_enum, help = "The category of this collecting data")]
        category: CollectingDataCategory,
        #[arg(long, help = "The title of this collecting information")]
        title: String,
        #[arg(long, help = "The full text of this collecting information")]
        text: String,
    },
    #[command(about = "Modify seed collecting information for a taxon", group(clap::ArgGroup::new("modify_props").args(["category", "title", "text"]).required(true).multiple(true)), alias="edit")]
    Modify {
        #[arg(help = "A collecting data ID")]
        id: u64,
        #[arg(short, long, value_enum, help = "The category of this collecting data")]
        category: Option<CollectingDataCategory>,
        #[arg(long, help = "The title of this collecting information")]
        title: Option<String>,
        #[arg(long, help = "The full text of this collecting information")]
        text: Option<String>,
    },
    #[command(about = "Remove seed collecting information")]
    Remove {
        #[arg(help = "A collecting data ID")]
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Manage citations for collecting information")]
    Citations {
        #[arg(help = "A collecting information ID")]
        id: u64,
        #[command(subcommand)]
        command: CitationCommands,
    },
}

impl TaxonCollectingCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonCollectingCommands::List => {
                match CollectingData::filter_by_taxon_id(taxon_id)
                    .include(CollectingData::fields().taxon())
                    .exec(db)
                    .await
                {
                    Ok(data) => {
                        let data: Vec<CollectingDataDetails> =
                            data.into_iter().map(Into::into).collect();
                        let output = match format {
                            OutputFormat::Text => CollectingDataListView::new(&data).render()?,
                            OutputFormat::Json => JsonView::new(&data).render()?,
                            OutputFormat::Yaml => YamlView::new(&data).render()?,
                        };
                        println!("{output}");
                    }
                    Err(e) if e.is_record_not_found() => println!(
                        "Taxon {taxon_id} does not current have any collecting information defined"
                    ),
                    Err(e) => return Err(e.into()),
                }
            }
            TaxonCollectingCommands::Show { id } => {
                load_and_display_collecting_details(id, db, format).await?;
            }
            TaxonCollectingCommands::Remove { id, assumeyes } => {
                load_and_display_collecting_details(id, db, format).await?;
                if *assumeyes
                    || confirm("Are you sure you wish to remove this collecting data?")
                        .selected(false)
                        .run()?
                {
                    CollectingData::delete_by_id(db, id).await?;
                    println!("Removed collecting data {id}")
                }
            }
            TaxonCollectingCommands::Add {
                category,
                title,
                text,
            } => {
                let data: CollectingDataDetails = CollectingData::create()
                    .taxon_id(taxon_id)
                    .category(category)
                    .title(title)
                    .text(text)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => CollectingDataView::new(&data).render()?,
                    OutputFormat::Json => JsonView::new(&data).render()?,
                    OutputFormat::Yaml => YamlView::new(&data).render()?,
                };
                println!("{output}");
            }
            TaxonCollectingCommands::Modify {
                id,
                category,
                title,
                text,
            } => {
                let mut query = CollectingData::update_by_id(id);
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
                println!("Modified collection information {id}");
            }
            TaxonCollectingCommands::Citations { id, command } => {
                match command {
                    CitationCommands::List => {
                        let citations: Vec<CitationDetails> =
                            CollectingDataCitation::filter_by_collecting_id(id)
                                .include(CollectingDataCitation::fields().citation())
                                .exec(db)
                                .await?
                                .into_iter()
                                .map(|val| val.citation.get().into())
                                .collect();
                        let output = match format {
                            OutputFormat::Text => CitationListView::new(&citations).render()?,
                            OutputFormat::Json => JsonView::new(&citations).render()?,
                            OutputFormat::Yaml => YamlView::new(&citations).render()?,
                        };
                        println!("{output}");
                    }
                    CitationCommands::Show { id: citation_id } => {
                        load_and_display_citation_details(db, citation_id, id, format).await?
                    }
                    CitationCommands::Add {
                        title,
                        url,
                        author,
                        date,
                    } => {
                        let citation = Citation::create()
                            .title(title)
                            .url(url)
                            .author(author)
                            .date(date)
                            .exec(db)
                            .await?;
                        CollectingDataCitation::create()
                            .citation_id(citation.id)
                            .collecting_id(id)
                            .exec(db)
                            .await?;
                        load_and_display_collecting_details(id, db, format).await?;
                    }
                    CitationCommands::Remove {
                        citation_id,
                        assumeyes,
                    } => {
                        if *assumeyes || {
                            load_and_display_citation_details(
                                db,
                                citation_id,
                                id,
                                OutputFormat::Text,
                            )
                            .await?;
                            confirm("Do you want to remove this citation?")
                                .selected(false)
                                .run()?
                        } {
                            CollectingDataCitation::delete_by_citation_id(db, citation_id).await?;
                            let citation = Citation::filter_by_id(citation_id)
                                .include(Citation::fields().propagation_procedures())
                                .include(Citation::fields().taxon_propagation_procedures())
                                .include(Citation::fields().cleaning_procedures())
                                .one()
                                .exec(db)
                                .await?;
                            // if the citation is no longer rused, remove it from the database
                            if citation.propagation_procedures.get().is_empty()
                                && citation.taxon_propagation_procedures.get().is_empty()
                                && citation.cleaning_procedures.get().is_empty()
                            {
                                Citation::delete_by_id(db, citation_id).await?;
                            }
                            load_and_display_collecting_details(id, db, format).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

async fn load_and_display_collecting_details(
    id: &u64,
    db: &mut Db,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let data: CollectingDataDetails = CollectingData::get_by_id(db, id).await?.into();
    let output = match format {
        OutputFormat::Text => CollectingDataView::new(&data).render()?,
        OutputFormat::Json => JsonView::new(&data).render()?,
        OutputFormat::Yaml => YamlView::new(&data).render()?,
    };
    println!("{output}");
    Ok(())
}

async fn load_and_display_citation_details(
    db: &mut Db,
    citation_id: &u64,
    collecting_id: &u64,
    format: OutputFormat,
) -> Result<(), anyhow::Error> {
    let pc: CitationDetails =
        CollectingDataCitation::filter_by_citation_id_and_collecting_id(citation_id, collecting_id)
            .include(CollectingDataCitation::fields().citation())
            .one()
            .exec(db)
            .await?
            .citation
            .get()
            .into();
    let output = match format {
        OutputFormat::Text => CitationDetailsView::new(&pc).render()?,
        OutputFormat::Json => JsonView::new(&pc).render()?,
        OutputFormat::Yaml => YamlView::new(&pc).render()?,
    };
    println!("{output}");
    Ok(())
}
