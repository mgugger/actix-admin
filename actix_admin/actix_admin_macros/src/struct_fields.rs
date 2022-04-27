use proc_macro2::{Span, Ident};
use syn::{
    Attribute, Fields, Meta, NestedMeta, Visibility, DeriveInput
};

const ATTR_META_SKIP: &'static str = "skip";

pub fn get_field_for_tokenstream(input: proc_macro::TokenStream) -> std::vec::Vec<(syn::Visibility, proc_macro2::Ident)> {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let names_struct_ident = Ident::new(&(ty.to_string() + "FieldStaticStr"), Span::call_site());

    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("FieldNames can only be derived for structs"),
    });
    fields
}

pub fn has_skip_attr(attr: &Attribute, path: &'static str) -> bool {
    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        if meta_list.path.is_ident(path) {
            for nested_item in meta_list.nested.iter() {
                if let NestedMeta::Meta(Meta::Path(path)) = nested_item {
                    if path.is_ident(ATTR_META_SKIP) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .filter_map(|field| {
            if field
                .attrs
                .iter()
                .find(|attr| has_skip_attr(attr, "struct_field_names"))
                .is_none()
                && field.ident.is_some()
            {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
                Some((field_vis, field_ident))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}