// setup
use sea_orm::sea_query::{ForeignKeyCreateStatement, ColumnDef, TableCreateStatement};
use sea_orm::{error::*, sea_query, ConnectionTrait, DbConn, ExecResult};
pub mod comment;
pub mod post;
pub mod helper;
pub use comment::Entity as Comment;
pub use post::Entity as Post;

pub mod prelude {
    pub use crate::test_setup::helper::{
        create_actix_admin_builder, 
        setup_db, 
        AppState,
        BodyTest
    };
    pub use super::comment;
    pub use super::post;
    pub use super::Comment;
    pub use super::Post;
}

// setup
async fn create_table(db: &DbConn, stmt: &TableCreateStatement) -> Result<ExecResult, DbErr> {
    let builder = db.get_database_backend();
    db.execute(builder.build(stmt)).await
}

pub async fn create_tables(db: &DbConn) -> Result<ExecResult, DbErr> {
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
        .col(ColumnDef::new(post::Column::InsertDate).date())
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
        .col(ColumnDef::new(comment::Column::MyDecimal).decimal().not_null())
        .col(ColumnDef::new(comment::Column::PostId).integer())
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-comment-post")
                .from_tbl(Comment)
                .from_col(comment::Column::PostId)
                .to_tbl(Post)
                .to_col(post::Column::Id),
        )
        .to_owned();

    create_table(db, &stmt).await
}
