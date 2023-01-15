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
        pub html_input_type: Option<syn::LitStr>,
        pub select_list: Option<syn::LitStr>,
        pub searchable: Option<()>,
        pub textarea: Option<()>,
        pub file_upload: Option<()>,
        pub not_empty: Option<()>,
        pub list_sort_position: Option<syn::LitStr>
        //pub inner_type: Option<syn::Type>,

        // Anything that implements `syn::parse::Parse` is supported.
        //mandatory_type: syn::Type,
        //mandatory_ident: syn::Ident,

        // Fields wrapped in `Option` are optional and default to `None` if
        // not specified in the attribute.
        //optional_missing: Option<syn::Type>,
        //optional_given: Option<syn::Type>,

        // A "switch" is something that doesn't take arguments.
        // All fields with type `Option<()>` are considered swiches.
        // They default to `None`.
        //switch: Option<()>,
    }
}