PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_collecting_data" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "ripening_indicators" TEXT,
    "storage" TEXT,
    "storage_life" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_collecting_data" ("id", "taxon_id", "ripening_indicators", "storage", "storage_life") SELECT "id", "taxon_id", "ripening_indicators", "storage", "storage_life" FROM "collecting_data";
-- #[toasty::breakpoint]
DROP TABLE "collecting_data";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_collecting_data" RENAME TO "collecting_data";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
