DROP TABLE failed_tasks;
ALTER TABLE tasks DROP COLUMN max_retries;
ALTER TABLE tasks DROP COLUMN retry_delay;
ALTER TABLE tasks DROP COLUMN last_retried_at;
ALTER TABLE tasks DROP COLUMN retry_count;
