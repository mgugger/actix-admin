// setup
use sea_orm::sea_query::{ColumnDef, TableCreateStatement};
use sea_orm::{error::*, sea_query, ConnectionTrait, DbConn, ExecResult};
pub mod comment;
pub mod post;
pub use comment::Entity as Comment;
pub use post::Entity as Post;

// setup
async fn create_table(db: &DbConn, stmt: &TableCreateStatement) -> Result<ExecResult, DbErr> {
    let builder = db.get_database_backend();
    db.execute(builder.build(stmt)).await
}

pub async fn create_post_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(post::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(post::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(post::Column::Title).string().not_null())
        .col(ColumnDef::new(post::Column::Text).string().not_null())
        .col(ColumnDef::new(post::Column::TeaMandatory).string().not_null())
        .col(ColumnDef::new(post::Column::TeaOptional).string())
        .to_owned();

    let _result = create_table(db, &stmt).await;

    let stmt = sea_query::Table::create()
        .table(comment::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(post::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(comment::Column::Comment).string().not_null())
        .col(ColumnDef::new(comment::Column::User).string().not_null())
        .col(ColumnDef::new(comment::Column::InsertDate).date_time().not_null())
        .col(ColumnDef::new(comment::Column::IsVisible).boolean().not_null())
        .to_owned();

    create_table(db, &stmt).await
}
