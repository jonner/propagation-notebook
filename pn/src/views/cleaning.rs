use libpropagation::taxonomy::dto::{TaxonCleaningProcedureDetails, TaxonCleaningProcedureNoTaxon};

use crate::style;

pub struct TaxonCleaningProcedureListView<'a> {
    procedures: &'a Vec<TaxonCleaningProcedureNoTaxon>,
}

impl<'a> TaxonCleaningProcedureListView<'a> {
    pub fn new(procedures: &'a Vec<TaxonCleaningProcedureNoTaxon>) -> Self {
        Self { procedures }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Procedure", "Notes"]);
        for proc in self.procedures {
            tbuilder.push_record([
                &proc.procedure.to_string(),
                proc.notes.as_deref().unwrap_or("-"),
            ]);
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct TaxonCleaningProcedureDetailView<'a> {
    tcp: &'a TaxonCleaningProcedureDetails,
}

impl<'a> TaxonCleaningProcedureDetailView<'a> {
    pub fn new(procedure: &'a TaxonCleaningProcedureDetails) -> Self {
        Self { tcp: procedure }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Taxon", &self.tcp.taxon.to_string()]);
        tbuilder.push_record(["Procedure", &self.tcp.procedure.to_string()]);
        tbuilder.push_record(["Notes", self.tcp.notes.as_deref().unwrap_or("-")]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
