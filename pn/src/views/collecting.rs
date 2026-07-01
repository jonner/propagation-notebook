use libpropagation::taxonomy::dto::CollectingDataDetails;
use tabled::builder::Builder;

pub struct CollectingDataView<'a> {
    data: &'a CollectingDataDetails,
}

impl<'a> CollectingDataView<'a> {
    pub fn new(data: &'a CollectingDataDetails) -> Self {
        Self { data }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = Builder::default();
        tbuilder.push_record(["Taxon", &self.data.taxon.to_string()]);
        tbuilder.push_record([
            "Ripening",
            self.data.ripening_indicators.as_deref().unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Harvesting",
            self.data.harvesting_notes.as_deref().unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Storage Conditions",
            self.data.storage.as_deref().unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Storage Life",
            self.data.storage_life.as_deref().unwrap_or("-"),
        ]);
        Ok(tbuilder.build().with(crate::style::DetailTable).to_string())
    }
}
