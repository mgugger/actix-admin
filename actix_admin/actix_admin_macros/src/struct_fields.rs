use proc_macro2::{Span, Ident};
use syn::{
    Attribute, Fields, Meta, NestedMeta, Visibility, DeriveInput, Type
};

use crate::attributes::derive_attr;

const ACTIX_ADMIN: &'static str = "actix_admin";

pub fn get_fields_for_tokenstream(input: proc_macro::TokenStream) -> std::vec::Vec<(syn::Visibility, proc_macro2::Ident, Type, bool)> {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (_vis, ty, _generics) = (&ast.vis, &ast.ident, &ast.generics);
    let _names_struct_ident = Ident::new(&(ty.to_string() + "FieldStaticStr"), Span::call_site());

    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("FieldNames can only be derived for structs"),
    });
    fields
}

pub fn has_skip_attr(attr: &Attribute, path: &'static str) -> bool {
    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        //println!("1");
        //println!("{:?}", meta_list.path);
        //println!("{}", path);
        if meta_list.path.is_ident(path) {
            //println!("2");
            for nested_item in meta_list.nested.iter() {
                if let NestedMeta::Meta(Meta::Path(path)) = nested_item {
                    //println!("3");
                    if path.is_ident(ACTIX_ADMIN) {
                        //println!("true");
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn get_field_type<'a>(actix_admin_attr: &'a Option<derive_attr::ActixAdmin>, field: &'a syn::Field) -> &'a syn::Type {
    match actix_admin_attr {
        Some(attr) => {
            match &attr.inner_type {
                Some(inner_type) => &inner_type,
                None => &field.ty
            }
        },
        _ => &field.ty
    }
}

pub fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident, Type, bool)> {
    fields
        .iter()
        .filter_map(|field| {
            let actix_admin_attr = derive_attr::ActixAdmin::try_from_attributes(&field.attrs).unwrap_or_default();
            
            if field
                .attrs
                .iter()
                .find(|attr| has_skip_attr(attr, ACTIX_ADMIN))
                .is_none()
                && field.ident.is_some()
            {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
                println!("{}", field_ident.to_string());
                let is_option = extract_type_from_option(&field.ty).is_some();
                let field_ty = get_field_type(&actix_admin_attr, &field).to_owned();
                Some((field_vis, field_ident, field_ty, is_option))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
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
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}