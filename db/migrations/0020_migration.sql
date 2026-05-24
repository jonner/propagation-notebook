PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_protocols" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
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
