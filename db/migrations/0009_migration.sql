CREATE TABLE "sessions" (
    "id" BLOB NOT NULL,
    "token_hash" BLOB NOT NULL,
    "user_id" BLOB NOT NULL,
    "expires_at" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_sessions_by_token_hash" ON "sessions" ("token_hash");
-- #[toasty::breakpoint]
CREATE INDEX "index_sessions_by_user_id" ON "sessions" ("user_id");
-- #[toasty::breakpoint]
CREATE TABLE "users" (
    "id" BLOB NOT NULL,
    "username" TEXT NOT NULL,
    "pwhash" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_users_by_username" ON "users" ("username");
-- #[toasty::breakpoint]
CREATE INDEX "index_users_by_pwhash" ON "users" ("pwhash");
-- #[toasty::breakpoint]
CREATE TABLE "roles" (
    "id" BLOB NOT NULL,
    "name" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_roles_by_name" ON "roles" ("name");
-- #[toasty::breakpoint]
CREATE TABLE "user_roles" (
    "user_id" BLOB NOT NULL,
    "role_id" BLOB NOT NULL,
    PRIMARY KEY ("user_id", "role_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_user_roles_by_user_id" ON "user_roles" ("user_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_user_roles_by_role_id" ON "user_roles" ("role_id");
-- #[toasty::breakpoint]
CREATE TABLE "permissions" (
    "id" BLOB NOT NULL,
    "code" TEXT NOT NULL CHECK ("code" IN ('taxon:sync')),
    "description" TEXT NOT NULL,
    PRIMARY KEY ("id")
);
-- #[toasty::breakpoint]
CREATE UNIQUE INDEX "index_permissions_by_code" ON "permissions" ("code");
-- #[toasty::breakpoint]
CREATE TABLE "role_permissions" (
    "role_id" BLOB NOT NULL,
    "permission_id" BLOB NOT NULL,
    PRIMARY KEY ("role_id", "permission_id")
);
-- #[toasty::breakpoint]
CREATE INDEX "index_role_permissions_by_role_id" ON "role_permissions" ("role_id");
-- #[toasty::breakpoint]
CREATE INDEX "index_role_permissions_by_permission_id" ON "role_permissions" ("permission_id");
