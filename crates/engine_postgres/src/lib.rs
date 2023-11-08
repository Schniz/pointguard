mod orm;

pub use orm::*;

pub use sea_orm;
use sea_orm::{QuerySelect, SqlxPostgresPoolConnection, Statement};

pub async fn poll_next_jobs(
    conn: SqlxPostgresPoolConnection,
    max_size: u64,
) -> Vec<orm::task::Model> {
    // let subscription = conn.clone();

    use sea_orm::prelude::*;

    let orm_connection = DatabaseConnection::SqlxPostgresPoolConnection(conn.clone());

    loop {
        orm::task::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                "
                    SELECT * FROM task
                    WHERE runnable_at <= NOW()
                    ORDER BY runnable_at ASC
                    LIMIT $1
                ",
                vec![max_size.into()],
            ))
            .all(&orm_connection)
            .await
            .ok();
    }
}
