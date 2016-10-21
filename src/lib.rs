// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//

//! # The `LaTeXML` converter, reimplemented in Rust

#![feature(plugin)]
#![plugin(rtx_macros)]

#[macro_use]
extern crate lazy_static;

extern crate glob;
extern crate libxml;
extern crate libc;
extern crate regex;
extern crate Archive;
extern crate rand;
extern crate tempfile;
extern crate time;

#[macro_use]
extern crate rtx_core;

pub mod util;
pub mod core;
pub mod converter;
pub mod package;
