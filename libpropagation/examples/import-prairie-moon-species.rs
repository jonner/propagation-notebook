use anyhow::Context;
use libpropagation::{
    citation::{Citation, TaxonPropagationProcedureCitation},
    taxonomy::{Taxon, TaxonPropagationProcedure},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct Cite {
    title: String,
    url: Option<String>,
    author: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Species {
    latin: String,
    common: String,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    propagation_id: u64,
    citation: Cite,
    species: Vec<Species>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let mut db = libpropagation::db().await?;
    let path = std::env::args().nth(1).unwrap();
    tracing::debug!(?path);
    let contents: File =
        serde_yaml::from_reader(std::fs::OpenOptions::new().read(true).open(path)?)?;
    tracing::debug!(?contents);
    let mut txn = db.transaction().await?;
    for sp in contents.species.iter() {
        tracing::debug!(?sp);
        let taxon = match Taxon::get_by_name_or_synonym(&mut txn, &sp.latin).await {
            Ok(t) => Ok(t),
            Err(_) => Taxon::get_by_name_or_synonym(&mut txn, &sp.common).await,
        }
        .with_context(|| format!("Looking up {:?}", sp))?;
        let tpp = toasty::create!(TaxonPropagationProcedure {
            taxon_id: taxon.id,
            propagation_id: contents.propagation_id,
        })
        .exec(&mut txn)
        .await?;
        let _citation = toasty::create!(TaxonPropagationProcedureCitation {
            propagation_id: tpp.propagation_id,
            taxon_id: tpp.taxon_id,
            citation: toasty::create!(Citation {
                title: &contents.citation.title,
                author: contents.citation.author.clone(),
                url: contents.citation.url.clone(),
                date: Some(jiff::Zoned::now().date())
            })
        })
        .exec(&mut txn)
        .await?;
    }
    txn.commit().await?;

    Ok(())
}
