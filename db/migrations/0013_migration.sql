ALTER TABLE "vernacular_names" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "vernacular_names" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "synonyms" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "synonyms" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedures" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "cleaning_procedures" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "propagation_procedures" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "propagation_procedures" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_propagation_procedures" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_propagation_procedures" ADD COLUMN "created_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_photos" ADD COLUMN "updated_at" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "taxon_photos" ADD COLUMN "created_at" TEXT;
