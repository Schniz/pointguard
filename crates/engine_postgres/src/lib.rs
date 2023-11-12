mod constants;

#[derive(Debug)]
pub struct InflightTask {
    pub id: i64,
    pub job_name: String,
    pub data: serde_json::Value,
    pub endpoint: String,
    pub name: String,

    pub max_retries: i32,
    pub retry_count: i32,

    cleaned_up: bool,
}

impl InflightTask {
    pub async fn done(mut self, conn: &sqlx::PgPool) {
        let result = sqlx::query!("DELETE FROM tasks WHERE id = $1", self.id)
            .execute(conn)
            .await;

        if let Err(err) = result {
            tracing::error!("failed to delete task {}: {err}", self.id);
            panic!("failed to delete task {}: {err}", self.id);
        }

        // TODO: maybe we should have a global error handler that will retry?
        self.cleaned_up = true;
    }

    pub async fn failed(mut self, conn: &sqlx::PgPool) {
        if self.max_retries == self.retry_count {
            tracing::error!(
                "task {} failed {} times, giving up",
                self.id,
                self.retry_count + 1
            );
            let mut tx = conn.begin().await.expect("failed to start transaction");
            sqlx::query!(
                "
                INSERT INTO failed_tasks
                (job_name, data, endpoint, name, retries, started_at, task_created_at)
                SELECT job_name, data, endpoint, name, retry_count, started_at, created_at
                FROM tasks WHERE id = $1
                ",
                self.id
            )
            .execute(&mut *tx)
            .await
            .expect("failed to insert failed task");
            sqlx::query!(
                "
                DELETE FROM tasks
                WHERE id = $1
                ",
                self.id
            )
            .execute(&mut *tx)
            .await
            .expect("failed to delete task");
            tx.commit().await.expect("failed to commit transaction");
        } else {
            sqlx::query!(
                "
                UPDATE
                    tasks
                SET
                    worker_id = NULL,
                    started_at = NULL,
                    run_at = now() + retry_delay,
                    updated_at = now(),
                    retry_count = retry_count + 1
                WHERE
                    id = $1
            ",
                self.id,
            )
            .execute(conn)
            .await
            .expect("failed to update task");
        }

        self.cleaned_up = true;
    }
}

impl Drop for InflightTask {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("Task {} was not cleaned up!", self.id);
        }
    }
}

pub struct TaskListener {
    listener: sqlx::postgres::PgListener,
}

#[derive(Debug, serde::Deserialize)]
pub struct NewTaskPayload {
    pub id: i64,
    pub run_at: chrono::DateTime<chrono::Utc>,
}

impl TaskListener {
    pub async fn new(db: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        let mut listener = sqlx::postgres::PgListener::connect_with(&db).await?;
        listener.listen(constants::NEW_TASK_QUEUE).await?;
        Ok(Self { listener })
    }

    pub async fn take(&mut self) -> Result<NewTaskPayload, sqlx::Error> {
        loop {
            let notification = self.listener.recv().await?;
            if let Ok(v) = serde_json::from_str(notification.payload()) {
                return Ok(v);
            }
        }
    }
}

pub async fn free_tasks(db: &sqlx::PgPool, count: i64) -> Result<Vec<InflightTask>, sqlx::Error> {
    let mut tx = db.begin().await.unwrap();
    let inflight_tasks = sqlx::query_as!(
        InflightTask,
        "
        WITH running_workers AS (
            SELECT DISTINCT application_name
            FROM pg_stat_activity
            WHERE application_name LIKE 'pointguard:%'
        )
        SELECT id, job_name, data, endpoint, name, false as \"cleaned_up!\", max_retries, retry_count
        FROM tasks
        LEFT JOIN running_workers ON tasks.worker_id = running_workers.application_name
        WHERE running_workers.application_name IS NULL
          AND run_at <= NOW()
        FOR UPDATE
        SKIP LOCKED
        LIMIT $1
        ",
        count,
    )
    .fetch_all(&mut *tx)
    .await?;

    if !inflight_tasks.is_empty() {
        let ids: Vec<i64> = inflight_tasks.iter().map(|t| t.id).collect();
        sqlx::query!(
            "
            UPDATE
                tasks
            SET
                worker_id = current_setting('application_name'),
                started_at = now(),
                updated_at = now()
            WHERE
                id = ANY($1)
            ",
            &ids,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(inflight_tasks)
}

#[derive(Debug)]
pub struct NewTask {
    pub job_name: String,
    pub data: serde_json::Value,
    pub endpoint: String,
    pub name: String,
    pub run_at: Option<chrono::DateTime<chrono::Utc>>,

    pub max_retries: Option<i32>,
}

pub async fn enqueue(db: &sqlx::PgPool, task: &NewTask) -> Result<i64, sqlx::Error> {
    let id = sqlx::query!(
        "
        INSERT INTO tasks (job_name, data, endpoint, name, run_at, max_retries)
        VALUES ($1, $2, $3, $4, COALESCE($5, now()), $6)
        ON CONFLICT (job_name, name, endpoint) DO UPDATE
        SET
            updated_at = now()
        RETURNING
            id,
            CASE WHEN run_at <= NOW()
                THEN pg_notify($7, json_build_object('run_at', run_at, 'id', id)::text)
                ELSE null
            END AS \"notify\"
        ",
        task.job_name,
        task.data,
        task.endpoint,
        task.name,
        task.run_at,
        task.max_retries.unwrap_or(0) as i64,
        constants::NEW_TASK_QUEUE,
    )
    .fetch_one(db)
    .await?;
    Ok(id.id)
}

use std::str::FromStr;

pub use sqlx::postgres;
use sqlx::PgPool;

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    let connection_opts = sqlx::postgres::PgConnectOptions::from_str(url)
        .expect("parse db url")
        .application_name(&format!("pointguard:{}", nanoid::nanoid!()));
    sqlx::postgres::PgPoolOptions::new()
        .connect_with(connection_opts)
        .await
}

pub async fn migrate(url: &str) -> Result<(), sqlx::Error> {
    let pool = connect(url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(())
}
