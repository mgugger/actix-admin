mod create_or_edit_get;
pub use create_or_edit_get::{create_get, edit_get};

mod create_or_edit_post;
pub use create_or_edit_post::{create_or_edit_post, create_post, edit_post};

mod index;
pub use index::{get_admin_ctx, index, not_found};

mod list;
pub use list::{export_csv, list, SortOrder};

mod show;
pub use show::show;

mod delete;
pub use delete::{delete, delete_many};

mod bulk_action;
pub use bulk_action::{bulk_action, ActixAdminBulkActionDispatch};

pub mod query;
pub use query::{parse_filters_from_query, ListQuery, Params};

mod helpers;
pub use helpers::{
    add_auth_context, begin_route, forbid_if_denied, render_create_or_edit_form, render_template,
    render_unauthorized, user_can_access_page, user_can_perform, validate_sort_by,
    view_model_or_500, AdminAction, RouteCtx, RoutePrelude,
};

mod file;
pub use file::{delete_file, download};

mod card_grid;
pub use card_grid::display_card_grid;

mod search;
pub use search::search;

pub(crate) const DEFAULT_ENTITIES_PER_PAGE: u64 = 10;
