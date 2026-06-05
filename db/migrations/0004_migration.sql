PRAGMA foreign_keys = OFF;
-- #[toasty::breakpoint]
CREATE TABLE "_toasty_new_taxon_protocols" (
    "taxon_id" INTEGER NOT NULL,
    "protocol_id" INTEGER NOT NULL,
    "confidence" INTEGER,
    "notes" TEXT,
    PRIMARY KEY ("taxon_id", "protocol_id")
);
-- #[toasty::breakpoint]
INSERT INTO "_toasty_new_taxon_protocols" ("taxon_id", "protocol_id", "confidence", "notes") SELECT "taxon_id", "protocol_id", "confidence", "notes" FROM "taxon_protocols";
-- #[toasty::breakpoint]
DROP TABLE "taxon_protocols";
-- #[toasty::breakpoint]
ALTER TABLE "_toasty_new_taxon_protocols" RENAME TO "taxon_protocols";
-- #[toasty::breakpoint]
PRAGMA foreign_keys = ON;
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_taxon_protocols_by_taxon_id_and_protocol_id" ON "taxon_protocols" ("taxon_id", "protocol_id");
