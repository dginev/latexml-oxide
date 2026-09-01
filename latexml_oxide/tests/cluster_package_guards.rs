//! Single-shot package / engine regression guards.
//!
//! Auto-consolidated test binary: each former file is an inline `mod`
//! below, body preserved verbatim, merged into one link unit for CI
//! economy. All members are subprocess- or few-conversion tests, so
//! co-locating them in one process stays far under the RSS fuse.

mod cluster;
mod common;

mod href_edef_loop {
  //! `\href` inside `\edef`/`\xdef` must not infinite-loop.
  //!
  //! Root cause (2110.10227): LaTeXML defines `\href` as an expandable macro
  //! whose body re-emits `\href` itself (for the `\lx@hyper@url@` constructor's
  //! reversion argument). In a partial-expansion context (`\edef`/`\xdef`) that
  //! re-emitted `\href` is expanded again and again — an unbounded expansion
  //! loop. ems-journal.sty's `\Emsaffil` → `\build@ffil` does
  //! `\xdef\ems@temp{… \href{mailto:…}{\mbox{…}} …}`, so raw-loading the class
  //! (INCLUDE_STYLES=true, as ar5iv does) drove the loop to a
  //! `Fatal:Timeout:PushbackLimit` / `Fatal:Stomach:Recursion`.
  //!
  //! Fix: mark `\href` `protected => true`. In real hyperref `\href` is a robust
  //! command (`\DeclareRobustCommand`/`\protected`), so `\edef` leaves the
  //! literal `\href{…}{…}` untouched. At top-level digestion (`fully_expand`)
  //! protected macros still expand, so normal `\href` is unchanged. Perl LaTeXML
  //! omits the flag and *hangs* on this input — this is a surpass-Perl
  //! robustness win that is also faithful to real-TeX semantics.
  //!
  //! Dump-independent: the hyperref binding (and `\href`) is compiled in.
  use latexml::util::test::convert_fixture;

  #[test]
  fn href_inside_xdef_does_not_loop() {
    let r = convert_fixture("tests/cluster_regressions/href_edef_loop.tex");

    // The loop manifested as a fatal recursion/timeout abort with no result.
    assert!(
      r.result.is_some(),
      "conversion produced no result — the \\href-in-\\xdef expansion loop \
       likely re-triggered (status_code={})",
      r.status_code
    );
    assert!(
      !r.log.contains("PushbackLimit") && !r.log.contains("Infinite digestion loop"),
      "detected an infinite-expansion / infinite-digestion fatal in the log — \
       `\\href` is expanding inside `\\xdef` again (it must be protected)"
    );
    // status_code 3 == fatal; the protected `\href` keeps this well below.
    assert!(
      r.status_code < 3,
      "conversion hit a fatal (status_code={}) — expected a clean run",
      r.status_code
    );
  }
}

mod href_semiverbatim_loop {
  //! `\href` inside a **Semiverbatim** argument must not infinite-loop.
  //!
  //! The other half of the defect [`58_href_edef_loop`](../58_href_edef_loop.rs)
  //! guards. LaTeXML expands `\href{u}{t}` to
  //! `\lx@hyper@url@\href{}{}{u}{t}` — the re-emitted `\href` exists only to fill
  //! the constructor's reversion slot `#1`. PR "href protected" stopped the
  //! `\edef`/`\xdef` re-expansion by marking `\href` `protected`, but ONE seam
  //! legitimately expands protected macros: `Parameter::digest`'s semiverbatim
  //! pre-expansion (Perl `Core/Parameter.pm` L123-132, "If semiverbatim, Expand
  //! (before digest), so tokens can be neutralized") reads with
  //! `fully_expand = true` (Perl `Core/Gullet.pm` L408-409). That pass linearizes
  //! tokens one at a time and never reaches `\lx@hyper@url@`'s parameter list, so
  //! it expanded the re-emitted `\href` as an ordinary macro — forever.
  //!
  //! Reached from a `.bib`: `\bib@field@default@doi` reads `Semiverbatim`, and
  //! INSPIRE exports DOIs as `doi = {\href{https://doi.org/…}{…}}`. Witnesses
  //! 2605.00181, 2605.19650, 2606.06645 — all three took
  //! `Fatal:Timeout:Recursion` ("a window of 6 token(s) repeated 100+ times")
  //! during the recursive bibliography session and lost the whole bibliography;
  //! the fatal aborted the document. Perl `latexmlc` **hangs** on the same input
  //! (rc=124 after 300 s on the 7-line reproducer), so this is a shared upstream
  //! bug — see `docs/parity/KNOWN_PERL_ERRORS.md`.
  //!
  //! Fix: the reversion slot carries the command NAME as an OTHER-catcode token
  //! instead of the live control sequence, exactly as the sibling `\url` path
  //! (`\lx@hyper@url`) has always done. Inert to every expansion regime, and
  //! stringifies/reverts identically — so the self-reference is structurally
  //! impossible rather than dependent on a flag one seam is entitled to ignore.
  use crate::cluster::convert_and_post_clean;

  /// The runaway manifested as a `Fatal:` with no bibliography at all;
  /// `convert_and_post_clean` asserts zero POST-stage `Error:` markers, which is
  /// where the recursive `.bib` session reports (a core-only guard was blind to
  /// this — see the helper's doc).
  #[test]
  fn href_in_semiverbatim_bib_field_does_not_loop() {
    let x = convert_and_post_clean("tests/cluster_regressions/bib_href_in_identifier_field.tex");

    // Both entries must survive. Before the fix the session died on the first
    // one and `MakeBibliography` fell back to no bibliography whatsoever.
    assert!(
      x.contains("<bibitem") || x.contains("bibitem"),
      "no bibliography at all — the \\href expansion loop likely re-triggered:\n{x}"
    );
    for needle in ["A Paper With A Wrapped DOI", "A Flux Concentrator"] {
      assert!(
        x.contains(needle),
        "{needle:?} missing from the bibliography:\n{x}"
      );
    }
    // The DOI field still produces its identifier element — the fix must not
    // have silenced the runaway by dropping the field. (What the identifier
    // *reads* is a separate, pre-existing matter: a link macro inside a
    // Semiverbatim field stringifies with its command name, and `\url` in the
    // same position has always done the same. Not asserted here.)
    // `MakeBibliography` rewrites `ltx:bib-identifier[@scheme='doi']` into the
    // entry's external link, so the surviving marker in POST output is the
    // `dx.doi.org` href it builds.
    assert!(
      x.contains("dx.doi.org"),
      "the wrapped DOI field produced no doi link:\n{x}"
    );
  }
}

mod natbib_label_dotless_i {
  //! natbib `\bibitem` label with a dotless-i (`\i`) must not infinite-loop.
  //!
  //! Root cause (2111.00584, revtex4-1 + aipnum `.bbl`): natbib's
  //! `\lx@NAT@parselabel` fully-expands a "bare" bibitem label (to locate the
  //! `(year)` paren). Under `[T1]{fontenc}` (here via mathptmx) the LaTeX kernel
  //! redefines `\i` to the `\@changed@cmd` dispatcher `\T1-cmd \i \T1\i`, whose
  //! typeset branch re-injects `\i` through
  //! `\csname\cf@encoding\string\i\endcsname`. Under full `Expand!` that
  //! re-expands forever → `Fatal:Timeout:PushbackLimit` + a box-list runaway,
  //! and the aborted bibliography then emits dozens of
  //! `malformed:ltx:bibitem in <ltx:bibblock>` errors. Perl's `Expand`
  //! (natbib.sty.ltxml:564) happens to terminate on these; ours did not.
  //!
  //! Fix: extend `\lx@NAT@parselabel`'s "don't force-expand" guard (already
  //! covering `\cite`/`\href`/`\bibinfo`) to text-encoding symbol commands
  //! (`\i`, `\j`, `\ss`, `\oe`, …). The `(year)` is always a literal paren in
  //! natbib/BibTeX output, so the raw label is sufficient.
  //!
  //! Fixture faithfulness: the label wraps its author in `\citenamefont`, which
  //! is supplied by the revtex4-1 `.bbl` `\providecommand` preamble
  //! (aipnum4-1.bst), NOT by natbib/revtex. The distilled reproducer originally
  //! dropped that preamble, so the conversion logged a (parity, both-engine)
  //! `undefined:\citenamefont` Error that the test silently tolerated. Restoring
  //! the preamble mirrors a real `.bbl`, drops the run to 0 errors, AND
  //! strengthens the guard test — `\citenamefont{…}` now expands to the dotless
  //! `\i` inside `\lx@NAT@parselabel`, the exact path that must not loop.
  //!
  //! Conditional: needs the kernel dump (so expl3/pgf load cleanly) AND
  //! revtex4-1 + mathptmx + pgfplots installed (the exact package set drives
  //! the encoding state into the looping `\T1-cmd` form).
  use latexml::util::test::{convert_fixture, dump_available, kpse_has};

  #[test]
  fn natbib_dotless_i_label_does_not_loop() {
    if !dump_available() {
      eprintln!(
        "SKIP natbib_dotless_i_label_does_not_loop: no latex kernel dump \
         in resources/dumps/ (run tools/make_formats.sh)"
      );
      return;
    }
    if !kpse_has("revtex4-1.cls") || !kpse_has("mathptmx.sty") || !kpse_has("pgfplots.sty") {
      eprintln!(
        "SKIP natbib_dotless_i_label_does_not_loop: revtex4-1/mathptmx/pgfplots \
         not installed in the host TeX tree"
      );
      return;
    }

    let r = convert_fixture("tests/cluster_regressions/natbib_label_dotless_i.tex");

    assert!(
      r.result.is_some(),
      "conversion produced no result — the `\\i`-in-natbib-label expansion loop \
       likely re-triggered (status_code={})",
      r.status_code
    );
    assert!(
      !r.log.contains("PushbackLimit") && !r.log.contains("Infinite digestion loop"),
      "detected an infinite-expansion / infinite-digestion fatal — \
       `\\lx@NAT@parselabel` is force-expanding a text-encoding symbol again"
    );
    assert!(
      r.status_code < 3,
      "conversion hit a fatal (status_code={}) — expected a clean run",
      r.status_code
    );
    // Strict: the faithful `.bbl` `\providecommand` preamble (aipnum4-1.bst)
    // supplies `\citenamefont` et al., so the conversion is now fully clean.
    // Previously the distilled fixture dropped that preamble and silently
    // tolerated an `undefined:\citenamefont` Error — a passing test that emitted
    // an error. Assert 0 so any future regression (a re-emerging loop-recovery
    // error, or a real binding gap) fails here rather than hiding in the log.
    let n_errors = latexml::util::test::error_count(&r.log);
    assert_eq!(
      n_errors, 0,
      "expected 0 errors but the conversion log carried {n_errors} Error:<class>: \
       markers (status_code={})",
      r.status_code
    );
  }
}

mod nul_byte_input {
  //! A stray NUL byte in the input must not abort the conversion.
  //!
  //! Real-world `.bbl` files carry stray NULs from BibTeX `\"u`-mangling
  //! (witness astro-ph0004127's spie4012-01a.bbl). Since commit 88f8bd44ce the
  //! NUL default catcode is 12/OTHER (matching Perl, so `` `^^@ `` reads 0),
  //! which lets the NUL survive tokenization — and a NUL inside math reaches
  //! `Document::set_attribute` (the `tex=` reversion), where libxml's
  //! `CString::new(value)` panics on the interior NUL (libxml node.rs:639),
  //! killing the whole conversion (a process abort under the maxperf
  //! `panic=abort` build). PR #249 review finding P0-1.
  //!
  //! The fix sanitizes XML-invalid characters at the serialization sinks, so
  //! catcode-12 Perl parity is kept while serialization stays total.
  //!
  //! Dump-independent.
  use latexml::util::test::convert_fixture;

  #[test]
  fn nul_byte_in_math_does_not_abort() {
    // The conversion runs in-process: a libxml CString panic would unwind
    // through (and fail) this test directly.
    let r = convert_fixture("tests/cluster_regressions/nul_byte_input.tex");

    let out = r
      .result
      .unwrap_or_else(|| {
        panic!(
          "conversion produced no result (status_code={}) — the NUL byte \
           likely aborted serialization",
          r.status_code
        )
      })
      .to_string();
    assert!(
      r.status_code < 3,
      "conversion hit a fatal (status_code={}) on a stray NUL byte",
      r.status_code
    );
    // The surrounding content must survive...
    assert!(
      out.contains("Before") && out.contains("after"),
      "document text around the NUL was lost"
    );
    // ...and no literal NUL may reach the XML (it is not a valid XML 1.0 char).
    assert!(
      !out.contains('\u{0000}'),
      "a literal NUL byte leaked into the XML output (invalid XML 1.0)"
    );
  }
}

mod deferred_load_retry {
  //! Regression test for the package-load "deferred miss must not poison a later
  //! raw-load" parity fix (`content.rs`).
  //!
  //! `nicematrix` faithfully `\RequirePackage{pgfcore}` (nicematrix.sty:23), which
  //! has no binding — so bare (INCLUDE_STYLES off) it "misses". Then
  //! `tcolorbox[most]` raw-loads and its `skins` library also needs pgfcore, this
  //! time under INCLUDE_STYLES=true (a raw read turns it on). Before the fix the
  //! Rust-only `_load_attempted` guard from nicematrix's deferred miss permanently
  //! blocked tcolorbox's pgfcore load → ~49 spurious `\pgf…`/`#`-token errors. The
  //! fix sets `_load_attempted` only when raw-loading was actually possible, so the
  //! later load retries — matching pdflatex, which loads pgfcore in either order.
  //!
  //! Driven through the binary (fresh process) so tcolorbox can raw-load its
  //! library files from the host texmf; no `--includestyles`/preload needed.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{nicematrix}\n\
    \\usepackage[most]{tcolorbox}\n\
    \\begin{document}\n\
    \\begin{tcolorbox}[enhanced,breakable]Hello box\\end{tcolorbox}\n\
    \\end{document}\n";

  #[test]
  fn deferred_pgfcore_miss_does_not_poison_tcolorbox_skins() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("d.tex"), TEX).expect("write d.tex");

    let output = Command::new(bin)
      .arg("d.tex")
      .arg("--dest")
      .arg("d.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    // The nicematrix-then-tcolorbox order must be error-clean (was ~49 pgf errors).
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "nicematrix-then-tcolorbox[most] should be error-clean, stderr had errors:\n{stderr}",
    );
    // Sanity: the box content still made it through.
    let xml = std::fs::read_to_string(workdir.path().join("d.xml")).expect("read d.xml");
    assert!(xml.contains("Hello box"), "tcolorbox body missing:\n{xml}");
  }
}

mod nicetabular_binding {
  //! `\begin{NiceTabular}[opts]{colspec}` must render a real table, not
  //! `Error:undefined:{NiceTabular}` + a dropped body.
  //!
  //! nicematrix's NiceTabular is a tabular over a standard colspec (nicematrix.sty
  //! L3806-3841 reduce it to `\NiceArray{colspec}` under a text-mode tabular flag),
  //! so binding it to `\tabular` recovers real tables for sandbox-arxiv-2605 papers
  //! (2605.08776, 2605.13835, 2605.18423) the placeholder stub previously errored on.
  //! Beyond-Perl: the ar5iv nicematrix.sty.ltxml stub still errors here.
  use crate::cluster::convert_to_xml_contrib;

  #[test]
  fn nicetabular_renders_real_table() {
    // Red before the fix: Error:undefined:{NiceTabular} + dropped body (no <tabular>).
    let xml = convert_to_xml_contrib("tests/cluster_regressions/nicetabular_binding.tex");
    assert!(
      xml.contains("<tabular"),
      "NiceTabular did not render a real table:\n{xml}"
    );
    assert!(
      xml.matches("<td").count() >= 6,
      "NiceTabular table is missing cells (expected the 6 `1..6`):\n{xml}",
    );
  }
}

mod neurips_anonymous {
  //! `\if@anonymous` (neurips_2026.sty L72 `\newif`) must be defined by the neurips
  //! binding. The binding intercepts the versioned name `neurips_2026` and never
  //! creates the conditional, so a paper copying the style's `\@maketitle` (which
  //! branches on `\if@anonymous`) hit `Error:undefined:\if@anonymous`. Rust-only
  //! divergence: Perl 0.8.8 converts the same paper (2605.17249) without it.
  //! Default false => the `\else` (authors-shown) branch, correct for arXiv uploads.
  use crate::cluster::convert_to_xml_contrib_clean;

  #[test]
  fn neurips_if_anonymous_defined() {
    // Red before the fix: Error:undefined:\if@anonymous. Green: 0 errors + the Named branch.
    let xml = convert_to_xml_contrib_clean("tests/cluster_regressions/neurips_anonymous.tex");
    assert!(
      xml.contains("Named") && !xml.contains(">Anon"),
      "default-false \\if@anonymous should take the authors-shown (`Named`) branch:\n{xml}",
    );
  }
}

mod biblatex_fallback_no_cite_loop {
  //! `\usepackage{myBiblatex}` hits the versioned-package fallback -> the native
  //! `biblatex` binding, which `find_file_fallback` double-runs (probe then load).
  //! The non-idempotent `\let\blx@saved@cite\cite` used to capture biblatex's OWN
  //! `\cite` on the 2nd init, so `\cite -> \blx@saved@cite -> \cite` looped to
  //! `Fatal:Timeout:TokenLimit`/`Recursion` (witness 2605.03965; Perl never loads
  //! biblatex on this name, so no loop). The save is now `\@ifundefined`-guarded.
  use crate::cluster::convert_to_xml_contrib_clean;

  #[test]
  fn mybiblatex_fallback_does_not_loop() {
    // Red before the fix: \cite loops to Fatal:Timeout:TokenLimit (fatal status / no result).
    // Green: converts clean (convert_to_xml_contrib_clean asserts 0 errors + non-fatal).
    let _ = convert_to_xml_contrib_clean("tests/cluster_regressions/biblatex_mybiblatex_loop.tex");
  }
}

mod expl3_nested_raw_load_catcodes {
  //! A `\ProvidesExplPackage` file that `\RequirePackage{derivative}` — a native
  //! binding (#630) that force-raw-loads its own expl3 `.sty` — used to leave `_`
  //! as SUB after the nested load, so later expl3 lines (`\seq_new:N` …) errored
  //! `unexpected:_` (witness 2605.21946, pomegranate.sty). The input_definitions
  //! expl3-frame stack (content.rs) makes the inner load inherit the outer frame's
  //! expl3 state instead of re-snapshotting after the outer `\@pushfilename`.
  //!
  //! Subprocess (not the in-process `convert_*` helpers) on purpose: reproducing the
  //! bug needs `--includestyles` AND the contrib dispatch (for the `derivative` binding)
  //! AND a paper-local `mymac.sty` on the search path, simultaneously — no single
  //! in-process helper combines all three. Same legitimate subprocess reason as the
  //! `newtcblisting_verbatim` / `deferred_load_retry` tests.
  use std::{path::Path, process::Command};

  #[test]
  fn nested_expl3_raw_load_preserves_catcodes() {
    // Self-skip green when derivative.sty is absent (trimmed CI texlive): with no
    // raw double-load there is nothing to guard.
    let has_derivative = Command::new("kpsewhich")
      .arg("derivative.sty")
      .output()
      .map(|o| o.status.success() && !o.stdout.is_empty())
      .unwrap_or(false);
    if !has_derivative {
      eprintln!("skip nested_expl3_raw_load_preserves_catcodes: derivative.sty not installed");
      return;
    }
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(
      workdir.path().join("mymac.sty"),
      "\\ProvidesExplPackage{mymac}{2025/01/01}{1.0}{repro}\n\
       \\RequirePackage{derivative}\n\
       \\seq_new:N \\l_mymac_seq\n",
    )
    .expect("write mymac.sty");
    std::fs::write(
      workdir.path().join("d.tex"),
      "\\documentclass{article}\n\\usepackage{mymac}\n\\begin{document}hi\\end{document}\n",
    )
    .expect("write d.tex");
    let output = Command::new(bin)
      .arg("d.tex")
      .arg("--dest")
      .arg("d.xml")
      .arg("--includestyles")
      .arg("--path")
      .arg(workdir.path())
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      !stderr.contains("unexpected:_"),
      "nested expl3 raw-load left `_` as SUB (expl3 catcodes lost after the inner load):\n{stderr}",
    );
  }
}

mod cleveref_class_stubs {
  //! lipics-v2021 and IEEEtaes are OmniBus/IEEEtran stub bindings that replace the
  //! paper's real `.cls` and omitted its `\RequirePackage{cleveref}`, so `\cref`/`\Cref`
  //! came out `Error:undefined`. Perl (no lipics/IEEEtaes binding) raw-loads the `.cls`
  //! and gets cleveref, so this was a Rust-only parity gap. The stubs now require
  //! cleveref after hyperref. Witnesses 2606.01187 (lipics), 2606.01169 (IEEEtaes).
  use crate::cluster::convert_to_xml_contrib_clean;

  #[test]
  fn lipics_stub_requires_cleveref() {
    // Red before the fix: `\cref`/`\Cref` come out Error:undefined; green: 0 errors.
    let _ = convert_to_xml_contrib_clean("tests/cluster_regressions/cleveref_lipics_stub.tex");
  }
}

mod aastex_contribution_appendix {
  //! aastex701/631/7 digit-strip to the aastex-v5 `aastex.cls.ltxml` shim, which predates
  //! aastex7, so the `{contribution}` env and `\restartappendixnumbering` came out
  //! `Error:undefined` (Perl errors identically — this is a beyond-Perl addition in the
  //! `aas_support_sty.rs` home that already carries the `\uat` beyond-Perl addition).
  //! Witnesses 2606.03375/04105 (contribution), 2606.00569/03850/07452 (restartappendixnumbering).
  use crate::cluster::convert_to_xml_contrib_clean;

  #[test]
  fn aastex_contribution_and_restartappendix_defined() {
    // Red before the fix: `{contribution}` / `\restartappendixnumbering` Error:undefined;
    // green: 0 errors.
    let _ = convert_to_xml_contrib_clean("tests/cluster_regressions/aastex_contribution.tex");
  }
}

mod subdir_dispatch_no_strip {
  //! Neither the package dispatcher (latexml_package::lib) nor `find_file_fallback`
  //! (latexml_core::binding::content) strips a leading directory any more, so `subdir/<name>`
  //! is a file PATH, not a binding name. Both tests drive the REAL fleet config via
  //! `convert_to_xml_ar5iv` — `ar5iv.sty` sets INCLUDE_STYLES="searchpaths" (`localrawstyles`:
  //! raw-load local `.sty`, classes OFF), the exact `cortex_worker --preload=ar5iv.sty` route.
  use crate::cluster::convert_to_xml_ar5iv;

  /// A paper-local `subdirdispatch/mathenv.sty`, whose basename collides with the CTAN `mathenv`
  /// binding, must raw-load under `localrawstyles` (via SOURCEDIRECTORY), not be shadowed by a
  /// directory-stripped binding match — the 2606.02073 cleveref/theorem bug. RED before the drop
  /// (strip -> `mathenv` binding no-op -> the local `\subdirstymarker` never defined), GREEN
  /// after. Guards both the dispatch strip and the `find_file_fallback` BasenameOnly strip stay
  /// gone.
  #[test]
  fn subdir_sty_raw_loads_not_shadowed() {
    let xml = convert_to_xml_ar5iv("tests/cluster_regressions/subdir_sty_not_shadowed.tex");
    assert!(
      xml.contains("SUBDIRSTYLOADED"),
      "subdirdispatch/mathenv.sty should raw-load its local def (not be shadowed by the CTAN \
       mathenv binding):\n{xml}",
    );
  }

  /// INCLUDE_CLASSES stays disabled under `localrawstyles` (styles-only): a paper-local subdir
  /// `.cls` must NOT raw-load — it falls to OmniBus, so its `\subdirclsmarker` stays undefined
  /// and SUBDIRCLSLOADED never reaches the output. (Enabling `rawclasses` would flip this.)
  #[test]
  fn subdir_cls_does_not_raw_load_include_classes_off() {
    let xml = convert_to_xml_ar5iv("tests/cluster_regressions/subdir_cls_not_rawloaded.tex");
    assert!(
      !xml.contains("SUBDIRCLSLOADED"),
      "subdir `.cls` raw-loaded despite INCLUDE_CLASSES off (localrawstyles is styles-only):\n{xml}",
    );
  }
}

mod rawclasses_binding_precedence_and_no_omnibus {
  //! Guards for the raw-interpretation preload technique
  //! (`--preload=[rawstyles,rawclasses]latexml.sty`, the perfect-kernel
  //! protocol — `docs/perfect_kernel/README.md`). User directive 2026-08-31:
  //!
  //! 1. A compiled `.rs` binding ALWAYS takes precedence, raw mode included —
  //!    `rawstyles`/`rawclasses` never demote an existing binding.
  //! 2. For a class with NO binding, `rawclasses` raw-loads the `.cls`
  //!    through the TeX engine and the OmniBus fallback must NOT fire.
  //!
  //! Exercised with workdir-local class files, so the test needs no host
  //! texmf packages: `scrartcl` (contrib-bound name) for precedence,
  //! `pkzzz` (no binding anywhere) for the no-OmniBus raw load.

  use std::{path::Path, process::Command};

  fn convert(class: &str, cls_body: &str, preload: Option<&str>) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join(format!("{class}.cls")), cls_body).expect("write cls");
    let tex = format!(
      "\\documentclass{{{class}}}\n\\begin{{document}}\n\
       \\ifdefined\\rawmarker\\rawmarker\\else NOMARKER\\fi\n\\end{{document}}\n"
    );
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let mut cmd = Command::new(bin);
    cmd.args(["t.tex", "--dest", "t.xml", "--nocomments"]);
    if let Some(spec) = preload {
      cmd.arg(format!("--preload={spec}"));
    }
    let output = cmd
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    (xml, stderr)
  }

  const RAW_CLS: &str = "\\ProvidesClass{whatever}\n\
    \\LoadClass{article}\n\
    \\newcommand{\\rawmarker}{RAWCLSLOADED}\n";

  /// Directive 1: the contrib `scrartcl` binding wins even under rawclasses —
  /// the local raw `.cls`'s marker must NOT appear.
  #[test]
  fn contrib_binding_keeps_precedence_under_rawclasses() {
    let (xml, _stderr) = convert(
      "scrartcl",
      RAW_CLS,
      Some("[rawstyles,rawclasses]latexml.sty"),
    );
    assert!(
      xml.contains("NOMARKER") && !xml.contains("RAWCLSLOADED"),
      "compiled scrartcl binding must outrank the raw .cls under rawclasses:\n{xml}",
    );
  }

  /// Directive 2: a bindingless class raw-loads under rawclasses; OmniBus
  /// stays out of the conversion entirely.
  #[test]
  fn bindingless_class_raw_loads_without_omnibus() {
    let (xml, stderr) = convert("pkzzz", RAW_CLS, Some("[rawstyles,rawclasses]latexml.sty"));
    assert!(
      xml.contains("RAWCLSLOADED"),
      "bindingless pkzzz.cls should raw-load under rawclasses:\n{xml}\nstderr:\n{stderr}",
    );
    assert!(
      !stderr.contains("OmniBus"),
      "OmniBus must not fire for a raw-loaded bindingless class:\n{stderr}",
    );
  }

  /// Control: without rawclasses the same bindingless class falls back to
  /// OmniBus (the pre-existing default behavior, unchanged by the mission).
  #[test]
  fn bindingless_class_defaults_to_omnibus_without_rawclasses() {
    let (xml, stderr) = convert("pkzzz", RAW_CLS, None);
    assert!(
      xml.contains("NOMARKER") && stderr.contains("OmniBus"),
      "default mode should keep the OmniBus fallback for unknown classes:\n{stderr}",
    );
  }
}

mod defplain_skips_blanks_before_brace {
  //! OXIDIZED_DESIGN #161 (surpass-Perl, approved 2026-08-31): the `DefPlain`
  //! parameter type must skip blanks before its required `{`, like real TeX's
  //! undelimited-argument scanning (tex.web `macro_call`) and LaTeXML's own
  //! `{}` reader. Perl 0.8.8 errors `Expected opening '{'` when a
  //! `\lstnewenvironment{x}[1][]` body sits on the NEXT line — the standard
  //! documentation style (~148 TL doc manuals; ltxdockit.sty, cnltx-example.sty).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{listings}\n\
    \\lstnewenvironment{ltxcode}[1][]\n\
    \x20 {\\lstset{#1}}\n\
    \x20 {}\n\
    \\begin{document}\n\
    \\begin{ltxcode}\n\
    hello code\n\
    \\end{ltxcode}\n\
    \\end{document}\n";

  #[test]
  fn lstnewenvironment_body_on_next_line_defines_cleanly() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Expected opening '{'"),
      "DefPlain must skip the newline before the body brace:\n{stderr}",
    );
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "the definition and its use must digest cleanly:\n{stderr}",
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("<listing"),
      "\\begin{{ltxcode}} should produce an ltx:listing:\n{xml}",
    );
    // OXIDIZED_DESIGN #162: the body's FIRST line must survive. The
    // optional-arg probe crosses the newline after `\begin{ltxcode}` and
    // unreads the body's first char; the raw-line reader must not then
    // discard that line as "leftover of the \begin line" (Perl 0.8.8 drops
    // it — base64 `data` came back holding only the later lines).
    // "aGVsbG8gY29kZQ==" = base64("hello code").
    assert!(
      xml.contains("data=\"aGVsbG8gY29kZQ==\""),
      "the listing body (incl. first line) must survive as data:\n{xml}",
    );
  }
}

mod process_key_options_sees_load_options {
  //! OXIDIZED_DESIGN #164: the loader must record `\@raw@opt@<name>.<ext>` —
  //! the ONLY thing the modern kernel's `\ProcessKeyOptions` reads
  //! (latex.ltx L19398). Without it every ltkeys key-option package silently
  //! drops its load-time options (Perl 0.8.8 shares the miss).

  use std::{path::Path, process::Command};

  const STY: &str = "\\NeedsTeXFormat{LaTeX2e}\n\
    \\ProvidesPackage{pkoguard}\n\
    \\RequirePackage{expl3}\n\
    \\ExplSyntaxOn\n\
    \\keys_define:nn {pkoguard}\n\
    \x20 {\n\
    \x20   flag .bool_set:N = \\l_pkoguard_flag_bool ,\n\
    \x20   flag .default:n  = {true} ,\n\
    \x20 }\n\
    \\ProcessKeyOptions [pkoguard]\n\
    \\bool_if:NT \\l_pkoguard_flag_bool { \\def\\FLAGON{yes} }\n\
    \\ExplSyntaxOff\n";
  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage[flag]{pkoguard}\n\
    \\begin{document}\n\
    flag=\\ifdefined\\FLAGON ON\\else OFF\\fi\n\
    \\end{document}\n";

  #[test]
  fn key_option_reaches_process_key_options() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("pkoguard.sty"), STY).expect("write sty");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--preload=[rawstyles]latexml.sty",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("flag=ON"),
      "\\ProcessKeyOptions must see the [flag] load option:\n{xml}\n{stderr}",
    );
  }
}

mod currsize_default {
  //! OXIDIZED_DESIGN #165: `\@currsize` must be defined (default
  //! `\normalsize`) — real LaTeX's begin-document invariant, which our font
  //! primitives (and Perl's) never establish via `\@setfontsize`. Raw
  //! packages (linguex family) call `{\@currsize …}` to restore text size.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\begin{document}\n\
    x{\\makeatletter\\@currsize\\makeatother restored}\n\
    \\end{document}\n";

  #[test]
  fn currsize_is_defined_and_usable() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "\\@currsize must be defined (begin-document invariant):\n{stderr}",
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("restored"),
      "content after \\@currsize lost:\n{xml}"
    );
  }
}

mod luatex_profile {
  //! OXIDIZED_DESIGN #168: the opt-in `luatex` latexml.sty option flips the
  //! document to LuaTeX identity — iftex probes consult LUATEX_PROFILE state
  //! (immune to load order), and `\directlua` exists under its REAL name for
  //! that conversion only. Without the option the pdfTeX-model identity is
  //! untouched (defining \directlua by default flipped 26 tests onto luatex
  //! paths — the regression this guard pins).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{iftex}\n\
    \\begin{document}\n\
    engine:\\iftutex LUA\\else PDF\\fi. \
    dl:\\ifdefined\\directlua DEF\\else UNDEF\\fi.\n\
    \\end{document}\n";

  fn convert(preload: &str) -> String {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .arg(format!("--preload={preload}"))
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      output.status.success(),
      "binary exited: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml")
  }

  #[test]
  fn profile_flips_identity_only_when_opted_in() {
    let on = convert("[rawstyles,rawclasses,luatex]latexml.sty");
    assert!(
      on.contains("engine:LUA") && on.contains("dl:DEF"),
      "[luatex] must flip iftex probes and expose \\directlua:\n{on}",
    );
    let off = convert("[rawstyles,rawclasses]latexml.sty");
    assert!(
      off.contains("engine:PDF") && off.contains("dl:UNDEF"),
      "without [luatex] the pdfTeX identity must be untouched:\n{off}",
    );
  }
}

mod luacode_bridge {
  //! The texlua bridge (`latexml_engine::lua_bridge`) + luacode.sty binding:
  //! `{luacode}` bodies and `\luaexec` chunks execute in a persistent
  //! external texlua, and their `tex.print` output re-enters the TeX stream.
  //! Self-skips without a host `texlua` (CI trimmed-TL trap: a green run on
  //! such a host does not prove the bridge ran).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{luacode}\n\
    \\begin{document}\n\
    E:\\luaexec{tex.print(3+4)}.\n\
    \\begin{luacode}\n\
    local sum = 0\n\
    for i = 1, 10 do sum = sum + i end\n\
    tex.print(\"Sum: \" .. sum)\n\
    \\end{luacode}\n\
    after\n\
    \\end{document}\n";

  #[test]
  fn luacode_executes_via_texlua() {
    if !Command::new("texlua")
      .arg("--version")
      .output()
      .is_ok_and(|o| o.status.success())
    {
      return; // no texlua on this host
    }
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "luacode must digest cleanly:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("E:7") && xml.contains("Sum: 55") && xml.contains("after"),
      "lua output and following content must both survive:\n{xml}",
    );
  }
}

mod lua_state_mirror {
  //! `tex.count`/`tex.dimen` reads AND writes inside `\directlua` chunks are
  //! LIVE against engine State, via the bridge's query protocol. This is the
  //! "rebind-as-we-emulate" seam (docs/perfect_kernel/LUA_REBINDING.md):
  //! texlua has no engine, so any tex-state access a chunk makes must
  //! round-trip to OUR State — the previous stub returned zeros, which made
  //! every register-branching Lua chunk take the wrong path silently.
  //! Systemic witness class: babel's luababel.def chunks under the `luatex`
  //! profile (every profiled doc logged `attempt to index a nil value
  //! (field 'locale_props')` — chunks die mid-sequence, later chunks see
  //! missing state). Self-skips without a host texlua.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    \\count255=7 \\dimen0=2pt\n\
    \\lx@directlua{tex.count[100] = tex.getcount(255) + 35\n\
      tex.sprint('C' .. tex.count[255] .. 'D' .. tex.dimen[0])}\n\
    E\\the\\count100.\n\
    \\end{document}\n";

  #[test]
  fn directlua_reads_and_writes_live_registers() {
    if !Command::new("texlua")
      .arg("--version")
      .output()
      .is_ok_and(|o| o.status.success())
    {
      return; // no texlua on this host
    }
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    // \count255=7 read back; \dimen0=2pt as 131072 sp (LuaTeX convention:
    // tex.dimen reads in scaled points); the Lua-side write of count 100
    // visible to the following \the.
    assert!(
      xml.contains("C7D131072") && xml.contains("E42."),
      "live register mirror must round-trip both directions:\n{xml}",
    );
  }
}

mod expanded_protected_brace_hunt {
  //! TeX's `scan_left_brace` (tex.web) uses plain `get_x_token`, so a
  //! `\protected` macro EXPANDS while hunting the required `{` of a
  //! <general text>; protection inhibits expansion only during body
  //! absorption. Live-probed (pdflatex 2026-08-31): `\protected\def\pp
  //! {{abc}}\expanded\pp` typesets abc. Our read_balanced brace hunt read
  //! with fully_expand=false, erroring `Expected opening '{'` — one error
  //! per `\xinttheexpr` (its `\expanded\csname XINTexprprint…` lands on a
  //! \protected macro), the sweep-11 `expected:{` cluster (~16 xint docs;
  //! witnesses sim-os-menus-doc, ipsum-doc, tikz-bagua-en). Same
  //! argument-scanning-fidelity family as OXIDIZED_DESIGN #161.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\protected\\def\\pp{{abc}}\n\
    \\begin{document}\n\
    X\\expanded\\pp Y\n\
    \\end{document}\n";

  #[test]
  fn brace_hunt_expands_protected_macros() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "protected macro must expand in the brace hunt:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(xml.contains("XabcY"), "expanded body must survive:\n{xml}");
  }
}

mod raw_provides_version_survives {
  //! OXIDIZED_DESIGN #169 (surpass-Perl, user-approved 2026-08-31): the
  //! raw-TeX loader must NOT clobber `\ver@<file>` with `\fmtversion` when
  //! the file's own `\ProvidesPackage` already recorded its version — real
  //! LaTeX keeps the declared string, and date guards (`\GetFileInfo`,
  //! toptesi.cls L44-73's version comparison) read it. Perl shares the
  //! clobber (Package.pm L2393). Witness cluster: all 12 toptesi manuals
  //! abort with "the sty file you are using has a date of <empty>".

  use std::{path::Path, process::Command};

  const STY: &str = "\\ProvidesPackage{vguard}[2001/01/01 v9.9 Version guard fixture]\n\
    \\endinput\n";
  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{vguard}\n\
    \\begin{document}\n\
    V[\\expandafter\\meaning\\csname ver@vguard.sty\\endcsname]\n\
    \\end{document}\n";

  #[test]
  fn provides_package_version_not_clobbered() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("vguard.sty"), STY).expect("write sty");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--preload=[rawstyles]latexml.sty",
        "--dest",
        "t.xml",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("2001/01/01 v9.9 Version guard fixture"),
      "\\ver@vguard.sty must keep the ProvidesPackage string:\n{xml}",
    );
  }
}

mod accent_meaning_robust_shape {
  //! OXIDIZED_DESIGN #170 (surpass-Perl, user-approved 2026-08-31): text
  //! accents carry the LaTeX kernel's ROBUST structure — `\u` is a plain
  //! macro `\protect \u␣` with the real accent in the space-suffixed CS —
  //! so `\meaning\u` starts with `macro:`, as in real LaTeX. Both Perl
  //! (protected primitive) and our previous eTeX-protected macro made
  //! `\meaning` start with `\protected`, and tikzmath's 4-char meaning
  //! sniff (tikzlibrarymath.code.tex L22-46) then misclassified accent-CS
  //! variables (`\tikzmath{\u=int(...);}`) as keywords — the 11-doc
  //! `Error:latex:(tikz) Unknown function or keyword '\lx@applyaccent…'`
  //! cluster (witnesses cahierprof-doc, tikz-mirror-lens, colorblind_doc,
  //! sunpath.track). The accent must still typeset.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\begin{document}\n\
    M[\\meaning\\u]\n\
    A[\\u{o}]\n\
    \\end{document}\n";

  #[test]
  fn accent_meaning_is_kernel_robust() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    // Typeset \meaning output font-decodes `\`/`>` via OT1 (“/-¿ glyphs;
    // wisdom_ot1_angle_brackets_inverted), so assert the discriminating
    // prefix: `macro:` — NOT `\protected macro:` — is what tikzmath's
    // meaning sniff reads.
    assert!(
      xml.contains("M[macro:-") && !xml.contains("protected macro"),
      "\\meaning of a text accent must have the kernel robust shape:\n{xml}",
    );
    assert!(
      xml.contains("A[\u{014F}]") || xml.contains("A[o\u{0306}]"),
      "the accent must still typeset o-breve:\n{xml}",
    );
  }
}

mod openright_kernel_contract {
  //! book.cls L52/L98/L119 and report.cls L52/L98-99/L117: `\newif
  //! \if@openright` with true/false defaults respectively, driven by the
  //! openright/openany class options. Derived classes and docs poke the
  //! switch directly (toptesi.sty L329-342, amscls-doc handbooks) — the
  //! sweep-12 `\if@openright` cluster. Same kernel-contract precedent as
  //! `\if@mainmatter` (commit dba2a7eab0).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass[openright]{report}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    A[\\if@openright OR\\else OA\\fi]\n\
    \\@openrightfalse B[\\if@openright OR\\else OA\\fi]\n\
    \\end{document}\n";

  #[test]
  fn openright_switch_and_options_work() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "openright contract must digest:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("A[OR]") && xml.contains("B[OA]"),
      "option must set the switch and the setter must flip it:\n{xml}",
    );
  }
}

mod unicode_caret_notation {
  //! XeTeX/LuaTeX extended caret notation: `^^^^hhhh` (and `^^^^^^hhhhhh`)
  //! produce one Unicode scalar. Packages PROBE for a Unicode engine with
  //! it — newunicodechar.sty L52-56 `\edef\next{\@gobble^^^^0021}` fell
  //! into its 8-bit branch without it and raised "ASCII character
  //! requested" for EVERY \newunicodechar call (9-doc cluster: eigo,
  //! verifica ×5, tikz-trackschematic ×2, uspace). Unicode-native-engine
  //! precedent: same as providing \Ucharcat.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{newunicodechar}\n\
    \\newunicodechar{\u{00D7}}{x}\n\
    \\begin{document}\n\
    C[^^^^0041] U[3\u{00D7}4] S[^^^^^^01d49e]\n\
    \\end{document}\n";

  #[test]
  fn four_and_six_caret_forms_scan() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--preload=[rawstyles]latexml.sty",
        "--dest",
        "t.xml",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "newunicodechar must take its Unicode branch:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("C[A]") && xml.contains("U[3x4]") && xml.contains("S[\u{1D49E}]"),
      "caret forms must scan and the active-char mapping must fire:\n{xml}",
    );
  }
}

mod memoir_output_streams {
  //! memoir.cls output streams (L10965-11063) are CONTENT-BEARING: docs
  //! write body fragments to \jobname.<ext> and \input them back
  //! (dlfltxbmarkup-showkeys routes its whole body that way). Our memoir
  //! binding delegates to REAL TeX write streams so the round-trip works.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{memoir}\n\
    \\begin{document}\n\
    \\newoutputstream{keys}\n\
    \\openoutputfile{\\jobname.keys}{keys}\n\
    \\addtostream{keys}{ROUNDTRIP}\n\
    \\closeoutputstream{keys}\n\
    K[\\input{\\jobname.keys}]\n\
    \\end{document}\n";

  #[test]
  fn stream_write_and_readback() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("K[ROUNDTRIP"),
      "stream content must round-trip through the aux file:\n{xml}\n{stderr}",
    );
  }
}

mod graphicx_internals {
  //! Raw packages poke graphicx/graphics INTERNALS our bindings reimplement
  //! around (`wisdom_latexml_reimpl_internal_name_mismatch` shape): hvfloat
  //! calls `\Gin@boolkey{true}{iso}` (hvfloat.sty L411) which drives the
  //! `\newif\ifGin@iso` from graphics.sty L579 via graphicx.sty L137's
  //! two-arg csname dispatcher. Sweep-11 cluster: `\Gin@boolkey` 34 docs +
  //! `\Gin@draftfalse` 9 (bohr, pagelayout, …). The binding must carry the
  //! real internal names, faithfully ported from the sources.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{graphicx}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    \\Gin@boolkey{true}{iso}\\ifGin@iso ISOK\\else ISNO\\fi\n\
    \\Gin@boolkey{}{clip}\\ifGin@clip CLOK\\else CLNO\\fi\n\
    \\Gin@draftfalse\\ifGin@draft DRNO\\else DROK\\fi\n\
    \\end{document}\n";

  #[test]
  fn gin_internal_names_defined() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "Gin@ internals must digest cleanly:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("ISOK") && xml.contains("CLOK") && xml.contains("DROK"),
      "boolkey must flip the real newifs (empty #1 = true per graphicx.sty L137):\n{xml}",
    );
  }
}

mod luatex_babel_api {
  //! Under the `luatex` profile, babel's Lua API layer (luababel.def L196+,
  //! creating `Babel.locale_props`, `Babel.lua_error`, …) must actually run.
  //! In a real lualatex job `\bbl@luapatterns` lives in the FORMAT, so
  //! babel.def L1135 skips the patterns-only first `\input luababel.def`
  //! and the single in-document load (babel.def L2285) takes the API
  //! branch. Without that format fact, the patterns-only load ran first,
  //! `\endinput`ed at luababel.def L195, and the loaded-flag suppressed the
  //! second `\input` — so every later `Babel.locale_props[...]` chunk died.
  //! Systemic witnesses: every profiled clean-lualatex corpus doc logged
  //! `attempt to index a nil value (field 'locale_props')` (abntexto,
  //! abntexto-uece, derivative, newpax). Self-skips without texlua.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage[english]{babel}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    \\lx@directlua{tex.sprint(Babel and Babel.locale_props and 'BOK' or 'BNO')}\n\
    \\end{document}\n";

  #[test]
  fn babel_lua_api_layer_initializes() {
    if !Command::new("texlua")
      .arg("--version")
      .output()
      .is_ok_and(|o| o.status.success())
    {
      return; // no texlua on this host
    }
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--preload=[luatex]latexml.sty",
        "--dest",
        "t.xml",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("BOK"),
      "Babel.locale_props must exist after babel loads under the luatex profile:\n{xml}\n{stderr}",
    );
  }
}

mod filelist_letter_catcodes {
  //! OXIDIZED_DESIGN #166: `\@filelist` entries carry kernel catcodes —
  //! alphabetic chars as LETTER (`\string@makeletter`, latex.ltx L1784) —
  //! so source-level delimited parses over the list match. hep-font.sty's
  //! `\def\hepfont@get@class#1.cls#2\relax` + `\expandafter…\@filelist`
  //! idiom got an empty #1 under all-OTHER tokens, and the mis-split
  //! desynced conditional bookkeeping (13-bundle `expected:\fi` cluster).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\makeatletter\n\
    \\def\\get#1.cls#2\\relax{\\def\\res{#1}}\n\
    \\expandafter\\get\\@filelist\\relax\n\
    \\makeatother\n\
    \\begin{document}\n\
    res=[\\res]\n\
    \\end{document}\n";

  #[test]
  fn delimited_parse_of_filelist_matches() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    // #1 = everything before the first ".cls" — must contain the class name,
    // not be empty.
    assert!(
      xml.contains("res=[") && xml.contains("article]"),
      "delimited .cls parse over \\@filelist must capture the prefix:\n{xml}",
    );
  }
}

mod raw_classoptionslist_recorded {
  //! OXIDIZED_DESIGN #164 (class half): the kernel records the raw
  //! `\documentclass` option text in `\@raw@classoptionslist` (latex.ltx
  //! L18718, first class only). Modern babel reads exactly that list for
  //! global language options (babel.sty L4199) — without it
  //! `[french]{article}` + babel loads nil.ldf and `\og`/`\fg` are
  //! undefined. Babel isn't needed to guard the record itself.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass[french,11pt]{article}\n\
    \\begin{document}\n\
    raw=[\\makeatletter\\@raw@classoptionslist\\makeatother]\n\
    \\end{document}\n";

  #[test]
  fn documentclass_options_recorded_raw() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("raw=[french,11pt]"),
      "\\@raw@classoptionslist must carry the raw \\documentclass options:\n{xml}",
    );
  }
}

mod makeindex_allocates_indexfile {
  //! OXIDIZED_DESIGN #163: `\makeindex` allocates the `\@indexfile` write
  //! stream (real latex.ltx contract) while staying otherwise nooped, so raw
  //! doc.sty/l3doc-style `\protected@write\@indexfile{…}` works instead of
  //! erroring `undefined \@indexfile` (Perl noops it entirely and fatals on
  //! l3kernel's own manuals). The semantic `\index` constructor must remain
  //! in charge — \makeindex must NOT restore the kernel's raw `\index`.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\makeindex\n\
    \\begin{document}\n\
    body\\index{alpha}\n\
    \\makeatletter\n\
    \\ifdefined\\@indexfile STREAMDEFINED\\else STREAMMISSING\\fi\n\
    \\protected@write\\@indexfile{}{raw-write-payload}\n\
    \\makeatother\n\
    \\end{document}\n";

  #[test]
  fn stream_allocated_semantic_index_intact() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "\\makeindex + raw \\@indexfile write must be error-free:\n{stderr}",
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("STREAMDEFINED"),
      "\\@indexfile not allocated:\n{xml}"
    );
    // Semantic \index survived — an indexmark, and the raw payload is NOT
    // typeset into the document.
    assert!(
      xml.contains("indexmark") && !xml.contains("raw-write-payload"),
      "semantic \\index must stay in charge and raw writes must not leak:\n{xml}",
    );
  }
}

mod newtcblisting_verbatim {
  //! Regression test: a `\newtcblisting`-defined code box captures its body
  //! verbatim and CLOSES at `\end{name}` (ar5iv #504 / #569 / #570).
  //!
  //! The tcolorbox `listings` library's `\newtcblisting` reads its body as a code
  //! listing. The raw library's body capture did not integrate with LaTeXML's
  //! verbatim reader, so the listing ran past its `\end{name}` and swallowed the
  //! following content — a `\section` after the box ended up nested inside
  //! `<ltx:verbatim>` (`<ltx:section> isn't allowed in <ltx:verbatim>`) and the
  //! document failed to close. The binding now delegates `\newtcblisting` to
  //! listings' `\lstnewenvironment`, whose verbatim reader terminates correctly.
  //!
  //! Binary-driven (fresh process) so tcolorbox can raw-load its library files.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage[most]{tcolorbox}\n\
    \\tcbuselibrary{listings}\n\
    \\newtcblisting{mycodebox}[1][]{listing only,#1}\n\
    \\begin{document}\n\
    \\section{First}\n\
    \\begin{mycodebox}\n\
    some code line\n\
    another line\n\
    \\end{mycodebox}\n\
    \\section{Second}\n\
    Text after the box.\n\
    \\end{document}\n";

  #[test]
  fn newtcblisting_body_is_verbatim_and_closes() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");

    let output = Command::new(bin)
      .arg("t.tex")
      .arg("--dest")
      .arg("t.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    // No malformed-nesting / unclosed errors: the box body must not swallow the
    // following section.
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "newtcblisting box should close cleanly, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    // The second section and the text after the box survive OUTSIDE the listing.
    assert!(
      xml.contains("Text after the box"),
      "content after the box was swallowed by the listing:\n{xml}",
    );
    // Two real sections are present (the second didn't get eaten).
    assert_eq!(
      xml.matches("<section").count(),
      2,
      "expected 2 sections (First, Second) outside the box:\n{xml}",
    );
  }
}

mod fatal_salvages_partial_document {
  //! Regression test: a recoverable Fatal must not throw away the document.
  //!
  //! `digest_internal` (`latexml_oxide/src/core_interface.rs`) deliberately keeps
  //! consuming input after a recoverable Fatal so it can "still produce partial
  //! output" — Perl's `finishDigestion` L219-220. That intent silently only
  //! worked when the failure landed in a LATER body: `digest_next_body`
  //! accumulates into the stomach's `box_list` and hands it back only on the
  //! success path, so a Fatal inside the FIRST body left the caller's `boxes`
  //! empty and the run wrote a **39-byte empty document**.
  //!
  //! One pathological `\tikz` picture therefore cost a whole paper. Witnesses,
  //! all ar5iv user reports and all previously 0-byte:
  //!   * 2508.07407 (#556) → 31 KB (title/authors/abstract recovered)
  //!   * 2405.19920 (#522) → 1.82 MB, 6 sections + 80 bibitems — essentially the
  //!     complete paper, where same-host Perl produces **nothing** in 5 minutes
  //!   * 2501.10235 (#551) → 1.7 KB
  //!
  //! `stomach::salvage_pending_box_lists` unwinds the stranded levels. For the
  //! runaway guards (`Stomach:Recursion`) the innermost level IS the pathology —
  //! a repeating window grown past 50k boxes — so it is dropped and the suspended
  //! outer levels are kept: drop the offending construct, keep the document.

  use std::{path::Path, process::Command};

  /// Text before, then the `calc`-coordinate `\tikz` picture that drives the
  /// box-cycle guard (reduced from arXiv:2508.07407), then text after.
  const RECURSION_TEX: &str = "\\documentclass{article}\n\
    \\usepackage{tikz}\n\
    \\usetikzlibrary{shapes.symbols,calc,positioning}\n\
    \\begin{document}\n\
    \\section{Before the bad picture}\n\
    UNIQUEMARKERBEFORE some ordinary prose that must survive.\n\
    \n\
    \\tikz[baseline=(env.base),node distance=4mm]{%\n\
      \\node[cloud, draw, inner sep=13pt, minimum width=40mm, minimum height=20mm] (env) {Env};\n\
      \\node[circle, draw, minimum size=6mm] (A1) at ($(env.west)+(10mm,6mm)$) {};\n\
      \\node[circle, draw, minimum size=6mm] (A2) at ($(env.east)+(-10mm,6mm)$) {};\n\
      \\node[circle, draw, minimum size=6mm] (A3) at ($(env.north)+(0,-24mm)$) {};\n\
      \\draw[->, thick] (A1) -- (A2);\n\
      \\draw[->, thick] (A2) -- (A3);\n\
      \\draw[->, thick] (A3) -- (A1);\n\
    }\n\
    \n\
    \\end{document}\n";

  #[test]
  fn recoverable_fatal_keeps_the_already_digested_document() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("rec.tex"), RECURSION_TEX).expect("write rec.tex");

    let output = Command::new(bin)
      .args([
        "rec.tex",
        "--dest",
        "rec.xml",
        "--nocomments",
        "--timeout",
        "120",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);

    let xml = std::fs::read_to_string(workdir.path().join("rec.xml")).unwrap_or_default();

    // The Fatal MUST still be reported — salvaging partial output is not a
    // licence to downgrade the diagnostic. If a future fix makes this input
    // convert outright the assertion below still holds and this one should be
    // revisited deliberately, not deleted.
    assert!(
      stderr.contains("Fatal:") || xml.contains("UNIQUEMARKERBEFORE"),
      "expected either the Fatal to be reported or the document to convert:\n{stderr}",
    );

    // The point of the test: content digested BEFORE the pathological construct
    // survives. Pre-fix this file was 39 bytes with the prose gone.
    assert!(
      xml.contains("UNIQUEMARKERBEFORE"),
      "prose preceding the runaway construct was lost — the whole document was \
       thrown away by one bad picture (rec.xml is {} bytes):\n{xml}",
      xml.len(),
    );
    assert!(
      xml.len() > 400,
      "output is a {}-byte stub, so nothing was salvaged:\n{xml}",
      xml.len(),
    );

    // ...and the SUMMARY must agree with the log. Recovering boxes is NOT a
    // licence to reclassify the verdict: a Fatal-level raise stays Fatal in the
    // document's reported outcome (user policy 2026-07-28), and the graceful
    // salvage below is a *feature* of that Fatal, not a downgrade of it.
    //
    // `digest_internal` used to emit its recovered Fatal with the raw
    // `log::error!` macro rather than `Error::log_fatal`, so nothing reached
    // `note_status` and the tally stayed empty: this very input printed
    // `Fatal:Stomach:Recursion` and then signed off with "Conversion complete:
    // No obvious problems" — status code 0, i.e. "ok" to cortex (which reads
    // `get_status_code`) and clean to any check that does not scrape the log. A
    // run that reports a Fatal and summarises as problem-free is the false
    // negative CLAUDE.md forbids outright.
    // There is exactly one verdict line (`converter.rs`, folding in
    // `bin/latexml:127`'s failed/complete choice), and it is the run's FINAL
    // word — so assert its exact text and its position, not merely that the word
    // "fatal" occurs somewhere in the stream.
    let verdict = stderr
      .lines()
      .find(|l| l.contains("Conversion failed:") || l.contains("Conversion complete:"))
      .unwrap_or_else(|| panic!("no conversion verdict line in stderr:\n{stderr}"));
    let tail: Vec<&str> = stderr.lines().rev().take(5).collect();
    assert!(
      tail.contains(&verdict),
      "the verdict is not among the last 5 lines of stderr, so it is not the \
       final status:\n{stderr}",
    );

    // Both directions, so neither seam can drift from the other again:
    // a `Fatal:` in the log REQUIRES the fatal verdict, and no `Fatal:` forbids
    // it. (The verdict is `(Finalizing... )`-prefixed, hence `ends_with`.)
    if stderr.contains("Fatal:") {
      // "1 warning; 1 fatal error", not "1 fatal error" alone: the salvage
      // path's own `Warning:…digest_internal` note is a raw `log::warn!`, and
      // since the lossless-tally fix (2026-08-02) every printed diagnostic
      // record counts — the warning's presence in the tally is that fix
      // working, not tally noise.
      assert!(
        verdict.ends_with("Conversion failed: 1 warning; 1 fatal error"),
        "the log reports a Fatal (and the salvage warning), so the final \
         status must be exactly \"Conversion failed: 1 warning; 1 fatal \
         error\" — recovering boxes is not a licence to reclassify the \
         verdict. Got:\n  {verdict}\n{stderr}",
      );
    } else {
      assert!(
        !verdict.contains("fatal"),
        "the final status claims a fatal that never appears in the log:\n  \
         {verdict}\n{stderr}",
      );
    }

    // And the runaway's own boxes must NOT be grafted in: the guard trips at
    // 50k repeated boxes, so salvaging that level would produce a vast garbage
    // document rather than a small honest one.
    assert!(
      xml.len() < 2_000_000,
      "output is {} bytes — the runaway box window looks like it was salvaged \
       into the document instead of dropped",
      xml.len(),
    );
  }
}

mod aligned_overset_includestyles {
  //! Regression test for the `aligned-overset` raw-load breaking amsmath
  //! alignments (`latexml_contrib/src/aligned_overset_sty.rs`).
  //!
  //! `aligned-overset.sty` is an expl3 package that rewrites `\overset`/`\underset`
  //! to wrap themselves in `\group_align_safe_begin: … \group_align_safe_end:`
  //! around an `\hbox_set:` box measurement — purely to re-centre the accent on the
  //! cell's alignment point, a PDF-visual cosmetic with no MathML meaning. When the
  //! raw `.sty` is loaded (INCLUDE_STYLES / the ar5iv profile — bare it is ignored),
  //! an `\overset` inside an `align` cell fires `\lx@begin@alignment Attempt to close
  //! a group that switched to mode math`, corrupts math mode for the rest of the
  //! block, and cascades into hundreds of `unexpected:_`/`^`. Witness 2203.05327
  //! (ar5iv): 411 errors → 0 with the near-no-op binding, which keeps amsmath's
  //! `\overset`/`\underset` and drops the cosmetic.
  //!
  //! Driven through the binary with `--includestyles` so the contrib binding must
  //! pre-empt the host-texmf raw `.sty` (the exact ar5iv path). Without the binding
  //! this run emits ~15 `\lx@begin@alignment`/`unexpected:_` errors.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{amsmath,aligned-overset}\n\
    \\newcommand{\\tor}{\\text{Tor}}\n\
    \\begin{document}\n\
    \\begin{align}\n\
    a\\overset{\\text{}}{=}0,&& \\tor^S_{q}(M,C_p)=0.\n\
    \\end{align}\n\
    After the align: $H_{q}(P_\\bullet)=0$ stays math.\n\
    \\end{document}\n";

  #[test]
  fn aligned_overset_rawload_does_not_break_amsmath_alignment() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");

    let output = Command::new(bin)
      .arg("a.tex")
      .arg("--dest")
      .arg("a.xml")
      .arg("--nocomments")
      .arg("--includestyles")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    // The near-no-op binding must pre-empt the raw expl3 `.sty`; the alignment is
    // then error-clean (was ~15 `\lx@begin@alignment`/`unexpected:_` errors).
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "aligned-overset + \\overset-in-align should be error-clean, stderr had errors:\n{stderr}",
    );
    // Sanity: the overset and the post-align subscript both made it into MathML.
    let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
    assert!(
      xml.contains("OVERACCENT"),
      "\\overset should still emit an OVERACCENT mover:\n{xml}",
    );
  }
}

mod lstinputlisting_range_crlf {
  //! Regression tests: `\lstinputlisting` over an externally-read source file —
  //! a truncating line range, and CRLF line terminators.
  //!
  //! Witness: arXiv 2412.04705 (arXiv/html_feedback#6735, "Wrong code snippet in
  //! html display"), whose `\inputpython` wraps
  //! `\lstinputlisting[firstline=32,lastline=35,...]` over CRLF Python sources.
  //! Both defects are shared with Perl LaTeXML; see OXIDIZED_DESIGN #68 / #69 and
  //! `KNOWN_PERL_ERRORS.md`.
  //!
  //! 1. **Truncating range** (`listings_sty.rs` "Remove trailing empty lines").
  //!    `lastline=N` on a file with MORE than N lines cut the generated token
  //!    vector at `emptyfrom`, discarding `}` tokens that closed groups opened
  //!    BEFORE the cut — measured discarded tail on the witness:
  //!    `["\@lst@startline", "{", "}", "}", "}", "}", "\@lst@endline"]`, three of
  //!    them closers. The listing body was emitted with unclosed groups, so
  //!    `\@@listings@block` read its arguments off the end of the DOCUMENT and
  //!    everything after the listing was swallowed.
  //!
  //! 2. **CRLF** (`listings_read_raw_file`). Every end-of-line test in the
  //!    listings processor is written against `\n`; a `\r` before it defeats them,
  //!    so a line comment never terminates and its STYLE (not its class — the
  //!    `ltx_lst_comment` wrapper does close) bleeds over every following line.
  //!    pdflatex on the witness renders only the `#` line in comment green
  //!    (9 green vs 69 black glyph groups); both LaTeXML engines painted the whole
  //!    snippet green.
  //!
  //! Binary-driven (fresh process) so the listing file is read from disk.

  use std::{path::Path, process::Command};

  /// CRLF on purpose — this is half of what is under test.
  const DATA_PY: &str = "# a comment line\r\nvalue = 1\r\nother = 2\r\nlast = 3\r\n";

  fn convert(tex: &str, data: &str) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("data.py"), data).expect("write data.py");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");

    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    (xml, stderr)
  }

  /// Collect the text of each `<listingline>`, in order.
  fn listing_lines(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in xml.split("<listingline").skip(1) {
      let Some(end) = chunk.find("</listingline>") else {
        continue;
      };
      out.push(chunk[..end].to_string());
    }
    out
  }

  fn strip_tags(fragment: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in fragment.chars() {
      match ch {
        '<' => in_tag = true,
        '>' => in_tag = false,
        c if !in_tag => text.push(c),
        _ => {},
      }
    }
    text
  }

  #[test]
  fn lastline_shorter_than_file_does_not_swallow_the_document() {
    // `lastline=3` over a 4-line file: the truncation path is exercised.
    let tex = "\\documentclass{article}\n\
      \\usepackage{listings}\n\
      \\begin{document}\n\
      \\lstinputlisting[lastline=3]{data.py}\n\
      Text after the listing.\n\
      \\end{document}\n";
    let (xml, stderr) = convert(tex, DATA_PY);

    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "truncating lastline should convert cleanly, stderr had:\n{stderr}",
    );
    // The document continues after the listing — the unbalanced body used to make
    // `\@@listings@block` read its arguments to EOF, losing everything after it.
    assert!(
      xml.contains("Text after the listing"),
      "content after the listing was swallowed:\n{xml}",
    );
    let lines = listing_lines(&xml);
    assert_eq!(
      lines.len(),
      3,
      "expected exactly lines 1..3 of the file, got {}:\n{xml}",
      lines.len()
    );
    assert!(
      strip_tags(&lines[2]).contains("other = 2"),
      "third listing line should be the file's line 3:\n{}",
      strip_tags(&lines[2])
    );
    assert!(
      !xml.contains("last = 3"),
      "line 4 is past lastline=3 and must not appear:\n{xml}",
    );
  }

  #[test]
  fn crlf_line_comment_style_does_not_bleed_past_its_line() {
    // `\r\n` terminators: only the `#` line is a comment. The class wrapper always
    // closed correctly; it is the STYLE that used to leak, so assert on `font`.
    let tex = "\\documentclass{article}\n\
      \\usepackage{listings}\n\
      \\lstdefinestyle{s}{morecomment=[l]{\\#},commentstyle=\\itshape}\n\
      \\begin{document}\n\
      \\lstinputlisting[style=s]{data.py}\n\
      \\end{document}\n";
    let (xml, stderr) = convert(tex, DATA_PY);

    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "CRLF listing should convert cleanly, stderr had:\n{stderr}",
    );
    let lines = listing_lines(&xml);
    assert_eq!(lines.len(), 4, "expected all 4 file lines:\n{xml}");

    assert!(
      lines[0].contains("font=\"italic\""),
      "the comment line should carry the commentstyle:\n{}",
      lines[0]
    );
    for (i, line) in lines.iter().enumerate().skip(1) {
      assert!(
        !line.contains("font=\"italic\""),
        "line {} is code, but the comment style bled into it:\n{line}",
        i + 1
      );
    }
  }
}

mod bib_field_digest_once {
  //! Regression test: a bibliography field value is digested EXACTLY ONCE.
  //!
  //! The original defect was two digesting paths in the since-deleted
  //! `convert_bib_file_to_xml` string route — `interpret_tex_markup` (XML
  //! fragment, so `\url`/`\href`/font switches survive) and `interpret_tex_text`
  //! (plain string) — both run over the SAME value, so every error that field
  //! raised was reported twice and every macro side effect ran twice. That route
  //! is gone (`BIBLIOGRAPHY_WORKLIST.md` re-port item 1: the recursive `.bib`
  //! session replaced it), but the PROPERTY it violated is a standing one for
  //! whatever route is current, and it is cheap to keep pinned.
  //!
  //! This is guarded by counting `Error:` lines rather than by inspecting the XML,
  //! because the duplicate is invisible in the output — the rendered entry looked
  //! perfectly fine while the document's error count silently doubled. Error
  //! counts are the canvas pass/fail signal, so inflating them is a real defect.
  //!
  //! Binary-driven: the count has to come from the conversion log.

  use std::{path::Path, process::Command};

  /// Two properties this fixture must have, both learned the hard way:
  ///
  /// * The probe must raise its error on EVERY digest. An undefined macro will
  ///   NOT do: it is defined as `<ltx:ERROR/>` on first sight and is therefore
  ///   silently self-healing on a second pass, so an undefined-macro fixture
  ///   passes even with the bug present. `\hline` in a `note` is the probe — it
  ///   expands to `\noalign`, which is a CONTEXT error (`\noalign cannot be used
  ///   here`) with nothing to memoize, so a second digest counts a second time.
  ///   Verified: two entries each carrying one `\hline` produce exactly 2.
  /// * The value must contain a BACKSLASH. The interpretation paths short-circuit
  ///   on a value with no `\`, `~` or `$`, so a punctuation-only probe never
  ///   digests at all and the test goes vacuously green (observed: "digested 0
  ///   times"). `\textbf` is the carrier in the second entry because it needs no
  ///   package.
  ///
  /// `_` and `^` were the two earlier probes and neither can be one any more:
  /// OXIDIZED_DESIGN #74 escapes `_ & # %` and `^` in a `.bib` field as DATA, so
  /// `note={a _ … ^ …}` now renders the literal characters and raises nothing.
  /// The `a2` entry keeps both of them as the standing check that the escaping did
  /// not disturb the once-only property — it must contribute ZERO errors — while
  /// `a1`'s `\hline` is the live probe.
  const BIB: &str = r"@article{a1, author={Doe, J.}, title={T}, year={2020},
    note={a \hline \textbf{b}} }
  @article{a2, author={Roe, R.}, title={T2}, year={2021},
    note={x _ y ^ z \textbf{w}} }
  ";

  const TEX: &str = r"\documentclass{article}
  \begin{document}
  See \cite{a1,a2}.
  \bibliographystyle{plain}
  \bibliography{refs}
  \end{document}
  ";

  #[test]
  fn bib_field_errors_are_reported_once_not_once_per_digest() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("refs.bib"), BIB).expect("write refs.bib");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");

    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.html",
        "--format=html5",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    // ANSI-strip before counting: a naive grep over coloured output matches zero
    // and would make this test vacuously green.
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));

    let needle = "\\noalign cannot be used here";
    let n = stderr.matches(needle).count();
    assert_eq!(
      n, 1,
      "the field was digested {n} times, not once — bibliography errors are \
       being multiplied into the document's error count.\nstderr:\n{stderr}"
    );
    // `_` and `^` are DATA in a `.bib` field (OXIDIZED_DESIGN #74), so `a2` must
    // raise nothing. Asserted rather than dropped: if the escaping ever regresses
    // this catches it here too, and once-per-digest would show up as a count of 2.
    for script in ['_', '^'] {
      let n = stderr
        .matches(&format!("Script {script} can only appear in math mode"))
        .count();
      assert_eq!(
        n, 0,
        "a `{script}` in a bib field is data and must raise nothing, got {n}.\n\
         stderr:\n{stderr}"
      );
    }
  }

  fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
      if c == '\u{1b}' && chars.peek() == Some(&'[') {
        for c in chars.by_ref() {
          if c.is_ascii_alphabetic() {
            break;
          }
        }
      } else {
        out.push(c);
      }
    }
    out
  }
}

mod silence_keeps_diagnostics {
  //! `silence.sty` must never cost us a real diagnostic
  //! (`latexml_contrib/src/silence_sty.rs`).
  //!
  //! Unlike the `arxiv.sty` sibling, the silence binding is deliberately NOT
  //! gated on `INCLUDE_STYLES`: it pre-empts the raw `.sty` in every
  //! configuration. The reason is measurable. The real silence.sty rebinds
  //! `\PackageError` / `\ClassError` / `\@latex@error` / `\GenericError`
  //! (silence.sty L582-599) so that `\ErrorsOff` drops messages before they
  //! are printed — and under LaTeXML those are the very definitions that turn
  //! a package's error into an `Error:` line. Measured on the fixture below,
  //! same-host Perl 0.8.8 with `--includestyles` reports **0 errors**; without
  //! `\usepackage{silence}` the same document reports **1**. The raw load
  //! silently downgrades a genuine diagnostic.
  //!
  //! The binding models only what silence contributes to the *document*
  //! (nothing) and leaves the error/warning definitions alone, so the
  //! diagnostic survives. This test pins that: the run must still report the
  //! `boompkg` error even with silence loaded and `\ErrorsOff` in force.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{silence}\n\
    \\ErrorsOff\n\
    \\usepackage{boompkg}\n\
    \\begin{document}\n\
    x\n\
    \\end{document}\n";

  const STY: &str = "\\ProvidesPackage{boompkg}\n\
    \\PackageError{boompkg}{Deliberate boom}{}\n";

  #[test]
  fn silence_errorsoff_does_not_swallow_a_package_error() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");
    std::fs::write(workdir.path().join("boompkg.sty"), STY).expect("write boompkg.sty");

    let output = Command::new(bin)
      .arg("a.tex")
      .arg("--dest")
      .arg("a.xml")
      .arg("--nocomments")
      .arg("--includestyles")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      stderr.contains("Deliberate boom"),
      "silence + \\ErrorsOff must not suppress the boompkg error:\n{stderr}",
    );
  }
}

mod preclass_kernel_autoload {
  //! The on-undefined LaTeX-kernel autoload and its two hard boundaries.
  //!
  //! `latexml_engine/src/latex_kernel.rs` loads `LaTeX.pool` when an *undefined*
  //! control sequence turns out to be one the ambient kernel dump defines, so a
  //! document may use a kernel command before `\documentclass` — as real LaTeX
  //! allows, since `latex.ltx` IS the format. The happy path is guarded by the
  //! `tests/structure/preclass_*.tex+.xml` pairs (`preclass_iffileexists_test`,
  //! `preclass_kernel_cs_test`).
  //!
  //! This file guards the two places the mechanism must deliberately stay out of.
  //! Both are binary-driven (fresh process) because they are process-level modes.

  use std::{path::Path, process::Command};

  /// The witness idiom (arXiv 2605.25877, 2606.06905): `\IfFileExists` is not on
  /// Perl's `TeX.pool.ltxml` L33-56 trigger list, so without the autoload the
  /// conditional collapses and the *rejected* branch's class is what gets picked.
  const PRECLASS_TEX: &str = concat!(
    "\\IfFileExists{ltxo-no-such-class.cls}",
    "{\\documentclass{ltxo-no-such-class}}{\\documentclass{article}}\n",
    "\\begin{document}\n",
    "Selected the fallback class.\n",
    "\\end{document}\n"
  );

  fn convert(env: &[(&str, &str)]) -> String {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("p.tex"), PRECLASS_TEX).expect("write p.tex");
    let mut cmd = Command::new(bin);
    cmd
      .args(["p.tex", "--dest", "p.xml", "--nocomments"])
      .current_dir(workdir.path());
    for (k, v) in env {
      cmd.env(k, v);
    }
    let output = cmd.output().expect("spawn latexml_oxide");
    // The logger TTY-gates colours, so a piped stderr is ANSI-free; strip anyway
    // (project signal-integrity rule — never let a parse miss hide a diagnostic).
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_default();
    format!("{stderr}\n<<<XML>>>\n{xml}")
  }

  /// Baseline for the two negative tests below: with a dump present the autoload
  /// fires, the FALSE branch's class wins, and nothing is reported undefined.
  #[test]
  fn pre_documentclass_kernel_cs_selects_the_right_class() {
    let out = convert(&[]);
    assert!(
      !out.contains("Error:undefined:\\IfFileExists"),
      "\\IfFileExists before \\documentclass must autoload the LaTeX kernel:\n{out}"
    );
    assert!(
      out.contains("<?latexml class=\"article\"?>"),
      "the \\IfFileExists FALSE branch must select `article`:\n{out}"
    );
  }

  /// `LoadFormat('latex')` has two mutually exclusive branches (CLAUDE.md durable
  /// parity rule 1). On the degraded one there is no dump to test membership
  /// against, so the autoload must not fire at all and behaviour must stay
  /// exactly as it was before the mechanism existed: the Perl `TeX.pool` L33-56
  /// trigger list is the only thing that loads the format, and `\IfFileExists` —
  /// which is not on it — is reported undefined.
  ///
  /// This asserts a *limitation on purpose*. If the no-dump branch ever gains a
  /// membership oracle of its own, change this test deliberately; do not delete
  /// it to make a run green.
  #[test]
  fn nodump_leaves_pre_documentclass_kernel_cs_undefined() {
    let out = convert(&[("LATEXML_NODUMP", "1")]);
    assert!(
      out.contains("Error:undefined:\\IfFileExists"),
      "with LATEXML_NODUMP the kernel autoload has no oracle and must stay inert:\n{out}"
    );
  }
}

mod acmart_description_aria {
  //! acmart `\Description` must reach the **HTML** as a usable text alternative.
  //!
  //! The core-XML fixture (`tests/complex/acm_aria.{tex,xml}`) pins the
  //! image-less shape, but everything that makes this feature actually work for a
  //! screen reader happens in post-processing: the description has to become the
  //! image's `@alt`, `aria:describedby` has to survive as `aria-describedby`, the
  //! referenced ids have to resolve, and the referenced text has to be clean. A
  //! core-only test is green on all of those failing.
  //!
  //! What this guards, all of which were broken before (see
  //! `KNOWN_PERL_ERRORS.md` #66, `OXIDIZED_DESIGN_DIVERGENCES.md` #83):
  //!   * the MANDATORY long description reached no output at all — Perl's
  //!     binding emits `#1`, the OPTIONAL short one, and drops `#2`
  //!   * the relation was `aria:labelledby`, then `aria:label`, on the FLOAT.
  //!     Both set the accessible NAME, so both displaced the caption — the
  //!     reviewer report that prompted the current shape
  //!     (brucemiller/LaTeXML#430 r3674103638). The text alternative belongs on
  //!     the image, as `@alt`; nothing here may emit `aria-label` at all.
  //!   * the note carried footnote scaffolding, so the announced text began
  //!     "†† : " (`ltx_note_mark` twice, then an `ltx_note_type` prefix)
  //!   * an intermediate fix emitted the short description with NO id, leaving
  //!     `aria-describedby` pointing at nothing — hence the dangling-ref check

  use std::{path::Path, process::Command};

  /// A real (1×1) PNG, so `\includegraphics` produces an `<img>` rather than a
  /// missing-file diagnostic — the whole point here is where the alt text lands.
  const PNG: &[u8] = include_bytes!("graphics/none.png");

  /// One figure per branch of the mapping in OXIDIZED_DESIGN_DIVERGENCES #83.
  /// The first four are the primary path (a lone image in the float); the last
  /// two are the cases that keep the wiring on the float.
  const TEX: &str = "\\documentclass[acmsmall]{acmart}\n\
    \\usepackage{graphicx}\n\
    \\begin{document}\n\
    \\begin{figure}\\includegraphics{none}\n\
    \\caption{CAPTIONONE}\n\
    \\Description[SHORTDESC]{LONGDESC with \\emph{markup} inside}\n\
    \\end{figure}\n\
    \\begin{figure}\\includegraphics{none}\n\
    \\caption{CAPTIONTWO}\n\
    \\Description{LONELYLONGDESC}\n\
    \\end{figure}\n\
    \\begin{figure}\\includegraphics{none}\n\
    \\caption{CAPTIONTHREE}\n\
    \\Description{MARKUPDESC with \\emph{emphasis}}\n\
    \\end{figure}\n\
    \\begin{figure}\\includegraphics[alt={AUTHORALT}]{none}\n\
    \\caption{CAPTIONFOUR}\n\
    \\Description[SHORTFOUR]{LONGFOUR text}\n\
    \\end{figure}\n\
    \\begin{figure}\\includegraphics{none}\\includegraphics{none}\n\
    \\caption{CAPTIONFIVE}\n\
    \\Description[SHORTFIVE]{LONGFIVE text}\n\
    \\end{figure}\n\
    \\begin{figure}NOIMAGEHERE\n\
    \\caption{CAPTIONSIX}\n\
    \\Description[SHORTSIX]{LONGSIX text}\n\
    \\end{figure}\n\
    \\end{document}\n";

  /// Every `<img ...>` in the document, in order.
  fn img_tags(html: &str) -> Vec<&str> {
    html
      .match_indices("<img ")
      .filter_map(|(i, _)| html[i..].find('>').map(|e| &html[i..i + e + 1]))
      .collect()
  }

  /// The value of `attr` on `tag`, if present.
  fn attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{attr}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    rest.find('"').map(|e| &rest[..e])
  }

  #[test]
  fn description_becomes_the_images_alt_text() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("d.tex"), TEX).expect("write d.tex");
    std::fs::write(workdir.path().join("none.png"), PNG).expect("write none.png");

    let output = Command::new(bin)
      .args([
        "d.tex",
        "--dest",
        "d.html",
        "--format",
        "html5",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let html = std::fs::read_to_string(workdir.path().join("d.html")).unwrap_or_default();
    assert!(!html.is_empty(), "no HTML produced:\n{stderr}");

    // 0. NOTHING here may set an accessible NAME. `aria-label` on the float was
    //    the reported defect: it replaces the name, and a figure's name is its
    //    caption, so the caption stopped being announced.
    assert!(
      !html.contains("aria-label"),
      "\\Description must never set an accessible name — that displaces the \
       caption (brucemiller/LaTeXML#430 r3674103638):\n{html}",
    );
    for caption in ["CAPTIONONE", "CAPTIONTWO", "CAPTIONSIX"] {
      assert!(
        html.contains(caption),
        "caption {caption} vanished:\n{html}"
      );
    }

    // 1. The long description — the alternative ACM mandates — must be present.
    for text in ["LONGDESC", "LONELYLONGDESC", "MARKUPDESC", "LONGSIX"] {
      assert!(
        html.contains(text),
        "the description {text} never reached the HTML:\n{stderr}",
      );
    }

    let imgs = img_tags(&html);
    assert!(
      imgs.len() >= 6,
      "expected an <img> per graphic, saw {}:\n{html}",
      imgs.len()
    );

    // 2. A lone image in the float IS what the description is an alternative to,
    //    so it carries it — as `@alt`, the attribute an <img> has for exactly
    //    this, not `aria-label`. `[short]` is the concise alternative; a lone
    //    plain `{long}` stands in directly.
    assert_eq!(
      attr(imgs[0], "alt"),
      Some("SHORTDESC"),
      "the short description should be the image's alt:\n{}",
      imgs[0]
    );
    assert_eq!(
      attr(imgs[1], "alt"),
      Some("LONELYLONGDESC"),
      "a lone plain description should become the alt directly:\n{}",
      imgs[1]
    );

    // 3. A lone description carrying MARKUP cannot go in an attribute, so the alt
    //    keeps the generic fallback and the text is referenced as a block.
    assert_eq!(
      attr(imgs[2], "alt"),
      Some("Refer to caption"),
      "markup cannot live in an alt attribute; it must fall back to a block:\n{}",
      imgs[2]
    );
    assert!(
      attr(imgs[2], "aria-describedby").is_some(),
      "a markup-bearing description must still be referenced:\n{}",
      imgs[2]
    );

    // 4. An explicit `\includegraphics[alt=…]` names ONE image while \Description
    //    names the float, so the more specific statement wins and we only add
    //    references — never clobber the author's alt.
    assert_eq!(
      attr(imgs[3], "alt"),
      Some("AUTHORALT"),
      "an explicit alt= must survive a competing \\Description:\n{}",
      imgs[3]
    );
    let refs_four = attr(imgs[3], "aria-describedby").unwrap_or_default();
    assert_eq!(
      refs_four.split_whitespace().count(),
      2,
      "with the alt already taken, BOTH descriptions should be referenced:\n{}",
      imgs[3]
    );

    // 5. Several images: the description covers the ensemble, so it stays on the
    //    float rather than being asserted as panel 1's alternative.
    for img in &imgs[4..6] {
      assert_eq!(
        attr(img, "alt"),
        Some("Refer to caption"),
        "a multi-panel figure's description must not be claimed by one panel:\n{img}",
      );
    }
    assert!(
      html.contains("aria-describedby=\"acmlabel5-short acmlabel5\""),
      "a multi-image float should carry the references itself:\n{html}",
    );
    // …and the image-less float likewise, which is the acm_aria fixture's shape.
    assert!(
      html.contains("aria-describedby=\"acmlabel6-short acmlabel6\""),
      "an image-less float should carry the references itself:\n{html}",
    );

    // 5b. Falling back to the float is second-best, so it is announced — but ONLY
    //     then. Exactly the two floats above may warn; the four lone-image
    //     figures must be silent, or every ordinary ACM paper turns noisy.
    let warnings = stderr.matches("Warning:unexpected:\\Description").count();
    assert_eq!(
      warnings, 2,
      "expected a warning for the multi-image and the image-less float, and \
       silence for the four that found their image:\n{stderr}",
    );
    for reason in ["more than one image", "no image to describe"] {
      assert!(
        stderr.contains(reason),
        "the warning should say WHY it fell back ({reason}):\n{stderr}",
      );
    }

    // 6. EVERY aria-describedby reference resolves to a real id. An unresolved
    //    reference is silently inert — the description is simply never announced.
    let ids: Vec<String> = html
      .match_indices("id=\"")
      .filter_map(|(i, _)| {
        let rest = &html[i + 4..];
        rest.find('"').map(|e| rest[..e].to_string())
      })
      .collect();
    let mut checked = 0;
    for (i, _) in html.match_indices("aria-describedby=\"") {
      let rest = &html[i + 18..];
      let end = rest.find('"').expect("unterminated aria-describedby");
      for r in rest[..end].split_whitespace() {
        assert!(
          ids.iter().any(|id| id == r),
          "aria-describedby references '{r}', which no element defines:\n{html}",
        );
        checked += 1;
      }
    }
    assert!(
      checked >= 6,
      "expected a reference per describing figure, saw {checked}"
    );

    // 7. The referenced text is CLEAN: no footnote scaffolding, which would
    //    otherwise be announced as part of the description.
    for marker in ["ltx_note_mark", "ltx_note_type"] {
      assert!(
        !html.contains(marker),
        "the description carries footnote scaffolding ({marker}), which lands in \
         the announced accessible description:\n{html}",
      );
    }

    // 8. And the whole thing converts cleanly — reading the description
    //    `Undigested` means nothing inside it is expanded, so no error can be
    //    manufactured from content pdflatex never expands either.
    assert!(
      !stderr.contains("Error:"),
      "expected a clean conversion:\n{stderr}",
    );
  }

  /// Two malformed-but-real shapes that must still not lose what the author
  /// wrote. Both were regressions caught in review of the change that moved
  /// `\Description` onto the image.
  const TEX_ODD: &str = "\\documentclass[acmsmall]{acmart}\n\
    \\usepackage{graphicx}\n\
    \\begin{document}\n\
    Loose text. \\Description[LOOSESHORT]{LOOSELONG text} more text.\n\
    \\begin{figure}\\includegraphics{none}\n\
    \\caption{TWICE}\n\
    \\Description[FIRSTSHORT]{FIRSTLONG text}\\Description[SECONDSHORT]{SECONDLONG text}\n\
    \\end{figure}\n\
    \\end{document}\n";

  #[test]
  fn description_never_loses_an_annotation() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("d.tex"), TEX_ODD).expect("write d.tex");
    std::fs::write(workdir.path().join("none.png"), PNG).expect("write none.png");

    let output = Command::new(bin)
      .args([
        "d.tex",
        "--dest",
        "d.html",
        "--format",
        "html5",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let html = std::fs::read_to_string(workdir.path().join("d.html")).unwrap_or_default();
    assert!(!html.is_empty(), "no HTML produced:\n{stderr}");

    // A \Description outside any float has no image AND no float to fall back to.
    // It still has to land somewhere, and the warning has to name the real cause
    // rather than blame a float that isn't there.
    assert!(
      html.contains("LOOSELONG"),
      "a \\Description outside a float was dropped:\n{html}",
    );
    assert!(
      stderr.contains("outside any figure or table"),
      "the warning must name the actual cause, not a missing image:\n{stderr}",
    );

    // A SECOND \Description in the same float must not overwrite the first one's
    // reference — `aria-describedby` is an id list, and a clobbered id leaves
    // that description hidden in the DOM and announced by nothing.
    let imgs = img_tags(&html);
    let refs = attr(imgs[0], "aria-describedby").unwrap_or_default();
    assert!(
      refs.split_whitespace().count() >= 3,
      "a second \\Description clobbered the first one's reference; expected the \
       first long id plus both of the second's, got {refs:?}:\n{}",
      imgs[0]
    );
    assert_eq!(
      attr(imgs[0], "alt"),
      Some("FIRSTSHORT"),
      "the first \\Description should still own the alt:\n{}",
      imgs[0]
    );

    // Everything referenced still resolves, and every authored text is present.
    let ids: Vec<String> = html
      .match_indices("id=\"")
      .filter_map(|(i, _)| {
        let rest = &html[i + 4..];
        rest.find('"').map(|e| rest[..e].to_string())
      })
      .collect();
    for r in refs.split_whitespace() {
      assert!(
        ids.iter().any(|id| id == r),
        "aria-describedby references '{r}', which no element defines:\n{html}",
      );
    }
    for text in ["FIRSTLONG", "SECONDSHORT", "SECONDLONG"] {
      assert!(html.contains(text), "{text} was lost:\n{html}");
    }
  }

  /// A `\Description` in a TABLE float is the author doing exactly what acmart
  /// asks — a table has no image, so the table itself is where the description
  /// belongs. That must be reported as INFO, never as a warning.
  ///
  /// It was a warning until 2026-07-30, and it dominated the regressions in that
  /// day's `sandbox-arxiv-2605` rerun: 27 of 45 sampled documents that fell from
  /// `no_problem` to `warning` carried this one message, purely for having a
  /// described table.
  const TEX_TABLE: &str = "\\documentclass[acmsmall]{acmart}\n\
    \\usepackage{graphicx}\n\
    \\begin{document}\n\
    \\begin{table}\n\
    \\caption{A table with no image in it}\n\
    \\begin{tabular}{ll}a & b\\\\c & d\\end{tabular}\n\
    \\Description[TABLESHORT]{TABLELONG description of the tabular data}\n\
    \\end{table}\n\
    \\end{document}\n";

  #[test]
  fn a_described_table_is_reported_as_info_not_a_warning() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX_TABLE).expect("write t.tex");

    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.html",
        "--format",
        "html5",
        "--nocomments",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
    assert!(!html.is_empty(), "no HTML produced:\n{stderr}");

    // The description must still be attached — this is a severity change, not a
    // behaviour change.
    assert!(
      html.contains("aria-describedby="),
      "the table must still carry the description:\n{html}",
    );
    assert!(
      html.contains("TABLELONG"),
      "the long description text must survive into the HTML:\n{html}",
    );

    // …and the paper must stay clean. A described table is not an anomaly.
    assert!(
      !stderr.contains("Warning:unexpected:\\Description"),
      "a described TABLE must not warn — it is the expected shape:\n{stderr}",
    );
    assert!(
      !stderr.contains("Error:"),
      "expected a clean conversion:\n{stderr}",
    );
  }
}

mod stex_raw_ltxml {
  //! Raw-loading a style package under `--includestyles` must (a) never read a
  //! Perl `.ltxml` binding as TeX, and (b) restore the `standalone → currfile →
  //! filehook` dependency chain so the package-file hooks exist.
  //!
  //! Origin: raw `stex.sty` (sTeX 3.x) under the ar5iv config. stex ships BOTH
  //! `stex.sty` (real TeX) and `stex.sty.ltxml` (a Perl LaTeXML binding) in TeX
  //! Live, and `stex.sty` uses `\AtEndOfPackageFile` (filehook, reached via
  //! standalone → currfile) and `\define@key` (xkeyval). Two bugs surfaced:
  //!   1. `find_file` returned `stex.sty.ltxml` (kpsewhich lists it) ahead of the
  //!      raw `stex.sty`, and the raw-loader tokenized the Perl source as TeX
  //!      (`$out =~ s/^\s+//;` → "Script ^…", `\DefMacroI`/`\stex@backend`
  //!      undefined). latexml-oxide can never read a `.ltxml`; binding availability
  //!      is decided by the dispatcher, not a `.ltxml` on disk.
  //!   2. the simplified `standalone` binding dropped standalone.sty's unconditional
  //!      `\RequirePackage{xkeyval}` / `\RequirePackage{currfile}` (→ filehook), so
  //!      `\AtEndOfPackageFile` / `\define@key` were undefined.

  use std::{path::Path, process::Command};

  use crate::common::strip_ansi;

  fn convert(work: &Path, doc: &str) -> String {
    std::fs::write(work.join("doc.tex"), doc).expect("write doc.tex");
    let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(["--includestyles", "--dest", "doc.xml", "doc.tex"])
      .current_dir(work)
      .output()
      .expect("spawn latexml_oxide");
    strip_ansi(&String::from_utf8_lossy(&out.stderr))
  }

  fn error_count(log: &str) -> usize {
    log
      .lines()
      .filter(|l| l.starts_with("Error:") || l.starts_with("Fatal:"))
      .count()
  }

  fn kpsewhich_has(name: &str) -> bool {
    Command::new("kpsewhich")
      .arg(name)
      .output()
      .map(|o| o.status.success() && !o.stdout.is_empty())
      .unwrap_or(false)
  }

  /// Self-contained (all bindings are compiled in — no TeX Live package needed):
  /// under `--includestyles`, `\usepackage{standalone}` must pull in the
  /// `xkeyval` + `currfile → filehook` chain so `\AtEndOfPackageFile` is defined.
  #[test]
  fn standalone_under_includestyles_provides_filehook_hooks() {
    let work = tempfile::tempdir().expect("tempdir");
    let log = convert(
      work.path(),
      "\\documentclass{article}\n\
       \\usepackage{standalone}\n\
       \\AtEndOfPackageFile{graphicx}{\\typeout{DEFERRED}}\n\
       \\begin{document}\nHello.\n\\end{document}\n",
    );
    assert!(
      !log.contains("AtEndOfPackageFile") && !log.contains("define@key"),
      "standalone under --includestyles must define the filehook/xkeyval hooks \
       (standalone → currfile → filehook, standalone → xkeyval); log:\n{log}"
    );
    assert_eq!(
      error_count(&log),
      0,
      "expected a clean conversion; log:\n{log}"
    );
  }

  /// The real witness: raw `stex.sty` must load — never its Perl `stex.sty.ltxml`
  /// — and convert cleanly. Skipped where TeX Live lacks stex.
  #[test]
  fn raw_stex_sty_loads_not_the_perl_ltxml() {
    if !kpsewhich_has("stex.sty") || !kpsewhich_has("stex.sty.ltxml") {
      eprintln!("stex.sty / stex.sty.ltxml not in TeX Live — skipping");
      return;
    }
    let work = tempfile::tempdir().expect("tempdir");
    let log = convert(
      work.path(),
      "\\documentclass{article}\n\\usepackage{stex}\n\
       \\begin{document}\nHello sTeX.\n\\end{document}\n",
    );
    // The Perl binding must never be read as TeX.
    assert!(
      !log.contains("stex.sty.ltxml"),
      "the Perl stex.sty.ltxml must never be read (latexml-oxide can't read .ltxml); log:\n{log}"
    );
    assert!(
      !log.contains("DefMacroI") && !log.contains("stex@backend"),
      "Perl-syntax-as-TeX errors present — the .ltxml was misread; log:\n{log}"
    );
    // And the raw load (stex → standalone → currfile → filehook, xkeyval) is clean.
    assert_eq!(
      error_count(&log),
      0,
      "raw stex.sty must convert with 0 errors; log:\n{log}"
    );
  }
}

mod texinputs_usepackage {
  //! GitHub #345: `\usepackage{X}` must find a runtime `X.sty.rhai` binding placed
  //! in a texmf tree on `$TEXINPUTS` — the same way `\input{file}` already resolves
  //! files there — without needing an explicit `--path`.
  //!
  //! The `.rhai` discovery (`converter.rs::rhai_dispatch`) searched the local
  //! search paths ONLY (`--path` + the source dir) and skipped kpsewhich, which is
  //! what honours `$TEXINPUTS`. kpsewhich locates a `.sty.rhai` on TEXINPUTS just
  //! fine (the extension is irrelevant to a `//` recursive search), so consulting
  //! it closes the `\input`-works-but-`\usepackage`-doesn't asymmetry the reporter
  //! hit.
  //!
  //! The TeX-tree probe is the **last** tier of the binding chain, not the first:
  //! a `.rhai` beside your document is an *override*, one that merely sits in a
  //! texmf tree only *fills a gap*. The two `..._shadow_...` tests below pin both
  //! halves of that split — see `converter.rs::install_binding_dispatch`.

  use std::{path::Path, process::Command};

  use crate::common::strip_ansi;

  /// Deliberately not a real CTAN package name: the fixture must be absent from
  /// every host texmf tree, or the "no binding" leg would resolve a real `.sty`.
  const PKG: &str = "lxonowrap";

  /// Marker text a loaded `.rhai` emits, so "did this binding run?" is a single
  /// unambiguous substring rather than an inference from the package name.
  fn binding(marker: &str) -> String {
    format!("DefEnvironment(\"{{{PKG}}}\", \"<ltx:block class='{marker}'>#body</ltx:block>\");\n")
  }

  const DOC: &str = "\\documentclass[12pt]{book}\n\
                     \\usepackage{lxonowrap}\n\
                     \\begin{document}\n\
                     \\begin{lxonowrap}wrapped text\\end{lxonowrap}\n\
                     \\end{document}\n";

  /// Convert `index.tex` in `dir`, returning (ANSI-free log, output HTML).
  fn convert(dir: &Path, texinputs: Option<&str>) -> (String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"));
    cmd
      .args(["--dest", "index.html", "index.tex"])
      .current_dir(dir);
    match texinputs {
      Some(paths) => cmd.env("TEXINPUTS", paths),
      // Set it to just `.` rather than leaving it unset: an ambient `$TEXINPUTS`
      // from the developer's shell must not be what decides the negative test.
      None => cmd.env("TEXINPUTS", "."),
    };
    let out = cmd.output().expect("spawn latexml_oxide");
    let html = std::fs::read_to_string(dir.join("index.html")).unwrap_or_default();
    (strip_ansi(&String::from_utf8_lossy(&out.stderr)), html)
  }

  /// A texmf tree holding `<name>` with `content`, plus a `doc/` dir holding
  /// `index.tex`. Returns (tempdir, doc path, `$TEXINPUTS` value reaching it).
  fn fixture(
    name: &str,
    content: &str,
    tex: &str,
  ) -> (tempfile::TempDir, std::path::PathBuf, String) {
    let work = tempfile::tempdir().expect("tempdir");
    // Buried a few levels deep, so only the recursive `//` search reaches it.
    let styles = work.path().join("texmf/tex/latex/mystyles");
    std::fs::create_dir_all(&styles).unwrap();
    std::fs::write(styles.join(name), content).unwrap();
    let doc = work.path().join("doc");
    std::fs::create_dir_all(&doc).unwrap();
    std::fs::write(doc.join("index.tex"), tex).unwrap();
    // kpathsea's path-list separator is `;` on Windows — `:` collides with the
    // drive letter (`C:\...`) — and `:` everywhere else. The `//` recursive suffix
    // is the same on both.
    let sep = if cfg!(windows) { ";" } else { ":" };
    let texinputs = format!(".{sep}{}//{sep}", work.path().join("texmf").display());
    (work, doc, texinputs)
  }

  /// A `<pkg>.sty.rhai` under a `$TEXINPUTS` texmf tree (recursive `//`, no
  /// `--path`) must be discovered and loaded by `\usepackage{<pkg>}`.
  #[cfg_attr(
    not(building_with_texlive),
    ignore = "requires a TeX Live installation (kpsewhich resolves $TEXINPUTS)"
  )]
  #[test]
  fn usepackage_finds_rhai_binding_on_texinputs() {
    let (_work, doc, texinputs) = fixture(&format!("{PKG}.sty.rhai"), &binding("lxo-loaded"), DOC);
    let (log, html) = convert(&doc, Some(&texinputs));

    assert!(
      !log.contains(&format!("missing_file:{PKG}")),
      "\\usepackage{{{PKG}}} must resolve {PKG}.sty.rhai via TEXINPUTS (no --path); log:\n{log}"
    );
    // The binding actually RAN: its constructor emitted the block. Asserting the
    // marker class (not the package name) is what makes this discriminating —
    // the undefined-environment fallback also prints the package name, into an
    // `ltx_ERROR` span, which is exactly what `..._is_undefined_without_texinputs`
    // pins below.
    assert!(
      html.contains("lxo-loaded") && html.contains("wrapped text"),
      "the {PKG} environment (from the TEXINPUTS .rhai) should render its block; html:\n{html}"
    );
    assert!(
      !html.contains("ltx_ERROR"),
      "a loaded binding leaves no error node in the document; html:\n{html}"
    );
  }

  /// The negative control for the assertions above: with the tree off
  /// `$TEXINPUTS`, the very same document must FAIL, and must fail without
  /// producing the marker. Without this, "html contains the package name" would
  /// pass on a broken binary too — the undefined-environment recovery emits
  /// `<span class="ltx_ERROR undefined">{lxonowrap}</span>`.
  #[test]
  fn package_is_undefined_without_texinputs() {
    let (_work, doc, _texinputs) = fixture(&format!("{PKG}.sty.rhai"), &binding("lxo-loaded"), DOC);
    let (log, html) = convert(&doc, None);

    assert!(
      log.contains(&format!("missing_file:{PKG}")),
      "with no TEXINPUTS there is nothing to find; log:\n{log}"
    );
    assert!(
      !html.contains("lxo-loaded") && html.contains("ltx_ERROR"),
      "the failing conversion must not produce the marker; html:\n{html}"
    );
  }

  /// Authority half of the tier split: a `.rhai` that merely sits on the TeX tree
  /// FILLS A GAP — it must not displace a compiled binding of the same name.
  /// Before the TeX-tree probe was demoted to the last tier, a stray
  /// `amsmath.sty.rhai` anywhere on `$TEXINPUTS` replaced the whole compiled
  /// `amsmath` binding, and `\begin{align}` became an undefined `\align`.
  #[cfg_attr(
    not(building_with_texlive),
    ignore = "requires a TeX Live installation (kpsewhich resolves $TEXINPUTS)"
  )]
  #[test]
  fn texmf_rhai_does_not_shadow_a_compiled_binding() {
    let (_work, doc, texinputs) = fixture(
      "amsmath.sty.rhai",
      "DefMacro(\"\\\\lxoprobe\", \"TEXMF-RHAI-WON\");\n",
      "\\documentclass{article}\n\
       \\usepackage{amsmath}\n\
       \\begin{document}\n\
       \\begin{align}a&=b\\end{align}\n\
       \\end{document}\n",
    );
    let (log, html) = convert(&doc, Some(&texinputs));

    assert!(
      !log.contains("undefined:\\align") && !log.contains("unexpected:&"),
      "the compiled amsmath binding must still win over a texmf .rhai; log:\n{log}"
    );
    assert!(
      !html.contains("TEXMF-RHAI-WON"),
      "the texmf .rhai must not have been loaded at all; html:\n{html}"
    );
  }

  /// Cost/authority half kept intact: a `.rhai` in the document's own directory
  /// still overrides the compiled binding of the same name (the documented
  /// tier-1 behaviour — `script_bindings_plan.md` §7).
  #[test]
  fn local_rhai_still_overrides_a_compiled_binding() {
    let work = tempfile::tempdir().expect("tempdir");
    let doc = work.path().join("doc");
    std::fs::create_dir_all(&doc).unwrap();
    std::fs::write(
      doc.join("amsmath.sty.rhai"),
      "DefMacro(\"\\\\lxoprobe\", \"LOCAL-RHAI-WON\");\n",
    )
    .unwrap();
    std::fs::write(
      doc.join("index.tex"),
      "\\documentclass{article}\n\
       \\usepackage{amsmath}\n\
       \\begin{document}\n\
       \\lxoprobe\n\
       \\end{document}\n",
    )
    .unwrap();
    let (_log, html) = convert(&doc, None);

    assert!(
      html.contains("LOCAL-RHAI-WON"),
      "a .rhai beside the document overrides the compiled binding; html:\n{html}"
    );
  }
}

/// A no-dump (degraded raw-load) conversion of an expl3-using document must
/// still succeed. Witness issue #651: a bare `\usepackage{fvextra}` reported
/// "Conversion failed: 1 fatal error" under DEGRADED mode even though the output
/// (`<p>text</p>`) was correct. The failure was a benign expl3-code.tex
/// group-end codepoint cascade (L33074-33180) that the dump avoids and Perl's
/// own raw-load never produces, leaking a fatal into the conversion status.
/// `expl3_sty.rs` now snapshots the error report across the degraded raw expl3
/// load (gated on `raw_load_will_run`, so dump mode — which short-circuits the
/// re-load — is untouched). The fixture uses `\usepackage{expl3}` directly (the
/// l3kernel is always present, unlike a trimmed-TL `fvextra`), which triggers
/// the same raw-load path.
///
/// Linux-only: the degraded raw-load re-runs the whole ~33k-line expl3-code.tex,
/// which under the unoptimized `ci`/`dev` test profile takes ~2 min (measured
/// 124 s local `dev`); the behavior is OS-independent, so guarding one platform
/// keeps the ~2-min cost off all four macOS shards. It also needs an explicit
/// `--timeout` far above the 60 s CLI default (which is calibrated for the fast
/// dump path): with the default, the legitimate slow bootstrap is killed as a
/// `Fatal:timeout:wallclock` before the load can finish. Release-optimized
/// binaries — what real degraded users run — complete the same load well under
/// 60 s, so this large timeout is purely a slow-test-build accommodation.
#[cfg(target_os = "linux")]
mod expl3_degraded_no_dump {
  use std::{path::Path, process::Command};

  const EXPL3_TEX: &str = r"\documentclass{article}
\usepackage{expl3}
\begin{document}
degraded-body-text
\end{document}
";

  fn convert_nodump() -> (bool, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("e.tex"), EXPL3_TEX).expect("write e.tex");
    let output = Command::new(bin)
      // `--timeout 900`: the unoptimized test-build raw expl3 load runs ~2 min,
      // far over the 60 s CLI default; 900 s stays well under nextest's 20 min
      // terminate-after so a genuine hang still surfaces.
      .args([
        "e.tex",
        "--dest",
        "e.xml",
        "--nocomments",
        "--timeout",
        "900",
      ])
      .env("LATEXML_NODUMP", "1")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("e.xml")).unwrap_or_default();
    (
      output.status.success(),
      format!("{stderr}\n<<<XML>>>\n{xml}"),
    )
  }

  #[test]
  fn expl3_converts_healthily_without_a_dump() {
    let (ok, out) = convert_nodump();
    assert!(
      ok,
      "degraded no-dump expl3 conversion exited non-zero:\n{out}"
    );
    assert!(
      !out.contains("fatal error"),
      "degraded no-dump conversion leaked a fatal (benign expl3 cascade):\n{out}"
    );
    assert!(
      out.contains("degraded-body-text"),
      "degraded no-dump conversion dropped the body:\n{out}"
    );
  }
}

/// Regression: minted's `\newmintinline`/`\newminted`/`\newmint` take an optional
/// `[env-name]` before the two mandatory `{language}{options}` args (real
/// minted.sty: `\newcommand{..}[3][]`). The Rust binding declared them with a
/// two-mandatory `#1#2` signature, so `\newminted[leancode]{lean4}{...}` captured
/// `#1 = "["` and ran `\expandafter\def\csname [\endcsname{...}` — and `\csname
/// [\endcsname` IS the control sequence `\[`, silently redefining display-math open
/// to `\begin{lstlisting}`. Every later `\[` then opened a listing that ran to
/// end-of-file, swallowing the body and `\bibliography` with no `Error:` — a silent
/// bibliography loss. Witness `arXiv:2606.05629` (issue #520). The reporter's first
/// hypothesis (the comment-wrapped `leancode` env runs away) is not the cause: that
/// block sits inside `\begin{comment}` and is discarded; the runaway starts at the
/// first `\[` far earlier.
mod minted_newminted_optional_env_name {
  use crate::cluster::convert_to_xml_contrib_clean;

  #[test]
  fn newminted_optional_name_does_not_clobber_display_math() {
    let xml =
      convert_to_xml_contrib_clean("tests/cluster_regressions/minted_newminted_optarg_comment.tex");
    // The runaway listing is the whole bug: with `\[` clobbered, a single
    // `<listing>` swallows everything after the first display equation. The
    // `leancode` block is inside `\begin{comment}` (discarded) and `\[` is math, so
    // a healthy conversion emits NO listing at all.
    assert!(
      !xml.contains("listingline"),
      "a runaway listing swallowed the document (\\[ clobbered by \\newminted):\n{xml}"
    );
    // Content on BOTH sides of the display equation must survive into the flow.
    assert!(
      xml.contains("Body text after the display equation"),
      "body after the display equation was swallowed:\n{xml}"
    );
    assert!(
      xml.contains("Tail paragraph immediately before the bibliography"),
      "tail before the bibliography was swallowed:\n{xml}"
    );
    // `\bibliography` executed (the element exists at the core stage) rather than
    // being consumed as listing text.
    assert!(
      xml.contains("<bibliography "),
      "\\bibliography never executed — swallowed by the runaway listing:\n{xml}"
    );
  }

  /// The title's literal claim: a `\newminted`-created env used the normal way
  /// (outside a comment) must close at its own `\end{name}` and NOT read on to the
  /// literal `\end{lstlisting}`. The created env now delegates to
  /// `\lstnewenvironment{name}`, whose verbatim reader stops at `\end{name}`.
  #[test]
  fn newminted_env_closes_at_its_own_end_marker() {
    let xml =
      convert_to_xml_contrib_clean("tests/cluster_regressions/minted_newminted_direct_env.tex");
    // The env body IS captured as a listing (the code is tokenized into
    // `<text class="ltx_lst_*">` spans, so the raw string is not contiguous — key
    // off the listing element instead).
    assert!(
      xml.contains("ltx_lstlisting"),
      "the \\newminted env body was not captured as a listing:\n{xml}"
    );
    // …and it closes at `\end{leancode}`, so text after it stays in the document
    // flow instead of being swallowed to EOF.
    assert!(
      xml.contains("After the code listing"),
      "content after the \\newminted env was swallowed (ran past \\end{{name}}):\n{xml}"
    );
  }
}

/// minted binding quality (follow-up to #520): the inline `\mintinline{lang}{code}`
/// brace form must render without erroring or swallowing following content, and
/// `\begin{minted}{language}` must activate listings' syntax highlighting.
mod minted_inline_and_highlighting {
  use crate::cluster::convert_to_xml_contrib_clean;

  /// `\mintinline{python}{lambda x: x + 1}` used to map to `\verb`, whose
  /// delimiter is the first char `{` — it ran past the closing `}`, emitted two
  /// unbalanced-group errors, and broke the `\begin{minted}` block that followed.
  /// Routing to `\lstinline` (which accepts braces) keeps it inline and clean.
  #[test]
  fn mintinline_brace_form_does_not_error_or_swallow() {
    // The strict 0-error gate is itself the primary canary: the old `\verb`
    // mapping raised `unexpected:}` / `unexpected:\endgroup` here.
    let xml =
      convert_to_xml_contrib_clean("tests/cluster_regressions/minted_inline_and_highlighting.tex");
    // Text on both sides of the inline snippet survives (the brace form did not
    // run away), and the following block still renders.
    assert!(
      xml.contains("in a sentence"),
      "text after the inline \\mintinline was swallowed:\n{xml}"
    );
    assert!(
      xml.contains("Tail paragraph after the block"),
      "the \\begin{{minted}} block or its trailing text was swallowed:\n{xml}"
    );
    assert!(
      xml.contains("ltx_lstlisting"),
      "the \\begin{{minted}} block did not render as a listing:\n{xml}"
    );
  }

  /// `\begin{minted}{python}` now feeds the language to `\lstset`, so listings
  /// tags `def`/`if`/`return` as keywords (was plain identifiers — no highlighting).
  #[test]
  fn minted_language_activates_keyword_highlighting() {
    let xml =
      convert_to_xml_contrib_clean("tests/cluster_regressions/minted_inline_and_highlighting.tex");
    assert!(
      xml.contains("ltx_lst_keyword"),
      "minted block produced no highlighted keywords (language not activated):\n{xml}"
    );
  }
}

mod overpic_renders_graphic_and_overlays {
  //! overpic: render the `\includegraphics` background + `\put` overlays as a
  //! populated `<ltx:picture>`. The prior binding faithfully mirrored Perl
  //! (empty `<ltx:picture tex=.../>` for the unwired LaTeX-image renderer), so
  //! it emitted NO graphic and dropped `#body` — 37 arXiv papers reported the
  //! missing overpic figure. See `overpic_sty.rs` (the surpass-perl divergence).
  use crate::cluster::convert_to_xml;

  #[test]
  fn overpic_emits_populated_sized_picture_with_graphic_and_overlays() {
    let xml = convert_to_xml("tests/cluster_regressions/overpic_render.tex");
    // Emitted and SIZED (the empty binding produced a `<ltx:picture/>` with no
    // `unitlength`).
    assert!(xml.contains("<picture"), "no picture element:\n{xml}");
    assert!(
      xml.contains("unitlength="),
      "picture not sized (no unitlength):\n{xml}"
    );
    // THE FIX: the `\includegraphics` background renders as a nested graphic
    // (the empty binding emitted no graphic at all). Holds whether or not
    // `example-image`'s file resolves — the element is a core-stage product.
    assert!(
      xml.contains(r#"graphic="example-image""#),
      "overpic dropped the background graphic:\n{xml}"
    );
    // The body `\put` overlays land as picture content (the empty binding
    // dropped `#body`).
    assert!(xml.contains("OVLABELA"), "overlay A was dropped:\n{xml}");
    assert!(xml.contains("OVLABELB"), "overlay B was dropped:\n{xml}");
  }

  /// A missing/unmeasurable image with no size option must NOT raise
  /// `Illegal \divide by 0` (common on arXiv, where submissions omit referenced
  /// images). `convert_to_xml` asserts zero `Error:` markers; the pre-guard
  /// binding raised `Error:misdefined:0`.
  #[test]
  fn overpic_missing_natural_size_image_does_not_divide_by_zero() {
    let xml = convert_to_xml("tests/cluster_regressions/overpic_missing_image.tex");
    assert!(
      xml.contains("<picture"),
      "no picture for missing image:\n{xml}"
    );
    assert!(
      xml.contains("OVLABELC"),
      "overlay dropped for missing image:\n{xml}"
    );
  }
}

mod xkeyval_internals {
  //! Packages built on xkeyval clone `\setkeys`' front-end out of xkeyval.tex
  //! and drive our key machinery through the raw internals: chessboard.sty
  //! L98-107 (`\XKV@testopta{\XKV@testoptc\board@XKVsetsinglekeys}` …
  //! `\XKV@s@tkeys`), xkeymask.sty (`\XKV@setkeys`, `\XKV@tempc`), xskak.
  //! Sweep-12 cluster: chessboard ×3, xskak ×2, xkeymask — hundreds of
  //! cascaded `\csname` errors each. The binding carries the scaffolding
  //! verbatim from xkeyval.tex/xkvutils.tex, with `\XKV@s@tkeys` as a thin
  //! shim onto the Rust `\setkeys` path.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{xkeyval}\n\
    \\makeatletter\n\
    \\define@key[UF]{fam}{mykey}{\\def\\myresult{got:#1}}\n\
    \\def\\my@do{\\XKV@testopta{\\XKV@testoptc\\my@XKVsetkeys}}\n\
    \\def\\my@XKVsetkeys[#1]#2{%\n\
      \\XKV@checksanitizea{#2}\\XKV@resb\n\
      \\let\\XKV@naa\\@empty\n\
      \\XKV@for@o\\XKV@resb\\XKV@tempa{%\n\
        \\expandafter\\XKV@g@tkeyname\\XKV@tempa=\\@nil\\XKV@tempa\n\
        \\XKV@addtolist@x\\XKV@naa\\XKV@tempa}%\n\
      \\expandafter\\XKV@s@tkeys\\expandafter{\\XKV@resb}{#1}}\n\
    \\begin{document}\n\
    \\my@do*[UF]{ fam }{ mykey = hello }\\myresult\n\
    \\setkeys[UF]{fam}{mykey=direct}\\myresult\n\
    \\end{document}\n";

  /// The chessboard-shape front-end clone must parse `*`/`[prefix]{fams}`
  /// through the raw `\XKV@testopta/c` scaffolding and land in the Rust
  /// `\setkeys` via the `\XKV@s@tkeys` shim, with keyval.tex's
  /// `\KV@@sp@def` space-trimming intact.
  #[test]
  fn setkeys_frontend_clone_reaches_rust_path() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "XKV internals must digest cleanly:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("got:hello"),
      "front-end clone did not reach the key code (spaces must be trimmed):\n{xml}"
    );
    assert!(
      xml.contains("got:direct"),
      "plain \\setkeys regression:\n{xml}"
    );
  }

  const PTR_TEX: &str = "\\documentclass{article}\n\
    \\usepackage{xkeyval}\n\
    \\makeatletter\n\
    \\define@key[UF]{fam}{ka}{\\def\\resa{A:#1}}\n\
    \\define@key[UF]{fam}{kb}{\\def\\resb{B:#1}}\n\
    \\define@cmdkey[UF]{fam}{kc}{\\def\\resc{C:#1}}\n\
    \\def\\xkv@tokdefault{tokdef}\n\
    \\define@key[UF]{fam}{kd}[\\xkv@tokdefault]{\\def\\resd{D:#1}}\n\
    \\savekeys[UF]{fam}{\\global{ka}}\n\
    \\begin{document}\n\
    \\setkeys[UF]{fam}{ka=hello}\n\
    \\setkeys[UF]{fam}{kb=\\usevalue{ka}}\n\
    \\setkeys[UF]{fam}{kc=cval}\n\
    \\setkeys[UF]{fam}{kd}\n\
    \\makeatletter\n\
    [\\resa][\\resb][\\resc][cmd:\\cmdUF@fam@kc]\n\
    [direct:\\csname XKV@UF@fam@ka@value\\endcsname][\\resd]\n\
    \\makeatother\n\
    \\end{document}\n";

  /// Three engine contracts on one doc: (1) the pointer system —
  /// `\savekeys` + `\usevalue` + raw `\XKV@<header><key>@value` readback
  /// (chessboard.sty L1059/L1221); (2) `\define@cmdkey` code receives the
  /// BARE value as `#1` and defines `\cmd<header>` (KNOWN_PERL_ERRORS #80 —
  /// Perl passes `#<value>`, leaking a PARAM token to the stomach); (3) a
  /// key DEFAULT holding an internal `\xxx@yyy` name survives as one TOKEN
  /// (xskak-keys.sty L25 `[\xskak@val@defaultid]` — the string round-trip
  /// re-tokenized it under the standard cattable, splitting the CS and
  /// cascading ~1000 csname errors per xskak/chessboard manual).
  #[test]
  fn pointer_system_cmdkey_and_token_defaults() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), PTR_TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "pointer/cmdkey/default paths must digest cleanly:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    for needle in [
      "[A:hello]",
      "[B:hello]",
      "[C:cval]",
      "[cmd:cval]",
      "[direct:hello]",
      "[D:tokdef]",
    ] {
      assert!(xml.contains(needle), "missing {needle} in:\n{xml}");
    }
  }
}

mod kernel_language_and_part_contracts {
  //! Two kernel contracts raw class files depend on: (1) babel assigns
  //! `\language=\l@<main>` DURING package load (babel.sty L1136-1142 →
  //! L828 `\bbl@patterns`), so a preamble `\iflanguage{english}{..}{..}`
  //! under `[italian]` takes the FALSE branch — we never set `\language`,
  //! so every non-English doc mis-branched (toptesi topfront manuals: 34
  //! undefined-CS errors from a block real LaTeX skips). (2) report/book
  //! define `\@endpart` (report.cls L318-327), invoked by `\@part`/`\@spart`
  //! and directly by raw classes (toptesi.sty L448).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{report}\n\
    \\usepackage[italian]{babel}\n\
    \\iflanguage{english}{\\def\\langprobe{EN}}{\\def\\langprobe{IT}}\n\
    \\begin{document}\n\
    [lang:\\langprobe]\n\
    \\part{Prima Parte}\n\
    testo\n\
    \\end{document}\n";

  #[test]
  fn babel_language_register_and_endpart() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "kernel contracts must digest cleanly:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("[lang:IT]"),
      "\\iflanguage must take the non-English branch under [italian]:\n{xml}"
    );
    assert!(
      xml.contains("Prima Parte"),
      "\\part content lost (\\@endpart contract):\n{xml}"
    );
  }
}

mod input_routing_and_bbx {
  //! Batch-7 contracts: (1) a document-position `\input{<name>.sty}` with no
  //! binding reads the raw file as CONTENT under the current catcodes (real
  //! TeX semantics) — the definitions mouth's forced `@`=letter corrupted
  //! doc.sty's `\CharacterTable` self-check on every `\DocInput` re-read
  //! (frankenstein bundle ×10 + pkgloader); (2) biblatex's
  //! `style=`/`bibstyle=`/`citestyle=` options load the raw `.bbx`/`.cbx`
  //! style files (biblatex.sty L2256/L11428), whose `\newtoggle`s etc. were
  //! undefined corpus-wide (windycity, biblatex-ext/-fiwi/-sbl).

  use std::{path::Path, process::Command};

  #[test]
  fn document_body_sty_input_is_content_catcodes() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(
      workdir.path().join("vguardcat.sty"),
      "\\edef\\guardcat{\\the\\catcode`\\@}\n",
    )
    .expect("write vguardcat.sty");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{article}\n\\begin{document}\n\
       \\input{vguardcat.sty}\n[cat:\\guardcat]\n\\end{document}\n",
    )
    .expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("[cat:12]"),
      "document-body \\input{{x.sty}} must read at current catcodes (@=12), got:\n{xml}"
    );
  }

  #[test]
  fn biblatex_style_option_loads_bbx() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(
      workdir.path().join("lxguardstyle.bbx"),
      "\\newtoggle{lxguardtoggle}\\toggletrue{lxguardtoggle}\n\
       \\DeclareBibliographyOption[boolean]{lxguardopt}[true]{}\n",
    )
    .expect("write lxguardstyle.bbx");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{article}\n\
       \\usepackage[style=lxguardstyle]{biblatex}\n\
       \\begin{document}\n\
       \\iftoggle{lxguardtoggle}{[BBX-LOADED]}{[BBX-FALSE]}\n\
       \\end{document}\n",
    )
    .expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "style-file load must digest cleanly:\n{stderr}"
    );
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
    assert!(
      xml.contains("[BBX-LOADED]"),
      ".bbx toggle not allocated — style file not loaded:\n{xml}"
    );
  }
}

mod expl3_state_and_param_replay {
  //! Batch-12 engine contracts (misdefined:# cluster, agent-bisected):
  //! (1) a nested raw load must not leave the half-ExplSyntax state
  //! space=ignored while `_` is not a letter (l3backend under ctex/jlreq
  //! did; pgfkeys' space-delimited `\def\: {…}` idiom then broke corpus-
  //! wide); (2) `\ProvidesExplFile` turns expl3 syntax ON (expl3.sty
  //! L33-47) — the siunitx binding stubbed it to nothing (Perl-origin);
  //! (3) `\@ifnextchar` re-scans its branches as macro bodies, collapsing
  //! `##`→`#` (latex.ltx L1756-1760; adtreesdoc witness, Perl shares).

  use std::{path::Path, process::Command};

  fn convert(tex: &str) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--preload=[rawstyles,rawclasses]latexml.sty",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  #[test]
  fn ifnextchar_collapses_doubled_params() {
    let (stderr, xml) = convert(
      "\\documentclass{article}\n\\makeatletter\n\
       \\def\\outerm#1{\\@ifnextchar[{X#1Y}{X#1Y}}\n\\makeatother\n\
       \\begin{document}\n\\outerm{\\def\\inner##1{[##1]}}\\inner{A}\n\\end{document}\n",
    );
    assert!(
      !stderr.contains("misdefined:#"),
      "PARAM leaked through \\@ifnextchar branch replay:\n{stderr}"
    );
    assert!(
      xml.contains("XY[A]"),
      "\\inner must receive its argument after ## collapse:\n{xml}"
    );
  }

  #[test]
  fn provides_expl_file_turns_syntax_on() {
    let workdir_tex = "\\documentclass{article}\n\\usepackage{siunitx}\n\
      \\usepackage{numerica}\n\\begin{document}x\\end{document}\n";
    let (stderr, _) = convert(workdir_tex);
    assert!(
      !stderr.contains("Script _ can only appear in math mode"),
      "expl3 file read with LaTeX catcodes after siunitx (ProvidesExplFile stub):\n{stderr}"
    );
  }
}

mod alignment_ledger_expansion_pushback {
  //! Batch-25 engine contract: the alignment brace ledger follows tex.web's
  //! protocol — braces count when SCANNED (get_next §342/§357), pushback of
  //! previously-scanned tokens retracts (back_input §325), and expansion
  //! output enters WITHOUT adjustment (begin_token_list; `read_balanced`
  //! likewise no longer localizes the ledger — scan_toks §473-482 doesn't).
  //! Perl instead retracts EVERY pushback and localizes readBalanced, which
  //! drifts the ledger on expl3 brace-tricks (`\if_true: { \else: } \fi:`
  //! halves traveling through different reads) and makes a later cell-top
  //! `&` go stray — Perl LaTeXML shares this failure (verified 2026-09-01
  //! on this exact repro). Root of the l3doc `{function}` stray-`&` family
  //! (17+ bundles: every l3doc manual with a `{syntax}` block).

  use std::{path::Path, process::Command};

  fn convert(tex: &str) -> String {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let output = Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    stderr
  }

  #[test]
  fn tl_greplace_then_protected_amp_in_cell() {
    // The l3doc shape shrunk to its core: an l3tl replace (whose delimited
    // scanning pushes net-unbalanced fragments) followed by a protected
    // macro that emits the cell separator. SURPASS-PERL: Perl errors
    // "Stray alignment &" here.
    let stderr = convert(
      "\\documentclass{article}\n\\ExplSyntaxOn\n\\tl_new:N \\g_my_tl\n\
       \\cs_new_protected:Npn \\my_amp: { & }\n\
       \\cs_new_protected:Npn \\my_row:\n  {\n    \\tl_gset:Nn \\g_my_tl { a~b }\n\
       \\tl_greplace_all:Nnn \\g_my_tl { ~ } { x }\n    name \\my_amp: e \\\\\n  }\n\
       \\ExplSyntaxOff\n\\begin{document}\n\\begin{tabular}{lr}\n\
       \\ExplSyntaxOn \\my_row: \\ExplSyntaxOff\n\\end{tabular}\n\\end{document}\n",
    );
    assert!(
      !stderr.contains("Stray alignment"),
      "expl3 brace-trick drift resurfaced (ledger protocol regression):\n{stderr}"
    );
    assert!(
      !stderr.contains("Error:"),
      "greplace+protected-& cell should convert clean:\n{stderr}"
    );
  }

  #[test]
  fn ifstar_reemitted_brace_keeps_borders() {
    // The compensating half of the protocol: a closure-backed expandable
    // (`\@ifstar`-family) re-emits a token it READ; that token's brace
    // count must be retracted (tex.web §368 back_input around the one-step
    // expansion) or every `\foo{...}` behind an \@ifstar guard drifts the
    // ledger +1 and a later `&` misses the column-end check silently
    // (borders lost via handle_template never firing — cells.xml/graphrot
    // golden regressions caught this).
    let stderr = convert(
      "\\documentclass{article}\n\\newsavebox{\\foo}\n\
       \\def\\testrot#1{\\savebox{\\foo}{\\parbox{1in}{whales}}\\framebox{---\\usebox{\\foo}---}}\n\
       \\begin{document}\n\\begin{tabular}{|c|c|}\n\\hline\n\
       \\testrot{0} & \\testrot{1}\\\\\na & b \\\\\n\\hline\n\\end{tabular}\n\\end{document}\n",
    );
    assert!(
      !stderr.contains("Error:"),
      "savebox/framebox row should convert clean:\n{stderr}"
    );
  }
}

mod autoload_trigger_identity {
  //! Batch-27 contract: `\@ifundefined{X}` must treat X as DEFINED when the
  //! kernel dump gave it a real definition, even though X is also a
  //! def_autoload trigger whose package never loaded. The `:autoload` flag
  //! alone goes stale for `\ProvidesExplPackage`/`\ProvidesExplClass`/
  //! `\ExplSyntaxOn` (triggers pre-dump AND real latex.ltx macros); the
  //! reader now also requires the CS to still HOLD the trigger definition
  //! (Rc identity snapshot). Witness: updatemarks-nums.sty's single-branch
  //! `\@ifundefined{ProvidesExplPackage}{\RequirePackage{expl3}}` swallowed
  //! the following `\ProvidesExplPackage` as the phantom second branch —
  //! expl3 catcodes never enabled, 89-error cascade (21-doc `unexpected:_`
  //! cluster; updatemarks 101→2).

  use std::{path::Path, process::Command};

  #[test]
  fn ifundefined_sees_dump_definition_over_stale_trigger() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(
      workdir.path().join("trigid.sty"),
      "\\@ifundefined{ProvidesExplPackage}{\\RequirePackage{expl3}}\n\
       \\ProvidesExplPackage{trigid}{2024/02/19}{v0.1}{x}\n\
       \\tl_new:N \\l__trigid_tmpa_tl\n\\ExplSyntaxOff\n",
    )
    .expect("write sty");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{article}\n\\usepackage{trigid}\n\\begin{document}\nx\n\\end{document}\n",
    )
    .expect("write tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--preload=[rawstyles,rawclasses]latexml.sty",
      ])
      .env("TEXINPUTS", format!("{}:", workdir.path().display()))
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert!(output.status.success(), "binary exited: {stderr}");
    assert!(
      !stderr.contains("Error:"),
      "stale autoload trigger masked the dump's \\ProvidesExplPackage:\n{stderr}"
    );
  }
}

mod perfect_kernel_batch40_43 {
  //! Red/green guards for the perfect-kernel batches 40-43 root-cause fixes.
  //! Each test is the minimal reproduction distilled during triage; the
  //! doc-comment names the ORIGINAL corpus witness (TeX Live doc corpus,
  //! `bundle/doc`) whose larger conversion was vetted separately.
  use std::{path::Path, process::Command};

  /// Convert an inline snippet in a tempdir under the perfect-kernel preload;
  /// return (ANSI-stripped stderr, XML string).
  fn convert(tex: &str) -> (String, String) { convert_with_files(tex, &[]) }

  /// Like `convert`, with sibling files (`.cls`/`.sty` under test) written
  /// next to `t.tex` and reachable through `TEXINPUTS`.
  fn convert_with_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    for (name, body) in files {
      std::fs::write(workdir.path().join(name), body).expect("write sibling file");
    }
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--timeout=110",
        "--preload=[rawstyles,rawclasses]latexml.sty",
      ])
      .env("TEXINPUTS", format!("{}:", workdir.path().display()))
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  fn error_count(stderr: &str) -> usize {
    stderr.lines().filter(|l| l.starts_with("Error:")).count()
  }

  /// Batch 40: the xparse TCB listing trio takes a LEADING `[init-options]`
  /// optional (tcblistingscore.code.tex:329). RED: the three `{}` args
  /// grabbed `[`, `u`, `s`; the options body digested raw (`misdefined:#`
  /// storm, `\thetcbcounter` undefined). Witness: atableau/atableau
  /// (1001-cap → 0 together with the stub removal).
  #[test]
  fn newtcblisting_xparse_leading_optional() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[skins]{tcolorbox}
\tcbuselibrary{listings}
\newcounter{example}
\NewTCBListing[use counter=example, number within=section]{example}{ O{} s m }{ title={\thetcbcounter}, #1 }
\begin{document}
\begin{example}{tst}
verbatim_body^here
\end{example}
\end{document}
",
    );
    assert!(
      !stderr.contains("misdefined"),
      "leading [init-options] mis-grabbed again:\n{stderr}"
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The lexer marks identifiers up, so match the split body parts.
    assert!(
      xml.contains("verbatim_body") && xml.contains("here"),
      "listing body was not captured verbatim:\n{xml}"
    );
  }

  /// Batch 40: beamer's full `\newif` surface. An UNDEFINED `\if…` inside a
  /// skipped branch is invisible to the meaning-counting skipper (tex.web
  /// §366; Conditional.pm:117 — the skipper is correct, the definition was
  /// missing), so its `\fi` closed the outer frame early. RED: 2 errors
  /// (`unexpected:\else` + `unexpected:fi`). Witnesses:
  /// beamerthemecelestia/Celestia-demo-* (2 → 0 each, ×6 docs).
  #[test]
  fn beamer_newif_invisible_in_skipped_branch() {
    let (stderr, xml) = convert(
      r"\documentclass{beamer}
\makeatletter
\iffalse
  \ifbeamer@plainframe X\fi
\else
  \def\elsebranch{ran}
\fi
\makeatother
\begin{document}
\begin{frame}ok \elsebranch\end{frame}
\end{document}
",
    );
    assert!(
      !stderr.contains("unexpected:fi") && !stderr.contains("unexpected:\\else"),
      "orphan fi/else came back:\n{stderr}"
    );
    assert!(xml.contains("ran"), "else-branch did not execute:\n{xml}");
  }

  /// Batch 41: beamer's COMMAND form `\frame{content}` must route through
  /// the frame environment. RED: the env-installed bare `\frame` opened the
  /// `_noautoclose` subsection and never closed it — later `\section`s
  /// nested inside (malformed:ltx beamer-sectioning family, 185 errors /
  /// 40 docs). Witness: beamerauxtheme (16 → 0).
  #[test]
  fn beamer_frame_command_form_closes() {
    let (stderr, xml) = convert(
      r"\documentclass{beamer}
\begin{document}
\section{S}\frame{f}\section{T}
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(
      xml.matches("<section").count(),
      2,
      "a section was swallowed by a dangling frame subsection:\n{xml}"
    );
  }

  /// Batch 41: amsopn re-asserts the 34 log-like operators exactly like
  /// real amsopn.sty L56-89. RED: amsldoc.cls makes `\arg{1}` doc-markup;
  /// `$\arg$` then ate its closing `$` — 101-error cascade (Perl shares the
  /// omission; pdflatex is the oracle). Witnesses: amsmath/amsldoc
  /// (101 → 0), amsldoc-it/itamsldoc + amsldoc-vn/amsldoc-vi (101 → 0 with
  /// the `\@nobslash` binding below).
  #[test]
  fn amsopn_reasserts_clobbered_operators() {
    let (stderr, _xml) = convert(
      r"\documentclass{amsldoc}
\usepackage{amsmath}
\begin{document}
text $\arg$ text $\det_n$.
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// Batch 43: amsldoc/amsdtx `\@nobslash` resolved at expansion time. RED:
  /// the raw `\ifnum`#1=\bslchar` test rode inside `\index` arguments; the
  /// SanitizedVerbatim untex→retokenize roundtrip welded its catcode-12 `\`
  /// into fake CSes (`Expected a relational token … Got \bslchar` + empty
  /// index entries). Witnesses: amsldoc-it/itamsldoc, amsldoc-vn/amsldoc-vi
  /// (2 residual errors each → 0).
  #[test]
  fn amsldoc_nobslash_expansion_time() {
    let (stderr, xml) = convert(
      r"\documentclass{amsldoc}
\begin{document}
\cn{\|} and \cn{\bslash} in text.
\end{document}
",
    );
    assert!(
      !stderr.contains("relational"),
      "\\bslchar reached the relational reader again:\n{stderr}"
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // Index-entry content is vetted on the full itamsldoc witness (the class
    // only opens its index stream in the full-manual configuration).
    let _ = xml;
  }

  /// Batch 40: any stub replacing a raw .sty must register `\ver@<file>`
  /// WITH the real ` v.` pattern. RED: `\GetFileInfo{curve2e.sty}`'s
  /// delimited ` v.` scan ran away over an undefined `\ver@curve2e.sty`,
  /// poisoning the whole document (locator-less pushback; shortverb regions
  /// executed raw). Witness: curve2e/curve2e-manual (88 errors + fatal →
  /// 41 honest stub-gap errors, all Match/Pair gone).
  #[test]
  fn curve2e_stub_registers_ver() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{curve2e}
\makeatletter
\providecommand\GetFileInfo[1]{%
  \def\@tempb##1 v.##2 ##3\relax##4\relax{\def\filedate{##1}\def\fileversion{##2}}%
  \edef\@tempa{\csname ver@#1\endcsname}%
  \expandafter\@tempb\@tempa\relax? ? \relax\relax}
\makeatother
\begin{document}
\GetFileInfo{curve2e.sty}
version \fileversion\ works.
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// Batch 42: `use counter=`/`listing file=` honored by generated TCB
  /// listing envs, with catcode-robust `\lxlstbeginwritefile` injection
  /// (a doc-level definition site has `@` = OTHER — the raw
  /// `\lst@BeginAlsoWriteFile` name split there). RED: nothing was ever
  /// written; `\input{\jobname.1.listing}` was a missing file. Witness:
  /// incgraph/incgraph (`\inputlisting{\n}` reading 12 such files).
  #[test]
  fn tcb_listing_file_written_and_input_back() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newcounter{texexp}
\newtcblisting[use counter=texexp]{texexptitled}[2][]{listing file={\jobname.\thetcbcounter.listing}}
\begin{document}
\begin{texexptitled}{t}{l}
replayed\_marker
\end{texexptitled}
\input{t.1.listing}
\end{document}
",
    );
    assert!(
      !stderr.contains("missing_file"),
      "listing file was not written to the VFS:\n{stderr}"
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // Once as the verbatim listing, once replayed via \input.
    assert!(
      xml.matches("replayed").count() >= 2,
      "the written listing did not replay via \\input:\n{xml}"
    );
  }

  /// Batch 44: `\newtcbinputlisting` — the defined command INPUTS its
  /// `listing file=` (after #-substitution) as a listing and shares the
  /// referenced env's counter via `use counter from=`. RED: the command was
  /// undefined; incgraph's `\inputexamplelisting` cascaded
  /// (`\tcb@cnt@texexptitled` csname errors ×15). Witness:
  /// incgraph/incgraph.
  #[test]
  fn tcbinputlisting_inputs_file() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newcounter{texexp}
\newtcblisting[use counter=texexp]{texexptitled}[2][]{listing file={\jobname.\thetcbcounter.listing}}
\newtcbinputlisting[use counter from=texexptitled]{\inputexamplelisting}[3][]{listing file={#2}}
\begin{document}
\begin{texexptitled}{t}{l}
roundtrip\_marker
\end{texexptitled}
\inputexamplelisting{t.1.listing}{lbl}
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.matches("roundtrip").count() >= 2,
      "\\inputexamplelisting did not display the recorded listing:\n{xml}"
    );
  }

  /// Batch 42: pgfmath `array({e0,e1,…}, i)` — brace-list first argument
  /// parsed in place, 0-based select. RED: "Unimplemented pgfmath operator
  /// 'array'" (Perl silently no-ops). Witness: colorblind/colorblind_doc
  /// (17 → 0).
  #[test]
  fn pgfmath_array_selects() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\pgfmathparse{array({3,7,5},1)}\edef\picked{\pgfmathresult}
picked=[\picked]
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("picked=[7"),
      "array(...,1) did not select the second element:\n{xml}"
    );
  }

  /// Batch 43: xkeyval presets implemented (OXIDIZED_DESIGN #173). RED: the
  /// six preset front-ends were warn-stubs and `\setkeys` bypassed the
  /// preset hooks — key code that only runs from presets never ran
  /// (cntperchap's "section level … is unknown"). Witness:
  /// cntperchap/cntperchap_doc (6 → 3, residual is unrelated surface).
  #[test]
  fn xkeyval_presets_apply_on_setkeys() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key{cpskeys}{tracklevel}[section]{\gdef\@cps@@keymacro@@tracklevel{#1}}
\presetkeys{cpskeys}{tracklevel=section}{}
\setkeys{cpskeys}{}
\begin{document}
\makeatletter
\expandafter\ifx\csname @cps@@keymacro@@tracklevel\endcsname\relax
LEVEL-UNKNOWN\else DEFINED: \@cps@@keymacro@@tracklevel\fi
\makeatother
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("DEFINED: section"),
      "preset key code did not run on \\setkeys:\n{xml}"
    );
  }

  /// Batch 43: Perl-faithful balanced-pair color-spec trim. RED: the old
  /// `trim_matches('{','}')` stripped braces from either end independently,
  /// so a name whose T1-mangled form ends in `}` (`é` →
  /// `…\lx@applyaccent…{e}`) lost its tail at LOOKUP while DEFINE stored it
  /// intact — "Can't find color named 'xFuchsiaFonc…'". Witness:
  /// couleurs-fr/couleurs-fr-doc (1 → 0).
  #[test]
  fn color_name_accent_symmetric_keys() {
    let (stderr, xml) = convert(
      "\\documentclass{article}
\\usepackage[T1]{fontenc}
\\usepackage{xcolor}
\\definecolor{caf\u{e9}}{HTML}{112233}
\\begin{document}
\\textcolor{caf\u{e9}}{x} \\textcolor[rgb]{0.5,0.5,0.5}{y}
\\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("#112233"),
      "accented color name failed to round-trip define→lookup:\n{xml}"
    );
    assert!(xml.contains("#808080"), "plain rgb spec regressed:\n{xml}");
  }

  /// Batch 43: `\index` sort keys brace-protected through the
  /// `\@indexphrase[]` re-parse (OXIDIZED_DESIGN #174, KPE #83 sibling).
  /// RED: a `]` inside the sort key truncated the optional-arg re-parse —
  /// key attribute cut at `gradetable[v`, display phrase spilled as illegal
  /// indexmark children. Witness: exam/examdoc (39-error family → 0),
  /// pgfornament docs.
  #[test]
  fn index_sortas_bracket_protected() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{makeidx}\makeindex
\newcommand{\indc}[1]{\index{#1@\texttt{\char`\\#1}}}
\begin{document}
x\indc{gradetable[v]} y\index{plain}\index{sorted@\textbf{shown}}
\printindex
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("key=\"gradetable[v]\""),
      "sort key with ] was truncated again:\n{xml}"
    );
    assert!(
      xml.contains("key=\"sorted\"") && xml.contains("key=\"plain\""),
      "plain sort keys must be byte-unchanged:\n{xml}"
    );
  }

  /// Batch 40: `\pscircle`'s coordinate pair is OPTIONAL (Perl
  /// pstricks_support ZeroPSCoord = ReadPSCoord || ZeroPair). RED:
  /// `Error:expected:Pair` on dsptricks.sty L535
  /// `\pscircle[…]{\PZCROC\dspUnitX}`. Witness: dsptricks/dspTricksManual.
  #[test]
  fn pscircle_pair_optional() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{pstricks}
\begin{document}
\begin{pspicture}(2,2)
\pscircle[linewidth=1pt]{0.5}
\pscircle(1,1){0.3}
\end{pspicture}
\end{document}
",
    );
    assert!(
      !stderr.contains("expected:Pair"),
      "pair-less \\pscircle regressed:\n{stderr}"
    );
  }

  /// Batch 45: `\DeclareMathOperator`'s text is expanded before the
  /// String round-trip into `def_math`. RED: numerica.sty:50-51 declares
  /// `\DeclareMathOperator{\asinh}{\cs_to_str:N \asinh}` under expl3
  /// catcodes; the stringified body re-tokenized as `\cs_to_str` `_` `:N`
  /// at every use → 100 malformed:ltx + Fatal Stomach:Recursion. Witness:
  /// numerica/numerica (manual line 148).
  #[test]
  fn declaremathoperator_expands_expl3_name() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{amsmath}
\ExplSyntaxOn
\DeclareMathOperator{\asinh}{\cs_to_str:N \asinh}
\ExplSyntaxOff
\begin{document}
$\asinh$
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<XMTok role="OPFUNCTION" scriptpos="post">asinh</XMTok>"#),
      "expl3-named operator did not resolve to plain letters:\n{xml}"
    );
  }

  /// Batch 45: `\PassOptionsToPackage`/`\PassOptionsToClass` store one
  /// `Stored::String` per option (Perl Package.pm:2435 spreads). RED: the
  /// whole list landed as ONE nested element, which the `\opt@<file>`
  /// rebuild skips, so options routed through the primitives read back
  /// EMPTY — a kvoptions class forwarding `\CurrentOption` to its own .sty
  /// after `\LoadClass[12pt]` (which clobbers `\@classoptionslist`) never
  /// saw `scheme`. Witness: brandeis-problemset/example (87 → tabu residual).
  #[test]
  fn passoptions_spreads_options() {
    let (stderr, xml) = convert_with_files(
      r"\documentclass[scheme]{pod}
\begin{document}\begin{scheme}hi\end{scheme}\end{document}
",
      &[
        (
          "pod.cls",
          r"\ProvidesClass{pod}
\RequirePackage{kvoptions}
\SetupKeyvalOptions{family=po,prefix=po@}
\DeclareVoidOption{scheme}{\PassOptionsToPackage{\CurrentOption}{pod}}
\ProcessKeyvalOptions*
\LoadClass[12pt]{article}
\RequirePackage{pod}
",
        ),
        (
          "pod.sty",
          r"\ProvidesPackage{pod}
\RequirePackage{kvoptions}
\SetupKeyvalOptions{family=po,prefix=po@}
\DeclareBoolOption{scheme}
\ProcessKeyvalOptions*
\ifpo@scheme\newenvironment{scheme}{}{}\fi
",
        ),
      ],
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("hi") && !xml.contains("ERROR"),
      "option passed via \\PassOptionsToPackage was lost:\n{xml}"
    );
  }

  /// Batch 45: amsmath's `\newif\if@display` (amsmath.sty:649) exists.
  /// RED: gaceta.cls:1666 redefines `\mod` with amsmath's own
  /// `\if@display…\else…\fi` body via babel's `\addto`; the undefined
  /// conditional orphaned 2×`\else`, 2×`\fi` and leaked `#1`
  /// (`misdefined:#`). SHARED with Perl. Witnesses: gaceta
  /// plantilla-articulo-suelto / -de-seccion (10 → 0 each).
  #[test]
  fn amsmath_if_display_defined() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{amsmath}
\usepackage[spanish]{babel}
\makeatletter
\addto\es@operators{%
  \renewcommand{\mod}[1]{\allowbreak\if@display\mkern18mu
  \else\mkern12mu\fi{\operator@font mod}\,\,#1}%
}
\makeatother
\begin{document}x $a \mod b$\end{document}
",
    );
    assert!(
      !stderr.contains("unexpected:fi") && !stderr.contains("misdefined"),
      "\\if@display went missing again:\n{stderr}"
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("mod"), "\\mod body did not digest:\n{xml}");
  }

  /// Batch 45: natbib's full `\newif` surface (`\ifNAT@full` :683,
  /// `\ifNAT@longnames` :284, …). RED: nmbib.sty:267 `\ifNAT@full` was
  /// undefined, so the `\fi` of its skipped branch closed the outer
  /// conditional (`unexpected:fi`). SHARED with Perl. Witness:
  /// nmbib/nmbib-sample (4 fi lines; the doc keeps a SHARED residual on
  /// ~20 other natbib internals).
  #[test]
  fn natbib_newif_surface_complete() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{nmbib}
\begin{document}
\citeall{X}
\end{document}
",
    );
    assert!(
      !stderr.contains("unexpected:fi") && !stderr.contains("undefined:\\ifNAT@"),
      "natbib \\newif surface regressed:\n{stderr}"
    );
  }

  /// Batch 45: `\@enumctr` is defined by enumerate lists (latex.ltx:16057
  /// `\edef\@enumctr{enum\romannumeral\the\@enumdepth}`). RED: beginItemize
  /// (Perl pool:1314, SHARED) only defined `\@listctr`; bullenum.sty:58/61
  /// `\csname the\@enumctr\endcsname` cascaded to the 100-error cap.
  /// Witness: bullcntr/bullcntr-man.
  #[test]
  fn enumerate_defines_enumctr() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{bullenum}
\begin{document}
\begin{bullenum}
\item First
\item Second
\end{bullenum}
\end{document}
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(
      xml.matches("<item").count(),
      2,
      "bullenum items did not materialize:\n{xml}"
    );
  }
}
