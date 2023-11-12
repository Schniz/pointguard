ALTER TABLE tasks ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN retry_delay INTERVAL NOT NULL DEFAULT '10 seconds';
ALTER TABLE tasks ADD COLUMN last_retried_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE tasks ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;

CREATE TABLE failed_tasks (
  id bigserial primary key,
  name varchar(255) not null,
  job_name varchar(255) not null,
  endpoint varchar(1024) not null,
  data jsonb not null,

  created_at timestamptz not null default now(),
  task_created_at timestamptz not null,

  worker_id varchar(100),
  started_at timestamptz,

  retries integer not null
);
