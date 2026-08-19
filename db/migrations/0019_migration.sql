DROP INDEX "index_regional_taxon_statuses_by_native_plant_community_id";
-- #[toasty::breakpoint]
ALTER TABLE "regional_taxon_statuses" DROP COLUMN "native_plant_community_id";
-- #[toasty::breakpoint]
DROP TABLE "native_plant_communities";
