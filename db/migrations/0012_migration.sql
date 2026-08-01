PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxa" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "itis_id" INTEGER NOT NULL,
    "inaturalist_id" INTEGER,
    "name1" TEXT NOT NULL,
    "name2" TEXT,
    "name3" TEXT,
    "complete_name" TEXT NOT NULL,
    "parent_id" INTEGER,
    "sequence" INTEGER NOT NULL,
    "rank" BIGINT NOT NULL,
    "life_form" BIGINT,
    "life_cycle" BIGINT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_taxa" ("id", "itis_id", "inaturalist_id", "name1", "name2", "name3", "complete_name", "parent_id", "sequence", "rank", "life_form", "life_cycle", "created_at", "updated_at") SELECT "id", "itis_id", "inaturalist_id", "name1", "name2", "name3", "complete_name", "parent_id", "sequence", "rank", "life_form", "life_cycle", "created_at", "updated_at" FROM "taxa";
-- #[toasty::breakpoint]
DROP TABLE "taxa";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxa" RENAME TO "taxa";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
