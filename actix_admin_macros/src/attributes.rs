//! `#[actix_admin(...)]` field-level attribute parsing.
//!
//! Ported from `bae` on `syn 1` to `darling` on `syn 2`.

pub mod derive_attr {
    use darling::FromField;

    /// Parsed contents of `#[actix_admin(...)]` on a struct field.
    ///
    /// Presence-only markers (`primary_key`, `searchable`, ...) are `Option<()>`.
    #[derive(Debug, FromField, Clone)]
    #[darling(attributes(actix_admin), forward_attrs(allow, doc, cfg))]
    pub struct ActixAdmin {
        // ---- our attributes (all optional) ----
        #[darling(default)]
        pub primary_key: Option<()>,
        #[darling(default)]
        pub foreign_key: Option<syn::LitStr>,
        #[darling(default)]
        pub html_input_type: Option<syn::LitStr>,
        #[darling(default)]
        pub dateformat: Option<syn::LitStr>,
        #[darling(default)]
        pub ceil: Option<syn::LitInt>,
        #[darling(default)]
        pub floor: Option<syn::LitInt>,
        #[darling(default)]
        pub shorten: Option<syn::LitInt>,
        #[darling(default)]
        pub select_list: Option<syn::LitStr>,
        #[darling(default)]
        pub searchable: Option<()>,
        #[darling(default)]
        pub textarea: Option<()>,
        #[darling(default)]
        pub file_upload: Option<()>,
        #[darling(default)]
        pub image: Option<()>,
        #[darling(default)]
        pub html_render: Option<()>,
        #[darling(default)]
        pub url: Option<()>,
        #[darling(default)]
        pub email: Option<()>,
        #[darling(default)]
        pub wysiwyg: Option<()>,
        #[darling(default)]
        pub readonly: Option<()>,
        #[darling(default)]
        pub not_empty: Option<()>,
        #[darling(default)]
        pub list_sort_position: Option<syn::LitStr>,
        #[darling(default)]
        pub list_hide_column: Option<()>,
        #[darling(default)]
        pub list_regex_mask: Option<syn::LitStr>,
        #[darling(default)]
        pub tenant_ref: Option<()>,
        #[darling(default)]
        pub use_tom_select_callback: Option<()>,

        // ---- required by `FromField` (not used by us) ----
        #[allow(dead_code)]
        pub ident: Option<syn::Ident>,
        #[allow(dead_code)]
        pub ty: syn::Type,
        #[allow(dead_code)]
        pub attrs: Vec<syn::Attribute>,
    }

    impl ActixAdmin {
        /// Parse `#[actix_admin(...)]` off a field. Returns `Ok(None)` when
        /// no such attribute is present (matches historical `bae` semantics).
        pub fn try_from_attributes(
            attrs: &[syn::Attribute],
        ) -> Result<Option<Self>, darling::Error> {
            let has_our_attr = attrs.iter().any(|a| a.path().is_ident("actix_admin"));
            if !has_our_attr {
                return Ok(None);
            }

            // Reuse `FromField` by parsing a synthetic field carrying the same
            // attributes; the placeholder ident/ty are discarded by the caller.
            let synthetic: syn::Field = syn::parse_quote! {
                #(#attrs)*
                __actix_admin_placeholder: ()
            };
            Ok(Some(<Self as FromField>::from_field(&synthetic)?))
        }
    }
}
