use libpropagation::{
    citation::{Citation, TaxonPropagationProcedureCitation},
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use serde::Deserialize;
use url::Url;

const PRAIRIE_MOON_REFERENCES: &str = include_str!("../prairie-moon-references.json");

#[derive(Debug, Deserialize)]
struct PrairieMoonReference {
    url: Url,
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let mut db = libpropagation::db(true).await?;
    tracing::debug!("opened db");
    let mut tx = db.transaction().await?;
    tracing::debug!("Created transaction");
    // strip any potential utf BOM
    let json_str = PRAIRIE_MOON_REFERENCES
        .strip_prefix('\u{FEFF}')
        .unwrap_or(PRAIRIE_MOON_REFERENCES);
    tracing::debug!(json_str);
    let references: Vec<PrairieMoonReference> = serde_json::from_str(json_str)?;
    tracing::debug!(?references);
    for reference in references.iter() {
        tracing::debug!(?reference);
        let taxon = Taxon::get_by_name_or_synonym(&mut tx, &reference.name).await?;
        let props = TaxonPropagationProcedure::filter_by_taxon_id(taxon.id)
            .exec(&mut tx)
            .await?;
        let citation = Citation::create()
            .title(format!("{} Plant Profile", reference.name))
            .url(reference.url.to_string())
            .author("Prairie Moon")
            .exec(&mut tx)
            .await?;
        for prop in props {
            TaxonPropagationProcedureCitation::create()
                .citation_id(citation.id)
                .taxon_id(prop.taxon_id)
                .propagation_id(prop.propagation_id)
                .exec(&mut tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}
