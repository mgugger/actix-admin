// setup
use sea_orm::sea_query::{ColumnDef, ForeignKeyCreateStatement, TableCreateStatement};
use sea_orm::{error::*, sea_query, ConnectionTrait, DbConn, ExecResult};
pub mod comment;
pub mod helper;
pub mod post;
pub mod sample_with_tenant_id;
pub mod webdriver;
pub use comment::Entity as Comment;
pub use post::Entity as Post;
pub use sample_with_tenant_id::Entity as SampleWithTenantId;

#[allow(dead_code)]
#[allow(unused_imports)]
pub mod prelude {
    pub use super::*;
    pub use crate::test_setup::helper::{create_actix_admin_builder, setup_db, BodyTest};
    pub use crate::test_setup::webdriver::{setup, teardown, wait_for_url_contains};
}

// setup
async fn create_table(db: &DbConn, stmt: &TableCreateStatement) -> Result<ExecResult, DbErr> {
    db.execute(stmt).await
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
        .col(
            ColumnDef::new(post::Column::TeaMandatory)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(post::Column::TeaOptional).string())
        .col(ColumnDef::new(post::Column::InsertDate).date())
        .col(ColumnDef::new(post::Column::SummaryHtml).string())
        .col(ColumnDef::new(post::Column::Homepage).string())
        .col(ColumnDef::new(post::Column::ContactEmail).string())
        .col(ColumnDef::new(post::Column::CoverImage).string())
        .col(ColumnDef::new(post::Column::NotesMd).string())
        .col(ColumnDef::new(post::Column::ExternalId).string())
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
        .col(
            ColumnDef::new(comment::Column::InsertDate)
                .date_time()
                .not_null(),
        )
        .col(
            ColumnDef::new(comment::Column::IsVisible)
                .boolean()
                .not_null(),
        )
        .col(
            ColumnDef::new(comment::Column::MyDecimal)
                .decimal()
                .not_null(),
        )
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

    let _result = create_table(db, &stmt).await;

    let stmt = sea_query::Table::create()
        .table(SampleWithTenantId)
        .if_not_exists()
        .col(
            ColumnDef::new(sample_with_tenant_id::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(sample_with_tenant_id::Column::Title)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(sample_with_tenant_id::Column::Text)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(sample_with_tenant_id::Column::TenantId)
                .integer()
                .not_null(),
        )
        .to_owned();

    let _result = create_table(db, &stmt).await;

    _result
}
