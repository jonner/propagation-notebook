use std::{fmt::Display, sync::LazyLock, time::Duration};

use jiff::civil::Date;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not enough iNaturalist observations found ({0})")]
    InsufficientObservations(usize),
    #[error("Unable to fetch details for iNaturalist taxon {0}")]
    TaxonNotFound(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Date(#[from] jiff::Error),
    #[error("Got an error response from iNaturalist: {0}")]
    Response(ErrorResponse),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Success(ResultsResponse<T>),
    Failure(ErrorResponse),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResultValue<T> {
    Vec(Vec<T>),
    Object(T),
}

impl<T> ResultValue<T> {
    // panics if value is not a vec
    pub fn vec(self) -> Vec<T> {
        match self {
            ResultValue::Vec(items) => items,
            ResultValue::Object(_) => panic!(),
        }
    }

    // panics if value is not an object
    pub fn object(self) -> T {
        match self {
            ResultValue::Vec(_) => panic!(),
            ResultValue::Object(obj) => obj,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResultsResponse<T> {
    total_results: u64,
    results: ResultValue<T>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    status: String,
    errors: Vec<ErrorObj>,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error {}:\n{}",
            self.status,
            self.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ErrorObj {
    #[serde(rename = "errorCode")]
    error_code: String,
    message: String,
}

impl Display for ErrorObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_code, self.message)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Taxon {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub name: String,
    pub rank: String,
    pub is_active: bool,
    pub current_synonymous_taxon_ids: Option<Vec<u64>>,
    pub preferred_common_name: Option<String>,
}

impl Taxon {
    fn fields() -> &'static str {
        "id,name,rank,is_active,parent_id,preferred_common_name,current_synonymous_taxon_ids"
    }
}

impl Display for Taxon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vec = Vec::with_capacity(4);
        vec.push(self.name.as_str());
        if let Some(cn) = &self.preferred_common_name {
            vec.push(cn);
        }
        vec.push(&self.rank);
        if !self.is_active {
            vec.push("inactive");
        }
        write!(f, "{}", vec.join(" // "))
    }
}

#[derive(Deserialize, Debug)]
pub struct Observation {
    pub id: u64,
    pub observed_on: Option<Date>,
}

impl Observation {
    fn fields() -> &'static str {
        "id,observed_on"
    }
}

#[derive(Debug, Deserialize)]
pub struct TaxonDefaultPhoto {
    pub id: u64,
    default_photo: Option<DefaultPhoto>,
}

#[derive(Debug, Deserialize)]
pub struct DefaultPhoto {
    pub id: u64,
    pub square_url: Option<String>,
    pub medium_url: Option<String>,
    pub large_url: Option<String>,
    pub attribution: Option<String>,
}

const PLANTAE_ID: &str = "47126";
const PLANT_PHENOLOGY: &str = "12";
const FRUITING: &str = "14";
static API_BASE_URL: LazyLock<reqwest::Url> = LazyLock::new(|| {
    reqwest::Url::parse("https://api.inaturalist.org/v2/").expect("Valid static base URL")
});

pub struct Client(reqwest_middleware::ClientWithMiddleware);

impl Client {
    pub fn new() -> Result<Self, reqwest::Error> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "User-Agent",
            HeaderValue::from_static("propagation-notebook/1.0 (jonathon@quotidian.org)"),
        );
        Ok(Self(
            reqwest_middleware::ClientBuilder::new(
                reqwest::ClientBuilder::new()
                    .connection_verbose(true)
                    .default_headers(default_headers)
                    .build()?,
            )
            .with(RetryTransientMiddleware::new_with_policy(
                ExponentialBackoff::builder()
                    .retry_bounds(Duration::from_secs(1), Duration::from_secs(30))
                    .jitter(reqwest_retry::Jitter::None)
                    .build_with_max_retries(5),
            ))
            .build(),
        ))
    }

    pub async fn taxa_info(&self, ids: &[u64]) -> Result<Vec<Taxon>, Error> {
        tracing::trace!(?ids, "getting taxa info");
        let ids_string: String = ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let taxa_endpoint = API_BASE_URL.join("taxa/")?.join(&ids_string)?;
        let res: Response<Taxon> = self
            .0
            .get(taxa_endpoint)
            .query(&[("fields", Taxon::fields())])
            .send()
            .await?
            .json()
            .await?;

        match res {
            Response::Success(results_response) => {
                let results = results_response.results.vec();
                if results.is_empty() {
                    Err(Error::TaxonNotFound(
                        ids.iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                    ))
                } else {
                    Ok(results)
                }
            }
            Response::Failure(error_response) => Err(Error::Response(error_response)),
        }
    }

    pub async fn taxon_default_photo(&self, taxon_id: u64) -> Result<Option<DefaultPhoto>, Error> {
        tracing::trace!(?taxon_id, "getting taxon default photo");
        let taxa_endpoint = API_BASE_URL.join("taxa/")?.join(&taxon_id.to_string())?;
        let res: Response<TaxonDefaultPhoto> = self
            .0
            .get(taxa_endpoint)
            .query(&[("fields", "(id:!t,default_photo:(id:!t,square_url:!t,medium_url:!t,large_url:!t,attribution:!t))")])
            .send()
            .await?
            .json()
            .await?;

        let mut val = match res {
            Response::Success(results_response) => {
                let results = results_response.results.vec();
                if results.is_empty() {
                    Err(Error::TaxonNotFound(taxon_id.to_string()))
                } else {
                    Ok(results)
                }
            }
            Response::Failure(error_response) => Err(Error::Response(error_response)),
        }?;
        Ok(val.pop().unwrap().default_photo)
    }

    pub async fn taxon_info(&self, taxon_id: u64) -> Result<Taxon, Error> {
        let mut taxa = self.taxa_info(&[taxon_id]).await?;

        // the parent taxa_info() guarantees that the returned vec is not empty
        // on success, so just unwrap
        Ok(taxa.pop().unwrap())
    }

    pub async fn taxon_search(&self, taxon_name: &str) -> Result<Vec<Taxon>, Error> {
        tracing::debug!("Searching for {taxon_name}");
        let taxa_endpoint = API_BASE_URL.join("taxa")?;
        let res: Response<Taxon> = self
            .0
            .get(taxa_endpoint)
            .query(&[
                // limit any results to plant taxa
                ("taxon_id", PLANTAE_ID),
                ("q", taxon_name),
                ("per_page", "25"),
                ("fields", Taxon::fields()),
            ])
            .send()
            .await?
            .json()
            .await?;

        match res {
            Response::Success(res) => Ok(res.results.vec()),
            Response::Failure(res) => Err(Error::Response(res)),
        }
    }

    pub async fn seed_observations(
        &self,
        taxon_id: u64,
        location: &SearchArea,
    ) -> Result<Vec<Observation>, Error> {
        let mut observations: Vec<Observation> = Vec::new();
        let (mut page, per_page) = (1, 200);
        let obs_endpoint = API_BASE_URL.join("observations")?;

        loop {
            let mut builder = self.0.get(obs_endpoint.clone()).query(&[
                ("taxon_id", taxon_id.to_string().as_str()),
                ("term_id", PLANT_PHENOLOGY),
                ("term_value_id", FRUITING),
                ("identifications", "most_agree"),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
                ("fields", Observation::fields()),
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

            let res: Response<Observation> = builder.send().await?.json().await?;

            match res {
                Response::Success(res) => {
                    let results = res.results.vec();
                    if results.is_empty() {
                        break;
                    }

                    observations.extend(results);

                    if page * per_page >= res.total_results as usize {
                        break;
                    }
                    page += 1;
                }
                Response::Failure(error_response) => println!("{error_response}"),
            }
        }

        Ok(observations)
    }

    pub async fn place_search(&self, q: &str) -> Result<Vec<Place>, Error> {
        let taxa_endpoint = API_BASE_URL.join("places")?;
        let res: Response<Place> = self
            .0
            .get(taxa_endpoint)
            .query(&[("q", q), ("per_page", "10"), ("fields", Place::fields())])
            .send()
            .await?
            .json()
            .await?;

        match res {
            Response::Success(results_response) => Ok(results_response.results.vec()),
            Response::Failure(error_response) => Err(Error::Response(error_response)),
        }
    }
}

#[derive(Debug)]
pub enum SearchArea {
    Place(u32),
    BoundingBox(geo::Rect),
}

#[derive(Debug, Deserialize)]
pub struct Place {
    pub id: u32,
    pub admin_level: Option<i32>,
    pub display_name: Option<String>,
    pub bounding_box_geojson: Option<geojson::Geometry>,
}

impl Place {
    fn fields() -> &'static str {
        "id,admin_level,display_name,bounding_box_geojson"
    }
}

impl Display for Place {
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

#[cfg(test)]
mod test {
    use crate::{Response, Taxon};

    #[test]
    fn test_deserialization() {
        const GOOD: &str = "{\"total_results\":1,\"page\":1,\"per_page\":30,\"results\":[{\"id\":79317,\"name\":\"Taraxacum erythrospermum\",\"rank\":\"species\",\"is_active\":true,\"parent_id\":967521}]}";
        const BAD: &str = "{\"status\":\"12345\",\"errors\":[{\"errorCode\":\"my-error-code\",\"message\":\"this is a message\",\"from\":\"some text here\",\"stack\":\"some text here also\"}]}";

        let _good_response: Response<Taxon> =
            serde_json::from_str(GOOD).expect("Failed to parse GOOD");
        let _bad_response: Response<Taxon> =
            serde_json::from_str(BAD).expect("Failed to parse BAD");
    }
}
