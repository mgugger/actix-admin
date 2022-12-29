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
    pub html_input_type: String,
    pub select_list: String,
    pub searchable: bool,
    pub textarea: bool,
    pub file_upload: bool,
    pub not_empty: bool
}

impl ModelField {
    pub fn is_option(&self) -> bool { 
        self.inner_type.is_some()
    }

    pub fn is_string(&self) -> bool {
        match &self.ty {
            Type::Path(type_path) if type_path.clone().into_token_stream().to_string() == "String" => {
                true
            }
            _ => false
        }
    }

    pub fn get_type_path_string(&self) -> String {
        let type_path_string: String;
        if self.is_option() {
            match &self.inner_type.clone().unwrap() {
                Type::Path(type_path) => type_path_string = type_path.clone().into_token_stream().to_string(),
                _ => panic!("not a type path")
            }
        } else {
            match &self.ty {
                Type::Path(type_path) => type_path_string = type_path.clone().into_token_stream().to_string(),
                _ => panic!("not a type path")
            }
        }
        
        type_path_string
    }
}