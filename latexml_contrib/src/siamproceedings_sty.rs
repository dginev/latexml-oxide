//! siamproceedings.sty — SIAM's proceedings style over `\documentclass{article}`. The paper's copy is read raw; the
//! binding only says which marks the copy's `\maketitle` prints, which the lock refused: without `titlepage`,
//! siamproceedings.sty:187-189 `\renewcommand\thefootnote{\@fnsymbol\c@footnote}`; with it, its title page's
//! (:153-176), which sets none and prints the one in force — unknown here, as article's title page (row R
//! TITLEPAGEMARKS). 2509.02371, 2509.13490.
use latexml_core::binding::content::if_condition;
use latexml_package::{
  prelude::*,
  shipped_class::{declare_shipped_copy_title_marks, input_shipped_package},
};

LoadDefinitions!({
  if input_shipped_package()? && if_condition(&T_CS!("\\if@titlepage"))? != Some(true) {
    declare_shipped_copy_title_marks("sty")?;
  }
});
