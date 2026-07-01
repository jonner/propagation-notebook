use libpropagation::propagation::TaxonProtocol;

use crate::style;

pub struct TaxonPropagationProtocolListView<'a> {
    tps: &'a Vec<TaxonProtocol>,
}

impl<'a> TaxonPropagationProtocolListView<'a> {
    pub fn new(tps: &'a Vec<TaxonProtocol>) -> Self {
        Self { tps }
    }
    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Taxon", "Protocol", "Confidence", "Notes"]);
        for tp in self.tps {
            tbuilder.push_record([
                &tp.taxon.get().reference(),
                &tp.protocol.get().reference(),
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
    tp: &'a TaxonProtocol,
}

impl<'a> TaxonPropagationProtocolDetailView<'a> {
    pub fn new(tp: &'a TaxonProtocol) -> Self {
        Self { tp }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record([
            "Taxon",
            &match self.tp.taxon.is_unloaded() {
                true => self.tp.taxon_id.to_string(),
                false => self.tp.taxon.get().reference(),
            },
        ]);
        tbuilder.push_record([
            "Confidence",
            self.tp
                .confidence
                .map(|v| v.to_string())
                .as_deref()
                .unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Taxon-specific notes",
            self.tp.notes.as_deref().unwrap_or("-"),
        ]);
        tbuilder.push_record([
            "Protocol",
            &match self.tp.protocol.is_unloaded() {
                true => self.tp.protocol_id.to_string(),
                false => self.tp.protocol.get().reference(),
            },
        ]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
