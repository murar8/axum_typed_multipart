use crate::case_conversion::RenameCase;
use crate::limit_bytes::LimitBytes;
use crate::util::strip_leading_rawlit;
use darling::{FromDeriveInput, FromField};
use quote::{quote, ToTokens};

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(try_from_multipart), supports(struct_named))]
pub(crate) struct InputData {
    pub(crate) ident: syn::Ident,

    pub(crate) data: darling::ast::Data<(), FieldData>,

    #[darling(default)]
    pub(crate) strict: bool,

    #[darling(default)]
    pub(crate) rename_all: Option<RenameCase>,

    #[darling(default)]
    pub(crate) state: Option<syn::Path>,
}

impl InputData {
    pub(crate) fn builder_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}Builder", self.ident), self.ident.span())
    }

    pub(crate) fn generic(&self) -> Option<impl ToTokens> {
        self.state.is_none().then(|| quote! { <S: Sync> })
    }

    pub(crate) fn state_ty(&self) -> impl ToTokens {
        self.state.as_ref().map(|s| quote! { #s }).unwrap_or(quote! { S })
    }
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(form_data))]
pub(crate) struct FieldData {
    pub(crate) ident: Option<syn::Ident>,

    pub(crate) ty: syn::Type,

    pub(crate) field_name: Option<String>,

    #[darling(default)]
    pub(crate) limit: Option<LimitBytes>,

    #[darling(default)]
    pub(crate) default: bool,
}

impl FieldData {
    pub(crate) fn name(&self, rename_all: Option<RenameCase>) -> String {
        if let Some(field_name) = &self.field_name {
            return field_name.to_string();
        }

        let ident = self.ident.as_ref().unwrap().to_string();
        let field_in_struct = strip_leading_rawlit(&ident);

        if let Some(case_conversion) = rename_all {
            case_conversion.convert_case(&field_in_struct)
        } else {
            field_in_struct
        }
    }
}
