ALTER TABLE "regional_taxon_statuses" RENAME COLUMN "window_start" TO "harvest_window_start";
-- #[toasty::breakpoint]
ALTER TABLE "regional_taxon_statuses" RENAME COLUMN "window_end" TO "harvest_window_end";
-- #[toasty::breakpoint]
DROP INDEX "index_collecting_data_by_taxon_id";
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_collecting_data_by_taxon_id" ON "collecting_data" ("taxon_id");
