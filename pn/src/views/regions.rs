use libpropagation::region::dto::{CompactRegion, FullRegion};

use crate::style;

pub struct RegionsListView<'a> {
    regions: &'a Vec<CompactRegion>,
}

impl<'a> RegionsListView<'a> {
    pub fn new(regions: &'a Vec<CompactRegion>) -> Self {
        Self { regions }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        if self.regions.is_empty() {
            Ok("No Regions found".to_string())
        } else {
            let mut tbuilder = tabled::builder::Builder::default();
            tbuilder.push_record(["ID", "Name", "Taxa"]);
            for region in self.regions {
                tbuilder.push_record([
                    &region.id.to_string(),
                    &region.name,
                    region
                        .n_taxa
                        .map(|n| n.to_string())
                        .as_deref()
                        .unwrap_or("-"),
                ])
            }
            Ok(tbuilder.build().with(style::ListTable).to_string())
        }
    }
}

pub struct RegionDetailsView<'a> {
    region: &'a FullRegion,
}

impl<'a> RegionDetailsView<'a> {
    pub fn new(region: &'a FullRegion) -> Self {
        Self { region }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.region.id.to_string()]);
        tbuilder.push_record(["Name", &self.region.name]);
        tbuilder.push_record(["Notes", self.region.notes.as_deref().unwrap_or("-")]);
        tbuilder.push_record([
            "Taxa",
            self.region
                .n_taxa
                .map(|n| n.to_string())
                .as_deref()
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Geometry",
            {
                self.region.geometry.as_ref().map(|v| match &v.value {
                    geojson::GeometryValue::Point { coordinates } => {
                        format!("Point: ({}, {})", coordinates[0], coordinates[1])
                    }
                    geojson::GeometryValue::LineString { coordinates } => {
                        format!("LineString: {} coordinates", coordinates.len())
                    }
                    geojson::GeometryValue::Polygon { coordinates } => {
                        format!("Polygon: {} linear rings", coordinates.len())
                    }
                    geojson::GeometryValue::MultiPoint { coordinates } => {
                        format!("MultiPoint: {} points", coordinates.len())
                    }
                    geojson::GeometryValue::MultiLineString { coordinates } => {
                        format!("MultiLineString: {} lines", coordinates.len())
                    }
                    geojson::GeometryValue::MultiPolygon { coordinates } => {
                        format!("MultiPolygon: {} polygons", coordinates.len())
                    }
                    geojson::GeometryValue::GeometryCollection { geometries } => {
                        format!("GeometryCollection: {} sub-geometries", geometries.len())
                    }
                })
            }
            .as_deref()
            .unwrap_or("-"),
        ]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
