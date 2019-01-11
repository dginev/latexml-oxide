// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Binding infrastructure for the `LaTeXML` converter, reimplemented in Rust
#![feature(custom_attribute)]
#![feature(stmt_expr_attributes)]
#![allow(dead_code, unused_variables, unused_mut, unused_macros)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rtx_codegen;
#[macro_use]
extern crate rtx_core;

#[macro_use]
mod macros;
pub mod converter;
pub mod core;
pub mod math_parser;
pub mod package;
pub mod util;
