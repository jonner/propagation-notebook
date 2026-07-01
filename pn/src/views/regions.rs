use libpropagation::region::{
    RegionalTaxonStatus,
    dto::{CompactRegion, FullRegion, RegionalTaxonHarvestInfo},
};

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

pub struct RegionalTaxonStatusDetailsView<'a> {
    status: &'a RegionalTaxonStatus,
}

impl<'a> RegionalTaxonStatusDetailsView<'a> {
    pub fn new(status: &'a RegionalTaxonStatus) -> Self {
        Self { status }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record([
            "Taxon",
            &match self.status.taxon.is_unloaded() {
                true => self.status.taxon_id.to_string(),
                false => self.status.taxon.get().reference(),
            },
        ]);
        tbuilder.push_record([
            "Region",
            &match self.status.region.is_unloaded() {
                true => self.status.region_id.to_string(),
                false => self.status.region.get().reference(),
            },
        ]);
        tbuilder.push_record([
            "Origin",
            &self
                .status
                .origin
                .unwrap_or(libpropagation::region::Origin::Unknown)
                .to_string(),
        ]);
        tbuilder.push_record([
            "C-value",
            &self
                .status
                .c_value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
        tbuilder.push_record([
            "Conservation Status",
            &self
                .status
                .conservation_status
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
        tbuilder.push_record([
            "Wetland Indicator",
            &self
                .status
                .wetland_indicator
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
        tbuilder.push_record(["Harvest Window", &self.status.harvest_window.to_string()]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct RegionalHarvestDateListView<'a> {
    regional_taxa: &'a Vec<RegionalTaxonHarvestInfo>,
}

impl<'a> RegionalHarvestDateListView<'a> {
    pub fn new(regional_taxa: &'a Vec<RegionalTaxonHarvestInfo>) -> Self {
        Self { regional_taxa }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Region", "Taxon", "Harvest Dates"]);
        for regional_taxon in self.regional_taxa {
            tbuilder.push_record([
                regional_taxon.region.to_string(),
                regional_taxon.taxon.to_string(),
                regional_taxon.harvest_window.to_string(),
            ])
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}
