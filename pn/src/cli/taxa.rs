use libpropagation::{
    collecting::TaxonCleaningProcedure,
    propagation::TaxonProtocol,
    region::RegionalTaxonStatus,
    taxonomy::{Synonym, Taxon, TaxonNote, TaxonomicAuthority, VernacularName},
};
use toasty::Db;

use crate::{
    cli::{OutputFormat, print_regional_taxa_table},
    style,
    util::IndicatifImportProgress,
    views::{JsonView, YamlView, taxa::TaxonView},
};

pub mod cleaning;
pub mod collecting;
pub mod note;
pub mod propagation;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCommands {
    #[command(about = "Print a list of all taxa")]
    List {
        #[arg(short, long, help = "Show only taxa in the specified region")]
        region_id: Option<u64>,
        #[arg(long, help = "Show only taxa with custom data", hide = true)]
        has_data: bool,
    },
    #[command(about = "Show detailed information about a Taxon")]
    Show { id: u64 },
    #[command(about = "Search for a taxon")]
    Search { search_string: String },
    #[command(about = "Import a new taxonomy for use with this tool")]
    Import {
        #[arg(help = "A URI to the external taxonomy database")]
        db_uri: String,
        #[arg(
            short,
            long,
            help = "The creator of the database",
            value_enum,
            default_value_t = TaxonomicAuthority::Itis
        )]
        authority: TaxonomicAuthority,
    },
    #[command(about = "Manage collecting information for a taxon")]
    Collecting {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: collecting::TaxonCollectingCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Cleaning {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: cleaning::TaxonCleaningCommands,
    },
    #[command(about = "Manage cleaning information for a taxon")]
    Propagation {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: propagation::TaxonPropagationCommands,
    },
    #[command(about = "Manage notes for a taxon")]
    Notes {
        #[arg(short, long, help = "A Taxon ID")]
        taxon_id: u64,
        #[command(subcommand)]
        command: note::TaxonNoteCommands,
    },
}

impl TaxonCommands {
    pub async fn run(&self, db: &mut Db, format: OutputFormat) -> anyhow::Result<()> {
        match self {
            TaxonCommands::Search { search_string } => {
                let wildcard = format!("%{search_string}%");
                let mut tbuilder = tabled::builder::Builder::default();
                tbuilder.push_record(["ID", "Name", "Common Names", "Synonym"]);

                if let Ok(found) = Taxon::filter(
                    Taxon::fields()
                        .complete_name()
                        .like(&wildcard)
                        .or(Taxon::fields()
                            .vernaculars()
                            .any(VernacularName::fields().name().like(&wildcard)))
                        .or(Taxon::fields()
                            .synonyms()
                            .any(Synonym::fields().complete_name().like(&wildcard))),
                )
                .order_by(Taxon::fields().sequence().asc())
                .include(Taxon::fields().vernaculars())
                .include(Taxon::fields().synonyms())
                .exec(db)
                .await
                {
                    for t in found {
                        tbuilder.push_record([
                            t.id.to_string(),
                            t.complete_name,
                            t.vernaculars
                                .get()
                                .iter()
                                .map(|v| v.name.as_str())
                                .collect::<Vec<_>>()
                                .join("\n"),
                            t.synonyms
                                .get()
                                .iter()
                                .filter_map(|s| {
                                    match s
                                        .complete_name
                                        .to_lowercase()
                                        .contains(&search_string.to_lowercase())
                                    {
                                        true => Some(s.complete_name.as_str()),
                                        false => None,
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ]);
                    }
                }

                println!("{}", tbuilder.build().with(style::ListTable));
            }
            TaxonCommands::Show { id } => {
                let taxon = Taxon::filter_by_id(id)
                    .include(Taxon::fields().parent())
                    .include(Taxon::fields().children())
                    .include(Taxon::fields().vernaculars())
                    .include(Taxon::fields().synonyms())
                    .include(Taxon::fields().regional_statuses().region())
                    .include(Taxon::fields().collecting_data())
                    .include(Taxon::fields().cleaning_procedures().procedure())
                    .include(Taxon::fields().propagation_protocols().protocol())
                    .include(Taxon::fields().notes())
                    .one()
                    .exec(db)
                    .await?;

                let output = match format {
                    OutputFormat::Table => TaxonView::new(&taxon).render()?,
                    OutputFormat::Json => JsonView::new(&taxon).render()?,
                    OutputFormat::Yaml => YamlView::new(&taxon).render()?,
                };
                println!("{output}");
                println!();
            }
            TaxonCommands::List {
                region_id,
                has_data,
            } => match region_id {
                Some(id) => {
                    let region_id = *id;
                    let regional_statuses = RegionalTaxonStatus::filter(
                        RegionalTaxonStatus::fields().region_id().eq(region_id),
                    )
                    // FIXME: We want to order by a taxon sequence, but
                    // toasty doesn't yet support ordering by data in a relation
                    .exec(db)
                    .await?;
                    print_regional_taxa_table(db, regional_statuses).await?;
                }
                None => {
                    let taxa = if *has_data {
                        Taxon::filter(
                            Taxon::fields()
                                .collecting_data()
                                .id()
                                .gt(0)
                                .or(Taxon::fields().regional_statuses().any(
                                    RegionalTaxonStatus::fields()
                                        .harvest_window()
                                        .start_doy()
                                        .is_some()
                                        .or(RegionalTaxonStatus::fields()
                                            .harvest_window()
                                            .end_doy()
                                            .is_some()),
                                ))
                                .or(Taxon::fields()
                                    .cleaning_procedures()
                                    .any(TaxonCleaningProcedure::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .propagation_protocols()
                                    .any(TaxonProtocol::fields().taxon_id().gt(0)))
                                .or(Taxon::fields()
                                    .notes()
                                    .any(TaxonNote::fields().taxon_id().gt(0))),
                        )
                        .order_by(Taxon::fields().sequence().asc())
                        .exec(db)
                        .await?
                    } else {
                        let taxa = Taxon::all()
                            .order_by(Taxon::fields().sequence().asc())
                            .exec(db)
                            .await?;
                        if taxa.is_empty() {
                            println!(
                                "The taxonomy has not been imported. Please download the ITIS taxonomy database from https://www.itis.gov/downloads/index.html and import it with `pn taxa import`"
                            )
                        }
                        taxa
                    };
                    let ntaxa = taxa.len();
                    let mut tbuilder = tabled::builder::Builder::default();
                    tbuilder.push_record(["ID", "Name"]);
                    for taxon in taxa {
                        tbuilder.push_record([taxon.id.to_string(), taxon.complete_name]);
                    }
                    println!("{}", tbuilder.build().with(style::ListTable));
                    println!("{} taxa found", ntaxa);
                }
            },
            TaxonCommands::Import { db_uri, authority } => {
                libpropagation::taxonomy::import(
                    db,
                    db_uri,
                    *authority,
                    &mut IndicatifImportProgress::default(),
                )
                .await?
            }
            TaxonCommands::Cleaning { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Collecting { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Propagation { taxon_id, command } => command.run(db, *taxon_id).await?,
            TaxonCommands::Notes { taxon_id, command } => command.run(db, *taxon_id).await?,
        }
        Ok(())
    }
}
