ALTER TABLE "taxa" ADD COLUMN "inaturalist_id" INTEGER;
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_inaturalist_id" ON "taxa" ("inaturalist_id");
