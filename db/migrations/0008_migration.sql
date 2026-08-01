PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "geometry" TEXT,
    "notes" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regions" ("id", "name", "geometry", "notes", "created_at", "updated_at") SELECT "id", "name", "geometry", "notes", "created_at", "updated_at" FROM "regions";
-- #[toasty::breakpoint]
DROP TABLE "regions";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regions" RENAME TO "regions";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
