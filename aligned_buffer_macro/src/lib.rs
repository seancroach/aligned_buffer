//! TODO

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::token::{Brace, Colon, Comma, Paren, Semi, Struct};
use syn::{braced, bracketed, parenthesized, Attribute, Error, Ident, LitInt, Visibility};

/// Creates a new buffer type that implements the [`AlignedBuffer`] trait.
///
/// # Examples
///
/// The attribute works with either a tuple struct:
///
/// ```ignore
/// use core::mem;
///
/// use aligned_buffer::prelude::*;
///
/// #[aligned_buffer(16)]
/// struct AlignedBuffer32([u8; 32]);
///
/// assert_eq!(mem::size_of::<AlignedBuffer32>(), 32);
/// assert_eq!(mem::align_of::<AlignedBuffer32>(), 16);
///
/// let buffer = AlignedBuffer32::new();
/// let slice = buffer.as_slice();
/// assert_eq!(slice.len(), 32);
/// ```
///
/// Or a struct with a single named field (the field can be any valid
/// identifier):
///
/// ```ignore
/// use core::mem;
///
/// use aligned_buffer::prelude::*;
///
/// #[aligned_buffer(16)]
/// struct AlignedBuffer32 {
///     data: [u8; 32],
/// }
///
/// assert_eq!(mem::size_of::<AlignedBuffer32>(), 32);
/// assert_eq!(mem::align_of::<AlignedBuffer32>(), 16);
///
/// let buffer = AlignedBuffer32::new();
/// let slice = buffer.as_slice();
/// assert_eq!(slice.len(), 32);
/// ```
///
/// [`AlignedBuffer`]: aligned_buffer_internals::AlignedBuffer
#[proc_macro_attribute]
pub fn aligned_buffer(args: TokenStream, item: TokenStream) -> TokenStream {
    aligned_buffer_inner(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn aligned_buffer_inner(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream2> {
    let alignment = syn::parse::<UnsuffixedPowerOfTwoUsize>(args)?;
    let item = syn::parse::<BufferItem>(item)?;

    let length_value = item.field.ty().len.value;
    let alignment_value = alignment.inner.value;

    if length_value == 0 {
        let message = "buffer length must be greater than zero";
        let error = Error::new(item.field.ty().len.expr.span(), message);
        return Err(error);
    }

    if length_value % alignment_value != 0 {
        let message =
            format!("buffer length `{length_value}` is not a multiple `{alignment_value}`");
        let error = Error::new(item.field.ty().len.expr.span(), message);
        return Err(error);
    }

    let item_tokens = quote! {
        #[repr(align(#alignment))]
        #item
    };

    let length = &item.field.ty().len;
    let ident = item.ident;
    let access = item.field.access();

    let from_bytes = match item.field {
        Field::Unnamed { .. } => {
            quote! {
                #[inline(always)]
                fn from_bytes(bytes: [u8; #length]) -> Self {
                    Self(bytes)
                }
            }
        }
        Field::Named { ref ident, .. } => {
            quote! {
                #[inline(always)]
                fn from_bytes(bytes: [u8; #length]) -> Self {
                    Self { #ident: bytes }
                }
            }
        }
    };

    let implementation = quote! {
        #[allow(clippy::missing_safety_doc)]
        unsafe impl ::aligned_buffer::AlignedBuffer<#length, #alignment> for #ident {
            #[inline(always)]
            fn new() -> Self {
                Self::from_bytes([0x00; #length])
            }

            #[inline(always)]
            fn splat(byte: u8) -> Self {
                Self::from_bytes([byte; #length])
            }

            #from_bytes

            #[inline(always)]
            fn as_ptr(&self) -> *const u8 {
                (self #access).as_ptr()
            }

            #[inline(always)]
            fn as_mut_ptr(&mut self) -> *mut u8 {
                (self #access).as_mut_ptr()
            }
        }
    };

    let tokens = quote! {
        #item_tokens
        #implementation
    };

    Ok(tokens)
}

struct BufferItem {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    field: Field,
}

impl Parse for BufferItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let vis = input.parse()?;
        input.parse::<Struct>()?;
        let ident = input.parse()?;
        let field = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            ident,
            field,
        })
    }
}

impl ToTokens for BufferItem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;
        let field = &self.field;

        tokens.append_all(quote! {
            #(#attrs)*
            #vis struct #ident #field
        });
    }
}

enum Field {
    Unnamed {
        attrs: Vec<Attribute>,
        vis: Visibility,
        ty: BufferType,
    },
    Named {
        attrs: Vec<Attribute>,
        vis: Visibility,
        ident: Ident,
        ty: BufferType,
    },
}

impl Field {
    fn access(&self) -> TokenStream2 {
        match self {
            Field::Unnamed { .. } => quote! { .0 },
            Field::Named { ident, .. } => quote! { .#ident },
        }
    }

    fn ty(&self) -> &BufferType {
        match self {
            Field::Named { ty, .. } | Field::Unnamed { ty, .. } => ty,
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        let fields = if lookahead.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let attrs = Attribute::parse_outer(&content)?;
            let vis = content.parse()?;
            let ty = content.parse()?;
            content.parse::<Option<Comma>>()?;
            input.parse::<Semi>()?;

            Field::Unnamed { attrs, vis, ty }
        } else if lookahead.peek(Brace) {
            let content;
            braced!(content in input);
            let attrs = Attribute::parse_outer(&content)?;
            let vis = content.parse()?;
            let ident = content.parse()?;
            content.parse::<Colon>()?;
            let ty = content.parse()?;
            content.parse::<Comma>()?;

            Field::Named {
                attrs,
                vis,
                ident,
                ty,
            }
        } else {
            return Err(lookahead.error());
        };

        Ok(fields)
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Field::Unnamed { attrs, vis, ty, .. } => {
                tokens.append_all(quote! {
                    (#(#attrs)* #vis #ty);
                });
            }
            Field::Named {
                attrs,
                vis,
                ident,
                ty,
                ..
            } => {
                tokens.append_all(quote! {
                    {
                        #(#attrs)*
                        #vis #ident: #ty
                    }
                });
            }
        }
    }
}

struct BufferType {
    len: UnsuffixedUsize,
}

impl Parse for BufferType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        bracketed!(content in input);

        let u8: Ident = content.parse()?;

        if u8 != "u8" {
            let error = Error::new(u8.span(), "expected `u8`");
            return Err(error);
        }

        content.parse::<Semi>()?;
        let len = content.parse::<UnsuffixedUsize>()?;

        Ok(Self { len })
    }
}

impl ToTokens for BufferType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let len = &self.len;
        tokens.append_all(quote!([u8; #len]));
    }
}

/// A `usize` literal without a suffix, which is a power of two, as required by
/// the `align` attribute.
struct UnsuffixedPowerOfTwoUsize {
    inner: UnsuffixedUsize,
}

impl Parse for UnsuffixedPowerOfTwoUsize {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner = input.parse::<UnsuffixedUsize>()?;

        if !inner.value.is_power_of_two() {
            let message = format!("expected a power of two, found `{}`", inner.value);
            let error = Error::new(inner.expr.span(), message);
            return Err(error);
        }

        Ok(Self { inner })
    }
}

impl ToTokens for UnsuffixedPowerOfTwoUsize {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.inner.to_tokens(tokens);
    }
}

/// A `usize` literal without a suffix.
struct UnsuffixedUsize {
    expr: LitInt,
    value: usize,
}

impl Parse for UnsuffixedUsize {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: LitInt = input.parse()?;
        let suffix = expr.suffix();

        if !suffix.is_empty() {
            let message = format!("expected unsuffixed `usize`, found suffix `{suffix}`");
            let error = Error::new(expr.span(), message);
            return Err(error);
        }

        let value = expr.base10_parse::<usize>()?;

        Ok(Self { expr, value })
    }
}

impl ToTokens for UnsuffixedUsize {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.expr.to_tokens(tokens);
    }
}
