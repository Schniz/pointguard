UPDATE "finished_tasks" SET "started_at" = "created_at" WHERE "started_at" IS NULL;
ALTER TABLE "finished_tasks" ALTER COLUMN "started_at" SET NOT NULL;
