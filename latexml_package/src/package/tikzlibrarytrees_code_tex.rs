//! tikzlibrarytrees.code.tex — TikZ trees library.
//!
//! Loads the real `tikzlibrarytrees.code.tex` library from TeX Live.
//! In `tikzlibrarytrees.code.tex:95-106`, edge-from-parent styles reference
//! `\tikzparentnode` and `\tikzchildnode`. In core `tikz.code.tex:1412-1414`,
//! `\tikzparentanchor` and `\tikzchildanchor` are initialized to `\pgfutil@empty`,
//! but `\tikzparentnode` and `\tikzchildnode` are only bound dynamically inside
//! `\tikz@children@collected` (tikz.code.tex:4591) and `\tikz@childnode` (tikz.code.tex:4664).
//! Provide default fallback definitions so that expanding or evaluating tree
//! keys outside an active child scope does not fail with undefined CS errors.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!(
    "tikzlibrarytrees.code",
    extension => Some(Cow::Borrowed("tex")),
    noltxml => true
  );
  RawTeX!(r"\providecommand\tikzparentnode{tikzparentnode}\providecommand\tikzchildnode{tikzchildnode}");
});
