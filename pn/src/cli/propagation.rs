use std::path::PathBuf;

use libpropagation::propagation::{Protocol, ProtocolType};
use serde::Deserialize;
use toasty::Db;

use crate::style;

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
    },
    #[command(about = "Add a seed propagation protocol", group(clap::ArgGroup::new("modify_fields").args(["name", "type", "notes", "instructions"]).required(true).multiple(true)))]
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
    pub async fn run(&self, db: &mut Db) -> anyhow::Result<()> {
        match self {
            PropagationCommands::List { r#type } => {
                let mut query = Protocol::all();
                if let Some(t) = r#type {
                    query = query.filter(Protocol::fields().r#type().eq(t));
                }
                let protocols = query.exec(db).await?;
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name", "Type"]);
                for protocol in protocols {
                    tbuilder.push_record([
                        protocol.id.to_string(),
                        protocol.name,
                        protocol.r#type.to_string(),
                    ])
                }
                println!("{}", tbuilder.build().with(style::ListTable));
            }
            PropagationCommands::Show { id } => {
                let mut table = propagation_protocol_details_table(db, id).await?;
                println!("{}", table.with(style::DetailTable));
            }
            PropagationCommands::Add {
                name,
                r#type,
                instructions,
                notes,
            } => {
                let item = Protocol::create()
                    .name(name)
                    .r#type(r#type)
                    .instructions(instructions)
                    .notes(notes)
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
                query.exec(db).await?;
                println!("Updated protocol {id}");
            }
            PropagationCommands::Remove { id, assumeyes } => {
                if *assumeyes || {
                    let mut table = propagation_protocol_details_table(db, id).await?;
                    println!("{}", table.with(style::DetailTable));
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

async fn propagation_protocol_details_table(
    db: &mut Db,
    id: &u64,
) -> Result<tabled::Table, anyhow::Error> {
    let p = Protocol::filter_by_id(id)
        .include(Protocol::fields().taxon_protocols().taxon())
        .one()
        .exec(db)
        .await?;
    let mut tbuilder = tabled::builder::Builder::default();
    tbuilder.push_record(["ID", &p.id.to_string()]);
    tbuilder.push_record(["Name", &p.name]);
    tbuilder.push_record(["Type", &p.r#type.to_string()]);
    tbuilder.push_record(["Notes", &p.notes.unwrap_or_else(|| "-".into())]);
    tbuilder.push_record(["Instructions", &p.instructions]);
    let mut inner_table = tabled::builder::Builder::default();
    let tps = p.taxon_protocols.get();
    if !tps.is_empty() {
        inner_table.push_record(["ID", "Name"]);
        for tp in tps {
            let taxon = tp.taxon.get();
            inner_table.push_record([&taxon.id.to_string(), &taxon.complete_name]);
        }
    }

    tbuilder.push_record([
        "Taxa",
        &(inner_table.build().with(style::ListTable).to_string() + "\n"),
    ]);
    Ok(tbuilder.build())
}
