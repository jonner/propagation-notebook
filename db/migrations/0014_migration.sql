PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_vernacular_names" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_vernacular_names" ("id", "taxon_id", "name", "created_at", "updated_at") SELECT "id", "taxon_id", "name", "created_at", "updated_at" FROM "vernacular_names";
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
    "complete_name" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_synonyms" ("id", "taxon_id", "name1", "name2", "name3", "complete_name", "created_at", "updated_at") SELECT "id", "taxon_id", "name1", "name2", "name3", "complete_name", "created_at", "updated_at" FROM "synonyms";
-- #[toasty::breakpoint]
DROP TABLE "synonyms";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_synonyms" RENAME TO "synonyms";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_cleaning_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "notes" TEXT,
    "instructions" TEXT NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_cleaning_procedures" ("id", "name", "notes", "instructions", "taxon_id", "created_at", "updated_at") SELECT "id", "name", "notes", "instructions", "taxon_id", "created_at", "updated_at" FROM "cleaning_procedures";
-- #[toasty::breakpoint]
DROP TABLE "cleaning_procedures";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_cleaning_procedures" RENAME TO "cleaning_procedures";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_propagation_procedures" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "instructions" TEXT NOT NULL,
    "notes" TEXT,
    "type" BIGINT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_propagation_procedures" ("id", "name", "instructions", "notes", "type", "created_at", "updated_at") SELECT "id", "name", "instructions", "notes", "type", "created_at", "updated_at" FROM "propagation_procedures";
-- #[toasty::breakpoint]
DROP TABLE "propagation_procedures";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_propagation_procedures" RENAME TO "propagation_procedures";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxon_propagation_procedures" (
    "taxon_id" INTEGER NOT NULL,
    "propagation_id" INTEGER NOT NULL,
    "confidence" INTEGER,
    "notes" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("taxon_id", "propagation_id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_taxon_propagation_procedures" ("taxon_id", "propagation_id", "confidence", "notes", "created_at", "updated_at") SELECT "taxon_id", "propagation_id", "confidence", "notes", "created_at", "updated_at" FROM "taxon_propagation_procedures";
-- #[toasty::breakpoint]
DROP TABLE "taxon_propagation_procedures";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxon_propagation_procedures" RENAME TO "taxon_propagation_procedures";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxon_photos" (
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
INSERT INTO "_toasty_new_taxon_photos" ("taxon_id", "square_url", "medium_url", "large_url", "is_default", "attribution", "created_at", "updated_at") SELECT "taxon_id", "square_url", "medium_url", "large_url", "is_default", "attribution", "created_at", "updated_at" FROM "taxon_photos";
-- #[toasty::breakpoint]
DROP TABLE "taxon_photos";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxon_photos" RENAME TO "taxon_photos";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
