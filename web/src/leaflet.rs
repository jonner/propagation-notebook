use topcoat::context::Cx;
use topcoat::view::NodeViewParts;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct Map<'a> {
    pub(crate) geometry: &'a geojson::Geometry,
    pub(crate) id: String,
}

impl<'a> Map<'a> {
    pub(crate) fn new(geometry: &'a geojson::Geometry) -> Self {
        Self {
            geometry,
            id: Uuid::new_v4().to_string(),
        }
    }
}

impl<'a> NodeViewParts for Map<'a> {
    fn into_view_parts(self, _cx: &Cx, parts: &mut topcoat::view::PartsWriter<'_>) {
        parts.push_str_unescaped(
            r#"<script>
// 1. Initialize the map container without setting a view
const map = L.map('"#,
        );
        parts.push_str(&self.id);
        parts.push_str(
            r#"', {
    dragging: !L.Browser.mobile,
    tap: !L.Browser.mobile,
});

// Add the base tile layer
L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; OpenStreetMap'
}).addTo(map);

// 2. Parse the injected Rust string
const geojsonData = JSON.parse(`"#,
        );
        parts.push_str(&self.geometry.to_string());
        parts.push_str_unescaped(
            r#"`);
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
requestAnimationFrame(() => {
    map.invalidateSize();
    map.fitBounds(bounds, {
        padding: [32, 32] // Optional: Adds buffer so markers don't hit the screen edge
    });
})
} else {
    // Fallback view if the GeoJSON dataset happens to be empty
    map.setView([0, 0], 6);
}
</script>"#,
        );
    }
}
