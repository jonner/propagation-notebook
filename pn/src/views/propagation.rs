use libpropagation::{
    propagation::dto::{PropagationProcedureCompact, PropagationProcedureDetails},
    taxonomy::dto::{TaxonPropagationProcedureCompact, TaxonPropagationProcedureDetails},
};

use crate::style;

pub struct TaxonPropagationProcedureListView<'a> {
    tps: &'a Vec<TaxonPropagationProcedureCompact>,
}

impl<'a> TaxonPropagationProcedureListView<'a> {
    pub fn new(tps: &'a Vec<TaxonPropagationProcedureCompact>) -> Self {
        Self { tps }
    }
    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Procedure", "Confidence", "Notes"]);
        for tp in self.tps {
            tbuilder.push_record([
                &tp.propagation.to_string(),
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

pub struct TaxonPropagationPropagationProcedureDetailView<'a> {
    tp: &'a TaxonPropagationProcedureDetails,
}

impl<'a> TaxonPropagationPropagationProcedureDetailView<'a> {
    pub fn new(tp: &'a TaxonPropagationProcedureDetails) -> Self {
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
        tbuilder.push_record(["Procedure", &self.tp.core.propagation.to_string()]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct PropagationProcedureListView<'a> {
    procedures: &'a Vec<PropagationProcedureCompact>,
}

impl<'a> PropagationProcedureListView<'a> {
    pub fn new(procedures: &'a Vec<PropagationProcedureCompact>) -> Self {
        Self { procedures }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Name", "Type"]);
        for procedure in self.procedures {
            tbuilder.push_record([
                &procedure.id.to_string(),
                &procedure.name,
                &procedure.r#type.to_string(),
            ])
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct PropagationProcedureDetailView<'a> {
    procedure: &'a PropagationProcedureDetails,
}

impl<'a> PropagationProcedureDetailView<'a> {
    pub fn new(procedure: &'a PropagationProcedureDetails) -> Self {
        Self { procedure }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.procedure.id.to_string()]);
        tbuilder.push_record(["Name", &self.procedure.name]);
        tbuilder.push_record(["Type", &self.procedure.r#type.to_string()]);
        tbuilder.push_record(["Notes", self.procedure.notes.as_deref().unwrap_or("-")]);
        tbuilder.push_record(["Instructions", &self.procedure.instructions]);
        if !self.procedure.taxa.is_empty() {
            let mut inner_table = tabled::builder::Builder::default();
            inner_table.push_record(["ID", "Name"]);
            for taxon in &self.procedure.taxa {
                inner_table.push_record([
                    &taxon.id.to_string(),
                    taxon.name.as_deref().unwrap_or_default(),
                ]);
            }
            tbuilder.push_record([
                "Taxa",
                &(inner_table.build().with(style::ListTable).to_string() + "\n"),
            ]);
        }

        if !self.procedure.citations.is_empty() {
            let mut inner_table = tabled::builder::Builder::default();
            inner_table.push_record(["ID", "Name"]);
            for citation in &self.procedure.citations {
                inner_table.push_record([
                    &citation.id.to_string(),
                    citation.name.as_deref().unwrap_or_default(),
                ]);
            }
            tbuilder.push_record([
                "Citations",
                &(inner_table.build().with(style::ListTable).to_string() + "\n"),
            ]);
        }

        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
