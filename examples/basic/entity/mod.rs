// setup
use sea_orm::sea_query::{ColumnDef, ForeignKeyCreateStatement, TableCreateStatement};
use sea_orm::{
    error::*, sea_query, ConnectionTrait, DbConn, EntityTrait, ExecResult, Set};
pub mod comment;
pub mod post;
pub mod user;
use chrono::{Duration, DurationRound, Local};
pub use comment::Entity as Comment;
pub use post::Entity as Post;
use sea_orm::prelude::Decimal;
pub use user::Entity as User;

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
        .col(
            ColumnDef::new(post::Column::TeaMandatory)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(post::Column::TeaOptional).string())
        .col(ColumnDef::new(post::Column::InsertDate).date().not_null())
        .col(ColumnDef::new(post::Column::Attachment).string())
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

    let _res = create_table(db, &stmt).await;

    let stmt = sea_query::Table::create()
        .table(user::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(post::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(user::Column::Name).string().not_null())
        .to_owned();

    let _res = create_table(db, &stmt).await;

    for i in 1..1000 {
        let row = post::ActiveModel {
            title: Set(format!("Test {}", i)),
            text: Set("some content".to_string()),
            tea_mandatory: Set(post::Tea::EverydayTea),
            tea_optional: Set(None),
            insert_date: Set(Local::now().date_naive()),
            ..Default::default()
        };
        let _res = Post::insert(row).exec(db).await;
    }

    for i in 1..1000 {
        let row = comment::ActiveModel {
            comment: Set(format!("Test {}", i)),
            user: Set("me@home.com".to_string()),
            my_decimal: Set(Decimal::new(105, 0)),
            insert_date: Set(Local::now()
                .naive_utc()
                .duration_round(Duration::minutes(1))
                .unwrap()),
            is_visible: Set(i % 2 == 0),
            ..Default::default()
        };
        let _res = Comment::insert(row).exec(db).await;
    }

    for i in 1..100 {
        let row = user::ActiveModel {
            name: Set(format!("user {}", i)),
            ..Default::default()
        };
        let _res = User::insert(row).exec(db).await;
    }

    _res
}
