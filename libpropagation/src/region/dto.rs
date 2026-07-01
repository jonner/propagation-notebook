use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::dto::ObjectReference;

#[derive(Debug, Serialize)]
pub struct CompactRegion {
    pub id: u64,
    pub name: String,
    pub n_taxa: Option<usize>,
}

impl From<super::Region> for CompactRegion {
    fn from(region: super::Region) -> Self {
        Self {
            id: region.id,
            name: region.name,
            n_taxa: match region.taxon_statuses.is_unloaded() {
                true => None,
                false => Some(region.taxon_statuses.get().len()),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FullRegion {
    pub id: u64,
    pub name: String,
    pub notes: Option<String>,
    pub n_taxa: Option<usize>,
    pub geometry: Option<geojson::Geometry>,
}

impl From<super::Region> for FullRegion {
    fn from(region: super::Region) -> Self {
        Self {
            id: region.id,
            name: region.name,
            notes: region.notes,
            geometry: region.geometry.map(|inner| inner.0),
            n_taxa: match region.taxon_statuses.is_unloaded() {
                true => None,
                false => Some(region.taxon_statuses.get().len()),
            },
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct RegionalTaxonStatusDetails {
    pub taxon: ObjectReference,
    pub region: ObjectReference,
    #[serde(flatten)]
    pub core: RegionalTaxonStatusDetailsCore,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct RegionalTaxonStatusDetailsNoRegion {
    pub taxon: ObjectReference,
    #[serde(flatten)]
    pub core: RegionalTaxonStatusDetailsCore,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct RegionalTaxonStatusDetailsNoTaxon {
    pub region: ObjectReference,
    #[serde(flatten)]
    pub core: RegionalTaxonStatusDetailsCore,
}

impl From<&super::RegionalTaxonStatus> for RegionalTaxonStatusDetailsNoTaxon {
    fn from(value: &super::RegionalTaxonStatus) -> Self {
        value.clone().into()
    }
}

impl From<super::RegionalTaxonStatus> for RegionalTaxonStatusDetailsNoTaxon {
    fn from(value: super::RegionalTaxonStatus) -> Self {
        Self {
            region: ObjectReference::from_deferred(value.region, value.region_id),
            core: RegionalTaxonStatusDetailsCore {
                origin: value.origin,
                c_value: value.c_value,
                conservation_status: value.conservation_status,
                wetland_indicator: value.wetland_indicator,
                harvest_window: value.harvest_window.clone(),
            },
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct RegionalTaxonStatusDetailsCore {
    pub origin: Option<super::Origin>,
    pub c_value: Option<u64>,
    pub conservation_status: Option<super::ConservationStatus>,
    pub wetland_indicator: Option<super::WetlandIndicator>,
    #[serde(skip_serializing_if = "super::RegionalHarvestWindow::is_empty")]
    pub harvest_window: super::RegionalHarvestWindow,
}

impl RegionalTaxonStatusDetailsNoRegion {
    pub fn from_taxa(taxa: Vec<super::Taxon>, region_id: u64) -> Vec<Self> {
        taxa.into_iter()
            .filter_map(|taxon| {
                taxon
                    .regional_statuses
                    .get()
                    .iter()
                    .find(|rts| rts.region_id == region_id)
                    .map(|rts| Self {
                        taxon: taxon.clone().into(),
                        core: RegionalTaxonStatusDetailsCore {
                            origin: rts.origin,
                            c_value: rts.c_value,
                            conservation_status: rts.conservation_status,
                            wetland_indicator: rts.wetland_indicator,
                            harvest_window: rts.harvest_window.clone(),
                        },
                    })
            })
            .collect()
    }
}

impl From<super::RegionalTaxonStatus> for RegionalTaxonStatusDetails {
    fn from(value: super::RegionalTaxonStatus) -> Self {
        Self {
            taxon: value.taxon.get().into(),
            region: value.region.get().into(),
            core: RegionalTaxonStatusDetailsCore {
                origin: value.origin,
                c_value: value.c_value,
                conservation_status: value.conservation_status,
                wetland_indicator: value.wetland_indicator,
                harvest_window: value.harvest_window,
            },
        }
    }
}

#[derive(Serialize)]
pub struct RegionalTaxonHarvestInfo {
    pub taxon: ObjectReference,
    pub region: ObjectReference,
    pub harvest_window: super::RegionalHarvestWindow,
}

impl From<super::RegionalTaxonStatus> for RegionalTaxonHarvestInfo {
    fn from(rts: super::RegionalTaxonStatus) -> Self {
        Self {
            taxon: rts.taxon.get().into(),
            region: rts.region.get().into(),
            harvest_window: rts.harvest_window,
        }
    }
}
