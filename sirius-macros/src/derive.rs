use proc_macro::TokenStream;
use quote::quote;
use syn::Attribute;

pub fn derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    match &ast.data {
        syn::Data::Struct(struct_data) => impl_struct(name, struct_data),
        syn::Data::Enum(enum_data) => impl_enum(name, enum_data, ast.attrs.clone()),
        syn::Data::Union(_) => {
            panic!("Sirius does not support unions, only structs and enums are supported")
        }
    }
}

/// # Struct Serialization & Deserialization
///
/// ## Serialization
/// The generated `serialize` method writes each field to the provided output sequentially:
/// ```no_run,rust
/// fn serialize(&self, output: &mut impl std::io::Write) -> usize {
///     let mut bytes_written = 0;
///     bytes_written += sirius::Sirius::serialize(&self.field1, output);
///     bytes_written += sirius::Sirius::serialize(&self.field2, output);
///     // ...
///     bytes_written
/// }
/// ```
///
/// ## Deserialization
/// The generated `deserialize` method reads each field in the same order:
/// ```no_run,rust
/// fn deserialize(data: &[u8]) -> Result<(Self, usize), sirius::SiriusError> {
///     let mut offset = 0;
///     let field1 = T1::deserialize(data.get(offset..).ok_or(...)?)?; offset += field1.1;
///     let field2 = T2::deserialize(data.get(offset..).ok_or(...)?)?; offset += field2.1;
///     // ...
///     Ok((Self { field1: field1.0, field2: field2.0, /* ... */ }, offset))
/// }
/// ```
///
/// Tuple structs are supported as well; construction switches to `Self(field1.0, field2.0, ...)`.
fn impl_struct(name: &syn::Ident, syn::DataStruct { fields, .. }: &syn::DataStruct) -> TokenStream {
    let is_tuple_struct = matches!(fields, syn::Fields::Unnamed(..));
    let (serialize_fields, deserialize_fields, collection) = (
        // serialization
        fields.iter().enumerate().map(|(field_idx, field)| {
            let field_name = field
                .ident
                .as_ref()
                .map(|i| quote! { #i })
                .unwrap_or_else(|| {
                    let literal = proc_macro2::Literal::usize_unsuffixed(field_idx);
                    quote! { #literal }
                });

            quote! {
                bytes_written += sirius::Sirius::serialize(&self.#field_name, output)?;
            }
        }),
        // deserialization
        fields.iter().enumerate().map(|(idx, field)| {
            let ty = &field.ty;
            let field_var_ident = make_ident(&format!("f{idx}"));

            quote! {
                let #field_var_ident = <#ty as sirius::Sirius>::deserialize(data.get(offset..)
                    .ok_or(sirius::SiriusError::NotEnoughData)?)?;
                offset += #field_var_ident.1;
            }
        }),
        // collection
        fields.iter().enumerate().map(|(idx, field)| {
            let field_var_ident = make_ident(&format!("f{idx}"));

            if is_tuple_struct {
                quote! { #field_var_ident.0 }
            } else {
                let field_name = field.ident.as_ref().unwrap();
                quote! { #field_name: #field_var_ident.0 }
            }
        }),
    );

    let collection = if is_tuple_struct {
        quote! { Self(#(#collection),*) }
    } else {
        quote! { Self{ #(#collection),* } }
    };

    quote! {
        impl sirius::Sirius for #name {
            fn serialize(&self, output: &mut impl std::io::Write) -> Result<usize, sirius::SiriusError> {
                let mut bytes_written = 0;
                #(#serialize_fields)*
                Ok(bytes_written)
            }

            fn deserialize(data: &[u8]) -> Result<(Self, usize), sirius::SiriusError> {
                let mut offset = 0;
                #(#deserialize_fields)*

                Ok((
                    #collection, offset
                ))
            }
        }
    }
    .into()
}

fn impl_enum(
    name: &syn::Ident,
    syn::DataEnum { variants, .. }: &syn::DataEnum,
    _attrs: Vec<Attribute>,
) -> TokenStream {
    let num_variants = variants.len();

    if num_variants > u8::MAX as usize {
        panic!(
            "Sirius does not support enums with more than 255 variants, found {}",
            num_variants
        );
    }

    let serialize = variants.iter().enumerate().map(|(variant_idx, variant)| {
        let (destructure, serialize) = match &variant.fields {
            syn::Fields::Unnamed(unnamed_fields) => {
                let field_idents = unnamed_fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(field_idx, _)| {
                        proc_macro2::Ident::new(
                            &format!("v{variant_idx}f{field_idx}"),
                            proc_macro2::Span::call_site(),
                        )
                    });

                let destructure = field_idents
                    .clone()
                    .map(|field_ident| quote! { #field_ident });

                let serialize = field_idents
                    .map(|field_ident| quote! { bytes_written += sirius::Sirius::serialize(#field_ident, output)?; });

                (quote! { (#(#destructure)*) }, quote! { #(#serialize)* })
            }

            syn::Fields::Named(named_fields) => {
                let field_idents = named_fields.named.iter().map(|i| i.ident.as_ref().unwrap());

                let destructure = {
                    let field_idents = field_idents.clone();
                    quote! { { #(#field_idents),* } }
                };

                let serialize = field_idents
                    .map(|field_ident| quote! { bytes_written += sirius::Sirius::serialize(#field_ident, output)?; });

                (destructure, quote! { #(#serialize)* })
            }

            syn::Fields::Unit => (
                proc_macro2::TokenStream::new(),
                proc_macro2::TokenStream::new(),
            ),
        };

        let variant_name = &variant.ident;

        quote! {
            Self::#variant_name #destructure => {
                bytes_written += (#variant_idx as u8).serialize(output)?;
                #serialize
            }
        }
    });

    let deserialize = variants.iter().enumerate().map(|(variant_idx, variant)| {
        let deserialize = match &variant.fields {
            syn::Fields::Unnamed(unnamed_fields) => {
                let deserializer = unnamed_fields.unnamed.iter().map(
                    |syn::Field { ty, .. }| quote! {{
                        let (data, inc) = <#ty as sirius::Sirius>::deserialize(&data.get(offset..).ok_or(sirius::SiriusError::NotEnoughData)?)?;
                        offset += inc;
                        data
                    }}
                );

                quote! { (#(#deserializer)*) }
            }

            syn::Fields::Named(named_fields) => {
                let deserializer= named_fields.named.iter().map(
                    |syn::Field { ident, ty, .. }| {
                        let ident = ident.as_ref().unwrap();
                        quote! {
                            #ident: {
                                let (data, inc) = <#ty as sirius::Sirius>::deserialize(&data.get(offset..).ok_or(sirius::SiriusError::NotEnoughData)?)?;
                                offset += inc;
                                data
                            },
                        }
                    }
                );

                quote! { { #(#deserializer)* } }
            }

            syn::Fields::Unit => proc_macro2::TokenStream::new(),
        };

        let variant_idx = variant_idx as u8;
        let variant_name = &variant.ident;

        quote! {
            #variant_idx => Self::#variant_name #deserialize,
        }
    });

    quote! {
        impl sirius::Sirius for #name {
            fn serialize(&self, output: &mut impl std::io::Write) -> Result<usize, sirius::SiriusError> {
                let mut bytes_written = 0;

                match self {
                    #(#serialize)*
                }

                Ok(bytes_written)
            }

            fn deserialize(data: &[u8]) -> Result<(Self, usize), sirius::SiriusError> {
                let mut offset = 0;
                let (variant_index, shift) = <u8 as sirius::Sirius>::deserialize(data)?;

                offset += shift;

                Ok((
                    match variant_index {
                        #(#deserialize)*

                        _ => return Err(sirius::SiriusError::ParsingError {
                            ty_name: stringify!(#name),
                            error: format!("invalid variant index: {}", variant_index),
                        }),
                    },
                    offset
                ))
            }
        }
    }
    .into()
}

fn make_ident(string: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(string, proc_macro2::Span::call_site())
}
