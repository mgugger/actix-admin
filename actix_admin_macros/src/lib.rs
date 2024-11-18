//! # Actix Admin Macros
//!
//! Macros used by the actix-admin crate

use proc_macro;
use quote::quote;

mod struct_fields;
use struct_fields::*;

mod selectlist_fields;
use selectlist_fields::{get_select_list_from_enum, get_select_list_from_model, get_select_lists};

mod attributes;
mod model_fields;

#[proc_macro_derive(DeriveActixAdminEnumSelectList, attributes(actix_admin))]
pub fn derive_actix_admin_enum_select_list(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    get_select_list_from_enum(input)
}

#[proc_macro_derive(DeriveActixAdminModelSelectList, attributes(actix_admin))]
pub fn derive_actix_admin_model_select_list(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    get_select_list_from_model(input)
}

#[proc_macro_derive(DeriveActixAdmin, attributes(actix_admin))]
pub fn derive_actix_admin(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote! {
        use std::convert::From;
        use actix_admin::prelude::*;
        use sea_orm::{
            ActiveValue::Set,
            ConnectOptions,
            DatabaseConnection,
            entity::*,
            query::*,
            EntityTrait
        };
        use std::collections::HashMap;
        use regex::Regex;
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(DeriveActixAdminViewModel, attributes(actix_admin))]
pub fn derive_actix_admin_view_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);

    let name_primary_field_str = get_primary_key_field_name(&fields);
    let fields_for_edit_model = get_fields_for_edit_model(&fields);
    let fields_searchable = get_actix_admin_fields_searchable(&fields);
    let has_searchable_fields = fields_searchable.len() > 0;

    let select_lists = get_select_lists(&fields);

    let tenant_ref_field = get_tenant_ref_field(&fields, false);

    let set_tenant_ref_field = get_set_tenant_ref_field(&fields);

    let expanded = quote! {
        impl From<Entity> for ActixAdminViewModel {
            fn from(entity: Entity) -> Self {
                ActixAdminViewModel {
                    primary_key: #name_primary_field_str.to_string(),
                    entity_name: entity.table_name().to_string(),
                    fields: Entity::get_fields(),
                    show_search: #has_searchable_fields,
                    user_can_access: None,
                    default_show_aside: Entity::get_filter().len() > 0,
                    inline_edit: false
                }
            }
        }

        #[actix_admin::prelude::async_trait(?Send)]
        impl ActixAdminViewModelTrait for Entity {
            async fn list(db: &DatabaseConnection, params: &ActixAdminViewModelParams) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError> {
                let filter_values: HashMap<String, Option<String>> = params.viewmodel_filter.iter().map(|f| (f.name.to_string(), f.value.clone())).collect();
                let entities = Entity::list_model(db, params, filter_values).await;
                entities
            }

            fn validate_entity(model: &mut ActixAdminModel) {
                Entity::validate_model(model);

                if !model.has_errors() {
                    let active_model = ActiveModel::from(model.clone());
                    let custom_errors = Entity::validate(&active_model);
                    model.custom_errors = custom_errors;
                }
            }

            async fn create_entity(db: &DatabaseConnection, mut model: ActixAdminModel, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError> {
                let mut active_model = ActiveModel::from(model.clone());

                #set_tenant_ref_field

                let insert_operation = Entity::insert(active_model).exec(db).await?;
                model.primary_key = Some(insert_operation.last_insert_id.to_string());

                Ok(model)
            }

            async fn get_viewmodel_filter(db: &DatabaseConnection) -> HashMap<String, ActixAdminViewModelFilter> {
                let mut hashmap: HashMap<String, ActixAdminViewModelFilter> = HashMap::new();

                for filter in Entity::get_filter() {
                    hashmap.insert(
                        filter.name.to_string(),
                        ActixAdminViewModelFilter {
                            name: filter.name.to_string(),
                            value: None,
                            values: Entity::get_filter_values(&filter, db).await,
                            filter_type: Some(filter.filter_type)
                        }
                    );
                };

                hashmap
            }

            async fn get_entity(db: &DatabaseConnection, id: i32, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError> {
                let mut query = Entity::find().filter(Column::Id.eq(id));

                #tenant_ref_field

                let entity = query.one(db).await?;

                match entity {
                    Some(e) => {
                        let model = ActixAdminModel::from(e);
                        let mut model_entities = Vec::<ActixAdminModel>::new();
                        model_entities.push(model);
                        let _load_fks = Self::load_foreign_keys(&mut model_entities, db).await;
                        Ok(model_entities.pop().unwrap())
                    },
                    _ => Err(ActixAdminError {
                        ty: ActixAdminErrorType::EntityDoesNotExistError,
                        msg: "".to_string()
                    })
                }
            }

            async fn edit_entity(db: &DatabaseConnection, id: i32, mut model: ActixAdminModel, tenant_ref: Option<i32>) -> Result<ActixAdminModel, ActixAdminError> {
                let mut query = Entity::find().filter(Column::Id.eq(id));

                #tenant_ref_field

                let entity = query.one(db).await?;

                match entity {
                    Some(e) => {
                        let mut entity: ActiveModel = e.into();
                        #(#fields_for_edit_model);*;
                        let entity: Model = entity.update(db).await?;
                        Ok(model)
                    },
                    _ => Err(ActixAdminError {
                        ty: ActixAdminErrorType::EntityDoesNotExistError,
                        msg: "".to_string()
                    })
                }
            }

            async fn delete_entity(db: &DatabaseConnection, id: i32, tenant_ref: Option<i32>) -> Result<bool, ActixAdminError> {
                let mut query = Entity::delete_many().filter(Column::Id.eq(id));

                #tenant_ref_field

                let del_result = query.exec(db).await?;

                if del_result.rows_affected > 0 {
                    return Ok(true)
                } else {
                    return Err(ActixAdminError {
                        ty: ActixAdminErrorType::DeleteError,
                        msg: "".to_string()
                    })
                }
            }

            async fn get_select_lists(db: &DatabaseConnection, tenant_ref: Option<i32>) -> Result<HashMap<String, Vec<(String, String)>>, ActixAdminError> {
                Ok(hashmap![
                    #(#select_lists),*
                ])
            }

            fn get_entity_name() -> String {
                Entity.table_name().to_string()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(DeriveActixAdminModel, attributes(actix_admin))]
pub fn derive_actix_admin_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fields = get_fields_for_tokenstream(input);

    let field_names = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.ident.to_string()
    });
    let field_html_input_type = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.html_input_type.to_string()
    });
    let field_ceil = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.ceil.clone().unwrap_or("".to_string())
    });
    let field_floor = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.floor.clone().unwrap_or("".to_string())
    });
    let field_dateformat = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.dateformat.to_string()
    });
    let field_shorten = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.shorten.clone().unwrap_or("".to_string())
    });
    let field_foreign_key = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.foreign_key.clone().unwrap_or("".to_string())
    });
    let field_list_regex_mask = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.list_regex_mask.to_string()
    });
    let field_select_list = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.select_list.to_string()
    });
    let is_option_list =
        get_fields_as_tokenstream(&fields, |model_field| -> bool { model_field.is_option() });
    let fields_for_create_model = get_fields_for_create_model(&fields);
    let fields_for_from_model = get_fields_for_from_model(&fields);
    let fields_for_load_foreign_key = get_fields_for_load_foreign_key(&fields);
    let field_for_primary_key = get_field_for_primary_key(&fields);
    let fields_for_validate_model = get_fields_for_validate_model(&fields);
    let fields_type_path = get_fields_as_tokenstream(&fields, |model_field| -> String {
        model_field.get_type_path_string()
    });
    let fields_textarea =
        get_fields_as_tokenstream(&fields, |model_field| -> bool { model_field.textarea });
    let fields_file_upload =
        get_fields_as_tokenstream(&fields, |model_field| -> bool { model_field.file_upload });
    let fields_match_name_to_columns = get_match_name_to_column(&fields);
    let fields_list_sort_positions = get_fields_as_tokenstream(&fields, |model_field| -> usize {
        model_field.list_sort_position
    });
    let fields_list_hide_column = get_fields_as_tokenstream(&fields, |model_field| -> bool {
        model_field.list_hide_column
    });
    let fields_use_tom_select_callback = get_fields_as_tokenstream(&fields, |model_field| -> bool {
        model_field.use_tom_select_callback
    });
    let fields_tenant_ref =
        get_fields_as_tokenstream(&fields, |model_field| -> bool { model_field.tenant_ref });
    let fields_searchable = get_actix_admin_fields_searchable(&fields);
    let has_searchable_fields = fields_searchable.len() > 0;
    let tenant_ref_field = get_tenant_ref_field(&fields, true);

    let expanded = quote! {
        actix_admin::prelude::lazy_static! {
            pub static ref ACTIX_ADMIN_VIEWMODEL_FIELDS: Vec<ActixAdminViewModelField> = {
                let mut vec = Vec::new();

            #(
                let field_name = stringify!(#field_names).replace("\"", "").replace(' ', "").replace('\n', "");
                let html_input_type = stringify!(#field_html_input_type).replace("\"", "").replace(' ', "");
                let select_list = stringify!(#field_select_list).replace("\"", "").replace(' ', "");
                let list_regex_mask_regex = match &stringify!(#field_list_regex_mask) {
                    s if !s.is_empty() => Some(Regex::new(s).unwrap()),
                    _ => None,
                };
                let dateformat = match &stringify!(#field_dateformat).replace("\"", "").trim_start().trim_end().replace('\n', "") {
                    s if !s.is_empty() => Some(s.to_string()),
                    _ => None,
                };
                let ceil = match &stringify!(#field_ceil).replace("\"", "").replace(' ', "").replace('\n', "") {
                    s if !s.is_empty() => s.parse::<u8>().ok(),
                    _ => None,
                };
                let floor = match &stringify!(#field_floor).replace("\"", "").replace(' ', "").replace('\n', "") {
                    s if !s.is_empty() => s.parse::<u8>().ok(),
                    _ => None,
                };
                let shorten = match &stringify!(#field_shorten).replace("\"", "").replace(' ', "").replace('\n', "") {
                    s if !s.is_empty() => s.parse::<u16>().ok(),
                    _ => None,
                };

                vec.push(ActixAdminViewModelField {
                    field_name: field_name.clone(),
                    html_input_type: html_input_type.clone(),
                    select_list: select_list.clone(),
                    is_option: #is_option_list,
                    list_sort_position: #fields_list_sort_positions,
                    field_type: ActixAdminViewModelFieldType::get_field_type(#fields_type_path, select_list.clone(), #fields_textarea, #fields_file_upload),
                    list_hide_column: #fields_list_hide_column,
                    list_regex_mask: list_regex_mask_regex,
                    foreign_key: stringify!(#field_foreign_key).to_string(),
                    is_tenant_ref: #fields_tenant_ref,
                    ceil: ceil,
                    floor: floor,
                    dateformat: dateformat,
                    shorten: shorten,
                    use_tom_select_callback: #fields_use_tom_select_callback
                });
            )*

            vec
        };

        }

        impl From<Model> for ActixAdminModel {
            fn from(model: Model) -> Self {
                let display_name = model.clone().to_string();
                ActixAdminModel {
                    #field_for_primary_key,
                    values: hashmap![
                        #(#fields_for_from_model),*
                    ],
                    errors: HashMap::new(),
                    custom_errors: HashMap::new(),
                    fk_values: HashMap::new(),
                    display_name: Some(display_name)
                }
            }
        }

        impl From<ActixAdminModel> for ActiveModel {
            fn from(model: ActixAdminModel) -> Self {
                ActiveModel
                {
                    #(#fields_for_create_model),*
                    ,
                    ..Default::default()
                }
            }
        }

        #[actix_admin::prelude::async_trait]
        impl ActixAdminModelTrait for Entity {
            async fn list_model(db: &DatabaseConnection, params: &ActixAdminViewModelParams, filter_values: HashMap<String, Option<String>>) -> Result<(Option<u64>, Vec<ActixAdminModel>), ActixAdminError> {

                let sort_column = match params.sort_by.as_ref() {
                    #(#fields_match_name_to_columns)*
                    _ => panic!("Unknown column")
                };

                let mut query = match params.sort_order {
                    SortOrder::Asc => Entity::find().order_by_asc(sort_column),
                    SortOrder::Desc =>  Entity::find().order_by_desc(sort_column),
                };

                if (#has_searchable_fields) {
                    query = query
                    .filter(
                        Condition::any()
                        #(#fields_searchable)*
                    )
                }

                #tenant_ref_field

                let filters = Entity::get_filter();
                for filter in filters {
                    let myfn = filter.filter;
                    let value = filter_values.get(&filter.name).unwrap_or_else(|| &None);
                    query = myfn(query, value.clone());
                }

                let mut entities;
                let mut model_entities = Vec::<ActixAdminModel>::new();
                let num_pages: Option<u64>;

                match (params.page, params.entities_per_page) {
                    (Some(p), Some(e)) => {
                        let paginator = query.paginate(db, e);
                        num_pages = Some(paginator.num_pages().await?);

                        if (num_pages.unwrap() == 0) { return Ok((num_pages, model_entities)) };
                        entities = paginator
                            .fetch_page(std::cmp::min(num_pages.unwrap() - 1, p - 1))
                            .await?;
                    },
                    (_, _) => {
                        entities = query.all(db).await?;
                        num_pages = None;
                    }
                };

                for entity in entities {
                    model_entities.push(
                        ActixAdminModel::from(entity)
                    );
                }

                let _load_fks = Self::load_foreign_keys(&mut model_entities, db).await;

                Ok((num_pages, model_entities))
            }

            async fn load_foreign_keys(models: &mut Vec<ActixAdminModel>, db: &DatabaseConnection) {
                for field in Self::get_fields().iter() {

                    if field.foreign_key != "" {
                        let ids_to_select: Vec<i32> = models.iter()
                            .map(|m| m.values.get(&field.field_name))
                            .filter_map(|value| {
                                value.and_then(|s| s.parse().ok())
                            })
                            .collect();

                        let foreign_key_entity = field.foreign_key.trim_start_matches("'").trim_end_matches("'").replace('"', "").replace(' ', "").replace('\\', "").replace(' ', "").to_string();

                        let foreign_key_values_opt: Option<HashMap<String, String>> = match foreign_key_entity.as_str() {
                            #(#fields_for_load_foreign_key)*
                            _ => None
                        };

                        if foreign_key_values_opt.is_some() {
                            let foreign_key_values = foreign_key_values_opt.unwrap();
                            for model in models.iter_mut() {
                                let fk_id = model.values.get(&field.field_name).unwrap();
                                let fk_val = foreign_key_values.get(fk_id);
                                if fk_val.is_some() {
                                    model.fk_values.insert(field.field_name.to_string(), fk_val.unwrap().to_string());
                                }
                            }
                        }
                    }
                }
            }

            fn validate_model(model: &mut ActixAdminModel) {
                let mut errors = HashMap::<String, String>::new();
                #(#fields_for_validate_model);*

                model.errors = errors;
            }

            fn get_fields() -> &'static[ActixAdminViewModelField] {
                ACTIX_ADMIN_VIEWMODEL_FIELDS.as_slice()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
