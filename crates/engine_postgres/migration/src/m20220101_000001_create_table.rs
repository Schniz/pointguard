use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Task::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Task::Name).string().not_null())
                    .col(ColumnDef::new(Task::JobName).string_len(255).not_null())
                    .col(
                        ColumnDef::new(Task::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Task::Endpoint).string().not_null())
                    .col(
                        ColumnDef::new(Task::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Task::RunnableAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Task::Data)
                            .json_binary()
                            .not_null()
                            .default(Expr::value("{}")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Task::Table)
                    .col(Task::Name)
                    .col(Task::JobName)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Task::Table)
                    .col(Task::JobName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,

    /// the ID
    Id,

    /// A unique task name for deduping
    Name,
    JobName,
    Endpoint,

    /// When the job was created
    CreatedAt,
    /// When the job was last updated
    UpdatedAt,

    /// Since when the job is runnable
    RunnableAt,

    /// The user JSON data
    Data,
}
