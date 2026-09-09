CREATE TABLE "user_emails" (
    "id" BLOB NOT NULL,
    "user_id" BLOB NOT NULL,
    "address" TEXT NOT NULL,
    "confirmed" BOOLEAN NOT NULL,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_user_emails_by_user_id" ON "user_emails" ("user_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_user_emails_by_address" ON "user_emails" ("address");
-- #[toasty::breakpoint]
CREATE TABLE "user_profiles" (
    "user_id" BLOB NOT NULL,
    "name" TEXT,
    "description" TEXT,
    "created_at" TEXT NOT NULL,
    "updated_at" TEXT NOT NULL,
    PRIMARY KEY ("user_id")
);
