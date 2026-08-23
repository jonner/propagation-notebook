CREATE TABLE "taxon_resources" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "url" TEXT NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_resources_by_taxon_id" ON "taxon_resources" ("taxon_id");
