use syn::{
    Visibility, Type
};
use quote::ToTokens;

pub struct ModelField {
    pub visibility: Visibility,
    pub ident: proc_macro2::Ident,
    pub ty: Type,
    //  struct field is option<>
    pub inner_type: Option<Type>,
    pub primary_key: bool,
    pub html_input_type: String,
    pub select_list: String,
    pub searchable: bool
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
}