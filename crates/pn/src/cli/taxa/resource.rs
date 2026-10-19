use libpropagation::taxonomy::{
    TaxonResource,
    dto::{TaxonResourceDetails, TaxonResourceNoTaxon},
};
use toasty::Db;

use crate::{
    cli::OutputFormat,
    util::dialog::confirm,
    views::{
        JsonView, YamlView,
        taxa::{TaxonResourceDetailsView, TaxonResourcesListView},
    },
};

#[derive(Debug, clap::Subcommand)]
pub enum TaxonResourceCommands {
    #[command(about = "List resources for the taxon", alias = "ls")]
    List,
    #[command(about = "Show a resource for the taxon")]
    Show {
        #[arg(help = "A resource ID")]
        resource_id: u64,
    },
    #[command(about = "Add a new resource to a taxon", alias = "new")]
    Add {
        #[arg(long, help = "A resource name")]
        name: String,
        #[arg(long, help = "A resource URL")]
        url: String,
    },
    #[command(about = "Modify a resource for a taxon", group(clap::ArgGroup::new("modify_props").args(["name", "url"]).required(true).multiple(true)), alias = "edit")]
    Modify {
        #[arg(help = "A resource ID assigned to this taxon")]
        resource_id: u64,
        #[arg(long, help = "A resource name")]
        name: Option<String>,
        #[arg(long, help = "A resource URL")]
        url: Option<String>,
    },
    #[command(about = "Remove a resource from the taxon")]
    Remove {
        #[arg(help = "A resource ID")]
        resource_id: u64,
        #[arg(
            short = 'y',
            long,
            help = "Assume yes for all questions requiring confirmation"
        )]
        assumeyes: bool,
    },
}

impl TaxonResourceCommands {
    pub async fn run(
        &self,
        db: &mut Db,
        taxon_id: u64,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match self {
            TaxonResourceCommands::List => {
                let resources: Vec<TaxonResourceNoTaxon> =
                    TaxonResource::filter_by_taxon_id(taxon_id)
                        .exec(db)
                        .await?
                        .into_iter()
                        .map(Into::into)
                        .collect();
                let output = match format {
                    OutputFormat::Text => TaxonResourcesListView::new(&resources).render()?,
                    OutputFormat::Json => JsonView::new(&resources).render()?,
                    OutputFormat::Yaml => YamlView::new(&resources).render()?,
                };
                println!("{output}");
            }
            TaxonResourceCommands::Show { resource_id } => {
                let resource: TaxonResourceDetails =
                    TaxonResource::get_by_id(db, resource_id).await?.into();
                let output = match format {
                    OutputFormat::Text => TaxonResourceDetailsView::new(&resource).render()?,
                    OutputFormat::Json => JsonView::new(&resource).render()?,
                    OutputFormat::Yaml => YamlView::new(&resource).render()?,
                };
                println!("{output}");
            }
            TaxonResourceCommands::Add { name, url } => {
                let resource: TaxonResourceDetails = TaxonResource::create()
                    .taxon_id(taxon_id)
                    .name(name)
                    .url(url)
                    .exec(db)
                    .await?
                    .into();
                let output = match format {
                    OutputFormat::Text => TaxonResourceDetailsView::new(&resource).render()?,
                    OutputFormat::Json => JsonView::new(&resource).render()?,
                    OutputFormat::Yaml => YamlView::new(&resource).render()?,
                };
                println!("{output}");
            }
            TaxonResourceCommands::Modify {
                resource_id,
                name,
                url,
            } => {
                let mut upd = TaxonResource::update_by_id(resource_id);
                if let Some(name) = name {
                    upd = upd.name(name);
                }
                if let Some(url) = url {
                    upd = upd.url(url);
                }
                upd.exec(db).await?;
                println!("Updated resource {resource_id}")
            }
            TaxonResourceCommands::Remove {
                resource_id,
                assumeyes,
            } => {
                let resource: TaxonResourceDetails =
                    TaxonResource::get_by_id(db, resource_id).await?.into();
                if *assumeyes || {
                    println!("{}", TaxonResourceDetailsView::new(&resource).render()?);
                    confirm(&format!(
                        "Are you sure you wish to remove resource {resource_id}?"
                    ))
                    .selected(false)
                    .run()?
                } {
                    {
                        TaxonResource::delete_by_id(db, resource_id).await?;
                        println!("Removed resource {resource_id}");
                    }
                }
            }
        }
        Ok(())
    }
}
