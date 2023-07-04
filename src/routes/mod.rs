mod create_or_edit_get;
pub use create_or_edit_get::{create_get, edit_get};

mod create_or_edit_post;
pub use create_or_edit_post::{ create_post, edit_post, create_or_edit_post };

mod index;
pub use index::{ index, not_found, get_admin_ctx };

mod list;
pub use list::{ list, SortOrder };

mod show;
pub use show::show;

mod delete;
pub use delete::{ delete, delete_many };

mod helpers;
pub use helpers::{ add_auth_context, user_can_access_page, render_unauthorized };

mod file;
pub use file::{download, delete_file};

use serde_derive::{Deserialize};
#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    entities_per_page: Option<u64>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<SortOrder>
}

const DEFAULT_ENTITIES_PER_PAGE: u64 = 10;