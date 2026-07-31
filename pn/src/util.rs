use anyhow::anyhow;
use demand::DemandOption;
use indicatif::ProgressBar;
use libpropagation::{ImportProgressReporter, taxonomy::Taxon};
use tracing::{debug, trace};

use crate::util::dialog::select;

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

pub async fn find_exact_inat_taxon(
    taxon: &Taxon,
    inat: &inaturalist::Client,
) -> Result<Option<inaturalist::Taxon>, anyhow::Error> {
    let mut found = None;
    let possible_matches = inat
        .taxon_search(&taxon.names())
        .await?
        .into_iter()
        .filter(|t| t.is_active)
        .collect::<Vec<_>>();
    trace!(?taxon, ?possible_matches);
    if !possible_matches.is_empty() {
        for possibility in possible_matches {
            if taxon.matches(&possibility) {
                debug!("Using {} for {}", possibility.name, taxon.reference());
                found = Some(possibility);
                break;
            }
        }
    }
    Ok(found)
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
                    if let Ok(selected) = select(
                        &format!(
                            "'{}' is not a valid taxon on iNaturalist. Choose one of the following active synonyms:",
                            query
                        )).options(active_synonyms.into_iter().map(DemandOption::new).collect())
                    .run() { return Ok(selected) }
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
            let Ok(taxon) = select(
                &format!("Couldn't find an exact match for '{query}', but iNaturalist returned the following matches:"))
            .options(active_options.into_iter().map(DemandOption::new).collect()).run()
            {
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
                && let Ok(result) = select(&format!(
                    "The following iNaturalist taxa match one of the common names of '{}'",
                    query
                ))
                .options(
                    common_name_options
                        .into_iter()
                        .map(DemandOption::new)
                        .collect::<Vec<DemandOption<_>>>(),
                )
                .run()
            {
                return Ok(result.taxon);
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

pub mod dialog {
    use std::sync::OnceLock;

    use demand::{Confirm, Input, Select, Theme};

    pub fn theme<'a>() -> &'a Theme {
        static DIALOG_THEME: OnceLock<Theme> = OnceLock::new();
        DIALOG_THEME.get_or_init(Theme::default)
    }

    pub fn confirm<'a>(title: &str) -> Confirm<'a> {
        Confirm::new(title).theme(theme())
    }

    pub fn input<'a>(title: &str) -> Input<'a> {
        Input::new(title).theme(theme())
    }

    pub fn select<'a, T>(title: &str) -> Select<'a, T> {
        Select::new(title).theme(theme())
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
