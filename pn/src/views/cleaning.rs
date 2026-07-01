use libpropagation::collecting::TaxonCleaningProcedure;

use crate::style;

pub struct TaxonCleaningProcedureListView<'a> {
    procedures: &'a Vec<TaxonCleaningProcedure>,
}

impl<'a> TaxonCleaningProcedureListView<'a> {
    pub fn new(procedures: &'a Vec<TaxonCleaningProcedure>) -> Self {
        Self { procedures }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["Taxon", "Procedure", "Notes"]);
        for proc in self.procedures {
            tbuilder.push_record([
                &match proc.taxon.is_unloaded() {
                    true => proc.taxon.get().reference(),
                    false => proc.taxon_id.to_string(),
                },
                &match proc.procedure.is_unloaded() {
                    true => proc.procedure_id.to_string(),
                    false => proc.procedure.get().reference(),
                },
                proc.notes.as_deref().unwrap_or("-"),
            ]);
        }
        Ok(tbuilder.build().with(style::ListTable).to_string())
    }
}

pub struct TaxonCleaningProcedureDetailView<'a> {
    tcp: &'a TaxonCleaningProcedure,
}

impl<'a> TaxonCleaningProcedureDetailView<'a> {
    pub fn new(procedure: &'a TaxonCleaningProcedure) -> Self {
        Self { tcp: procedure }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record([
            "Taxon",
            &match self.tcp.taxon.is_unloaded() {
                true => self.tcp.taxon_id.to_string(),
                false => self.tcp.taxon.get().reference(),
            },
        ]);
        tbuilder.push_record([
            "Procedure",
            &match self.tcp.procedure.is_unloaded() {
                true => self.tcp.procedure_id.to_string(),
                false => self.tcp.procedure.get().reference(),
            },
        ]);
        tbuilder.push_record(["Notes", self.tcp.notes.as_deref().unwrap_or("-")]);
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
