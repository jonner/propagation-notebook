use std::path::PathBuf;

use propagation_notebook::propagation::ProtocolType;

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
        #[arg(long, help = "Notes specific to this protocol")]
        notes: Option<String>,
    },
    #[command(about = "Add a seed propagation protocol", group(clap::ArgGroup::new("modify_fields").args(["name", "type", "notes"]).required(true).multiple(false)))]
    Modify {
        #[arg(help = "A protocol ID")]
        id: u64,
        #[arg(short, long, help = "A short name for the protocol")]
        name: Option<String>,
        #[arg(short, long, value_enum)]
        r#type: Option<ProtocolType>,
        #[arg(long, help = "Notes specific to this protocol")]
        notes: Option<String>,
    },
    #[command(about = "Remove a seed propagation protocol")]
    Remove { id: u64 },
    #[command(about = "Import seed propagation protocols from YAML")]
    Import { path: PathBuf },
}
