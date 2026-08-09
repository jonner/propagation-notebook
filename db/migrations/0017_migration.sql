CREATE TABLE "taxon_hierarchies" (
    "ancestor_id" INTEGER NOT NULL,
    "descendant_id" INTEGER NOT NULL,
    "depth" INTEGER NOT NULL,
    PRIMARY KEY ("ancestor_id", "descendant_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_hierarchies_by_ancestor_id" ON "taxon_hierarchies" ("ancestor_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_hierarchies_by_descendant_id" ON "taxon_hierarchies" ("descendant_id");
