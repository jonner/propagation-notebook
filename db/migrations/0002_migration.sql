ALTER TABLE "citations" RENAME COLUMN "text" TO "title";
-- #[toasty::breakpoint]
ALTER TABLE "citations" ADD COLUMN "date" TEXT;
