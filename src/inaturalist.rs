use std::{f64::consts::PI, fmt::Display, sync::LazyLock, time::Duration};

use jiff::civil::Date;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use tracing::{debug, trace};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not enough observations found {0}")]
    InsufficientObservations(usize),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
}

#[derive(Deserialize, Debug)]
pub struct InaturalistTaxon {
    pub id: u32,
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

pub fn client() -> anyhow::Result<reqwest::Client> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        "User-Agent",
        HeaderValue::from_static("propagation-notebook/1.0 (jonathon@quotidian.org)"),
    );
    reqwest::Client::builder()
        .connection_verbose(true)
        .default_headers(default_headers)
        .build()
        .map_err(Into::into)
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

async fn fetch_seed_observations(
    client: &reqwest::Client,
    taxon_id: u32,
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

async fn calculate_harvest_window(
    observations: &Vec<ObservationDate>,
) -> Result<(u16, u16), Error> {
    if observations.is_empty() {
        return Err(Error::InsufficientObservations(0));
    }
    let observations_doy: Vec<u16> = observations
        .into_iter()
        .filter_map(|ob| ob.observed_on.map(|d| d.day_of_year().try_into().unwrap()))
        .collect();

    let total_count = observations_doy.len();
    // 2. CALCULATE CIRCULAR MEAN
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;

    for &day in &observations_doy {
        let angle = (day as f64 / 365.25) * 2.0 * PI;
        sum_sin += angle.sin();
        sum_cos += angle.cos();
    }

    let avg_sin = sum_sin / total_count as f64;
    let avg_cos = sum_cos / total_count as f64;

    // REPAIRED DIRECTIONAL ANGLE CALCULATION
    let mut mean_angle = avg_sin.atan2(avg_cos);
    if mean_angle < 0.0 {
        mean_angle += 2.0 * PI; // Safely normalizes standard negative radians to 0..2*PI range
    }
    let mean_day = ((mean_angle / (2.0 * PI)) * 365.25).round() as i16 % 365;

    let r = (avg_sin.powi(2) + avg_cos.powi(2)).sqrt();
    let r_clamped = r.clamp(0.001, 1.0);
    let circ_std_dev_radians = (-2.0 * r_clamped.ln()).sqrt();
    let std_dev_days = (circ_std_dev_radians / (2.0 * PI)) * 365.25;

    // Use a conservative threshold factor (1.25 to 1.5 dev standard bounds)
    let threshold_days = (std_dev_days * 1.25).max(14.0);

    trace!("Data Center: Day {}", mean_day);
    trace!("Data Clustering Strength (R): {:.2}", r);
    trace!("Calculated Standard Deviation: {:.1} days", std_dev_days);
    trace!(
        "Filtering out entries further than {:.1} days from center...",
        threshold_days
    );

    // 4. FILTER OUTLIERS USING CIRCULAR DISTANCE
    let mut valid_days: Vec<u16> = observations_doy
        .iter()
        .copied()
        .filter(|&day| {
            let diff = (day as i16 - mean_day).abs();
            let circular_distance = diff.min(365 - diff) as f64;
            circular_distance <= threshold_days
        })
        .collect();

    if valid_days.is_empty() {
        debug!("All observations filtered out as statistical noise.");
        return Err(Error::InsufficientObservations(0));
    }

    // 5. CORRECT CHRONOLOGICAL SORTING (WINTER-SAFE)
    let anchor_day = ((mean_day + 182) % 365) as u16;

    valid_days.sort_by_key(|&day| {
        if day > anchor_day {
            day - anchor_day
        } else {
            (day + 365) - anchor_day
        }
    });

    Ok((valid_days[0], valid_days[valid_days.len() - 1]))
}

#[derive(Debug)]
pub enum SearchArea {
    Place(u32),
    BoundingBox(geo::Rect),
}

pub async fn seed_observation_window(
    client: &reqwest::Client,
    taxon_id: u32,
    area: &SearchArea,
    min_samples: usize,
) -> Result<((Date, Date), usize), Error> {
    trace!(
        "Fetching fruiting observations for taxon {} in area {:?}...",
        taxon_id, area
    );
    let observation_list = fetch_seed_observations(client, taxon_id, area).await?;
    if observation_list.len() < min_samples {
        return Err(Error::InsufficientObservations(observation_list.len()));
    }
    trace!(
        "Got {} observations for {}",
        observation_list.len(),
        taxon_id
    );
    let (start, end) = calculate_harvest_window(&observation_list).await?;

    let target_year = 2000;
    let map_back_to_date = |actual_day: u16| -> Date {
        let day_normalized = if actual_day == 0 { 365 } else { actual_day };
        Date::default()
            .with()
            .year(target_year)
            .day_of_year(day_normalized as i16)
            .build()
            .unwrap()
    };

    let start_date = map_back_to_date(start);
    let end_date = map_back_to_date(end);
    debug!(
        "Harvest dates for {}: {} - {}",
        taxon_id,
        start_date.strftime("%b-%d"),
        end_date.strftime("%b-%d")
    );
    Ok(((start_date, end_date), observation_list.len()))
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
) -> anyhow::Result<Vec<InaturalistPlace>> {
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
