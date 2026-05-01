ALTER TABLE "regional_taxon_statuses" DROP COLUMN "id";
-- #[toasty::breakpoint]
DROP INDEX "index_regional_taxon_statuses_by_id";
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_regional_taxon_statuses_by_taxon_id_and_region_id" ON "regional_taxon_statuses" ("taxon_id", "region_id");
