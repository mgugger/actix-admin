use actix_admin::model::ActixAdminModelFilterTrait;
use actix_admin::prelude::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(
    Clone,
    Debug,
    PartialEq,
    DeriveEntityModel,
    Deserialize,
    Serialize,
    DeriveActixAdmin,
    DeriveActixAdminViewModel,
    DeriveActixAdminModel,
    DeriveActixAdminModelSelectList,
)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[actix_admin(primary_key)]
    pub id: i32,
    #[actix_admin(searchable, not_empty)]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    #[actix_admin(searchable, textarea, list_hide_column)]
    pub text: String,
    #[actix_admin(select_list = "Tea")]
    pub tea_mandatory: Tea,
    #[actix_admin(select_list = "Tea")]
    pub tea_optional: Option<Tea>,
    #[sea_orm(column_type = "Date")]
    #[actix_admin(list_sort_position = "1")]
    pub insert_date: Date,
    #[actix_admin(file_upload)]
    pub attachment: Option<String>,

    // --- New nullable columns exercising every new field type ---
    /// Renders a small snippet of HTML (marked as safe) directly in the
    /// list and show views. Never enable this on user-controlled input.
    #[actix_admin(html_render)]
    pub summary_html: Option<String>,

    /// Rendered as a clickable `<a href>` in the list/show views.
    #[actix_admin(url)]
    pub homepage: Option<String>,

    /// Rendered as a `mailto:` link in the list/show views.
    #[actix_admin(email)]
    pub contact_email: Option<String>,

    /// Filename of an uploaded image; renders a thumbnail on the list
    /// view and a larger preview on show / edit.
    #[actix_admin(image)]
    pub cover_image: Option<String>,

    /// Markdown-editable rich text (EasyMDE) on create/edit.
    #[actix_admin(wysiwyg)]
    pub notes_md: Option<String>,

    /// Read-only field on the create/edit form (still visible on list/show).
    #[actix_admin(readonly)]
    pub external_id: Option<String>,
}

impl Display for Model {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            _ => write!(
                formatter,
                "{} {}",
                self.title, "" /* &self.insert_date*/
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
    Debug,
    Clone,
    PartialEq,
    EnumIter,
    DeriveDisplay,
    DeriveActiveEnum,
    Deserialize,
    Serialize,
    DeriveActixAdminEnumSelectList,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tea")]
pub enum Tea {
    #[sea_orm(string_value = "EverydayTea")]
    EverydayTea,
    #[sea_orm(string_value = "BreakfastTea")]
    BreakfastTea,
}

impl FromStr for Tea {
    type Err = ();

    fn from_str(input: &str) -> Result<Tea, Self::Err> {
        match input {
            "EverydayTea" => Ok(Tea::EverydayTea),
            "BreakfastTea" => Ok(Tea::BreakfastTea),
            _ => Err(()),
        }
    }
}

impl ActixAdminModelValidationTrait<ActiveModel> for Entity {}

impl ActixAdminModelFilterTrait<Entity> for Entity {}

// Custom bulk action registered via `add_bulk_action_for_entity` in main.rs.
// This implementation simply logs which ids were selected; a real handler
// would run an update via SeaORM. Note the trait implementation is generated
// with a default empty impl by `DeriveActixAdminViewModel` — we override it
// here.
#[actix_admin::prelude::async_trait(?Send)]
impl actix_admin::routes::ActixAdminBulkActionDispatch for Entity {
    async fn run_bulk_action(
        name: &str,
        _db: &sea_orm::DatabaseConnection,
        ids: Vec<Self::Id>,
        _tenant_ref: Option<i32>,
    ) -> Result<Option<String>, ActixAdminError> {
        match name {
            "mark_reviewed" => Ok(Some(format!("marked {} post(s) as reviewed", ids.len()))),
            _ => Ok(None),
        }
    }
}
