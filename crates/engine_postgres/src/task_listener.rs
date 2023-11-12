use crate::constants;

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
