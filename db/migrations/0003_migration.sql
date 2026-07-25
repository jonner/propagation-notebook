CREATE TABLE "taxon_note_citations" (
    "citation_id" INTEGER NOT NULL,
    "note_id" INTEGER NOT NULL,
    PRIMARY KEY ("citation_id", "note_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_note_citations_by_citation_id" ON "taxon_note_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_note_citations_by_note_id" ON "taxon_note_citations" ("note_id");
