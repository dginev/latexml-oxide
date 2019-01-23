// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Binding infrastructure for the `LaTeXML` converter, reimplemented in Rust
#![allow(dead_code, unused_variables, unused_mut, unused_macros, clippy::implicit_hasher, clippy::trivial_regex)]

#[macro_use]
extern crate rtx_core;
#[macro_use]
extern crate rtx_codegen;

#[macro_use]
pub mod macros;
pub mod converter;
pub mod core;
pub mod math_parser;
#[macro_use]
pub mod package;
pub mod util;

// allow external crates to be have the full binding infra via a simple
//  use rtx_package::*;
pub use package::*;
