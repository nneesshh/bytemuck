//! Derive macros for [bytemuck](https://docs.rs/bytemuck) traits.

extern crate proc_macro;

mod traits;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result};

use crate::traits::{
  AnyBitPattern, CheckedBitPattern, Contiguous, Derivable, NoUninit, Pod,
  TransparentWrapper, Zeroable,
};

/// Derive the `Pod` trait for a struct
///
/// The macro ensures that the struct follows all the the safety requirements
/// for the `Pod` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
///
/// - All fields in the struct must implement `Pod`
/// - The struct must be `#[repr(C)]` or `#[repr(transparent)]`
/// - The struct must not contain any padding bytes
/// - The struct contains no generic parameters
///
/// ## Example
///
/// ```rust
/// # use bytemuck_derive::{Pod, Zeroable};
///
/// #[derive(Copy, Clone, Pod, Zeroable)]
/// #[repr(C)]
/// struct Test {
///   a: u16,
///   b: u16,
/// }
/// ```
#[proc_macro_derive(Pod)]
pub fn derive_pod(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let expanded =
    derive_marker_trait::<Pod>(parse_macro_input!(input as DeriveInput));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `AnyBitPattern` trait for a struct
///
/// The macro ensures that the struct follows all the the safety requirements
/// for the `AnyBitPattern` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
///
/// - All fields in the struct must to implement `AnyBitPattern`
#[proc_macro_derive(AnyBitPattern)]
pub fn derive_anybitpattern(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let expanded = derive_marker_trait::<AnyBitPattern>(parse_macro_input!(
    input as DeriveInput
  ));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `Zeroable` trait for a struct
///
/// The macro ensures that the struct follows all the the safety requirements
/// for the `Zeroable` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
///
/// - All fields in the struct must to implement `Zeroable`
///
/// ## Example
///
/// ```rust
/// # use bytemuck_derive::{Zeroable};
/// #[derive(Copy, Clone, Zeroable)]
/// #[repr(C)]
/// struct Test {
///   a: u16,
///   b: u16,
/// }
/// ```
///
/// # Custom bounds
///
/// Custom bounds for the derived `Zeroable` impl can be given using the
/// `#[zeroable(bound = "")]` helper attribute.
///
/// Using this attribute additionally opts-in to "perfect derive" semantics,
/// where instead of adding bounds for each generic type parameter, bounds are
/// added for each field's type.
///
/// ## Examples
///
/// ```rust
/// # use bytemuck::Zeroable;
/// # use std::marker::PhantomData;
/// #[derive(Clone, Zeroable)]
/// #[zeroable(bound = "")]
/// struct AlwaysZeroable<T> {
///   a: PhantomData<T>,
/// }
///
/// AlwaysZeroable::<std::num::NonZeroU8>::zeroed();
/// ```
///
/// ```rust,compile_fail
/// # use bytemuck::Zeroable;
/// # use std::marker::PhantomData;
/// #[derive(Clone, Zeroable)]
/// #[zeroable(bound = "T: Copy")]
/// struct ZeroableWhenTIsCopy<T> {
///   a: PhantomData<T>,
/// }
///
/// ZeroableWhenTIsCopy::<String>::zeroed();
/// ```
///
/// The restriction that all fields must be Zeroable is still applied, and this
/// is enforced using the mentioned "perfect derive" semantics.
///
/// ```rust
/// # use bytemuck::Zeroable;
/// #[derive(Clone, Zeroable)]
/// #[zeroable(bound = "")]
/// struct ZeroableWhenTIsZeroable<T> {
///   a: T,
/// }
/// ZeroableWhenTIsZeroable::<u32>::zeroed();
/// ```
///
/// ```rust,compile_fail
/// # use bytemuck::Zeroable;
/// # #[derive(Clone, Zeroable)]
/// # #[zeroable(bound = "")]
/// # struct ZeroableWhenTIsZeroable<T> {
/// #   a: T,
/// # }
/// ZeroableWhenTIsZeroable::<String>::zeroed();
/// ```
#[proc_macro_derive(Zeroable, attributes(zeroable))]
pub fn derive_zeroable(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let expanded =
    derive_marker_trait::<Zeroable>(parse_macro_input!(input as DeriveInput));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `NoUninit` trait for a struct or enum
///
/// The macro ensures that the type follows all the the safety requirements
/// for the `NoUninit` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
/// (the rest of the constraints are guaranteed by the `NoUninit` subtrait
/// bounds, i.e. the type must be `Sized + Copy + 'static`):
///
/// If applied to a struct:
/// - All fields in the struct must implement `NoUninit`
/// - The struct must be `#[repr(C)]` or `#[repr(transparent)]`
/// - The struct must not contain any padding bytes
/// - The struct must contain no generic parameters
///
/// If applied to an enum:
/// - The enum must be explicit `#[repr(Int)]`
/// - All variants must be fieldless
/// - The enum must contain no generic parameters
#[proc_macro_derive(NoUninit)]
pub fn derive_no_uninit(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let expanded =
    derive_marker_trait::<NoUninit>(parse_macro_input!(input as DeriveInput));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `CheckedBitPattern` trait for a struct or enum.
///
/// The macro ensures that the type follows all the the safety requirements
/// for the `CheckedBitPattern` trait and derives the required `Bits` type
/// definition and `is_valid_bit_pattern` method for the type automatically.
///
/// The following constraints need to be satisfied for the macro to succeed
/// (the rest of the constraints are guaranteed by the `CheckedBitPattern`
/// subtrait bounds, i.e. are guaranteed by the requirements of the `NoUninit`
/// trait which `CheckedBitPattern` is a subtrait of):
///
/// If applied to a struct:
/// - All fields must implement `CheckedBitPattern`
///
/// If applied to an enum:
/// - All requirements already checked by `NoUninit`, just impls the trait
#[proc_macro_derive(CheckedBitPattern)]
pub fn derive_maybe_pod(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let expanded = derive_marker_trait::<CheckedBitPattern>(parse_macro_input!(
    input as DeriveInput
  ));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `TransparentWrapper` trait for a struct
///
/// The macro ensures that the struct follows all the the safety requirements
/// for the `TransparentWrapper` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
///
/// - The struct must be `#[repr(transparent)]`
/// - The struct must contain the `Wrapped` type
///
/// If the struct only contains a single field, the `Wrapped` type will
/// automatically be determined if there is more then one field in the struct,
/// you need to specify the `Wrapped` type using `#[transparent(T)]`
///
/// ## Example
///
/// ```rust
/// # use bytemuck_derive::TransparentWrapper;
/// # use std::marker::PhantomData;
///
/// #[derive(Copy, Clone, TransparentWrapper)]
/// #[repr(transparent)]
/// #[transparent(u16)]
/// struct Test<T> {
///   inner: u16,
///   extra: PhantomData<T>,
/// }
/// ```
#[proc_macro_derive(TransparentWrapper, attributes(transparent))]
pub fn derive_transparent(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let expanded = derive_marker_trait::<TransparentWrapper>(parse_macro_input!(
    input as DeriveInput
  ));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `Contiguous` trait for an enum
///
/// The macro ensures that the enum follows all the the safety requirements
/// for the `Contiguous` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
///
/// - The enum must be `#[repr(Int)]`
/// - The enum must be fieldless
/// - The enum discriminants must form a contiguous range
///
/// ## Example
///
/// ```rust
/// # use bytemuck_derive::{Contiguous};
///
/// #[derive(Copy, Clone, Contiguous)]
/// #[repr(u8)]
/// enum Test {
///   A = 0,
///   B = 1,
///   C = 2,
/// }
/// ```
#[proc_macro_derive(Contiguous)]
pub fn derive_contiguous(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let expanded =
    derive_marker_trait::<Contiguous>(parse_macro_input!(input as DeriveInput));

  proc_macro::TokenStream::from(expanded)
}

/// Derive the `PartialEq` and `Eq` trait for a type
///
/// The macro implements `PartialEq` and `Eq` by casting both sides of the
/// comparison to a byte slice and then compares those.
///
/// ## Warning
///
/// Since this implements a byte wise comparison, the behavior of floating point
/// numbers does not match their usual comparison behavior. Additionally other
/// custom comparison behaviors of the individual fields are also ignored. This
/// also does not implement `StructuralPartialEq` / `StructuralEq` like
/// `PartialEq` / `Eq` would. This means you can't pattern match on the values.
///
/// ## Example
///
/// ```rust
/// # use bytemuck_derive::{ByteEq, NoUninit};
/// #[derive(Copy, Clone, NoUninit, ByteEq)]
/// #[repr(C)]
/// struct Test {
///   a: u32,
///   b: char,
///   c: f32,
/// }
/// ```
#[proc_macro_derive(ByteEq)]
pub fn derive_byte_eq(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let ident = input.ident;

  proc_macro::TokenStream::from(quote! {
    impl ::core::cmp::PartialEq for #ident {
      #[inline]
      #[must_use]
      fn eq(&self, other: &Self) -> bool {
        ::bytemuck::bytes_of(self) == ::bytemuck::bytes_of(other)
      }
    }
    impl ::core::cmp::Eq for #ident { }
  })
}

/// Derive the `Hash` trait for a type
///
/// The macro implements `Hash` by casting the value to a byte slice and hashing
/// that.
///
/// ## Warning
///
/// The hash does not match the standard library's `Hash` derive.
///
/// ## Example
///
/// ```rust
/// # use bytemuck_derive::{ByteHash, NoUninit};
/// #[derive(Copy, Clone, NoUninit, ByteHash)]
/// #[repr(C)]
/// struct Test {
///   a: u32,
///   b: char,
///   c: f32,
/// }
/// ```
#[proc_macro_derive(ByteHash)]
pub fn derive_byte_hash(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let ident = input.ident;

  proc_macro::TokenStream::from(quote! {
    impl ::core::hash::Hash for #ident {
      #[inline]
      fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        ::core::hash::Hash::hash_slice(::bytemuck::bytes_of(self), state)
      }

      #[inline]
      fn hash_slice<H: ::core::hash::Hasher>(data: &[Self], state: &mut H) {
        ::core::hash::Hash::hash_slice(::bytemuck::cast_slice::<_, u8>(data), state)
      }
    }
  })
}

/// Basic wrapper for error handling
fn derive_marker_trait<Trait: Derivable>(input: DeriveInput) -> TokenStream {
  derive_marker_trait_inner::<Trait>(input)
    .unwrap_or_else(|err| err.into_compile_error())
}

/// Find `#[name(key = "value")]` helper attributes on the struct, and return
/// their `"value"`s parsed with `parser`.
///
/// Returns an error if any attributes with the given `name` do not match the
/// expected format. Returns `Ok([])` if no attributes with `name` are found.
fn find_and_parse_helper_attributes<P: syn::parse::Parser + Copy>(
  attributes: &[syn::Attribute], name: &str, key: &str, parser: P,
  example_value: &str, invalid_value_msg: &str,
) -> Result<Vec<P::Output>> {
  let invalid_format_msg =
    format!("{name} attribute must be `{name}({key} = \"{example_value}\")`",);
  let values_to_check = attributes.iter().filter_map(|attr| match &attr.meta {
    // If a `Path` matches our `name`, return an error, else ignore it.
    // e.g. `#[zeroable]`
    syn::Meta::Path(path) => path
      .is_ident(name)
      .then(|| Err(syn::Error::new_spanned(path, &invalid_format_msg))),
    // If a `NameValue` matches our `name`, return an error, else ignore it.
    // e.g. `#[zeroable = "hello"]`
    syn::Meta::NameValue(namevalue) => {
      namevalue.path.is_ident(name).then(|| {
        Err(syn::Error::new_spanned(&namevalue.path, &invalid_format_msg))
      })
    }
    // If a `List` matches our `name`, match its contents to our format, else
    // ignore it. If its contents match our format, return the value, else
    // return an error.
    syn::Meta::List(list) => list.path.is_ident(name).then(|| {
      let namevalue: syn::MetaNameValue = syn::parse2(list.tokens.clone())
        .map_err(|_| {
          syn::Error::new_spanned(&list.tokens, &invalid_format_msg)
        })?;
      if namevalue.path.is_ident(key) {
        match namevalue.value {
          syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(strlit), ..
          }) => Ok(strlit),
          _ => {
            Err(syn::Error::new_spanned(&namevalue.path, &invalid_format_msg))
          }
        }
      } else {
        Err(syn::Error::new_spanned(&namevalue.path, &invalid_format_msg))
      }
    }),
  });
  // Parse each value found with the given parser, and return them if no errors
  // occur.
  values_to_check
    .map(|lit| {
      let lit = lit?;
      lit.parse_with(parser).map_err(|err| {
        syn::Error::new_spanned(&lit, format!("{invalid_value_msg}: {err}"))
      })
    })
    .collect()
}

fn derive_marker_trait_inner<Trait: Derivable>(
  mut input: DeriveInput,
) -> Result<TokenStream> {
  let trait_ = Trait::ident(&input)?;
  // If this trait allows explicit bounds, and any explicit bounds were given,
  // then use those explicit bounds. Else, apply the default bounds (bound
  // each generic type on this trait).
  if let Some(name) = Trait::explicit_bounds_attribute_name() {
    // See if any explicit bounds were given in attributes.
    let explicit_bounds = find_and_parse_helper_attributes(
      &input.attrs,
      name,
      "bound",
      <syn::punctuated::Punctuated<syn::WherePredicate, syn::Token![,]>>::parse_terminated,
      "Type: Trait",
      "invalid where predicate",
    )?;

    if !explicit_bounds.is_empty() {
      // Explicit bounds were given.
      // Enforce explicitly given bounds, and emit "perfect derive" (i.e. add
      // bounds for each field's type).
      let explicit_bounds = explicit_bounds
        .into_iter()
        .flatten()
        .collect::<Vec<syn::WherePredicate>>();

      let predicates = &mut input.generics.make_where_clause().predicates;

      predicates.extend(explicit_bounds);

      let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields.clone(),
        syn::Data::Union(_) => {
          return Err(syn::Error::new_spanned(
            trait_,
            &"perfect derive is not supported for unions",
          ));
        }
        syn::Data::Enum(_) => {
          return Err(syn::Error::new_spanned(
            trait_,
            &"perfect derive is not supported for enums",
          ));
        }
      };

      for field in fields {
        let ty = field.ty;
        predicates.push(syn::parse_quote!(
          #ty: #trait_
        ));
      }
    } else {
      // No explicit bounds were given.
      // Enforce trait bound on all type generics.
      add_trait_marker(&mut input.generics, &trait_);
    }
  } else {
    // This trait does not allow explicit bounds.
    // Enforce trait bound on all type generics.
    add_trait_marker(&mut input.generics, &trait_);
  }

  let name = &input.ident;

  let (impl_generics, ty_generics, where_clause) =
    input.generics.split_for_impl();

  Trait::check_attributes(&input.data, &input.attrs)?;
  let asserts = Trait::asserts(&input)?;
  let (trait_impl_extras, trait_impl) = Trait::trait_impl(&input)?;

  let implies_trait = if let Some(implies_trait) = Trait::implies_trait() {
    quote!(unsafe impl #impl_generics #implies_trait for #name #ty_generics #where_clause {})
  } else {
    quote!()
  };

  let where_clause =
    if Trait::requires_where_clause() { where_clause } else { None };

  Ok(quote! {
    #asserts

    #trait_impl_extras

    unsafe impl #impl_generics #trait_ for #name #ty_generics #where_clause {
      #trait_impl
    }

    #implies_trait
  })
}

/// Add a trait marker to the generics if it is not already present
fn add_trait_marker(generics: &mut syn::Generics, trait_name: &syn::Path) {
  // Get each generic type parameter.
  let type_params = generics
    .type_params()
    .map(|param| &param.ident)
    .map(|param| {
      syn::parse_quote!(
        #param: #trait_name
      )
    })
    .collect::<Vec<syn::WherePredicate>>();

  generics.make_where_clause().predicates.extend(type_params);
}
