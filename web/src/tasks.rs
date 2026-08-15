use std::time::Duration;

use libpropagation::{
    region::{QueryHarvestError, RegionalTaxonStatus},
    taxonomy::{Rank, Taxon},
};
use tracing::{debug, warn};
const MIN_SAMPLES_HARVEST_WINDOW: usize = 20;

#[derive(Debug, thiserror::Error)]
pub enum BackgroundError {
    #[error(transparent)]
    Db(#[from] toasty::Error),
    #[error(transparent)]
    QueryHarvest(#[from] QueryHarvestError),
}

async fn update_regional_status(
    db: &mut toasty::Db,
    rts: &RegionalTaxonStatus,
) -> Result<(), BackgroundError> {
    let taxon = rts.taxon.get();
    let region = rts.region.get();
    debug!(
        "Looking up harvest window for {} in region {}",
        taxon.complete_name, region.name
    );
    let (samples, window) = rts.query_harvest_info(db).await?;
    if samples > MIN_SAMPLES_HARVEST_WINDOW && window != rts.harvest_window {
        debug!("Updating harvest window: {}", window);
        RegionalTaxonStatus::update_by_id(rts.id)
            .harvest_window(window)
            .exec(db)
            .await?;
    } else {
        debug!("Not enough samples found to calculate a harvest window");
        // if this is a subspecies or variety and there are no
        // sibling subspecies and no parent species, we can just use
        // the parent taxa for harvest window
        if taxon.rank == Rank::Subspecies || taxon.rank == Rank::Variety {
            debug!("This taxon is a ssp. or var... Checking for siblings");
            let n_siblings = RegionalTaxonStatus::filter(
                RegionalTaxonStatus::fields().region_id().eq(region.id).and(
                    RegionalTaxonStatus::fields()
                        .taxon()
                        .parent_id()
                        .eq(taxon.parent_id),
                ),
            )
            .count()
            .exec(db)
            .await?;
            if let Some(parent_id) = taxon.parent_id {
                debug!("Checking for parent within the region...");
                let has_parent = RegionalTaxonStatus::filter(
                    RegionalTaxonStatus::fields()
                        .region_id()
                        .eq(region.id)
                        .and(RegionalTaxonStatus::fields().taxon_id().eq(parent_id)),
                )
                .count()
                .exec(db)
                .await?
                    > 0;
                if !has_parent && n_siblings == 1 {
                    debug!(
                        "No parent or siblings listed in this region. let's use the parent taxon to calculate a harvest window..."
                    );
                    if let Ok(parent) = Taxon::get_by_id(db, parent_id).await
                        && let Ok((samples, window)) = region.query_harvest_info(&parent, db).await
                        && samples > MIN_SAMPLES_HARVEST_WINDOW
                    {
                        debug!(
                            "Updating harvest window of {} using parent taxon {} : {}",
                            taxon.complete_name, parent.complete_name, window,
                        );
                        RegionalTaxonStatus::update_by_id(rts.id)
                            .harvest_window(window)
                            .exec(db)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn update_regions(mut db: toasty::Db) -> Result<(), BackgroundError> {
    debug!("Updating regional taxon status");
    let mut page = RegionalTaxonStatus::all()
        .include(RegionalTaxonStatus::fields().taxon())
        .include(RegionalTaxonStatus::fields().region())
        .order_by(RegionalTaxonStatus::fields().updated_at().asc())
        .paginate(100)
        .exec(&mut db)
        .await?;
    loop {
        for rts in page.iter() {
            if let Err(e) = update_regional_status(&mut db, rts).await {
                warn!("Failed to query harvest info: {e}")
            }
            // for a constantly-running update thread, run slowly
            std::thread::sleep(Duration::from_secs(5));
        }
        match page.next(&mut db).await? {
            Some(next) => page = next,
            None => break,
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn background_tasks(db: toasty::Db) -> () {
    let harvest_handle = std::thread::spawn(async move || {
        loop {
            if let Err(e) = update_regions(db.clone()).await {
                warn!("{e}");
            }
            std::thread::sleep(Duration::from_secs(10));
        }
    });

    harvest_handle.join().unwrap().await;
}
