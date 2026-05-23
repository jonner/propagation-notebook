use propagation_notebook::propagation::{LightRequirement, ProtocolStepType, ProtocolType};

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
    #[command(about = "Managing propagation protocol steps")]
    Steps {
        #[command(subcommand)]
        command: PropagationStepsCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum PropagationStepsCommands {
    #[command(about = "List steps for a seed propagation protocol")]
    List {
        #[arg(short, long, help = "A protocol ID")]
        protocol_id: u64,
    },
    #[command(about = "Add a step to a seed propagation protocol")]
    Add {
        #[arg(short, long, help = "A protocol ID")]
        protocol_id: u64,
        #[arg(help = "A short name for the step")]
        title: String,
        #[arg(
            short,
            long,
            value_enum,
            help = "The type of operation described in this step"
        )]
        r#type: ProtocolStepType,
        #[arg(
            short,
            long,
            help = "a value to determine the order of this step in the protocol"
        )]
        order: u64,
        #[arg(short, long, help = "A longer description of the step (if necessary)")]
        instructions: Option<String>,
        #[arg(
            short,
            long,
            value_name = "DAYS",
            help = "duration of the step in days"
        )]
        duration: Option<u64>,
        #[arg(long, help = "Minimum temperature for this step (in Celsius)")]
        min_temp: Option<f32>,
        #[arg(long, help = "Maximum temperature for this step (in Celsius)")]
        max_temp: Option<f32>,
        #[arg(short, long, value_enum, help = "Light requirements for this step")]
        light: Option<LightRequirement>,
        #[arg(long, help = "Moisture requirements for this step")]
        moisture: Option<String>,
        #[arg(long, help = "Additional materials needed for this step")]
        materials: Option<String>,
        #[arg(long, help = "Whether this step is optional or required")]
        is_optional: bool,
        #[arg(short, long, help = "Additional notes for this step")]
        notes: Option<String>,
    },
    #[command(about = "Modify a step to a seed propagation protocol", group(clap::ArgGroup::new("modify_fields").args(["title", "type", "order", "instructions", "duration","min_temp", "max_temp", "light", "moisture", "materials", "is_optional", "notes"]).required(true).multiple(false)))]
    Modify {
        #[arg(short, long, help = "A step ID")]
        id: u64,
        #[arg(help = "A short name for the step")]
        title: Option<String>,
        #[arg(
            short,
            long,
            value_enum,
            help = "The type of operation described in this step"
        )]
        r#type: Option<ProtocolStepType>,
        #[arg(
            short,
            long,
            help = "a value to determine the order of this step in the protocol"
        )]
        order: Option<u64>,
        #[arg(short, long, help = "A longer description of the step (if necessary)")]
        instructions: Option<String>,
        #[arg(
            short,
            long,
            value_name = "DAYS",
            help = "duration of the step in days"
        )]
        duration: Option<u64>,
        #[arg(long, help = "Minimum temperature for this step (in Celsius)")]
        min_temp: Option<f32>,
        #[arg(long, help = "Maximum temperature for this step (in Celsius)")]
        max_temp: Option<f32>,
        #[arg(short, long, value_enum, help = "Light requirements for this step")]
        light: Option<LightRequirement>,
        #[arg(long, help = "Moisture requirements for this step")]
        moisture: Option<String>,
        #[arg(long, help = "Additional materials needed for this step")]
        materials: Option<String>,
        #[arg(long, help = "Whether this step is optional or required")]
        is_optional: Option<bool>,
        #[arg(short, long, help = "Additional notes for this step")]
        notes: Option<String>,
    },
    #[command(about = "Remove a step for a seed propagation protocol")]
    Remove { id: u64 },
}
