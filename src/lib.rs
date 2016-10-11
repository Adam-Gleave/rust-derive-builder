//! Derive a builder for a struct
//!
//! This crate implements the _non-consuming_ [builder pattern].
//! When applied to a struct, it will derive **setter-methods** for all struct fields.
//!
//! **Please note**:
//!
//! * There are slightly different ways to implement the builder pattern in rust.
//!   The preferred way to do it, is the so called _non-consuming_ variant.
//!   That means: all generated setter-methods take and return `&mut self`.
//! * To complete the builder pattern you only have to implement at least one method
//!   which actually builds something based on the struct.
//!   These custom build methods of yours should also take `&mut self` to take advantage of the
//!   non-consuming pattern.
//! * **Don't worry at all** if you have to `clone` or `copy` data in your build methods,
//!   because luckily the Compiler is smart enough to optimize them away in release builds
//!   for your every-day use cases. Thats quite a safe bet - we checked this for you. ;-)
//!   Switching to consuming signatures (=`self`) would not give you any performance
//!   gain, but only restrict your API for every-day use cases
//!
//! [builder pattern]: https://aturon.github.io/ownership/builders.html
//!
//! # Examples
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Debug, PartialEq, Default, Clone, Builder)]
//! struct Lorem {
//!     ipsum: String,
//!     dolor: i32,
//! }
//!
//! fn main() {
//!     let x = Lorem::default().ipsum("sit").dolor(42).clone();
//!     assert_eq!(x, Lorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! In `main()`: The final call of `clone()` represents the act of **building a new struct**
//! when our builder is ready. For the sake of brevity we chose clone and pretend we get
//! something brand new. As already mentioned, the compiler will optimize this away in release
//! mode.
//!
//! ## Generic structs
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Debug, PartialEq, Default, Clone, Builder)]
//! struct GenLorem<T> {
//!     ipsum: String,
//!     dolor: T,
//! }
//!
//! fn main() {
//!     let x = GenLorem::default().ipsum("sit").dolor(42).clone();
//!     assert_eq!(x, GenLorem { ipsum: "sit".into(), dolor: 42 });
//! }
//! ```
//!
//! ## Doc-Comments and Attributes
//!
//! `#[derive(Builder)]` copies doc-comments and attributes `#[...]` from your fields
//! to the according setter-method, if it is one of the following:
//!
//! * `/// ...`
//! * `#[doc = ...]`
//! * `#[cfg(...)]`
//! * `#[allow(...)]`
//!
//! ```rust
//! #[macro_use] extern crate derive_builder;
//!
//! #[derive(Builder)]
//! struct Lorem {
//!     /// `ipsum` may be any `String` (be creative).
//!     ipsum: String,
//!     #[doc = r"`dolor` is the estimated amount of work."]
//!     dolor: i32,
//!     // `#[derive(Builder)]` understands conditional compilation via cfg-attributes,
//!     // i.e. => "no field = no setter".
//!     #[cfg(target_os = "macos")]
//!     #[allow(non_snake_case)]
//!     Im_a_Mac: bool,
//! }
//! # fn main() {}
//! ```
//!
//! ## Gotchas
//!
//! - Tuple structs and unit structs are not supported as they have no field
//!   names.
//! - When defining a generic struct, you cannot use `VALUE` as a generic
//!   parameter as this is what all setters are using.

#![crate_type = "proc-macro"]
#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = builder_for_struct(ast);

    format!("{}\n{}", input, result).parse().expect("couldn't parse string to tokens")
}

fn builder_for_struct(ast: syn::MacroInput) -> quote::Tokens {
    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("#[derive(new)] can only be used with braced structs"),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let funcs = fields.iter().map(|f| {
        let f_name = &f.ident;
        let ty = &f.ty;
        quote!(pub fn #f_name<VALUE: Into<#ty>>(&mut self, value: VALUE) -> &mut Self {
            self.#f_name = value.into();
            self
        })
    });

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(funcs)*
        }
    }
}
