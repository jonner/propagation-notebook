ALTER TABLE "cleaning_procedures" ADD COLUMN "taxon_id" INTEGER NOT NULL;
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedures_by_taxon_id" ON "cleaning_procedures" ("taxon_id");
-- #[toasty::breakpoint]
DROP TABLE "taxon_cleaning_procedures";
-- #[toasty::breakpoint]
DROP TABLE "taxon_cleaning_procedure_citations";
