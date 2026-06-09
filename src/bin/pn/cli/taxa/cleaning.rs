use propagation_notebook::collecting::TaxonCleaningProcedure;

use toasty::Db;

use crate::style;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCleaningCommands {
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    List,
    #[command(about = "Show all seed cleaning procedures for a taxon")]
    Show {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
    },
    #[command(about = "Associate a taxon with a seed cleaning procedure")]
    Add {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "Taxon-specific notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Modify taxon-specific information seed cleaning information")]
    Modify {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(short, long, help = "Taxon-specific notes for this procedure")]
        notes: Option<String>,
    },
    #[command(about = "Remove a cleaning procedure from the specified taxon")]
    Remove {
        #[arg(short, long, help = "A cleaning procedure ID")]
        procedure_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonCleaningCommands {
    pub async fn run(&self, db: &mut Db, taxon_id: u64) -> anyhow::Result<()> {
        match self {
            TaxonCleaningCommands::List => {
                let procedures = TaxonCleaningProcedure::filter_by_taxon_id(taxon_id)
                    .exec(db)
                    .await?;

                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["Taxon ID", "Procedure ID", "Notes"]);
                for proc in procedures {
                    tbuilder.push_record([
                        proc.taxon_id.to_string(),
                        proc.procedure_id.to_string(),
                        proc.notes.unwrap_or_else(|| "-".into()),
                    ]);
                }
                println!("{}", tbuilder.build().with(style::BasicTable));
            }
            TaxonCleaningCommands::Show { procedure_id } => {
                let tcp = TaxonCleaningProcedure::filter_by_taxon_id_and_procedure_id(
                    taxon_id,
                    procedure_id,
                )
                .include(TaxonCleaningProcedure::fields().taxon())
                .include(TaxonCleaningProcedure::fields().procedure())
                .one()
                .exec(db)
                .await?;

                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record([
                    "Taxon",
                    &format!("{}: {}", tcp.taxon_id, tcp.taxon.get().complete_name),
                ]);
                tbuilder.push_record(["Procedure", &tcp.procedure_id.to_string()]);
                tbuilder.push_record(["Notes", &tcp.notes.unwrap_or_else(|| "-".into())]);
                println!("{}", tbuilder.build().with(style::DetailTable));
            }
            TaxonCleaningCommands::Add {
                procedure_id,
                notes,
            } => {
                TaxonCleaningProcedure::create()
                    .taxon_id(taxon_id)
                    .procedure_id(procedure_id)
                    .notes(notes)
                    .exec(db)
                    .await?;
                println!("Procedure {} assigned to taxon {}", taxon_id, procedure_id);
            }
            TaxonCleaningCommands::Modify {
                procedure_id,
                notes,
            } => {
                TaxonCleaningProcedure::update_by_taxon_id_and_procedure_id(taxon_id, procedure_id)
                    .notes(notes)
                    .exec(db)
                    .await?;
                println!("Procedure {} updated for taxon {}", procedure_id, taxon_id);
            }
            TaxonCleaningCommands::Remove {
                procedure_id,
                assumeyes,
            } => {
                if *assumeyes
                    || inquire::Confirm::new("Are you sure you wish to remove this procedure?")
                        .with_default(false)
                        .prompt()?
                {
                    TaxonCleaningProcedure::delete_by_taxon_id_and_procedure_id(
                        db,
                        taxon_id,
                        procedure_id,
                    )
                    .await?;
                    println!("Assignment removed");
                }
            }
        }
        Ok(())
    }
}
