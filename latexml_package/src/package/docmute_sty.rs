//! docmute.sty — `\input` stand-alone documents.
//!
//! docmute exists so a paper can `\input` files that are each a complete
//! document (own `\documentclass`, own preamble, own `\begin{document}` …
//! `\end{document}`). It strips the included file's preamble and turns the
//! included file's `\end{document}` into `\endinput`, so only the body is
//! spliced in and the outer document keeps going.
//!
//! **Why a native binding rather than the raw `.sty`.** The real package
//! (docmute.sty v1.4) works by `\let\documentclass=\docmute@gobblepreamble`
//! and by `\renewenvironment{document}` so that a nested `\end{document}` runs
//! `\endinput`. Neither hook reaches us: `\begin{document}` and
//! `\end{document}` are their own control sequences here, not the `document`
//! environment docmute patches, so raw-loading docmute.sty is inert. The
//! included file's preamble then leaks — `\usepackage … can only appear in the
//! preamble`, once per line — and its `\end{document}` terminates the OUTER
//! document, discarding everything after the `\input`, bibliography included.
//!
//! Witness **2606.09184**: `main.tex` loads docmute, `\input`s five
//! stand-alone files and then carries its own `\begin{thebibliography}` with 18
//! entries. pdflatex renders all 29 pages and 17 references; we stopped after
//! the abstract with 9 `\usepackage` errors and no bibliography. 6 papers in
//! the 2605+2606 bibliography-absence residual load docmute.
//!
//! Perl has no docmute binding either, so this is surpass-Perl; ground truth is
//! the arXiv PDF. Audit `docs/parity/BIB_ABSENCE_AUDIT_2026-07-29.md`.
use crate::prelude::*;

/// How many included stand-alone documents are currently open. Only an
/// `\end{document}` seen while this is positive belongs to an included file.
const NESTING: &str = "lx@docmute@nesting";

LoadDefinitions!({
  // The included file's preamble. `\documentclass` can only be reached inside
  // an `\input` here — docmute is loaded from the outer preamble, so the outer
  // `\documentclass` has already run. Skip RAW LINES up to and including the
  // one that carries `\begin{document}` (the comment.sty idiom): the included
  // preamble must not be digested, and raw lines cost nothing to discard.
  DefConstructor!(T_CS!("\\documentclass"), None, None,
  after_digest => {
    let mut nlines = 0;
    let mut found = false;
    // The remainder of the `\documentclass…` line first — a file could in
    // principle put `\begin{document}` on it.
    let mut line = read_raw_line();
    while let Some(text) = line {
      nlines += 1;
      if text.contains("\\begin{document}") {
        found = true;
        break;
      }
      line = read_raw_line();
    }
    if found {
      assign_value(NESTING, Number::new(lookup_int(NESTING) + 1), Some(Scope::Global));
      note_progress(&s!("[docmute: skipped {nlines} preamble lines]"));
    }
    Ok(Vec::new())
  });

  // The included file's `\end{document}`: end the FILE, not the document.
  // Mirrors docmute.sty's `\endinput` branch, including the nesting counter
  // that tells an included file's end from the real one.
  Let!("\\lx@docmute@orig@enddocument", T_CS!("\\end{document}"));
  DefMacro!(T_CS!("\\end{document}"), None, sub[_args] {
    let depth = lookup_int(NESTING);
    if depth > 0 {
      assign_value(NESTING, Number::new(depth - 1), Some(Scope::Global));
      return Ok(Tokens!(T_CS!("\\endinput")));
    }
    Ok(Tokens!(T_CS!("\\lx@docmute@orig@enddocument")))
  });

  // docmute's only option is a backwards-compatibility no-op.
  DeclareOption!("nested", {});
  DeclareOption!(None, {});
  ProcessOptions!();
});
