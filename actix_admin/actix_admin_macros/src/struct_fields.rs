use proc_macro2::{Span, Ident, TokenStream};
use syn::{
    Fields, DeriveInput, LitStr
};
use quote::quote;
use crate::attributes::derive_attr;
use crate::model_fields::{ ModelField };

pub fn get_fields_for_tokenstream(input: proc_macro::TokenStream) -> std::vec::Vec<ModelField> {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (_vis, ty, _generics) = (&ast.vis, &ast.ident, &ast.generics);
    //let _names_struct_ident = Ident::new(&(ty.to_string() + "FieldStaticStr"), Span::call_site());

    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("FieldNames can only be derived for structs"),
    });
    fields
}

pub fn filter_fields(fields: &Fields) -> Vec<ModelField> {
    fields
        .iter()
        .filter_map(|field| {
            let actix_admin_attr = derive_attr::ActixAdmin::try_from_attributes(&field.attrs).unwrap_or_default();
            
            if field.ident.is_some()
            {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
                let inner_type = extract_type_from_option(&field.ty);
                let field_ty = field.ty.to_owned();
                let is_primary_key = actix_admin_attr.clone().map_or(false, |attr| attr.primary_key.is_some());
                let select_list = actix_admin_attr.clone().map_or("".to_string(), |attr| attr.select_list.map_or("".to_string(), 
                    |attr_field| (LitStr::from(attr_field)).value()
                ));
                let html_input_type = actix_admin_attr.map_or("text".to_string(), |attr| attr.html_input_type.map_or("text".to_string(), 
                    |attr_field| (LitStr::from(attr_field)).value()
                ));

                let model_field = ModelField {
                    visibility: field_vis,
                    ident: field_ident,
                    ty: field_ty,
                    inner_type: inner_type,
                    primary_key: is_primary_key,
                    html_input_type: html_input_type,
                    select_list: select_list
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

pub fn get_actix_admin_fields(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|model_field| !model_field.primary_key)
        .map(|model_field| {
            let ident_name = model_field.ident.to_string();

            quote! {
                #ident_name
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_actix_admin_fields_html_input(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|model_field| !model_field.primary_key)
        .map(|model_field| {
            let html_input_type = model_field.html_input_type.to_string();

            quote! {
                #html_input_type
            }
        })
        .collect::<Vec<_>>()
}

pub fn get_actix_admin_fields_select_list(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|model_field| !model_field.primary_key)
        .map(|model_field| {
            let select_list = model_field.select_list.to_string();

            quote! {
                #select_list
            }
        })
        .collect::<Vec<_>>()
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
                    Some(val) => val.to_string(),
                    None => "".to_owned()
                }
            }
        },
        false => {
            quote! {
                #ident_name => model.#ident.to_string()
            }
        }
    }
    })
    .collect::<Vec<_>>()
}

pub fn get_fields_for_create_model(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
    .iter()
    // TODO: filter id attr based on struct attr or sea_orm primary_key attr
    .filter(|model_field| !model_field.primary_key)
    .map(|model_field| {
        let ident_name = model_field.ident.to_string();
        let ident = model_field.ident.to_owned();
        let ty = model_field.ty.to_owned();

        match model_field.is_option() {
            true => {
                let inner_ty = model_field.inner_type.to_owned().unwrap();
                quote! {
                    #ident: Set(model.get_value::<#inner_ty>(#ident_name))
                }
            },
            false => {
                quote! {
                    #ident: Set(model.get_value::<#ty>(#ident_name).unwrap())
                }
            }
        }
    })
    .collect::<Vec<_>>()
}

pub fn get_fields_for_edit_model(fields: &Vec<ModelField>) -> Vec<TokenStream> {
    fields
    .iter()
    // TODO: filter id attr based on struct attr or sea_orm primary_key attr
    .filter(|model_field| !model_field.primary_key)
    .map(|model_field| {
        let ident_name = model_field.ident.to_string();
        let ident = model_field.ident.to_owned();
        let ty = model_field.ty.to_owned();

        match model_field.is_option() {
            true => {
                let inner_ty = model_field.inner_type.to_owned().unwrap();
                quote! {
                    entity.#ident = Set(model.get_value::<#inner_ty>(#ident_name))
                }
            },
            false => {
                quote! {
                    entity.#ident = Set(model.get_value::<#ty>(#ident_name).unwrap())
                }
            }
        }
    })
    .collect::<Vec<_>>()
}