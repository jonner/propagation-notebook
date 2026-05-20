use propagation_notebook::collecting::OperationType;

#[derive(Debug, clap::Subcommand)]
pub enum CleaningCommands {
    #[command(about = "List all seed cleaning procedures")]
    List,
    #[command(about = "Show detailed information about a seed cleaning procedure")]
    Show { id: u64 },
    #[command(about = "Add a new seed cleaning procedure")]
    Add {
        #[arg(short, long, help = "A name for the procedure")]
        name: String,
        #[arg(long, help = "General notes about the procedure")]
        notes: Option<String>,
    },
    #[command(about = "Show all steps for the specified seed cleaning procedure")]
    Steps { procedure_id: u64 },
    #[command(about = "Add a new step to the seed cleaning procedure")]
    AddStep {
        #[arg(short, long, help = "A procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "The order of this step within the procedure")]
        order: u64,
        #[arg(short = 't', long, help = "The type of this step")]
        step_type: OperationType,
        #[arg(short, long, help = "equipment used for this step")]
        equipment: Option<String>,
        #[arg(short, long, help = "A description of the step")]
        notes: String,
    },
    #[command(about = "Edit the details of an existing cleaning step", group(clap::ArgGroup::new("step_fields").args(["order", "step_type", "equipment", "notes"]).required(true).multiple(false)))]
    ModifyStep {
        id: u64,
        #[arg(short, long, help = "The order of this step within the procedure")]
        order: Option<u64>,
        #[arg(short = 't', long = "type", help = "The type of this step")]
        step_type: Option<OperationType>,
        #[arg(short, long, help = "equipment used for this step")]
        equipment: Option<String>,
        #[arg(short, long, help = "A description of the step")]
        notes: Option<String>,
    },
    #[command(about = "Associate a taxon with the seed cleaning procedure")]
    Assign {
        procedure_id: u64,
        #[arg(short, long, help = "A taxon ID")]
        taxon_id: u64,
        #[arg(
            short,
            long,
            help = "Custom notes for this taxon",
            conflicts_with = "remove"
        )]
        notes: Option<String>,
        #[arg(short, long, help = "Remove the assignment")]
        remove: bool,
    },
}
