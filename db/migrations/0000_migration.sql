CREATE TABLE "taxon_note_citations" (
    "citation_id" INTEGER NOT NULL,
    "note_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "note_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_note_citations_by_citation_id" ON "taxon_note_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_note_citations_by_note_id" ON "taxon_note_citations" ("note_id");
-- #[toasty::breakpoint]
CREATE TABLE "citations" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "title" TEXT NOT NULL,
    "url" TEXT,
    "author" TEXT,
    "date" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "vernacular_names" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_vernacular_names_by_taxon_id" ON "vernacular_names" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "geometry" TEXT,
    "notes" TEXT,
    "category" TEXT NOT NULL CHECK ("category" IN ('nation', 'province', 'county', 'municipality', 'other')),
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_regions_by_name" ON "regions" ("name");
-- #[toasty::breakpoint]
CREATE INDEX "index_regions_by_category" ON "regions" ("category");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_notes" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "text" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_notes_by_taxon_id" ON "taxon_notes" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "regional_taxon_statuses" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "origin" BIGINT,
    "c_value" INTEGER,
    "conservation_status" BIGINT,
    "wetland_indicator" BIGINT,
    "harvest_window_start_doy" SMALLINT,
    "harvest_window_end_doy" SMALLINT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_taxon_id_and_region_id" ON "regional_taxon_statuses" ("taxon_id", "region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_taxon_id" ON "regional_taxon_statuses" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_region_id" ON "regional_taxon_statuses" ("region_id");
-- #[toasty::breakpoint]
CREATE TABLE "propagation_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "propagation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_procedure_citations_by_citation_id" ON "propagation_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_procedure_citations_by_propagation_id" ON "propagation_procedure_citations" ("propagation_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxa" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "itis_id" INTEGER NOT NULL,
    "inaturalist_id" INTEGER,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL,
    "parent_id" INTEGER,
    "sequence" INTEGER NOT NULL,
    "rank" BIGINT NOT NULL,
    "hybrid" BOOLEAN NOT NULL,
    "life_form" BIGINT,
    "life_cycle" BIGINT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_itis_id" ON "taxa" ("itis_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_inaturalist_id" ON "taxa" ("inaturalist_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_name1" ON "taxa" ("name1");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_name2" ON "taxa" ("name2");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_name3" ON "taxa" ("name3");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_complete_name" ON "taxa" ("complete_name");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_parent_id" ON "taxa" ("parent_id");
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_taxa_by_sequence" ON "taxa" ("sequence");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_hierarchies" (
    "ancestor_id" INTEGER NOT NULL,
    "descendant_id" INTEGER NOT NULL,
    "depth" INTEGER NOT NULL,
    PRIMARY KEY ("ancestor_id", "descendant_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_hierarchies_by_ancestor_id" ON "taxon_hierarchies" ("ancestor_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_hierarchies_by_descendant_id" ON "taxon_hierarchies" ("descendant_id");
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "notes" TEXT,
    "instructions" TEXT NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedures_by_taxon_id" ON "cleaning_procedures" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "propagation_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "instructions" TEXT NOT NULL,
    "notes" TEXT,
    "type" BIGINT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_procedures_by_name" ON "propagation_procedures" ("name");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_propagation_procedures" (
    "taxon_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "confidence" INTEGER,
    "notes" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("taxon_id", "propagation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_propagation_procedures_by_taxon_id" ON "taxon_propagation_procedures" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_propagation_procedures_by_propagation_id" ON "taxon_propagation_procedures" ("propagation_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_photos" (
    "taxon_id" INTEGER NOT NULL,
    "square_url" TEXT,
    "medium_url" TEXT,
    "large_url" TEXT,
    "is_default" BOOLEAN NOT NULL,
    "attribution" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("taxon_id")
);
-- #[toasty::breakpoint]
CREATE TABLE "collecting_data" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "ripening_indicators" TEXT,
    "harvesting_notes" TEXT,
    "storage" TEXT,
    "storage_life" TEXT
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_collecting_data_by_taxon_id" ON "collecting_data" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_propagation_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "propagation_id", "taxon_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_propagation_procedure_citations_by_taxon_id_and_propagation_id" ON "taxon_propagation_procedure_citations" ("taxon_id", "propagation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_propagation_procedure_citations_by_citation_id" ON "taxon_propagation_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "cleaning_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "cleaning_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedure_citations_by_citation_id" ON "cleaning_procedure_citations" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedure_citations_by_cleaning_id" ON "cleaning_procedure_citations" ("cleaning_id");
-- #[toasty::breakpoint]
CREATE TABLE "synonyms" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_taxon_id" ON "synonyms" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_name1" ON "synonyms" ("name1");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_name2" ON "synonyms" ("name2");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_name3" ON "synonyms" ("name3");
-- #[toasty::breakpoint]
CREATE INDEX "index_synonyms_by_complete_name" ON "synonyms" ("complete_name");
