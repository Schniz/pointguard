use futures::Future;
use pointguard_engine_postgres::{self as db, postgres::PgPool};
use pointguard_types::{Event, InvokedTaskPayload, InvokedTaskResponse};

#[tracing::instrument(skip_all, fields(id = %task.id, endpoint = %task.endpoint))]
async fn execute_task(
    http: reqwest::Client,
    task: db::InflightTask,
    db: PgPool,
    events_tx: flume::Sender<Event>,
) {
    let response = http
        .post(&task.endpoint)
        .json(&InvokedTaskPayload {
            job_name: &task.job_name[..],
            input: &task.data,
            retry_count: task.retry_count,
            max_retries: task.max_retries,
            created_at: &task.created_at,
        })
        .send()
        .await
        .and_then(|res| res.error_for_status());

    let response = match response {
        Err(err) => Err(err),
        Ok(res) => res.json::<InvokedTaskResponse>().await,
    }
    .unwrap_or_else(|err| InvokedTaskResponse::Failure {
        reason: err.to_string(),
        retriable: true,
    });

    match response {
        InvokedTaskResponse::Success {} => {
            events_tx
                .send_async(Event::TaskFinished)
                .await
                .expect("send event");
            tracing::info!("invocation completed");
            task.done(&db).await;
        }
        InvokedTaskResponse::Failure { reason, retriable } => {
            events_tx
                .send_async(Event::TaskFailed)
                .await
                .expect("send event");
            tracing::error!("invocation failed: {reason}");
            task.failed(&db, &reason, retriable).await;
        }
    };
}

pub async fn run(
    db: db::postgres::PgPool,
    termination: impl Future<Output = ()>,
    events_tx: flume::Sender<Event>,
) {
    tokio::pin!(termination);
    let mut listener = db::TaskListener::new(&db)
        .await
        .expect("listen to task queue");

    let http = reqwest::Client::new();

    loop {
        tokio::select! {
            _ = &mut termination => {
                tracing::info!("shutting down");
                break;
            },
            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {
            }
        };

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
                _ = &mut termination => {
                    tracing::info!("shutting down");
                    break;
                }
            }
            continue;
        }

        for task in tasks.drain(..) {
            events_tx
                .send_async(Event::TaskInvoked)
                .await
                .expect("send event");
            tokio::spawn(execute_task(
                http.clone(),
                task,
                db.clone(),
                events_tx.clone(),
            ));
        }
    }
}
