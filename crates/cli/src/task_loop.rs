use pointguard_engine_postgres::{self as db, postgres::PgPool};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InvokedTask<'a> {
    job_name: &'a str,
    input: &'a serde_json::Value,
    retry_count: i32,
    max_retries: i32,
    created_at: &'a chrono::DateTime<chrono::Utc>,
}

#[tracing::instrument(skip_all, fields(id = %task.id, endpoint = %task.endpoint))]
async fn execute_task(http: reqwest::Client, task: db::InflightTask, db: PgPool) {
    let response = http
        .post(&task.endpoint)
        .json(&InvokedTask {
            job_name: &task.job_name[..],
            input: &task.data,
            retry_count: task.retry_count,
            max_retries: task.max_retries,
            created_at: &task.created_at,
        })
        .send()
        .await
        .and_then(|res| res.error_for_status());

    match response {
        Ok(_) => {
            tracing::info!("invocation completed");
            task.done(&db).await;
        }
        Err(err) => {
            tracing::error!("invocation failed: {err}");
            task.failed(&db).await;
        }
    }
}

pub async fn run(db: db::postgres::PgPool) {
    let mut listener = db::TaskListener::new(&db)
        .await
        .expect("listen to task queue");
    loop {
        let mut tasks = db::free_tasks(&db, 5).await.unwrap_or_else(|err| {
            tracing::error!("Can't fetch tasks: {err}");
            vec![]
        });

        if tasks.is_empty() {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(20)) => {},
                _ = listener.take() => {
                    tracing::info!("woke up from listener");
                },
            }
            continue;
        }

        let http = reqwest::Client::new();

        for task in tasks.drain(..) {
            tokio::spawn(execute_task(http.clone(), task, db.clone()));
        }
    }
}
