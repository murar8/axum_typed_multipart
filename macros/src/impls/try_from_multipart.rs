use crate::case_conversion::RenameCase;
use crate::util::{matches_option_signature, matches_vec_signature, strip_leading_rawlit};
use darling::{FromDeriveInput, FromField};
use proc_macro_error2::abort;
use quote::quote;
use ubyte::ByteUnit;

/// Struct-level options parsed from `#[try_from_multipart(...)]`.
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

/// Field-level options parsed from `#[form_data(...)]`.
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
pub fn macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data, strict, rename_all, state, separator } =
        match InputData::from_derive_input(&input) {
            Ok(input) => input,
            Err(err) => abort!(input, err.to_string()),
        };

    let rename_all = RenameCase::from_option_fallible(&ident, rename_all);
    let fields = data.take_struct().unwrap();

    let impl_generics = state.is_none().then(|| quote! { <S: Sync> });
    let state_ty = state.as_ref().map(|s| quote! { #s }).unwrap_or(quote! { S });

    let struct_field_decls = fields.iter().map(|field| {
        let ident = field.ident();
        let ty = &field.ty;
        if field.flatten {
            let nested_builder_ident = builder_ident(&field.ty, field.ident().span());
            quote! { #ident: #nested_builder_ident }
        } else if matches_vec_signature(ty) || matches_option_signature(ty) {
            quote! { #ident: #ty }
        } else {
            quote! { #ident: std::option::Option<#ty> }
        }
    });

    // Default impl: Vec::new() or None
    let default_field_inits = fields.iter().map(|field| {
        let ident = field.ident();
        if field.flatten {
            let nested_builder_ident = builder_ident(&field.ty, field.ident().span());
            quote! { #ident: #nested_builder_ident::default() }
        } else if matches_vec_signature(&field.ty) {
            quote! { #ident: std::vec::Vec::new() }
        } else {
            quote! { #ident: std::option::Option::None }
        }
    });

    let separator = separator.as_deref().unwrap_or(".");
    let process_field_branches = fields.iter().map(|field| {
        if field.flatten {
            gen_flatten_handler(rename_all, separator, field, &state_ty)
        } else {
            gen_field_handler(strict, rename_all, field)
        }
    });

    // Build: validate required fields and construct target struct
    let build_field_exprs = fields.iter().map(|field| {
        let ident = field.ident();
        let field_name = field.name(rename_all);
        if field.flatten {
            let nested_builder_ident = builder_ident(&field.ty, field.ident().span());
            quote! { #ident: <#nested_builder_ident as axum_typed_multipart::MultipartBuilder<#state_ty>>::build(self.#ident)? }
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

    let builder_ident = builder_ident(&ident, ident.span());

    let on_missing_name = if strict {
        quote! { return Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
    } else {
        quote! { continue }
    };

    let on_unknown_field = strict.then_some(quote! {
        if let Some(_) = __result__ {
            return Err(axum_typed_multipart::TypedMultipartError::UnknownField { field_name: __field_name__ });
        }
    });

    let output = quote! {
        struct #builder_ident {
            #(#struct_field_decls),*
        }

        impl std::default::Default for #builder_ident {
            fn default() -> Self {
                Self {
                    #(#default_field_inits),*
                }
            }
        }

        #[axum_typed_multipart::async_trait]
        impl #impl_generics axum_typed_multipart::MultipartBuilder<#state_ty> for #builder_ident {
            type Target = #ident;

            async fn process_field<'f>(
                &mut self,
                __field_name__: &str,
                __field__: axum::extract::multipart::Field<'f>,
                __state__: &#state_ty,
            ) -> Result<Option<axum::extract::multipart::Field<'f>>, axum_typed_multipart::TypedMultipartError> {
                #(#process_field_branches)*
                Ok(Some(__field__))
            }

            fn build(self) -> Result<#ident, axum_typed_multipart::TypedMultipartError> {
                Ok(#ident {
                    #(#build_field_exprs),*
                })
            }
        }

        #[axum_typed_multipart::async_trait]
        impl #impl_generics axum_typed_multipart::TryFromMultipartWithState<#state_ty> for #ident {
            async fn try_from_multipart_with_state(multipart: &mut axum::extract::multipart::Multipart, state: &#state_ty) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                let mut __builder__ = #builder_ident::default();

                while let Some(__field__) = multipart.next_field().await? {
                    let __field_name__ = match __field__.name() {
                        | Some("")
                        | None => #on_missing_name,
                        | Some(name) => name.to_string(),
                    };

                    let __result__ = <#builder_ident as axum_typed_multipart::MultipartBuilder<#state_ty>>::process_field(&mut __builder__, &__field_name__, __field__, state).await?;
                    #on_unknown_field
                }

                <#builder_ident as axum_typed_multipart::MultipartBuilder<#state_ty>>::build(__builder__)
            }
        }
    };

    output.into()
}

fn gen_field_handler(
    strict: bool,
    rename_all: Option<RenameCase>,
    field: &FieldData,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident();
    let field_name = field.name(rename_all);
    let limit_bytes =
        field.limit_bytes().map(|limit| quote! { Some(#limit) }).unwrap_or(quote! { None });
    let parsed_value = quote! {
        <_ as axum_typed_multipart::TryFromFieldWithState<_>>::try_from_field_with_state(__field__, #limit_bytes, __state__).await?
    };

    let assignment = if matches_vec_signature(&field.ty) {
        quote! {
            self.#field_ident.push(#parsed_value);
        }
    } else if strict {
        quote! {
            if let None = self.#field_ident {
                self.#field_ident = Some(#parsed_value);
            } else {
                return Err(
                    axum_typed_multipart::TypedMultipartError::DuplicateField {
                        field_name: String::from(#field_name)
                    }
                );
            }
        }
    } else {
        quote! {
            self.#field_ident = Some(#parsed_value);
        }
    };

    quote! {
        if __field_name__ == #field_name {
            #assignment
            return Ok(None);
        }
    }
}

fn gen_flatten_handler(
    rename_all: Option<RenameCase>,
    separator: &str,
    field: &FieldData,
    state_ty: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let field_ident = field.ident();
    let field_prefix = format!("{}{}", field.name(rename_all), separator);
    let nested_builder_ident = builder_ident(&field.ty, field.ident().span());
    quote! {
        if let Some(__stripped__) = __field_name__.strip_prefix(#field_prefix) {
            match <#nested_builder_ident as axum_typed_multipart::MultipartBuilder<#state_ty>>::process_field(&mut self.#field_ident, __stripped__, __field__, __state__).await? {
                None => return Ok(None),
                Some(f) => return Ok(Some(f)),
            }
        }
    }
}

fn builder_ident(tokens: impl quote::ToTokens, span: proc_macro2::Span) -> syn::Ident {
    syn::Ident::new(&format!("{}Builder", quote!(#tokens)), span)
}
