ALTER TABLE "collection_data" RENAME TO "collecting_data";
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_collecting_data_by_id" ON "collecting_data" ("id");
-- #[toasty::breakpoint]
DROP INDEX "index_collection_data_by_taxon_id";
-- #[toasty::breakpoint]
CREATE INDEX "index_collecting_data_by_taxon_id" ON "collecting_data" ("taxon_id");
