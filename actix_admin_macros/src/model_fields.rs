use syn::{
    Visibility, Type
};
use quote::ToTokens;

pub struct ModelField {
    pub visibility: Visibility,
    pub ident: proc_macro2::Ident,
    pub ty: Type,
    pub inner_type: Option<Type>,
    pub primary_key: bool,
    pub foreign_key: Option<String>,
    pub ceil: Option<String>,
    pub floor: Option<String>,
    pub shorten: Option<String>,
    pub html_input_type: String,
    pub dateformat: String,
    pub select_list: String,
    pub searchable: bool,
    pub textarea: bool,
    pub file_upload: bool,
    pub not_empty: bool,
    pub list_sort_position: usize,
    pub list_hide_column: bool,
    pub list_regex_mask: String,
    pub tenant_ref: bool,
    pub use_tom_select_callback: bool,
}

impl ModelField {
    pub fn is_option(&self) -> bool { 
        self.inner_type.is_some()
    }

    pub fn is_string(&self) -> bool {
        matches!(&self.ty, Type::Path(type_path) if type_path.path.is_ident("String"))
    }

    pub fn get_type_path_string(&self) -> String {
        let ty = if self.is_option() {
            self.inner_type.as_ref().unwrap()
        } else {
            &self.ty
        };

        match ty {
            Type::Path(type_path) => type_path.into_token_stream().to_string(),
            _ => panic!("not a type path"),
        }
    }
}