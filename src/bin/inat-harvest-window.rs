use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use jiff::civil::Date;
use propagation_notebook::region::RegionalTaxonStatus;
use propagation_notebook::taxonomy::Taxon;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::sync::LazyLock;
use std::thread::sleep;
use std::{error::Error, io::stdout};
use toasty::Db;
use tracing::{debug, trace, warn};
use uuid::Uuid;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long)]
    region_id: Option<u64>,
    #[arg(long)]
    taxon_id: Option<u64>,
    #[arg(long)]
    taxon_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TaxonItem {
    id: u32,
    name: String,
    rank: String,
    is_active: bool,
}

#[derive(Deserialize, Debug)]
struct TaxonSearchResponse {
    results: Vec<TaxonItem>,
}

#[derive(Deserialize, Debug)]
struct Observation {
    uuid: Uuid,
    observed_on: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ObservationResponse {
    total_results: u32,
    page: u32,
    per_page: u32,
    results: Vec<Observation>,
}

#[derive(Serialize, Debug)]
struct TaxonHarvestDates {
    taxon_id: u64,
    start: jiff::civil::Date,
    end: jiff::civil::Date,
}

const PLANT_PHENOLOGY: &str = "12";
const FRUITING: &str = "14";
const MINNESOTA_PLACE_ID: u32 = 38;
static API_BASE_URL: LazyLock<reqwest::Url> = LazyLock::new(|| {
    reqwest::Url::parse("https://api.inaturalist.org/v2/").expect("Valid static base URL")
});

async fn find_taxon(client: &reqwest::Client, taxon_name: &str) -> anyhow::Result<u32> {
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

    let matched_taxon = res.results.iter().find(|t| t.is_active);
    match matched_taxon {
        Some(t) => {
            trace!(
                "Found matching Taxon: {} (ID: {}, Rank: {})",
                t.name, t.id, t.rank
            );
            Ok(t.id)
        }
        None => Err(anyhow::anyhow!(
            "No active taxon found matching that string."
        )),
    }
}

async fn seed_observations(
    client: &reqwest::Client,
    taxon_id: u32,
    place_id: u32,
) -> anyhow::Result<Vec<u16>> {
    let mut day_of_year_list: Vec<u16> = Vec::new();
    let (mut page, per_page) = (1, 200);
    let obs_endpoint = API_BASE_URL.join("observations")?;

    loop {
        let res: ObservationResponse = client
            .get(obs_endpoint.clone())
            .query(&[
                ("taxon_id", taxon_id.to_string().as_str()),
                ("place_id", &place_id.to_string()),
                ("term_id", PLANT_PHENOLOGY),
                ("term_value_id", FRUITING),
                // ("identifications", "most_agree"),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
                ("fields", "observed_on"),
            ])
            .send()
            .await?
            .json()
            .await?;

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

async fn inat_seed_dates(
    client: &reqwest::Client,
    taxon: &Taxon,
) -> anyhow::Result<(jiff::civil::Date, jiff::civil::Date)> {
    trace!("Searching for taxon matching '{}'...", taxon.complete_name);
    let taxon_id = find_taxon(client, &taxon.complete_name).await?;

    trace!("Fetching fruiting observations for region...");
    let day_of_year_list = seed_observations(client, taxon_id, MINNESOTA_PLACE_ID).await?;
    if day_of_year_list.len() < 10 {
        return Err(anyhow!(
            "{}: not enough observations for an accurate estimate: {}",
            taxon.complete_name,
            day_of_year_list.len()
        ));
    }
    trace!(
        "Got {} observations for {}",
        day_of_year_list.len(),
        taxon.complete_name
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
        taxon.complete_name,
        start_date.strftime("%b-%d"),
        end_date.strftime("%b-%d")
    );
    Ok((start_date, end_date))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let project_dir = ProjectDirs::from("org", "quotidian", "propagation-notebook")
        .ok_or_else(|| anyhow!("Unable to determine project data directory"))?
        .data_dir()
        .to_path_buf();
    let db_uri = match std::env::var("PN_DB_URI") {
        Ok(s) => Ok(s),
        Err(std::env::VarError::NotPresent) => Ok(format!(
            "sqlite:{}",
            project_dir
                .join("propagation-notebook.sqlite")
                .to_str()
                .unwrap()
        )),
        e => e,
    }?;
    let mut db = Db::builder()
        .models(propagation_notebook::models())
        .connect(&db_uri)
        .await?;

    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        "User-Agent",
        HeaderValue::from_static("propagation-notebook/1.0 (jonathon@quotidian.org)"),
    );
    let client = reqwest::Client::builder()
        .connection_verbose(true)
        .default_headers(default_headers)
        .build()?;

    let args = Args::parse();
    let taxa = if let Some(region_id) = args.region_id {
        Taxon::filter(
            Taxon::fields()
                .regional_statuses()
                .any(RegionalTaxonStatus::fields().region_id().eq(region_id)),
        )
        .exec(&mut db)
        .await?
    } else if let Some(taxon_name) = args.taxon_name {
        vec![Taxon::get_by_complete_name(&mut db, taxon_name).await?]
    } else if let Some(taxon_id) = args.taxon_id {
        vec![Taxon::get_by_id(&mut db, taxon_id).await?]
    } else {
        return Err(anyhow!("Please specify an argument"));
    };

    let mut results = Vec::default();
    for taxon in taxa {
        match inat_seed_dates(&client, &taxon).await {
            Ok(dates) => results.push(TaxonHarvestDates {
                taxon_id: taxon.id,
                start: dates.0,
                end: dates.1,
            }),
            Err(e) => warn!(?e),
        };
        sleep(std::time::Duration::from_secs(1));
    }

    serde_yaml::to_writer(stdout(), &results)?;
    Ok(())
}
