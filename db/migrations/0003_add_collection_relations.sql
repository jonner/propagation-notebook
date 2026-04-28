CREATE INDEX "index_cleaning_procedure_steps_by_procedure_id" ON "cleaning_procedure_steps" ("procedure_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_phenologies_by_taxon_id" ON "phenologies" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_phenologies_by_region_id" ON "phenologies" ("region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedures_by_taxon_id" ON "cleaning_procedures" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_collection_data_by_taxon_id" ON "collection_data" ("taxon_id");
