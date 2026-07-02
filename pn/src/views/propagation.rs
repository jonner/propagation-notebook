use libpropagation::{
    propagation::dto::{ProtocolCompact, ProtocolDetails},
    taxonomy::dto::{TaxonProtocolDetails, TaxonProtocolNoTaxon},
};

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

pub struct PropagationProtocolListView<'a> {
    protocols: &'a Vec<ProtocolCompact>,
}

impl<'a> PropagationProtocolListView<'a> {
    pub fn new(protocols: &'a Vec<ProtocolCompact>) -> Self {
        Self { protocols }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Name", "Type"]);
        for protocol in self.protocols {
            tbuilder.push_record([
                &protocol.id.to_string(),
                &protocol.name,
                &protocol.r#type.to_string(),
            ])
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct PropagationProtocolDetailView<'a> {
    protocol: &'a ProtocolDetails,
}

impl<'a> PropagationProtocolDetailView<'a> {
    pub fn new(protocol: &'a ProtocolDetails) -> Self {
        Self { protocol }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.protocol.id.to_string()]);
        tbuilder.push_record(["Name", &self.protocol.name]);
        tbuilder.push_record(["Type", &self.protocol.r#type.to_string()]);
        tbuilder.push_record(["Notes", self.protocol.notes.as_deref().unwrap_or("-")]);
        tbuilder.push_record(["Instructions", &self.protocol.instructions]);
        let mut inner_table = tabled::builder::Builder::default();
        if !self.protocol.taxa.is_empty() {
            inner_table.push_record(["ID", "Name"]);
            for taxon in &self.protocol.taxa {
                inner_table.push_record([
                    &taxon.id.to_string(),
                    taxon.name.as_deref().unwrap_or_default(),
                ]);
            }
        }

        tbuilder.push_record([
            "Taxa",
            &(inner_table.build().with(style::ListTable).to_string() + "\n"),
        ]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
