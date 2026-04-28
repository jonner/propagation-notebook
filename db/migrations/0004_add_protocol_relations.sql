ALTER TABLE "culture_environments" DROP COLUMN "taxon_id";
-- #[toasty::breakpoint]
ALTER TABLE "procedures" DROP COLUMN "protocol_id";
-- #[toasty::breakpoint]
ALTER TABLE "procedures" DROP COLUMN "environment_id";
-- #[toasty::breakpoint]
ALTER TABLE "procedures" ADD COLUMN "propagation_type" BIGINT NOT NULL;
-- #[toasty::breakpoint]
ALTER TABLE "procedures" ADD COLUMN "difficulty" BIGINT NOT NULL;
-- #[toasty::breakpoint]
ALTER TABLE "asexual_method_steps" RENAME COLUMN "protocol_id" TO "procedure_id";
-- #[toasty::breakpoint]
CREATE INDEX "index_asexual_method_steps_by_procedure_id" ON "asexual_method_steps" ("procedure_id");
-- #[toasty::breakpoint]
ALTER TABLE "protocols" DROP COLUMN "propagation_type";
-- #[toasty::breakpoint]
ALTER TABLE "protocols" DROP COLUMN "difficulty";
-- #[toasty::breakpoint]
ALTER TABLE "culture_environments" ADD COLUMN "environment_id" INTEGER NOT NULL;
-- #[toasty::breakpoint]
ALTER TABLE "culture_environments" ADD COLUMN "protocol_id" INTEGER NOT NULL;
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_protocol_id" ON "protocols" ("protocol_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_taxon_id" ON "protocols" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_citation_id" ON "protocols" ("citation_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_protocols_by_environment_id" ON "protocols" ("environment_id");
-- #[toasty::breakpoint]
ALTER TABLE "sexual_method_steps" RENAME COLUMN "protocol_id" TO "procedure_id";
-- #[toasty::breakpoint]
CREATE INDEX "index_sexual_method_steps_by_procedure_id" ON "sexual_method_steps" ("procedure_id");
