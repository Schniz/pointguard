#[derive(Debug)]
pub struct InflightTask {
    pub id: i64,
    pub job_name: String,
    pub data: serde_json::Value,
    pub endpoint: String,
    pub name: String,

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

    pub async fn release(self, conn: &sqlx::PgPool) {
        self.done(conn).await
        // TODO: actually release, but we need to manage retries..
        // sqlx::query!(
        //     "
        //     UPDATE
        //         tasks
        //     SET
        //         worker_id = NULL,
        //         started_at = NULL,
        //         runnable_at = now() + INTERVAL '1 minute',
        //         updated_at = now()
        //     WHERE
        //         id = $1
        //     ",
        //     self.id,
        // )
        // .execute(conn)
        // .await
        // .unwrap();
        // self.cleaned_up = true;
    }
}

impl Drop for InflightTask {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("Task {} was not cleaned up!", self.id);
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
        SELECT id, job_name, data, endpoint, name, false as \"cleaned_up!\" FROM tasks
        LEFT JOIN running_workers ON tasks.worker_id = running_workers.application_name
        WHERE running_workers.application_name IS NULL
          AND runnable_at <= NOW()
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
}

pub async fn enqueue(db: &sqlx::PgPool, task: &NewTask) -> Result<i64, sqlx::Error> {
    let id = sqlx::query!(
        "
        INSERT INTO tasks (job_name, data, endpoint, name)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (job_name, name, endpoint) DO UPDATE
        SET
            updated_at = now()
        RETURNING id
        ",
        task.job_name,
        task.data,
        task.endpoint,
        task.name,
    )
    .fetch_one(db)
    .await?;
    Ok(id.id)
}

pub use sqlx::postgres;
