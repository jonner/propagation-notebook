CREATE TABLE "taxon_photos" (
    "taxon_id" INTEGER NOT NULL,
    "square_url" TEXT,
    "medium_url" TEXT,
    "large_url" TEXT,
    "is_default" BOOLEAN NOT NULL,
    "attribution" TEXT,
    PRIMARY KEY ("taxon_id")
);
