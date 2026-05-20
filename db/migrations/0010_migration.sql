PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_cleaning_procedure_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "procedure_id" INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    "cleaning_type" BIGINT NOT NULL,
    "equipment" TEXT,
    "notes" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_cleaning_procedure_steps" ("id", "procedure_id", "order", "cleaning_type", "equipment", "notes") SELECT "id", "procedure_id", "order", "cleaning_type", "equipment", "notes" FROM "cleaning_procedure_steps";
-- #[toasty::breakpoint]
DROP TABLE "cleaning_procedure_steps";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_cleaning_procedure_steps" RENAME TO "cleaning_procedure_steps";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
