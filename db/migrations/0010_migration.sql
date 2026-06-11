CREATE TABLE "harvest_events" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "taxon_id" INTEGER NOT NULL,
    "date" TEXT NOT NULL,
    "notes" TEXT,
    "location_id" INTEGER NOT NULL
);
-- #[toasty::breakpoint]
CREATE INDEX "index_harvest_events_by_taxon_id" ON "harvest_events" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_harvest_events_by_location_id" ON "harvest_events" ("location_id");
-- #[toasty::breakpoint]
CREATE TABLE "harvest_locations" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "latitude" REAL NOT NULL,
    "longitude" REAL NOT NULL
);
