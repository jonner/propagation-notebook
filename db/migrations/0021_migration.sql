PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_native_plant_communities" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "region_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_native_plant_communities" ("id", "region_id", "name") SELECT "id", "region_id", "name" FROM "native_plant_communities";
-- #[toasty::breakpoint]
DROP TABLE "native_plant_communities";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_native_plant_communities" RENAME TO "native_plant_communities";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "bounds" TEXT,
    "notes" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regions" ("id", "name", "bounds", "notes") SELECT "id", "name", "bounds", "notes" FROM "regions";
-- #[toasty::breakpoint]
DROP TABLE "regions";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regions" RENAME TO "regions";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regional_taxon_statuses" (
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
INSERT INTO "_toasty_new_regional_taxon_statuses" ("id", "taxon_id", "region_id", "origin", "c_value", "conservation_status", "wetland_indicator", "window_start", "window_end", "native_plant_community_id") SELECT "id", "taxon_id", "region_id", "origin", "c_value", "conservation_status", "wetland_indicator", "window_start", "window_end", "native_plant_community_id" FROM "regional_taxon_statuses";
-- #[toasty::breakpoint]
DROP TABLE "regional_taxon_statuses";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regional_taxon_statuses" RENAME TO "regional_taxon_statuses";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxa" (
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
INSERT INTO "_toasty_new_taxa" ("id", "itis_id", "name1", "name2", "name3", "complete_name", "parent_id", "sequence", "rank", "life_form", "life_cycle") SELECT "id", "itis_id", "name1", "name2", "name3", "complete_name", "parent_id", "sequence", "rank", "life_form", "life_cycle" FROM "taxa";
-- #[toasty::breakpoint]
DROP TABLE "taxa";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxa" RENAME TO "taxa";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_vernacular_names" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_vernacular_names" ("id", "taxon_id", "name") SELECT "id", "taxon_id", "name" FROM "vernacular_names";
-- #[toasty::breakpoint]
DROP TABLE "vernacular_names";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_vernacular_names" RENAME TO "vernacular_names";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_synonyms" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_synonyms" ("id", "taxon_id", "name1", "name2", "name3", "complete_name") SELECT "id", "taxon_id", "name1", "name2", "name3", "complete_name" FROM "synonyms";
-- #[toasty::breakpoint]
DROP TABLE "synonyms";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_synonyms" RENAME TO "synonyms";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_collecting_data" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "ripening_indicators" TEXT NOT NULL,
    "storage" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_collecting_data" ("id", "taxon_id", "ripening_indicators", "storage") SELECT "id", "taxon_id", "ripening_indicators", "storage" FROM "collecting_data";
-- #[toasty::breakpoint]
DROP TABLE "collecting_data";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_collecting_data" RENAME TO "collecting_data";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_cleaning_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_cleaning_procedures" ("id", "name", "notes") SELECT "id", "name", "notes" FROM "cleaning_procedures";
-- #[toasty::breakpoint]
DROP TABLE "cleaning_procedures";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_cleaning_procedures" RENAME TO "cleaning_procedures";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_cleaning_procedure_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "procedure_id" INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    "operation_type" BIGINT NOT NULL,
    "equipment" TEXT,
    "notes" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_cleaning_procedure_steps" ("id", "procedure_id", "order", "operation_type", "equipment", "notes") SELECT "id", "procedure_id", "order", "operation_type", "equipment", "notes" FROM "cleaning_procedure_steps";
-- #[toasty::breakpoint]
DROP TABLE "cleaning_procedure_steps";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_cleaning_procedure_steps" RENAME TO "cleaning_procedure_steps";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
DROP INDEX "index_taxon_protocols_by_pretreatment_protocol_id";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_protocols" DROP COLUMN "pretreatment_protocol_id";
-- #[toasty::breakpoint]
DROP INDEX "index_taxon_protocols_by_germination_protocol_id";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_protocols" DROP COLUMN "germination_protocol_id";
-- #[toasty::breakpoint]
DROP INDEX "index_taxon_protocols_by_establishment_protocol_id";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_protocols" DROP COLUMN "establishment_protocol_id";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_protocols" DROP COLUMN "success_rate";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_protocols" ADD COLUMN "protocol_id" INTEGER;
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_protocol_id" ON "taxon_protocols" ("protocol_id");
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_protocols" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "instructions" TEXT NOT NULL,
    "notes" TEXT,
    "type" BIGINT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_protocols" ("id", "name", "notes", "type") SELECT "id", "name", "notes", "type" FROM "protocols";
-- #[toasty::breakpoint]
DROP TABLE "protocols";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_protocols" RENAME TO "protocols";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_name" ON "protocols" ("name");
-- #[toasty::breakpoint]
DROP TABLE "protocol_steps";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_citations" (
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
INSERT INTO "_toasty_new_citations" ("id", "type", "title", "author", "author_organization", "publication_year", "url_doi", "reliability") SELECT "id", "type", "title", "author", "author_organization", "publication_year", "url_doi", "reliability" FROM "citations";
-- #[toasty::breakpoint]
DROP TABLE "citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_citations" RENAME TO "citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
