CREATE TABLE "regional_taxon_sync_tasks" (
    "regional_taxon_status_id" INTEGER NOT NULL,
    "last_attempt" TEXT NOT NULL,
    PRIMARY KEY ("regional_taxon_status_id")
);
-- #[toasty::breakpoint]
INSERT INTO "regional_taxon_sync_tasks" (regional_taxon_status_id, last_attempt) SELECT id, last_sync_attempt FROM regional_taxon_statuses WHERE last_sync_attempt IS NOT NULL;
-- #[toasty::breakpoint]
ALTER TABLE "regional_taxon_statuses" DROP COLUMN "last_sync_attempt";
