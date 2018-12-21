
// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//

//! # The `LaTeXML` converter, reimplemented in Rust
#![feature(custom_attribute)]
#![allow(dead_code, unused_variables, unused_mut, unused_macros)]

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
extern crate time;
extern crate unidecode;

#[macro_use]
mod macros;
pub mod converter;
pub mod core;
pub mod math_parser;
pub mod package;
pub mod util;
