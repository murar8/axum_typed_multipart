use crate::case_conversion::RenameCase;
use crate::limit_bytes::LimitBytes;
use darling::{FromDeriveInput, FromField};
use quote::{quote, ToTokens};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(try_from_multipart), supports(struct_named))]
pub struct InputData {
    pub ident: syn::Ident,

    pub vis: syn::Visibility,

    pub data: darling::ast::Data<(), FieldData>,

    #[darling(default)]
    pub strict: bool,

    #[darling(default)]
    pub rename_all: Option<RenameCase>,

    #[darling(default)]
    pub state: Option<syn::Path>,
}

impl InputData {
    pub fn builder_ident(&self) -> syn::Ident {
        crate::impls::multipart_builder::builder_ident(&self.ident)
    }

    pub fn generic(&self) -> Option<impl ToTokens> {
        self.state.is_none().then(|| quote! { <S: Sync> })
    }

    pub fn state_ty(&self) -> impl ToTokens {
        self.state.as_ref().map(|s| quote! { #s }).unwrap_or(quote! { S })
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(form_data))]
pub struct FieldData {
    pub ident: Option<syn::Ident>,

    pub ty: syn::Type,

    pub field_name: Option<String>,

    #[darling(default)]
    pub limit: LimitBytes,

    #[darling(default)]
    pub default: bool,

    #[darling(default)]
    pub nested: bool,
}
