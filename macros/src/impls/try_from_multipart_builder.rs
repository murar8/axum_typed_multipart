use crate::case_conversion::RenameCase;
use crate::util::{matches_option_signature, matches_vec_signature, strip_leading_rawlit};
use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use ubyte::ByteUnit;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(try_from_multipart), supports(struct_named))]
struct InputData {
    ident: syn::Ident,

    data: darling::ast::Data<(), FieldData>,

    #[darling(default)]
    strict: bool,

    #[darling(default)]
    rename_all: Option<String>,

    #[darling(default)]
    state: Option<syn::Path>,
}

#[derive(Debug, FromField)]
#[darling(attributes(form_data))]
struct FieldData {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    field_name: Option<String>,

    #[darling(default)]
    limit: Option<String>,

    #[darling(default)]
    default: bool,
}

impl FieldData {
    /// Get the name of the field from the `field_name` attribute, falling back
    /// to the field identifier.
    fn name(&self, rename_all: Option<RenameCase>) -> String {
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

    /// Parse the supplied human-readable size limit into a byte limit.
    fn limit_bytes(&self) -> Option<usize> {
        match self.limit.as_deref() {
            None | Some("unlimited") => None,
            Some(limit) => match limit.parse::<ByteUnit>() {
                Ok(limit) => Some(limit.as_u64() as usize),
                Err(_) => abort!(self.ident.as_ref().unwrap(), "limit must be a valid byte unit"),
            },
        }
    }
}

/// Derive the `TryFromMultipartBuilder` trait for arbitrary named structs.
pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data, strict, rename_all, state } =
        match InputData::from_derive_input(&input) {
            Ok(input) => input,
            Err(err) => abort!(input, err.to_string()),
        };
    let rename_all = RenameCase::from_option_fallible(&ident, rename_all);

    let fields = data.take_struct().unwrap();
    let builder_ident = syn::Ident::new(&format!("{}Builder", ident), ident.span());

    // Builder struct fields: wrap non-Option/non-Vec in Option (like original macro's local vars)
    let builder_fields = fields.iter().map(|FieldData { ident, ty, .. }| {
        if matches_vec_signature(ty) || matches_option_signature(ty) {
            quote! { #ident: #ty }
        } else {
            quote! { #ident: std::option::Option<#ty> }
        }
    });

    // Default impl: Vec::new() or None
    let default_fields = fields.iter().map(|FieldData { ident, ty, .. }| {
        if matches_vec_signature(ty) {
            quote! { #ident: std::vec::Vec::new() }
        } else {
            quote! { #ident: std::option::Option::None }
        }
    });

    // Assignments in process_field
    let assignments = fields
        .iter()
        .map(|field @ FieldData { ident, ty, .. }| {
            let name = field.name(rename_all);
            let limit_bytes =
                field.limit_bytes().map(|limit| quote! { Some(#limit) }).unwrap_or(quote! { None });
            let value = quote! {
                <_ as axum_typed_multipart::TryFromFieldWithState<_>>::try_from_field_with_state(__field__, #limit_bytes, __state__).await?
            };

            let assignment = if matches_vec_signature(ty) {
                quote! { self.#ident.push(#value); }
            } else if strict {
                quote! {
                    if self.#ident.is_some() {
                        return Err(
                            axum_typed_multipart::TypedMultipartError::DuplicateField {
                                field_name: String::from(#name)
                            }
                        );
                    }
                    self.#ident = Some(#value);
                }
            } else {
                quote! { self.#ident = Some(#value); }
            };

            quote! {
                if __field_name__ == #name {
                    #assignment
                    return Ok(true);
                }
            }
        })
        .collect::<Vec<_>>();

    // Build: validate required fields and construct target struct
    let build_fields = fields.iter().map(|field @ FieldData { ident, ty, default, .. }| {
        let field_name = field.name(rename_all);
        if matches_vec_signature(ty) || matches_option_signature(ty) {
            quote! { #ident: self.#ident }
        } else if *default {
            quote! { #ident: self.#ident.unwrap_or_default() }
        } else {
            quote! {
                #ident: self.#ident.ok_or(
                    axum_typed_multipart::TypedMultipartError::MissingField {
                        field_name: String::from(#field_name)
                    }
                )?
            }
        }
    });

    let generic = state.is_none().then(|| quote! { <__S__: Sync> });
    let state_ty = state.map(|state| quote! { #state }).unwrap_or(quote! { __S__ });

    let output = quote! {
        struct #builder_ident {
            #(#builder_fields),*
        }

        impl std::default::Default for #builder_ident {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl #builder_ident {
            pub async fn process_field #generic (
                &mut self,
                __field__: axum::extract::multipart::Field<'_>,
                __state__: &#state_ty,
            ) -> Result<bool, axum_typed_multipart::TypedMultipartError> {
                let __field_name__ = __field__.name().unwrap_or("").to_string();

                #(#assignments)*

                Ok(false)
            }

            pub fn build(self) -> Result<#ident, axum_typed_multipart::TypedMultipartError> {
                Ok(#ident {
                    #(#build_fields),*
                })
            }
        }
    };

    output.into()
}
