//! This crate provides procedural macros for the `tlv` crate. The macros are
//! used to derive the `Encode` and `Decode` traits for structs. The `Encode`
//! trait is used to encode a struct into a TLV format. The `Decode` trait is
//! used to decode a struct from a TLV format.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

/// Generates an implementation of the `Encode` trait for the annotated struct.
/// The implementation will encode the struct into a TLV format.
#[proc_macro_derive(Encode, attributes(tlv))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let name = ident;
    let entities = match data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|f| parse_field_tlv_entity(&f).unwrap())
                .collect::<Vec<_>>(),
            _ => panic!("only named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    };
    let tags = entities
        .iter()
        .map(|e| match e.tag {
            tlv::Tag::U8(tag) => quote! { tlv::Tag::U8(#tag) },
            tlv::Tag::U16(tag) => quote! { tlv::Tag::U16(#tag) },
        })
        .collect::<Vec<_>>();
    let idents = entities.iter().map(|e| &e.ident).collect::<Vec<_>>();
    let gen = quote! {
        impl tlv::Encode for #name {
            fn encode<W>(&self, writer: &mut W) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                #(tlv::value::encode_tagged(#tags, &self.#idents, writer)?;)*
                Ok(())
            }

            fn encoded_length(&self) -> std::io::Result<tlv::Length> {
                let mut length = tlv::Length::new(0);
                #(length += tlv::value::length_tagged(#tags, &self.#idents)?;)*
                Ok(length)
            }
        }
    };
    gen.into()
}

/// Generates an implementation of the `Decode` trait for the annotated struct.
/// The implementation will decode the struct from a TLV format.
#[proc_macro_derive(Decode, attributes(tlv))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let name = ident;
    let entities = match data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|f| parse_field_tlv_entity(&f).unwrap())
                .collect::<Vec<_>>(),
            _ => panic!("only named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    };
    let tags = entities.iter().map(|e| match e.tag {
        tlv::Tag::U8(tag) => quote! { tlv::Tag::U8(#tag) },
        tlv::Tag::U16(tag) => quote! { tlv::Tag::U16(#tag) },
    });
    let idents = entities.iter().map(|e| &e.ident).collect::<Vec<_>>();
    let gen = quote! {
        impl tlv::Decode for #name {
            #[allow(clippy::match_single_binding)]
            fn decode<R>(reader: &mut R) -> std::io::Result<(Self, usize)>
            where
                R: std::io::Read
            {
                let __tlv_decode_read: usize = 0;
                #(
                    let (#idents, __tlv_decode_read) = match tlv::value::decode_tagged(#tags, reader)? {
                        (value, read) => (value, read + __tlv_decode_read),
                    };
                )*
                Ok((Self { #(#idents),* }, __tlv_decode_read))
            }
        }
    };
    gen.into()
}

#[derive(Debug)]
struct TlvEntity {
    ident: syn::Ident,
    tag: tlv::Tag,
}

fn parse_field_tlv_entity(field: &syn::Field) -> Result<TlvEntity, syn::Error> {
    let tlv_attr = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("tlv"))
        .ok_or_else(|| {
            syn::Error::new(
                field.span(),
                "missing tlv attribute on field, expected #[tlv(tag = <tag>)]",
            )
        })?;

    let mut tag: Option<u16> = None;

    tlv_attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("tag") {
            let value = meta.value()?;
            let lit: syn::LitInt = value.parse()?;
            tag = Some(lit.base10_parse()?);
            Ok(())
        } else {
            Err(syn::Error::new(meta.path.span(), "expected tag attribute"))
        }
    })?;

    let tag = match tag {
        Some(tag) if tag <= u8::MAX as u16 => tlv::Tag::U8(tag as u8),
        Some(tag) => tlv::Tag::U16(tag),
        None => {
            return Err(syn::Error::new(
                tlv_attr.span(),
                "missing tag attribute on field, expected #[tlv(tag = <tag>)]",
            ))
        }
    };

    let ident = field.ident.clone().ok_or_else(|| {
        syn::Error::new(
            field.span(),
            "missing field identifier, expected named fields",
        )
    })?;

    Ok(TlvEntity { ident, tag })
}
