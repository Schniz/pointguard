#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvokedTaskPayload<'a> {
    /// The job name to invoke
    pub job_name: &'a str,
    /// The input data of the task
    pub input: &'a serde_json::Value,
    /// The amount of times we retried this task
    pub retry_count: i32,
    /// The maximum amount of times we can retry this task
    pub max_retries: i32,
    /// The time when this task was enqueued at
    pub created_at: &'a chrono::DateTime<chrono::Utc>,
}

const fn bool_true() -> bool {
    true
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InvokedTaskResponse {
    /// A successful invocation
    Success {},
    /// A failed invocation
    Failure {
        /// The reason why it failed
        reason: String,
        /// Whether or not this task is retriable
        #[serde(default = "bool_true")]
        retriable: bool,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Event {
    /// A task was enqueued
    TaskEnqueued,
    /// A task was invoked
    TaskInvoked,
    /// A task failed
    TaskFailed,
    /// A task finished
    TaskFinished,
}
