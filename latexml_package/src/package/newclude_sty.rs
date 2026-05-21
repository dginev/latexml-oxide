//! newclude.sty (frankenstein) — \include with finer-grained control.
//!
//! At end of package it \inputs tag.sto which defines \IncludeName as
//! the name of the part being processed. We don't materialize the
//! aux-tag mechanism (LaTeXML doesn't track per-include parts in the
//! same way), but defining the macro avoids "undefined" errors when
//! frankenstein-aware bibstyles probe it.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // \IncludeName — expands to current \include argument name. We don't
  // track it; expand to empty string. Witness 2409.14290, 2409.17764,
  // 2410.01942, 2409.19473 (newclude/frankenstein users).
  def_macro_noop("\\IncludeName")?;
  // \input is handled at the kernel level — newclude doesn't redefine.
  // \include hooks: defensively gobbled.
  def_macro_noop("\\IncludeOnly{}")?;
  def_macro_noop("\\NotInMain{}")?;
  def_macro_noop("\\MainName")?;
});
