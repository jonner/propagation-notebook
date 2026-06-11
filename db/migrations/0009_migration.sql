CREATE TABLE "taxon_notes" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "text" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_notes_by_taxon_id" ON "taxon_notes" ("taxon_id");
