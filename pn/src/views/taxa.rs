use libpropagation::{
    dto::ObjectReference,
    region::dto::RegionalTaxonStatusDetailsNoRegion,
    taxonomy::{Taxon, TaxonNote},
};

use crate::{cli::taxa::TaxonSearchResult, style, util::join_or_default};

pub struct TaxonView<'a> {
    taxon: &'a Taxon,
}

impl<'a> TaxonView<'a> {
    pub fn new(taxon: &'a Taxon) -> Self {
        Self { taxon }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.taxon.id.to_string()]);
        tbuilder.push_record(["Name", &self.taxon.complete_name]);
        tbuilder.push_record(["Rank", &self.taxon.rank.to_string()]);
        tbuilder.push_record([
            "Parent",
            &self
                .taxon
                .parent
                .get()
                .as_ref()
                .map(|p| format!("{} ({})", p.reference(), p.rank))
                .unwrap_or_else(|| "-".into()),
        ]);
        tbuilder.push_record([
            "Synonyms",
            &join_or_default(self.taxon.synonyms.get(), "-", |v| v.complete_name.clone()),
        ]);
        tbuilder.push_record([
            "Common Name(s)",
            &join_or_default(self.taxon.vernaculars.get(), "-", |v| v.name.clone()),
        ]);
        tbuilder.push_record([
            "Child taxa",
            &join_or_default(self.taxon.children.get(), "-", |t| {
                format!("{} ({})", t.reference(), t.rank)
            }),
        ]);
        tbuilder.push_record([
            "Ripening",
            self.taxon
                .collecting_data
                .get()
                .as_ref()
                .and_then(|d| d.ripening_indicators.as_deref())
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Harvesting Notes",
            self.taxon
                .collecting_data
                .get()
                .as_ref()
                .and_then(|d| d.harvesting_notes.as_deref())
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Storage Conditions",
            self.taxon
                .collecting_data
                .get()
                .as_ref()
                .and_then(|d| d.storage.as_deref())
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Storage Life",
            self.taxon
                .collecting_data
                .get()
                .as_ref()
                .and_then(|d| d.storage_life.as_deref())
                .unwrap_or("-"),
        ]);
        tbuilder.push_record(["Seed Cleaning", &{
            match self.taxon.cleaning_procedures.get() {
                procedures if procedures.is_empty() => "-".to_string(),
                procedures => {
                    let mut inner_table = tabled::builder::Builder::default();
                    inner_table.push_record(["ID", "Name"]);
                    procedures.iter().for_each(|tcp| {
                        let proc = tcp.procedure.get();
                        inner_table.push_record([&proc.id.to_string(), &proc.name]);
                    });
                    inner_table.build().with(style::DetailTable).to_string() + "\n"
                }
            }
        }]);
        tbuilder.push_record(["Propagation Protocols", &{
            match self.taxon.propagation_protocols.get() {
                tp if tp.is_empty() => "-".to_string(),
                tps => {
                    let mut inner_table = tabled::builder::Builder::default();
                    inner_table.push_record(["ID", "Name", "Type"]);
                    tps.iter().for_each(|tp| {
                        let protocol = tp.protocol.get();
                        inner_table.push_record([
                            &protocol.id.to_string(),
                            &protocol.name,
                            &protocol.r#type.to_string(),
                        ]);
                    });
                    inner_table.build().with(style::ListTable).to_string() + "\n"
                }
            }
        }]);
        tbuilder.push_record(["Regions", &{
            let regions = self.taxon.regional_statuses.get();
            if regions.is_empty() {
                "-".to_string()
            } else {
                let mut inner_table = tabled::builder::Builder::default();
                inner_table.push_record(["ID", "Name", "Origin", "Harvest Window"]);
                for rs in regions.iter() {
                    inner_table.push_record([
                        rs.region.get().id.to_string(),
                        rs.region.get().name.clone(),
                        rs.origin
                            .map(|val| val.to_string())
                            .unwrap_or_else(|| "-".into()),
                        rs.harvest_window.to_string(),
                    ]);
                }
                inner_table.build().with(style::ListTable).to_string() + "\n"
            }
        }]);
        tbuilder.push_record(["Notes", &{
            let notes = self.taxon.notes.get();
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
    notes: &'a Vec<TaxonNote>,
}

impl<'a> TaxonNotesListView<'a> {
    pub fn new(notes: &'a Vec<TaxonNote>) -> Self {
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
    note: &'a TaxonNote,
}

impl<'a> TaxonNoteDetailsView<'a> {
    pub fn new(note: &'a TaxonNote) -> Self {
        Self { note }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.note.id.to_string()]);
        tbuilder.push_record(["Text", &self.note.text]);
        tbuilder.push_record([
            "Taxon",
            &match self.note.taxon.is_unloaded() {
                true => self.note.taxon_id.to_string(),
                false => self.note.taxon.get().reference(),
            },
        ]);
        tbuilder.push_record(["Created", &self.note.created_at.to_string()]);
        tbuilder.push_record(["Updated", &self.note.updated_at.to_string()]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
