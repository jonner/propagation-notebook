DROP INDEX "index_protocols_by_taxon_id";
-- #[toasty::breakpoint]
DROP INDEX "index_protocols_by_protocol_id";
-- #[toasty::breakpoint]
DROP INDEX "index_protocols_by_environment_id";
-- #[toasty::breakpoint]
DROP INDEX "index_protocols_by_citation_id";
-- #[toasty::breakpoint]
ALTER TABLE "protocols" RENAME TO "propagation_protocols";
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_propagation_protocols_by_id" ON "propagation_protocols" ("id");
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_protocols_by_taxon_id" ON "propagation_protocols" ("taxon_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_protocols_by_protocol_id" ON "propagation_protocols" ("protocol_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_protocols_by_environment_id" ON "propagation_protocols" ("environment_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_propagation_protocols_by_citation_id" ON "propagation_protocols" ("citation_id");
