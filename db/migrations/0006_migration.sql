PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_citations" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "title" TEXT NOT NULL,
    "url" TEXT,
    "author" TEXT NOT NULL,
    "access_date" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    "publication_year" SMALLINT,
    "container_title" TEXT,
    "doi" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_citations" ("id", "title", "url", "author", "access_date", "created_at", "updated_at") SELECT "id", "title", "url", "author", "date", "created_at", "updated_at" FROM "citations";
-- #[toasty::breakpoint]
DROP TABLE "citations";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_citations" RENAME TO "citations";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
