PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_regional_taxon_statuses" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "region_id" INTEGER NOT NULL,
    "origin" BIGINT,
    "c_value" INTEGER,
    "conservation_status" BIGINT,
    "wetland_indicator" BIGINT,
    "harvest_window_start_doy" SMALLINT,
    "harvest_window_end_doy" SMALLINT,
    "native_plant_community_id" INTEGER
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_regional_taxon_statuses" ("id", "taxon_id", "region_id", "origin", "c_value", "conservation_status", "wetland_indicator", "harvest_window_start_doy", "harvest_window_end_doy", "native_plant_community_id") SELECT "id", "taxon_id", "region_id", "origin", "c_value", "conservation_status", "wetland_indicator", strftime('%j',"harvest_window_start"), strftime('%j',"harvest_window_end"), "native_plant_community_id" FROM "regional_taxon_statuses";
-- #[toasty::breakpoint]
DROP TABLE "regional_taxon_statuses";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_regional_taxon_statuses" RENAME TO "regional_taxon_statuses";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
