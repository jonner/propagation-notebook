use std::{f64::consts::PI, fmt::Display, sync::LazyLock};

use anyhow::anyhow;
use jiff::civil::Date;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

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
struct Observation {
    observed_on: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ObservationResponse {
    total_results: u32,
    results: Vec<Observation>,
}

#[derive(Serialize, Debug)]
struct TaxonHarvestDates {
    taxon_id: u64,
    name: String,
    start: jiff::civil::Date,
    end: jiff::civil::Date,
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
) -> anyhow::Result<Vec<InaturalistTaxon>> {
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
    location: ObservationLocation,
) -> anyhow::Result<Vec<u16>> {
    let mut day_of_year_list: Vec<u16> = Vec::new();
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
            ("fields", "observed_on"),
        ]);

        match location {
            ObservationLocation::Place(place_id) => {
                builder = builder.query(&[("place_id", &place_id.to_string())])
            }
            ObservationLocation::BoundingBox(rect) => {
                builder = builder.query(&[
                    ("swlat", rect.min().y),
                    ("swlng", rect.min().x),
                    ("nelat", rect.max().y),
                    ("nelng", rect.max().x),
                ])
            }
        }

        let res: ObservationResponse = builder.send().await?.json().await?;

        if res.results.is_empty() {
            break;
        }

        for obs in res.results {
            if let Some(date_str) = obs.observed_on
                && let Ok(parsed_date) = date_str.parse::<Date>()
            {
                day_of_year_list.push(parsed_date.day_of_year().try_into().unwrap());
            }
        }

        // Break based entirely on total page allocations or API bounds
        if page * per_page >= res.total_results as usize || page >= 5 {
            break;
        }
        page += 1;
    }

    Ok(day_of_year_list)
}

async fn calculate_harvest_window(observations: Vec<u16>) -> anyhow::Result<(u16, u16)> {
    if observations.is_empty() {
        return Err(anyhow!("No fruiting observations found."));
    }

    let total_count = observations.len();
    // 2. CALCULATE CIRCULAR MEAN
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;

    for &day in &observations {
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
    let mut valid_days: Vec<u16> = observations
        .iter()
        .copied()
        .filter(|&day| {
            let diff = (day as i16 - mean_day).abs();
            let circular_distance = diff.min(365 - diff) as f64;
            circular_distance <= threshold_days
        })
        .collect();

    if valid_days.is_empty() {
        return Err(anyhow!(
            "All observations filtered out as statistical noise."
        ));
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
pub enum ObservationLocation {
    Place(u32),
    BoundingBox(geo::Rect),
}
pub async fn seed_observation_window(
    client: &reqwest::Client,
    taxon_id: u32,
    location: ObservationLocation,
) -> anyhow::Result<(jiff::civil::Date, jiff::civil::Date)> {
    trace!(
        "Fetching fruiting observations for taxon {} in place {:?}...",
        taxon_id, location
    );
    let day_of_year_list = fetch_seed_observations(client, taxon_id, location).await?;
    if day_of_year_list.len() < 10 {
        return Err(anyhow!(
            "{}: not enough observations for an accurate estimate: {}",
            taxon_id,
            day_of_year_list.len()
        ));
    }
    trace!(
        "Got {} observations for {}",
        day_of_year_list.len(),
        taxon_id
    );
    let (start, end) = calculate_harvest_window(day_of_year_list).await?;

    let target_year = 2000;
    let map_back_to_date = |actual_day: u16| -> jiff::civil::Date {
        let day_normalized = if actual_day == 0 { 365 } else { actual_day };
        jiff::civil::Date::new(target_year, 1, 1)
            .unwrap()
            .with()
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
    Ok((start_date, end_date))
}

pub enum AdminLevel {
    Continent = -10,
    Region = -5,
    Country = 0,
    State = 10,
    County = 20,
    Town = 30,
    Park = 100,
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
