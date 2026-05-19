use propagation_notebook::{region::Region, taxonomy::Taxon};

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 3)]
fn bench_region_taxa_with_full_include(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| rt.block_on(async { get_region_taxa_with_full_include().await }))
}

async fn get_region_taxa_with_full_include() {
    const ID: u64 = 1;
    let mut db = toasty::Db::builder()
        .models(propagation_notebook::models())
        .connect("sqlite:./propagation-notebook.sqlite")
        .await
        .unwrap();
    let region = Region::filter_by_id(ID)
        .include(Region::fields().taxon_statuses().taxon())
        .one()
        .exec(&mut db)
        .await
        .unwrap();
    let _taxa: Vec<_> = region
        .taxon_statuses
        .get()
        .iter()
        .map(|item| item.taxon.get())
        .collect();
}

#[divan::bench(sample_count = 3)]
fn bench_region_taxa_by_id_list(bencher: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| rt.block_on(async { get_region_taxa_by_id_list().await }))
}

async fn get_region_taxa_by_id_list() {
    const ID: u64 = 1;
    let mut db = toasty::Db::builder()
        .models(propagation_notebook::models())
        .connect("sqlite:./propagation-notebook.sqlite")
        .await
        .unwrap();
    let region = Region::filter_by_id(ID)
        .include(Region::fields().taxon_statuses())
        .one()
        .exec(&mut db)
        .await
        .unwrap();
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
