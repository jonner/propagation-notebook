use libpropagation::taxonomy::dto::{TaxonProtocolDetails, TaxonProtocolNoTaxon};

use crate::style;

pub struct TaxonPropagationProtocolListView<'a> {
    tps: &'a Vec<TaxonProtocolNoTaxon>,
}

impl<'a> TaxonPropagationProtocolListView<'a> {
    pub fn new(tps: &'a Vec<TaxonProtocolNoTaxon>) -> Self {
        Self { tps }
    }
    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Protocol", "Confidence", "Notes"]);
        for tp in self.tps {
            tbuilder.push_record([
                &tp.protocol.to_string(),
                tp.confidence
                    .map(|v| v.to_string())
                    .as_deref()
                    .unwrap_or("-"),
                tp.notes.as_deref().unwrap_or("-"),
            ])
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct TaxonPropagationProtocolDetailView<'a> {
    tp: &'a TaxonProtocolDetails,
}

impl<'a> TaxonPropagationProtocolDetailView<'a> {
    pub fn new(tp: &'a TaxonProtocolDetails) -> Self {
        Self { tp }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Taxon", &self.tp.taxon.to_string()]);
        tbuilder.push_record([
            "Confidence",
            self.tp
                .core
                .confidence
                .map(|v| v.to_string())
                .as_deref()
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Taxon-specific notes",
            self.tp.core.notes.as_deref().unwrap_or("-"),
        ]);
        tbuilder.push_record(["Protocol", &self.tp.core.protocol.to_string()]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
