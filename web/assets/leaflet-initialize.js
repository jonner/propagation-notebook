function initializeLeaflet(id, geometry) {
  // Initialize the map container without setting a view
  const map = L.map(id, {
      dragging: !L.Browser.mobile,
      tap: !L.Browser.mobile,
  });

  // Add the base tile layer
  L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '© OpenStreetMap'
  }).addTo(map);

  // Create the GeoJSON layer
  const geojsonLayer = L.geoJSON(geometry, {
      onEachFeature: function (feature, layer) {
          if (feature.properties && feature.properties.name) {
              layer.bindPopup(feature.properties.name);
          }
      }
  }).addTo(map);

  // Extract bounds and adjust map zoom/position automatically
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
  return map;
};
