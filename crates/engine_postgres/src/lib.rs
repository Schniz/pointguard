mod constants;
mod inflight_task;
mod task_listener;

pub use inflight_task::*;
pub use sqlx::postgres;
use sqlx::PgPool;
use std::str::FromStr;
pub use task_listener::{NewTaskPayload, TaskListener};

#[derive(Debug, serde::Serialize)]
pub struct FinishedTask {
    pub id: i64,
    pub job_name: String,
    pub name: String,
    pub endpoint: String,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
    pub retries: i32,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnqueuedTask {
    pub id: i64,
    pub job_name: String,
    pub name: String,
    pub endpoint: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
    pub run_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub worker_id: Option<String>,
}

pub async fn cancel_task(db: &PgPool, id: i64) -> Result<Option<i64>, sqlx::Error> {
    let task = sqlx::query!(
        "
        DELETE FROM
            tasks
        WHERE
            id = $1
            AND (
                worker_id IS NULL
                OR worker_id NOT IN (
                    SELECT
                        application_name
                    FROM
                        running_workers
                )
            )
        RETURNING
            id
        ",
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(task.map(|t| t.id))
}

pub async fn enqueued_tasks(db: &PgPool) -> Result<Vec<EnqueuedTask>, sqlx::Error> {
    sqlx::query_as!(
        EnqueuedTask,
        "
        SELECT
            id,
            job_name,
            name,
            endpoint,
            created_at,
            data,
            run_at,
            retry_count,
            max_retries,
            worker_id
        FROM
            tasks
        LEFT OUTER JOIN
            running_workers ON tasks.worker_id = running_workers.application_name
        ORDER BY
            run_at ASC
        "
    )
    .fetch_all(db)
    .await
}

pub async fn finished_tasks(db: &PgPool) -> Result<Vec<FinishedTask>, sqlx::Error> {
    sqlx::query_as!(
        FinishedTask,
        "
        SELECT
            id,
            job_name,
            name,
            endpoint,
            error_message,
            created_at,
            data,
            retries
        FROM
            finished_tasks
        ORDER BY
            created_at DESC
        "
    )
    .fetch_all(db)
    .await
}

#[derive(serde::Serialize, Debug)]
pub struct OngoingTask {
    pub id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub job_name: String,
    pub data: serde_json::Value,
    pub endpoint: String,
    pub name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub worker_id: String,

    pub max_retries: i32,
    pub retry_count: i32,
}

pub async fn ongoing_tasks(db: &sqlx::PgPool) -> Result<Vec<OngoingTask>, sqlx::Error> {
    let tasks = sqlx::query_as!(
        OngoingTask,
        "
        SELECT
            id,
            created_at,
            job_name,
            data,
            endpoint,
            name,
            \"started_at\" as \"started_at!\",
            max_retries,
            retry_count,
            worker_id as \"worker_id!\"
        FROM tasks
        JOIN running_workers ON tasks.worker_id = running_workers.application_name
        "
    )
    .fetch_all(db)
    .await?;

    Ok(tasks)
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
    tracing::info!("enqueued task {:?}", id);
    Ok(id.id)
}

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
