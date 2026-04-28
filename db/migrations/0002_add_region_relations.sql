CREATE INDEX "index_native_plant_communities_by_name" ON "native_plant_communities" ("name");
-- #[toasty::breakpoint]
CREATE INDEX "index_native_plant_communities_by_region_id" ON "native_plant_communities" ("region_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_region_statuses_by_taxon_id" ON "region_statuses" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_region_statuses_by_region_id" ON "region_statuses" ("region_id");
-- #[toasty::breakpoint]
PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "bound" TEXT NOT NULL
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regions" ("id", "name", "bound") SELECT "id", "name", "bound" FROM "regions";
-- #[toasty::breakpoint]
DROP TABLE "regions";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regions" RENAME TO "regions";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
CREATE INDEX "index_regions_by_name" ON "regions" ("name");
