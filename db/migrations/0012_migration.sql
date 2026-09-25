PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_permissions" (
    "id" BLOB NOT NULL,
    "code" TEXT NOT NULL CHECK ("code" IN ('taxon:sync', 'citation:create', 'citation:edit', 'citation:delete')),
    "description" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_permissions" ("id", "code", "description", "created_at", "updated_at") SELECT "id", "code", "description", "created_at", "updated_at" FROM "permissions";
-- #[toasty::breakpoint]
DROP TABLE "permissions";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_permissions" RENAME TO "permissions";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
