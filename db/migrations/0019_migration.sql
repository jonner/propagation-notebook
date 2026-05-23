ALTER TABLE "protocols" ADD COLUMN "type" TEXT NOT NULL CHECK ("type" IN ('pretreatment', 'germination', 'establishment'));
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_protocol_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "protocol_id" INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    "step_type" BIGINT NOT NULL,
    "title" TEXT NOT NULL,
    "instructions" TEXT,
    "duration" INTEGER,
    "min_temp" REAL,
    "max_temp" REAL,
    "light" BIGINT,
    "moisture" TEXT,
    "materials" TEXT,
    "is_optional" BOOLEAN NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_protocol_steps" ("id", "protocol_id", "order", "step_type", "title", "instructions", "duration", "min_temp", "max_temp", "light", "moisture", "materials", "is_optional", "notes") SELECT "id", "protocol_id", "step_order", "step_type", "title", "instructions", "duration_days", "temperature_min_c", "temperature_max_c", "light_requirement", "moisture_requirement", "materials", "is_optional", "notes" FROM "protocol_steps";
-- #[toasty::breakpoint]
DROP TABLE "protocol_steps";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_protocol_steps" RENAME TO "protocol_steps";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
