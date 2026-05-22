#[derive(Debug, clap::Subcommand)]
pub enum PropagationCommands {
    #[command(about = "List all seed propagation protocols")]
    List,
}
