use libpropagation::collecting::dto::{CleaningProcedureCompact, CleaningProcedureDetails};

use crate::style;

pub struct CleaningProcedureListView<'a> {
    procedures: &'a Vec<CleaningProcedureCompact>,
}

impl<'a> CleaningProcedureListView<'a> {
    pub fn new(procedures: &'a Vec<CleaningProcedureCompact>) -> Self {
        Self { procedures }
    }
    pub fn render(&self) -> anyhow::Result<String> {
        let nitems = self.procedures.len();
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", "Name"]);
        for item in self.procedures {
            tbuilder.push_record([&item.id.to_string(), &item.name])
        }
        Ok(format!(
            "{}\n{nitems} found",
            tbuilder.build().with(style::ListTable)
        ))
    }
}

pub struct CleaningProcedureDetailsView<'a> {
    procedure: &'a CleaningProcedureDetails,
}

impl<'a> CleaningProcedureDetailsView<'a> {
    pub fn new(procedure: &'a CleaningProcedureDetails) -> Self {
        Self { procedure }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.procedure.id.to_string()]);
        tbuilder.push_record(["Name", &self.procedure.name]);
        tbuilder.push_record(["Instructions", &self.procedure.instructions]);
        tbuilder.push_record(["Notes", self.procedure.notes.as_deref().unwrap_or("-")]);

        if !self.procedure.citations.is_empty() {
            let mut inner_table = tabled::builder::Builder::default();
            inner_table.push_record(["ID", "Name"]);
            for citation in &self.procedure.citations {
                inner_table.push_record([
                    &citation.id.to_string(),
                    citation.name.as_deref().unwrap_or_default(),
                ])
            }
            tbuilder.push_record([
                "Citations",
                &(inner_table.build().with(style::ListTable).to_string() + "\n"),
            ]);
        }
        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
