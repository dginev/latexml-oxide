// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//

//! # The `LaTeXML` converter, reimplemented in Rust
#![allow(unused_variables, unused_mut, unused_macros)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(match_default_bindings)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rtx_codegen;
#[macro_use]
extern crate rtx_core;

extern crate glob;
extern crate libxml;
extern crate rand;
extern crate regex;
extern crate tempfile;
extern crate time;

#[macro_use]
mod macros;
pub mod util;
pub mod core;
pub mod converter;
pub mod package;
pub mod math_parser;
