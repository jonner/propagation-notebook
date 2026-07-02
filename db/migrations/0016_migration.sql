ALTER TABLE "taxon_protocols" ADD COLUMN "citation" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "protocols" ADD COLUMN "citation" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedures" ADD COLUMN "citation" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_cleaning_procedures" ADD COLUMN "citation" TEXT;
