use std::{fmt::Display, sync::LazyLock, time::Duration};

use jiff::civil::Date;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not enough iNaturalist observations found ({0})")]
    InsufficientObservations(usize),
    #[error("Unable to fetch details for iNaturalist taxon {0}")]
    TaxonNotFound(u64),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Date(#[from] jiff::Error),
}

#[derive(Deserialize, Debug)]
pub struct InaturalistTaxon {
    pub id: u64,
    pub name: String,
    pub rank: String,
    pub is_active: bool,
}

impl Display for InaturalistTaxon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}){}", self.name, self.rank, {
            if self.is_active { "" } else { " (inactive)" }
        })
    }
}

#[derive(Deserialize, Debug)]
struct TaxonSearchResponse {
    results: Vec<InaturalistTaxon>,
}

#[derive(Deserialize, Debug)]
pub struct ObservationDate {
    pub id: u64,
    pub observed_on: Option<Date>,
}

#[derive(Deserialize, Debug)]
struct ObservationDateResponse {
    total_results: u32,
    results: Vec<ObservationDate>,
}

const PLANT_PHENOLOGY: &str = "12";
const FRUITING: &str = "14";
static API_BASE_URL: LazyLock<reqwest::Url> = LazyLock::new(|| {
    reqwest::Url::parse("https://api.inaturalist.org/v2/").expect("Valid static base URL")
});

pub fn client() -> Result<reqwest::Client, reqwest::Error> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        "User-Agent",
        HeaderValue::from_static("propagation-notebook/1.0 (jonathon@quotidian.org)"),
    );
    reqwest::Client::builder()
        .connection_verbose(true)
        .default_headers(default_headers)
        .build()
}

pub async fn taxon_info(
    client: &reqwest::Client,
    taxon_id: u64,
) -> Result<InaturalistTaxon, Error> {
    let taxa_endpoint = API_BASE_URL.join("taxa/")?.join(&taxon_id.to_string())?;
    let mut res: TaxonSearchResponse = client
        .get(taxa_endpoint)
        .query(&[("fields", "id,name,rank,is_active")])
        .send()
        .await?
        .json()
        .await?;

    res.results
        .pop()
        .ok_or_else(|| Error::TaxonNotFound(taxon_id))
}

pub async fn find_taxon(
    client: &reqwest::Client,
    taxon_name: &str,
) -> Result<Vec<InaturalistTaxon>, Error> {
    let taxa_endpoint = API_BASE_URL.join("taxa")?;
    let res: TaxonSearchResponse = client
        .get(taxa_endpoint)
        .query(&[
            ("q", taxon_name),
            ("per_page", "5"),
            ("fields", "id,name,rank,is_active"),
        ])
        .send()
        .await?
        .json()
        .await?;

    Ok(res.results)
}

pub async fn fetch_seed_observations(
    client: &reqwest::Client,
    taxon_id: u64,
    location: &SearchArea,
) -> Result<Vec<ObservationDate>, Error> {
    let mut observations: Vec<ObservationDate> = Vec::new();
    let (mut page, per_page) = (1, 200);
    let obs_endpoint = API_BASE_URL.join("observations")?;

    loop {
        let mut builder = client.get(obs_endpoint.clone()).query(&[
            ("taxon_id", taxon_id.to_string().as_str()),
            ("term_id", PLANT_PHENOLOGY),
            ("term_value_id", FRUITING),
            // ("identifications", "most_agree"),
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("fields", "id,observed_on"),
        ]);

        match location {
            SearchArea::Place(place_id) => {
                builder = builder.query(&[("place_id", &place_id.to_string())])
            }
            SearchArea::BoundingBox(rect) => {
                builder = builder.query(&[
                    ("swlat", rect.min().y),
                    ("swlng", rect.min().x),
                    ("nelat", rect.max().y),
                    ("nelng", rect.max().x),
                ])
            }
        }

        let res: ObservationDateResponse = builder.send().await?.json().await?;

        if res.results.is_empty() {
            break;
        }

        observations.extend(res.results);

        if page * per_page >= res.total_results as usize {
            break;
        }
        page += 1;
        // short pause to avoid triggering API limits
        std::thread::sleep(Duration::from_millis(200));
    }

    Ok(observations)
}

#[derive(Debug)]
pub enum SearchArea {
    Place(u32),
    BoundingBox(geo::Rect),
}

#[derive(Debug, Deserialize)]
pub struct InaturalistPlace {
    pub id: u32,
    pub admin_level: Option<i32>,
    pub display_name: Option<String>,
    pub bounding_box_geojson: Option<geojson::Geometry>,
}

impl Display for InaturalistPlace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.display_name.as_deref().unwrap_or("unnamed location"),
            match self.admin_level {
                Some(-10) => " (Continent)",
                Some(-5) => " (Region)",
                Some(0) => " (Country)",
                Some(10) => " (State)",
                Some(20) => " (County)",
                Some(30) => " (Town)",
                Some(100) => " (Park)",
                _ => "",
            }
        )
    }
}

#[derive(Debug, Deserialize)]
struct PlacesSearchResults {
    results: Vec<InaturalistPlace>,
}

pub async fn places_search(
    client: &reqwest::Client,
    q: &str,
) -> Result<Vec<InaturalistPlace>, Error> {
    let taxa_endpoint = API_BASE_URL.join("places")?;
    let res: PlacesSearchResults = client
        .get(taxa_endpoint)
        .query(&[
            ("q", q),
            ("per_page", "10"),
            ("fields", "id,admin_level,display_name,bounding_box_geojson"),
        ])
        .send()
        .await?
        .json()
        .await?;

    Ok(res.results)
}
