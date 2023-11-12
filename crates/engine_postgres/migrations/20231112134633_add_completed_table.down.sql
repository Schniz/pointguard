ALTER TABLE finished_tasks DROP COLUMN error_message;
ALTER TABLE finished_tasks RENAME TO failed_tasks;
