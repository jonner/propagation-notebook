use libpropagation::citation::dto::CitationDetails;

use crate::style;

pub struct CitationDetailsView<'a> {
    citation: &'a CitationDetails,
}

impl<'a> CitationDetailsView<'a> {
    pub fn new(citation: &'a CitationDetails) -> Self {
        Self { citation }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        tbuilder.push_record(["ID", &self.citation.id.to_string()]);
        tbuilder.push_record(["Subject", &self.citation.subject]);
        tbuilder.push_record(["Author", self.citation.author.as_deref().unwrap_or("-")]);
        tbuilder.push_record(["URL", self.citation.url.as_deref().unwrap_or("-")]);

        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}

pub struct CitationListView<'a> {
    citations: &'a Vec<CitationDetails>,
}

impl<'a> CitationListView<'a> {
    pub fn new(citations: &'a Vec<CitationDetails>) -> Self {
        Self { citations }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        let mut tbuilder = tabled::builder::Builder::default();
        for citation in self.citations {
            tbuilder.push_record(["ID", "Subject", "Author", "URL"]);
            tbuilder.push_record([
                &citation.id.to_string(),
                &citation.subject,
                citation.author.as_deref().unwrap_or("-"),
                citation.url.as_deref().unwrap_or("-"),
            ]);
        }

        Ok(tbuilder.build().with(style::DetailTable).to_string())
    }
}
