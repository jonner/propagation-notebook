ALTER TABLE "protocols" RENAME TO "propagation_procedures";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_protocols" RENAME TO "taxon_propagation_procedures";
-- #[toasty::breakpoint]
ALTER TABLE "taxon_propagation_procedures" RENAME COLUMN "protocol_id" TO "propagation_id";
