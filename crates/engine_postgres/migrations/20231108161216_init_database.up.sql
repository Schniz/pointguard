CREATE TABLE tasks (
  id bigserial primary key,
  name varchar(255) not null,
  job_name varchar(255) not null,
  endpoint varchar(1024) not null,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now(),
  runnable_at timestamp not null default now(),
  data jsonb not null default 'null',

  worker_id varchar(100),
  started_at timestamp,

  unique(name, job_name, endpoint)
);

comment on column tasks.id is 'the ID';
comment on column tasks.name is 'A unique task name for deduping';
comment on column tasks.job_name is 'the job to execute';
comment on column tasks.endpoint is 'the endpoint to execute the job on';
comment on column tasks.created_at is 'When the job was created';
comment on column tasks.updated_at is 'When the job was last updated';
comment on column tasks.runnable_at is 'Time to run the job';
comment on column tasks.data is 'The task JSON data';
comment on column tasks.worker_id is 'the worker that is currently working on this task';
comment on column tasks.started_at is 'when the worker started working on this task';

CREATE INDEX tasks_runnable_at_idx ON tasks (runnable_at);
CREATE INDEX tasks_worker_id_idx ON tasks (worker_id);
