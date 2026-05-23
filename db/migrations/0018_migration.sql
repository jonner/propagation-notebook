DROP TABLE "propagation_protocols";
-- #[toasty::breakpoint]
DROP TABLE "procedures";
-- #[toasty::breakpoint]
DROP TABLE "sexual_method_steps";
-- #[toasty::breakpoint]
DROP TABLE "asexual_method_steps";
-- #[toasty::breakpoint]
DROP TABLE "culture_environments";
-- #[toasty::breakpoint]
CREATE TABLE "protocol_steps" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "protocol_id" INTEGER NOT NULL,
    "step_order" INTEGER NOT NULL,
    "step_type" BIGINT NOT NULL,
    "title" TEXT NOT NULL,
    "instructions" TEXT,
    "duration_days" INTEGER,
    "temperature_min_c" REAL,
    "temperature_max_c" REAL,
    "light_requirement" BIGINT,
    "moisture_requirement" TEXT,
    "materials" TEXT,
    "is_optional" BOOLEAN NOT NULL,
    "notes" TEXT
);
-- #[toasty::breakpoint]
CREATE INDEX "index_protocol_steps_by_protocol_id" ON "protocol_steps" ("protocol_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_protocols" (
    "id" INTEGER NOT NULL,
    "taxon_id" INTEGER NOT NULL,
    "pretreatment_protocol_id" INTEGER,
    "germination_protocol_id" INTEGER,
    "establishment_protocol_id" INTEGER,
    "confidence" INTEGER,
    "success_rate" REAL,
    "notes" TEXT,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_taxon_id" ON "taxon_protocols" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_pretreatment_protocol_id" ON "taxon_protocols" ("pretreatment_protocol_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_germination_protocol_id" ON "taxon_protocols" ("germination_protocol_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocols_by_establishment_protocol_id" ON "taxon_protocols" ("establishment_protocol_id");
-- #[toasty::breakpoint]
CREATE TABLE "protocol_citations" (
    "protocol_id" INTEGER NOT NULL,
    "citation_id" INTEGER NOT NULL,
    PRIMARY KEY ("protocol_id", "citation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_protocol_citations_by_protocol_id" ON "protocol_citations" ("protocol_id");
-- #[toasty::breakpoint]
CREATE TABLE "taxon_protocol_citations" (
    "id" INTEGER NOT NULL,
    "taxon_protocol_id" INTEGER NOT NULL,
    "citation_id" INTEGER NOT NULL,
    PRIMARY KEY ("id", "citation_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_taxon_protocol_citations_by_taxon_protocol_id" ON "taxon_protocol_citations" ("taxon_protocol_id");
-- #[toasty::breakpoint]
CREATE TABLE "protocols" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "notes" TEXT
);
