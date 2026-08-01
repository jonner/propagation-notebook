PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "geometry" TEXT,
    "notes" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regions" ("id", "name", "geometry", "notes") SELECT "id", "name", "geometry", "notes" FROM "regions";
-- #[toasty::breakpoint]
DROP TABLE "regions";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regions" RENAME TO "regions";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
CREATE TABLE "taxon_photos" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "square_url" TEXT,
    "medium_url" TEXT,
    "large_url" TEXT,
    "is_default" BOOLEAN NOT NULL,
    "attribution" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_photos_by_taxon_id" ON "taxon_photos" ("taxon_id");
