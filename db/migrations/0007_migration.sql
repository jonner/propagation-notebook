DROP INDEX "index_cleaning_procedures_by_taxon_id";
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedures" DROP COLUMN "taxon_id";
-- #[toasty::breakpoint]
CREATE TABLE "taxon_cleaning_procedures" (
    "taxon_id" INTEGER NOT NULL,
    "procedure_id" INTEGER NOT NULL,
    PRIMARY KEY ("taxon_id", "procedure_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_cleaning_procedures_by_taxon_id" ON "taxon_cleaning_procedures" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_cleaning_procedures_by_procedure_id" ON "taxon_cleaning_procedures" ("procedure_id");
