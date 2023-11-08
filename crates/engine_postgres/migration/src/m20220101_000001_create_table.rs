use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Job::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Job::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Job::Name).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(Job::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Job::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Job::RunnableAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Job::Data)
                            .json_binary()
                            .not_null()
                            .default(Expr::value("{}")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Job::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Job {
    Table,

    /// the ID
    Id,

    /// A unique name for deduping
    Name,

    /// When the job was created
    CreatedAt,
    /// When the job was last updated
    UpdatedAt,

    /// Since when the job is runnable
    RunnableAt,

    /// The user JSON data
    Data,
}
