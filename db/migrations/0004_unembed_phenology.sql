ALTER TABLE "regional_taxon_statuses" RENAME COLUMN "phenology_window_start" TO "window_start";
-- #[toasty::breakpoint]
ALTER TABLE "regional_taxon_statuses" RENAME COLUMN "phenology_window_end" TO "window_end";
