use crate::case_conversion::RenameCase;
use crate::limit_bytes::LimitBytes;
use crate::util::{
    builder_ident, extract_inner_type, matches_option_signature, matches_vec_signature,
    strip_leading_rawlit,
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

    #[darling(default)]
    nested: bool,
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
            let inner = self.inner_builder_ty(extract_inner_type(ty));
            quote! { std::option::Option<#inner> }
        } else if matches_vec_signature(ty) {
            let inner = self.inner_builder_ty(extract_inner_type(ty));
            quote! { std::vec::Vec<#inner> }
        } else {
            self.inner_builder_ty(ty)
        }
    }

    /// Returns the builder type for a single (unwrapped) field type.
    fn inner_builder_ty(&self, ty: &syn::Type) -> proc_macro2::TokenStream {
        if self.nested {
            let ident = builder_ident(ty);
            quote! { #ident }
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

    // -- Builder struct --------------------------------------------------

    let builder_ident = quote::format_ident!("{}MultipartBuilder", ident);

    let builder_fields = fields.iter().map(|field @ FieldData { ident, .. }| {
        let builder_ty = field.builder_ty();
        quote! { #ident: #builder_ty }
    });

    // -- FieldBuilder::matches -------------------------------------------

    let match_arms = fields.iter().map(|field| {
        let name = field.name(rename_all);
        let builder_ty = field.builder_ty();
        quote! {
            if <#builder_ty as axum_typed_multipart::FieldBuilder<#state>>::matches(__segments__, #name) {
                return true;
            }
        }
    });

    // -- FieldBuilder::push_field ----------------------------------------

    let mut push_arms = fields
        .iter()
        .map(|field @ FieldData { ident, limit, .. }| {
            let name = field.name(rename_all);

            let duplicate_check = if strict {
                quote! {
                    if <_ as axum_typed_multipart::FieldBuilder<#state>>::has_value(&self.#ident)
                        && !<_ as axum_typed_multipart::FieldBuilder<#state>>::allows_multiple(&self.#ident)
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

            let builder_ty = field.builder_ty();

            quote! {
                if <#builder_ty as axum_typed_multipart::FieldBuilder<#state>>::matches(__segments__, #name) {
                    #duplicate_check
                    <_ as axum_typed_multipart::FieldBuilder<#state>>::push_field(&mut self.#ident, __field__, __segments__, __field_name__, #limit, __state__).await?;
                }
            }
        })
        .collect::<Vec<_>>();

    if strict {
        push_arms.push(quote! {
            {
                return Err(
                    axum_typed_multipart::TypedMultipartError::UnknownField {
                        field_name: __field_name__.to_owned()
                    }
                );
            }
        })
    }

    // -- FieldBuilder::finalize ------------------------------------------

    let finalizations = fields.iter().map(|field @ FieldData { ident, .. }| {
        let name = field.name(rename_all);
        quote! {
            let #ident = <_ as axum_typed_multipart::FieldBuilder<#state>>::finalize(self.#ident, #name)?;
        }
    });

    let idents: Vec<_> = fields.iter().map(|FieldData { ident, .. }| ident).collect();

    // -- FieldBuilder::has_value -----------------------------------------

    let has_value_checks = fields.iter().map(|FieldData { ident, .. }| {
        quote! { <_ as axum_typed_multipart::FieldBuilder<#state>>::has_value(&self.#ident) }
    });

    // -- TryFromMultipartWithState ---------------------------------------

    let missing_field_name_fallback = if strict {
        quote! { return Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
    } else {
        quote! { continue }
    };

    let output = quote! {
        #[doc(hidden)]
        struct #builder_ident {
            #(#builder_fields,)*
        }

        impl std::default::Default for #builder_ident {
            fn default() -> Self {
                Self {
                    #(#idents: std::default::Default::default(),)*
                }
            }
        }

        impl #builder_ident {
            /// Dispatch a field to the matching inner builder.
            ///
            /// `segments` are the path components relevant to *this* struct's
            /// fields (i.e. the nesting prefix has already been stripped by the
            /// caller). `field_name` is the original full field name for error
            /// messages.
            async fn __dispatch__ #generic (
                &mut self,
                __field__: axum::extract::multipart::Field<'_>,
                __segments__: &[axum_typed_multipart::Segment<'_>],
                __field_name__: &str,
                __state__: &#state,
            ) -> Result<(), axum_typed_multipart::TypedMultipartError> {
                #(#push_arms) else *
                Ok(())
            }
        }

        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::FieldBuilder<#state> for #builder_ident {
            type Output = #ident;

            fn matches(__segments__: &[axum_typed_multipart::Segment<'_>], __base_name__: &str) -> bool {
                let Some(axum_typed_multipart::Segment::Key(__first__)) = __segments__.first() else {
                    return false;
                };
                if *__first__ != __base_name__ {
                    return false;
                }
                let __segments__ = &__segments__[1..];
                #(#match_arms)*
                false
            }

            async fn push_field(
                &mut self,
                __field__: axum::extract::multipart::Field<'_>,
                __segments__: &[axum_typed_multipart::Segment<'_>],
                __field_name__: &str,
                _limit_bytes: Option<usize>,
                __state__: &#state,
            ) -> Result<(), axum_typed_multipart::TypedMultipartError> {
                self.__dispatch__(__field__, &__segments__[1..], __field_name__, __state__).await
            }

            fn finalize(self, _field_name: &str) -> Result<Self::Output, axum_typed_multipart::TypedMultipartError> {
                #(#finalizations)*
                Ok(#ident { #(#idents),* })
            }

            fn has_value(&self) -> bool {
                false #(|| #has_value_checks)*
            }

            fn allows_multiple(&self) -> bool {
                true
            }
        }

        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::TryFromMultipartWithState<#state> for #ident {
            async fn try_from_multipart_with_state(multipart: &mut axum::extract::multipart::Multipart, state: &#state) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                let mut __builder__ = #builder_ident::default();

                while let Some(__field__) = multipart.next_field().await? {
                    let __field_name__ = match __field__.name() {
                        | Some("")
                        | None => #missing_field_name_fallback,
                        | Some(name) => name.to_string(),
                    };

                    let __segments__ = axum_typed_multipart::parse_field_name(&__field_name__)?;

                    __builder__.__dispatch__(__field__, &__segments__, &__field_name__, state).await?;
                }

                <_ as axum_typed_multipart::FieldBuilder<#state>>::finalize(__builder__, "")
            }
        }
    };

    output.into()
}
