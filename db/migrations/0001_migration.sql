DROP INDEX "index_collecting_data_by_taxon_id";
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_collecting_data_by_taxon_id" ON "collecting_data" ("taxon_id");
