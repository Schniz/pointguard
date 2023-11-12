#[derive(Debug)]
pub struct InflightTask {
    pub id: i64,
    pub job_name: String,
    pub data: serde_json::Value,
    pub endpoint: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,

    pub max_retries: i32,
    pub retry_count: i32,

    cleaned_up: bool,
}

impl InflightTask {
    pub async fn done(mut self, conn: &sqlx::PgPool) {
        let mut tx = conn.begin().await.expect("failed to start transaction");
        sqlx::query!(
            "
            INSERT INTO finished_tasks
            (job_name, data, endpoint, name, retries, started_at, task_created_at)
            SELECT job_name, data, endpoint, name, retry_count, started_at, created_at
            FROM tasks WHERE id = $1
            ",
            self.id,
        )
        .execute(&mut *tx)
        .await
        .expect("failed to insert finished task");
        sqlx::query!("DELETE FROM tasks WHERE id = $1", self.id)
            .execute(&mut *tx)
            .await
            .expect("failed to delete task");

        tx.commit().await.expect("failed to commit transaction");

        // TODO: maybe we should have a global error handler that will retry?
        self.cleaned_up = true;
    }

    pub async fn failed(mut self, conn: &sqlx::PgPool, message: &str) {
        if self.max_retries == self.retry_count {
            tracing::error!(
                "task {} failed {} times, giving up",
                self.id,
                self.retry_count + 1
            );
            let mut tx = conn.begin().await.expect("failed to start transaction");
            sqlx::query!(
                "
                INSERT INTO finished_tasks
                (job_name, data, endpoint, name, retries, started_at, task_created_at, error_message)
                SELECT job_name, data, endpoint, name, retry_count, started_at, created_at, $2
                FROM tasks WHERE id = $1
                ",
                self.id,
                message,
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

pub async fn free_tasks(db: &sqlx::PgPool, count: i64) -> Result<Vec<InflightTask>, sqlx::Error> {
    let mut tx = db.begin().await.unwrap();
    let inflight_tasks = sqlx::query_as!(
        InflightTask,
        "
        SELECT id, created_at, job_name, data, endpoint, name, false as \"cleaned_up!\", max_retries, retry_count
        FROM tasks
        LEFT JOIN running_workers ON tasks.worker_id = running_workers.application_name
        WHERE running_workers.application_name IS NULL
          AND run_at <= NOW()
        FOR UPDATE of tasks
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
