use anyhow::anyhow;
use indicatif::ProgressBar;
use libpropagation::{ImportProgressReporter, taxonomy::Taxon};

pub(crate) fn join_or_default<T, F>(items: &[T], default: &str, extract: F) -> String
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        default.to_string()
    } else {
        items.iter().map(extract).collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, Default)]
pub struct IndicatifImportProgress {
    pb: Option<ProgressBar>,
}

impl ImportProgressReporter for IndicatifImportProgress {
    fn begin_step(&mut self, name: &str, total: usize) {
        println!("{name}");
        self.pb = Some(ProgressBar::new(total as u64));
    }

    fn increment(&mut self) {
        if let Some(pb) = &self.pb {
            pb.inc(1);
        }
    }

    fn finish_step(&mut self) {
        if let Some(pb) = self.pb.take() {
            pb.finish_and_clear();
        }
    }
}

// assumes loaded vernacular names
pub async fn inat_taxon_for_taxon(
    taxon: &Taxon,
    client: &inaturalist::Client,
) -> anyhow::Result<inaturalist::Taxon> {
    let query: &str = &taxon.names();
    let possible_taxa = client.taxon_search(query).await?;
    tracing::debug!(?possible_taxa);
    if let Some(it) = possible_taxa.iter().find(|item| taxon.matches(item)) {
        if it.is_active {
            return Ok(it.clone());
        } else {
            // lookup synonyms for the inactive taxon
            if let Some(synonyms) = &it.current_synonymous_taxon_ids {
                let active_synonyms = client.taxa_info(synonyms).await?;
                if active_synonyms.len() == 1 {
                    return Ok(active_synonyms[0].clone());
                } else {
                    if let Some(idx) = dialoguer::Select::new().with_prompt(
                        &format!(
                            "'{}' is not a valid taxon on iNaturalist. Choose one of the following active synonyms",
                            query
                        )).items(&active_synonyms)
                    .interact_opt()? { return Ok(active_synonyms[idx].clone()) }
                };
            }
        }
    } else {
        let mut active_options: Vec<_> =
            possible_taxa.into_iter().filter(|t| t.is_active).collect();
        if active_options.len() == 1 {
            return Ok(active_options.pop().unwrap());
        } else {
            if !active_options.is_empty() &&
            let Some(idx) = dialoguer::Select::new()
                .with_prompt(
                &format!("Couldn't find an exact match for '{query}', but iNaturalist returned the following matches:"))
            .items(&active_options).interact_opt()?
            {
                let taxon = active_options[idx].clone();
                return Ok(taxon);
            }
        }
    }
    tracing::debug!(
        "Couldn't find a matching taxon for the scientific name '{}'",
        taxon.complete_name
    );
    let mut common_name_options = Vec::default();
    if taxon.vernaculars.is_unloaded() {
        tracing::debug!("Can't look up by vernacular name, not loaded");
    } else {
        let vernaculars = taxon.vernaculars.get();
        if !vernaculars.is_empty() {
            tracing::debug!("Attempting to find a match by common name...");
            for vn in taxon.vernaculars.get() {
                let options = client.taxon_search(&vn.name).await?;
                tracing::debug!(?options, "Got matching common name options");
                common_name_options.extend(options.into_iter().map(|t| CommonNameSearchResult {
                    common_name: vn.name.clone(),
                    taxon: t,
                }));
            }
            tracing::debug!(?common_name_options, "all common name results");
            if !common_name_options.is_empty()
                && let Some(idx) = dialoguer::Select::new()
                    .with_prompt(&format!(
                        "The following iNaturalist taxa match one of the common names of '{}'",
                        query
                    ))
                    .items(&common_name_options)
                    .interact_opt()?
            {
                let taxon = &common_name_options[idx].taxon;
                return Ok(taxon.clone());
            }
        }
    }
    Err(anyhow!(
        "Unable to find a match for '{}' in iNaturalist",
        taxon.reference()
    ))
}

#[derive(Debug)]
struct CommonNameSearchResult {
    common_name: String,
    taxon: inaturalist::Taxon,
}

impl std::fmt::Display for CommonNameSearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} matches '{}'", self.taxon, self.common_name)
    }
}

#[cfg(test)]
mod test {
    use libpropagation::taxonomy::Taxon;

    use crate::util::inat_taxon_for_taxon;

    #[tokio::test]
    async fn test_inat_taxon() {
        tracing_subscriber::fmt::init();
        let mut db = libpropagation::db().await.unwrap();
        let taxon = Taxon::filter_by_complete_name("Nuttallanthus canadensis")
            .include(Taxon::fields().vernaculars())
            .one()
            .exec(&mut db)
            .await
            .unwrap();
        let client = inaturalist::Client::new().unwrap();
        let inat_taxon = inat_taxon_for_taxon(&taxon, &client).await.unwrap();
        tracing::debug!(?inat_taxon);
    }
}
