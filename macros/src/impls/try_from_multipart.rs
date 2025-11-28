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

    /// Separator between prefix and nested field name for flatten (defaults to ".").
    #[darling(default)]
    separator: Option<String>,
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

    #[darling(default)]
    flatten: bool,
}

impl FieldData {
    fn ident(&self) -> &syn::Ident {
        self.ident.as_ref().unwrap()
    }

    fn builder_ident(&self) -> syn::Ident {
        let ty = &self.ty;
        syn::Ident::new(&format!("{}Builder", quote!(#ty)), self.ident().span())
    }

    /// Get the name of the field from the `field_name` attribute, falling back
    /// to the field identifier.
    fn name(&self, rename_all: Option<RenameCase>) -> String {
        if let Some(field_name) = &self.field_name {
            return field_name.to_string();
        }

        let field_in_struct = strip_leading_rawlit(&self.ident().to_string());

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
                Err(_) => abort!(self.ident(), "limit must be a valid byte unit"),
            },
        }
    }
}

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data, strict, rename_all, state, separator } =
        match InputData::from_derive_input(&input) {
            Ok(input) => input,
            Err(err) => abort!(input, err.to_string()),
        };

    let rename_all = RenameCase::from_option_fallible(&ident, rename_all);
    let fields = data.take_struct().unwrap();
    let separator = separator.as_deref().unwrap_or(".");

    let missing_field_name_handling = if strict {
        quote! { return Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
    } else {
        quote! { continue }
    };

    let unknown_field_handling = strict.then_some(quote! {
        if let Some(_) = __result__ {
            return Err(axum_typed_multipart::TypedMultipartError::UnknownField { field_name: __field_name__ });
        }
    });

    let generic = state.is_none().then(|| quote! { <S: Sync> });
    let state_ty = state.as_ref().map(|s| quote! { #s }).unwrap_or(quote! { S });

    let builder_fields = fields.iter().map(|field| {
        let ident = field.ident();
        let ty = &field.ty;
        if field.flatten {
            let builder = field.builder_ident();
            quote! { #ident: #builder }
        } else if matches_vec_signature(ty) || matches_option_signature(ty) {
            quote! { #ident: #ty }
        } else {
            quote! { #ident: std::option::Option<#ty> }
        }
    });

    // Default impl: Vec::new() or None
    let default_fields = fields.iter().map(|field| {
        let ident = field.ident();
        if field.flatten {
            let builder = field.builder_ident();
            quote! { #ident: #builder::default() }
        } else if matches_vec_signature(&field.ty) {
            quote! { #ident: std::vec::Vec::new() }
        } else {
            quote! { #ident: std::option::Option::None }
        }
    });

    // Assignments in process_field (skip flattened fields)
    let assignments = fields
        .iter()
        .filter(|f| !f.flatten)
        .map(|field| process_field(strict, rename_all, field))
        .collect::<Vec<_>>();

    // Delegate to flattened builders (strip prefix if present)
    let flatten_delegation = {
        let checks = fields.iter().filter(|f| f.flatten).map(|field| {
            let ident = field.ident();
            let prefix = format!("{}{}", field.name(rename_all), separator);
            quote! {
                if let Some(__stripped__) = __field_name__.strip_prefix(#prefix) {
                    match self.#ident.process_field(__stripped__, __field__, __state__).await? {
                        None => return Ok(None),
                        Some(f) => return Ok(Some(f)),
                    }
                }
            }
        });
        quote! { #(#checks) else * }
    };

    // Build: validate required fields and construct target struct
    let build_fields = fields.iter().map(|field| {
        let ident = field.ident();
        let field_name = field.name(rename_all);
        if field.flatten {
            quote! { #ident: self.#ident.build()? }
        } else if matches_vec_signature(&field.ty) || matches_option_signature(&field.ty) {
            quote! { #ident: self.#ident }
        } else if field.default {
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

    let mut generic_params = vec![quote! { 'f }];
    let builder_state_ty = if let Some(state) = state {
        quote! { #state }
    } else {
        generic_params.push(quote! { __S__: Sync });
        quote! { __S__ }
    };

    let builder_ident = syn::Ident::new(&format!("{}Builder", ident), ident.span());
    let builder = quote! {
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
            pub async fn process_field<#(#generic_params),*> (
                &mut self,
                __field_name__: &str,
                __field__: axum::extract::multipart::Field<'f>,
                __state__: &#builder_state_ty,
            ) -> Result<Option<axum::extract::multipart::Field<'f>>, axum_typed_multipart::TypedMultipartError> {
                #(#assignments)*

                #flatten_delegation

                Ok(Some(__field__))
            }

            pub fn build(self) -> Result<#ident, axum_typed_multipart::TypedMultipartError> {
                Ok(#ident {
                    #(#build_fields),*
                })
            }
        }
    };

    let output = quote! {
        #builder

        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::TryFromMultipartWithState<#state_ty> for #ident {
            async fn try_from_multipart_with_state(multipart: &mut axum::extract::multipart::Multipart, state: &#state_ty) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                let mut __builder__ = #builder_ident::default();

                while let Some(__field__) = multipart.next_field().await? {
                    let __field_name__ = match __field__.name() {
                        | Some("")
                        | None => #missing_field_name_handling,
                        | Some(name) => name.to_string(),
                    };

                    let __result__ = __builder__.process_field(&__field_name__, __field__, state).await?;
                    #unknown_field_handling
                }

                __builder__.build()
            }
        }
    };

    output.into()
}

fn process_field(
    strict: bool,
    rename_all: Option<RenameCase>,
    field: &FieldData,
) -> proc_macro2::TokenStream {
    let ident = field.ident();
    let name = field.name(rename_all);
    let limit_bytes =
        field.limit_bytes().map(|limit| quote! { Some(#limit) }).unwrap_or(quote! { None });
    let value = quote! {
        <_ as axum_typed_multipart::TryFromFieldWithState<_>>::try_from_field_with_state(__field__, #limit_bytes, __state__).await?
    };

    let assignment = if matches_vec_signature(&field.ty) {
        quote! {
            self.#ident.push(#value);
        }
    } else if strict {
        quote! {
            if let None = self.#ident {
                self.#ident = Some(#value);
            } else {
                return Err(
                    axum_typed_multipart::TypedMultipartError::DuplicateField {
                        field_name: String::from(#name)
                    }
                );
            }
        }
    } else {
        quote! {
            self.#ident = Some(#value);
        }
    };

    quote! {
        if __field_name__ == #name {
            #assignment
            return Ok(None);
        }
    }
}
