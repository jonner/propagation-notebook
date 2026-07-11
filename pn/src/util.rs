use anyhow::anyhow;
use indicatif::ProgressBar;
use inquire::InquireError;
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

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionWithNone<T> {
    Some(T),
    None,
}

impl<T: std::fmt::Display> std::fmt::Display for SelectionWithNone<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SelectionWithNone::Some(val) => val.to_string(),
                SelectionWithNone::None => "None of these options".to_string(),
            }
        )
    }
}

fn selections<T>(value: Vec<T>) -> Vec<SelectionWithNone<T>> {
    let mut newvec: Vec<SelectionWithNone<T>> =
        value.into_iter().map(SelectionWithNone::Some).collect();
    newvec.push(SelectionWithNone::None);
    newvec
}

// assumes loaded vernacular names
pub async fn inat_taxon_for_taxon(
    taxon: &Taxon,
    client: &inaturalist::Client,
) -> anyhow::Result<inaturalist::Taxon> {
    let query: &str = &taxon.names();
    let possible_taxa = client.taxon_search(query).await?;
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
                    match inquire::Select::new(
                        &format!(
                            "'{}' is not a valid taxon on inaturalist. Choose one of the following active synonyms",
                            query
                        ),
                        selections(active_synonyms),
                    )
                    .prompt()? {
                        SelectionWithNone::Some(t) => return Ok(t),
                        SelectionWithNone::None => (),
                    }
                };
            }
        }
    }
    // there were no exact matches above, and no synonyms were chosen, so present active options to the user
    let active_options: Vec<_> = possible_taxa.into_iter().filter(|t| t.is_active).collect();
    if active_options.len() > 1 {
        // FIXME: add an explicit option to "search by common name" if common names exist
        match inquire::Select::new(
            &format!(
                "Please select an iNaturalist taxon that matches '{}'",
                query
            ),
            selections(active_options),
        )
        .prompt()
        {
            Ok(SelectionWithNone::Some(taxon)) => return Ok(taxon),
            Ok(SelectionWithNone::None) => (),
            Err(InquireError::OperationCanceled) => (),
            Err(e) => return Err(e.into()),
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
                common_name_options.extend(options);
            }
            tracing::debug!(?common_name_options, "all common name results");
            if !common_name_options.is_empty()
                && let SelectionWithNone::Some(taxon) = inquire::Select::new(
                    &format!(
                        "The following iNaturalist taxa match one of the common names of '{}'",
                        query
                    ),
                    selections(common_name_options),
                )
                .prompt()?
            {
                return Ok(taxon);
            }
        }
    }
    Err(anyhow!(
        "Unable to find a match for '{}' in iNaturalist",
        taxon.reference()
    ))
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
