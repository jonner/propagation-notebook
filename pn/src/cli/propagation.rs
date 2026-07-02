use std::path::PathBuf;

use libpropagation::propagation::{
    Protocol, ProtocolType,
    dto::{ProtocolCompact, ProtocolDetails},
};
use serde::Deserialize;
use toasty::Db;

use crate::{
    cli::OutputFormat,
    views::{
        JsonView, YamlView,
        propagation::{PropagationProtocolDetailView, PropagationProtocolListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum PropagationCommands {
    #[command(about = "List all seed propagation protocols")]
    List {
        #[arg(
            short,
            long,
            value_enum,
            help = "limit list to the selected protocol type"
        )]
        r#type: Option<ProtocolType>,
    },
    #[command(about = "Show a seed propagation protocol")]
    Show { id: u64 },
    #[command(about = "Add a seed propagation protocol")]
    Add {
        #[arg(help = "A short name for the protocol")]
        name: String,
        #[arg(short, long, value_enum)]
        r#type: ProtocolType,
        #[arg(long, help = "Instructions for this protocol")]
        instructions: String,
        #[arg(long, help = "Additional notes for this protocol")]
        notes: Option<String>,
        #[arg(long, help = "A citation for this protocol")]
        citation: Option<String>,
    },
    #[command(about = "Add a seed propagation protocol", group(clap::ArgGroup::new("modify_fields").args(["name", "type", "notes", "instructions", "citation"]).required(true).multiple(true)))]
    Modify {
        #[arg(help = "A protocol ID")]
        id: u64,
        #[arg(short, long, help = "A short name for the protocol")]
        name: Option<String>,
        #[arg(short, long, value_enum)]
        r#type: Option<ProtocolType>,
        #[arg(long, help = "Instructions for this protocol")]
        instructions: Option<String>,
        #[arg(long, help = "Additional notes for this protocol")]
        notes: Option<String>,
        #[arg(long, help = "A citation for this protocol")]
        citation: Option<String>,
    },
    #[command(about = "Remove a seed propagation protocol")]
    Remove {
        id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
    #[command(about = "Import seed propagation protocols from YAML")]
    Import { path: PathBuf },
}

impl PropagationCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            PropagationCommands::List { r#type } => {
                let mut query = Protocol::all();
                if let Some(t) = r#type {
                    query = query.filter(Protocol::fields().r#type().eq(t));
                }
                let protocols: Vec<ProtocolCompact> =
                    query.exec(db).await?.into_iter().map(Into::into).collect();
                let output = match format {
                    OutputFormat::Text => PropagationProtocolListView::new(&protocols).render()?,
                    OutputFormat::Json => JsonView::new(&protocols).render()?,
                    OutputFormat::Yaml => YamlView::new(&protocols).render()?,
                };
                println!("{output}");
            }
            PropagationCommands::Show { id } => {
                let protocol: ProtocolDetails = Protocol::filter_by_id(id)
                    .include(Protocol::fields().taxon_protocols().taxon())
                    .one()
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => PropagationProtocolDetailView::new(&protocol).render()?,
                    OutputFormat::Json => JsonView::new(&protocol).render()?,
                    OutputFormat::Yaml => YamlView::new(&protocol).render()?,
                };
                println!("{output}");
            }
            PropagationCommands::Add {
                name,
                r#type,
                instructions,
                notes,
                citation,
            } => {
                let item = Protocol::create()
                    .name(name)
                    .r#type(r#type)
                    .instructions(instructions)
                    .notes(notes)
                    .citation(citation)
                    .exec(db)
                    .await?;
                println!("Added protocol {}", item.id);
            }
            PropagationCommands::Modify {
                id,
                name,
                r#type,
                instructions,
                notes,
                citation,
            } => {
                let mut query = Protocol::update_by_id(id);
                if let Some(name) = name {
                    query = query.name(name);
                }
                if let Some(t) = r#type {
                    query = query.r#type(t);
                }
                if let Some(instructions) = instructions {
                    query = query.instructions(instructions);
                }
                if let Some(notes) = notes {
                    query = query.notes(notes);
                }
                if let Some(citation) = citation {
                    query = query.citation(citation);
                }
                query.exec(db).await?;
                println!("Updated protocol {id}");
            }
            PropagationCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    let protocol: ProtocolDetails = Protocol::filter_by_id(id)
                        .include(Protocol::fields().taxon_protocols().taxon())
                        .one()
                        .exec(db)
                        .await?
                        .into();

                    println!(
                        "{}",
                        PropagationProtocolDetailView::new(&protocol).render()?
                    );
                    inquire::Confirm::new(
                        "Are you sure you wish to remove this Propagation protocol?",
                    )
                    .with_default(false)
                    .with_help_message("It will remove all related steps")
                    .prompt()?
                } {
                    Protocol::delete_by_id(db, id).await?;
                    println!("Removed propagation protocol {id}");
                }
            }
            PropagationCommands::Import { path } => {
                #[derive(Debug, Deserialize)]
                struct ProtocolInfo {
                    pub name: String,
                    pub instructions: String,
                    pub notes: Option<String>,
                    pub r#type: ProtocolType,
                }
                let protocols: Vec<ProtocolInfo> =
                    serde_yaml::from_reader(std::fs::File::open(path)?)?;
                for p in protocols {
                    Protocol::create()
                        .name(p.name)
                        .instructions(p.instructions)
                        .notes(p.notes)
                        .r#type(p.r#type)
                        .exec(db)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
