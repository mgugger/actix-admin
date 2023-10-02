pub mod derive_attr {
    use bae::FromAttributes;

    #[derive(
        Debug,
        Eq,
        PartialEq,
        FromAttributes,
        Default,
        Clone
    )]
    pub struct ActixAdmin {
        pub primary_key: Option<()>,
        pub foreign_key: Option<syn::LitStr>,
        pub html_input_type: Option<syn::LitStr>,
        pub select_list: Option<syn::LitStr>,
        pub searchable: Option<()>,
        pub textarea: Option<()>,
        pub file_upload: Option<()>,
        pub not_empty: Option<()>,
        pub list_sort_position: Option<syn::LitStr>,
        pub list_hide_column: Option<()>,
        pub list_regex_mask: Option<syn::LitStr>,
        pub tenant_ref: Option<()>
    }
}