CREATE INDEX "index_vernacular_names_by_taxon_id" ON "vernacular_names" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_name2" ON "taxa" ("name2");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_name3" ON "taxa" ("name3");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_complete_name" ON "taxa" ("complete_name");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_name1" ON "taxa" ("name1");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_parent_id" ON "taxa" ("parent_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_taxon_id" ON "synonyms" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_name3" ON "synonyms" ("name3");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_name1" ON "synonyms" ("name1");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_name2" ON "synonyms" ("name2");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_complete_name" ON "synonyms" ("complete_name");
