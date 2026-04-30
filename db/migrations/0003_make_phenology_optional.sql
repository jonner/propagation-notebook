PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regional_taxon_statuses" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "native_status" BIGINT,
    "c_value" INTEGER,
    "conservation_status" BIGINT,
    "wetland_indicator" BIGINT,
    "phenology_window_start" TEXT,
    "phenology_window_end" TEXT,
    "native_plant_community_id" INTEGER
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regional_taxon_statuses" ("id", "taxon_id", "region_id", "native_status", "c_value", "conservation_status", "wetland_indicator", "phenology_window_start", "phenology_window_end", "native_plant_community_id") SELECT "id", "taxon_id", "region_id", "native_status", "c_value", "conservation_status", "wetland_indicator", "phenology_window_start", "phenology_window_end", "native_plant_community_id" FROM "regional_taxon_statuses";
-- #[toasty::breakpoint]
DROP TABLE "regional_taxon_statuses";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regional_taxon_statuses" RENAME TO "regional_taxon_statuses";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
