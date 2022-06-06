//! Provides [`ok!`] and [`ok_unchecked!`] macros for replacing `?` with `unwrap` and
//! `unwrap_unchecked` calls.
//!
//! # Examples
//!
//! ```
//! use its_ok::ok;
//! use std::io::Write;
//!
//! ok! {
//!     let mut buffer = Vec::new();
//!     buffer.write_all(b"bytes")?;
//! }
//!
//! // The code above gets expanded into this.
//! let mut buffer = Vec::new();
//! buffer.write_all(b"bytes").unwrap();
//! ```

mod replacement;

use proc_macro::TokenStream;
use quote::quote;
use replacement::ReplaceTryWithUnwrap;
use syn;

/// Replaces every `?` with `unwrap` call.
///
/// # Examples
///
/// ```
/// use its_ok::ok;
/// use std::io::Write;
///
/// ok! {
///     let mut buffer = Vec::new();
///     buffer.write_all(b"bytes")?;
/// }
///
/// // The code above gets expanded into this.
/// let mut buffer = Vec::new();
/// buffer.write_all(b"bytes").unwrap();
/// ```
///
/// The macro ignores `?` inside closures, inner functions, traits, etc.
///
/// ```
/// use its_ok::ok;
/// use std::io::{self, Write};
///
/// ok! {
///     let mut buffer = Vec::new();
///     (|| -> io::Result<()> {
///         buffer.write_all(b"bytes")?;
///         Ok(())
///     })()?;
/// }
///
/// // The code above gets expanded into this.
/// let mut buffer = Vec::new();
/// (|| -> io::Result<()> {
///     buffer.write_all(b"bytes")?;
///     Ok(())
/// })().unwrap();
/// ```
#[proc_macro]
pub fn ok(input: TokenStream) -> TokenStream {
    let use_unwrap_unchecked = false;
    ok_impl(input, use_unwrap_unchecked)
}

/// Replaces every `?` with `unwrap_unchecked` call.
///
/// # Example
///
/// ```
/// use its_ok::ok_unchecked;
/// use std::io::Write;
///
/// unsafe {
///     ok_unchecked! {
///         let mut buffer = Vec::new();
///         buffer.write_all(b"bytes")?;
///     }
/// }
///
/// unsafe {
///     // The code above gets expanded into this.
///     let mut buffer = Vec::new();
///     buffer.write_all(b"bytes").unwrap_unchecked();
/// }
/// ```
///
/// The macro ignores `?` inside closures, inner functions, traits, etc.
///
/// ```
/// use its_ok::ok_unchecked;
/// use std::io::{self, Write};
///
/// unsafe {
///     ok_unchecked! {
///         let mut buffer = Vec::new();
///         (|| -> io::Result<()> {
///             buffer.write_all(b"bytes")?;
///             Ok(())
///         })()?;
///     }
/// }
///
/// unsafe {
///     // The code above gets expanded into this.
///     let mut buffer = Vec::new();
///     (|| -> io::Result<()> {
///         buffer.write_all(b"bytes")?;
///         Ok(())
///     })().unwrap_unchecked();
/// }
/// ```
#[proc_macro]
pub fn ok_unchecked(input: TokenStream) -> TokenStream {
    let use_unwrap_unchecked = true;
    ok_impl(input, use_unwrap_unchecked)
}

fn ok_impl(input: TokenStream, unchecked: bool) -> TokenStream {
    match syn::parse::Parser::parse(syn::Block::parse_within, input) {
        Ok(statements) => {
            let mut replacer = ReplaceTryWithUnwrap::new(unchecked);
            let statements = replacer.fold_statements::<_, Vec<_>>(statements);
            TokenStream::from(quote! { { #(#statements)* } })
        }
        Err(error) => TokenStream::from(error.to_compile_error()),
    }
}
