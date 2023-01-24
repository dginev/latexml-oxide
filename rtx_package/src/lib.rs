// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//

//! Binding infrastructure for the `LaTeXML` converter, reimplemented in Rust
#![recursion_limit = "1024"]
#![allow(dead_code, unused_variables, unused_mut, unused_macros)]
#![allow(clippy::unused_unit, clippy::implicit_hasher, clippy::trivial_regex)]

#[macro_use]
extern crate rtx_core;
#[macro_use]
extern crate rtx_codegen;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod package;

// allow external crates to be have the full binding infra via a simple
//  use rtx_package::*;
pub use package::*;
pub use rtx_core::{Tokens,T_CS,T_OTHER,T_COMMENT,T_BEGIN,T_END,T_ALIGN,T_ACTIVE,T_ARG,T_CR,T_MARKER,T_LETTER,T_MATH,T_PARAM,T_SPACE,T_SUB,T_SUPER};