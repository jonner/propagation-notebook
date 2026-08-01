ALTER TABLE "regional_taxon_statuses" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "regional_taxon_statuses" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedure_citations" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedure_citations" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "citations" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "citations" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "propagation_procedure_citations" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "propagation_procedure_citations" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_propagation_procedure_citations" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_propagation_procedure_citations" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_note_citations" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_note_citations" ADD COLUMN "updated_at" TEXT;
