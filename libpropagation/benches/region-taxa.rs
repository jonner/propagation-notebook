use libpropagation::{
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 2)]
fn bench_region_taxa_from_region_with_full_include(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| {
        rt.block_on(async { get_region_taxa_from_region_with_full_include().await })
    })
}

async fn get_region_taxa_from_region_with_full_include() {
    let id: u64 = std::env::var("BENCHMARK_REGION_ID")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let mut db = libpropagation::db().await.unwrap();
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses().taxon())
        .one()
        .exec(&mut db)
        .await
        .expect("Failed to query region. Set BENCHMARK_REGION_ID if necessary");
    let _taxa: Vec<_> = region
        .taxon_statuses
        .get()
        .iter()
        .map(|item| item.taxon.get())
        .collect();
}

#[divan::bench(sample_count = 20)]
fn bench_region_taxa_from_region_by_id_list(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| rt.block_on(async { get_region_taxa_from_region_by_id_list().await }))
}

async fn get_region_taxa_from_region_by_id_list() {
    let id: u64 = std::env::var("BENCHMARK_REGION_ID")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let mut db = libpropagation::db().await.unwrap();
    let region = Region::filter_by_id(id)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await
        .expect("Failed to query region. Set BENCHMARK_REGION_ID if necessary");
    let taxon_ids: Vec<_> = region
        .taxon_statuses
        .get()
        .iter()
        .map(|item| item.taxon_id)
        .collect();
    let _taxa = Taxon::filter(Taxon::fields().id().in_list(taxon_ids))
        .exec(&mut db)
        .await
        .unwrap();
}

#[divan::bench(sample_count = 20)]
fn bench_region_taxa_from_taxon(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| rt.block_on(async { get_region_taxa_from_taxon().await }))
}

async fn get_region_taxa_from_taxon() {
    let id: u64 = std::env::var("BENCHMARK_REGION_ID")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let mut db = libpropagation::db().await.unwrap();
    let _taxa = Taxon::all()
        .filter(
            Taxon::fields()
                .regional_statuses()
                .any(RegionalTaxonStatus::fields().region_id().eq(id)),
        )
        .include(Taxon::fields().regional_statuses())
        .include(Taxon::fields().regional_statuses())
        .one()
        .exec(&mut db)
        .await
        .expect("Failed to query region. Set BENCHMARK_REGION_ID if necessary");
}

#[divan::bench(sample_count = 20)]
fn bench_region_taxa_from_taxon_with_region(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| rt.block_on(async { get_region_taxa_from_taxon_with_region().await }))
}

async fn get_region_taxa_from_taxon_with_region() {
    let id: u64 = std::env::var("BENCHMARK_REGION_ID")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let mut db = libpropagation::db().await.unwrap();
    let _taxa = Taxon::all()
        .filter(
            Taxon::fields()
                .regional_statuses()
                .any(RegionalTaxonStatus::fields().region_id().eq(id)),
        )
        .include(Taxon::fields().regional_statuses().region())
        .one()
        .exec(&mut db)
        .await
        .expect("Failed to query region. Set BENCHMARK_REGION_ID if necessary");
}

#[divan::bench(sample_count = 2)]
fn bench_region_taxa_from_pivot_with_full_include(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher
        .bench_local(|| rt.block_on(async { get_region_taxa_from_pivot_with_full_include().await }))
}

async fn get_region_taxa_from_pivot_with_full_include() {
    let id: u64 = std::env::var("BENCHMARK_REGION_ID")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let mut db = libpropagation::db().await.unwrap();
    let statuses = RegionalTaxonStatus::filter_by_region_id(id)
        .include(RegionalTaxonStatus::fields().taxon())
        .include(RegionalTaxonStatus::fields().region())
        .exec(&mut db)
        .await
        .expect("Failed to query region. Set BENCHMARK_REGION_ID if necessary");
    let _taxa: Vec<_> = statuses.iter().map(|item| item.taxon.get()).collect();
}
