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
    "harvest_window_start_doy" SMALLINT,
    "harvest_window_end_doy" SMALLINT,
    "native_plant_community_id" INTEGER,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regional_taxon_statuses" ("id", "taxon_id", "region_id", "origin", "c_value", "conservation_status", "wetland_indicator", "harvest_window_start_doy", "harvest_window_end_doy", "native_plant_community_id", "created_at", "updated_at") SELECT "id", "taxon_id", "region_id", "origin", "c_value", "conservation_status", "wetland_indicator", "harvest_window_start_doy", "harvest_window_end_doy", "native_plant_community_id", "created_at", "updated_at" FROM "regional_taxon_statuses";
-- #[toasty::breakpoint]
DROP TABLE "regional_taxon_statuses";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regional_taxon_statuses" RENAME TO "regional_taxon_statuses";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_cleaning_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "cleaning_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "cleaning_id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_cleaning_procedure_citations" ("citation_id", "cleaning_id", "created_at", "updated_at") SELECT "citation_id", "cleaning_id", "created_at", "updated_at" FROM "cleaning_procedure_citations";
-- #[toasty::breakpoint]
DROP TABLE "cleaning_procedure_citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_cleaning_procedure_citations" RENAME TO "cleaning_procedure_citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_citations" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "title" TEXT NOT NULL,
    "url" TEXT,
    "author" TEXT,
    "date" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_citations" ("id", "title", "url", "author", "date", "created_at", "updated_at") SELECT "id", "title", "url", "author", "date", "created_at", "updated_at" FROM "citations";
-- #[toasty::breakpoint]
DROP TABLE "citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_citations" RENAME TO "citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_propagation_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "propagation_id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_propagation_procedure_citations" ("citation_id", "propagation_id", "created_at", "updated_at") SELECT "citation_id", "propagation_id", "created_at", "updated_at" FROM "propagation_procedure_citations";
-- #[toasty::breakpoint]
DROP TABLE "propagation_procedure_citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_propagation_procedure_citations" RENAME TO "propagation_procedure_citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxon_propagation_procedure_citations" (
    "citation_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "propagation_id", "taxon_id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_taxon_propagation_procedure_citations" ("citation_id", "propagation_id", "taxon_id", "created_at", "updated_at") SELECT "citation_id", "propagation_id", "taxon_id", "created_at", "updated_at" FROM "taxon_propagation_procedure_citations";
-- #[toasty::breakpoint]
DROP TABLE "taxon_propagation_procedure_citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxon_propagation_procedure_citations" RENAME TO "taxon_propagation_procedure_citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxon_note_citations" (
    "citation_id" INTEGER NOT NULL,
    "note_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("citation_id", "note_id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_taxon_note_citations" ("citation_id", "note_id", "created_at", "updated_at") SELECT "citation_id", "note_id", "created_at", "updated_at" FROM "taxon_note_citations";
-- #[toasty::breakpoint]
DROP TABLE "taxon_note_citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxon_note_citations" RENAME TO "taxon_note_citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
