use libpropagation::{
    dto::ObjectReference,
    region::dto::RegionalTaxonStatusDetailsNoRegion,
    taxonomy::dto::{TaxonDetails, TaxonNoteDetails, TaxonNoteNoTaxon},
};

use crate::{cli::taxa::TaxonSearchResult, style, util::join_or_default};

pub struct TaxonDetailsView<'a> {
    taxon: &'a TaxonDetails,
}

impl<'a> TaxonDetailsView<'a> {
    pub fn new(taxon: &'a TaxonDetails) -> Self {
        Self { taxon }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.taxon.id.to_string()]);
        tbuilder.push_record(["Name", &self.taxon.name]);
        tbuilder.push_record(["Rank", &self.taxon.rank.to_string()]);
        tbuilder.push_record([
            "Parent",
            self.taxon
                .parent
                .as_ref()
                .map(|x| x.to_string())
                .as_deref()
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Synonyms",
            &join_or_default(&self.taxon.synonyms, "-", |v| v.clone()),
        ]);
        tbuilder.push_record([
            "Common Name(s)",
            &join_or_default(&self.taxon.common_names, "-", |v| v.clone()),
        ]);
        tbuilder.push_record([
            "Child taxa",
            &join_or_default(&self.taxon.children, "-", |t| t.to_string()),
        ]);
        tbuilder.push_record(["ITIS taxon ID", &self.taxon.itis_id.to_string()]);
        if let Some(inat_id) = self.taxon.inaturalist_id {
            tbuilder.push_record(["iNaturalist taxon ID", &inat_id.to_string()]);
        }
        if let Some(collecting_data) = self.taxon.collecting_data.as_ref() {
            tbuilder.push_record([
                "Ripening",
                collecting_data
                    .ripening_indicators
                    .as_deref()
                    .unwrap_or("-"),
            ]);
            tbuilder.push_record([
                "Harvesting Notes",
                collecting_data.harvesting_notes.as_deref().unwrap_or("-"),
            ]);
            tbuilder.push_record([
                "Storage Conditions",
                collecting_data.storage.as_deref().unwrap_or("-"),
            ]);
            tbuilder.push_record([
                "Storage Life",
                collecting_data.storage_life.as_deref().unwrap_or("-"),
            ]);
        }
        tbuilder.push_record(["Seed Cleaning", &{
            match &self.taxon.seed_cleaning {
                procedures if procedures.is_empty() => "-".to_string(),
                procedures => {
                    let mut inner_table = tabled::builder::Builder::default();
                    inner_table.push_record(["ID", "Name"]);
                    procedures.iter().for_each(|tcp| {
                        inner_table.push_record([
                            &tcp.procedure.id.to_string(),
                            tcp.procedure.name.as_deref().unwrap_or_default(),
                        ]);
                    });
                    inner_table.build().with(style::ListTable).to_string() + "\n"
                }
            }
        }]);
        tbuilder.push_record(["Propagation Procedures", &{
            match &self.taxon.propagation_procedures {
                tp if tp.is_empty() => "-".to_string(),
                tps => {
                    let mut inner_table = tabled::builder::Builder::default();
                    inner_table.push_record(["ID", "Name"]);
                    tps.iter().for_each(|tp| {
                        inner_table.push_record([
                            &tp.propagation.id.to_string(),
                            tp.propagation.name.as_deref().unwrap_or_default(),
                        ]);
                    });
                    inner_table.build().with(style::ListTable).to_string() + "\n"
                }
            }
        }]);
        tbuilder.push_record(["Regions", &{
            let regions = &self.taxon.regions;
            if regions.is_empty() {
                "-".to_string()
            } else {
                let mut inner_table = tabled::builder::Builder::default();
                inner_table.push_record(["ID", "Name", "Origin", "Harvest Window"]);
                for rs in regions.iter() {
                    inner_table.push_record([
                        &rs.region.id.to_string(),
                        rs.region.name.as_deref().unwrap_or("-"),
                        rs.core
                            .origin
                            .map(|val| val.to_string())
                            .as_deref()
                            .unwrap_or("-"),
                        &rs.core.harvest_window.to_string(),
                    ]);
                }
                inner_table.build().with(style::ListTable).to_string() + "\n"
            }
        }]);
        tbuilder.push_record(["Notes", &{
            let notes = &self.taxon.notes;
            if notes.is_empty() {
                "-".to_string()
            } else {
                let mut inner_table = tabled::builder::Builder::default();
                inner_table.push_record(["ID", "Text"]);
                for note in notes.iter() {
                    inner_table.push_record([&note.id.to_string(), &note.text]);
                }
                inner_table.build().with(style::ListTable).to_string() + "\n"
            }
        }]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct RegionalTaxaListView<'a> {
    statuses: &'a Vec<RegionalTaxonStatusDetailsNoRegion>,
}

impl<'a> RegionalTaxaListView<'a> {
    pub fn new(statuses: &'a Vec<RegionalTaxonStatusDetailsNoRegion>) -> Self {
        Self { statuses }
    }

    pub fn render(&self) -> Result<String, anyhow::Error> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record([
            "ID",
            "Taxon",
            "Origin",
            "Status",
            "C-value",
            "Wetland Indicator",
        ]);
        for status in self.statuses {
            tbuilder.push_record([
                &status.taxon.id.to_string(),
                status.taxon.name.as_deref().unwrap_or("-"),
                status
                    .core
                    .origin
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or("-"),
                status
                    .core
                    .conservation_status
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or("-"),
                status
                    .core
                    .c_value
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or("-"),
                status
                    .core
                    .wetland_indicator
                    .map(|s| s.to_string())
                    .as_deref()
                    .unwrap_or("-"),
            ]);
        }
        Ok(format!(
            "{}\n{} taxa",
            tbuilder.build().with(style::ListTable),
            self.statuses.len()
        ))
    }
}

pub struct TaxaListView<'a> {
    taxa: &'a Vec<ObjectReference>,
}

impl<'a> TaxaListView<'a> {
    pub fn new(taxa: &'a Vec<ObjectReference>) -> Self {
        Self { taxa }
    }

    pub fn render(&self) -> Result<String, anyhow::Error> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Taxon"]);
        for taxon in self.taxa {
            tbuilder.push_record([
                &taxon.id.to_string(),
                taxon.name.as_deref().unwrap_or_default(),
            ]);
        }
        let ntaxa = self.taxa.len();
        Ok(format!(
            "{}\n{ntaxa} taxa found",
            tbuilder.build().with(style::ListTable)
        ))
    }
}

pub struct TaxaSearchResultsView<'a> {
    results: &'a Vec<TaxonSearchResult>,
}

impl<'a> TaxaSearchResultsView<'a> {
    pub fn new(results: &'a Vec<TaxonSearchResult>) -> Self {
        Self { results }
    }

    pub fn render(&self) -> Result<String, anyhow::Error> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Name", "Common Names", "Synonym"]);
        for result in self.results {
            tbuilder.push_record([
                &result.id.to_string(),
                &result.name,
                &result.common_names.join("\n"),
                &result.synonyms.join("\n"),
            ]);
        }
        let ntaxa = self.results.len();
        Ok(format!(
            "{}\n{ntaxa} taxa found",
            tbuilder.build().with(style::ListTable)
        ))
    }
}

pub struct TaxonNotesListView<'a> {
    notes: &'a Vec<TaxonNoteNoTaxon>,
}

impl<'a> TaxonNotesListView<'a> {
    pub fn new(notes: &'a Vec<TaxonNoteNoTaxon>) -> Self {
        Self { notes }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Note"]);
        for note in self.notes {
            tbuilder.push_record([&note.id.to_string(), &note.text]);
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct TaxonNoteDetailsView<'a> {
    note: &'a TaxonNoteDetails,
}

impl<'a> TaxonNoteDetailsView<'a> {
    pub fn new(note: &'a TaxonNoteDetails) -> Self {
        Self { note }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.note.core.id.to_string()]);
        tbuilder.push_record(["Text", &self.note.core.text]);
        tbuilder.push_record(["Taxon", &self.note.taxon.to_string()]);
        tbuilder.push_record(["Created", &self.note.core.created_at.to_string()]);
        tbuilder.push_record(["Updated", &self.note.core.updated_at.to_string()]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
