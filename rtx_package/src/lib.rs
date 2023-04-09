//! Binding definitions for the `LaTeXML` converter, reimplemented in Rust
#![recursion_limit = "1024"]

#[macro_use]
extern crate rtx_codegen;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod package;

// allow external crates to be have the full binding infra via a simple
//  use rtx_package::*;
pub use package::*;
pub use rtx_core::{
  Tokens, T_ACTIVE, T_ALIGN, T_ARG, T_BEGIN, T_COMMENT, T_CR, T_CS, T_LETTER, T_MARKER, T_MATH,
  T_OTHER, T_PARAM, T_SPACE, T_SUB, T_SUPER,
};
