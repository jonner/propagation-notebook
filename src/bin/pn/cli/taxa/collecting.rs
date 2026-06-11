use propagation_notebook::collecting::CollectingData;

use tabled::builder::Builder as TableBuilder;
use toasty::Db;

#[derive(Debug, clap::Subcommand)]
pub enum TaxonCollectingCommands {
    #[command(about = "Show seed collecting information")]
    Show,
    #[command(about = "Modify seed collecting information for a taxon", group(clap::ArgGroup::new("modify_props").args(["ripening_indicators", "harvesting_notes", "storage_conditions", "storage_life"]).required(true).multiple(true)))]
    Modify {
        #[arg(
            short,
            long,
            help = "What to look for to determine if the seed is ready for collecting"
        )]
        ripening_indicators: Option<String>,
        #[arg(long, help = "Harvesting notes")]
        harvesting_notes: Option<String>,
        #[arg(short, long, help = "Instructions for storing the seed")]
        storage_conditions: Option<String>,
        #[arg(
            short = 'l',
            long,
            help = "How long the seed will stay viable in storage"
        )]
        storage_life: Option<String>,
    },
    #[command(about = "Remove seed collecting information")]
    Remove {
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonCollectingCommands {
    pub async fn run(&self, db: &mut Db, taxon_id: u64) -> anyhow::Result<()> {
        match self {
            TaxonCollectingCommands::Show => {
                match CollectingData::filter_by_taxon_id(taxon_id)
                    .include(CollectingData::fields().taxon())
                    .one()
                    .exec(db)
                    .await
                {
                    Ok(data) => {
                        let mut tbuilder = TableBuilder::default();
                        tbuilder.push_record(["Taxon", &data.taxon.get().reference()]);
                        tbuilder.push_record([
                            "Ripening",
                            data.ripening_indicators.as_deref().unwrap_or("-"),
                        ]);
                        tbuilder.push_record([
                            "Harvesting",
                            data.harvesting_notes.as_deref().unwrap_or("-"),
                        ]);
                        tbuilder.push_record([
                            "Storage Conditions",
                            data.storage.as_deref().unwrap_or("-"),
                        ]);
                        tbuilder.push_record([
                            "Storage Life",
                            data.storage_life.as_deref().unwrap_or("-"),
                        ]);
                        println!("{}", tbuilder.build().with(crate::style::DetailTable))
                    }
                    Err(e) if e.is_record_not_found() => println!(
                        "Taxon {taxon_id} does not current have any collecting information defined"
                    ),
                    Err(e) => return Err(e.into()),
                }
            }
            TaxonCollectingCommands::Remove { assumeyes } => {
                if *assumeyes
                    || inquire::Confirm::new(
                        "Are you sure you wish to remove this collecting data?",
                    )
                    .with_default(false)
                    .prompt()?
                {
                    CollectingData::delete_by_taxon_id(db, taxon_id).await?;
                    println!("Removed collecting data {taxon_id}")
                }
            }
            TaxonCollectingCommands::Modify {
                ripening_indicators,
                harvesting_notes,
                storage_conditions,
                storage_life,
            } => {
                // Try to create the object first
                match CollectingData::create()
                    .taxon_id(taxon_id)
                    .ripening_indicators(ripening_indicators)
                    .harvesting_notes(harvesting_notes)
                    .storage(storage_conditions)
                    .storage_life(storage_life)
                    .exec(db)
                    .await
                {
                    Ok(data) => {
                        println!("Added collection information for taxon {}", data.taxon_id)
                    }
                    Err(e)
                        if e.is_driver_operation_failed()
                            // FIXME: it would be nicer if the error categories
                            // were more fine-grained and I didn't have to
                            // examine the error string
                            && e.to_string().contains("constraint failed") =>
                    {
                        // The insertion failed because CollectionData already
                        // exists for taxon_id, which has a unique constraint.
                        // Just update the existing row
                        let mut query = CollectingData::update_by_taxon_id(taxon_id);
                        if let Some(ripening) = ripening_indicators {
                            query = query.ripening_indicators(ripening);
                        }
                        if let Some(harvesting) = harvesting_notes {
                            query = query.harvesting_notes(harvesting);
                        }
                        if let Some(storage) = storage_conditions {
                            query = query.storage(storage);
                        }
                        if let Some(storage_life) = storage_life {
                            query = query.storage_life(storage_life);
                        }
                        query.exec(db).await?;
                        println!("Modified collection information {taxon_id}");
                    }
                    Err(e) => Err(e)?,
                };
            }
        }
        Ok(())
    }
}
