ALTER TABLE failed_tasks RENAME TO finished_tasks;
ALTER TABLE finished_tasks ADD COLUMN error_message TEXT DEFAULT 'unknown error';
ALTER TABLE finished_tasks ALTER COLUMN error_message DROP DEFAULT;
