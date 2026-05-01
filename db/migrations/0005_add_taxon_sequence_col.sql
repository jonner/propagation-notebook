ALTER TABLE "taxa" ADD COLUMN "sequence" INTEGER NOT NULL;
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_taxa_by_sequence" ON "taxa" ("sequence");
