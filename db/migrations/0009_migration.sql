ALTER TABLE "regions" ADD COLUMN "category" TEXT CHECK ("category" IN ('nation', 'province', 'county', 'municipality', 'other'));
-- #[toasty::breakpoint]
CREATE INDEX "index_regions_by_category" ON "regions" ("category");
