//! pasa.cls (Publications of the Astronomical Society of Australia, 2013/04/01 v1.0): the papers ship the class (it is
//! not in TeX Live), and it is read raw. Its :45-52 redefine `\@pushfilename` with the body of the LaTeX2e kernel
//! before 2020 and call it once, bare. latex.ltx's `\@pushfilename` (2020-10 on) also pushes the default hook label
//! and the file path (`\@expl@push@filename@@`, `\@kernel@currpathstack`, its last token reading the three that follow
//! it — `\xdef\@currname{…}` in `\@onefilewithoptions`) and its `\@popfilename` pops both, so every package loaded
//! after the class — the class's own `\usepackage{graphicx}` at :1339 first — popped a label nobody pushed: "LaTeX
//! hooks Error: Extra \PopDefaultHookLabel", one per load (pdflatex on TL 2025 raises the same, with "File ended while
//! scanning use of \@p@pfilepath@aux"; the class is clean on the LaTeX of its era). Its copy is the push of that era,
//! and the bare call at :52 needs it, so the pop of the same era (latex.ltx before 2020-10) is held while the class is
//! read, its loads balanced, and the kernel's push and pop both return after it (OXIDIZED_DESIGN_DIVERGENCES #764). Witnesses
//! 1307.3127, 1501.02922, 1510.05346, 1703.06635, 1802.01188, 1407.1937, 1607.08620, 1602.04538, 1401.2223,
//! 1705.02067, 1609.00634, 1511.02959, 1703.03169 (every Error of each was this one, 13-39 per paper).
use latexml_package::{
  prelude::*,
  shipped_class::{RawRead, read_raw},
};

const PASA: RawRead = RawRead {
  shipped: true,
  held: &[(
    "\\@popfilename",
    r"\def\@popfilename{\expandafter\@p@pfilename\@currnamestack\@nil}",
  )],
  restored: &["\\@pushfilename"],
  ..RawRead::new("pasa", "cls")
};

LoadDefinitions!({
  read_raw(&PASA)?;
});
