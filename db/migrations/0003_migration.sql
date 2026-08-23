DROP TABLE "collecting_data";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_notes" ADD COLUMN "category" TEXT NOT NULL CHECK ("category" IN ('general', 'ripening', 'harvesting', 'storage'));
-- #[toasty::breakpoint]
ALTER TABLE "taxon_notes" ADD COLUMN "title" TEXT NOT NULL;
