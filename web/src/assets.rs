use topcoat::{
    asset::{Asset, asset},
    font::{Font, fontsource::fontsource_font},
    icon::iconify,
};

iconify::include!(pub(crate) "mdi");
pub(crate) const FONT_HEAD: Font = fontsource_font!(AVERIA_SERIF_LIBRE, host: Asset);
pub(crate) const FONT_BODY: Font = fontsource_font!(AVERIA_SANS_LIBRE, host: Asset);

// All images from iNaturalist licensed in the public domain
pub(crate) const HEADER_IMAGES: &[Asset] = &[
    // https://www.inaturalist.org/photos/210648286
    asset!("assets/cypripedium-reginae.webp"),
    // https://www.inaturalist.org/photos/12597071
    asset!("assets/asclepias-incarnata.webp"),
    // https://www.inaturalist.org/photos/147597352
    asset!("assets/empetrum-nigrum.webp"),
    // https://www.inaturalist.org/photos/135132632
    asset!("assets/escobaria-vivipara.webp"),
    // https://www.inaturalist.org/photos/102029239
    asset!("assets/hamamelis-virginiana.webp"),
    // https://www.inaturalist.org/photos/531026295
    asset!("assets/desmanthus-illinoensis.webp"),
    // https://www.inaturalist.org/photos/690459895
    asset!("assets/hydrastis-canadensis.webp"),
];

pub(crate) const LEAFLET_JS: Asset = asset!("https://unpkg.com/leaflet@1.9.4/dist/leaflet.js", checksum:"sha256:db49d009c841f5ca34a888c96511ae936fd9f5533e90d8b2c4d57596f4e5641a");
pub(crate) const LEAFLET_CSS: Asset = asset!("https://unpkg.com/leaflet@1.9.4/dist/leaflet.css", checksum:"sha256:a7837102824184820dfa198d1ebcd109ff6d0ff9a2672a074b9a1b4d147d04c6");
