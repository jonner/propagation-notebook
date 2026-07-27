use super::util::Dimension;
use topcoat::context::Cx;
use topcoat::view::NodeViewParts;

#[derive(Debug)]
pub(crate) struct Map<'a> {
    pub(crate) geometry: &'a geojson::Geometry,
    pub(crate) width: Option<Dimension>,
    pub(crate) height: Option<Dimension>,
}

impl<'a> NodeViewParts for Map<'a> {
    fn into_view_parts(self, _cx: &Cx, parts: &mut topcoat::view::PartsWriter<'_>) {
        parts.push_str_unescaped("<div id=\"map\" style=\"height: ");
        parts.push_str(self.height.unwrap_or(Dimension::Pixels(600)).to_string());
        parts.push_str("; width: ");
        parts.push_str(self.width.unwrap_or(Dimension::Percent(100)).to_string());
        parts.push_str_unescaped(
            r#";"></div>
            <script>
            // 1. Initialize the map container without setting a view
            const map = L.map('map');

            // Add the base tile layer
            L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
                maxZoom: 19,
                attribution: '&copy; OpenStreetMap'
            }).addTo(map);

            // 2. Parse the injected Rust string
            const geojsonData = JSON.parse(`"#,
        );
        parts.push_str(self.geometry.to_string());
        parts.push_str_unescaped(r#"`);
            // 3. Create the GeoJSON layer
            const geojsonLayer = L.geoJSON(geojsonData, {
                onEachFeature: function (feature, layer) {
                    if (feature.properties && feature.properties.name) {
                        layer.bindPopup(feature.properties.name);
                    }
                }
            }).addTo(map);

            // 4. Extract bounds and adjust map zoom/position automatically
            const bounds = geojsonLayer.getBounds();
            if (bounds.isValid()) {
                map.fitBounds(bounds, {
                    padding: [50, 50] // Optional: Adds 50px buffer so markers don't hit the screen edge
                });
            } else {
                // Fallback view if the GeoJSON dataset happens to be empty
                map.setView([0, 0], 6);
            }
        </script>"#);
    }
}
