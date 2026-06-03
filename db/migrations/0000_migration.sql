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
CREATE TABLE "cleaning_procedure_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "procedure_id" INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    "operation_type" BIGINT NOT NULL,
    "equipment" TEXT,
    "notes" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_cleaning_procedure_steps_by_procedure_id" ON "cleaning_procedure_steps" ("procedure_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_protocols" (
    "id" INTEGER NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "protocol_id" INTEGER,
    "confidence" INTEGER,
    "notes" TEXT,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_taxon_id" ON "taxon_protocols" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_protocol_id" ON "taxon_protocols" ("protocol_id");
-- #[toasty::breakpoint]
CREATE TABLE "protocols" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "instructions" TEXT NOT NULL,
    "notes" TEXT,
    "type" BIGINT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_name" ON "protocols" ("name");
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
CREATE TABLE "regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "bounds" TEXT,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_regions_by_name" ON "regions" ("name");
-- #[toasty::breakpoint]
CREATE TABLE "regional_taxon_statuses" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "origin" BIGINT,
    "c_value" INTEGER,
    "conservation_status" BIGINT,
    "wetland_indicator" BIGINT,
    "window_start" TEXT,
    "window_end" TEXT,
    "native_plant_community_id" INTEGER
);
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_taxon_id_and_region_id" ON "regional_taxon_statuses" ("taxon_id", "region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_taxon_id" ON "regional_taxon_statuses" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_region_id" ON "regional_taxon_statuses" ("region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_regional_taxon_statuses_by_native_plant_community_id" ON "regional_taxon_statuses" ("native_plant_community_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxa" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "itis_id" INTEGER NOT NULL,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL,
    "parent_id" INTEGER,
    "sequence" INTEGER NOT NULL,
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
CREATE UNIQUE INDEX "index_taxa_by_sequence" ON "taxa" ("sequence");
-- #[toasty::breakpoint]
CREATE TABLE "collecting_data" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "ripening_indicators" TEXT NOT NULL,
    "storage" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_collecting_data_by_taxon_id" ON "collecting_data" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "vernacular_names" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_vernacular_names_by_taxon_id" ON "vernacular_names" ("taxon_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_cleaning_procedures" (
    "taxon_id" INTEGER NOT NULL,
    "notes" TEXT,
    "procedure_id" INTEGER NOT NULL,
    PRIMARY KEY ("taxon_id", "procedure_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_cleaning_procedures_by_taxon_id" ON "taxon_cleaning_procedures" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_cleaning_procedures_by_procedure_id" ON "taxon_cleaning_procedures" ("procedure_id");
-- #[toasty::breakpoint]
CREATE TABLE "cleaning_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE TABLE "protocol_citations" (
    "protocol_id" INTEGER NOT NULL,
    "citation_id" INTEGER NOT NULL,
    PRIMARY KEY ("protocol_id", "citation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_protocol_citations_by_protocol_id" ON "protocol_citations" ("protocol_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_protocol_citations" (
    "id" INTEGER NOT NULL,
    "taxon_protocol_id" INTEGER NOT NULL,
    "citation_id" INTEGER NOT NULL,
    PRIMARY KEY ("id", "citation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocol_citations_by_taxon_protocol_id" ON "taxon_protocol_citations" ("taxon_protocol_id");
