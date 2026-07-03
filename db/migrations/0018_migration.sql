ALTER TABLE "propagation_procedures" DROP COLUMN "citation";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_propagation_procedures" DROP COLUMN "citation";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_cleaning_procedures" DROP COLUMN "citation";
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedures" DROP COLUMN "citation";
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "cleaning_id" INTEGER NOT NULL,
    PRIMARY KEY ("citation_id", "cleaning_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedure_citations_by_citation_id" ON "cleaning_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedure_citations_by_cleaning_id" ON "cleaning_procedure_citations" ("cleaning_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_propagation_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    PRIMARY KEY ("citation_id", "propagation_id", "taxon_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_propagation_procedure_citations_by_taxon_id_and_propagation_id" ON "taxon_propagation_procedure_citations" ("taxon_id", "propagation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_propagation_procedure_citations_by_citation_id" ON "taxon_propagation_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_cleaning_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "procedure_id" INTEGER NOT NULL,
    PRIMARY KEY ("citation_id", "taxon_id", "procedure_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_cleaning_procedure_citations_by_taxon_id_and_procedure_id" ON "taxon_cleaning_procedure_citations" ("taxon_id", "procedure_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_cleaning_procedure_citations_by_citation_id" ON "taxon_cleaning_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE TABLE "propagation_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    PRIMARY KEY ("citation_id", "propagation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_procedure_citations_by_citation_id" ON "propagation_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_procedure_citations_by_propagation_id" ON "propagation_procedure_citations" ("propagation_id");
-- #[toasty::breakpoint]
CREATE TABLE "citations" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "text" TEXT NOT NULL,
    "url" TEXT,
    "author" TEXT
);
