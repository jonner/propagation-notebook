CREATE TABLE "asexual_method_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "protocol_id" INTEGER NOT NULL,
    "method_type" BIGINT NOT NULL,
    "hormone_treatment" TEXT,
    "substrate_media" TEXT,
    "moise_humidity_requirement" TEXT,
    "optimal_timing" TEXT
);
-- #[toasty::breakpoint]
CREATE TABLE "protocols" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "citation_id" INTEGER NOT NULL,
    "propagation_type" BIGINT NOT NULL,
    "difficulty" BIGINT NOT NULL,
    "notes" TEXT
);
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
CREATE TABLE "collection_data" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "ripening_indicators" TEXT NOT NULL,
    "storage" TEXT
);
-- #[toasty::breakpoint]
CREATE TABLE "procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "protocol_id" INTEGER NOT NULL,
    "environment_id" INTEGER NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE TABLE "culture_environments" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
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
CREATE TABLE "sexual_method_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "protocol_id" INTEGER NOT NULL,
    "step_order" INTEGER NOT NULL,
    "treatment_type" BIGINT NOT NULL,
    "duration_days" INTEGER NOT NULL,
    "temp_day" INTEGER NOT NULL,
    "temp_night" INTEGER NOT NULL,
    "light_requirements" BIGINT NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "native_plant_communities" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "region_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "notes" TEXT
);
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
CREATE TABLE "region_statuses" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "c_value" INTEGER,
    "conservation_status" BIGINT NOT NULL,
    "wetland_indicator" BIGINT NOT NULL,
    "native_plant_community_id" INTEGER NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" INTEGER NOT NULL,
    "bound" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "vernacular_names" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "taxa" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
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
CREATE TABLE "synonyms" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE TABLE "phenologies" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "window_start" TEXT NOT NULL,
    "window_end" TEXT NOT NULL
);
