ALTER TABLE "regions" RENAME COLUMN "bound" TO "bounds";
-- #[toasty::breakpoint]
ALTER TABLE "region_statuses" RENAME TO "regional_taxon_statuses";
-- #[toasty::breakpoint]
ALTER TABLE "native_plant_communities" ADD COLUMN "phenology_window_end" TEXT NOT NULL;
-- #[toasty::breakpoint]
ALTER TABLE "native_plant_communities" ADD COLUMN "phenology_window_start" TEXT NOT NULL;
-- #[toasty::breakpoint]
DROP INDEX "index_region_statuses_by_id";
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_regional_taxon_statuses_by_id" ON "regional_taxon_statuses" ("id");
-- #[toasty::breakpoint]
DROP INDEX "index_region_statuses_by_taxon_id";
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_taxon_id" ON "regional_taxon_statuses" ("taxon_id");
-- #[toasty::breakpoint]
DROP INDEX "index_region_statuses_by_region_id";
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_region_id" ON "regional_taxon_statuses" ("region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_native_plant_community_id" ON "regional_taxon_statuses" ("native_plant_community_id");
-- #[toasty::breakpoint]
DROP TABLE "phenologies";
