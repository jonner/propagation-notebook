CREATE TABLE "culture_environments" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "sowing_depth" INTEGER NOT NULL,
    "depth_description" TEXT,
    "media_type" TEXT,
    "compaction_level" TEXT,
    "moisture_regime" TEXT,
    "container_type" TEXT,
    "is_experimental" BOOLEAN NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedures_by_taxon_id" ON "cleaning_procedures" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "bounds" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_regions_by_name" ON "regions" ("name");
-- #[toasty::breakpoint]
CREATE TABLE "collection_data" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "ripening_indicators" TEXT NOT NULL,
    "storage" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_collection_data_by_taxon_id" ON "collection_data" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "protocols" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "protocol_id" INTEGER NOT NULL,
    "environment_id" INTEGER NOT NULL,
    "citation_id" INTEGER NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_taxon_id" ON "protocols" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_protocol_id" ON "protocols" ("protocol_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_environment_id" ON "protocols" ("environment_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_citation_id" ON "protocols" ("citation_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxa" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "itis_id" INTEGER NOT NULL,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL,
    "parent_id" INTEGER,
    "rank" BIGINT NOT NULL,
    "life_form" BIGINT,
    "life_cycle" BIGINT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxa_by_itis_id" ON "taxa" ("itis_id");
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
CREATE TABLE "asexual_method_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "procedure_id" INTEGER NOT NULL,
    "method_type" BIGINT NOT NULL,
    "hormone_treatment" TEXT,
    "substrate_media" TEXT,
    "moise_humidity_requirement" TEXT,
    "optimal_timing" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_asexual_method_steps_by_procedure_id" ON "asexual_method_steps" ("procedure_id");
-- #[toasty::breakpoint]
CREATE TABLE "native_plant_communities" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "region_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_native_plant_communities_by_region_id" ON "native_plant_communities" ("region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_native_plant_communities_by_name" ON "native_plant_communities" ("name");
-- #[toasty::breakpoint]
CREATE TABLE "regional_taxon_statuses" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "c_value" INTEGER,
    "conservation_status" BIGINT NOT NULL,
    "wetland_indicator" BIGINT NOT NULL,
    "phenology_window_start" TEXT NOT NULL,
    "phenology_window_end" TEXT NOT NULL,
    "native_plant_community_id" INTEGER NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_taxon_id" ON "regional_taxon_statuses" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_region_id" ON "regional_taxon_statuses" ("region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_native_plant_community_id" ON "regional_taxon_statuses" ("native_plant_community_id");
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedure_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "procedure_id" INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    "cleaning_type" BIGINT NOT NULL,
    "equipment" TEXT NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedure_steps_by_procedure_id" ON "cleaning_procedure_steps" ("procedure_id");
-- #[toasty::breakpoint]
CREATE TABLE "citations" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "type" BIGINT NOT NULL,
    "title" TEXT NOT NULL,
    "author" TEXT NOT NULL,
    "author_organization" TEXT,
    "publication_year" INTEGER,
    "url_doi" TEXT,
    "reliability" INTEGER
);
-- #[toasty::breakpoint]
CREATE TABLE "synonyms" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL
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
-- #[toasty::breakpoint]
CREATE TABLE "procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "propagation_type" BIGINT NOT NULL,
    "difficulty" BIGINT NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE TABLE "vernacular_names" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_vernacular_names_by_taxon_id" ON "vernacular_names" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "sexual_method_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "procedure_id" INTEGER NOT NULL,
    "step_order" INTEGER NOT NULL,
    "treatment_type" BIGINT NOT NULL,
    "duration_days" INTEGER NOT NULL,
    "temp_day" INTEGER NOT NULL,
    "temp_night" INTEGER NOT NULL,
    "light_requirements" BIGINT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_sexual_method_steps_by_procedure_id" ON "sexual_method_steps" ("procedure_id");
