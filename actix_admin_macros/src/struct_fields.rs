use crate::attributes::derive_attr;
use crate::model_fields::ModelField;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{DeriveInput, Fields, LitStr, Ident, parse_str, Type, LitInt};

pub fn get_fields_for_tokenstream(input: proc_macro::TokenStream) -> std::vec::Vec<ModelField> {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("FieldNames can only be derived for structs"),
    });
    fields
}

fn capitalize_first_letter(s: &str) -> String {
    s.split('_')
        .map(|word| {
            if word.len() > 0 {
                word[0..1].to_uppercase() + &word[1..]
            } else {
                String::new()
            }
        })
        .collect::<Vec<_>>()
        .join("")
}


fn to_camelcase(s: &str) -> String {
    s.split("_").fold(String::new(), |a, b| capitalize_first_letter(&a) + &capitalize_first_letter(b))
}

pub fn filter_fields(fields: &Fields) -> Vec<ModelField> {
    fields
        .iter()
        .filter_map(|field| {
            let actix_admin_attr =
                derive_attr::ActixAdmin::try_from_attributes(&field.attrs).unwrap_or_default();

            if field.ident.is_some() {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
                let inner_type = extract_type_from_option(&field.ty);
                let field_ty = field.ty.to_owned();
                let is_primary_key = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.primary_key.is_some());
                let foreign_key = actix_admin_attr.clone()
                    .and_then(|attr| attr.foreign_key)
                    .and_then(|attr_field| LitStr::from(attr_field).value().parse().ok());
                let is_searchable = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.searchable.is_some());
                let round = actix_admin_attr.clone()
                    .and_then(|attr| attr.round)
                    .and_then(|attr_field| LitStr::from(attr_field).value().parse().ok());
                let shorten = actix_admin_attr.clone()
                    .and_then(|attr| attr.shorten)
                    .and_then(|attr_field| attr_field.base10_parse().ok());
                let is_textarea = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.textarea.is_some());
                let is_file_upload = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.file_upload.is_some());
                let is_list_hide_column = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.list_hide_column.is_some() || attr.tenant_ref.is_some());
                let is_tenant_ref = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.tenant_ref.is_some());
                let is_not_empty = actix_admin_attr
                    .clone()
                    .map_or(false, |attr| attr.not_empty.is_some());
                let list_regex_mask = actix_admin_attr.clone().map_or("".to_string(), |attr| {
                    attr.list_regex_mask
                        .map_or("".to_string(), |attr_field| {
                            (LitStr::from(attr_field)).value()
                        })
                });
                let list_sort_position: usize = actix_admin_attr.clone().map_or(99, |attr| {
                    attr.list_sort_position.map_or( 99, |attr_field| {
                        let sort_pos = LitStr::from(attr_field).value().parse::<usize>();
                        match sort_pos {
                            Ok(pos) => pos,
                            _ => 99
                        }
                    })
                });
                let select_list = actix_admin_attr.clone().map_or("".to_string(), |attr| {
                    attr.select_list.map_or("".to_string(), |attr_field| {
                        (LitStr::from(attr_field)).value()
                    })
                });
                let html_input_type = actix_admin_attr.map_or("".to_string(), |attr| {
                    attr.html_input_type
                        .map_or("".to_string(), |attr_field| {
                            (LitStr::from(attr_field)).value()
                        })
                });

                let model_field = ModelField {
                    visibility: field_vis,
                    ident: field_ident,
                    ty: field_ty,
                    inner_type: inner_type,
                    primary_key: is_primary_key,
                    foreign_key: foreign_key,
                    html_input_type: html_input_type,
                    select_list: select_list,
                    searchable: is_searchable,
                    textarea: is_textarea,
                    file_upload: is_file_upload,
                    not_empty: is_not_empty,
                    list_sort_position: list_sort_position,
                    list_hide_column: is_list_hide_column,
                    list_regex_mask: list_regex_mask,
                    tenant_ref: is_tenant_ref,
                    round: round,
                    shorten: shorten
                };
                Some(model_field)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn extract_type_from_option(ty: &syn::Type) -> Option<syn::Type> {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty.to_owned()),
            _ => None,
        })
}

pub fn get_fields_as_tokenstream<T: ToTokens>(fields: &Vec<ModelField>, accessor: fn(&ModelField) -> T) -> Vec<TokenStream> {
    fields
    .iter()
    .filter(|model_field| !model_field.primary_key)
    .filter(|model_field| !model_field.tenant_ref)
    .map(|model_field| {
        let ident_name = accessor(model_field);

        quote! {
            #ident_name
        }
    })
    .collect::<Vec<_>>()
}

pub fn get_match_name_to_column(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
    .iter()
    .map(|model_field| {
        let column_name = model_field.ident.to_string();
        let column_name_capitalized = to_camelcase(&column_name);
        let column_ident = Ident::new(&column_name_capitalized, Span::call_site());
        quote! {
            #column_name => Column::#column_ident,
        }
    })
    .collect::<Vec<_>>()
}

pub fn get_actix_admin_fields_searchable(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|model_field| model_field.searchable)
        .map(|model_field| {
            let column_name = capitalize_first_letter(&model_field.ident.to_string());
            let column_ident = Ident::new(&column_name, Span::call_site());
            quote! {
                .add(Column::#column_ident.contains(&params.search))
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_set_tenant_ref_field(fields: &Vec<ModelField>) -> TokenStream {
    let tenant_ref_fields: Vec<&ModelField> = fields.iter().filter(|model_field| model_field.tenant_ref).collect();

    match tenant_ref_fields.len() {
        0 => quote! {},
        1 => {
            let tenant_ref_field = tenant_ref_fields[0];
            let column_ident = Ident::new(&tenant_ref_field.ident.to_string(), Span::call_site());
            quote! { if let Some(tenant_ref) = tenant_ref { active_model.#column_ident = Set(tenant_ref); } }
        }
        _ => panic!("Model has multiple tenant_ref fields, but only one is allowed"),
    }
}

pub fn get_tenant_ref_field(fields: &Vec<ModelField>, wrap_in_params: bool) -> TokenStream {
    let tenant_ref_fields: Vec<&ModelField> = fields.iter().filter(|model_field| model_field.tenant_ref).collect();

    match tenant_ref_fields.len() {
        0 => quote! {},
        1 => {
            let tenant_ref_field = tenant_ref_fields[0];
            let column_ident = Ident::new(&capitalize_first_letter(&tenant_ref_field.ident.to_string()), Span::call_site());
            let tenant_ref = if wrap_in_params { quote! { params.tenant_ref } } else { quote! { tenant_ref } };
            quote! {
                if #tenant_ref.is_some() {
                    query = query.filter(Column::#column_ident.eq(#tenant_ref.unwrap()));
                }
            }
        }
        _ => panic!("Model has multiple tenant_ref fields, but only one is allowed"),
    }
}

pub fn get_field_for_primary_key(fields: &Vec<ModelField>) -> TokenStream {
    let primary_key_model_field = fields
        .iter()
        // TODO: filter id attr based on struct attr or sea_orm primary_key attr
        .find(|model_field| model_field.primary_key)
        .expect("model must have a single primary key");

    let ident = primary_key_model_field.ident.to_owned();

    quote! {
        primary_key: Some(model.#ident.to_string())
    }
}

pub fn get_primary_key_field_name(fields: &Vec<ModelField>) -> String {
    let primary_key_model_field = fields
        .iter()
        // TODO: filter id attr based on struct attr or sea_orm primary_key attr
        .find(|model_field| model_field.primary_key)
        .expect("model must have a single primary key");

    primary_key_model_field.ident.to_string()
}

fn split_at_uppercase(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;

    for (i, c) in input.char_indices() {
        if i > start && c.is_ascii_uppercase() {
            parts.push(&input[start..i]);
            start = i;
        }
    }

    if start < input.len() {
        parts.push(&input[start..]);
    }

    parts
}

fn combine_uppercase_with_underscore(strings: Vec<&str>) -> String {
    let mut result = Vec::new();

    for (i, s) in strings.iter().enumerate() {
        if s.chars().next().map(char::is_uppercase) == Some(true) && i > 0 {
            if !result.last().map(|s: &String| s.ends_with("::")).unwrap_or(false) {
                let combined = format!("{}_{}", result.pop().unwrap(), s);
                result.push(combined);
                continue;
            }
        }

        result.push(s.to_string());
    }

    result.concat().to_lowercase()
}

pub fn get_fields_for_load_foreign_key(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields.iter()
        .filter_map(|model_field| model_field.foreign_key.as_ref())
        .map(|fk| {
            let ty = parse_str::<Type>(fk).unwrap();
            let split = combine_uppercase_with_underscore(split_at_uppercase(fk));
            let ty2 = parse_str::<Type>(&split).unwrap();
            quote! {
                #fk => #ty::find().filter(#ty2::Column::Id.is_in(ids_to_select)).all(db).await
                    .ok()
                    .map(|models| models.iter().map(|m| (m.id.to_string(), format!("{}", m))).collect::<HashMap<_, _>>()),
            }
        })
        .chain(std::iter::once(quote! {
            _ => None,
        }))
        .collect()
}

pub fn get_fields_for_from_model(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|model_field| !model_field.primary_key)
        .map(|model_field| {
            let ident_name = model_field.ident.to_string();
            let ident = model_field.ident.to_owned();

            match model_field.is_option() {
                true => {
                    quote! {
                        #ident_name => match model.#ident {
                            Some(val) => val.to_string().trim_start_matches("'").trim_end_matches("'").to_string(),
                            None => "".to_owned()
                        }
                    }
                }
                false => {
                    quote! {
                        #ident_name => model.#ident.to_string().trim_start_matches("'").trim_end_matches("'").to_string()
                    }
                }
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_fields_for_validate_model(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields.iter()
        .filter(|model_field| !model_field.primary_key && !model_field.tenant_ref)
        .map(|model_field| {
            let ident_name = model_field.ident.to_string();
            let ty = model_field.ty.to_owned();
            let type_path = model_field.get_type_path_string();
            let is_option_or_string = model_field.is_option() || model_field.is_string();
            let is_allowed_to_be_empty = !model_field.not_empty;

            let res = match (model_field.is_option(), type_path.as_str()) {
                (_, "DateTime") => quote! { model.get_datetime(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).map_err(|err| errors.insert(#ident_name.to_string(), err)).ok(); },
                (_, "Date") => quote! { model.get_date(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).map_err(|err| errors.insert(#ident_name.to_string(), err)).ok(); },
                (_, "bool") => quote! { model.get_bool(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).map_err(|err| errors.insert(#ident_name.to_string(), err)).ok(); },
                (true, _) => {
                    let inner_ty = model_field.inner_type.to_owned().unwrap();
                    quote! { model.get_value::<#inner_ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).map_err(|err| errors.insert(#ident_name.to_string(), err)).ok(); }
                },
                (false, _) => quote! { model.get_value::<#ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).map_err(|err| errors.insert(#ident_name.to_string(), err)).ok(); }
            };

            res
        })
        .collect()
}

pub fn get_fields_for_create_model(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        // TODO: filter id attr based on struct attr or sea_orm primary_key attr
        .filter(|model_field| !model_field.primary_key)
        .filter(|model_field| !model_field.tenant_ref)
        .map(|model_field| {
            let ident_name = model_field.ident.to_string();
            let ident = model_field.ident.to_owned();
            let ty = model_field.ty.to_owned();
            let type_path = model_field.get_type_path_string();

            let is_option_or_string = model_field.is_option() || model_field.is_string();
            let is_allowed_to_be_empty = !model_field.not_empty;

            let res = match (model_field.is_option(), model_field.is_string(), type_path.as_str()) {
                // is DateTime
                (true , _, "DateTime") => {
                    quote! {
                        #ident: Set(model.get_datetime(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap())
                    }
                },
                (false , _, "DateTime") => {
                    quote! {
                        #ident: Set(model.get_datetime(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                },
                (true , _, "Date") => {
                    quote! {
                        #ident: Set(model.get_date(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap())
                    }
                },
                (false , _, "Date") => {
                    quote! {
                        #ident: Set(model.get_date(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                },
                (_ , _, "bool") => {
                    quote! {
                        #ident: Set(model.get_bool(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                },
                // Default fields
                (true, _, _) => {
                    let inner_ty = model_field.inner_type.to_owned().unwrap();
                    quote! {
                        #ident: Set(model.get_value::<#inner_ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap())
                    }
                },
                // is string which can be empty
                (false, true, _) => {
                    quote! {
                        #ident: Set(model.get_value::<#ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap_or(String::new()))
                    }
                },
                // no string
                (false, false, _) => {
                    quote! {
                        #ident: Set(model.get_value::<#ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                }
            };

            res
        })
        .collect::<Vec<_>>()
}

pub fn get_fields_for_edit_model(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        // TODO: filter id attr based on struct attr or sea_orm primary_key attr
        .filter(|model_field| !model_field.primary_key)
        .filter(|model_field| !model_field.tenant_ref)
        .map(|model_field| {
            let ident_name = model_field.ident.to_string();
            let ident = model_field.ident.to_owned();
            let ty = model_field.ty.to_owned();
            let type_path = model_field.get_type_path_string();

            let is_option_or_string = model_field.is_option() || model_field.is_string();
            let is_allowed_to_be_empty = !model_field.not_empty;
            
            let res = match (model_field.is_option(), model_field.is_string(), type_path.as_str()) {
                (_, _, "bool") => {
                    quote! {
                        entity.#ident = Set(model.get_bool(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                },
                (true , _, "DateTime") => {
                    quote! {
                        entity.#ident = Set(model.get_datetime(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap())
                    }
                },
                (false , _, "DateTime") => {
                    quote! {
                        entity.#ident = Set(model.get_datetime(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                },
                (true , _, "Date") => {
                    quote! {
                        entity.#ident = Set(model.get_date(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap())
                    }
                },
                (false , _, "Date") => {
                    quote! {
                        entity.#ident = Set(model.get_date(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                },
                (true, _, _) => {
                    let inner_ty = model_field.inner_type.to_owned().unwrap();
                    quote! {
                        entity.#ident = Set(model.get_value::<#inner_ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap())
                    }
                },
                (false, true, _) => {
                    quote! {
                        entity.#ident = Set(model.get_value::<#ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap_or(String::new()))
                    }
                },
                (false, false, _) => {
                    quote! {
                        entity.#ident = Set(model.get_value::<#ty>(#ident_name, #is_option_or_string, #is_allowed_to_be_empty).unwrap().unwrap())
                    }
                }
            };

            res
        })
        .collect::<Vec<_>>()
}
