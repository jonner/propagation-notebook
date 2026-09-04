use libpropagation::citation::dto::{CitationCompact, CitationDetails};

use crate::style;

pub struct CitationDetailsView<'a> {
    citation: &'a CitationDetails,
    show_refs: bool,
}

impl<'a> CitationDetailsView<'a> {
    pub fn new(citation: &'a CitationDetails, show_refs: bool) -> Self {
        Self {
            citation,
            show_refs,
        }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.citation.id.to_string()]);
        tbuilder.push_record(["Subject", &self.citation.subject]);
        tbuilder.push_record(["Author", self.citation.author.as_deref().unwrap_or("-")]);
        tbuilder.push_record(["URL", self.citation.url.as_deref().unwrap_or("-")]);
        tbuilder.push_record([
            "Date",
            self.citation
                .date
                .map(|d| d.to_string())
                .as_deref()
                .unwrap_or("-"),
        ]);
        if self.show_refs {
            tbuilder.push_record(["References", &format!("{} cleaning procedures\n{} propagation procedures\n{} taxon propagation procedures\n{} taxon notes", self.citation.cleaning_procedures.len(), self.citation.propagation_procedures.len(), self.citation.taxon_propagation_procedures.len(), self.citation.taxon_notes.len())]);
        }

        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct CitationListView<'a> {
    citations: &'a Vec<CitationCompact>,
}

impl<'a> CitationListView<'a> {
    pub fn new(citations: &'a Vec<CitationCompact>) -> Self {
        Self { citations }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        for citation in self.citations {
            tbuilder.push_record(["ID", "Subject", "Author", "URL", "Date"]);
            tbuilder.push_record([
                &citation.id.to_string(),
                &citation.subject,
                citation.url.as_deref().unwrap_or("-"),
            ]);
        }

        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
