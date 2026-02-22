use crate::case_conversion::RenameCase;
use crate::limit_bytes::LimitBytes;
use crate::util::{
    extract_inner_type, matches_option_signature, matches_vec_signature, strip_leading_rawlit,
};
use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(try_from_multipart), supports(struct_named))]
struct InputData {
    ident: syn::Ident,

    data: darling::ast::Data<(), FieldData>,

    #[darling(default)]
    strict: bool,

    #[darling(default)]
    rename_all: Option<RenameCase>,

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
    limit: LimitBytes,

    #[darling(default)]
    default: bool,
}

/// Returns the token stream for the leaf builder wrapping the given type.
fn leaf_builder(ty: &syn::Type, default: bool) -> proc_macro2::TokenStream {
    if default {
        quote! { axum_typed_multipart::DefaultBuilder<#ty> }
    } else {
        quote! { axum_typed_multipart::RequiredBuilder<#ty> }
    }
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

    /// Returns the token stream for the builder type used for this field.
    fn builder_ty(&self) -> proc_macro2::TokenStream {
        let ty = &self.ty;
        if matches_option_signature(ty) {
            let builder = leaf_builder(extract_inner_type(ty), self.default);
            quote! { std::option::Option<#builder> }
        } else if matches_vec_signature(ty) {
            let builder = leaf_builder(extract_inner_type(ty), self.default);
            quote! { std::vec::Vec<#builder> }
        } else {
            leaf_builder(ty, self.default)
        }
    }
}

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data, strict, rename_all, state } =
        match InputData::from_derive_input(&input) {
            Ok(input) => input,
            Err(err) => return err.write_errors().into(),
        };

    let fields = data.take_struct().unwrap();

    let generic = state.is_none().then(|| quote! { <S: Sync> });
    let state = state.map(|state| quote! { #state }).unwrap_or(quote! { S });

    let declarations = fields.iter().map(|field @ FieldData { ident, .. }| {
        let builder_ty = field.builder_ty();
        quote! { let mut #ident: #builder_ty = std::default::Default::default(); }
    });

    let mut assignments = fields
        .iter()
        .map(|field @ FieldData { ident, limit, .. }| {
            let name = field.name(rename_all);

            let duplicate_check = if strict {
                quote! {
                    if <_ as axum_typed_multipart::FieldBuilder<#state>>::has_value(&#ident)
                        && !<_ as axum_typed_multipart::FieldBuilder<#state>>::allows_multiple(&#ident)
                    {
                        return Err(
                            axum_typed_multipart::TypedMultipartError::DuplicateField {
                                field_name: String::from(#name)
                            }
                        );
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                if __field_name__ == #name {
                    #duplicate_check
                    <_ as axum_typed_multipart::FieldBuilder<#state>>::push_field(&mut #ident, __field__, #limit, state).await?;
                }
            }
        })
        .collect::<Vec<_>>();

    if strict {
        assignments.push(quote! {
            {
                return Err(
                    axum_typed_multipart::TypedMultipartError::UnknownField {
                        field_name: __field_name__
                    }
                );
            }
        })
    }

    let finalizations = fields.iter().map(|field @ FieldData { ident, .. }| {
        let name = field.name(rename_all);
        quote! {
            let #ident = <_ as axum_typed_multipart::FieldBuilder<#state>>::finalize(#ident, #name)?;
        }
    });

    let idents = fields.iter().map(|FieldData { ident, .. }| ident);

    let missing_field_name_fallback = if strict {
        quote! { return Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
    } else {
        quote! { continue }
    };

    let output = quote! {
        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::TryFromMultipartWithState<#state> for #ident {
            async fn try_from_multipart_with_state(multipart: &mut axum::extract::multipart::Multipart, state: &#state) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                #(#declarations)*

                while let Some(__field__) = multipart.next_field().await? {
                    let __field_name__ = match __field__.name() {
                        | Some("")
                        | None => #missing_field_name_fallback,
                        | Some(name) => name.to_string(),
                    };

                    #(#assignments) else *
                }

                #(#finalizations)*

                Ok(Self { #(#idents),* })
            }
        }
    };

    output.into()
}
