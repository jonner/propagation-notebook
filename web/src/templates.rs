use libpropagation::{
    propagation::PropagationProcedure,
    region::{Region, RegionalTaxonStatus},
    taxonomy::Taxon,
};
use maud::{DOCTYPE, Markup, html};

pub mod pages;

pub enum Dimension {
    Pixels(u64),
    Percent(u64),
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Pixels(v) => write!(f, "{v}px"),
            Dimension::Percent(v) => write!(f, "{v}%"),
        }
    }
}

pub fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=" crossorigin="" {}
        script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js" integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=" crossorigin="" {}
        title { (page_title) }
    }
}

pub fn map(
    geometry: &geojson::Geometry,
    width: Option<Dimension>,
    height: Option<Dimension>,
) -> Markup {
    html! {
            div id="map" style={"height: " (height.unwrap_or(Dimension::Pixels(600))) "; width: " (width.unwrap_or(Dimension::Percent(100))) "px;"} {}
            script {
                    (maud::PreEscaped(format!(
                        r#"
                    // 1. Initialize the map container without setting a view
                    const map = L.map('map');

                    // Add the base tile layer
                    L.tileLayer('https://tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
                        maxZoom: 19,
                        attribution: '&copy; OpenStreetMap'
                    }}).addTo(map);

                    // 2. Parse the injected Rust string
                    const geojsonData = JSON.parse(`{geojson_data}`);

                    // 3. Create the GeoJSON layer
                    const geojsonLayer = L.geoJSON(geojsonData, {{
                        onEachFeature: function (feature, layer) {{
                            if (feature.properties && feature.properties.name) {{
                                layer.bindPopup(feature.properties.name);
                            }}
                        }}
                    }}).addTo(map);

                    // 4. Extract bounds and adjust map zoom/position automatically
                    const bounds = geojsonLayer.getBounds();
                    if (bounds.isValid()) {{
                        map.fitBounds(bounds, {{
                            padding: [50, 50] // Optional: Adds 50px buffer so markers don't hit the screen edge
                        }});
                    }} else {{
                        // Fallback view if the GeoJSON dataset happens to be empty
                        map.setView([0, 0], 6);
                    }}
                    "#,
                        geojson_data = geometry
                    )))
            }
    }
}

pub trait Path {
    fn path(&self) -> String;
}

impl Path for PropagationProcedure {
    fn path(&self) -> String {
        format!("/propagation/{}", self.id)
    }
}

impl Path for Taxon {
    fn path(&self) -> String {
        format!("/taxa/{}", self.id)
    }
}

impl Path for Region {
    fn path(&self) -> String {
        format!("/regions/{}", self.id)
    }
}

impl Path for RegionalTaxonStatus {
    fn path(&self) -> String {
        format!("/regions/{}/taxa/{}", self.region_id, self.taxon_id)
    }
}
