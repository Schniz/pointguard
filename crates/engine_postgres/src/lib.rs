mod constants;
mod inflight_task;
mod task_listener;

pub use inflight_task::*;
pub use sqlx::postgres;
use sqlx::{Executor, PgPool};
use std::{num::NonZeroU32, str::FromStr};
pub use task_listener::{NewTaskPayload, TaskListener};

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FinishedTask {
    pub id: i64,
    pub job_name: String,
    pub name: String,
    pub endpoint: String,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub struct PaginationCursor {
    pub page: Option<NonZeroU32>,
    pub limit: Option<i32>,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_pages: usize,
    pub page: usize,
}

pub async fn finished_tasks(
    db: &PgPool,
    cursor: &PaginationCursor,
) -> Result<Paginated<FinishedTask>, sqlx::Error> {
    let limit = cursor.limit.unwrap_or(100) + 1;
    let offset = cursor.page.map_or(0, |p| (p.get() - 1) * limit as u32) as i64;

    let count =
        sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM finished_tasks").fetch_one(db);
    let items = sqlx::query_as!(
        FinishedTask,
        "
        SELECT
            id,
            job_name,
            name,
            endpoint,
            started_at,
            error_message,
            created_at,
            data,
            retries
        FROM
            finished_tasks
        ORDER BY
            created_at DESC
        LIMIT $1::int
        OFFSET $2::bigint
        ",
        limit,
        offset.into(),
    )
    .fetch_all(db);

    let (items, count) = tokio::join!(items, count);
    let (items, count) = (items?, count?);

    let total_pages = count / limit as i64;
    let current_page = cursor.page.map_or(1, |p| p.get() as usize);

    Ok(Paginated {
        items,
        total_pages: total_pages as usize,
        page: current_page,
    })
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

#[derive(Default)]
pub struct DbOptions {
    pub schema: Option<String>,
}

pub async fn connect(url: &str, options: &DbOptions) -> Result<PgPool, sqlx::Error> {
    let connection_opts = sqlx::postgres::PgConnectOptions::from_str(url)
        .expect("parse db url")
        .application_name(&format!("pointguard:{}", nanoid::nanoid!()));
    let mut pgpool_options = sqlx::postgres::PgPoolOptions::new();

    if let Some(schema) = options.schema.clone() {
        pgpool_options = pgpool_options.after_connect(move |conn, _| {
            let schema = schema.to_string();
            Box::pin(async move {
                conn.execute(&format!("SET search_path = '{schema}';")[..])
                    .await?;
                Ok(())
            })
        });
    }

    pgpool_options.connect_with(connection_opts).await
}

pub async fn migrate(pool: &PgPool, options: &DbOptions) -> Result<(), sqlx::Error> {
    let result = sqlx::migrate!().run(pool).await;

    if let Some(schema) = options.schema.clone() {
        if let Err(sqlx::migrate::MigrateError::Execute(sqlx::Error::Database(err))) =
            result.as_ref()
        {
            if let Some(code) = err.code() {
                if code == "3F000" {
                    tracing::info!("schema {schema:?} not found. trying to create it.");
                    pool.execute(&format!("CREATE SCHEMA {};", schema)[..])
                        .await?;
                    tracing::info!("schema {schema:?} created!");
                    sqlx::migrate!().run(pool).await?;
                    return Ok(());
                }
            }
        }
    }

    result.map_err(|e| e.into())
}
