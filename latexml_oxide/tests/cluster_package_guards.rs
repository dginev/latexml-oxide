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
  //! texmf packages: `daj` (contrib-bound name whose binding is a compiled
  //! article-based definition set — `scrartcl` served here until batch 53
  //! made its binding a raw shim that itself `\input`s the `.cls`) for
  //! precedence, `pkzzz` (no binding anywhere) for the no-OmniBus raw load.

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

  /// Directive 1: the contrib `daj` binding wins even under rawclasses —
  /// the local raw `.cls`'s marker must NOT appear.
  #[test]
  fn contrib_binding_keeps_precedence_under_rawclasses() {
    let (xml, _stderr) = convert("daj", RAW_CLS, Some("[rawstyles,rawclasses]latexml.sty"));
    assert!(
      xml.contains("NOMARKER") && !xml.contains("RAWCLSLOADED"),
      "compiled daj binding must outrank the raw .cls under rawclasses:\n{xml}",
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
  //! (1) A document-position `\input{<name>.sty}` with no binding reads the
  //! raw file as CONTENT under the current catcodes — real TeX's `\input`
  //! (batch 7; re-read on every `\input`, batch 52). Perl (Package.pm:2289-2302)
  //! instead loads it as definitions (`@`=letter, `[cat:11]`, text
  //! suppressed) and SKIPS a second `\input`/`\DocInput` of the same file —
  //! which is how it dodges doc.sty's `\CharacterTable` self-check and also
  //! why it never typesets a `.sty`'s documentation body (frankenstein bundle;
  //! the content route runs those bodies and still errors — PLANS P66).
  //! (2) biblatex's `style=`/`bibstyle=`/`citestyle=` options load the raw
  //! `.bbx`/`.cbx` style files (biblatex.sty L2256/L11428), whose
  //! `\newtoggle`s etc. were undefined corpus-wide (windycity,
  //! biblatex-ext/-fiwi/-sbl).

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
  pub(crate) fn convert_with_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
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
  // `listing only`: the body is C-like text that the default `listing and
  // text` mode would also execute in real LaTeX (codebox.sty declares its
  // listings `listing only` for the same reason).
  #[test]
  fn newtcblisting_xparse_leading_optional() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[skins]{tcolorbox}
\tcbuselibrary{listings}
\newcounter{example}
\NewTCBListing[use counter=example, number within=section]{example}{ O{} s m }{ title={\thetcbcounter}, listing only, #1 }
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

pub(crate) mod perfect_kernel_batch46 {
  //! Red/green guards for perfect-kernel batch 46 (PLANS P27/P28/P29/P23/P24).
  //! Each test is the minimal reproduction distilled during triage; the
  //! doc-comment names the ORIGINAL corpus witness (TeX Live doc corpus,
  //! `bundle/doc`) whose larger conversion was vetted separately.
  use std::{path::Path, process::Command};

  /// Convert an inline snippet in a tempdir; `raw` selects the perfect-kernel
  /// preload, otherwise the default (arXiv) configuration. Returns
  /// (ANSI-stripped stderr, XML string).
  pub(crate) fn convert(tex: &str, raw: bool) -> (String, String) {
    convert_with(
      tex,
      if raw {
        Some("[rawstyles,rawclasses]latexml.sty")
      } else {
        None
      },
    )
  }

  /// `convert` with the raw preload plus extra CLI arguments (`--streaming`,
  /// `--max-memory=N`, …).
  pub(crate) fn convert_args(tex: &str, extra: &[&str]) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let mut args = vec![
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ];
    args.extend_from_slice(extra);
    let output = Command::new(bin)
      .args(&args)
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  /// Like `convert_args` with the raw preload, after writing `files`
  /// (`(name, content)`) into the work directory — for repros that need a
  /// package, class or data file beside the document.
  pub(crate) fn convert_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    for (name, content) in files {
      std::fs::write(workdir.path().join(name), content).expect("write file");
    }
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--timeout=110",
        "--preload=[rawstyles,rawclasses]latexml.sty",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  pub(crate) fn convert_with(tex: &str, preload: Option<&str>) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let mut args = vec!["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=110"];
    let preload_arg = preload.map(|p| format!("--preload={p}"));
    if let Some(ref p) = preload_arg {
      args.push(p);
    }
    let output = Command::new(bin)
      .args(&args)
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  pub(crate) fn error_count(stderr: &str) -> usize {
    stderr.lines().filter(|l| l.starts_with("Error:")).count()
  }

  const MEMOIR: &str = r"\documentclass{memoir}
\begin{document}
\chapter{C}
\onelineskip
\section{S}
Body.
\end{document}
";

  /// P27: memoir.cls is raw-interpreted through the engine (the binding is a
  /// raw-load shim, tlp/czjphys precedent). RED: the former stub hid the real
  /// class — `\onelineskip` and every memoir-only macro undefined. Witnesses:
  /// titlepages/titlepages (4→0), dlfltxb/dlfltxbmarkup (3→0), memexsupp.
  /// Both preload modes must agree, since the binding is what makes the
  /// class raw-load under the default arXiv configuration too.
  #[test]
  fn memoir_raw_loads_in_both_modes() {
    for raw in [true, false] {
      let (stderr, xml) = convert(MEMOIR, raw);
      assert_eq!(error_count(&stderr), 0, "raw={raw}:\n{stderr}");
      assert!(
        xml.contains("<chapter") && xml.contains("<section"),
        "raw={raw}:\n{xml}"
      );
    }
  }

  /// P28: nicematrix.sty / tabularray.sty ARE implemented — the stale
  /// `missing_file` "not implemented and will not be interpreted raw"
  /// warnings misreported every document using them.
  #[test]
  fn nicematrix_tabularray_no_stale_missing_file_warning() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{nicematrix,tabularray}
\begin{document}
\begin{NiceTabular}{cc} a & b \\ \end{NiceTabular}
\begin{tblr}{cc} a & b \\ \end{tblr}
\end{document}
",
      true,
    );
    assert!(
      !stderr.contains("missing_file:nicematrix") && !stderr.contains("missing_file:tabularray"),
      "stale missing_file warning is back:\n{stderr}"
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// P29: `\index` expands its entry before splitting on `@`/`!`/`|`, as
  /// real `\@wrindex` writes it via `\protected@write` (latex.ltx:17720),
  /// and a sort key holding sanitized specials is a plain makeindex string.
  /// RED: macro-built entries never met their `@` (tcolorbox documentation
  /// library `\kvtcb@doc@sortindex\idx@actual…`), so the sort key was
  /// digested as text and every `_` in it errored; a literal `a_b@…` key
  /// errored too and rendered `˙`. Witness: tagpdf/tagpdf (113→21).
  #[test]
  fn index_entry_expands_and_keys_sanitized_specials() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\makeindex
\begin{document}
\def\key{x_y}\def\show{\texttt{x\_y}}
A\index{a_b@\texttt{a\_b}}
B\index{\key @\show}
D\index{plain}\index{p|see{plain}}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    for key in ["key=\"a_b\"", "key=\"x_y\"", "key=\"plain\""] {
      assert!(xml.contains(key), "missing {key}:\n{xml}");
    }
    assert!(!xml.contains('˙'), "sort key rendered through OT1:\n{xml}");
  }

  /// P29 witness shape: tcolorbox `docCommand{tag_if_active:TF}` writes
  /// `\index{\kvtcb@doc@sortindex\idx@actual\tcbIndexPrintComC{…}}`
  /// (tcbdocumentation.code.tex:495). RED: 4 `Script _` errors per entry.
  #[test]
  fn tcolorbox_doccommand_index_key_expands() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[documentation]{tcolorbox}
\begin{document}
\begin{docCommand}{tag_if_active:TF}{}\end{docCommand}
Text.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("key=\"tag_if_active:TF\""), "{xml}");
  }

  /// P23: `NiceTabularX{width}[opts]{colspec}[opts]` (nicematrix.sty:3788)
  /// is a tabularx — its `X` columns need the tabularx column engine. RED:
  /// the reduction to `\tabular` dropped every X cell ("Unrecognized tabular
  /// template X" + "Extra alignment tab"). Witness: nicematrix/nicematrix
  /// `\begin{NiceTabularX}{\linewidth}{l||*{\LastDay}{X}}`.
  #[test]
  fn nicetabularx_is_a_tabularx() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{nicematrix}
\newcommand\LastDay{3}
\begin{document}
\begin{NiceTabularX}{\linewidth}{l||*{\LastDay}{X}}[hvlines]
a & b & c & d \\
\end{NiceTabularX}
\begin{NiceTabular*}{\linewidth}[hvlines]{cc}
e & f \\
\end{NiceTabular*}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("Unrecognized tabular template"),
      "{stderr}"
    );
    assert_eq!(xml.matches("<td").count(), 6, "{xml}");
  }

  /// P24: tabularray's template API — `\DeclareTblrTemplate` (:5673; the
  /// bound `\DefTblrTemplate` is only its alias), `\UseTblrTemplate`,
  /// `\MapTblrRemarks`, `\InsertTblrRemarkTag`. Witness: tabularray-abnt.
  #[test]
  fn tabularray_template_api_defined() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{tabularray}
\DeclareTblrTemplate{remark-tag}{x}{\InsertTblrRemarkTag}
\SetTblrTemplate{remark-tag}{x}
\begin{document}
\UseTblrTemplate{remark-tag}{x}\MapTblrRemarks{\InsertTblrRemarkTag}
\begin{tblr}{cc} a & b \\ \end{tblr}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }
}

mod perfect_kernel_batch47 {
  //! Red/green guards for perfect-kernel batch 47 (PLANS P40/P41/P32/P33/P45).
  use super::perfect_kernel_batch46::{convert, error_count};

  /// P40: an unknown column type followed by `{arg}` makes the template
  /// reader's "safety valve" (Perl Alignment.pm:906, shared) re-read the arg
  /// as column letters; `m` then swallows the template's closing brace and
  /// the reader runs into the table body. Batch 33's macro-valued-colspec
  /// expansion made that worse by invoking primitive expandables met on the
  /// way (`\csname` → scanned to EOF: nicematrix/nicematrix 109→1002+Fatal).
  /// Two cuts: nicematrix registers its `V{width}` (nicematrix.sty:2541),
  /// and the reader only expands token-bodied macros.
  #[test]
  fn nicematrix_v_column_and_macro_colspec() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{varwidth}
\usepackage{nicematrix}
\begin{document}
\begin{NiceTabular}{V{3cm}V{3cm}}
a & b \\
\end{NiceTabular}
\def\mycol{cc}
\begin{tabular}{\mycol}c&d\\\end{tabular}
\begin{tabular}{\csname mycol\endcsname}e&f\\\end{tabular}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("Unrecognized tabular template \"V\""),
      "{stderr}"
    );
    assert!(
      !stderr.contains("should not appear between \\csname"),
      "{stderr}"
    );
    assert_eq!(xml.matches("<td").count(), 6, "{xml}");
  }

  /// P41: `\\` outside any alignment hit a Rust-only `Err` in
  /// `read_newline_args` → Fatal. Perl pool:557-571 never guards. Shape:
  /// aguplus.cls:305-307 `\pt@tabular` does `\let\\\@tabularcr\@tabarray`
  /// — `\@tabarray` is the bare `\@@array[c]` constructor without
  /// `\@array@bindings`/`\lx@begin@alignment` (only `\array`/`tabular` add
  /// them), so `\@tabularcr` (= `\lx@alignment@newline`) fires with no
  /// Alignment in State. Same-host Perl: 2 `Stray alignment "&"` errors,
  /// no Fatal. Witness aguplus/aguplus `planotable`: Fatal → full doc.
  /// Runs non-raw too (the shape is kernel-only).
  #[test]
  fn newline_outside_alignment_is_not_fatal() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\makeatletter
\begin{document}
\let\\\@tabularcr\@tabarray{lcc}\\
Brix & 45 & 90
\endtabular
\end{document}
",
      false,
    );
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert!(!stderr.contains("read_newline_args"), "{stderr}");
    // `\@tabarray` is now the full array setup (batch 54x): in text mode
    // real TeX errors too ("Missing $ inserted" for the `\vcenter`), so the
    // guard only requires the non-fatal recovery.
    assert!(stderr.contains("Error:"), "{stderr}");
  }

  /// P32: `[first-col]`/`[last-col]` add a label cell to every source row of
  /// the NiceTabular family (nicematrix.tex:2569/2617), so the preamble must
  /// grow a column — the array family already did. The trailing `[opts]`
  /// counts too (nicematrix.sty:2007 merges both). RED: 2 `Extra alignment
  /// tab` per row. P33: `\EmptyColumn{j}` / `\EmptyRow{i}` are
  /// `\CodeBefore`-scoped (nicematrix.sty:1808) — were `undefined`
  /// (nicematrix.tex:2716).
  #[test]
  fn nicetabular_first_col_growth_and_codebefore_surface() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
\begin{NiceTabular}{ccc}[hvlines,first-row,first-col]
  & 0 & 1 & 2 \\
0 & 1 & 2 & 3 \\
\end{NiceTabular}
\begin{NiceTabular}{ccc}[no-cell-nodes]
\CodeBefore
  \EmptyColumn{3}
  \EmptyRow{1}
\Body
   one & two & three \\
\end{NiceTabular}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Extra alignment tab"), "{stderr}");
    assert_eq!(xml.matches("<td").count(), 11, "{xml}");
  }

  /// P45: an EMPTY virtual file still exists. tcolorbox's `\tcbwritetemp`
  /// over an empty `posterboxenv` body (tcbposter.code.tex:168-171) stores
  /// "" and `\tcbusetemp` `\input`s it back; `find_file_aux` read the empty
  /// entry as absent and fell through to disk. Witness
  /// tcolorbox/tcolorbox-tutorial-poster (`missing_file:<job>.tcbtemp` ×7).
  #[test]
  fn empty_virtual_file_exists() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[poster]{tcolorbox}
\begin{document}
\begin{tcbposter}[poster={columns=2,rows=2}]
\begin{posterboxenv}[adjusted title=Core]{name=algo,column=1,row=1}
\end{posterboxenv}
\posterbox[adjusted title=Contact]{name=contact,column=2,row=1}{Contact body.}
\end{tcbposter}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("missing_file"), "{stderr}");
    assert!(xml.contains("Contact body"), "{xml}");
  }
}

mod perfect_kernel_batch48 {
  //! Red/green guards for perfect-kernel batch 48 (PLANS P43/P44 + the
  //! `{VerbatimOut}` zero-diffs trap + the virtual-file-store abort).
  //! RED on the batch-47 binary, green now; each doc-comment names the
  //! corpus witness whose full conversion was vetted separately.
  use super::perfect_kernel_batch46::{convert, error_count};

  /// P43: currfile.sty (+ filehook) raw-loads and its `\ifcurrfilename`
  /// family compares against the sanitized `\filename@parse` pieces. RED:
  /// the former binding left `\ifcurrfile*` undefined (pythontex/pythontex
  /// ×2 → 1/0/0) and the comparison always said "no".
  #[test]
  fn currfile_ifcurrfilename_compares() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{currfile}
\begin{document}
[\currfilename][\currfilebase][\currfileext]
\ifcurrfilename{x.tex}{yes}{no} done
\ifcurrfilename{t.tex}{yes}{no} done
\ifcurrfilebase{t}{yes}{no} done
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[t.tex][t][tex]"), "{xml}");
    assert!(xml.contains("no done\nyes done\nyes done"), "{xml}");
  }

  /// `{VerbatimOut}` runs RAW (fancyvrb.sty:1053-1066 `\FVB@VerbatimOut`
  /// writes each `\FV@ProcessLine` to `\FV@OutFile`), so a wrapper
  /// environment opened with `\VerbatimEnvironment` (the pythontex/minted
  /// idiom, fancyvrb.sty:1034) captures its body and `\VerbatimInput`s it
  /// back. RED: the Rust override read the body itself and lost the wrapper.
  /// Witnesses: pythontex/pythontex_gallery, fancybox/fancybox-doc (fancybox
  /// re-implements the same layer, fancybox.sty:1000-1020).
  #[test]
  fn verbatimout_wrapper_environment_round_trips() {
    for pkg in ["fancyvrb", "fancybox"] {
      let (stderr, xml) = convert(
        &format!(
          r"\documentclass{{article}}
\usepackage{{{pkg}}}
\newenvironment{{pyg}}{{\VerbatimEnvironment\begin{{VerbatimOut}}{{\jobname.pyg}}}}{{\end{{VerbatimOut}}\VerbatimInput{{\jobname.pyg}}}}
\begin{{document}}
before
\begin{{pyg}}
x = 1
\end{{pyg}}
after
\end{{document}}
"
        ),
        true,
      );
      assert_eq!(error_count(&stderr), 0, "{pkg}:\n{stderr}");
      assert!(xml.contains("x = 1"), "{pkg}:\n{xml}");
      assert!(xml.contains("after"), "{pkg}:\n{xml}");
    }
  }

  /// A package may `\let\FVB@VerbatimOut` to its own line processor
  /// (pythontex.sty's `\pytx@FVB@…` counts and stores lines); the raw
  /// `\FV@Scan` loop must call THAT, and a later plain `{VerbatimOut}` still
  /// writes its file. RED: the override bypassed `\FVB@VerbatimOut` entirely.
  #[test]
  fn verbatimout_custom_fvb_hook_is_honoured() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{fancyvrb}
\makeatletter
\newcounter{mylines}
\def\my@FVB@VerbatimOut{\begingroup\let\FV@ProcessLine\my@line\let\FV@FontScanPrep\relax\let\@noligs\relax\FV@Scan}
\def\my@FVE@VerbatimOut{\endgroup}
\def\my@line#1{\stepcounter{mylines}}
\newenvironment{mycode}{\VerbatimEnvironment\let\FVB@VerbatimOut\my@FVB@VerbatimOut\let\FVE@VerbatimOut\my@FVE@VerbatimOut\begin{VerbatimOut}}{\end{VerbatimOut}[\themylines\ lines]}
\makeatother
\begin{document}
before
\begin{mycode}
x = {1
y = 2}
\end{mycode}
after
\begin{VerbatimOut}{\jobname.vo}
z = 3
\end{VerbatimOut}
\VerbatimInput{\jobname.vo}
end
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[2 lines]"), "{xml}");
    assert!(xml.contains("z = 3"), "{xml}");
  }

  /// The zero-diffs trap: `\begin{VerbatimOut}[keys]{file}` (fancyvrb.sty:1053
  /// takes the optional key list first). RED: the override read `[` as the
  /// file name — "Cached VerbatimOut for [" — and swallowed the REST OF THE
  /// DOCUMENT into that file, reporting a clean run with nothing rendered.
  /// 18 corpus docs were masked this way (spath3/spath3, mhchem/mhchem,
  /// kblocks/kblocks-doc, xcolor/xcolor2, yquant/yquant-doc, …).
  #[test]
  fn verbatimout_optional_keys_do_not_swallow_document() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
before
\begin{VerbatimOut}[gobble=0]{\jobname.vo}
k = 4
\end{VerbatimOut}
\VerbatimInput{\jobname.vo}
after
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Cached VerbatimOut for ["), "{stderr}");
    assert!(xml.contains("k = 4"), "{xml}");
    assert!(xml.contains("after"), "{xml}");
  }

  /// fancybox's own verbatim layer (`{SaveVerbatim}`/`\UseVerbatim`/`\Verb`/
  /// `{Verbatim}`, fancybox.sty:680-1000) runs raw; back-quotes stay verbatim
  /// characters. RED: `undefined` errors for the whole family once the
  /// fancybox-doc demos actually executed. Witness: fancybox/fancybox-doc.
  #[test]
  fn fancybox_verbatim_layer_raw() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{fancybox}
\begin{document}
\begin{SaveVerbatim}{\sv}
x `y' z
\end{SaveVerbatim}
A\UseVerbatim{\sv}B

\Verb|`q'| and \begin{Verbatim}
v `w'
\end{Verbatim}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("x ‘y’ z"), "{xml}");
    assert!(xml.contains("v ‘w’"), "{xml}");
  }

  /// A `{VerbatimOut}` left open at end of input. `\FV@Scan` re-reads one
  /// `Until:^^M` line per iteration (fancyvrb.sty:1090); at true end of all
  /// input the `Until` reader must report the runaway (tex.web §338 "File
  /// ended while scanning"; Perl Parameter.pm:93-97 → `Missing argument
  /// Until:` ×100 → `Fatal:TooManyErrors`; since batch 52 one report then
  /// the tex.web §360 job-abort Fatal, see `until_miss_at_eof_is_fatal_once`).
  /// RED: the reader mapped the miss
  /// to an empty line quietly, so the loop spun forever while each `\write`
  /// re-pinned the whole growing file in the never-freed interner — the
  /// buffer offset overran `u32` and the binary ABORTED (`string-interner
  /// get_unchecked` precondition). Witness: fancyvrb/fancyvrb-doc cut at
  /// its open `{SideBySideExample}`.
  #[test]
  fn until_miss_at_true_eof_errors_like_perl() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
\begin{VerbatimOut}{\jobname.tmp}
never closed
",
      true,
    );
    assert!(
      stderr.contains("Error:expected:Until: Missing argument Until:"),
      "{stderr}"
    );
    assert!(stderr.contains("Fatal:Mouth:EoF"), "{stderr}");
    assert!(
      !stderr.contains("panicked") && !stderr.contains("precondition"),
      "{stderr}"
    );
  }

  /// The `\write`-per-line append shape over a long stream stays cheap and
  /// reads back whole (the owned-map virtual file store; the LSP overlay and
  /// `{filecontents}` share it).
  #[test]
  fn virtual_file_many_line_append_reads_back() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\begin{document}
\newwrite\w\immediate\openout\w=\jobname.big
\newcount\n
\loop\immediate\write\w{L\the\n}\advance\n1 \ifnum\n<2000 \repeat
\immediate\closeout\w
\input{\jobname.big}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("L0\n") && xml.contains("L1999"), "{xml}");
  }
}

mod perfect_kernel_batch49 {
  //! Red/green guards for perfect-kernel batch 49 (PLANS P53/P51/P54a).
  use super::perfect_kernel_batch46::{convert, error_count};

  /// P53: `\@currenvir` for a `DefEnvironment!` environment is the
  /// environment name's CHARACTER tokens (Perl Package.pm:1927 `DefMacroI`
  /// → TokenizeInternal; latex.ltx:15350 `\def\@currenvir{#1}`), so the
  /// `\ifx\reserved@a\@currenvir` idiom (`\@checkend` latex.ltx:15394,
  /// collectbox:208, storebox:36, nag:258, powerdot:529 …) matches. RED: one
  /// multi-char letter token — stringified right, never `\ifx`-equal, so
  /// every such consumer took the wrong branch silently (Perl: `SAME`).
  /// `lstlisting` (listings binding) set the same single-token shape.
  #[test]
  fn currenvir_is_character_tokens() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{listings}
\makeatletter
\def\check#1{\def\a{#1}\edef\b{\@currenvir}\ifx\a\b SAME\else DIFF\fi}
\begin{document}
\begin{center}\check{center}\end{center}
\begin{itemize}\item \check{itemize}\end{itemize}
\begin{lstlisting}[title={\check{lstlisting}}]
x
\end{lstlisting}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("DIFF"), "{xml}");
    // center, itemize, and the listing title (rendered as title + caption).
    assert_eq!(xml.matches("SAME").count(), 4, "{xml}");
  }

  /// P51: `\definecolor[ps]{name}{model}{PostScript}` still REGISTERS the
  /// color, with the model's white as its non-PostScript value
  /// (xcolor.sty:531-533, `\XC@clr@<model>@white` :510-516). RED: dropped
  /// entirely (Perl xcolor.sty.ltxml:403-409 too — KNOWN_PERL_ERRORS), so
  /// `\color{lambda}` errored 101× → `Fatal:TooManyErrors`. Witness
  /// xcolor/xcolor2 (xcolor2.tex:143 defines, :134 uses in `\multiput`).
  #[test]
  fn xcolor_ps_color_registers_as_model_white() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\definecolor[ps]{lambda}{rgb}{Red Corr Green Corr Blue Corr}
\providecolor[ps]{mu}{cmyk}{0 0 0 0}
\textcolor{lambda}{hello} and \textcolor{mu}{there}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      stderr.contains("Ignoring definition of postscript color"),
      "{stderr}"
    );
    assert!(xml.contains("<text color=\"#FFFFFF\">hello"), "{xml}");
    assert!(xml.contains("<text color=\"#FFFFFF\">there"), "{xml}");
  }

  /// P54a: ctex's pdfTeX layer requires CJKpunct (ctex-engine-pdftex.def:122),
  /// whose raw `\CJKpunct@utfasymbol` (CJKpunct.sty:449) routes the six
  /// declared punctuation codepoints through `\CJK@punctchar{\CJK@uniPunct}…`
  /// — supplied by CJK.enc:291 / `*.chr` in real CJK, never loaded behind the
  /// CJK binding. RED: 2 `undefined` per doc across 18 ctex manuals
  /// (Perl identical). Witnesses: jnuexam/jnuexam (2→0), joinbox/joinbox,
  /// suanpan-l3/suanpan-l3.
  #[test]
  fn ctex_cjkpunct_unicode_punctuation() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[scheme=plain]{ctex}
\begin{document}
A“B”C—D…E‘F’G
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("CJK@punctchar") && !stderr.contains("CJK@uniPunct"),
      "{stderr}"
    );
    assert!(xml.contains("A“B”C—D…E‘F’G"), "{xml}");
  }
}

/// Red/green guards for perfect-kernel batch 50 (PLANS P50 …).
mod perfect_kernel_batch50 {
  use super::perfect_kernel_batch46::{convert, error_count};

  /// P50: `\filename@parse` is latex.ltx:228-281's own macro, so
  /// `\filename@area/base/ext` keep the ARGUMENT's tokens. The Perl-shaped
  /// primitive re-lettered the pieces (`ExplodeText`), so a caller that
  /// `\@onelevel@sanitize`d its argument first (currfile.sty:78-85 and its
  /// `\ifx\@tempa\currfilename` compare; import.sty; docstrip) got a catcode
  /// mismatch real LaTeX never has. RED: T4/T5 answered `no`. Witnesses: the
  /// currfile users in the TL doc corpus (pythontex/pythontex, knowledge/
  /// knowledge, milsymb/milsymb, dlrg/dlrg), repro `cf4.tex`.
  #[test]
  fn filename_parse_keeps_argument_catcodes() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\makeatletter
\begin{document}
\edef\x{abc}\@onelevel@sanitize\x \edef\y{abc}
T1:\ifx\x\y yes\else no\fi

\def\@filef@und{sub/dir/cf4.tar.tex}\@onelevel@sanitize\@filef@und
\expandafter\filename@parse\expandafter{\@filef@und}
\edef\a{\filename@base}\edef\b{\filename@ext}\edef\c{\filename@area}
\edef\p{\detokenize{cf4.tar}}\edef\q{\detokenize{tex}}\edef\r{\detokenize{sub/dir/}}
T4:\ifx\a\p yes\else no\fi
T5:\ifx\b\q yes\else no\fi
T6:\ifx\c\r yes\else no\fi

\filename@parse{plain}
T7:\ifx\filename@ext\relax yes\else no\fi
\edef\d{\filename@base}\edef\e{plain}
T8:\ifx\d\e yes\else no\fi
\end{document}
",
      false,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    for probe in ["T1:no", "T4:yes", "T5:yes", "T6:yes", "T7:yes", "T8:yes"] {
      assert!(xml.contains(probe), "missing {probe}\n{xml}");
    }
  }

  /// calc `\widthof{$…$}` evaluated INSIDE inline math (mhchem.sty:2898-2904
  /// `\mhchem@minispace`, run from the prescript path `\ce{^{227}Th}` under an
  /// open `\ensuremath`, :2781): Perl calc.sty.ltxml:140 measures the argument
  /// in a fresh `restricted_horizontal` box, so its nested `$…$` is its own
  /// math; the Rust port digested it in the ambient (math) mode and the inner
  /// `$` closed the ENCLOSING math frame. RED: 6× `Error:unexpected:
  /// \lx@end@inline@math Attempt to end mode math` (Perl 0). Witness: TL doc
  /// corpus mhchem/mhchem (67 → 13; the rest is the SHARED `\ce` in `align*`
  /// R3c family).
  #[test]
  fn calc_widthof_in_math_measures_own_box() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[version=4]{mhchem}
\usepackage{calc}
\newlength\mylen
\begin{document}
\ce{^{227}Th}

X $a\setlength{\mylen}{\widthof{$b$}}c$ Y
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("lx@end@inline@math"), "{stderr}");
    // Prescript stays inside ONE well-formed inline Math.
    assert!(
      xml.contains(r#"<XMTok role="SUPERSCRIPTOP" scriptpos="pre1"/>"#),
      "{xml}"
    );
    assert!(xml.contains("227"), "{xml}");
    assert_eq!(xml.matches("<Math ").count(), 2, "{xml}");
    assert!(
      xml.contains("X <Math") && xml.contains("</Math> Y"),
      "{xml}"
    );
  }

  /// verbatim.sty:107-112 `\verbatim@start#1` swallows a following control
  /// sequence (`\if\noexpand#1\noexpand~` is true for any CS), which is the
  /// documented `\verbatim@start\relax` idiom for opening a capture from a
  /// macro body (curve2e-manual.tex:95 `{Esempio}`, newfile.sty:131
  /// `\writeverbatim`). RED: the pending `\relax` was serialised as the
  /// first captured line (`<verbatim>\relax\n…`), and the `\write`-then-
  /// `\verbatiminput` round trip carried it. Oracle pdflatex: no such line.
  /// Witnesses: curve2e/curve2e-manual (100+Fatal → 41, all 26
  /// `missing_file` gone), digiconfigs/digiconfigs (9 → 5).
  #[test]
  fn verbatim_start_relax_idiom() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{verbatim}
\makeatletter
\newwrite\example@out
\newenvironment{Esempio}{\par
\begingroup
\@bsphack
\immediate\openout\example@out\jobname-temp.tex
\let\do\@makeother\dospecials\catcode`\^^M\active
\def\verbatim@processline{%
  \immediate\write\example@out{\the\verbatim@line}}%
\verbatim@start\relax}%
{\immediate\closeout\example@out\@esphack\endgroup
\verbatiminput{\jobname-temp.tex}
\input{\jobname-temp}%
\par}
\makeatother
\begin{document}
before
\begin{Esempio}
Hello \textbf{world} 1
second line
\end{Esempio}
after
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains(r"\relax"), "{xml}");
    assert!(
      xml.contains(
        "<verbatim font=\"typewriter\">Hello \\textbf{world} 1\nsecond line\n</verbatim>"
      ),
      "{xml}"
    );
    assert!(
      xml.contains("<p>Hello <text font=\"bold\">world</text> 1\nsecond line</p>"),
      "{xml}"
    );
  }
}

mod perfect_kernel_batch51 {
  //! Red/green guards for perfect-kernel batch 51 (sweep 28 "Until … at end
  //! of input" cluster A and the `\endlx@list` cluster B). Each test is the
  //! minimal reproduction distilled during triage; the doc-comment names the
  //! ORIGINAL corpus witness (TeX Live doc corpus) whose larger conversion
  //! was vetted separately.
  use std::{path::Path, process::Command};

  use super::perfect_kernel_batch46::{convert, error_count};

  /// Like [`convert`] (raw preload), but first drops extra `(name, content)`
  /// files into the tempdir so the snippet can `\input` them.
  pub(super) fn convert_with_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    for (name, content) in files {
      std::fs::write(workdir.path().join(name), content).expect("write aux file");
    }
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--timeout=110",
        "--preload=[rawstyles,rawclasses]latexml.sty",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  /// P15 (file side): eTeX §362 begins the `\everyeof` token list at the end
  /// of EVERY `\input` file, before the file is closed — so a delimited
  /// argument opened across the `\input` PRIMITIVE (`\expandafter\eat
  /// \@@input f` — LaTeX's `\input` is a macro, and real TeX runs away on
  /// it too) is terminated by the register and never scans past the file.
  /// pdflatex oracle: `[alpha beta ]Tail.`, no errors. RED: the register was
  /// inserted only for `\scantokens`; `\eat#1\stopper` ran to the end of the
  /// document ("Missing argument Until:\stopper at end of input"), captured
  /// nothing, and `Tail.` followed an empty `[]`. Witnesses:
  /// tikzmarmots-doc.tex:44-105 `\CommentInput` (`\tex_everyeof:D` +
  /// `\tex_input:D`; 0 → 501 errors + Fatal on the sweep-28 binary),
  /// tikzlings-doc (35 → 536), stex.sty:2633 smsmode
  /// `\everyeof{\q__stex_smsmode_break\exp_not:N}`, expl3-code.tex
  /// `\__file_get_do:Nw`.
  #[test]
  fn input_file_end_inserts_everyeof() {
    let (stderr, xml) = convert_with_files(
      r"\documentclass{article}
\begin{document}
\makeatletter
\def\stopper{STOP}
\begingroup
\everyeof{\stopper}%
\def\eat#1\stopper{[\detokenize{#1}]}%
\expandafter\eat\@@input lines.tex
\endgroup
Tail.
\end{document}
",
      &[("lines.tex", "alpha\nbeta\n")],
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("[]"), "{xml}");
    assert!(xml.contains("[alpha beta ]"), "{xml}");
    assert!(xml.contains("Tail."), "{xml}");
  }

  /// amsgen.sty:54-62 `\new@ifnextchar` does NOT skip spaces — Perl
  /// (amsgen.sty.ltxml:42) Lets it to the space-skipping `\@ifnextchar`,
  /// KNOWN_PERL_ERRORS #113. bibleref.sty:969 `\bibleverse` uses it to look
  /// for an immediately-following `(`; with the space skipped,
  /// `\bibleverse{Psalms} (Einzahl)` opened `\@bibleverse(#1:` and scanned to
  /// the end of the document. RED: `<relationaltoken>` ×2 + `Until::` at end
  /// of input, the whole paragraph lost. Witnesses: en-bibleref-german,
  /// de-bibleref-german (bibleref-german-preamble.tex:120; 12 `Until::`
  /// misses each, sweep 28).
  #[test]
  fn new_ifnextchar_keeps_space() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{bibleref}
\begin{document}
Beispiel: \bibleverse{Psalms} (Einzahl) und \bibleverse{Psalms}(23:1) hier.
Ende.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("(Einzahl)"), "{xml}");
    assert!(xml.contains("23:1"), "{xml}");
    assert!(xml.contains("Ende."), "{xml}");
  }

  /// The contrib `\printbibliography` (mirroring ar5iv-bindings
  /// biblatex.sty.ltxml:410) rebinds `\verb` to `\biblatex@verb{} Until:
  /// \endverb` for reading the `.bbl` and never restored it, so every
  /// `\verb+x+` after the bibliography scanned to the end of the document
  /// (KNOWN_PERL_ERRORS #114). RED: two "Missing argument Until:\endverb at
  /// end of input", delimiters leaked as text (`foo.dtx+`), no verbatim
  /// element. Witnesses: docsurvey.tex:2876-2898 (7 `\verb+.dtx+` after the
  /// bibliographies, ~500 lines of body lost), rub-kunstgeschichte-example.
  #[test]
  fn verb_survives_printbibliography() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{filecontents}
\begin{filecontents}{t.bib}
@book{knuth84, author={Donald Knuth}, title={The TeXbook}, year={1984}, publisher={Addison-Wesley}}
\end{filecontents}
\usepackage[backend=biber]{biblatex}
\addbibresource{t.bib}
\begin{document}
Cite \cite{knuth84}.
\printbibliography
Files: \verb+foo.dtx+ and \verb|bar.ins| here.
Trailing text survives.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("foo.dtx+"), "{xml}");
    assert!(
      xml.contains(">foo.dtx<") && xml.contains(">bar.ins<"),
      "{xml}"
    );
    assert!(xml.contains("Trailing text survives."), "{xml}");
  }

  /// memoir.cls:4580 defines `\list` raw, ending in `\@trivlist` — no group
  /// of its own — while `\endlist` is still our `\endlx@list`, which
  /// unconditionally `egroup`ed — popping the ENCLOSING frame whenever it
  /// was a plain `{` group, after which every later `\global`/`\let` in the
  /// document cascaded. The closer now pops only a frame that `\lx@list`
  /// itself opened (groupInitiator) and otherwise reports Perl's
  /// `endMode` error without popping (Stomach.pm:524-531). RED here:
  /// "Attempt to close boxing group … due to \begingroup". Witnesses: memman
  /// (144 → 1001 errors on the sweep-28 binary), biblatex-oxref ×4,
  /// verbatimcopy, dlfltxb.
  #[test]
  fn endlist_without_lx_list_frame() {
    let (stderr, xml) = convert(
      r"\documentclass{memoir}
\begin{document}
\chapter{Test}
\begin{list}{--}{}
\item one
\item two
\end{list}
After.
\end{document}
",
      true,
    );
    // OXIDIZED_DESIGN #180 (P38): `\@trivlist` now opens the list the raw
    // `\list` asked for, so the items are real items and nothing cascades
    // (before P38 this was Perl's one "Attempt to end mode" per list).
    assert!(!stderr.contains("Attempt to close"), "{stderr}");
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<itemize") && xml.matches("<item ").count() == 2,
      "{xml}"
    );
    assert!(xml.contains("one") && xml.contains("two"), "{xml}");
    assert!(xml.contains("After."), "{xml}");
  }
}

mod perfect_kernel_batch52 {
  //! Red/green guards for perfect-kernel batch 52 (sweep 28 tikzlings `hsb`
  //! cluster, the memoir/dpfloat `\csname` runaway, the xspace pending-space
  //! exception, and the l3prg `Until` runaway at end of input). Each test is
  //! the minimal reproduction distilled during triage; the doc-comment names
  //! the ORIGINAL corpus witness (TeX Live doc corpus) whose larger
  //! conversion was vetted separately.
  use super::{
    perfect_kernel_batch46::{convert, error_count},
    perfect_kernel_batch51::convert_with_files,
  };

  /// `\selectcolormodel{rgb}` (xcolor.sty:137-147) sets `\convertcolorsDtrue`
  /// so every later `\definecolor`/`\colorlet` is CONVERTED to the target
  /// model at definition time (`\XC@definecolor`, xcolor.sty:535-537). pgf
  /// only knows rgb/cmy/cmyk/gray (pgfcoregraphicstate.code.tex:133-137), so
  /// an `hsb` colour reaching `\draw[fill=…]` unconverted is
  /// "Unsupported color model" — pdflatex exit 0 with the selection, and
  /// errors without it. RED: the stub `\selectcolormodel` was a no-op (3
  /// errors). Witness: tikzlings/tikzlings-doc (tikzlings-doc.tex:36 +
  /// tikzlings-bears.sty:124, 29× hsb across the animal styles).
  #[test]
  fn selectcolormodel_converts_definecolor() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{tikz}
\selectcolormodel{rgb}
\definecolor{bb}{hsb}{0.1,0.5,0.5}
\begin{document}
\tikz\draw[bb,fill=bb] (0,0)--(1,1);
\textcolor{bb}{text}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Unsupported color model"), "{stderr}");
    assert!(xml.contains("<text") && xml.contains(">text<"), "{xml}");
  }

  /// `\@currbox` is a BOX REGISTER (latex.ltx:17443 `\@next\@currbox
  /// \@freelist`, the free list being `\newbox`es at :424/442); dpfloat.sty
  /// :82-88 keys its per-box store on `\expandafter\string\@currbox`. Perl
  /// (latex_constructs.pool.ltxml:1025) makes it an EMPTY macro, so
  /// `\csname LP:\endcsname`-style lookups hit `\@namedef` with a `\string`
  /// of nothing and the float body plus everything after it was swallowed
  /// into a `\csname` scan (memoir/memman, oxref ×4: 1001 errors).
  /// KNOWN_PERL_ERRORS #115.
  #[test]
  fn currbox_is_a_box_register() {
    let (stderr, xml) = convert(
      r"\documentclass{memoir}
\usepackage{dpfloat}
\newfloat[chapter]{tegresult}{loe}{Typeset Example}
\begin{document}
Before float.
\begin{tegresult}
Inside custom float.
\end{tegresult}
SWALLOWED text one. SWALLOWED text two. SWALLOWED text three.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("between \\csname and \\endcsname"),
      "{stderr}"
    );
    assert!(xml.contains("Inside custom float."), "{xml}");
    assert!(xml.contains("SWALLOWED text three."), "{xml}");
  }

  /// xspace.sty:49 lists `\@sptoken` — a pending SPACE token — among the
  /// exceptions, so `\bazA[x] and` (the space after a `]`-delimited argument
  /// survives) gets exactly one space. Perl's @XSPACES compares the literal
  /// CS `\@sptoken` and doubles it (KNOWN_PERL_ERRORS #116). Surfaced by the
  /// batch-51 non-space-skipping `\new@ifnextchar` through glossaries
  /// `\gls{potato} and` (structure/glossary golden).
  #[test]
  fn xspace_pending_space_token_is_an_exception() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{xspace}
\def\bazA[#1]{baz#1\xspace}
\begin{document}
D \bazA[x] and E \bazA[x]{} and G.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("D bazx and E bazx and G."), "{xml}");
    assert!(!xml.contains("bazx  and"), "{xml}");
  }

  /// A delimited scan that runs off the TRUE end of all input is reported
  /// ONCE and then ends the job: tex.web §338 abandons the macro call after
  /// the "File ended while scanning" report and §360 aborts with no `\end`
  /// left to find (pdflatex: one report, emergency stop). Perl hands the
  /// caller an empty argument instead, so l3prg's self-recursive
  /// `\prg_map_break:Nn` (expl3-code.tex:2452-2458; reached when
  /// `\prop_map_inline:cn` names an undefined prop) re-misses to the error
  /// cap (Perl 100 → too_many_errors; ours 513 under tikz's raised
  /// `MAX_ERRORS`). Witness: stex/stex-doc.
  #[test]
  fn until_miss_at_eof_is_fatal_once() {
    let (stderr, _xml) = convert(
      r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\prop_map_inline:cn { l_no_such_prop_xyz } { [#1=#2] }
\ExplSyntaxOff
\begin{document}
text
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(
      stderr.contains("Missing argument Until:\\prg_break_point:Nn at end of input"),
      "{stderr}"
    );
    assert!(stderr.contains("Fatal:Mouth:EoF"), "{stderr}");
  }

  /// `\input{pkg.sty}` of a package an earlier `\usepackage` already
  /// raw-loaded must READ THE FILE AGAIN — real TeX's `\input` always reads,
  /// and Perl re-reads it too (Package.pm:2270 `loadTeXDefinitions
  /// (reloadable=>1)`). RED: the binding-only reloadable probe in `input()`
  /// returned a silent `Ok` for a `_raw_loaded` file, so nothing was read
  /// and tikzlings-doc's `\tex_input:D` comment harvester
  /// (tikzlings-doc.tex:60-72, `\__tikzlings_process_line:w #1^^M` under
  /// `\c_other_cctab`) scanned the enclosing document to its end.
  #[test]
  fn input_rereads_loaded_sty() {
    let (stderr, xml) = convert_with_files(
      r"\documentclass{article}
\usepackage{mypkg}
\begin{document}
A\input{mypkg.sty}B
\end{document}
",
      &[("mypkg.sty", "\\typeout{MYPKG READ}\n")],
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(stderr.matches("MYPKG READ").count(), 2, "{stderr}");
    assert!(xml.contains("<p>A\nB</p>"), "{xml}");
  }

  /// latex.ltx:19677-19716 `\declare@file@substitution{orig}{repl}` makes
  /// `\input{orig}` read `repl` (applied by `\set@curr@file`,
  /// latex.ltx:19794); `\undeclare@file@substitution` restores the original.
  /// pdflatex: `[REPLACEMENT ] [REPLACEMENT ] [ORIGINAL ]`. Perl's `\input`
  /// binding bypasses `\set@curr@file` and re-reads the original.
  /// Witness: tikzlings-doc.tex:73-75 substitutes each animal `.sty` by the
  /// `\jobname.cif` comment harvest and `\input`s it — without the
  /// substitution the raw `.sty` re-executes in the body and the per-animal
  /// documentation is lost.
  #[test]
  fn input_honors_file_substitution() {
    let (stderr, xml) = convert_with_files(
      r"\documentclass{article}
\begin{document}
\makeatletter
\declare@file@substitution{orig.tex}{repl.tex}
\makeatother
[\input{orig}]
[\input{orig.tex}]
\makeatletter
\undeclare@file@substitution{orig.tex}
\makeatother
[\input{orig}]
\end{document}
",
      &[("orig.tex", "ORIGINAL\n"), ("repl.tex", "REPLACEMENT\n")],
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
      flat.contains("[REPLACEMENT ] [REPLACEMENT ] [ORIGINAL ]"),
      "{flat}"
    );
  }

  /// xcolor.sty:1033-1036: the plural `\extractcolorspecs{c}{\m}{\s}` stores
  /// the BARE spec (`1,0,0`), unlike the singular `\extractcolorspec`
  /// (`{rgb}{1,0,0}`), so `\definecolor{x}{\m}{\s}` round-trips. Perl's
  /// xcolor.sty.ltxml:808 braces the plural too (KNOWN_PERL_ERRORS #117;
  /// witness pgfPT.colorSchemes.info).
  #[test]
  fn extractcolorspecs_plural_is_unbraced() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\definecolor{src}{rgb}{0.5,0.25,0}
\extractcolorspecs{src}{\m}{\s}
[\m;\s]
\definecolor{dst}{\m}{\s}
\textcolor{dst}{X}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[rgb;0.5,0.25,0]"), "{xml}");
    assert!(xml.contains("color=\"#804000\""), "{xml}");
  }

  /// tagpdf.sty:1594-1665 fills `\g__tag_role_NS_pdf_prop` from
  /// `tagpdf-ns-pdf.def` with `{role}{}` per tag; the manual
  /// (tagpdf.tex:2161-2200) lists the standard structure names by iterating
  /// it. The binding must populate the same tables from the TL data files
  /// rather than leave the props undefined.
  #[test]
  fn tagpdf_role_namespace_props_are_populated() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{tagpdf}
\begin{document}
\ExplSyntaxOn
\clist_clear:N\l_tmpa_clist
\prop_map_inline:cn {g__tag_role_NS_pdf_prop}
  { \str_if_eq:eeT {#1} {\use_i:nn #2} { \clist_put_right:Nn \l_tmpa_clist {#1} } }
[\clist_use:Nn \l_tmpa_clist {,\c_space_tl}]
\clist_clear:N\l_tmpa_clist
\prop_map_inline:cn { g__tag_role_NS_pdf_prop }
  { \prop_if_in:cnF { g__tag_role_NS_pdf2_prop } {#1} { \clist_put_right:Nn \l_tmpa_clist {#1} } }
[\clist_use:Nn \l_tmpa_clist {,\c_space_tl}]
\ExplSyntaxOff
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
      flat.contains("[StructTreeRoot, Document, Part, Sect, Div, Caption,"),
      "{flat}"
    );
    // PDF 1.7 names dropped by the PDF 2.0 namespace (tagpdf-ns-pdf2.def).
    assert!(
      flat.contains("[Art, BlockQuote, TOC, TOCI, Index, Private,"),
      "{flat}"
    );
  }
}

mod perfect_kernel_batch53 {
  //! Red/green guards for perfect-kernel batch 53 (sweep 28 KOMA cluster:
  //! scrkbase font elements, `\DeclareSectionCommand` family, tocbasic).
  //! Each test is the minimal reproduction distilled during triage; the
  //! doc-comment names the ORIGINAL corpus witness (TeX Live doc corpus)
  //! whose larger conversion was vetted separately.
  use super::perfect_kernel_batch46::{convert, error_count};

  /// eTeX `\numexpr`/`\dimexpr`/`\glueexpr` factor scanning (etex.ch
  /// `scan_expr`, "Scan a factor f of type o or start a subexpression")
  /// reads the next non-blank token with `get_x_token` and `back_input`s it
  /// unless it is `(`. `back_input` re-inserts `cur_tok = cs_token_flag +
  /// cur_cs` — the PLAIN control sequence — so a `\noexpand`'d macro at the
  /// head of a factor loses its `no_expand_flag` and IS expanded by the
  /// following `scan_int` (pdfTeX-probed: `\count255=\numexpr\noexpand\one+1
  /// \relax` → 2, while the plain `\count255=\noexpand\one` → "Missing
  /// number"). tocbasic.sty:2688-2690 relies on this:
  /// `\edef…{\the\numexpr \noexpand\@nameuse{sectiontocdepth}+\@ne\relax}`.
  /// RED: the expression reader unread the `\special_relax`-family token
  /// unchanged, so `\@nameuse` stayed noexpand'd and `\numexpr` warned
  /// "Missing number, treated as zero" (witness: every raw-tocbasic manual —
  /// tikzlings-doc, glossaries-user, the KOMA classes' `\DeclareTOCStyleEntries`
  /// probe). Perl's `readXToken` returns a bare `\special_relax` and warns
  /// the same way; this is the noexpand-identity fidelity refinement already
  /// recorded at the `\dont_expand` site in gullet.rs.
  #[test]
  fn numexpr_factor_reexpands_noexpanded_macro() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\makeatletter
\@namedef{sectiontocdepth}{1}
\edef\x{\the\numexpr \noexpand\@nameuse{sectiontocdepth}+\@ne\relax}
\def\one{1}
\count255=\numexpr(\noexpand\one+1)*2\relax
\dimen0=\dimexpr\noexpand\one pt+1pt\relax
\makeatother
\begin{document}
A\x.B\the\count255.C\the\dimen0.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Missing number"), "{stderr}");
    assert!(xml.contains("A2.B4.C2.0pt."), "{xml}");
  }

  /// `\IfFormatAtLeastTF` is a REAL definition in the latex.ltx dump
  /// (`\@ifl@t@r\fmtversion`, latex.ltx L18405) — the always-true stub in
  /// latex_constructs_rust_only.rs (issue #739, witnesses 2408.03197 /
  /// 2408.04893, from before the dump carried it) shadowed it, so
  /// scrbase.sty's `\IfLTXAtLeastTF{<KOMA year+2>/…}` (scrartcl.cls
  /// L2028-2035) fired "Your are using a KOMA-Script version, that has not
  /// been tested" on every KOMA document. RED: `{2099/01/01}` → Y.
  #[test]
  fn ifformatatleast_compares_real_fmtversion() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\begin{document}
A=\IfFormatAtLeastTF{2099/01/01}{Y}{N}.
B=\IfFormatAtLeastTF{2020/01/01}{Y}{N}.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("A=N."), "{xml}");
    assert!(xml.contains("B=Y."), "{xml}");
  }

  /// latex.ltx `\@sect` compares the `\@startsection` level with
  /// `\ifnum #2>\c@secnumdepth` — a TeX <number>. scrartcl.cls L3421/L3425
  /// pass every heading's level as `{\numexpr #2\relax}` (`#2` =
  /// `\csname <name>numdepth\endcsname`). RED: the level was string-parsed
  /// (Perl's `$level > …` coercion → 0), so `\paragraph` (level 4 >
  /// secnumdepth 3) got NUMBERED under every raw KOMA class, and a
  /// `\DeclareSectionCommand` heading with an unknown type opened a warned
  /// `ltx:section` regardless of its level (witness tudaexercise
  /// `\DeclareNewSectionCommand[level=2]{task}`: `<section xml:id="task1">`
  /// plus `Warning:malformed:ltx:task`). The unknown type is now bound to
  /// the element of its level (`SECTION_ELEMENT` mapping, OXIDIZED_DESIGN #175).
  #[test]
  fn startsection_level_is_a_tex_number() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\makeatletter
\newcounter{deep}\def\deepnumdepth{4}
\newcommand\deep{\@startsection{deep}{\numexpr\deepnumdepth\relax}{\z@}{1ex}{1ex}{\bfseries}}
\newcounter{task}[section]\renewcommand\thetask{\thesection.\arabic{task}}
\newcommand\task{\@startsection{task}{\numexpr 2\relax}{\z@}{1ex}{1ex}{\bfseries}}
\makeatother
\begin{document}
\section{S}
\task{T}
\deep{D}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("malformed"), "{stderr}");
    assert!(
      xml.contains(r#"<subsection inlist="toc" xml:id="S1.task1">"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<title><tag close=" ">1.1</tag>T</title>"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<paragraph inlist="toc" xml:id="deepx1">"#),
      "{xml}"
    );
    assert!(xml.contains("<title>D</title>"), "{xml}");
  }

  const KOMA_TASK: &str = r"\documentclass{scrartcl}
\DeclareNewSectionCommand[style=section,level=2,counterwithin=section,tocstyle=section,indent=0pt,tocindent=1.5em,tocnumwidth=2.3em,beforeskip=1ex,afterskip=1ex,font=\bfseries]{task}
\begin{document}
\section{One}
\subsection{Sub}
\task{A task}
Body.
\paragraph{Para} text.
\end{document}
";

  /// Raw scrartcl (host TeX Live; the class binding is a raw shim since
  /// batch 53, `scrartcl_cls.rs`): `\DeclareNewSectionCommand[level=2]{task}`
  /// opens a `<subsection>` and the raw class's `\paragraph` is unnumbered.
  /// Witness: tudaexercise (DEMO-TUDaExercise), tikzlings-doc.
  #[test]
  fn koma_declaresectioncommand_heading_is_a_subsection() {
    let (stderr, xml) = convert(KOMA_TASK, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("malformed"), "{stderr}");
    assert!(
      xml.contains(r#"<subsection inlist="toc" xml:id="task1">"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<title><tag close=" ">1.1</tag>A task</title>"#),
      "{xml}"
    );
    assert!(
      xml.contains(
        r#"<paragraph inlist="toc" xml:id="section1.subsection1.subsubsection0.paragraphx1">"#
      ),
      "{xml}"
    );
    assert!(xml.contains("<title>Para</title>"), "{xml}");
  }

  /// Raw scrkbase font-element API (scrkbase.sty L452-670): `\newkomafont`
  /// registers the element, `\usekomafont` EXPANDS to its switches; a later
  /// `\usepackage{scrlayer-scrpage}` (→ scrlayer.sty L81 `\RequirePackage
  /// {scrkbase}`) must not re-load anything that forgets the element. RED:
  /// the scrartcl stub's no-op `\newkomafont` registered nothing, so the
  /// real `\usekomafont` died with "font element myel not defined"
  /// (witness contract-example-de/en).
  #[test]
  fn newkomafont_survives_scrlayer_scrpage_and_usekomafont_expands() {
    let (stderr, xml) = convert(
      r"\documentclass{scrartcl}
\newkomafont{myel}{\itshape}
\setkomafont{section}{\Large\bfseries}
\addtokomafont{title}{\rmfamily}
\usepackage{scrlayer-scrpage}
\begin{document}
Text {\usekomafont{myel}elem} plain.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<text font="italic">elem</text> plain."#),
      "{xml}"
    );
  }

  /// KOMA title-page pieces (scrartcl.cls L2768-2803) are stored for the
  /// class's own `\maketitle` (L2815), which is a locked constructor here;
  /// `koma_script.rs` re-targets them at the frontmatter (witness 2305.01582
  /// `\titlehead`; ar5iv #498).
  #[test]
  fn koma_title_pieces_reach_frontmatter() {
    let (stderr, xml) = convert(
      r"\documentclass{scrartcl}
\titlehead{Head}\subject{Subj}\subtitle{Sub}\title{Title}\author{A. U. Thor}\date{2026}\publishers{Pub}
\begin{document}
\maketitle
Body.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<subtitle>Sub</subtitle>"), "{xml}");
    assert!(
      xml.contains(r#"<note role="titlehead">Head</note>"#),
      "{xml}"
    );
    assert!(xml.contains(r#"<note role="subject">Subj</note>"#), "{xml}");
    assert!(
      xml.contains(r#"<note role="publishers">Pub</note>"#),
      "{xml}"
    );
    assert!(xml.contains("<title>Title</title>"), "{xml}");
  }

  /// Raw typearea (`typearea_sty.rs` is a raw shim since batch 53). RED: the
  /// former typearea STUB left `\if@areasetadvanced` undefined, and
  /// scrartcl.cls L2594-2628 tests it inside a skipped `\if…\else…\fi` branch
  /// — tex.web `pass_text` only counts `if_test` commands, so an undefined
  /// `\if@…` in the skipped text is not a conditional and its `\else`
  /// terminated the OUTER skip ("Too many }'s" / "Extra \fi", exactly as
  /// pdftex does with the same undefined `\if`). `\recalctypearea` /
  /// `\areaset` exercised the same class code at body time (witness
  /// bohr/bohr_en; arXiv 1502.06768, 1504.00554, 1504.00666).
  #[test]
  fn raw_typearea_defines_areaset_conditionals() {
    let (stderr, xml) = convert(
      r"\documentclass[11pt,DIV=12]{scrartcl}
\begin{document}
A\recalctypearea B\areaset{10cm}{20cm}C
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("unexpected:"), "{stderr}");
    assert!(
      xml.contains("A") && xml.contains("B") && xml.contains("C"),
      "{xml}"
    );
  }

  /// tikzlings-doc: `\RedeclareSectionCommand` + tocbasic's `\deftocheading`
  /// on a raw scrartcl (both `undefined:` under the former stub).
  #[test]
  fn koma_redeclaresectioncommand_and_deftocheading() {
    let (stderr, xml) = convert(
      r"\documentclass{scrartcl}
\RedeclareSectionCommand[beforeskip=1ex,afterskip=1ex]{section}
\deftocheading{toc}{\section*{##1}}
\begin{document}
\tableofcontents
\section{One}
Body.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<title><tag close=" ">1</tag>One</title>"#),
      "{xml}"
    );
  }

  /// l2tabu/l2tabuen: `\@declaredoptions` must expand to the declared option
  /// list (latex.ltx L18536 `\xdef\@declaredoptions{\@declaredoptions,#1}`;
  /// the Perl pool L784 binds it EMPTY). scrbase.sty L365 walks it after
  /// `\FamilyProcessOptions` to retire every `\ds@<opt>`; with an empty list
  /// the `\ds@` of a KOMA deprecated option (scrkbase.sty L365-407
  /// `\KOMA@DeclareDeprecatedOption`) survived into typearea's own
  /// `\KOMAProcessOptions` (typearea.sty L1053), which re-ran it as
  /// "unknown option `captions=tableheading'".
  #[test]
  fn declaredoptions_lists_declared_options() {
    let (stderr, xml) = convert(
      r"\documentclass[tablecaptionabove]{scrartcl}
\makeatletter
\DeclareOption{alpha}{}\DeclareOption{beta}{}
\edef\x{\@declaredoptions}
\makeatother
\begin{document}
[\x]
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("unknown option"), "{stderr}");
    assert!(xml.contains("alpha,beta]"), "{xml}");
  }

  /// DEMO-TUDaPhD/TUDaThesis: `\@classoptionslist` / `\@raw@classoptionslist`
  /// / `\@raw@opt@<file>` carry standard catcodes (Perl Package.pm L2564
  /// `DefMacroI` string body → TokenizeInternal; the kernel stores the real
  /// argument tokens). With every character OTHER, tudapub.cls L173/L358
  /// forwarded an unknown option value to `\KOMAoption{parskip}{half-}` and
  /// scrbase.sty L2354 `\FamilySetNumerical`'s `\ifx` against scrbook.cls
  /// L825's literal `half-` failed ("unknown value"). The `\ifx` here is the
  /// same comparison; the braced value checks that `{`/`}` group.
  #[test]
  fn classoptionslist_has_letter_catcodes() {
    let (stderr, xml) = convert(
      r"\documentclass[parskip=half-,thesis={type=dr,dr=rernat}]{article}
\begin{document}
\makeatletter
\def\lit{parskip=half-,thesis={type=dr,dr=rernat}}
\ifx\lit\@classoptionslist [same]\else [DIFFERENT]\fi
\ifx\lit\@raw@classoptionslist [rawsame]\else [rawDIFFERENT]\fi
\makeatother
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[same]") && xml.contains("[rawsame]"), "{xml}");
  }

  /// tutodoc-en/fr: a `\def` parameter text built by expansion keeps BOTH of
  /// two adjacent space tokens in its delimiter (tex.web §473-476); Perl
  /// TeX_Macro.pool.ltxml L127 collapsed them (KNOWN_PERL_ERRORS #119), so
  /// expkv's `\ekv@set@was@blank` delimiter (two real spaces) never matched
  /// and `\ekvset{clrstrip}{}` ran away to `Timeout:TokenLimit`.
  #[test]
  fn def_delimiter_keeps_adjacent_spaces() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\makeatletter
\def\A{}\def\B{}\def\SP{ }
\protected@edef\deltoks{\noexpand\A\SP\SP\noexpand\B}
\expandafter\def\expandafter\x\expandafter#\expandafter1\deltoks{[GOT:#1]OK}
\makeatother
\begin{document}
Before.
\expandafter\x\expandafter Q\deltoks
After.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(xml.contains("[GOT:Q]OK"), "{xml}");
  }

  /// The expkv shape of the same defect: an EMPTY key list goes through
  /// `\ekv@set@was@blank` (expkv.tex L709-712).
  #[test]
  fn expkv_blank_entry_does_not_leak_markers() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{expkv}
\ekvdef{foo}{bar}{[V=#1]}
\begin{document}
X\ekvset{foo}{}Y\ekvset{foo}{bar=1, ,}Z
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(xml.contains("XY[V=1]Z"), "{xml}");
  }

  /// Convert `t.tex` next to a sidecar package file under the perfect-kernel
  /// preload; `--includestyles --path .` makes the sidecar raw-loadable.
  pub(super) fn convert_with_sty(tex: &str, sty_name: &str, sty_body: &str) -> (String, String) {
    use std::{path::Path, process::Command};
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    std::fs::write(workdir.path().join(sty_name), sty_body).expect("write sidecar sty");
    let output = Command::new(bin)
      .args([
        "t.tex",
        "--dest",
        "t.xml",
        "--nocomments",
        "--timeout=110",
        "--includestyles",
        "--path",
        ".",
        "--preload=[rawstyles,rawclasses]latexml.sty",
      ])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  /// latex.ltx `\@pass@ptions` (L18509-18526) is the single writer of
  /// `\@raw@opt@<name>.<ext>`, reached from `\PassOptionsToPackage` AND from
  /// `\@onefilewithoptions` for the explicit `[…]` list; ltkeys'
  /// `\ProcessKeyOptions` reads only that record (`\__keys_options_local:`,
  /// L19457-19470). RED: the record was built from the explicit list alone,
  /// so tudapub.cls L194 `\exp_args:Nx \PassOptionsToPackage{paper=…}
  /// {tudarules}` never reached tudarules' `\ProcessKeyOptions[ptxcd/rules]`
  /// and `\c_ptxcd_{large,small}rule_dim` stayed undefined (witness
  /// DEMO-TUDaPhD, DEMO-TUDaThesis). pdflatex: `P=[A5]`.
  #[test]
  fn process_key_options_sees_passed_options() {
    let (stderr, xml) = convert_with_sty(
      r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\keys_define:nn {my/cls} {
  paper .choices:nn = { a4,a5 } {
    \exp_args:Nx \PassOptionsToPackage{paper=\l_keys_choice_tl}{mypk}
  },
}
\keys_set:nn {my/cls} {paper=a5}
\ExplSyntaxOff
\usepackage{mypk}
\begin{document}
P=[\csname g_my_paper_tl\endcsname] C=[\csname g_my_color_tl\endcsname]
\end{document}
",
      "mypk.sty",
      r"\ProvidesPackage{mypk}
\RequirePackage{expl3}
\ExplSyntaxOn
\tl_new:N \g_my_paper_tl
\keys_define:nn {my/rules} {
  paper .choice:,
  paper/a4 .code:n = { \tl_gset:Nn \g_my_paper_tl {A4} },
  paper/a5 .code:n = { \tl_gset:Nn \g_my_paper_tl {A5} },
  color .tl_gset:N = \g_my_color_tl,
  color .initial:n = black,
}
\ProcessKeyOptions[my/rules]
\ExplSyntaxOff
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("P=[A5] C=[black]"), "{xml}");
  }

  /// TeX's `\addcontentsline` (latex.ltx L17351-17363) writes its title to
  /// the .toc through `\protected@write`, never typesetting it, which is
  /// what makes LaTeX's write-only self-`\protect` idiom
  /// `\def\appfmt#1{\protect\appfmt{#1}}` (nlctuserguide.sty L1553
  /// `\@loe@disable@cmds`) safe. RED: the constructor digested the (then
  /// discarded) title with `\protect`=`\relax`, so the macro re-expanded to
  /// itself — `Fatal:Timeout:Recursion` (9-token window) or `TokenLimit`
  /// (13-token, past the cycle guard's window). Witness glossaries-user
  /// examples `ex:xdy`/`ex:mkidx`; Perl 0.8.8 hangs on this repro
  /// (KNOWN_PERL_ERRORS #120).
  #[test]
  fn addcontentsline_title_is_not_digested() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\newcommand*{\appfmt}[1]{\texttt{#1}}
\makeatletter
\begin{document}
\def\thetitle{uses \appfmt{xindy}}%
\def\appfmt#1{\protect\appfmt{#1}}% \@loe@disable@cmds idiom
\addcontentsline{toc}{section}{\thetitle}%
done\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(xml.contains(">done<"), "{xml}");
  }

  /// latex.ltx L18297-18300 defines `\pagestyle` as a plain `\def`;
  /// scrlayer.sty L2183-2196 redefines it with the triple-`\expandafter`
  /// freeze that inlines the OLD body at definition time. RED: `\pagestyle`
  /// was a non-expandable primitive no-op, so the literal `\pagestyle{#1}`
  /// survived in the new body and every later call recursed
  /// (`Fatal:Timeout:Recursion`; raw scrlayer: `PushbackLimit` at
  /// `\begin{document}` from `\AtBeginDocument{\pagestyle{test}}`). Perl
  /// 0.8.8 hangs the same way (KNOWN_PERL_ERRORS #121). Witnesses
  /// DEMO-TUDaPhD/TUDaThesis, neoschool, bfh-ci (raw scrlayer-scrpage).
  #[test]
  fn pagestyle_expandafter_freeze_terminates() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\makeatletter
\expandafter\expandafter\expandafter\renewcommand
\expandafter\expandafter\expandafter*%
\expandafter\expandafter\expandafter\pagestyle
\expandafter\expandafter\expandafter[%
\expandafter\expandafter\expandafter1%
\expandafter\expandafter\expandafter]%
\expandafter\expandafter\expandafter{\pagestyle{#1}}%
\makeatother
\begin{document}
\pagestyle{plain}\thispagestyle{empty}
Hello\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(xml.contains(">Hello<"), "{xml}");
  }

  /// Raw `scrlayer-scrpage` on top of raw `scrlayer`/scrkbase: the former
  /// stub defined no KOMA option keys, so a class's
  /// `\KOMAoptions{headwidth=text,footsepline=…}` raised `unknown option`
  /// and `\RedeclareLayer`/`\layerwidth`/`\DeclarePageStyleByLayers` were
  /// undefined (witness DEMO-TUDaPhD, DEMO-TUDaThesis, neoschool, bfh-ci).
  /// Structural: the body paragraph survives `\begin{document}` (where the
  /// old raw load died with `PushbackLimit`).
  #[test]
  fn raw_scrlayer_scrpage_loads_and_sets_keys() {
    let (stderr, xml) = convert(
      r"\documentclass{scrbook}
\usepackage[automark]{scrlayer-scrpage}
\KOMAoptions{headwidth=text,footsepline=.5pt}
\DeclareNewLayer[background,contents={\layerwidth}]{mylayer}
\DeclareNewPageStyleByLayers{mystyle}{mylayer}
\RedeclareLayer[foreground]{mylayer}
\pagestyle{mystyle}
\begin{document}
\chapter{One}
Hello\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(xml.contains(">Hello<"), "{xml}");
  }

  /// xltabular.sty L86-96: the environment restores longtable's
  /// `\caption`/`\endhead`/`\endfirsthead`/`\endfoot`/`\endlastfoot` and runs
  /// `\expandafter\longtable\the\toks@\endlongtable` — it IS a longtable with
  /// `X` columns. RED: the binding aliased `\xltabular` to `\tabularx`, so the
  /// class `\caption` ran inside a tabularx (`Use of \caption outside any
  /// known float`); under raw KOMA the tocbasic expl3 `\caption` then read an
  /// undefined `\@captype` and cascaded into a `TooManyErrors` fatal
  /// (witnesses xltabular-doc 36→101 errors + fatal, hvfloat 27→90). GREEN:
  /// one `<ltx:table>` carrying the caption, a `<thead>` from `\endfirsthead`
  /// and the body rows.
  #[test]
  fn xltabular_is_a_longtable() {
    let (stderr, xml) = convert(
      r"\documentclass{scrartcl}
\usepackage{booktabs,xltabular}
\begin{document}
\begin{xltabular}{\textwidth}{@{} l>{\small\ttfamily}cX @{}}
\caption{The optional keywords}\label{tab:options}\\\toprule
Keyword & Default & Description\\\midrule
\endfirsthead
\midrule
\endhead
\bottomrule
\endfoot
\endlastfoot
onlyText & false & Only the text \\
capPos & b & caption position\\
\end{xltabular}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(
      xml.contains("<table inlist=\"lot\" labels=\"LABEL:tab:options\""),
      "{xml}"
    );
    assert!(xml.contains("The optional keywords</caption>"), "{xml}");
    assert!(xml.contains("<thead>"), "{xml}");
    assert!(xml.contains(">caption position<"), "{xml}");
  }

  /// hvfloat surface the manual exercises (witness hvfloat manual, 79→7
  /// errors; the 7 left are the SHARED `\endflushleft`-in-group mode-frame
  /// family and a SHARED eager `backgroundcolor=\color` evaluation in the
  /// listings binding). hvfloat.sty L630-636: an EMPTY float type is
  /// `nonFloat,onlyText` — object and caption as plain text, no float, no
  /// counter (was `undefined:{}` and a `\caption` outside any float);
  /// L1264-1266 `hvFloatEnv` is a minipage; L24-36 `[fbox,hyperref]` options
  /// are `\newif` switches; L306-307 `\hvDefFloatStyle` = `\defhvstyle`;
  /// L55 `\RequirePackage{ifoddpage}` — pure kernel TeX that now loads raw
  /// in both preload modes (`\checkoddpage`/`\ifoddpage` were `undefined`).
  #[test]
  fn hvfloat_only_text_env_options_and_ifoddpage() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[fbox,hyperref]{hvfloat}
\begin{document}
\hvDefFloatStyle{main}{capPos=r}
\checkoddpage\ifoddpage odd\else even\fi
\hvFloat[onlyText=true]{}{\rule{2cm}{1cm}}{Only text, no float}{txt:only}
\begin{hvFloatEnv}
\rule{1cm}{1cm}
\captionof{figure}{Inside the env}\label{fig:env}
\end{hvFloatEnv}
\hvFloat{figure}{\rule{1cm}{1cm}}[Short]{A real figure}{fig:real}
\makeatletter\hv@fboxfalse\makeatother
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(!stderr.contains("missing_file"), "{stderr}");
    assert!(xml.contains(">odd<") || xml.contains("odd\n"), "{xml}");
    assert!(xml.contains("Only text, no float"), "{xml}");
    assert!(
      xml.contains("<figure inlist=\"lof\" labels=\"LABEL:fig:env\""),
      "{xml}"
    );
    assert!(
      xml.contains("<figure inlist=\"lof\" labels=\"LABEL:fig:real\""),
      "{xml}"
    );
    assert!(xml.contains("A real figure</caption>"), "{xml}");
    // The onlyText form opens no float: exactly the two real figures.
    assert_eq!(xml.matches("<figure ").count(), 2, "{xml}");
  }

  /// collcell binding (witness onedown-ref.tex:469 `\begin{bidding}`,
  /// onedown.sty:1326 `>{\collectcell\ODw@BTfer}c<{\endcollectcell}`): the
  /// raw `\collectcell#1#2\ignorespaces` (collcell.sty:76) is delimited by the
  /// kernel cell template's `\ignorespaces` (latex.ltx:16671-16675), which
  /// LaTeXML's alignment never inserts — the scan ran to end of input and
  /// Rust's Until-at-EOF Fatal lost the whole 500-line manual (39-byte XML;
  /// Perl: 6 errors, completes). The binding collects the cell unexpanded like
  /// `\collect@cell@look`, letting a `\\`/`\tabularnewline` row end expand so
  /// the `<{\endcollectcell}` column-after template fires (a plain
  /// `Until:\endcollectcell` swallowed the rest of the document whenever the
  /// collected column was the LAST of the row); `\cci{…}` is skipped
  /// (onedown.sty:1338 bidding headers).
  #[test]
  fn collcell_hands_cell_to_macro() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{array}
\usepackage{collcell}
\newcommand\Xfer[1]{[#1]}
\newcolumntype{B}{>{\collectcell\Xfer}c<{\endcollectcell}}
\begin{document}
\begin{tabular}{BB}
\cci{ West} & \cci{ North} \\
1S & 2 H \\[2pt]
{pass} \textbf{x} & 3NT \tabularnewline
a & b \cr
\end{tabular}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    // `\cci{ West}` hands the macro the braced group as is, space included
    // (onedown's "there MUST be a ' '" first-token probe relies on it).
    for cell in [
      "[ West]", "[ North]", "[1S]", "[2 H]", "[3NT]", "[a]", "[b]",
    ] {
      assert!(xml.contains(&format!(">{cell}</td>")), "{cell}: {xml}");
    }
    assert!(
      xml.contains(">[pass <text font=\"bold\">x</text>]</td>"),
      "{xml}"
    );
    assert_eq!(xml.matches("<td ").count(), 8, "{xml}");
  }

  /// The `\opt@<file>` macro (latex.ltx `\@pass@ptions`, `\@ptionlist`)
  /// holds the ARGUMENT tokens of `\usepackage[...]`, so `{`/`}` group. Ours
  /// was rebuilt with `ExplodeText!` (braces OTHER), and every brace-aware
  /// consumer of `\@ptionlist` split a braced value at its inner comma:
  /// l3clist (`\clist_set:cx {…}{\@ptionlist{…}}`, URspecialopts → tudapub
  /// lost `thesis={type=dr,dr=rernat}`, so DEMO-TUDaPhD never input
  /// tudathesis.cfg: `\department`/`\affidavit` undefined) and xkeyval's
  /// `\ProcessOptionsX` (glossaries-extra.sty:811 `\@for` read
  /// `stylemods={mcols,bookindex}` as the style file `glossary-{mcols.sty`;
  /// witness glossaries-user). Perl: 0 errors. pdflatex: `I=[mcols][bookindex]
  /// T=[type=dr,dr=rernat]`.
  #[test]
  fn opt_macro_keeps_braced_option_values() {
    let (stderr, xml) = convert_with_sty(
      r"\documentclass{article}
\usepackage[english,stylemods={mcols,bookindex},thesis={type=dr,dr=rernat}]{mypk}
\begin{document}
I=[\csname my@items\endcsname] T=[\csname g_my_thesis_tl\endcsname]
\end{document}
",
      "mypk.sty",
      r"\ProvidesPackage{mypk}
\RequirePackage{expl3,xkeyval}
\ExplSyntaxOn
\tl_new:N \g_my_thesis_tl
\cs_new:Npn \my_kv:nn #1#2 { \tl_gset:Nn \g_my_thesis_tl {#2} }
\clist_set:Nx \l_tmpa_clist {\@ptionlist{mypk.sty}}
\clist_map_inline:Nn \l_tmpa_clist {
  \tl_if_in:nnT {#1} {thesis=} { \keyval_parse:NNn \use_none:n \my_kv:nn {#1} }
}
\ExplSyntaxOff
\def\my@items{}
\define@key{mypk.sty}{stylemods}{\@for\my@tmp:=#1\do{\edef\my@items{\my@items[\my@tmp]}}}
\define@key{mypk.sty}{thesis}{}
\define@key{mypk.sty}{english}[]{}
\ProcessOptionsX
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("I=[[mcols][bookindex]] T=[type=dr,dr=rernat]"),
      "{xml}"
    );
  }

  /// tcbxparse's `\NewTCBListing{code}{ O{} m !O{} !O{x} !O{y} }`
  /// (neoschool.cls:5168) takes ONE mandatory argument and, absent a `[`,
  /// no optionals; our delegate counted every specifier as a mandatory
  /// `\lstnewenvironment` argument, so a bare `\begin{code}{latex}` grabbed
  /// body tokens through its own `\end`, the verbatim scan ran to the NEXT
  /// `\end{code}` and swallowed the `\begin{sidebyside}` in between —
  /// tcolorbox's global layer counter (tcolorbox.sty:1411 `\tcb@layer@inc`
  /// never run for the eaten begin, `\tcb@layer@dec` run at its end) went
  /// negative and every later box errored `every box on layer 0/-N`
  /// (witness neoschool, 251 of 273 errors; Perl 0; pdflatex clean).
  #[test]
  fn tcb_listing_trailing_optionals_not_mandatory() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage[most]{tcolorbox}\usepackage{listings}
\NewTCBListing{code}{ O{} m !O{} !O{x} !O{y} }{listing only}
\newtcolorbox{sidebyside}[1][]{sidebyside,enhanced,bicolor,#1}
\begin{document}
\begin{code}{latex}
xx
\end{code}
\begin{sidebyside}[righthand width=.5\linewidth]
\begin{code}[numbers=none]{latex}
inner
\end{code}
\tcblower l\end{sidebyside}
\begin{sidebyside}u2\tcblower l2\end{sidebyside}
\end{document}
",
      false,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("every box on layer"), "{stderr}");
    // Two separate listings: the first one ends at ITS `\end{code}`.
    assert_eq!(xml.matches("<listing ").count(), 2, "{xml}");
    assert!(
      !xml.contains("begin{sidebyside}"),
      "listing swallowed the box: {xml}"
    );
    assert!(xml.contains("u2"), "{xml}");
  }

  /// latex.ltx L1185-1188: `\typeout` writes its argument under
  /// `\set@display@protect` (L1438 `\let\protect\string`), so a robust
  /// command is written by NAME. Expanding it with `\protect`=`\relax`
  /// entered raw KOMA's `\DeclareRobustCommand\small` (scrsize10pt.clo:62),
  /// whose `\@setfontsize\small…` re-expands `\small` without end — the same
  /// overflow pdflatex gives `\edef\x{\small}` — from hvextern.sty:325
  /// `\hv@ex@typeout{Running BodyVerbatim with fontsize=\small,…}`
  /// (witness hvextern manual: `Fatal:Timeout:PushbackLimit`; Perl 0, its
  /// `\small` being a primitive). pdflatex writes exactly
  /// `SIZE:[fontsize=\small \add@extra@listi{sml},fontfamily=tt]`.
  #[test]
  fn typeout_writes_robust_commands_by_name() {
    let (stderr, xml) = convert(
      r"\documentclass{scrartcl}
\begin{document}
\typeout{SIZE:[fontsize=\small,fontfamily=tt]}
x
\end{document}
",
      true,
    );
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(
      stderr.contains("SIZE:[fontsize=\\small \\add@extra@listi{sml},fontfamily=tt]"),
      "{stderr}"
    );
    assert!(xml.contains("<p>x</p>"), "{xml}");
  }

  /// siunitx.sty:5014-5016 loads translations.sty when it exists, and
  /// translations.sty:36 loads etoolbox — that is how a class using siunitx
  /// early has `\AtEndPreamble` (neoschool.cls:1123). The binding required
  /// neither, so `\AtEndPreamble` was `undefined` (Perl 0, pdflatex clean).
  #[test]
  fn siunitx_loads_translations() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{siunitx}
\AtEndPreamble{\def\marker{HOOKED}}
\begin{document}
\marker
\end{document}
",
      false,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("HOOKED"), "{xml}");
  }

  /// hyperref.sty:3979-3992 defaults `\@pdftitle`/`\@pdfauthor`/… to
  /// `\@empty` and each metadata key runs `\pdfstringdef\@pdf<key>{#1}`
  /// (L3543-3596). nlctuserguide.sty:1630 reads `\@pdfauthor` directly
  /// (witness glossaries-extra-manual.tex:48); Perl leaves it undefined.
  #[test]
  fn hypersetup_defines_pdf_info_macros() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{hyperref}
\makeatletter
\begin{document}
A=[\@pdfauthor]
\hypersetup{pdfauthor={Nicola Talbot},pdftitle={The Title}}
B=[\@pdfauthor/\@pdftitle/\@pdfcreator]
\end{document}
",
      false,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("A=[]"), "{xml}");
    assert!(
      xml.contains("B=[Nicola Talbot/The Title/LaTeX with hyperref]"),
      "{xml}"
    );
  }

  /// xkeyval.tex:496-502: in `\ProcessOptionsX` — star or not — an unknown
  /// option runs the `\DeclareOptionX*` handler `\XKV@doxs` when one is
  /// defined; the star only adds the class options to the scan. The binding
  /// (like Perl xkeyval.sty.ltxml:355, KNOWN_PERL_ERRORS #122) armed the
  /// handler for `\ProcessOptionsX*` only, so the non-star form dropped
  /// `english`/`foo=bar` with two "unknown KeyVals key" warnings. pdflatex:
  /// `E=[[english][foo=bar]] W=[3cm]`.
  #[test]
  fn processoptionsx_unknown_option_reaches_star_handler() {
    let (stderr, xml) = convert_with_sty(
      r"\documentclass{article}
\usepackage[english,width=3cm,foo=bar]{mypk}
\makeatletter
\begin{document}
E=[\my@extra] W=[\my@width]
\end{document}
",
      "mypk.sty",
      r"\ProvidesPackage{mypk}
\RequirePackage{xkeyval}
\def\my@extra{}
\define@key{mypk.sty}{width}{\def\my@width{#1}}
\DeclareOptionX*{\edef\my@extra{\my@extra[\CurrentOption]}}
\ProcessOptionsX
",
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("unknown KeyVals key"), "{stderr}");
    assert!(xml.contains("E=[[english][foo=bar]] W=[3cm]"), "{xml}");
  }
}

mod perfect_kernel_batch54 {
  //! Red/green guards for perfect-kernel batch 54 (wave-4 root-causer
  //! reports over the sweep-28 residuals). Each test is the minimal
  //! reproduction distilled during triage; the doc-comment names the
  //! ORIGINAL corpus witness (TeX Live doc corpus) whose larger conversion
  //! was vetted separately.
  use super::{
    perfect_kernel_batch40_43::convert_with_files,
    perfect_kernel_batch46::{convert, convert_with, error_count},
    perfect_kernel_batch53::convert_with_sty,
  };

  /// biblatex.sty:4407-4425 defines `\DeclareIndex{Name,List,Field}Format`
  /// through the same `\blx@defformat` as their non-Index siblings, and
  /// :14133 `\DeclareDriverSourcemap[2][]`. The native binding no-ops the
  /// siblings but omitted these, so an undefined-CS stub (zero args) left
  /// each declaration BODY in the document: `#1` reached the Stomach
  /// (`misdefined:#`) and `\nameparts`/`\usebibmacro`/`\actualoperator`/
  /// `\map`/`\step` fired as undefined (cnltx.bbx:131-210; witnesses
  /// cnltx_en, endiagram_en, chemformula-manual — 7 `#` + 11 undefined each).
  #[test]
  fn bbx_declaration_bodies_are_absorbed() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{biblatex}
\DeclareIndexFieldFormat[package]{title}{#1}
\DeclareIndexListFormat{cnltx}{#1}
\DeclareIndexNameFormat{cnltx}{\nameparts{#1}\usebibmacro{index:entry}{#1}\actualoperator}
\DeclareDriverSourcemap[datatype=bibtex]{\map{\step[fieldsource=info, fieldtarget=subtitle]}}
\begin{document}
Hello.\usebibmacro*{index:entry}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("misdefined"), "{stderr}");
    assert!(
      !xml.contains("ltx:ERROR") && !xml.contains("<ERROR"),
      "{xml}"
    );
    assert!(!xml.contains("[package]title"), "{xml}");
    assert!(xml.contains("<p>Hello.</p>"), "{xml}");
  }

  /// biblatex.sty:9436 `\defbibcheck[2]`, :7029 `\DeclareRedundantLanguages[2]`
  /// and :9784 `\printbibheading` (one `\@ifnextchar[` optional). Undefined
  /// `\defbibcheck` (arthistory-bonn.bbx:199) leaked its check body, whose
  /// `\ifcsdef{\strfield{series}}` mis-nested into a live `\iffalse` that
  /// scanned to the .bbx end of file (`expected:\fi`), eating the document's
  /// `\printbibheading` (witness rub-kunstgeschichte-example: 4 errors → 0).
  #[test]
  fn biblatex_check_and_heading_commands_absorb_args() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{biblatex}
\DeclareRedundantLanguages{german}{german,ngerman}
\defbibcheck{shortseries}{\iffieldundef{series}{\skipentry}{\ifcsdef{\strfield{series}}{\skipentry}{}}}
\begin{document}
A.\printbibheading[title=Works]B.\printbibheading C.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("iffalse"), "{stderr}");
    assert!(xml.contains("A.B.C."), "{xml}");
  }

  /// `\usetikzlibrary{hobby}` → tikzlibraryhobby.code.tex:16 → pgflibraryhobby
  /// .code.tex:16 `\input{hobby.code.tex}`. A `Warn!`-only refusal stub
  /// (`hobby_code_tex.rs`, added for arXiv 2111.02755 "until our LaTeX3 support
  /// is ready") intercepted that `\input`, so `\hobbyVersion`/`\hobbyDate`
  /// (hobby.code.tex:36/40) and `\hobbyinit` (:668) were undefined and the
  /// zero-arg ERROR stub for `\hobbyinit` left `\curvethrough`'s
  /// (tikzlibraryhobby.code.tex:210) `\relax`-delimited point scan to run to
  /// end of input (witness hobby/hobby: 5 errors + EoF Fatal). The real file
  /// (expl3 + pml3array) now raw-loads clean; the stub is retired.
  #[test]
  fn hobby_code_tex_raw_loads() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{hobby}
\begin{document}
V=\hobbyVersion\ from \hobbyDate.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("hobby.code.tex is not implemented"),
      "{stderr}"
    );
    assert!(xml.contains("V=1.12 from 2023-09-01."), "{xml}");
  }

  /// tex.web §373: a non-character CS inside `\csname…\endcsname` is
  /// "Missing \endcsname inserted" via `back_error` — the name ENDS there and
  /// the offending token is re-read after the constructed CS. The gullet
  /// reported the error but kept scanning to the real `\endcsname`, so every
  /// token in between was lost (witness tikzpingus-doc: `\csname …\relax…`
  /// style key builders dropped their content). Two errors remain, exactly
  /// real TeX's: "Missing \endcsname inserted" then "Extra \endcsname"
  /// (tex.web §1135) for the orphaned closer.
  #[test]
  fn csname_missing_endcsname_reinserts_after_offender() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\begin{document}
X\csname foo\relax bar\endcsname Y
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 2, "{stderr}");
    assert!(stderr.contains("Extra \\endcsname"), "{stderr}");
    assert!(xml.contains("bar"), "{xml}");
    assert!(xml.contains("Y"), "{xml}");
  }

  /// The forest binding replaces the raw forest.sty, whose :1
  /// `\ProvidesPackage{forest}` is what `\@ifpackageloaded{forest}` in
  /// dependants keys on (forest-doc preamble, neoschool.cls).
  #[test]
  fn forest_binding_registers_as_loaded() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{forest}
\begin{document}
\makeatletter\@ifpackageloaded{forest}{LOADED}{ABSENT}\makeatother
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("LOADED") && !xml.contains("ABSENT"), "{xml}");
  }

  /// xltabular.sty:19-21 user toggles `\normalLTpagebreak`/`\specialLTpagebreak`
  /// (page-break policy only) were dropped by the binding that replaces the
  /// raw .sty (witness xltabular-doc: 2 undefined).
  #[test]
  fn xltabular_pagebreak_toggles() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{xltabular}
\begin{document}
Special: \specialLTpagebreak Normal: \normalLTpagebreak Done.
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Special: Normal: Done."), "{xml}");
  }

  /// memoir.cls:8811 redefines only `\endminipage` (raw latex.ltx closer);
  /// tcolorbox `\let\endtcb@lrbox=\endminipage` (tcolorbox.sty:1118) then
  /// closed our NATIVE minipage with it: the live dump `\@iiiparbox` got an
  /// undefined `\@mpargs` and its `Until:[` scan ate the next box's option
  /// list (witness biblatex-oxref/oxalph-doc: 983× `\csname bm@bicolor,…`
  /// + Fatal TooManyErrors). The binding now keeps the native pair paired.
  #[test]
  fn memoir_keeps_native_endminipage() {
    let (stderr, xml) = convert(
      r"\documentclass[oneside]{memoir}
\usepackage{tcolorbox}
\begin{document}
\begin{tcolorbox}[colframe=red]
A\par B\tcblower C
\end{tcolorbox}
\begin{tcolorbox}[colframe=blue]
D
\end{tcolorbox}
\end{document}
",
      true,
    );
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(!stderr.contains("bm@"), "{stderr}");
    assert!(xml.contains("ltx_minipage"), "{xml}");
    assert!(xml.contains(">D<") || xml.contains("D\n"), "{xml}");
  }

  /// latex.ltx:15913 `\endlist` decrements `\@listdepth`; our `\endlist`
  /// never did, so a raw class whose `\list` is latex.ltx's (memoir.cls:4580,
  /// with the `>5 → \@toodeep` check) hit "Too deeply nested" on the seventh
  /// list (witness memman: 88 errors from `adjustwidth`, memoir.cls:11268).
  #[test]
  fn endlist_decrements_listdepth() {
    let (stderr, xml) = convert(
      r"\documentclass{memoir}
\begin{document}
\begin{adjustwidth}{1em}{1em}A\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}B\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}C\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}D\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}E\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}F\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}G\end{adjustwidth}
\makeatletter\the\@listdepth\makeatother
\end{document}
",
      true,
    );
    assert!(!stderr.contains("Too deeply nested"), "{stderr}");
    assert!(xml.contains("G"), "{xml}");
    // The lists are real lists since OXIDIZED_DESIGN #180; the depth
    // reads 0 after the seventh has closed.
    assert!(xml.contains("<p>0</p>"), "{xml}");
  }

  /// listings.sty:320 `\let\lst@UserCommand\gdef`; patches such as
  /// tagpdfdocu-patches.sty:65 `\lst@UserCommand\lstrenewenvironment#1#2#{…}`
  /// otherwise leaked their `#` PARAM tokens into digestion (tagpdf manual:
  /// 7× "should never reach Stomach").
  #[test]
  fn lst_usercommand_is_gdef() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{listings}
\makeatletter
\lst@UserCommand\lst@mytest#1#2#{[#1/#2]}
\makeatother
\begin{document}
\makeatletter\lst@mytest ab{x}\makeatother DONE
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("misdefined"), "{stderr}");
    assert!(
      xml.contains("[a/b]xDONE") || xml.contains("[a/b]x DONE"),
      "{xml}"
    );
  }

  /// `\lx@lstinline` opened its group with a direct `bgroup()` but closed it
  /// with a raw `T_END`, which the gullet counts as −1 on the alignment
  /// ledger; with a non-brace delimiter nothing compensated, so inside a
  /// `p{}` cell the row's `\\` was never recognised and the tabular never
  /// closed (witness bibleref-parse L172: 28-error cascade).
  #[test]
  fn lstinline_pipe_in_p_column() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{listings}
\begin{document}
\begin{tabular}{p{3cm}p{3cm}}
A & \lstinline|\foo| here\\
C & \lstinline{\bar} third\\
\end{tabular}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
    assert!(xml.contains("ltx_lst_identifier\">foo"), "{xml}");
    assert!(xml.contains("third"), "{xml}");
  }

  /// Real `\DocumentMetadata` always loads tagpdf (documentmetadata-support
  /// .ltx:72 → latex-lab-testphase-latest.sty:39); the tagpdf manual iterates
  /// `\g__tag_role_NS_pdf_prop` (tagpdf.tex:2163) and an undefined prop turned
  /// `\prop_map_inline:cn` into a `\prg_break_point:Nn` runaway to EOF.
  #[test]
  fn documentmetadata_loads_tagpdf() {
    let (stderr, xml) = convert(
      r"\DocumentMetadata{tagging=on}
\documentclass{article}
\begin{document}
\ExplSyntaxOn
\clist_clear:N \l_tmpa_clist
\prop_map_inline:cn { g__tag_role_NS_pdf_prop }
  { \clist_put_right:Nn \l_tmpa_clist {#1} }
\clist_use:Nn \l_tmpa_clist {,}
\ExplSyntaxOff
DONE
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(xml.contains("StructTreeRoot"), "{xml}");
    assert!(xml.contains("DONE"), "{xml}");
  }

  /// nicematrix.sty:1644/3745 `\NotEmpty` (flags a cell for `hvlines`, no
  /// content) and :394 public `\g_nicematrix_code_before_tl` were missing
  /// from the binding that replaces the raw .sty (witness cahierprof.sty:619
  /// and :519/531 — cahierprof-exemple 2 errors).
  #[test]
  fn nicematrix_notempty_and_code_before_hook() {
    let (stderr, xml) = convert(
      r"\documentclass{article}
\usepackage{nicematrix}
\ExplSyntaxOn
\tl_gput_right:Nn \g_nicematrix_code_before_tl { x }
\ExplSyntaxOff
\begin{document}
\begin{NiceTabular}{cc}
a & b \\
c & \NotEmpty \\
\end{NiceTabular}
\end{document}
",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<ERROR"), "{xml}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  }

  /// tikz-network.sty L400 presets `Network=false` on family `[NW]{vertex}`,
  /// then `\@vertex` (L414-433) sets every key explicitly. Our beyond-Perl
  /// `\XKV@setkeys` runs the Rust reader once per `preseth` hook before the
  /// main list; the reader wipes `\XKV@fams`/`\XKV@prefix` on exit (faithful
  /// to Perl KeyVals.pm L389-400), so the main call used to be rebuilt from an
  /// empty family list and every real key became "unknown" (1001 errors on
  /// tikz-network.tex, all `\cmdNW@vertex@*`). Real `\XKV@s@tkeys` never
  /// mutates them (xkeyval.tex L464-469); the shim now saves/restores both.
  #[test]
  fn xkeyval_preset_hook_keeps_family_for_main_list() {
    let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@cmdkey  [NW] {vertex} {color}{}
\define@cmdkey  [NW] {vertex} {fontcolor}{}
\define@boolkey [NW] {vertex} {Network}[true]{}
\presetkeys     [NW] {vertex} {Network = false,}{}
\begin{document}
\setkeys[NW]{vertex}{color={red}, fontcolor={blue}}
C=[\cmdNW@vertex@color] F=[\cmdNW@vertex@fontcolor]
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("C=[red] F=[blue]"), "{xml}");
  }

  /// KNOWN_PERL_ERRORS #123: `` `<char> `` must take the character code of
  /// any character token (tex.web §442). Perl strips a leading `\` from the
  /// token *string* — fine for `\a`, but a catcode-12 backslash (what
  /// `\detokenize`/`\string` produce) becomes "" → 0. Witness
  /// bibleref-parse.sty L481-486 `\brp@ifcs` (backslash test → every
  /// `\foreach`-variable book name "unknown"). Same root aborts every
  /// `\fpeval{\dimen0 > \dimen1}`: l3fp's comparison chain-detect
  /// (expl3-code.tex L17662-17673) routes `\if_case:w` on
  /// `` ` \token_to_str:N <register> `` → 0 instead of 92 → the `@` sentinel
  /// is never emitted → `Missing argument Until:@` + Fatal EoF (witness
  /// swfigure `\fptest`/`\DFscalefactor`).
  #[test]
  fn backquote_charcode_of_other_backslash() {
    let tex = r"\documentclass{article}
\begin{document}
\def\name{x}
\def\first#1#2\end{[\number`#1]}
A\expandafter\first\detokenize{\name}aa\end
B\expandafter\first\string\name aa\end
C[\number`\\]D[\number`\a]E[\number`a]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Missing number"), "{stderr}");
    assert!(xml.contains("A[92]"), "{xml}");
    assert!(xml.contains("B[92]"), "{xml}");
    assert!(xml.contains("C[92]D[97]E[97]"), "{xml}");
  }

  #[test]
  fn fpeval_register_right_operand_of_comparison() {
    let tex = r"\documentclass{book}
\usepackage{xfp}
\newdimen\Ah\newdimen\Bt\Ah=10pt\Bt=5pt
\begin{document}
\edef\x{\fpeval{\Ah > \Bt}}[\x]
\edef\y{\fpeval{\Ah < \Bt}}[\y]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert!(xml.contains("[1]\n[0]"), "{xml}");
  }

  /// OXIDIZED_DESIGN #170's named residual: `\angle` as a `\tikzmath`
  /// variable (sunpath.sty L44-47). Real LaTeX's `\angle` is a robust
  /// command, so `\meaning` starts with `macro:` and tikzmath's sniff
  /// (tikzlibrarymath.code.tex L22-46) treats it as assignable; a primitive
  /// math atom hits the keyword path and `\csname pgfmath\angle\endcsname`
  /// loops to the error cap (Rust 1001, Perl 101). The math meaning must
  /// survive in the space-suffixed inner CS.
  #[test]
  fn angle_tikzmath_variable() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{math}
\begin{document}
$\angle ABC$
\tikzmath{ \angle = 90 - 30; }
V=[\angle]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(">\u{2220}</XMTok>"), "{xml}");
    assert!(xml.contains("V=[60.0]"), "{xml}");
  }

  /// Real listings sets its line counter at register level
  /// (listings.sty L1516 `\global\c@lstnumber\lst@firstnumber`), never via
  /// `\setcounter`; our block emitter used user-level `\setcounter`, which
  /// xassoccnt.sty L2553 wraps in an expl3 body that (under our engine) runs
  /// away into the following `\@lst@startline` → the first line leaks as
  /// loose text under `<ltx:listing>` (518 malformed errors, xassoccnt_doc).
  #[test]
  fn listings_line_counter_init_is_register_level() {
    let tex = r"\documentclass{article}
\usepackage{xassoccnt}
\usepackage{listings}
\begin{document}
\section{S}
\begin{lstlisting}
Hello world
Second line
\end{lstlisting}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<listingline").count(), 2, "{xml}");
  }

  /// tabularray.sty:3461-3470 `\NewTblrEnviron{name}` creates a `tblr`-alias
  /// environment; the binding had it as a no-op so `\begin{MPMtache}` was
  /// undefined. Witness: profsio ProfSio-doc-fr (ProfSio.sty:98).
  #[test]
  fn tabularray_newtblrenviron_defines_environment() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\NewTblrEnviron{MPMtache}
\SetTblrInner[MPMtache]{colspec={Q[c]Q[c]}}
\begin{document}
\begin{MPMtache}{hlines={wd=1pt},vlines={wd=1pt}}
\SetCell[c=2]{c} {X} & \\
a & b \\
\end{MPMtache}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<tabular"), "{xml}");
    assert!(xml.matches("<tr").count() >= 2, "{xml}");
  }

  /// xcolor.sty L1461 runs `\color{black}` at load, which defines the current
  /// color `.` (`\color@.`); `\draw[.]` resolves via tikz's colour fallback.
  /// Witness: twoxtwogame_doc (twoxtwogame.sty:493 `row player color=.`).
  #[test]
  fn xcolor_current_color_dot_defined_at_load() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\draw[line width=1pt, ., ] (0,0) -- (1,1);
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<svg:path"), "{xml}");
  }

  /// pgf driver handler for xcolor's core model `hsb` (xcolor.sty L1121-1132
  /// folds Hsb/HSB/tHsb/wave into it); pgfcoregraphicstate.code.tex L195-202
  /// errors "Unsupported color model" when `\pgfsys@color@hsb` is missing.
  /// Witness: tikz-3dplot_documentation (tikz-3dplot.sty:731).
  #[test]
  fn pgfsys_hsb_color_model_supported() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\definecolor{tdplotfillcolor}{hsb}{0.5, 1, 1}
\fill[tdplotfillcolor] (0,0) rectangle (1,1);
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Unsupported color model"), "{stderr}");
    assert!(xml.contains("#00FFFF"), "{xml}");
  }

  /// eTeX `quotient` (etex.ch, `scan_expr`) rounds half AWAY from zero on
  /// magnitudes: `\numexpr -1/2` = -1, `-7/2` = -4 (pdflatex-probed). Perl
  /// Number.pm `int(0.5 + n/d)` truncates toward zero (KNOWN_PERL_ERRORS
  /// #124), and l3fp's `\__fp_mul_cases_o:NnNnww` case index
  /// (expl3-code.tex:18724-18760) relies on the TeX rounding — with the
  /// Perl rounding `0 * x` inside a `+`/`-` expression collapsed the whole
  /// `\fp_eval:n` to 0. Witness: wheelchart (wheelchart.sty:2423 transform
  /// determinant → 1001 errors).
  #[test]
  fn numexpr_division_rounds_half_away_from_zero() {
    let tex = r"\documentclass{article}
\begin{document}
K[\the\numexpr -1/2\relax][\the\numexpr 1/2\relax][\the\numexpr -3/2\relax][\the\numexpr -7/2\relax][\the\numexpr 7/2\relax][\the\numexpr -5/-2\relax][\the\numexpr 5/-2\relax]

\ExplSyntaxOn
F[\fp_eval:n { 800 - 0 * 3 }][\fp_eval:n { (0*3) + 800 }][\fp_eval:n { -0 * 3 }]
\ExplSyntaxOff
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("K[-1][1][-2][-4][4][3][-3]"), "{xml}");
    assert!(xml.contains("F[800][800][-0]"), "{xml}");
    // `\dimexpr` shares `quotient` (pdflatex-probed 2026-09-02; the xy
    // `\dimexpr(\X@p+2\A@)/3` curve-control shape, xytest golden re-blessed).
    let tex = r"\documentclass{article}
\begin{document}
\newdimen\A \A=-107.6pt
D[\the\dimexpr -1sp/2\relax][\the\dimexpr 1sp/2\relax][\the\dimexpr -3sp/2\relax][\number\dimexpr\A/3\relax][\number\dimexpr(-1pt+2\A)/3\relax]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("D[-0.00002pt][0.00002pt][-0.00003pt][-2350558][-4722961]"),
      "{xml}"
    );
  }

  /// `\read` past end-of-file reads the synthetic empty line + `\endlinechar`
  /// in state N (tex.web §345-349): an IGNORE (catcode 9) endline char can
  /// never become a token. Perl Mouth.pm:303-307 emits it and the Stomach
  /// reports `misdefined` (KNOWN_PERL_ERRORS #125). Witness: liftarm
  /// (pgfmanual `codeexample` sets `\catcode`\^^M=9` around `\scantokens`,
  /// animate.sty `\@anim@buildtmln` `\read`s the timeline to EOF — 501
  /// errors capped).
  #[test]
  fn read_at_eof_drops_ignored_endlinechar() {
    let tex = r"\documentclass{article}
\begin{document}
\newread\myr
\openin\myr=rdtest.dat
\catcode`\^^M=9\relax
\read\myr to \la
\read\myr to \lb
\catcode`\^^M=5\relax
\closein\myr
X\lb X\la X
\end{document}
";
    let (stderr, xml) = convert_with_sty(tex, "rdtest.dat", "lineone\n");
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("should never reach Stomach"), "{stderr}");
    assert!(xml.contains("XXlineoneX"), "{xml}");
  }

  /// xkeyval.tex:518-529 + 560-583: `\XKV@s@tk@ys@` saves the raw value and
  /// then `\XKV@replacepointers` splices every `\usevalue{X}` EAGERLY, under
  /// the key's own header, before the key code runs. The binding resolved
  /// `\usevalue` lazily at expansion time, so a value stored by the key code
  /// and expanded later (inside another `\setkeys`) looked the pointer up
  /// under the wrong family. Witness: pmdraw (pmdraw.sty:1704-1706 stores
  /// `\pmdraw@tikz`, consumed at :1857-1860 — 501 errors capped). Perl
  /// stubs the pointer system outright (xkeyval.sty.ltxml:397-432).
  #[test]
  fn xkeyval_usevalue_is_replaced_eagerly_at_setkeys() {
    let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key{d}{v}{\def\stored{#1}}
\define@key{dDefault}{v}{\setkeys{d}{\savevalue{v}=#1}}
\makeatother
\begin{document}
\makeatletter
\setkeys{dDefault}{v=42}
\setkeys{d}{v=\usevalue{v}}
\setkeys{e}{whatever}
[\stored]
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[42]"), "{xml}");
  }

  /// lstmisc.sty:60-64 `\lst@WFBegin` runs `\immediate\openout\lst@WF=#2`
  /// on every FRESH `\lst@BeginWriteFile`/`\lst@BeginAlsoWriteFile`, so the
  /// file is truncated per begin/end span (pdflatex: only the second write
  /// survives). The display-time tee appended across spans, so
  /// forest-doc.sty:59's per-example `\jobname.tmp` kept a stale
  /// `\usepackage[linguistics]{forest}` line that every later
  /// `\lst@sampleInput` re-`\input` — 440 "can only appear in the preamble"
  /// errors. Witness: forest-doc (1001 errors + TooManyErrors).
  #[test]
  fn listings_writefile_truncates_on_fresh_begin() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\makeatletter
\lst@BeginAlsoWriteFile{\jobname.tmp}
\begin{lstlisting}
\usepackage[foo]{bar}
\end{lstlisting}
\lst@EndWriteFile
\lst@BeginAlsoWriteFile{\jobname.tmp}
\begin{lstlisting}
hello world
\end{lstlisting}
\lst@EndWriteFile
\makeatother
[\input{\jobname.tmp}]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The displayed listings still show both bodies (AlsoWriteFile); the
    // `\input`-back must see only the second span.
    assert!(xml.contains("[hello world"), "{xml}");
    assert_eq!(
      xml.matches("foo").count(),
      1,
      "stale first write re-input: {xml}"
    );
  }

  /// etoolbox.sty:1740-1746: under a 2020-10+ format `\AtEndPreamble` IS
  /// `\AddToHook{begindocument/before}`, so it takes the hook system's
  /// optional `[label]`. tcbdocumentation.code.tex:69 defines `\meta`
  /// inside `\AtEndPreamble[tcolorbox]{…}`; the binding read `[tcolorbox]`
  /// as the hook code. Witness: xassoccnt_doc (`undefined:\meta`).
  #[test]
  fn etoolbox_atendpreamble_accepts_hook_label() {
    let tex = r"\documentclass{article}
\usepackage[most,documentation]{tcolorbox}
\begin{document}
Syntax: \meta{true,false}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("true,false"), "{xml}");
    assert!(!xml.contains("tcolorbox]"), "label leaked as text: {xml}");
  }

  /// forest.sty:8506 `\NewDocumentEnvironment{forest}{D(){}}` also defines
  /// the bare `\forest … \endforest` pair, and :1413 `\bracketset`;
  /// neoschool.cls:8567-8581 builds `neotree` on the bare form and calls
  /// `\bracketset{action character=@}` at load. The stub knew only
  /// `\begin{forest}`, so the tree body (`w=\frac{1}{3}`) leaked into text
  /// as XMApp errors. Witness: neoschool (4 errors). The stub's own
  /// one-per-kind report is the single expected error.
  #[test]
  fn forest_bare_cs_form_discards_body() {
    let tex = r"\documentclass{article}
\usepackage{forest}
\bracketset{action character=@}
\begin{document}
A\forest [root [w=\frac{1}{3}] [b]]\endforest B
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    // The stub diagnostic is a Warn since batch 56k (`forest_stub_is_a_warning`).
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("stub binding"), "{stderr}");
    assert!(!stderr.contains("undefined:\\forest "), "{stderr}");
    assert!(!stderr.contains("bracketset"), "{stderr}");
    assert!(!xml.contains("XMApp"), "tree body leaked: {xml}");
    assert!(xml.contains("A") && xml.contains("B"), "{xml}");
  }

  /// latex.ltx:14103-14107 `\@setfontsize` only `\let\@currsize#1` under
  /// `\ifx\protect\@typeset@protect`, so it is inert inside `\protected@edef`.
  /// Our binding (and Perl latex_constructs.pool:5622, which OOMs same-host)
  /// dropped the guard: a raw class routing its size commands through
  /// `\@setfontsize` (tufte-common.def:368-405) re-expanded
  /// `\@currsize`→`\normalsize`→`\@setfontsize\normalsize…` without bound
  /// once pgf edef'd tikz-network's `font=\normalsize` label. Witness:
  /// tikz-network manual (PushbackLimit Fatal, no output).
  #[test]
  fn setfontsize_is_inert_inside_protected_edef() {
    let tex = r"\documentclass{article}
\makeatletter
\renewcommand\normalsize{\@setfontsize\normalsize\@xpt{14}}
\protected@edef\lx@probe{\normalsize}
\makeatother
\begin{document}
probe ok
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("PushbackLimit"), "{stderr}");
    assert!(xml.contains("<p>probe ok</p>"), "{xml}");
  }

  /// xkeyval.tex:569 `\XKV@ifundefined{XKV@<header><key>@value}` tests
  /// DEFINEDNESS: a key saved with an EMPTY value (`\savevalue{k}={}`, L525-527)
  /// is `\let` to an empty-bodied macro and `\usevalue{k}` splices nothing.
  /// `replace_pointers` read `get_expansion()`, which is `None` for an empty
  /// body, and reported "no value recorded". Witness: pmdraw
  /// (pmdraw.sty:2191-2264 sets 16 defaults to `{}` — 501 errors + cap).
  #[test]
  fn xkeyval_usevalue_of_empty_saved_value_is_empty() {
    let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key{fam}{k}{\def\myval{#1}}
\define@key{famDefault}{k}{\setkeys{fam}{\savevalue{k}=#1}}
\setkeys{famDefault}{k={}}
\setkeys{fam}{k=\usevalue{k}}
\makeatother
\begin{document}
START\myval END
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("START END") || xml.contains("STARTEND"),
      "{xml}"
    );
  }

  /// Perl TeX_Debugging.pool.ltxml:110-113 reduces a primitive / conditional /
  /// constructor to its cs-or-alias token before rendering `\meaning`; our
  /// `DefMath` atoms are a separate `Stored::MathPrimitive` and fell to the
  /// catch-all `Stored[??]`. Witness: sunpath (tikzmath `\meaning` sniffing).
  #[test]
  fn meaning_of_defmath_atom_is_its_cs() {
    let tex = r"\documentclass{article}
\begin{document}
[\expandafter\detokenize\expandafter{\meaning\forall}][\expandafter\detokenize\expandafter{\meaning\infty}]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // (the backslash itself is OT1-decoded in text; the names are the signal)
    assert!(xml.contains("forall][") && xml.contains("infty]"), "{xml}");
    assert!(!xml.contains("Stored["), "{xml}");
  }

  /// delarray.sty:43-58 — `\@@array[pos]` peeks with `\@ifnextchar\bgroup`;
  /// a non-brace is a delimiter pair around the column spec,
  /// `\begin{array}({cc})…\end{array}` = `\left(` array `\right)`. Both
  /// engines' own `\array[]{}` read `(` as the template, so every `&`
  /// reported "Extra alignment tab" (memoir manual, memoir.cls:5468
  /// `\RequirePackage{delarray}`: 33 errors; SHARED). pdflatex clean.
  #[test]
  fn delarray_delimited_array_form() {
    let tex = r"\documentclass{article}
\usepackage{delarray}
\begin{document}
$\begin{array}({cc}) a & b \\ c & d \end{array}$
$\begin{array}[t]\{{lL}. x \\ y \end{array}$
$\begin{array}{c} p \\ q \end{array}$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Extra alignment tab"), "{stderr}");
    assert_eq!(xml.matches("<XMArray").count(), 3, "{xml}");
    // the delimiters survive as fence tokens around the arrays
    assert!(
      xml.contains(r#"role="OPEN">(</XMTok>"#) || xml.contains(r#">(</XMTok>"#),
      "{xml}"
    );
    assert!(xml.contains(r#">{</XMTok>"#), "{xml}");
  }

  /// etoolbox.sty:1743: `\AtEndPreamble` IS `\AddToHook{begindocument/before}`,
  /// so it queues in order with doc.sty:907-910's chunk that loads hypdoc
  /// (→ hyperref) at `\begin{document}`; a private list that fired before
  /// the L3 hook saw `\hypersetup` undefined. Witnesses: liftarm.tex:39,
  /// wheelchart.tex:128 (ltxdoc manuals; SHARED with Perl).
  #[test]
  fn etoolbox_atendpreamble_runs_after_earlier_begindocument_before_chunks() {
    let tex = r"\documentclass{ltxdoc}
\usepackage{etoolbox}
\AtEndPreamble{\hypersetup{colorlinks=true}}
\begin{document}
Hello.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>Hello.</p>"), "{xml}");
  }

  /// pgfsys-latexml.def.ltxml:392-398 opens a self-contained `svg:svg` when
  /// `\lxSVG@begingroup@` fires inside an `ltx:` box within a picture; a
  /// BARE-style path (no dash/color) never passes through the group opener,
  /// so `\phantom{\draw …}` inside a tikzpicture relocated its `svg:path` up
  /// to the picture group and desynced every later close — pmdraw manual
  /// (`vertices top phantom`, pmdraw.sty:56-66): 64 errors. SHARED (Perl 7 on
  /// this repro); `ensure_svg_context` now guards the path emitters too.
  #[test]
  fn pgf_bare_path_inside_phantom_stays_in_its_box() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{center}\begin{minipage}{0.85\textwidth}\begin{minipage}[c]{0.4\linewidth}
\raisebox{0.5cm}{\begin{tikzpicture}\phantom{\draw (0,0)--(1,1);}\draw (0,0)--(2,0);\end{tikzpicture}}
\end{minipage}\end{minipage}\end{center}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // the phantom's drawing nests inside its own foreignObject box
    let fo = xml
      .find("<svg:foreignObject")
      .expect("phantom foreignObject");
    let after = &xml[fo..];
    let close = after.find("</svg:foreignObject>").unwrap();
    assert!(
      after[..close].contains("<svg:path"),
      "phantom path escaped its box: {xml}"
    );
    assert_eq!(xml.matches("<svg:svg").count(), 2, "{xml}");
  }

  /// End-to-end hobby curve: the raw `hobby.code.tex` load (stub retired) plus
  /// l3fp's comparison chain-detect (`\__fp_parse_compare_auxi:NNNNNNN`,
  /// expl3-code.tex:17662 — the backquote of a detokenized backslash must
  /// read 92, KPE #123) yield a Hobby-smoothed cubic. Witness: hobby manual
  /// (373 paths, was ~empty + `Until:@` Fatal; Perl 101 errors + Fatal).
  #[test]
  fn hobby_shortcut_draws_a_cubic_path() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{hobby}
\begin{document}
\begin{tikzpicture}[use Hobby shortcut]
\draw (0,0) .. (1,1) .. (2,0);
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Until:@"), "{stderr}");
    assert!(xml.contains(r#"d="M 0 0 C "#), "no Hobby cubic: {xml}");
  }

  /// tabularray.sty:3472-3477 builds `longtblr`/`talltblr` with the same
  /// factory as `tblr`; the binding knew only `tblr`, so `{longtblr}` was an
  /// undefined environment whose body cascaded (panda manual: 149
  /// `<relationaltoken>` errors + `Until:` EoF Fatal).
  #[test]
  fn tabularray_longtblr_and_talltblr_are_tblr() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{longtblr}[theme=naked]{colspec={Xll}, rowhead=1}
A & B & C \\
1 & 2 & 3 \\
\end{longtblr}
\begin{talltblr}{colspec={cc}}
x & y \\
\end{talltblr}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
    assert_eq!(xml.matches("<tr").count(), 3, "{xml}");
  }

  /// expl3-code.tex:3758-3790: `\tl_set_rescan:Nnn` captures the WHOLE
  /// `\scantokens` output (PARAM tokens included) through `\everyeof` +
  /// `\__tl_rescan:NNw`'s delimited scan. Our `\scantokens` cannot carry the
  /// `\everyeof` payload (P15 dead-end), so the scan ran to EOF and a
  /// rescanned macro MEANING leaked its `#`s to digestion — substances.sty:452
  /// (substances manual, 720 `misdefined:#`; Perl identical). The core now
  /// rescans atomically under the caller's catcodes.
  #[test]
  fn tl_set_rescan_captures_param_tokens() {
    let tex = r"\documentclass{article}
\ExplSyntaxOn
\cs_new:Npn \FooEntry #1#2#3 { #1@#3|see{#2} }
\cs_new_protected:Npn \contains_see:N #1
  {
    \tl_set_rescan:Nnx \l_tmpa_tl {} {\cs_meaning:N #1 }
    \tl_if_in:VnT \l_tmpa_tl { |see } { YESSEE }
  }
\tl_set_rescan:Nnn \l_tmpb_tl { \char_set_catcode_other:N \\ } { A\B }
\ExplSyntaxOff
\begin{document}
\ExplSyntaxOn \contains_see:N \FooEntry [\tl_use:N \l_tmpb_tl] \ExplSyntaxOff
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("misdefined"), "{stderr}");
    assert!(xml.contains("YESSEE"), "rescan lost the meaning: {xml}");
    // the caller's catcode setup governs the rescan: `\` as OTHER is text
    assert!(xml.contains("[A") && xml.contains("B]"), "{xml}");
  }

  /// tex.web §442: a brace read as a character constant (`` `} ``) undoes the
  /// `align_state` step `get_token` applied, so `\iffalse{\fi\ifnum0=`}\fi`
  /// (expl3 `\group_align_safe_begin:`, amsmath) leaves ALIGN_STATE +1 with no
  /// group open. Without the undo the idiom netted 0 and an alignment-catcode
  /// token in a delimited-macro definition inside a cell — l3tl
  /// `\tl_replace_all` with a rescanned `_`(4), l3doc `\marg` inside `syntax`
  /// (every l3doc manual) — was taken as the cell end (`Until:…@after_` EoF
  /// Fatal). Perl Gullet.pm:926 shares the gap.
  #[test]
  fn backquote_brace_charcode_keeps_align_state() {
    let tex = r"\documentclass{article}
\usepackage{expl3}
\begin{document}
\begin{tabular}{l}
\begin{minipage}{3cm}
\ExplSyntaxOn
\tl_set_rescan:Nnn \l_tmpa_tl { \char_set_catcode:nn { `_ } {4} } { _ }
\tl_set:Nn \l_tmpb_tl { a_b }
\tl_replace_all:NVn \l_tmpb_tl \l_tmpa_tl { X }
[\tl_use:N \l_tmpb_tl]
\ExplSyntaxOff
\end{minipage}
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Until:"), "{stderr}");
    // the letter-catcode `_` in `a_b` is not the catcode-4 pattern: untouched
    assert!(xml.contains("[a") && xml.contains("b]"), "{xml}");
    assert_eq!(xml.matches("<td").count(), 1, "{xml}");
  }

  /// l3doc `\marg`/`\oarg` inside `{syntax}` (a tabular+minipage) — the
  /// corpus-wide face of `backquote_brace_charcode_keeps_align_state`
  /// once `\tl_set_rescan` captures alignment-catcode tokens.
  #[test]
  fn l3doc_marg_inside_syntax_env() {
    let tex = r"\documentclass{l3doc}
\begin{document}
\begin{function}{\zcheck}
\begin{syntax}
\cs{zcheck} \oarg{options} \marg{labels}
\end{syntax}
Typesets \meta{text}.
\end{function}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("labels"), "{xml}");
  }

  /// latex-lab-testphase-tikz.sty:228-260 adds `/tikz/alt` and friends when
  /// `\DocumentMetadata` is active; the picture's alt text is recorded (no
  /// XML slot yet) and `\tagtool`/`\DebugBlocksOff` are PDF-structure-only.
  /// Witness: tagpdf manual (9× "I do not know the key '/tikz/alt'").
  #[test]
  fn documentmetadata_tikz_alt_key_and_tagging_tools() {
    let tex = r"\DocumentMetadata{tagging=on}
\documentclass{article}
\usepackage{tikz}
\DebugBlocksOff
\begin{document}
\tagtool{para/tag=P}
\begin{tikzpicture}[alt={A red circle}]
\fill[red] (0,0) circle (2pt);
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<svg:path"), "{xml}");
  }

  /// tabularray.sty:3444 `\SetTblrInner[<envs>]{keys}` records per-environment
  /// inner defaults that every `\begin{<env>}` prepends, and a table with no
  /// colspec anywhere takes its column count from the rows. The combination
  /// `\NewTblrEnviron` with `\SetTblrInner[spectblr]{hlines…}` and
  /// `\begin{spectblr}[…]{}` had become a zero-column template (pegmatch
  /// manual: 52 "Extra alignment tab").
  #[test]
  fn tabularray_settblrinner_defaults_and_inferred_columns() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\NewTblrEnviron{spectblr}
\SetTblrInner[spectblr]{hlines, rowhead=1}
\SetTblrInner[tblr]{colspec={lc}}
\begin{document}
\begin{spectblr}[caption=Basic]{}
Command & Description & More \\
a & b & c \\
\end{spectblr}
\begin{tblr}{}
x & y \\
\end{tblr}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
    assert_eq!(xml.matches("<tr").count(), 3, "{xml}");
    assert_eq!(xml.matches("<td").count(), 8, "{xml}");
    assert!(
      xml.contains(r#"align="center""#),
      "stored colspec lost: {xml}"
    );
  }

  /// tabularray colspec inter-column material `@{…}`/`!{…}` translates through
  /// (not a column). A bailed `colspec={@{}Xll@{}}` made the WHOLE inner spec
  /// the tabular template, whose `cell{…}={cmd={…}}` value was edef-expanded
  /// in the preamble — panda manual (`\BusyPanda` fp → `Until:\__fp_sep:`
  /// EoF Fatal).
  #[test]
  fn tabularray_colspec_intercolumn_material() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{colspec={@{}Xll@{}}, cell{2-Z}{2}={cmd={\textbf}}}
a & b & c \\
d & e & f \\
\end{tblr}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
    assert_eq!(xml.matches("<td").count(), 6, "{xml}");
  }

  /// Begin-document hook code runs RE-LOCKED (Perl State.pm:502-514 ignores a
  /// redefinition of a `:locked` cs under `$UNLOCKED=0`; Perl never fires the
  /// L3 `begindocument` hook at all). Our `\hook_use:n{begindocument}` digest
  /// ran unlocked, so polyglossia.sty:1442-1456's `\cs_set:Npn \@caption
  /// #1[#2]#3` replaced the locked `\@caption` and its `[`-scan overshot every
  /// figure. Witness: beamerdarkthemes user guide (101 caption errors).
  #[test]
  fn begindocument_hook_code_cannot_redefine_locked_caption() {
    let tex = r"\documentclass{article}
\makeatletter
\AddToHook{begindocument}{%
  \let\xpgsave\@caption
  \long\def\@caption#1[#2]#3{\xpgsave{#1}[{\ignorespaces#2}]{#3}}}
\makeatother
\begin{document}
\begin{figure}\caption{cormorant color theme}\label{fig:a}\end{figure}
\begin{figure}[p]\caption{magpie}\label{fig:b}\end{figure}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<caption").count(), 2, "{xml}");
    assert!(xml.contains("cormorant color theme"), "{xml}");
  }

  /// latex.ltx:15392 `\end` ends with `\if@ignore\@ignorefalse\ignorespaces\fi`
  /// after the `env/<name>/after` hook; noindentafter.sty:44
  /// `\nia@afterendenv#1\ignorespaces\fi` is delimited by those tokens and
  /// otherwise scans to EOF (pkgloader manual, 102 errors; Perl identical).
  #[test]
  fn end_environment_emits_the_ignorespaces_epilogue() {
    let tex = r"\documentclass{article}
\usepackage{noindentafter}
\NoIndentAfterEnv{itemize}
\begin{document}
\begin{itemize}\item a\end{itemize}
Text after list. $x$
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Until:"), "{stderr}");
    assert!(xml.contains("<itemize"), "{xml}");
    assert!(
      xml.contains("Text after list.") && xml.contains("<Math"),
      "{xml}"
    );
  }

  /// ulem.sty:232-233 extension contract: `\bgroup\markoverwith{…}\ULon{text}`
  /// — the word machinery closes the group (`\UL@end *`, :59). The binding's
  /// inert internals lacked `\markoverwith`/`\ULon` and never closed it, so
  /// CJKfntef's `\CJKunderline` (CJKfntef.sty:258-283) unbalanced the
  /// enclosing list. Witness: jnuexam examfc-a-answer (+8 CJKfntef manuals).
  #[test]
  fn ulem_markoverwith_ulon_contract_closes_its_group() {
    let tex = r"\documentclass{article}
\usepackage{ulem}
\makeatletter
\def\myul{\bgroup\markoverwith{\hbox{x}}\ULon}
\makeatother
\begin{document}
\begin{description}
\item[A] before \myul{XYZ} after \myul{second}.
\end{description}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("XYZ") && xml.contains("second"), "{xml}");
    assert_eq!(xml.matches("<description").count(), 1, "{xml}");
  }

  /// tex.web §370: an UNDEFINED control sequence met inside `\csname…\endcsname`
  /// is reported and discarded; the name scan continues to `\endcsname`. The
  /// error stub made it look defined, so the §373 `back_error` path ended the
  /// name early and the real `\endcsname` went stray ("Extra \endcsname":
  /// 1693 lines / 65 manuals — beamer2thesis's babel `\csname l@\beamer@…`,
  /// gckanbun's pgf arrow declarations). One error, like pdflatex.
  #[test]
  fn csname_discards_an_undefined_cs_and_keeps_scanning() {
    let tex = r"\documentclass{article}
\makeatletter
\begin{document}
A\csname l@\beamer@torinoth@language\endcsname B
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(!stderr.contains("Extra \\endcsname"), "{stderr}");
    assert!(!stderr.contains("should not appear"), "{stderr}");
    assert!(xml.contains("A") && xml.contains("B"), "{xml}");
  }

  /// etex.sty's register allocators (`\globcount`…`\loctoks`…, etex.sty:332-348)
  /// are PACKAGE macros; Perl's eTeX pool defines none. In the always-on pool
  /// they made l3sort freeze the `\cs_if_exist:NT \loctoks` branch of
  /// `\__sort_compute_range:` (expl3-code.tex:23356-23364, `\count265`/
  /// `\count275`) into the dump, and once a package load left `\count265` > 0
  /// every `\seq_sort` ran an inverted range to the TokenLimit (spath3
  /// `insert gaps after components`, tabularray/testidx/cistercian manuals).
  /// Needs the regenerated dump (base `\count15` branch, Perl
  /// latex_dump.pool:8589).
  #[test]
  fn l3sort_after_package_registers_terminates() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{spath3}
\begin{document}
\ExplSyntaxOn
\seq_set_from_clist:Nn \l_tmpa_seq { 3 , 1 , 2 }
\seq_sort:Nn \l_tmpa_seq { \int_compare:nNnTF {#1} < {#2} { \sort_return_same: } { \sort_return_swapped: } }
[\seq_use:Nn \l_tmpa_seq { - }]
\ExplSyntaxOff
\begin{tikzpicture}
\draw[spath/save=p] (0,0) -- (1,0) (2,0) -- (3,0);
\tikzset{spath/.cd, insert gaps after components={p}{10pt}{1}}
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("TokenLimit"), "{stderr}");
    assert!(xml.contains("[1-2-3]"), "{xml}");
    assert!(xml.contains("<svg:path"), "{xml}");
    // no etex.sty loaded: the allocators stay undefined, like Perl/LaTeX
    let (stderr2, xml2) = convert(
      r"\documentclass{article}\begin{document}[\ifdefined\loctoks Y\else N\fi]\end{document}",
      false,
    );
    assert_eq!(error_count(&stderr2), 0, "{stderr2}");
    assert!(xml2.contains("[N]"), "{xml2}");
  }

  /// A `fnum@font@<type>` value wraps the number as a braced argument
  /// (enumitem.sty:451/1478 `\enit@format{<label>}`); as a bare prefix an
  /// argument-taking font command grabbed the following `\@ifundefined`
  /// (non-decimal-units manual: `\setlist[description]{font=\docAuxKey}`,
  /// 40 errors; Perl Base_Utility.pool:1041 identical).
  #[test]
  fn enumitem_font_wraps_the_item_tag() {
    let tex = r"\documentclass{article}
\usepackage{enumitem}
\setlist[description]{font=\textbf}
\begin{document}
\begin{description}
\item[british] Currencies
\item[danish] Areas
\end{description}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("british") && xml.contains("danish"), "{xml}");
    assert!(
      xml.contains(r#"font="bold""#),
      "font not applied to the tag: {xml}"
    );
  }

  /// memoir.cls:5477-5719 auto-tables reduce to `\tabular`: `\autorows` fills
  /// `num` columns row-major, `\autocols` column-major with the `\linespercol`
  /// heights (:5665-5675, greedy ceil — column 0 tallest; the manual's own
  /// `\showit` mock is wrong), `{ctabular}`'s `[pos]` is horizontal. The raw
  /// code drives `\valign`/`\@mkpream` internals the engine never provides
  /// (memman: ~157 errors; SHARED).
  #[test]
  fn memoir_auto_tables_reduce_to_tabular() {
    let tex = r"\documentclass{memoir}
\begin{document}
\autorows{c}{5}{c}{one, two, three, four, five, six, seven, eight, nine, ten,
eleven, twelve, thirteen, fourteen}
\autocols{c}{5}{l}{one, two, three, four, five, six, seven, eight, nine, ten,
eleven, twelve, thirteen, fourteen}
\begin{ctabular}[l]{lcr}
LEFT & CENTER & RIGHT \\
l & c & r \\
\end{ctabular}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tabular").count(), 3, "{xml}");
    assert_eq!(xml.matches("<tr").count(), 8, "{xml}");
    // autorows: first row one…five; autocols: first row one,four,seven,ten,thirteen
    let rows: Vec<&str> = xml.split("<tr").skip(1).collect();
    let cells = |r: &str| -> Vec<String> {
      r.split("<td")
        .skip(1)
        .map(|c| {
          c.split('>')
            .nth(1)
            .unwrap_or("")
            .split('<')
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
        })
        .collect()
    };
    assert_eq!(
      cells(rows[0]),
      ["one", "two", "three", "four", "five"],
      "{xml}"
    );
    assert_eq!(
      cells(rows[2])[..4],
      ["eleven", "twelve", "thirteen", "fourteen"],
      "{xml}"
    );
    assert_eq!(
      cells(rows[3]),
      ["one", "four", "seven", "ten", "thirteen"],
      "{xml}"
    );
    assert_eq!(
      cells(rows[5])[..4],
      ["three", "six", "nine", "twelve"],
      "{xml}"
    );
    assert_eq!(cells(rows[6]), ["LEFT", "CENTER", "RIGHT"], "{xml}");
  }

  /// pdfTeX `\pdfuniformdeviate <n>` expands to a random integer in [0,n);
  /// the empty macro (Perl pdfTeX.pool:110) also ate the next token, so expl3's
  /// `\int_rand:nn` (`\tex_uniformdeviate:D 268435456 \__fp_sep:`) lost its
  /// separator and returned the midpoint — rejection-sampling loops never
  /// terminated (randintlist-l3 manual, TokenLimit). Deterministic seed:
  /// the same document converts identically; `\pdfsetrandomseed` re-seeds.
  #[test]
  fn pdfuniformdeviate_is_a_random_integer() {
    let tex = r"\documentclass{article}
\begin{document}
\ExplSyntaxOn
[\int_rand:nn{1}{1000},\int_rand:nn{1}{1000},\int_rand:nn{1}{1000},\int_rand:nn{1}{1000}]
[\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ]
\pdfsetrandomseed 42 [\the\pdfrandomseed]
\ExplSyntaxOff
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    let (_, xml2) = convert(tex, false);
    assert_eq!(
      xml, xml2,
      "random stream must be deterministic per conversion"
    );
    let first = xml.split('[').nth(1).unwrap().split(']').next().unwrap();
    let vals: Vec<i64> = first
      .split(',')
      .map(|v| v.trim().parse().unwrap())
      .collect();
    assert_eq!(vals.len(), 4, "{xml}");
    assert!(vals.iter().all(|&v| (1..=1000).contains(&v)), "{xml}");
    assert!(
      vals.windows(2).any(|w| w[0] != w[1]),
      "constant stream: {xml}"
    );
    let second = xml.split('[').nth(2).unwrap().split(']').next().unwrap();
    assert!(
      second
        .split(',')
        .all(|v| (0..10).contains(&v.trim().parse::<i64>().unwrap())),
      "{xml}"
    );
    assert!(xml.contains("[42]"), "{xml}");
  }

  /// listings `\lstnewenvironment{x}{<begin>}{<end>}`: the end code runs at
  /// the environment's group level AFTER the listing (listings.sty
  /// `\lst@EndProcess`/`\lstnewenvironment`→`\newenvironment`), so a
  /// mode-switching begin/end pair (`\mdframed`…`\endmdframed`, cnltx's
  /// `sourcecode` env, `\begin{minipage}`…) balances. The display wrapper
  /// `{\def\lstname{…} <block>}` had the postamble INSIDE its braces, so
  /// `\endmdframed` met the wrapper's `{` frame ("Attempt to end mode
  /// internal_vertical" — cnltx_en 921×, chemnum 654×, pixelart 703×,
  /// modiagram 896×, tasks 171×; Perl listings.sty.ltxml:205-212 identical).
  #[test]
  fn lstnewenvironment_end_code_runs_outside_the_listing_group() {
    let tex = r"\documentclass{article}
\usepackage{mdframed,listings}
\lstnewenvironment{foo}{\mdframed}{\endmdframed}
\lstnewenvironment{bar}{\begin{minipage}{3cm}}{\end{minipage}}
\begin{document}
Before.
\begin{foo}
code here
\end{foo}
\begin{bar}
more code
\end{bar}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<listing class").count(), 2, "{xml}");
    assert!(
      xml.contains("framed=\"rectangle\""),
      "mdframed block lost: {xml}"
    );
    // the frame WRAPS the listing (the constructor body extends over it)
    let frame = xml.find("framed=\"rectangle\"").unwrap();
    let first_listing = xml.find("<listing class").unwrap();
    assert!(frame < first_listing, "{xml}");
    assert!(xml.contains("<p>After.</p>"), "{xml}");
  }

  /// tex.web §982/§987: `\pagegoal` is `\vsize` once the page has content;
  /// with no page builder the standing value must serve every "free space"
  /// probe — Perl's 0 loops fullwidth.sty:243-273, `\maxdimen` sent
  /// fillwith.sty:319's coffin stacking after a 16384pt goal (TokenLimit).
  /// `\strutbox` is the real latex.ltx:12596 strut (.7/.3 `\baselineskip`),
  /// not void, so `\strut`-based line heights are honest.
  #[test]
  fn pagegoal_is_vsize_and_strutbox_is_real() {
    let tex = r"\documentclass{article}
\begin{document}
[\the\pagegoal][\the\vsize][\the\ht\strutbox,\the\dp\strutbox]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[1000.0pt][1000.0pt]"), "{xml}");
    assert!(xml.contains("[8.39996pt,3.60004pt]"), "{xml}");
  }

  /// fullwidth.sty:243-273 `\fwd@freepagevspace` retries `\vfill\eject` while
  /// `\pagegoal - \pagetotal < 2\baselineskip`; with `\pagegoal=\vsize` the
  /// frame is placed at once (Perl's `\pagegoal=0` loops).
  #[test]
  fn fullwidth_frame_does_not_retry_forever() {
    let tex = r"\documentclass{article}
\usepackage{fullwidth}
\begin{document}
Before.
\begin{fullwidth}
Wide text.
\end{fullwidth}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Not enough space"), "{stderr}");
    assert!(
      xml.contains("Wide text.") && xml.contains("After."),
      "{xml}"
    );
  }

  /// OXIDIZED_DESIGN #176 / KNOWN_PERL_ERRORS #131: a zero-width `\vrule` is a
  /// strut, not a column rule — with the real `\strutbox` every TeXbook
  /// `\halign{\strut#&\vrule#&…}` template (halignatt.tex) otherwise grew an
  /// empty bordered cell per row, and Perl marks the explicit idiom
  /// `border="ll"`.
  #[test]
  fn zero_width_vrule_is_a_strut_not_a_border() {
    let tex = r"\documentclass{article}
\begin{document}
\halign{\vrule height 12pt width 0pt#&\vrule#&#\cr &&a\cr}
\halign{\strut#&\vrule#&#\cr &&b\cr}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<td align="left" border="l" class="ltx_nopad_l ltx_nopad_r">a</td>"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<td align="left" border="l" class="ltx_nopad_l ltx_nopad_r">b</td>"#),
      "{xml}"
    );
    assert_eq!(xml.matches("<td").count(), 2, "{xml}");
  }

  /// latex.ltx:18856 runs the `-h@@k` before `\@popfilename` restores
  /// `\catcode`\@`, so `\AtEndOfPackage` code reads `@`-names as single
  /// control sequences: europecv.cls:27 inputs `ecven.def` from the hook and
  /// its `\ecv@utf` split into `\ecv`+`@utf` looped the title row to the
  /// pushback limit (KNOWN_PERL_ERRORS #132).
  #[test]
  fn at_end_of_package_hook_runs_with_at_letter() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(
      workdir.path().join("hookcls.cls"),
      "\\ProvidesClass{hookcls}\n\
       \\AtEndOfPackage{\\InputIfFileExists{hookcls.def}{}{}}\n\
       \\newcommand\\hook@one{ONE}\n\
       \\LoadClass{article}\n",
    )
    .expect("write cls");
    std::fs::write(
      workdir.path().join("hookcls.def"),
      "\\providecommand\\hooktwo{[\\hook@one]}\n",
    )
    .expect("write def");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{hookcls}\n\\begin{document}\n\\hooktwo\n\\end{document}\n",
    )
    .expect("write tex");
    let output = std::process::Command::new(bin)
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
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[ONE]"), "{xml}");
  }

  /// catchfile.sty:264-296: `\CatchFileDef\cs{file}{setup}` runs `setup`
  /// inside a group and reads the file's tokens under the catcodes it left —
  /// `\catcode`\#=12` (codehigh/fontscale `\dochighinput` reads a .sty whose
  /// `#` would otherwise be a parameter token) and `\endlinechar=-1` — while
  /// `\CatchFileEdef` expands the contents. Both define the target at the
  /// outer level (`\let#1` after `\endgroup`). Witnesses: fontscale-code,
  /// cistercian manuals (codehigh); arXiv 2210.08043, 1611.01359.
  #[test]
  fn catchfiledef_reads_under_setup_catcodes_and_edef_expands() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("caught.txt"), "A#1\\foo B\nC\n").expect("write txt");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{article}\\usepackage{catchfile}\n\
       \\def\\foo{FOO}\n\
       \\begin{document}\n\
       \\CatchFileDef\\raw{caught.txt}{\\catcode`\\#=12 \\endlinechar=-1 }\n\
       \\CatchFileEdef\\exp{caught.txt}{\\catcode`\\#=12 \\endlinechar=-1 }\n\
       [\\detokenize\\expandafter{\\raw}][\\detokenize\\expandafter{\\exp}]\n\
       \\end{document}\n",
    )
    .expect("write tex");
    let output = std::process::Command::new(bin)
      .args(["t.tex", "--dest", "t.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // `\detokenize`'s backslash renders through OT1 as `“`.
    assert!(xml.contains("[A#1“foo BC][A#1FOOBC ]"), "{xml}");
  }

  /// OXIDIZED_DESIGN #177: `\usepackage{../tex/pkg}` (CTAN source layout,
  /// tikzpingus-doc.tex:16 and 60 more manuals) resolves nowhere in the
  /// installed tree; the basename does, so it is loaded instead. A relative
  /// path that DOES resolve still loads the local file.
  #[test]
  fn relative_package_path_falls_back_to_basename() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::create_dir_all(workdir.path().join("local")).expect("mkdir");
    std::fs::write(
      workdir.path().join("local/xspace.sty"),
      "\\ProvidesPackage{xspace}\\newcommand\\localmarker{LOCALXSPACE}\n",
    )
    .expect("write local sty");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{article}\n\
       \\usepackage{../tex/xcolor}\n\
       \\usepackage{./local/xspace}\n\
       \\begin{document}\n\
       \\textcolor{red}{R}\\localmarker\n\
       \\end{document}\n",
    )
    .expect("write tex");
    let output = std::process::Command::new(bin)
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
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r##"color="#FF0000""##), "{xml}");
    assert!(xml.contains("LOCALXSPACE"), "{xml}");
  }

  /// OXIDIZED_DESIGN #178: `\clearpage`/`\newpage` advance `\c@page`
  /// (latex.ltx:15271), so knowledge.tex:803-809's pad-to-page loop
  /// terminates — Perl hangs, Rust's box-cycle guard fataled.
  #[test]
  fn clearpage_advances_the_page_counter() {
    let tex = r"\documentclass{article}
\begin{document}
\newcommand{\filluptopage}[1]{\clearpage\loop\ifnum\value{page}<#1\relax\null\clearpage\repeat}
\filluptopage{4}
Done [\thepage].
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert_eq!(
      xml.matches("<pagination role=\"newpage\"").count(),
      3,
      "{xml}"
    );
    assert!(xml.contains("Done [4]."), "{xml}");
  }

  /// tex.web §977: `\vsplit` stores the remainder at the register's existing
  /// eq_level, so a drain inside `{…}` survives the group — eledmac.sty:1363
  /// `\do@line` relies on it (eledform example: box-list runaway). The
  /// `\ifnum>50` cap turns a regression into a failed assertion, not a hang.
  #[test]
  fn vsplit_drain_survives_the_enclosing_group() {
    let tex = r"\documentclass{article}
\begin{document}
\newbox\rawt\setbox\rawt=\vbox{a\par b\par c}\count255=0
\loop\ifvbox\rawt {\global\setbox0=\vsplit\rawt to 100pt}\advance\count255 by1
  \ifnum\count255>50 \global\setbox\rawt=\box\voidb@x\fi\repeat
[\the\count255]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[1]"), "{xml}");
  }

  /// biblatex.sty:8995-9024 binds the `.bbl` commands (`\list`, `\name`,
  /// `\field`…) only while the `.bbl` is read; a document-wide `\list{}{}{}`
  /// (KNOWN_PERL_ERRORS #133) shadowed LaTeX's `\list{label}{setup}` for
  /// every list environment (cnltx-doc `commands` under `add-bib`).
  #[test]
  fn biblatex_bbl_commands_do_not_shadow_list() {
    let tex = r"\documentclass{article}
\usepackage{biblatex}
\newenvironment{mylist}{\list{}{\leftmargin=0pt}}{\endlist}
\begin{document}
\begin{mylist}\item one\end{mylist}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<itemize"), "{xml}");
    assert!(xml.contains("<item"), "{xml}");
  }

  /// fontspec-xetex.sty:755-767: `\newfontfamily\Foo{…}` DEFINES `\Foo` as a
  /// robust font switch (papiergurvan `\BelleAllureGras`; unicodefonttable's
  /// `\setfontface` target must be non-empty for `\tl_if_empty:NF`).
  #[test]
  fn fontspec_definers_define_a_font_switch() {
    let tex = r"\documentclass{article}
\usepackage{fontspec}
\newfontfamily\Foo[Scale=1.1]{Belle Allure}[Ligatures=TeX]
\setfontface\Bar{Some Font.otf}
\begin{document}
{\Foo abc}{\Bar def}[\ifx\Bar\empty EMPTY\else BODY\fi]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("abcdef[BODY]"), "{xml}");
  }

  /// latex.ltx:18843-18875: `package/<name>/before|after` (and the
  /// `file/<name>.sty/after` form) fire around a package load whichever way
  /// it loads — binding (xspace) or raw (a local .sty). tudapub.cls hooks
  /// scrbook's `\addchap` via `class/scrbook/after` (DEMO-TUDaPhD).
  #[test]
  fn package_after_hook_fires_for_a_binding_load() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(
      workdir.path().join("rawpkg.sty"),
      "\\ProvidesPackage{rawpkg}\\newcommand\\rawmark{RAW}\n",
    )
    .expect("write sty");
    std::fs::write(
      workdir.path().join("t.tex"),
      "\\documentclass{article}\n\
       \\AddToHook{package/xspace/after}{\\def\\afterx{AX}}\n\
       \\AddToHook{package/xspace/before}{\\def\\beforex{BX}}\n\
       \\AddToHook{file/rawpkg.sty/after}{\\def\\afterraw{AR}}\n\
       \\AddToHook{package/rawpkg/after}{\\let\\rawmarktwo\\rawmark}\n\
       \\usepackage{xspace}\\usepackage{rawpkg}\n\
       \\begin{document}\n\
       [\\beforex\\afterx\\afterraw\\rawmarktwo]\n\
       \\end{document}\n",
    )
    .expect("write tex");
    let output = std::process::Command::new(bin)
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
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[BXAXARRAW]"), "{xml}");
  }

  /// chemmacros raw-loads (its stub's `\ch` → `\ensuremath{\mathrm{#1}}`
  /// overrode chemformula's `\ch`: chemformula manual, 90+ errors) and its
  /// `formula=chemformula` method finds the chemformula l3 API
  /// (chemmacros.sty:1358-1366 → `\chemformula_chcpd:nn`).
  #[test]
  fn chemmacros_raw_load_keeps_chemformula_ch() {
    let tex = r"\documentclass{article}
\usepackage{chemformula}
\usepackage{chemmacros}
\begin{document}
\ch{CrO4^2-} \ox{+1,Na} \NMR{1,H} \pH
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<Math"), "{xml}");
    assert!(xml.contains("NMR"), "{xml}");
  }

  /// datetime2 raw-loads (the stub left `\DTMsetup`/`\DTMdate` undefined —
  /// cnltx/chemformula manuals) once `\pdfcreationdate` is pdfTeX's
  /// `D:YYYYMMDD…` stamp (pdfTeX manual §8.11; datetime2.sty:46-48).
  #[test]
  fn datetime2_raw_dates_render() {
    let tex = r"\documentclass{article}
\usepackage[en-GB]{datetime2}
\begin{document}
\DTMsetup{datesep=/}[\DTMdate{2026-09-02}][\DTMdisplaydate{2020}{3}{7}{-1}][\DTMsetdatestyle{iso}\DTMdate{2026-09-02}]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("[2nd September 2026][7th March 2020][2026-09-02]"),
      "{xml}"
    );
  }

  /// OXIDIZED_DESIGN #179: a chapterless class leaves `\chapter` undefined
  /// (latex.ltx/article define none), so `\@ifundefined{chapter}` takes the
  /// article branch — blindtext.sty:243 `\blinddocument` under scrartcl
  /// (hvfloat ×50, coseoul, xassoccnt: `undefined:\thechapter`). A class
  /// with a chapter counter keeps it.
  #[test]
  fn chapter_is_undefined_in_a_chapterless_class() {
    let tex = r"\documentclass{article}
\begin{document}
\makeatletter[\@ifundefined{chapter}{NOCHAP}{CHAP}]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[NOCHAP]"), "{xml}");
    let tex = r"\documentclass{report}
\begin{document}
\makeatletter[\@ifundefined{chapter}{NOCHAP}{CHAP}]
\chapter{One}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[CHAP]") && xml.contains("<chapter"), "{xml}");
    let tex = r"\documentclass{scrartcl}
\usepackage{blindtext}
\begin{document}
\blinddocument
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<section"), "{xml}");
  }

  /// OXIDIZED_DESIGN #180 (P38): a raw `\list` (latex.ltx:15848 verbatim, as
  /// memoir.cls:4580 redefines it) ends in `\@trivlist`, which now opens the
  /// list our `\endlist` closes — memoir `adjustwidth` (digiconfigs, memman)
  /// and hand-rolled `\@trivlist`…`\endtrivlist` pairs (0802.2207
  /// `mathtrivlist`) both nest cleanly.
  #[test]
  fn raw_list_opens_through_trivlist() {
    let tex = r"\documentclass{article}
\makeatletter
\renewcommand*{\list}[2]{\ifnum\@listdepth>5\relax\@toodeep\else\global\advance\@listdepth\@ne\fi
  \rightmargin\z@\listparindent\z@\itemindent\z@
  \csname @list\romannumeral\the\@listdepth\endcsname\def\@itemlabel{#1}\let\makelabel\@mklab
  \@nmbrlistfalse#2\@trivlist\parskip\parsep\parindent\listparindent\advance\linewidth-\rightmargin
  \advance\linewidth-\leftmargin\advance\@totalleftmargin\leftmargin\parshape\@ne\@totalleftmargin\linewidth\ignorespaces}
\newenvironment{adjw}[2]{\begin{list}{}{\topsep\z@}\item[]}{\end{list}}
\newenvironment{mtl}{\@trivlist\item[]}{\endtrivlist}
\makeatother
\begin{document}
\begin{adjw}{1em}{0pt}Inside A\end{adjw}
\begin{mtl}Inside B\end{mtl}
\begin{enumerate}\item one\end{enumerate}
After
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<itemize").count(), 2, "{xml}");
    assert!(
      xml.contains("Inside A") && xml.contains("Inside B") && xml.contains("<enumerate"),
      "{xml}"
    );
  }

  /// tcolorbox.sty:2339 `\tcb@proc@options@init` processes a listing env's
  /// `[init]` (`auto counter`, `number within`), so a later
  /// `\newtcolorbox[use counter from=<env>]` finds `\tcb@cnt@<env>`
  /// (tcolorbox manual preamble D: `texexptitledspec` from `texexptitled`).
  #[test]
  fn tcblisting_init_counter_is_shared_by_use_counter_from() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}\tcbuselibrary{listings}
\tcbset{example/.style 2 args={title={Example \thetcbcounter: #1},label={#2}}}
\newtcblisting[auto counter,number within=section]{texexptitled}[3][]{example={#2}{#3},#1}
\newtcolorbox[use counter from=texexptitled]{texexptitledspec}[3][]{example={#2}{#3},#1}
\begin{document}
\section{S}
\begin{texexptitled}{T1}{l1}
x
\end{texexptitled}
\begin{texexptitledspec}{T2}{l2}y\end{texexptitledspec}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The listing box renders no title (presentation-only in the native env);
    // the shared counter stepped once for it, so the tcolorbox is 1.2.
    assert!(xml.contains("Example 1.2"), "{xml}");
  }

  /// nicematrix.sty:1953/3665: `NiceArray` takes `[opts]{cols}[opts]`; the
  /// leading optional was read as the preamble (nicematrix.tex:409 →
  /// `Unrecognized tabular template "["`, 57 extra `&`, `Until:\Body` EOF).
  #[test]
  fn nicearray_takes_a_leading_option_list() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
$\begin{NiceArray}[t]{lcc}[no-cell-nodes]
n & 0 & 1 \\
u & 2 & 3 \\
\end{NiceArray}$
$\begin{pNiceArray}{cc}[first-col] a & b \\ \end{pNiceArray}$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("Unrecognized tabular template"),
      "{stderr}"
    );
    assert_eq!(xml.matches("<XMArray").count(), 2, "{xml}");
  }

  /// OXIDIZED_DESIGN #181: a `\\` inside a brace group of a cell is an
  /// in-cell break, not a row end (latex.ltx:16583 `{\ifnum0=`}\fi` keeps
  /// `\cr` from firing at align_state≠0; tabularray makes it a line break —
  /// ProfSio.sty:2917 `\SetCell{l}{… \\ …}`). Ending the row misread the
  /// cell's `}` as the alignment's `\egroup` (3 errors per cell; Perl
  /// truncates the table). An empty cell before `\\` still ends the row.
  #[test]
  fn newline_inside_a_cell_group_is_an_in_cell_break() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{colspec={XQ[3cm]},hlines}
\SetCell[c=2]{c}{S} & \\
NOM : X & \SetCell{l}{A\\B} \\
{Y \\} & C \\
\end{tblr}
\begin{tabular}{ll}
S & \\
{Y \\} & C \\
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(
      xml.matches("<tr>").count() + xml.matches("<tr ").count(),
      5,
      "{xml}"
    );
    assert!(xml.contains("A<break/>B"), "{xml}");
    assert!(
      xml.contains("Y <break/>") || xml.contains("Y<break/>"),
      "{xml}"
    );
  }

  /// latex.ltx:14060/14131: a `\newcommand` optional default passes through
  /// two `\def` bodies, so `[########1]` reaches the macro as `##1`
  /// (pdflatex-probed). etoolbox/biditools `\patchcmd` builds on it;
  /// biditools' load errored `misdefined:#` (crbox, lineno, multiple-choice …).
  #[test]
  fn newcommand_default_halves_param_tokens_twice() {
    let tex = r"\documentclass{article}
\newcommand{\foo}[2][########1]{[\detokenize{#1}|#2]}
\begin{document}
\foo{A} \foo[x]{B}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // OT1 renders `|` as an em-dash.
    assert!(xml.contains("[####1—A] [x—B]"), "{xml}");
    let tex = r"\documentclass{article}
\usepackage{biditools}
\begin{document}
x
\end{document}
";
    let (stderr, _xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// A binding-loaded package exposes the installed file's
  /// `\ProvidesPackage` version in `\ver@<name>.sty` (setspace-doc.tex:60-64
  /// splits it at spaces with `\def\pkginfo#1 #2 #3\relax`; a space-free
  /// `\fmtversion` ran to EOF).
  #[test]
  fn binding_ver_macro_carries_the_installed_provides_version() {
    let tex = r"\documentclass{article}
\usepackage{setspace}
\makeatletter
\def\pkginfo#1 #2 #3\relax{\def\filedate{#1}\def\fileversion{#2}}
\expandafter\expandafter\expandafter\pkginfo\csname ver@setspace.sty\endcsname\relax
\makeatother
\begin{document}
[\filedate][\fileversion]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[20") && xml.contains("][v"), "{xml}");
  }

  /// latex.ltx:16187 `\secdef#1#2` = `\@ifstar{#2}{\@dblarg{#1}}` (Perl drops
  /// the `\@dblarg`; memoir.cls:2787 `\book` ran to EOF, srbook-mem ×3).
  #[test]
  fn secdef_doubles_the_title_for_the_unstarred_form() {
    let tex = r"\documentclass{article}
\makeatletter
\long\def\@bk[#1]#2{[BK:#1|#2]}
\def\@sbk#1{[SBK:#1]}
\newcommand*{\bk}{\secdef\@bk\@sbk}
\makeatother
\begin{document}
\bk{Ovo} \bk[short]{Long} \bk*{Star}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("[BK:Ovo—Ovo] [BK:short—Long] [SBK:Star]"),
      "{xml}"
    );
  }

  /// Real microtype.sty:80 defines only the `\microtypecontext{…}`
  /// declaration — no environment, so `\endmicrotypecontext` is undefined
  /// and synthslant.sty:302's `\ifcsdef{endmicrotypecontext}` takes the
  /// false branch (a live env-end errored "Attempt to end mode", ×101 in
  /// synthslant-gauge). The env form still works through `\begin`/`\end`.
  #[test]
  fn microtypecontext_is_a_declaration_not_an_environment() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{microtype}
\NewDocumentEnvironment{slantenv}{}
  {\ifcsdef{microtypecontext}{\microtypecontext{tracking=x}}{}}
  {\ifcsdef{endmicrotypecontext}{\endmicrotypecontext}{}}
\begin{document}
\begin{slantenv}Hello\end{slantenv}
\begin{microtypecontext}{tracking=y}World\end{microtypecontext}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hello") && xml.contains("World"), "{xml}");
  }

  /// beamerbaseoptions.sty:34-38: theme options are keyval, and the themes'
  /// internals come from `\ExecuteOptionsBeamer` defaults
  /// (beamerouterthemesidebar.sty:30-32 `\beamer@sidebarside`,
  /// beamerinnerthemerounded.sty:11-12 `\beamer@themerounded@shadow`);
  /// beamerbaseframe.sty:730 creates the `framenumber` counter
  /// (appendixnumberbeamer.sty:43).
  #[test]
  fn beamer_theme_option_defaults_define_their_internals() {
    for theme in ["Berkeley", "Madrid"] {
      let tex = format!(
        "\\documentclass{{beamer}}\n\\usetheme{{{theme}}}\n\\begin{{document}}\n\\begin{{frame}}{{Title}}Hello \\theframenumber\\end{{frame}}\n\\end{{document}}\n"
      );
      let (stderr, xml) = convert(&tex, false);
      assert_eq!(error_count(&stderr), 0, "{theme}: {stderr}");
      assert!(xml.contains("Hello"), "{xml}");
    }
  }

  /// beamerthemeVerona.sty:174-190 uses `\addtobeamertemplate{background}{...}{}`
  /// and `\newcommand<>{\sidegraphics}[3][]{...}` with an optional default argument.
  /// `\addtobeamertemplate` executes at frame start and `\newcommand<>` / `\newenvironment<>`
  /// pack and remap overlay and optional-default arguments so the frame closes cleanly.
  #[test]
  fn beamer_frame_sidebar_overlay_template() {
    let tex = r"\documentclass{beamer}
\usetheme[sidebar]{Verona}
\begin{document}
\begin{frame}\sidegraphics<1>{plato}{scale=1.1}\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<subsection"), "{xml}");
  }

  /// numprint.sty:779 `\DeclareRobustCommand*\numprint`: a `\the\toks255`
  /// register-number lookahead (tex.web §440-448) stops at `\protect`
  /// instead of pre-expanding the `\ifmmode` dispatch into the stored list
  /// (calctab.sty:334-335; calctab manual: 94 "Extra \or already saw \else").
  #[test]
  fn numprint_is_robust_under_a_the_toks_lookahead() {
    let tex = r"\documentclass{article}
\usepackage{numprint}
\begin{document}
\toks0={}\edef\r{\noexpand\numprint{12500.90}}
\toks0=\expandafter\expandafter\expandafter{\expandafter\the\expandafter\toks0\r}
[\the\toks0]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("ltx_number").count(), 1, "{xml}");
  }

  /// showexpl.sty:66-86 load-time state survives a document that rebuilds
  /// `LTXexample` from the internals (lshort-german l2kurz.tex:73-100 —
  /// `\def\SX@codefile{\SX@codefile}` "expands into itself" ×96).
  #[test]
  fn showexpl_internals_exist_for_rebuilt_ltxexample() {
    let tex = r"\documentclass{article}
\usepackage{showexpl}
\makeatletter
\begingroup
\edef\x{\endgroup\def\noexpand\SX@codefile{\SX@codefile}}
\x
\begin{document}
Codefile:[\SX@codefile]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(".tmp]"), "{xml}");
  }

  /// newverbs.sty:52-69: `\newverbcommand{\cverb}{before}{after}` wraps a
  /// verbatim argument; the real command's extra `\bgroup` is closed by
  /// `\verb@egroup`, which a native `\verb` never runs (homework.cls demos:
  /// "Attempt to end mode internal_vertical" at the next `\end{…}`).
  #[test]
  fn newverbcommand_wraps_the_verb_body() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{newverbs}
\newverbcommand{\cverb}{\color{red}}{}
\begin{document}
\begin{quote}
Use \cverb|\qedhere| here and \qverb|x|.
\end{quote}
done
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<quote"), "{xml}");
    assert!(xml.contains(r"\qedhere") && xml.contains("done"), "{xml}");
    assert!(xml.contains("color=\"#FF0000\""), "{xml}");
  }

  /// Bare `\flushleft`…`\endflushleft` (comment.tex:12-18 `noverb`,
  /// bidicode.sty:195 `BDef`): the declaration opens no frame, so its
  /// `\end…` partner is a no-op; `\begin{flushleft}` still aligns.
  #[test]
  fn bare_endflushleft_is_a_noop() {
    let tex = r"\documentclass{article}
\newenvironment*{noverb}{\flushleft}{\endflushleft}
\begin{document}
Text.
\begin{noverb}
content
\end{noverb}
\begin{flushleft}left\end{flushleft}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("content") && xml.contains(r#"<p align="left">left</p>"#),
      "{xml}"
    );
  }

  /// listings.sty:1968 `\lst@InlineG`: a `{`-delimited `\lstinline` ends at
  /// the balanced `}` (coolfn `\mintinline{latex}{\renewcommand{\fnindent}{1.25em}}`).
  #[test]
  fn lstinline_brace_delimiter_is_balanced() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\lstinline{\renewcommand{\fnindent}{1.25em}}. Then \lstinline|a{b|.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The inline listing is token-marked-up; the group and the unit survive.
    assert!(xml.contains("fnindent</text>}{1.25"), "{xml}");
    assert!(xml.contains("a</text>{<text"), "{xml}");
  }

  /// `\centering`/`\raggedright` are macros (latex.ltx:16419-16433): expl3's
  /// V-expansion register test (expl3-code.tex:2507-2517) must not `\the` a
  /// `\let\raggedsignature=\centering` (DIN.lco:130; scrlttr2.cls:5095
  /// `\closing`: KOMA letters ×5).
  #[test]
  fn centering_is_expandable_for_expl3_v_expansion() {
    let tex = r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\let\raggedsignature=\centering
\tl_if_in:nVTF { \raggedright\LaTeXraggedright } \raggedsignature
  { \def\got{L} } { \def\got{NOTL} }
\ExplSyntaxOff
\begin{document}
[\got]\begin{center}c\end{center}{\raggedleft r\par}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[NOTL]"), "{xml}");
    assert!(
      xml.contains(r#"align="center""#) && xml.contains("ltx_align_right"),
      "{xml}"
    );
  }

  /// OXIDIZED_DESIGN #182: a `\caption` with `\@captype` set inside an `lrbox`
  /// minipage (tufte-common.def:1110-1133 `marginfigure`: pgfornament 40+40,
  /// memman 46+46 errors) has no float ancestor; it degrades to the inline
  /// `ltx_caption` text instead of `<ltx:caption> isn't allowed in <ltx:block>`.
  /// A real `figure` keeps the tagged `ltx:caption` + `ltx:toccaption`.
  #[test]
  fn caption_without_a_float_ancestor_degrades_to_text() {
    let tex = r"\documentclass{article}
\makeatletter
\newsavebox\mybox
\newenvironment{marginfig}{\begin{lrbox}{\mybox}\begin{minipage}{3cm}\def\@captype{figure}}{\end{minipage}\end{lrbox}\marginpar{\usebox{\mybox}}}
\makeatother
\begin{document}
\begin{marginfig}
X
\caption{A caption}
\end{marginfig}
\begin{figure}\centering Y\caption{Real float}\end{figure}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<text class="ltx_caption">A caption</text>"#),
      "{xml}"
    );
    assert!(
      xml.contains(
        r#"<caption class="ltx_centering"><tag close=": ">Figure 2</tag>Real float</caption>"#
      ),
      "{xml}"
    );
    assert!(xml.contains("<toccaption"), "{xml}");
  }

  /// KNOWN_PERL_ERRORS #140: `\index{packages!#1@\texttt{#1}}` with `#1` =
  /// `\TIKZ` (pgfornament usefulcommands.tex:93) must re-read as `\TIKZ` + `@`
  /// (the `.idx` `\write` form), not the undefined `\TIKZ@`.
  #[test]
  fn index_control_word_before_at_is_not_glued() {
    let tex = r"\documentclass{article}
\usepackage{makeidx}
\makeindex
\newcommand*{\TIKZ}{Ti\emph{k}Z}
\newcommand{\docpkg}[1]{\texttt{#1}\index{#1 package@\texttt{#1} package}\index{packages!#1@\texttt{#1}}}
\begin{document}
Uses \docpkg{\TIKZ} here.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<indexphrase key="packages">packages</indexphrase>"#),
      "{xml}"
    );
    assert!(xml.contains(r#"<indexphrase key="Ti\emph{k}Z">"#), "{xml}");
  }

  /// KNOWN_PERL_ERRORS #141: `\renewcommand\part{\secdef\@part\@spart}` and a
  /// document-made `\chapter` (source3body.tex:96-123: l3kernel interface3 +
  /// source3, 2 → 101 errors) find the class-level workers and an unlocked
  /// `\chapter` in a chapterless class.
  #[test]
  fn secdef_part_and_chapter_workers_exist() {
    let tex = r"\documentclass{article}
\makeatletter
\renewcommand\part{\par\secdef\@part\@spart}
\newcounter{chapter}
\renewcommand\thesection{\thechapter.\@arabic\c@section}
\newcommand\chapter{\clearpage\secdef\@chapter\@schapter}
\makeatother
\begin{document}
\part{First part}
\chapter{A chapter}
\section{A section}
\chapter*{Unnumbered}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<part inlist="toc" xml:id="Pt1">"#), "{xml}");
    assert!(
      xml.contains(r#"<chapter inlist="toc" xml:id="chapter1">"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<tag close=" ">1.1</tag>A section"#),
      "{xml}"
    );
    assert!(xml.contains("<title>Unnumbered</title>"), "{xml}");
  }

  /// The German `"` shorthands belong to babel's German, not only to
  /// `\usepackage{german}`: `\usepackage[ngerman]{babel}` (80 TL manuals)
  /// rendered `Sch"one` as `Sch”one`, and `\mdqon` errored `T_ACTIVE["]`.
  /// Non-shorthand follow-characters print the quote itself
  /// (pdflatex `A "x" B "1"` → `A "x" B "1"`), `"ck`/`"ff` the letter.
  #[test]
  fn babel_ngerman_umlaut_shorthands() {
    let tex = r#"\documentclass{article}
\usepackage[ngerman]{babel}
\begin{document}
Sch"one Gr"u"se "`Zitat"' A "x" B "1" C "ck D "ff E {\mdqoff "y"} \mdqon "a
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Schöne Grüße „Zitat“ A \"x\" B \"1\" C ck D ff E ”y” ä"),
      "{xml}"
    );
  }

  /// german.sty's `\germanTeX` (german.sty:666-671) — run by the kernel first
  /// aid `file/german.sty/after` (latex2e-first-aid-for-external-files.ltx:160)
  /// and by documents written for german.sty (a0poster a0/a0_eng, adrconv,
  /// akletter … 16 TL manuals with `\ngermanTeX`).
  #[test]
  fn german_sty_germantex_switch_is_defined() {
    let tex = r#"\documentclass{article}
\usepackage{german}
\begin{document}
Sch"one \germanTeX Gr"u"se
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Schöne Grüße"), "{xml}");
    let tex = tex
      .replace("{german}", "{ngerman}")
      .replace(r"\germanTeX", r"\ngermanTeX");
    let (stderr, xml) = convert(&tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Schöne Grüße"), "{xml}");
  }

  /// beamer builds on article (Perl beamer.cls.ltxml:1361 `LoadClass`); the
  /// binding's `RequirePackage!("article")` missed silently, leaving
  /// `\subsection` with `undefined:\thesubsection` (bfh-ci DEMO-BFHBeamer,
  /// metropolis/gotham demos).
  #[test]
  fn beamer_has_article_sectioning_counters() {
    let tex = r"\documentclass{beamer}
\begin{document}
\section{Introduction}
\begin{frame}{A}x\end{frame}
\subsection{Sub}
\begin{frame}{B}y\end{frame}
\section{Second}
\subsection{Sub two}
\begin{frame}{C}z\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<subsection inlist="toc" xml:id="S1.SS1">"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<subsection inlist="toc" xml:id="S2.SS1">"#),
      "{xml}"
    );
  }

  /// A `\bibliography` inside a beamer frame (metropolis demo, simpleplus /
  /// simpledarkblue / pure-minimalistic samples): the frame's `ltx:subsection`
  /// never auto-closes, so placing the bibliography "as an `ltx:section`" erred
  /// `<ltx:section> isn't allowed in <ltx:p>` and left it inside the `<p>`.
  /// The subsection may hold an `ltx:bibliography`, which is where beamer
  /// typesets it (`backmatter_insertion_target`).
  #[test]
  fn bibliography_inside_a_beamer_frame_stays_in_the_frame() {
    let tex = r"\documentclass{beamer}
\begin{document}
\section{Intro}
\begin{frame}{A}x\end{frame}
\begin{frame}{References}
  \bibliography{nonexistent}
  \bibliographystyle{abbrv}
\end{frame}
\begin{frame}{After}z\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("</p>\n      </para>\n      <bibliography"),
      "{xml}"
    );
    assert!(xml.contains("</bibliography>\n    </subsection>"), "{xml}");
  }

  /// nameref.sty:189-192 `\NR@gettitle` (memoir.cls:7025 routes `\M@gettitle`
  /// — heads, `\PoemTitle` — through it; srbook-mem Test/TestLight/
  /// SerbianBookMem, serbian-apostrophe ×2: sole error).
  #[test]
  fn nameref_gettitle_records_the_title() {
    let tex = r"\documentclass{article}
\usepackage{nameref}
\begin{document}
\makeatletter
\NR@gettitle{Guarded Title}[\@currentlabelname]
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[Guarded Title]"), "{xml}");
  }

  /// fancyhdr.sty:577-608 `\f@nch@initialise` — executed by ctex's
  /// end-of-package hook (ctex-heading-article.def:686; inkpaper, sduthesis,
  /// shtthesis, caspervector) after patching it.
  #[test]
  fn fancyhdr_initialise_is_defined() {
    let tex = r"\documentclass{article}
\usepackage{fancyhdr}
\pagestyle{fancy}
\makeatletter
\f@nch@initialise
\makeatother
\begin{document}
\section{One}
x
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>x</p>"), "{xml}");
  }

  /// biblatex.sty:16439-16440 loads the bbx before the cbx (oxref.bbx:489
  /// `\newtoggle` vs oxnum.cbx:26 `\providetoggle`), and the raw style chain's
  /// declaration-only commands (`\DeclareDataInheritance`, `\NumCheckSetup`,
  /// `\defbibfilter`, `\defbibnote` …) are accepted (biblatex-oxref ×4,
  /// biblatex-cse-doc, biblatex-musuos).
  #[test]
  fn biblatex_loads_bbx_before_cbx() {
    let tex = r"\documentclass{article}
\usepackage[style=oxnum]{biblatex}
\defbibfilter{books}{type=book}
\defbibnote{pre}{A note.}
\begin{document}
Hello.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>Hello.</p>"), "{xml}");
  }

  /// titlesec.sty:112-165 `\titleclass` defines a NEW heading command
  /// (regulatory.sty:116/121 `\article`/`\para`; regulatory example1/2 ×4).
  #[test]
  fn titleclass_defines_a_new_heading_command() {
    let tex = r"\documentclass{article}
\usepackage{titlesec}
\newcounter{article}
\titleclass{\article}[0]{straight}
\newcounter{para}
\titleclass{\para}{straight}[\article]
\begin{document}
\article{Hello}
\para{World}
Text.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<section"), "{xml}");
    assert!(xml.contains("Hello</title>"), "{xml}");
    assert!(xml.contains("<subsection"), "{xml}");
    assert!(xml.contains("World</title>"), "{xml}");
  }

  /// caption3.sty:1753 `\DeclareCaptionType` lazy-loads newfloat and delegates,
  /// and newfloat.sty:117-125 reads the trailing `[singular][listname]`
  /// (pygmentex.sty:23; pygmentex ×2, hvpygmentex).
  #[test]
  fn declare_caption_type_makes_a_float() {
    let tex = r"\documentclass{article}
\usepackage{caption}
\DeclareCaptionType{pygcode}[Listagem][Lista de listagens]
\begin{document}
\begin{pygcode}code\caption{A code listing}\end{pygcode}
[\pygcodename/\listpygcodename]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<float class="ltx_float_pygcode""#), "{xml}");
    assert!(xml.contains("[Listagem/Lista de listagens]"), "{xml}");
    assert!(!xml.contains("[Listagem][Lista"), "{xml}");
  }

  /// KNOWN_PERL_ERRORS #142: `\valign{…}` consumes its alignment
  /// (fancyvrb.sty:570 `\FancyVerbTab`: one `#`-reaches-stomach error per
  /// tab-bearing `Verbatim` line under `showtabs`, pygmentex_demo).
  #[test]
  fn valign_swallows_its_alignment() {
    let tex = "\\documentclass{article}
\\usepackage{fancyvrb}
\\begin{document}
\\begin{Verbatim}[showtabs,tabsize=1]
A\tB
\\end{Verbatim}
\\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"class="ltx_verbatim""#) && xml.contains("A<text"),
      "{xml}"
    );
  }

  /// The `@`-is-a-letter sibling of KNOWN_PERL_ERRORS #140 (pgfmanual-en-macros
  /// .tex:281 `\index{Internals!\strippedat @…}` under `\makeatletter`:
  /// tikz-cd-doc, tikz-dependency-doc, pdfmarginpar): the print_cs space must
  /// not depend on `@`'s catcode.
  #[test]
  fn index_control_word_before_letter_at_is_not_glued() {
    let tex = r"\documentclass{article}
\usepackage{makeidx}
\makeindex
\makeatletter
\def\strippedat{foo}
\def\extractinternalcommand{\index{Internals!\strippedat @\protect\texttt{\strippedat}}}
\makeatother
\begin{document}
\extractinternalcommand Text.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<indexphrase key="foo">"#), "{xml}");
  }

  /// latex.ltx:9512/9537: `\document` is preamble-only and its hooks one-time —
  /// a second `\begin{document}` (ltnews.tex:236/296, l3news.tex:109/177
  /// `\renewenvironment{document}` + per-issue `\input`) re-fired csquotes'
  /// end-preamble block whose hooks are `\undef`ed after use (csquotes.sty:2434-2446).
  #[test]
  fn second_begin_document_fires_no_hooks() {
    let tex = r"\documentclass{article}
\usepackage{csquotes}
\usepackage{hyperref}
\begin{document}
Hello \enquote{world}.
\begin{document}
Second begin.
\end{document}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Second begin."), "{xml}");
    assert_eq!(xml.matches("<document ").count(), 1, "{xml}");
  }

  /// pdfcomment annotations become `ltx:note`s (pdfcomment example ×3: raw
  /// pdfcomment.sty took the dvips `\pdfmark` branch and dumped PDF
  /// dictionaries into the text); `\pdfstringdef` is global (hyperref.sty:386).
  #[test]
  fn pdfcomment_annotations_are_notes() {
    let tex = r"\documentclass{article}
\usepackage[author={Me}]{pdfcomment}
\begin{document}
A\pdfcomment[color=red,subject={S},deadline={2009/11/11}]{Hello comment.} B
\pdftooltip{visible}{tip text} $x\pdftooltip{y}{math tip}$
\pdfmarkupcomment[markup=Highlight]{marked}{note}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<note role="pdfcomment">Hello comment.</note>"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"visible<note role="tooltip">tip text</note>"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"marked<note role="pdfmarkupcomment">note</note>"#),
      "{xml}"
    );
    assert!(!xml.contains("pdfmark="), "{xml}");
  }

  /// memoir.cls:2640-2672 patches `\title`/`\author` to set `\thetitle`/
  /// `\theauthor` (biblatex-oxref docs typeset them on their own title page).
  #[test]
  fn memoir_title_defines_thetitle() {
    let tex = r"\documentclass{memoir}
\title{My Title\thanks{T}}\author{An Author}
\begin{document}
[\thetitle/\theauthor]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[My Title/An Author]"), "{xml}");
  }

  /// mathtools.sty:1576 `\mathmakebox[<width>]`: the width is a <dimen>
  /// (`\widthof{$x$}` measured), not content (optidef `\bodySubjectTo` in
  /// `align*`, 58 errors).
  #[test]
  fn mathmakebox_width_is_measured_not_typeset() {
    let tex = r"\documentclass{article}
\usepackage{amsmath,mathtools,calc}
\begin{document}
\begin{align*}
a &= \mathmakebox[\widthof{$x$}][c]{y} b \\
c &= \mathmakebox[2em]{d} \mathmakebox[][c]{e}
\end{align*}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(">y<") && xml.contains(">d<") && xml.contains(">e<"),
      "{xml}"
    );
  }

  /// scrlfile.sty raw: `\BeforePackage`/`\AfterPackage` are the kernel file
  /// hooks (scrlfile-hook.sty:85-230). scrbook.cls:5466-5477 pairs them to
  /// save/restore `\@addchap` around hyperref — the absorbed "before" left
  /// `\addchap` undefined after `\usepackage{hyperref}` (cleanthesis, bfh-ci).
  #[test]
  fn scrlfile_before_and_after_package_hooks_fire() {
    let tex = r"\documentclass{article}
\usepackage{scrlfile}
\makeatletter
\BeforePackage{hyperref}{\def\before@ran{yes}}
\AfterPackage{hyperref}{\def\after@ran{yes}}
\AfterPackage*{hyperref}{\def\afterstar@early{yes}}
\makeatother
\usepackage{hyperref}
\makeatletter
\AfterPackage*{hyperref}{\def\afterstar@late{yes}}
\makeatother
\begin{document}
\makeatletter
[\before@ran/\after@ran/\afterstar@early/\afterstar@late]
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[yes/yes/yes/yes]"), "{xml}");
    let tex = r"\documentclass{scrbook}
\usepackage{hyperref}
\begin{document}
\addchap{Declaration}
Text.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<title>Declaration</title>"), "{xml}");
  }

  /// makeidx.sty defines no `\makeindex` (makeidx.sty:44-51); the binding's
  /// no-op clobbered the kernel's `\@indexfile` allocation that manyind /
  /// robustindex write to (mindsample, robustmanual, multisample).
  #[test]
  fn makeidx_keeps_the_allocating_makeindex() {
    let tex = r"\documentclass{article}
\usepackage{makeidx}
\makeindex
\begin{document}
\makeatletter
\protected@write\@indexfile{}{payload}%
\ifdefined\@indexfile STREAMDEFINED\fi
\makeatother
\index{alpha}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("STREAMDEFINED"), "{xml}");
    assert!(xml.contains(r#"<indexphrase key="alpha">"#), "{xml}");
    assert!(!xml.contains("payload"), "{xml}");
  }

  /// PLANS P37: a `{lstlisting}` in a tabbing field / `l` cell is wrapped in
  /// an auto-opened `ltx:inline-block` (the `p{}`-column shape) instead of
  /// `<ltx:listing> isn't allowed in <ltx:td>` (engtlc ×2, lexref,
  /// expex-glossonly).
  #[test]
  fn listing_in_a_tabular_cell_gets_an_inline_block() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\begin{tabbing}
\hspace{3cm}\=\kill
\begin{lstlisting}
$x$
\end{lstlisting} \> value
\end{tabbing}
\begin{tabular}{ll}
\begin{lstlisting}
code
\end{lstlisting} & right \\
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<inline-block").count(), 2, "{xml}");
    assert!(
      xml.contains("<inline-block>\n            <listing")
        || xml.contains("<inline-block><listing"),
      "{xml}"
    );
  }

  /// tex.web §1211: the register reader skips spaces/`\relax` and absorbs
  /// `\global` (a0poster.cls.ltxml `\setlength { \paperwidth }{…}`:
  /// modernposter; xtab.sty:146 `\setlength{\global\ST@toadd}{#1}`: rec-thy,
  /// altverse).
  #[test]
  fn variable_reader_skips_spaces_and_takes_prefixes() {
    let tex = r"\documentclass{article}
\usepackage{xtab}
\newlength{\mylen}
\begin{document}
\setlength { \mylen }{ 5pt }%
{\setlength{\global\mylen}{7pt}}[\the\mylen]
\begin{xtabular}{l}a\\[6pt]b\\ \end{xtabular}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[7.0pt]"), "{xml}");
    assert_eq!(xml.matches("<tr>").count(), 2, "{xml}");
  }

  /// beamerbasetitle.sty:148/169/233/238: `\inst{n}` is `\textsuperscript{n}`
  /// (detlevcm, beamerstructure2), `\partpage` (beamerbasetitle.sty:30) re-shows
  /// the part page, and beamer.cls:32-49 declares the sidebar/margin dimension
  /// family themes read (beamerthemeVerona.sty:287).
  #[test]
  fn beamer_inst_partpage_and_sidebar_dimens() {
    let tex = r"\documentclass{beamer}
\title{T}
\author{Alice\inst{1} \and Bob\inst{2}}
\institute{\inst{1}Univ A \and \inst{2}Univ B}
\makeatletter
\newlength{\myx}
\setlength{\myx}{\dimexpr(\paperwidth-\beamer@rightsidebar-2mm)}
\makeatother
\begin{document}
\begin{frame}\titlepage\end{frame}
\part{Background}
\begin{frame}\partpage\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<sup>1</sup>Univ A"), "{xml}");
    assert!(xml.contains("<tag>Part I</tag>"), "{xml}");
  }

  /// tabu.sty:6-8 `\begin{tabu} to <dimen>{cols}`: the `to` prefix and `X`
  /// columns (brandeis-problemset example.tex:228, 41 errors).
  #[test]
  fn tabu_to_width_and_x_columns() {
    let tex = r"\documentclass{article}
\usepackage{tabu}
\begin{document}
\begin{tabu} to 0.25\linewidth{X[1,$]rr}
a & b & c \\
\end{tabu}
\begin{tabu}{lX}
d & e \\
\end{tabu}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<td").count(), 5, "{xml}");
  }

  /// xkeyval.tex:38 `\let\XKeyValLoaded\endinput`: expex.tex:65 must not
  /// re-input raw xkeyval over the binding (fragoli, rainbowbrackets:
  /// `undefined:\ep@preambleanchor` on a `\pex` with preamble text).
  #[test]
  fn xkeyval_sets_the_loaded_sentinel() {
    let tex = r"\documentclass{article}
\usepackage{expex}
\begin{document}
\pex
This is a preamble.
\a First item.
\b Second item.
\xe
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("This is a preamble."), "{xml}");
    assert!(xml.contains("First item."), "{xml}");
  }

  /// biblatex.sty:809-870 declares its counters with `\newcounter` (fiwi.bbx:59
  /// `\defcounter{lownamepenalty}` → "No counter defined"; biblatex-fiwi ×3);
  /// blx-compat.def:155 `\AtBeginShorthands` (philosophy/windycity styles);
  /// hyperref.sty:237 `\Hy@AtBeginDocument` (biblatex2bibitem ×2).
  #[test]
  fn biblatex_counters_hooks_and_hy_atbegindocument() {
    let tex = r"\documentclass{article}
\usepackage[colorlinks]{hyperref}
\usepackage{biblatex}
\AtBeginShorthands{\relax}
\makeatletter
\defcounter{lownamepenalty}{0}
\Hy@AtBeginDocument{\def\@pdfborder{0 0 1}}
\makeatother
\setcounter{lownamepenalty}{5}
\begin{document}
[\arabic{lownamepenalty}/\arabic{maxnames}] \href{http://x}{y}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[5/3]"), "{xml}");
    assert!(xml.contains(r#"href="http://x""#), "{xml}");
  }

  /// KNOWN_PERL_ERRORS #145: the frontmatter copy of `\title`/`\author`/`\date`
  /// comes from the stored (once-halved) macro — the RCS-keyword idiom
  /// `\date{\def\$##1: ##2 ##3${##2}…}` (ulineno.tex:16) put a literal `#`
  /// in the stomach.
  #[test]
  fn frontmatter_copies_the_halved_macro() {
    let tex = r"\documentclass{article}
\date{\def\$##1: ##2 ##3${##2}%$
   Version \$Revision: 3.1 $, \$Date: 2001/08/03 03:29:19 $
}
\title{T\def\x##1{##1}\x{ok}}\author{A \and B\thanks{t}}
\begin{document}
\maketitle
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Version 3.1, 2001/08/03"), "{xml}");
    assert!(xml.contains("<title>Tok</title>"), "{xml}");
    assert_eq!(xml.matches("<personname>").count(), 2, "{xml}");
  }

  /// pgfplotscore.code.tex:74-89 `\pgfplotsenablelua{0}`: under the `[luatex]`
  /// profile `\directlua` exists but pgfplots' Lua bootstrap cannot run here
  /// (colorblind_doc: `\pgfplotsglobalretval`, `\pgfplotsutil@savecatcodetable`).
  #[test]
  fn pgfplots_lua_backend_is_off_under_luatex_profile() {
    let tex = r"\documentclass{article}
\usepackage{pgfplots}
\pgfplotsset{compat=1.18}
\begin{document}
\begin{tikzpicture}\begin{axis}\addplot {x^2};\end{axis}\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<svg"), "{xml}");
  }

  /// url.sty:84: `\` is literal inside `\url`/`\path` (latex4wp.tex:451
  /// `\path{C:\localtexmf\tex\}` swallowed the rest of the manual).
  #[test]
  fn url_backslash_is_literal() {
    let tex = r"\documentclass{article}
\usepackage{url}
\begin{document}
See the path \path{C:\localtexmf\tex\} here. \url{http://x/a_b#c\}
More text after it.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r"C:\localtexmf\tex\</text>") || xml.contains(r"C:\localtexmf\tex\<"),
      "{xml}"
    );
    assert!(xml.contains("More text after it."), "{xml}");
  }

  /// `\index{foo@\string\verb\string"bar}` (amsldoc.cls `\cs`; amsldoc-it/-vn):
  /// a `\verb` "delimited" by a control sequence is index text, not verbatim.
  #[test]
  fn index_verb_followed_by_cs_is_text() {
    let tex = r#"\documentclass{article}
\usepackage{makeidx}
\makeindex
\begin{document}
Text\index{foo@\string\verb\string"bar}. More text here.
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<indexmark"), "{xml}");
    assert!(xml.contains("More text here."), "{xml}");
  }

  /// physics2 is its own package, not "physics v2": the glued-suffix fallback
  /// loaded the physics binding (`undefined:\usephysicsmodule`, every
  /// `\ab`/`\bra`/`\ket`; physics2 manuals, whatsnote). Registered
  /// INTERPRETABLE, it raw-loads even without `--includestyles`.
  #[test]
  fn physics2_is_not_a_version_of_physics() {
    let tex = r"\documentclass{article}
\usepackage{physics2}
\usephysicsmodule{ab,braket}
\begin{document}
\[ \ab(x) \quad \bra{\psi}\ket{\phi} \braket{\psi}{\phi} \]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"name="rangle""#), "{xml}");
    assert!(xml.contains(r#"role="MIDDLE""#), "{xml}");
    let (stderr, _) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// beamerbaseframe.sty:91 sets `\ifbeamer@inframe` inside a frame (the BFH
  /// theme's `\sectionpage` otherwise nests a `\frame[plain]`: DEMO-BFHBeamer
  /// ×2), and beamerbasesection.sty:45-93's lecture layer captures the
  /// `\AtBeginLecture` body instead of running it (beamerthemeVerona.sty:354).
  #[test]
  fn beamer_inframe_flag_and_lecture_layer() {
    let tex = r"\documentclass{beamer}
\makeatletter
\def\sectionpage{\ifbeamer@inframe\else\frame{X}\fi}
\AtBeginLecture{\begin{frame}[plain]\thelecture.\quad \insertlecture\end{frame}}
\makeatother
\begin{document}
\section{S}
\frame{\sectionpage}
\begin{frame}{T}x\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<subsection").count(), 2, "{xml}");
    assert!(!xml.contains("plain"), "{xml}");
  }

  /// `\maketitle` inside a box capture (ltx-talk.cls:515 frames, unifront,
  /// `\parbox{…}{\maketitle}`) degrades its frontmatter to `ltx:text`
  /// elements instead of `<ltx:title> isn't allowed in <ltx:_CaptureBlock_>`.
  #[test]
  fn maketitle_inside_a_box_degrades_to_text() {
    let tex = r"\documentclass{article}
\title[Short]{My Title}
\author{Alice}
\begin{document}
\parbox{\textwidth}{\maketitle}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<title>"), "{xml}");
    assert!(
      xml.contains(r#"<text class="ltx_title">My Title</text>"#),
      "{xml}"
    );
    assert!(xml.contains("Alice"), "{xml}");
  }

  /// keyval.sty reads each option as a delimited argument, so a `{…}` inside
  /// a KEY is opaque (enumitem shortlabels expanding to a box: verifica.cls
  /// `\setlist[test]{\@risp,leftmargin=*}`, 3 mode errors × 5 docs).
  #[test]
  fn keyval_key_is_brace_aware() {
    let tex = r"\documentclass{article}
\usepackage[shortlabels,inline]{enumitem}
\makeatletter
\newcommand{\labelbox}[1]{\fbox{\parbox[][.2cm][c]{.2cm}{#1}}}
\def\@risp{\labelbox{\alph*}}
\newlist{test}{enumerate}{1}
\setlist[test]{\@risp,leftmargin=*}
\setlist[esercizi]{\bfseries 1.,leftmargin=*}
\makeatother
\begin{document}
x
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>x</p>"), "{xml}");
  }

  /// ngermanb.ldf:123-127 / french.ldf: the babel `\extras<lang>` hooks must
  /// exist, or cleveref's `\cref@addto` `\edef`s a self-referential hook that
  /// loops at `\begin{document}` (homework-demo-de/-fr, jwjournal-demo-de).
  #[test]
  fn babel_extras_hooks_are_defined() {
    for lang in ["ngerman", "french"] {
      let tex = format!(
        r#"\documentclass[{lang}]{{article}}
\usepackage[{lang}]{{babel}}
\usepackage{{cleveref}}
\begin{{document}}
\selectlanguage{{{lang}}}
Sch\"one Gr\"u\ss e
\end{{document}}
"#
      );
      let (stderr, xml) = convert(&tex, false);
      assert_eq!(error_count(&stderr), 0, "{lang}: {stderr}");
      assert!(!stderr.contains("expands into itself"), "{lang}: {stderr}");
      assert!(xml.contains("Schöne Grüße"), "{lang}: {xml}");
    }
  }

  /// xkeyval.tex:497/618 fetch `\XKV@rm` one step: a leftover value may name
  /// a macro defined only when its key code finally runs (chessboard.sty:1439
  /// `trimarea=\board`, `\board` \edef'd at :1087 — chessboard-skakps).
  #[test]
  fn setrmkeys_keeps_leftover_values_unexpanded() {
    let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key[p]{A}{k}{\def\got{#1}}
\setkeys*[p]{B}{k=\m}
\def\m{VAL}
\setrmkeys[p]{A}
\makeatother
\begin{document}
[\got]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[VAL]"), "{xml}");
  }

  /// amsopn.sty:90 `\operatorfont` (glosmathtools `\sbu`, ~54× per manual).
  #[test]
  fn amsopn_operatorfont_is_defined() {
    let tex = r"\documentclass{article}
\usepackage{amsmath}
\newcommand*{\sbu}[1]{_{\operatorfont{#1}}}
\begin{document}
$x\sbu{i}$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<XMApp>") && xml.contains("i</XMTok>"),
      "{xml}"
    );
  }

  /// fontspec-luatex.sty:3980 `\strong`; under the `luatex` profile
  /// nlctuserguide.sty:177 relies on fontspec for it (glossariesbegin,
  /// mfirstuc-manual: their only error).
  #[test]
  fn fontspec_strong_is_bold() {
    let tex = r"\documentclass{article}
\usepackage{fontspec}
\begin{document}
\strong{hi} there
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<text font="bold">hi</text>"#), "{xml}");
  }

  /// scalerel.sty:56 loads graphicx; :152-186 is the documented low-level
  /// API (`\ThisStyle`, `\SavedStyle`, `\@obj`, `\LMex`); `\@obj` re-enters
  /// math so a math-mode `\scaleobj` keeps its scripts (scalerel.tex:422-508,
  /// hwemoji, stackengine).
  #[test]
  fn scalerel_low_level_api_and_math_objects() {
    let tex = r"\documentclass{article}
\usepackage{scalerel}
\begin{document}
\scalebox{2}{X}
\(\scaleobj{2}{\sum_{i=0}^{n}}\)
\makeatletter
$\ThisStyle{\hbox{\@obj{\LMex=1ex \SavedStyle x}}}$
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<ERROR"), "{xml}");
    assert!(xml.contains(r#"xscale="2.0""#), "{xml}");
    assert!(xml.contains("∑") && xml.contains("SUBSCRIPTOP"), "{xml}");
  }

  /// cas-common.sty:1560 `{graphicalabstract}` (cas-sc / cas-dc).
  #[test]
  fn cas_graphicalabstract_is_a_note() {
    let tex = r"\documentclass{cas-sc}
\begin{document}
\begin{graphicalabstract}
Some abstract figure.
\end{graphicalabstract}
Body text.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<note role="graphicalabstract">"#), "{xml}");
  }

  /// spanish.ldf:680 `\deactivatetilden` (gaceta.cls:1612).
  #[test]
  fn babel_spanish_deactivatetilden_is_defined() {
    let tex = r"\documentclass{article}
\usepackage[spanish]{babel}
\makeatletter
\deactivatetilden
\makeatother
\begin{document}
Espa\~nol
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Español"), "{xml}");
  }

  /// xcolor.sty:168 `\XC@@names`, called by xcolor-patches-tmp-ltx.sty:83
  /// under pdfmanagement's `package/xcolor/after` hook (doc-use-newpax).
  #[test]
  fn xcolor_names_hook_is_defined() {
    let tex = r"\RequirePackage{pdfmanagement}
\documentclass{article}
\usepackage{xcolor}
\begin{document}
\textcolor{red}{hello}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r##"color="#FF0000""##), "{xml}");
  }

  /// tabu.sty:1066/1081 `X[1,$]` is a MATH column (brandeis-problemset
  /// example.tex:228).
  #[test]
  fn tabu_math_x_column() {
    let tex = r"\documentclass{article}
\usepackage{tabu}
\begin{document}
\begin{tabu} to 0.5\linewidth{X[1,$]rr}
P_1 & 10 & 3 \\
P_2 & 1 & 1 \\
\end{tabu}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("SUBSCRIPTOP"), "{xml}");
    assert!(xml.contains("<td") && xml.contains(">10</td>"), "{xml}");
  }

  /// biblatex.sty defines its `\if<test>` commands as BRANCH-SELECTING
  /// macros (`\iffieldundef{f}{true}{false}`, :6205), not TeX conditionals;
  /// plus the round-3 declarations (`\DeclareLabeltitle`, `\letbibmacro`,
  /// `\uspunctuation`, `\footfullcite`).
  #[test]
  fn biblatex_tests_are_branch_macros() {
    let tex = r"\documentclass{article}
\usepackage{biblatex}
\DeclareLabeltitle{\field{title}}
\DeclareLabelalphaTemplate{\labelelement{\field{label}}}
\letbibmacro{foo}{bar}
\uspunctuation
\begin{document}
[\iffieldundef{title}{U}{D}]
[\ifcitation{C}{N}]
[\ifentrytype{book}{B}{N}]
[\ifuseauthor{A}{N}]
[\ifhyperref{H}{N}]
\stdpunctuation
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[U]\n[N]\n[N]\n[A]\n[H]"), "{xml}");
  }

  /// biditools.sty:792 `\bidi@ifscanable` rebuilds a macro from its
  /// `\meaning`; a native (closure) `\begin`/`\end` must fail that `\ifx`
  /// round-trip as in Perl, or the patched `\begin` loses its `\begingroup`
  /// (crbox-doc, ghab-doc: "close a group that switched to mode horizontal").
  #[test]
  fn biditools_env_patch_leaves_begin_end_intact() {
    let tex = r"\documentclass{article}
\usepackage{biditools}
\begin{document}
\begin{tabular}{ll}a & b\\\end{tabular}
\begin{center}x\end{center}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(">a</td>") && xml.contains(">b</td>"), "{xml}");
    assert!(xml.contains(r#"<p align="center">x</p>"#), "{xml}");
  }

  /// enumitem.sty:108 `\enitkv@key` adds a list key (verifica.cls:307);
  /// italian.ldf:155/179 `\setISOcompliance`, `\IntelligentComma`.
  #[test]
  fn enitkv_key_and_babel_italian_extras() {
    let tex = r"\documentclass{article}
\usepackage[italian]{babel}
\usepackage{enumitem}
\makeatletter
\enitkv@key{}{mykey}{\gdef\gotkey{#1}}
\makeatother
\setISOcompliance
\begin{document}
\IntelligentComma
\begin{enumerate}[mykey=7]
\item a
\end{enumerate}
[\gotkey]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[7]"), "{xml}");
  }

  /// latex.ltx:16061/16072: itemize/enumerate locally reset `\makelabel`, so
  /// a document's global 2-argument `\makelabel` (mathfont-user-guide.tex:85)
  /// never receives the item labels (Perl errs the same way).
  #[test]
  fn global_makelabel_does_not_reach_list_items() {
    let tex = r"\documentclass{article}
\usepackage{enumitem}
\makeatletter
\def\makelabel#1#2{\expandafter\gdef\csname fig@#1\endcsname{#2}}
\makeatother
\begin{document}
\begin{itemize}
\item First bullet item.
\item Second item.
\end{itemize}
\begin{enumerate}[label=(\alph*)]
\item a
\end{enumerate}
\makelabel{x}{y}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<item ").count(), 3, "{xml}");
  }

  /// LuaTeX manual §7.3 `\Udelimiter`/`\Uradical`/`\Umathcodenum` +
  /// `\mathnolimitsmode`/`\scantextokens` (mathfont.sty:670,1405,2818-2925;
  /// mathfont-symbol-list).
  #[test]
  fn umath_delimiter_radical_and_codenum_under_luatex_profile() {
    let tex = r"\documentclass{article}
\mathnolimitsmode=4\relax
\begin{document}
$\Umathcharnumdef\myrel=\Umathcodenum`\- \relax$
$\Udelimiter+4+0+123\relax$
$\Uradical+0+8730\relax{x}$
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("{") && xml.contains("<XMApp"), "{xml}");
  }

  /// beamerbasethemes.sty:25 `\usefonttheme` loads its theme file (the
  /// uantwerpen font theme carries `\usetikzlibrary{calc}`), and
  /// beamerbasecompatibility.sty:309 `\beamer@ifempty` (graphbox's
  /// `\includegraphics`). Witness beamerthemeuantwerpenuserguide.
  #[test]
  fn beamer_font_theme_loads_and_ifempty_is_defined() {
    let tex = r"\documentclass{beamer}
\usepackage{tikz}
\usepackage{graphbox}
\usefonttheme{serif}
\makeatletter
\begin{document}
\begin{frame}
\beamer@ifempty{}{EMPTY}{FULL}
\includegraphics[width=1cm]{example-image}
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("EMPTY"), "{xml}");
    assert!(xml.contains("<graphics"), "{xml}");
  }

  /// italian.ldf:156-171: with ISO compliance on, `\unit` is the babel-italian
  /// unit macro (verifica example4/5 `$25\unit{m}$`).
  #[test]
  fn babel_italian_unit_under_iso_compliance() {
    // `\setISOcompliance` must precede babel's own `begindocument` chunk
    // (italian.ldf:155-165 tests `\it@ISOcompliance` there). A document-level
    // `\AtBeginDocument{\setISOcompliance}` runs LAST in lthooks (`top-level`
    // after package labels) — pdflatex then also reports `\unit` undefined
    // (probed TL2025) — so the compliance switch is set in the preamble.
    let tex = r"\documentclass{article}
\usepackage[italian]{babel}
\setISOcompliance
\begin{document}
$25\unit{m}$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"font="upright""#) || xml.contains("mathrm"),
      "{xml}"
    );
  }

  /// latex_constructs.pool.ltxml:2588-2605: the bare text command is the
  /// call-time encoding dispatcher, so textalpha's `normalize-symbols`
  /// override of `\LGR\textbetasymbol` reaches `\textbetasymbol`
  /// (greek-fontenc char-list, hyperref-with-greek); `\UseTextSymbol` runs
  /// the encoding-specific body inside its encoding (`\textsigma` under T1
  /// is σ, not a Latin `s`; KPE #148 slot 0x73).
  #[test]
  fn provide_text_command_dispatches_on_encoding() {
    let tex = r"\documentclass{article}
\usepackage[LGR,T1]{fontenc}
\usepackage[normalize-symbols]{textalpha}
\begin{document}
X\textbetasymbol Y\textthetasymbol Z \textsigma\textalpha
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>XβYϑZ σα</p>"), "{xml}");
  }

  /// ctex-heading-article.def:747 makes `\p@section` argument-taking; the
  /// refnum formatter must close its `\csname` first (KPE #149; caspervector,
  /// sduthesis, tabular2, inkpaper-en).
  #[test]
  fn ctex_argument_taking_p_macro_keeps_the_refnum() {
    let tex = r"\documentclass{article}
\makeatletter
\def\p@section#1{\thesection}
\makeatother
\begin{document}
\section{X}
Body.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<tag role="refnum">1</tag>"#), "{xml}");
  }

  /// latex.ltx:107-110 + :896-1058: the lualatex format surface (attributes,
  /// catcode tables, lua-function allocators, hyphenation chars) under the
  /// `luatex` profile — luaotfload's `\input ltluatex`, luacolor's
  /// `\setattribute`, tuenc.def's `\newprotectedluacmd`, babel's
  /// `\prehyphenchar` (17 lualatex-oracle manuals).
  #[test]
  fn ltluatex_format_surface_under_luatex_profile() {
    let tex = r"\documentclass{article}
\usepackage{luaotfload}
\usepackage[TU]{fontenc}
\usepackage{luainputenc}
\makeatletter
\begin{document}
\prehyphenchar=`\- \newattribute\myattr \setattribute\myattr{7}[\the\myattr]
\newprotectedluacmd\mycmd \newcatcodetable\mytable \catcodetable\mytable
[\the\e@alloc@attribute@count]
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[7]"), "{xml}");
  }

  /// OXIDIZED_DESIGN #184: `\DeclareTextAccent` (lgrenc.def:439-470) defines
  /// the Greek diacritics as combining-mark accents with an encoding
  /// dispatcher (Perl ignores it: teubner.sty:165 `\let\~\accperispomeni`
  /// then made `\~` undefined; textalpha's `\<`/`\>` breathings errored);
  /// the dispatcher is `\fi`-free so an argument-taking text command sees
  /// its argument. Also the section-type name of `\@@numbered@section` is
  /// taken from the reverted tokens, not the LGR-decoded text
  /// (`\theσεςτιον`).
  #[test]
  fn declare_text_accent_defines_greek_diacritics() {
    let tex = r"\documentclass{article}
\usepackage[LGR,T1]{fontenc}
\usepackage{textalpha}
\begin{document}
\fontencoding{LGR}\selectfont
\section{A}
[\<a][\accperispomeni{a}][\>'\textalpha][\accdialytika{i}][\accpsili{}]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[ἁ][ᾶ][ἄ][ϊ][\u{0313}]"), "{xml}");
    assert!(xml.contains(r#"<tag role="refnum">1</tag>"#), "{xml}");
  }

  /// multicol.sty.ltxml:22 closed an `ltx:p` a block spanning text had
  /// already closed (KPE #150; thuaslogos-doc).
  #[test]
  fn multicols_spanning_section_is_not_double_closed() {
    let tex = r"\documentclass{article}
\usepackage{multicol}
\begin{document}
Intro.
\begin{multicols}{2}[\section*{Contents}]
Column text.
\end{multicols}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<title>Contents</title>"), "{xml}");
  }

  /// latex.ltx:9525 `begindocument/end` and :15257 `enddocument` fire; the
  /// former is UNREAD so a `+b` environment opened from it (jwjournal.cls:643
  /// wraps the whole body) reads the body from the file.
  #[test]
  fn begindocument_end_and_enddocument_hooks_fire() {
    let tex = r"\documentclass{article}
\ExplSyntaxOn
\NewDocumentEnvironment{wrapall}{+b}{[\regex_replace_all:nnN{\#\#}{\c{section}\*}\l_tmpa_tl\tl_set:Nn\l_tmpa_tl{#1}\regex_replace_all:nnN{\#\#}{\c{section}\*}\l_tmpa_tl\tl_use:N\l_tmpa_tl]}{}
\hook_gput_code:nnn{begindocument/end}{t}{\begin{wrapall}}
\hook_gput_code:nnn{enddocument}{t}{END-HOOK}
\ExplSyntaxOff
\begin{document}
Body text.

## {A New Section}

More.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<title>A New Section</title>"), "{xml}");
    assert!(xml.contains("END-HOOK"), "{xml}");
  }

  /// PLANS P37 (svg half): block content in a TikZ node (`\verb`) gets an
  /// auto-opened `svg:foreignObject` (Flow model) — makeshape, optikz.
  #[test]
  fn verbatim_in_a_tikz_node_gets_a_foreign_object() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\node at (0,0) [draw] (a) {\verb|x  x|};
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<svg:foreignObject") && xml.contains("x  x</verbatim>"),
      "{xml}"
    );
  }

  /// The trivial-recursion guard anchors on the INVOKING token: a `\let`
  /// alias of a macro whose body starts with the original CS is not a loop
  /// by itself (musixlyr.tex:709-722 `\der@kontext`; recorder-fingering,
  /// undar-digitacion-doc), while `\def\x{\x}` invoked as `\x` still is.
  #[test]
  fn recursion_guard_anchors_on_the_invoking_token() {
    let tex = r"\documentclass{article}
\begin{document}
\def\selfx{\selfx}
\def\ctx{\ctx A}
\let\alias\ctx
\def\ctx{}
\edef\zz{\alias}[\zz]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[A]"), "{xml}");
    let tex2 = r"\documentclass{article}
\begin{document}
\def\selfx{\selfx}\edef\zz{\selfx}
\end{document}
";
    let (stderr2, _) = convert(tex2, false);
    assert!(stderr2.contains("expands into itself"), "{stderr2}");
  }

  /// codehigh.sty:508 takes its `\directlua` parser under the luatex profile;
  /// the binding degrades that path to plain verbatim text in bounded time
  /// (the l3regex parser is O(n²), PLANS P65 — CreationBoites-doc,
  /// tkz-bernoulli, tabularray-abnt, functional all timed out on it).
  #[test]
  fn codehigh_highlights_without_lua() {
    let tex = r"\documentclass{article}
\usepackage{codehigh}
\CodeHigh{language=latex/latex2}
\begin{document}
\begin{codehigh}
\foo{bar}
\end{codehigh}
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("bar") && xml.contains("foo"), "{xml}");
  }

  /// beamer internals the themes reach: beamerbasesection's `\secname`
  /// family, `\beamer@slideinframe`, the gotham font theme's `\patchcmd`
  /// targets, and `\titlegraphic` STORING its argument (Verona's `\node`).
  #[test]
  fn beamer_section_names_slide_counter_and_patch_targets() {
    let tex = r"\documentclass{beamer}
\usetheme{gotham}
\title{T}
\titlegraphic{\node[anchor=north]at(0,0){G};}
\makeatletter
\begin{document}
\section{Intro}
\begin{frame}\frametitle{\secname}[\number\beamer@slideinframe]\framebreak Body\end{frame}
\begin{frame}\titlepage\end{frame}
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[1]") && xml.contains("Body"), "{xml}");
  }

  /// Wave-11 package internals: `\captionbox` (caption.sty:454), pict2e's
  /// `\polyline` family under curve2e, graphics' `\Grot@setangle`/`\Grot@box`
  /// (isorot), xcolor's `\xcolor@`, hyperref's `\IfHyperBoolean`, biblatex's
  /// `\AtUsedriver`/`\delimcontext`/`\DeclareAutoCiteCommand`, cas's xspace.
  #[test]
  fn wave11_package_internals_are_defined() {
    let tex = r"\documentclass{article}
\usepackage{caption}
\usepackage{curve2e}
\usepackage{isorot}
\usepackage{hyperref}
\usepackage{xspace}
\usepackage{xcolor}
\makeatletter
\begin{document}
\begin{figure}\captionbox{A caption\label{f}}[\linewidth]{Content}\end{figure}
\begin{picture}(10,10)\polyline(0,0)(10,10)(20,0)\polygon(0,0)(5,5)(10,0)\end{picture}
\begin{sideways}Hi\end{sideways}
[\IfHyperBoolean{hyperfootnotes}{yes}{no}][\xcolor@{}{X}{}{}]
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<caption>") && xml.contains("A caption"),
      "{xml}"
    );
    assert!(
      xml.contains("<line points=\"0,0 13.84,13.84 27.67,0\""),
      "{xml}"
    );
    assert!(xml.contains("angle=\"90"), "{xml}");
    assert!(xml.contains("[no][X]"), "{xml}");
  }

  /// tuenc.def:106-121 `\DeclareUnicodeAccent` under the luatex profile
  /// (tipauni.sty:349) and the LuaTeX PDF primitives beside `\directlua`
  /// (`\pdfvariable pageattr`, multimedia.sty:30).
  #[test]
  fn unicode_accent_and_pdfvariable_under_luatex_profile() {
    let tex = r#"\documentclass{article}
\usepackage{multimedia}
\begin{document}
\DeclareUnicodeAccent{\textsyllabic}{TU}{"0329}
[\textsyllabic{n}]
\end{document}
"#;
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[n\u{0329}]"), "{xml}");
  }

  /// ltxtable.sty:9 `\LTXtable{width}{file}` — a longtable with `X` columns
  /// from a file; the raw macro reaches `\TX@target`/`\LT@echunk` internals
  /// the bindings do not model (tikzcodeblocks-documentation, vhistory).
  #[test]
  fn ltxtable_inputs_a_longtable_with_x_columns() {
    let tex = r"\documentclass{article}
\usepackage{ltxtable}
\begin{document}
\LTXtable{\textwidth}{mytab.tex}
\end{document}
";
    let table = r"\begin{longtable}{lX}
a & some longer text that would wrap \\
b & more \\
\end{longtable}
";
    let (stderr, xml) = convert_with_files(tex, &[("mytab.tex", table)]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<tabular") && xml.contains("<p>more</p>"),
      "{xml}"
    );
  }

  /// tex.web §1091 `new_graf` fires `\everypar` when a *list* starts a
  /// paragraph; a constructor's digested `{}` argument is macro-parameter
  /// text, not a list. An armed `\everypar` (latex.ltx:8090 `\@afterheading`'s
  /// `{\setbox\z@\lastbox}`, left by ltugboat.cls:1214 `\aftergroup\@afterheading`
  /// in `\@maketitle`) used to fire inside `\@@numbered@section`'s *type*
  /// argument and revert as `{}section` — counter `\c@{}section`, tag
  /// `ltx:{}section` (lazylist, parnotes). The body paragraph after the
  /// heading still fires it.
  #[test]
  fn everypar_does_not_fire_inside_a_constructor_argument() {
    let tex = r"\documentclass{article}
\begin{document}
A\everypar{{\setbox0\lastbox}}
\section{Why lists?}
Text.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("{}section"), "{stderr}");
    assert!(
      xml.contains(r#"<section inlist="toc" xml:id="S1">"#) && xml.contains("<tag>1</tag>"),
      "{xml}"
    );
    // A paragraph of the current list is `new_graf`: it still fires (the
    // algorithm2e `\nl` numbering rides on this).
    let tex = r"\documentclass{article}
\begin{document}
\everypar{EP:}Text.

More.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<p>EP:Text.</p>") && xml.contains("<p>EP:More.</p>"),
      "{xml}"
    );
  }

  /// lineno.sty:1077 `\newif\ifLineNumbers` — the binding lacked lineno's
  /// switches, and `\lx@deposit@maketitle` (OD #124) runs a class's
  /// `\@maketitle`, which for homework.cls:128 reaches minimalist.sty:144
  /// `\LocallyStopLineNumbers` = `…\ifLineNumbers\LNturnsONtrue\fi…`
  /// (homework-demo-{cn,de,en,es,fr,jp}).
  #[test]
  fn lineno_binding_defines_the_line_number_switches() {
    let tex = r"\documentclass{article}
\usepackage{lineno}
\makeatletter
\renewcommand{\@maketitle}{\ifLineNumbers\fi\ifoddNumberedPage\fi\ifcolumnwiselinenumbers\fi Body}
\makeatother
\title{X}\author{Y}
\begin{document}
\maketitle
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<title>X</title>") && xml.contains("<personname>Y</personname>"),
      "{xml}"
    );
  }

  /// examdesign.cls:323-344 owns `\section` as a *non-sectioning* macro and
  /// `\begin{section}…\end{section}` wraps every question block
  /// (examdesign.cls:802-812); the locked kernel `\section` ran
  /// `\@startsection` on the environment body instead (examplea/b/c: Perl 67
  /// errors, Rust Fatal after 100). The class binding unlocks it before the
  /// raw load.
  #[test]
  fn examdesign_owns_section_as_an_environment() {
    let tex = r"\documentclass{examdesign}
\begin{document}
\begin{matching}[title={T}]
  \pair{Elvis}{Spike}
  \pair{Nirvana}{Nevermind}
\end{matching}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<section"), "{xml}");
    assert!(xml.contains("Elvis") && xml.contains("Nevermind"), "{xml}");
    assert!(xml.matches("<item").count() >= 4, "{xml}");
  }

  /// latex.ltx:9551 `\protected@write` freezes `\protect`ed macros into the
  /// index entry (`\let\protect\@unexpandable@protect`); expanding them at
  /// `\index` time ran manyind.sty:100/119's `\protect\def\nwletre{…}` and
  /// `\protect\nxtletre` (`\proc@letter`'s caller-closing `\fi`) in the
  /// gullet (mindsample: `undefined \nwletre`, stray `\fi`).
  #[test]
  fn index_entry_defers_protected_macros() {
    let tex = r"\documentclass{article}
\usepackage{makeidx}\makeindex
\long\def\ltest#1{\ifx#1\ltest\else X\fi}
\newcommand\nxt{\def\item{\ltest}}
\begin{document}
A\index{key@\protect\nxt}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<indexphrase key="key"/>"#), "{xml}");
  }

  /// pdfTeX `annot type spec` = `[useobjnum n] [rule spec] general text`;
  /// the `(width|height|depth) dimen` rule spec was never read (Perl
  /// pdfTeX.pool:156-171 too; KPE #151). pdfmarginpar.sty:142 passes it
  /// whenever a `width=`/`height=` key is set (pdfmarginpar doc).
  #[test]
  fn pdfannot_reads_its_rule_spec() {
    let tex = r"\documentclass{article}
\begin{document}
Hi\pdfannot width 4cm height 0.5cm {/Subtype /Text /Contents (x)} there
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>Hi there</p>"), "{xml}");
  }

  /// robustindex.sty:201-216 `\gobblepageref`/`\wrappageref` scan for the
  /// `, \indpageref{N}` makeindex writes into an `.ind` line; LaTeXML's index
  /// has no such line (robustsample.tex:82; multisample, robustmanual).
  #[test]
  fn robustindex_page_reference_hooks_are_inert() {
    let tex = r"\documentclass{article}
\usepackage{makeidx}
\usepackage{robustindex}
\makeindex
\begin{document}
A\index{alpha!see also gamma\gobblepageref}
B\index{beta\wrappageref\textbf}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(">alpha</indexphrase>"), "{xml}");
    assert!(xml.contains(">see also gamma</indexphrase>"), "{xml}");
    assert!(xml.contains(">beta</indexphrase>"), "{xml}");
  }

  /// `\abstract{…}` is the environment's begin code plus a plain group in
  /// LaTeX, read incrementally — a `\makeatletter` inside it precedes the
  /// `\patch@level` that follows (char-list-alphabeta.tex:88-103; PLANS P74,
  /// SHARED). It was taken as one pre-tokenized `{}` argument.
  #[test]
  fn braced_abstract_reads_its_body_incrementally() {
    let tex = r"\documentclass{article}
\makeatletter\def\patch@level{7}\makeatother
\title{T}
\begin{document}
\maketitle
\abstract{ \noindent Test.
\makeatletter
patch-level \patch@level{} here.
\makeatother
}
Body text.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<abstract"), "{xml}");
    assert!(xml.contains("patch-level 7 here."), "{xml}");
    assert!(xml.contains("<p>Body text.</p>"), "{xml}");
  }

  /// latex.ltx:10140-10156 keep a counter's reset list as the macro
  /// `\cl@<ctr>` = `\@elt{child}…`; raw code expands and rewrites it
  /// (contract.sty:336 `\edef\cl@Clause{\cl@Clause\cl@contractClause}`,
  /// afthesis.cls:44-49 `\@removefromreset` re-`\edef`). LaTeXML's State value
  /// stays authoritative; the macro mirrors it after every mutation.
  #[test]
  fn reset_list_is_an_expandable_cl_macro() {
    let tex = r"\documentclass{article}
\makeatletter
\newcounter{Clause}\newcounter{contractClause}[Clause]
\newcounter{Extra}\@addtoreset{Extra}{Clause}
\edef\cl@Clause{\cl@Clause\cl@contractClause}
\def\@elt#1{[#1]}
\begin{document}
A\cl@Clause B\cl@contractClause C
\stepcounter{contractClause}\stepcounter{Clause}\thecontractClause
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("[contractClause]") && xml.contains("[Extra]"),
      "{xml}"
    );
    assert!(xml.contains("BC"), "{xml}");
    // the Value list still drives the reset: Clause stepped -> contractClause back to 0
    assert!(xml.contains("0</p>") && !xml.contains("1</p>"), "{xml}");
  }

  /// `\mbox\bgroup A … B\egroup` (syntax.sty:158 `\syn@assist`; the newcommand
  /// manual's `grammar` environment): TeX hands `\bgroup` to `\mbox#1` as its
  /// one-token argument and the box then runs to the `\egroup`. A `{}` argument
  /// that is exactly an implicit begin-group reads its group by digestion.
  #[test]
  fn implicit_bgroup_argument_reads_its_group() {
    let tex = r"\documentclass{article}
\def\OPEN{\mbox\bgroup A}
\def\CLOSE{ B\egroup}
\begin{document}
X\OPEN\CLOSE Y
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("XA BY"), "{xml}");
    let tex = r#"\documentclass{article}
\usepackage{syntax}
\begin{document}
\begin{grammar}
<decl> ::= \[[ "MACRO" <ident> \]]
\end{grammar}
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("MACRO"), "{xml}");
  }

  /// amsldoc.cls:84-89 `\indexcs` writes the sort key of `\cn{\\*}` as the
  /// `\string`ed `\*` — catcode-12 after `\@sanitize` (latex.ltx:1778); the
  /// whole-string re-tokenization welded it into the live `\*` (amsldoc.cls:213)
  /// which ate the entry (itamsldoc, amsldoc-vi; PLANS P73, SHARED).
  #[test]
  fn index_sanitized_backslash_symbol_stays_literal() {
    let tex = r#"\documentclass{amsldoc}
\usepackage{guit}
\usepackage{makeidx}\makeindex
\begin{document}
Il comando \cn{\\*} qui.
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<indexmark").count(), 1, "{xml}");
    assert!(!xml.contains("<ERROR"), "{xml}");
  }

  /// expl3-code.tex:7846-7861 fixes `\c_sys_engine_str` and the
  /// `\sys_if_engine_<e>` conditionals at format-build time; the `luatex`
  /// profile must re-derive them (polyglossia gloss-latin.ldf:125 else takes
  /// the XeTeX branch — hang, sample), and unicode-math's `\math<style>`
  /// aliases (unicode-math-luatex.sty:2273-2306; toptesi topcoman.sty:76
  /// `\mathup`) must exist.
  #[test]
  fn l3sys_engine_identity_under_luatex_profile() {
    let tex = r"\documentclass{article}
\usepackage{unicode-math}
\begin{document}
\ExplSyntaxOn
[\c_sys_engine_str][\sys_if_engine_luatex:TF{L}{X}][\sys_if_engine_pdftex:TF{P}{N}][\c_sys_engine_format_str]
\ExplSyntaxOff
$\mathup{\mu}+\mathbfit{x}+\symscr{S}$
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[luatex][L][N][lualatex]"), "{xml}");
    assert!(xml.contains(r#"tex="\mathrm{\mu}+"#), "{xml}");
  }

  /// tex.web §368: `\expandafter` expands the second token FIRST and only
  /// then `back_input`s the saved one, so a saved `{` still counts in
  /// `align_state` while the expansion reads its arguments. Rust retracted
  /// the brace before the expansion: in `\exp_after:wN { \use_none:nn & …}`
  /// (numerica.sty:1748 `\__nmc_delim_arg:` on the slash path of
  /// `\eval{1/8}`) the `&` was read at ledger 0 and fired the cell template
  /// mid-cell — the amsmath after-`$` inserted early, the before-`$` frame
  /// left open (numerica 83, mhchem `\ce` 14, tablists-rus 101; Perl shares
  /// it). The package-free shape is the second document. The plain `{$b$}`
  /// in a cell stays a TeX error (§1065 `off_save`). NB tex.web `macro_call`
  /// keeps `align_state` LIVE during parameter scanning — a freeze there
  /// broke `columncolor_lbrack_cell_does_not_cascade_the_column_mode`.
  #[test]
  fn argument_scan_is_align_state_neutral() {
    let tex = r"\documentclass{article}\usepackage{amsmath}
\ExplSyntaxOn
\newcommand\doit{\exp_after:wN { \use_none:nn & Z } }
\ExplSyntaxOff
\begin{document}
\begin{align*}
a &= 1 \doit + 2
\end{align*}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<equationgroup"), "{xml}");
    let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{numerica}
\begin{document}
\begin{align*}
a & =\eval{1/8} \\
b & =\eval{1/8} & c &= 2
\end{align*}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<equationgroup") && xml.contains(">0.125</XMTok>"),
      "{xml}"
    );
    let tex = r"\documentclass{article}\usepackage{amsmath}
\begin{document}
\begin{align*}
a &= {$b$} + c
\end{align*}
\end{document}
";
    let (stderr, _xml) = convert(tex, true);
    assert!(
      error_count(&stderr) > 0,
      "a $ under a simple group must stay an error:\n{stderr}"
    );
  }

  /// latex.ltx:1729-1737 `\@ifundefined` probes with `\ifcsname` and leaves the
  /// name undefined; the `\relax` pollution broke every reentrancy-guarded
  /// `.def` loaded as `\@ifundefined{sentinel}{\input file}{}` — polyglossia's
  /// gloss-latin.ldf:591 + babelsh.def:1 (hang, sample; Perl pollutes too).
  #[test]
  fn ifundefined_does_not_define_the_name() {
    let tex = r"\documentclass{article}
\makeatletter
\@ifundefined{zz@undef}{}{}
\ifx\zz@undef\@undefined [STILL-UNDEFINED]\else [POLLUTED]\fi
\@ifundefined{zz@undef}{[U]}{[D]}
\def\zz@def{}\@ifundefined{zz@def}{[U]}{[D]}
\makeatother
\begin{document}
\makeatletter\ifx\zz@undef\@undefined [BODY-UNDEFINED]\fi\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[BODY-UNDEFINED]"), "{xml}");
    let tex = r"\documentclass{article}
\usepackage{polyglossia}
\setdefaultlanguage{english}
\setotherlanguage{latin}
\begin{document}
Text \textlatin{lingua latina} here.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("lingua latina"), "{xml}");
  }

  /// latex.ltx:16369 `\DeclareRobustCommand\underline`: robust, so an
  /// `\edef`/`\write` freezes the whole `\ifmmode…\fi` body (bibarts.sty:2231
  /// `\edef\@tempa{\write\@auxout{…\underline{Publ.}…}}`; Perl's non-robust
  /// body tears at `\else`).
  #[test]
  fn underline_is_robust_in_an_edef_write() {
    let tex = r"\documentclass{article}
\makeatletter
\begin{document}
\let\protect\@unexpandable@protect
\edef\x{\underline{Publ.}\overline{X}}
\let\protect\relax
\x
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<text framed="underline">Publ.</text>"#) || xml.contains(">Publ.</text>"),
      "{xml}"
    );
  }

  /// A raw class's full `\@maketitle` (ascelike.cls:406-411 `\AB@authlist`)
  /// runs under `\lx@deposit@maketitle` (OD #124); the bindings' semantic
  /// `\author` never fills authblk's visual accumulators, which therefore exist
  /// at their package-initial empty value so the layout collapses to nothing.
  #[test]
  fn class_maketitle_layout_over_binding_accumulators() {
    let tex = r"\documentclass{article}
\usepackage{authblk}
\author{Alice}
\title{T}
\makeatletter
\renewcommand{\@maketitle}{\begin{center}\@title\\ \AB@authlist\thankses\end{center}}
\makeatother
\begin{document}
\maketitle
Body.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<personname>Alice</personname>"), "{xml}");
    assert!(
      !xml.contains("<ERROR") && !xml.contains("AB@authlist"),
      "{xml}"
    );
  }

  /// tex.web §1083 `begin_box` pushes nest and save level TOGETHER for
  /// `\hbox\bgroup`; Perl `readBoxContents` (TeX_Box.pool:164-185) uses one
  /// frame. The two-frame hbox reader left ulem's open-here/close-there word
  /// boxes (examdesign.cls:186-200 `\UL@start`/`\UL@stop`) around a
  /// `\makebox` meeting the wrong frame (examdesign examplea/b/c; Perl shares).
  #[test]
  fn hbox_reader_is_one_frame() {
    let tex = r"\documentclass[10pt]{examdesign}
\Fullpages
\ContinuousNumbering
\DefineAnswerWrapper{}{}
\NumberOfVersions{2}
\class{{\Large A sample exam}}
\begin{document}
\begin{truefalse}[title={T/F}]
\begin{question}
  \answer{True} This sentence is not false.
\end{question}
\end{truefalse}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("This sentence is not false."), "{xml}");
    // `\hbox{a}` still reverts with its braces
    let tex = r"\documentclass{article}
\begin{document}
$\hbox{ab}$ \setbox0\hbox\bgroup x\egroup\box0
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("ab</text> x"), "{xml}");
  }

  /// latex.ltx:18557 `\ProcessOptions` reads `\@ptionlist{\@currname.\@currext}`
  /// = the MACRO `\opt@<pkg>.<ext>`, which babel.sty:316-347 rewrites to strip
  /// its `language.modifier` syntax (`greek.polutoniko` → `greek`,
  /// `\bbl@mod@greek`=polutoniko). Reading the loader's State list instead
  /// raised "Unknown option 'greek.polutoniko'" (alphabeta-doc,
  /// hyperref-with-greek; Perl shares it).
  #[test]
  fn processoptions_reads_the_rewritten_opt_macro() {
    let tex = r"\documentclass{article}
\usepackage[greek.polutoniko,english]{babel}
\begin{document}
\makeatletter[\bbl@mod@greek]\makeatother \textgreek{a}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[polutoniko]"), "{xml}");
    let tex = r"\documentclass{article}
\makeatletter
\def\lx@rewriter@sty{}
\DeclareOption{alpha}{\gdef\seen{ALPHA}}\DeclareOption{beta}{\gdef\seen{BETA}}
\def\@currname{article}\def\@currext{cls}
\expandafter\def\csname opt@article.cls\endcsname{beta}
\ProcessOptions\relax
\makeatother
\begin{document}
[\seen]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[BETA]"), "{xml}");
  }

  /// latex.ltx `\mbox{#1}` = `\leavevmode\hbox{#1}`: the content is an hbox
  /// BODY read in the same list, so ulem's `\hss` (`\UL@hskip` →
  /// `\afterassignment\UL@reskip` → `\UL@stop` `\egroup\egroup` … `\UL@start`)
  /// inside `\makebox[.5in][r]{\hss}` (examdesign.cls:1210) closes the makebox
  /// and the makebox's own `}` closes the box ulem reopened (OD #188). The
  /// common shapes keep their structure.
  #[test]
  fn box_constructor_content_is_a_live_hbox_body() {
    let tex = r"\documentclass{article}
\begin{document}
\makebox[2cm][r]{mk} \mbox{x y} \fbox{fb} \raisebox{1pt}{rb} \framebox[3cm]{fr} $\fbox{$op$}$
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<text align="right" width="56.9pt">mk</text>"#),
      "{xml}"
    );
    assert!(xml.contains("x y"), "{xml}");
    assert!(xml.contains(r#"framed="rectangle">fb</text>"#), "{xml}");
    assert!(xml.contains(r#"<text yoffset="1.0pt">rb</text>"#), "{xml}");
    assert!(
      xml.contains("<XMArg enclose=\"box\">") || xml.contains(r#"tex="\framebox{$op$}""#),
      "{xml}"
    );
  }

  /// latex.ltx:14978 `\labelformat#1` = `\expandafter\def\csname p@#1\endcsname##1`
  /// (kernel since 2019-10-01; varioref only re-exports it). contract.sty:978
  /// probes it with `\scr@ifundefinedorrelax{labelformat}` and, when it is
  /// missing, falls back to the pre-2019 `\p@sentence`=`\expandafter\p@@sentence`
  /// prefix, whose one-token grab of `\thesentence`'s expansion (`\arabic`)
  /// leaves `{sentence}` behind and ends `\refstepcounter`'s `\@currentlabel`
  /// with `\arabic}` ("You can't use } after \the" ×3 per sentence,
  /// contract-example-en 44 errors; Perl shares it, KPE #160). With the kernel
  /// macro the `\labelformat` branch wins and `\p@sentence` takes
  /// `\thesentence` whole, as it does under pdflatex.
  #[test]
  fn labelformat_is_a_kernel_macro() {
    let tex = r"\documentclass{article}
\makeatletter
\newcounter{par}\newcounter{sentence}[par]
\renewcommand*{\thesentence}{\arabic{sentence}}
\def\p@par{[P]}
\@ifundefined{labelformat}{%
  \renewcommand*{\p@sentence}{\expandafter\p@@sentence}%
  \newcommand*{\p@@sentence}[1]{\p@par{{\thepar}-}{S:#1}}%
}{\labelformat{sentence}{\p@par{{\thepar}-}{S:#1}}}
\makeatother
\labelformat{equation}{[E:#1]}
\newtheorem{thm}{Theorem}\labelformat{thm}{[T:#1]}
\begin{document}
\refstepcounter{par}\refstepcounter{sentence}\label{s}
X Y \ref{s}
\begin{equation}\label{e}x\end{equation}
\begin{thm}\label{t}x\end{thm}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<tag role="refnum">[E:1]</tag>"#), "{xml}");
    // typerefnum goes through the same `\p@<ctr>\the<ctr>` helper.
    assert!(
      xml.contains(r#"<tag role="typerefnum">Theorem [T:1]</tag>"#),
      "{xml}"
    );
  }

  /// LuaTeX's `\matheqdirmode` integer parameter (LuaTeX manual §6) beside
  /// its profile siblings (`\matheqnogapstep`, `\breakafterdirmode`);
  /// minim-math.tex:19 sets it (lettrine-demo-arabic, 1 error).
  #[test]
  fn luatex_profile_defines_matheqdirmode() {
    let tex = r"\documentclass{article}
\matheqdirmode=1
\begin{document}
[\the\matheqdirmode]
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[1]"), "{xml}");
  }

  /// pdfTeX `\pdfoutline [attr spec] action spec [count N] general text` and
  /// `\pdfdest <id> <dest type>` (manual §8.13-8.14) produce PDF navigation
  /// only, but the specs must be CONSUMED: tools-overview.tex:93 `\pdfoutline
  /// attr {…} user {…} {[#1]}` leaked `attr`/`user` into the text (Perl
  /// pdfTeX.pool:179-180 only comments them, KPE #162).
  #[test]
  fn pdfoutline_and_pdfdest_consume_their_specs() {
    let tex = r"\documentclass{article}
\begin{document}
\pdfoutline attr {/C[0 0 1]} user {<< /S/GoToR /F(x.pdf) >>} {[Section 1]}\relax
\pdfoutline goto name {sec1} count -2 {Sec}\pdfoutline goto file {o.pdf} page 3 {top} newwindow {Other}%
\pdfdest name {sec1} xyz zoom 1000 \pdfdest num 7 fitr width 2cm height 1cm \pdfdest name {a} fith
Body text.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>Body text.</p>"), "{xml}");
    assert!(
      !xml.contains("attr") && !xml.contains("user") && !xml.contains("zoom"),
      "{xml}"
    );
  }

  /// utf8.def:253-265 `\parse@UTFviii@a`/`@b` are KERNEL macros (latex.ltx:
  /// 22224 inputs utf8.def at format time); paresse-utf8.sty:203-204 `\let`s
  /// them to build its own UTF-8 sequences (paresse-eng 3, -fra 6 errors;
  /// Perl utf8.def.ltxml omits them too, KPE #163).
  #[test]
  fn utf8_octet_parsers_are_defined() {
    let tex = r"\documentclass{article}
\makeatletter
\count@=233 \parse@UTFviii@a;\parse@UTFviii@b C\UTFviii@two@octets.;
\edef\x{\expandafter\meaning\csname UTFviii@tmp\endcsname}
\makeatother
\begin{document}
[\x]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // 233 = 0xE9 → octets C3 A9, uppercased to the bytes' glyphs.
    assert!(xml.contains("UTFviii@two@octets Ã©"), "{xml}");
  }

  /// ltablex.sty makes `tabularx` a multi-page (longtable-driven) table and
  /// defines `\keepXColumns`/`\convertXColumns` (:146-153) as toggles of
  /// `\ifTX@convertX@`; the former stub defined neither (milsymb.tex, 44
  /// errors; Perl raw-loads the file). `\endhead` is legal inside.
  #[test]
  fn ltablex_tabularx_is_a_longtable_with_toggles() {
    let tex = r"\documentclass{article}
\usepackage{ltablex}
\keepXColumns
\begin{document}
\begin{tabularx}{\textwidth}{|c|l|X|}
 h1 & h2 & h3 \\ \hline \endhead
 a & b & c \\ \hline
 d & e & f \\ \hline
\end{tabularx}
\makeatletter\ifTX@convertX@ [CONVERT]\else [KEEP]\fi\convertXColumns\ifTX@convertX@ [CONVERT]\fi\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<thead"), "{xml}");
    assert!(xml.matches("<td").count() >= 9, "{xml}");
    assert!(xml.contains("[KEEP]") && xml.contains("[CONVERT]"), "{xml}");
  }

  /// pgfmath functions that real pgf defines with an integer literal result
  /// (`sign`, `iseven`/`isodd`/`isprime`, `gcd`, `div`, `scalar`, `true`/
  /// `false`, `!`) must print without `.0`, because packages feed them to
  /// `\ifnum`: tikzbricks.sty:146-151 `\ifnum\brick@sin<0` on `sign(sin(…))`
  /// broke at the `.` ("Expected a relational token"; tikzbricks doc 90
  /// errors, Perl identical). Probed against pdflatex/pgf TL2025.
  #[test]
  fn pgfmath_integer_functions_yield_integers() {
    let tex = r"\documentclass{article}
\usepackage{pgfmath}
\begin{document}
\def\P#1{\pgfmathparse{#1}[\pgfmathresult]}
\P{sign(-2.5)}\P{sign(0)}\P{iseven(4)}\P{isodd(4)}\P{isprime(7)}\P{gcd(12,18)}\P{div(7,2)}\P{scalar(3)}\P{true}\P{false}\P{!0}%
\P{floor(3.7)}\P{abs(-3)}\P{2+sign(1)}
\pgfmathparse{sign(-3)}\let\s\pgfmathresult \ifnum\s<0 [NEG]\fi
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("[-1][0][1][0][1][6][3][3][1][0][1][3.0][3.0][3.0]"),
      "{xml}"
    );
    assert!(xml.contains("[NEG]"), "{xml}");
  }

  /// xcolor's `\color@<name>` storage is `\xcolor@{}{}{model}{spec}` and
  /// `\xcolor@` is a real macro (xcolor.sty:603 `\def\xcolor@#1#2#3#4{#2}`),
  /// so the fallback lookup must read the REPLACEMENT TEXT, not an
  /// expansion (which collapsed to ""): ydoc-desc.sty:22's empty `none`
  /// color raised "Can't find color named 'none'" (iodhbwm; Perl identical).
  #[test]
  fn xcolor_storage_macro_is_read_as_a_body() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\makeatletter
\expandafter\def\csname\string\color@none\endcsname{\xcolor@ {}{}{}{}}
\expandafter\def\csname\string\color@myred\endcsname{\xcolor@ {}{}{rgb}{1,0,0}}
\makeatother
\colorlet{cls}{none}
\begin{document}
\textcolor{cls}{hello} \textcolor{myred}{red}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("hello"), "{xml}");
    assert!(xml.contains(r##"color="#FF0000">red"##), "{xml}");
  }

  /// When xcolor is loaded, `\definecolor` registers `\\color@<name>` using the
  /// standard LaTeX shape `\xcolor@{}{<driver_spec>}{<model>}{<spec_comma>}`.
  /// Packages like colorspace.sty hook into `\xcolor@` inside `\definespotcolor`
  /// to inspect components and driver commands (colorspace.tex).
  #[test]
  fn def_color_macro_emits_xcolor_representation() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\definecolor{testc}{cmyk}{0.8,0.2,0.5,0.3}
\makeatletter
\def\spctest#1{%
  \begingroup
    \def\xcolor@##1##2##3##4{%
      \gdef\extractedmodel{##3}%
      \gdef\extractedspec{##4}}%
    \csname\string\color@#1\endcsname
  \endgroup}
\spctest{testc}
\makeatother
\begin{document}
Model: \extractedmodel, Spec: \extractedspec
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Model: cmyk, Spec: 0.8,0.2,0.5,0.3"), "{xml}");
  }

  /// LaTeX runs `\section`/`\paragraph` inside an `\item` or a float body (the
  /// heading is set in the list's indentation; ddphonism, phonrule, prerex,
  /// pdfmarginpar — pdflatex clean). Both engines build the nested
  /// `<ltx:item><ltx:subsection>`; Perl errors and inserts anyway
  /// (Document.pm openElement), so only the diagnostic differed. The builder's
  /// sectioning-in-frontmatter leniency now covers the whole sectioning
  /// family inside `ltx:item`/`ltx:figure` (OD #189).
  #[test]
  fn sectioning_unit_inside_item_or_figure_is_lenient() {
    let tex = r"\documentclass{article}
\begin{document}
\begin{itemize}
\item First item.
\subsection{Heading inside item}
More text.
\end{itemize}
\begin{figure}
Figure body text.
\paragraph{Notes} inside the figure.
\end{figure}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<subsection"), "{xml}");
    assert!(xml.contains("<paragraph"), "{xml}");
    assert!(xml.contains("<figure"), "{xml}");
  }

  /// A math node arriving in an Inline-model element opened in math mode — a
  /// `\hyperref[l]{b}` or glossaries' `\glsdisp{k}{k}` under `\ensuremath`
  /// (glosmathtools.sty:74; `<ltx:XMTok> isn't allowed in <ltx:glossaryref>`,
  /// sample_glosmathtools ×2 53 errors; Perl TeX_Math.pool:42 autoOpens only
  /// XMText, so it shares the error) — takes the `\text{$k$}` shape: an
  /// auto-opened inline `ltx:Math`/`ltx:XMath` inside the ref (OD #190).
  #[test]
  fn math_content_in_a_ref_gets_an_inline_math() {
    let tex = r"\documentclass{article}
\usepackage{hyperref}
\usepackage{glossaries}
\newglossaryentry{k}{name={\ensuremath{k}},description={discrete time}}
\begin{document}
\label{s}$x+\hyperref[s]{b}$ and \(a = \glsdisp{k}{k} + 1\).
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<ref font="italic" labelref="LABEL:s"><Math mode="inline""#),
      "{xml}"
    );
    assert!(xml.contains(r#"key="k"><Math mode="inline""#), "{xml}");
  }

  /// hyperref.sty:8183-8203 `\autopageref{label}` = `\hyperref[{label}]
  /// {\HyRef@autopagerefname\pageref*{label}}` — "page <n>" through the
  /// language's `\pageautorefname`; absent in Perl's hyperref.sty.ltxml
  /// (abntex2cite.tex:1367; KPE #164).
  #[test]
  fn autopageref_is_a_page_reference() {
    let tex = r"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\section{A}\label{s}
See \autopageref{s} and \autopageref*{s}.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(
        "See page\u{a0}<ref labelref=\"LABEL:s\"/> and page\u{a0}<ref labelref=\"LABEL:s\"/>."
      ),
      "{xml}"
    );
  }

  /// LaTeX's tabular entry template is a brace group (latex.ltx `\@classz`:
  /// `{\hfil\hskip1sp\ignorespaces\@sharp\unskip\hfil}`), so `\aftergroup`
  /// in a cell fires at the entry's `}` — inside the cell, before `&`/`\cr`
  /// is acted on. The cell frame's tokens used to be unread after the column
  /// ended, so babel's `\selectlanguage` (`\aftergroup\bbl@pop@language`) in
  /// a non-first cell ran as the NEXT cell and, after the last cell, opened a
  /// spurious one ("`\@end@tabular` Attempt to close boxing group";
  /// uantwerpenexam-example2 41, derivative 101; Perl identical).
  #[test]
  fn aftergroup_in_a_tabular_cell_fires_inside_the_cell() {
    let tex = r"\documentclass{article}
\usepackage[dutch,english]{babel}
\def\foo{\gdef\fired{[FIRED]}}
\begin{document}
\begin{tabular}{cc}%
\selectlanguage{english}A%
&
\selectlanguage{dutch}B%
\end{tabular}
\begin{tabular}{cc} a & \aftergroup\foo b \\ c & d \end{tabular}\fired
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
    assert!(xml.matches("<td").count() >= 6, "{xml}");
    assert!(xml.contains("[FIRED]"), "{xml}");
  }

  /// After a sectioning unit is leniently nested in a list item (OD #189),
  /// the NEXT sectioning command closes it and becomes its SIBLING inside the
  /// item — latex.ltx's `\@startsection` ends the previous heading's scope,
  /// not the list; a `\section` after `\end{itemize}` is at the outer level
  /// (ddphonism; Perl nests Y inside X with a second error).
  #[test]
  fn next_sectioning_unit_in_an_item_is_a_sibling() {
    let tex = r"\documentclass{article}
\begin{document}
\begin{itemize}
\item A \subsection{X} text \subsection{Y} more
\end{itemize}
\section{Z}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    let x = xml
      .find(r#"<subsection inlist="toc" xml:id="S0.SS1">"#)
      .expect("X");
    let x_end = xml[x..].find("</subsection>").expect("X end") + x;
    let y = xml
      .find(r#"<subsection inlist="toc" xml:id="S0.SS2">"#)
      .expect("Y");
    assert!(y > x_end, "Y must follow X's close as a sibling:\n{xml}");
    let item_end = xml.find("</item>").expect("item end");
    assert!(y < item_end, "Y stays inside the item:\n{xml}");
    let z = xml
      .find(r#"<section inlist="toc" xml:id="S1">"#)
      .expect("Z");
    assert!(z > xml.find("</itemize>").unwrap(), "{xml}");
  }

  /// beamer.cls:144-156 `\beamer@size` = the size .clo the class inputs (:363);
  /// themes read it (beamerthemeAlbi.sty:192 `size/.expanded=\beamer@size`
  /// as a pgfkeys choice). The binding's option remap never set it
  /// (beamer-theme-albi-doc; Perl identical).
  #[test]
  fn beamer_size_option_is_recorded() {
    let tex = r"\documentclass[14pt]{beamer}
\makeatletter
\def\showsize{[\expandafter\@firstofone\beamer@size]}
\makeatother
\begin{document}
\begin{frame}\showsize\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("[size14.clo]") || xml.contains("[size11.clo]"),
      "{xml}"
    );
  }

  /// beamerbaseoverlay.sty:590-597 wraps `\color` and the `\text<font>`
  /// commands with an `<overlay>` reader (Perl beamer.cls.ltxml:1345-1356
  /// `%BEAMER_WRAPPED`); without it `\color<2>{red}` read `<` as the color
  /// ("Can't find color named '<'", xskak_and_beamer 34 errors).
  #[test]
  fn beamer_color_and_text_commands_take_an_overlay() {
    let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}
\color<2>{red}Hello \textbf<2->{bold} \textcolor{blue}{b}
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r##"color="#FF0000""##), "{xml}");
    assert!(xml.contains(r#"font="bold">bold"#), "{xml}");
  }

  /// latex.ltx:15438 `\@xverbatim` is delimited by the catcode-12 string
  /// `\end{verbatim}` and `\end` runs `\endgroup` BEFORE the rest of that
  /// line is tokenized, so a `\verb` on the same line scans with restored
  /// catcodes. The pre-tokenized remainder (Perl latex_constructs.pool:1777)
  /// handed `\verb` frozen tokens: its delimiter never matched and the rest
  /// of the DOCUMENT was re-read under `\dospecials` (ddphonism:87; KPE #165).
  /// A TAB keeps catcode 10 in verbatim, so it is a space, not OT1 slot 9.
  #[test]
  fn verb_on_the_endverbatim_line_scans_raw() {
    let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{itemize}\n\\item A\n\\begin{verbatim}\n\tx\n\\end{verbatim} same \\verb|z| y.\n\\item B\n\\end{itemize}\n\\section{Next}\nT\n\\end{document}\n";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"same <verbatim font="typewriter">z</verbatim> y."#),
      "{xml}"
    );
    assert_eq!(xml.matches("<item xml:id").count(), 2, "{xml}");
    assert!(xml.contains("<section"), "{xml}");
    assert!(!xml.contains('Ψ'), "{xml}");
    assert!(
      xml.contains("\n x\n") || xml.contains("\n\u{2423}x\n") || xml.contains(">\n x"),
      "{xml}"
    );
  }

  /// marginnote.sty:319-343 routes the note body through three macro-argument
  /// layers (`\@dblarg\@mn@marginnote` → `\@mn@@marginnote` →
  /// `\@mn@@@marginnote`); a binding that expands straight to `\marginpar`
  /// (Perl marginnote.sty.ltxml:37-40) is one layer short, so skdoc.cls:631's
  /// `\marginnote{…\clist_map_inline:Nn…{\index@option*{####1}}}` leaked a
  /// literal `#1` and mis-keyed every glossary entry (iodhbwm 146 errors).
  #[test]
  fn marginnote_body_rides_three_argument_layers() {
    let tex = r"\documentclass{article}
\usepackage{marginnote}
\usepackage{xparse}
\ExplSyntaxOn
\DeclareDocumentCommand\Options{m}{
  \clist_set:Nn\l_tmpa_clist{#1}
  \marginnote{
    \clist_map_inline:Nn\l_tmpa_clist{ [####1] }
  }
}
\ExplSyntaxOff
\begin{document}
Body.\Options{alpha,beta} \marginnote[L]{R}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[alpha]") && xml.contains("[beta]"), "{xml}");
    assert!(!xml.contains("#1"), "{xml}");
    assert!(xml.contains(">R<") || xml.contains("R</note>"), "{xml}");
  }

  /// A registered contrib binding REPLACES the raw file: the schooldocs
  /// binding must load schooldocs.sty first (`\RequirePackage{xcolor}` :32,
  /// `titlecolor` :100, `\subject`…) and patch on top (schooldocs-examples
  /// 17 errors: `\definecolor` undefined; Perl raw-loads it clean).
  #[test]
  fn schooldocs_binding_loads_the_real_style() {
    let tex = r"\documentclass{article}
\usepackage{schooldocs}
\definecolor{darkbrown}{rgb}{0.5,0.1,0.1}
\begin{document}
\textcolor{darkbrown}{y}\textcolor{titlecolor}{t}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r##"color="#801A1A">y"##), "{xml}");
  }

  /// soul-ori.sty:557-567 `\SOUL@setup` resets the scanner's redefinable
  /// hooks; highlightx.sty:193 / proofread.sty:74 run it, redefine the hooks
  /// and hand text to the scanner `\SOUL@` (:131). The binding has no
  /// character scanner, so the hooks are plain macros and `\SOUL@` sets its
  /// argument as text (Perl: `\SOUL@setup` undefined; KPE #167).
  #[test]
  fn soul_scanner_surface_is_defined() {
    let tex = r"\documentclass{article}
\usepackage{soul}
\makeatletter
\begin{document}
\SOUL@setup\def\SOUL@preamble{}\SOUL@{highlighted text} \SOUL@ X \so{spaced}
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("highlighted text"), "{xml}");
    assert!(xml.contains("letter-spacing"), "{xml}");
  }

  /// latex.ltx:1832 `\g@addto@macro` appends at DIGESTION (its `\xdef`); an
  /// expandable side-effecting version (Perl :968) was executed by the
  /// `\ifnum` number scan's look-ahead (tex.web §444) even in a false branch
  /// (numspell-english.sty:79-105 `\ifnum…>0\numspell@{ hundred}\fi`; KPE #170).
  #[test]
  fn g_addto_macro_appends_at_digestion() {
    let tex = r"\documentclass{article}
\makeatletter
\def\out{}%
\def\g{\ifnum0>0\g@addto@macro\out{WRONG}\else\g@addto@macro\out{RIGHT}\fi}%
\g
\g@addto@macro\out{+MORE}
\AtBeginDocument{\g@addto@macro\out{+ABD}}
\makeatother
\begin{document}
[\out]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[RIGHT+MORE+ABD]"), "{xml}");
  }

  /// tabularray parses its own body and tolerates a row wider than the
  /// colspec (circularglyphs-doc.tex:196 `*{13}{X[m,c]}` with a 14-cell
  /// row; pdflatex and Perl clean); the kernel template is only a cap, so
  /// the tblr translation carries a margin of fallback columns. A plain
  /// tabular keeps erroring on an extra `&`.
  #[test]
  fn tblr_row_wider_than_the_colspec_is_tolerated() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{colspec={*{2}{c}}}
a & b \\
Null & & \\
\end{tblr}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(">Null<"), "{xml}");
    let tex = r"\documentclass{article}
\begin{document}
\begin{tabular}{cc} a & b & c \end{tabular}
\end{document}
";
    let (stderr, _xml) = convert(tex, false);
    assert!(
      error_count(&stderr) > 0,
      "a plain tabular's extra & stays an error:\n{stderr}"
    );
  }

  /// latex.ltx's `\nocite` writes `\citation{#1}` through
  /// `\protected@write` at the call site, so a key held in a transient
  /// macro is expanded there; the deferred raw key (Perl :4214) was expanded
  /// at `\end{document}` when tufte-common.def:934's `\@for\@temp@bibkeyx`
  /// loop variable no longer existed (tufte sample-book; KPE #171).
  #[test]
  fn nocite_expands_its_key_at_the_call_site() {
    let tex = r"\documentclass{article}
\makeatletter
\begin{document}
\def\keys{key1,key2}\marginpar{\@for\@temp@bibkeyx:=\keys\do{\nocite{\@temp@bibkeyx}}}
\nocite{*}
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"bibrefs="key1""#) && xml.contains(r#"bibrefs="key2""#),
      "{xml}"
    );
    assert!(xml.contains(r#"bibrefs="*""#), "{xml}");
  }

  /// report/book define `{titlepage}` with `\newenvironment`, so a class may
  /// `\def\titlepage{…}` as a plain vertical macro (uwthesis.cls:610, used as
  /// `{… \titlepage }`); the locked environment refused the `\def` and the
  /// bare `\titlepage` opened an environment frame the `}` then met
  /// (KPE #172). The environment itself still works.
  #[test]
  fn titlepage_environment_is_overridable() {
    let tex = r"\documentclass{report}
\makeatletter
\def\titlepage{\par TITLE STUFF\par}
\makeatother
\begin{document}
{\titlepage}After.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("TITLE STUFF") && !xml.contains("<titlepage"),
      "{xml}"
    );
    let tex = r"\documentclass{report}
\begin{document}
\begin{titlepage}\title{T}\author{A}\maketitle\end{titlepage}
Body.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<title>T</title>"), "{xml}");
  }

  /// The l3draw binding carries the full public surface; the path/state
  /// functions are absorbed but `\draw_box_use:N`/`\draw_coffin_use:Nnn`
  /// (l3draw.sty:40/:98) typeset their CONTENT (circledtext, tabular2,
  /// suanpan-l3 under lualatex).
  #[test]
  fn l3draw_surface_keeps_box_content() {
    let tex = r"\documentclass{article}
\usepackage{l3draw}
\ExplSyntaxOn
\box_new:N \l_tmp_box \hbox_set:Nn \l_tmp_box { INSIDE-BOX }
\NewDocumentCommand \mydraw { } {
  \draw_begin:
    \draw_set_linewidth:n { 1pt }
    \draw_path_scope_begin: \draw_path_circle:nn {0pt,0pt}{5pt} \draw_path_scope_end:
    \draw_box_use:N \l_tmp_box
  \draw_end: }
\ExplSyntaxOff
\begin{document}Before \mydraw{} After\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Before INSIDE-BOX After"), "{xml}");
  }

  /// The babel language stubs are FALLBACKS for a missing `.ldf`: when the
  /// real file is installed it is raw-loaded, so its `\DeclareOption
  /// {mexico}` (spanish.ldf:66-88) and `\bbl@declare@ttribute{czech}{split}`
  /// (czech.ldf:328) are honoured — the stub shadowed them ("Unknown option
  /// 'mexico'", unamthesis; "attribute split", csbulletin).
  #[test]
  fn installed_ldf_outranks_the_language_stub() {
    let tex = r"\documentclass{article}
\usepackage[english,spanish,mexico]{babel}
\begin{document}
\selectlanguage{spanish}Hola \selectlanguage{english}Hello
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hola") && xml.contains("Hello"), "{xml}");
    let tex = r"\documentclass{article}
\usepackage[czech,english]{babel}
\languageattribute{czech}{split}
\begin{document}
\selectlanguage{czech}Ahoj
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Ahoj"), "{xml}");
  }

  /// Real soul resolves a stored color name through `\color` at use time,
  /// which expands a macro-valued name (europasscv.cls:560 `\setulcolor
  /// {\ecv@textcolor}`); the binding stored it unexpanded (Perl
  /// soul.sty.ltxml:75 too; KPE #173). Same for `\setstcolor`/`\sethlcolor`.
  #[test]
  fn soul_color_setters_expand_a_macro_name() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{soul}
\definecolor{mycol}{HTML}{3E3A38}
\def\mycolname{mycol}
\begin{document}
\setulcolor{\mycolname}\ul{underlined text} \sethlcolor{\mycolname}\hl{hi}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r##"framecolor="#3E3A38""##), "{xml}");
    assert!(xml.contains(r##"backgroundcolor="#3E3A38""##), "{xml}");
  }

  /// nmbib.sty's `\citeall` (:343) runs natbib's low-level engine
  /// (`\NAT@reset@parser`, natbib.sty:780) that the natbib binding — a
  /// high-level `<ltx:cite>` emulation, like Perl's — does not carry (nmbib-
  /// sample 22 errors); the binding emulates it as `\citet*`.
  #[test]
  fn nmbib_citeall_is_a_cite() {
    let tex = r"\documentclass{article}
\usepackage{nmbib}
\begin{document}
Text \citeall{Markey:Tame_the_BeaST} and \citealn{Markey:Tame_the_BeaST}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"bibrefs="Markey:Tame_the_BeaST""#), "{xml}");
  }

  /// latex.ltx:16585-16594's array/tabular row CONTINUATION macros carry the
  /// closing half of `\@arraycr`'s `${` trick; reached directly (tablists.sty's
  /// `\TeXr@arraycr` inside its own raw `\halign`) the `$` had no partner and
  /// opened inline math the row's `\cr` could not balance (tablists-rus 101;
  /// Perl 12; KPE #174).
  #[test]
  fn array_continuation_macros_carry_no_math_shift() {
    let tex = r"\documentclass{article}\usepackage{tablists}
\begin{document}
\begin{tabenum}[\bfseries1)]
\tabenumitem aa;\\
\tabenumitem bb;\\[2pt]
\tabenumitem $c$;
\end{tabenum}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("aa") && xml.contains("bb"), "{xml}");
    assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  }

  /// A plain content `.tex` re-`\InputIfFileExists`ed while a `.sty` is being
  /// read is re-read every time (TeX; Perl `Input`); the once-only package
  /// guard skipped the second read, so babel's second `babel-french.tex` scan
  /// (french as BOTH class option and `main=`) never recorded french and
  /// french.ldf (→ `\og`, `\ieme`) never loaded (paresse-fra; KPE #175).
  #[test]
  fn content_tex_reinput_during_definitions_rereads() {
    let tex = r"\documentclass{article}
\usepackage{reinstyx}
\begin{document}
[\afterone][\aftertwo]
\end{document}
";
    let (stderr, xml) = convert_with_files(tex, &[
      ("helperx.tex", "\\def\\hmarker{SET}\\endinput\n"),
      (
        "reinstyx.sty",
        "\\ProvidesPackage{reinstyx}\n\\def\\hmarker{INIT}\\InputIfFileExists{helperx.tex}{}{}\\edef\\afterone{\\hmarker}%\n\\def\\hmarker{RESET}\\InputIfFileExists{helperx.tex}{}{}\\edef\\aftertwo{\\hmarker}%\n",
      ),
    ]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[SET][SET]"), "{xml}");
    let tex = r"\documentclass[french]{article}
\usepackage[english,main=french]{babel}
\begin{document}
\og guillemets\fg{} 1\ier{} 2\ieme
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("«") || xml.contains("guillemets"), "{xml}");
  }

  /// pdfmanagement's l3pdffile/l3pdfdict user surface (`\pdffile_embed_file:nnn`
  /// pdfmanagement.ltx:3389, `\pdfdict_put:nnn`) builds PDF/A associated-file
  /// objects nothing reads back (tagpdf's ex-AF-file.tex:29-32) — absorbed.
  #[test]
  fn pdffile_and_pdfdict_are_absorbed() {
    let tex = r"\DocumentMetadata{tagging=on,pdfversion=2.0,lang=de}
\documentclass{article}
\ExplSyntaxOn
\pdffile_embed_file:nnn{t.tex}{}{tag/AFtest}
\pdfdict_put:nnn {l_pdffile/Filespec} {AFRelationship}{/Supplement}
\ExplSyntaxOff
\begin{document}AF done\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("AF done"), "{xml}");
  }

  /// nicematrix's `\CodeAfter` grab must be environment-balanced: a
  /// `\begin{tikzpicture}…\end{tikzpicture}` inside it has its own `\end`
  /// (nicematrix-french: 23 stray `\endgroup`s + leaked pgf node errors).
  #[test]
  fn nicematrix_codeafter_grab_is_environment_balanced() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\usetikzlibrary{fit}
\begin{document}
\[\begin{pNiceMatrix}
121 & 23 & 345 \\ 45 & 346 & 863 \\ 3462 & 38458 & 34
\CodeAfter
\SubMatrix\{{2-2}{3-3}\}[name=A]
\begin{tikzpicture}
\node [fit = (A),fill = red!15] {} ;
\end{tikzpicture}
\end{pNiceMatrix}\]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("38458"), "{xml}");
  }

  /// The executed `\CodeBefore` block keeps its color commands but drops the
  /// drawing `tikzpicture`/`scope` overlays that reference cell nodes LaTeXML
  /// never materializes (`create-cell-nodes`, nicematrix-french ×280).
  #[test]
  fn nicematrix_codebefore_drops_drawing_environments() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\usetikzlibrary{fit}
\begin{document}
\[\begin{pNiceMatrix}
\CodeBefore [create-cell-nodes]
\cellcolor{red}{1-1}
\begin{tikzpicture}
\node [fit = (2-2), fill=red!15] {} ;
\end{tikzpicture}
\Body
a & a + b \\ a & a
\end{pNiceMatrix}\]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("backgroundcolor="), "{xml}");
  }

  /// Under `ampersand-in-blocks` a `\Block` body holding `&` is a sub-grid
  /// (nicematrix.sty:7592 `\__nicematrix_Block_vii`, a tabular in text / an
  /// array in math split on `&`); emitting it bare re-exposed the `&` to the
  /// outer alignment (nicematrix.tex:1152; "Extra alignment tab").
  #[test]
  fn nicematrix_block_ampersand_body_is_a_subgrid() {
    let tex = r"\documentclass{article}
\usepackage[ampersand-in-blocks]{nicematrix}
\begin{document}
\begin{NiceTabular}{ll}
\Block{}{one & two & three} & x \\
a & b
\end{NiceTabular}
$\begin{pNiceMatrix}
\Block{}{1 & 2} & c \\ d & e
\end{pNiceMatrix}$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(">three<"), "{xml}");
    assert!(xml.matches("<tabular").count() >= 2, "{xml}");
  }

  /// A raw class redefining a locked frontmatter command as a plain setter
  /// (afthesis.cls:520 `\def\author#1{\def\auth@r{#1}}`) is dropped
  /// (Perl State.pm:502-517), and its readers then fail on the internal it
  /// would have defined (`\flyleaf`/`\titlepage` :637/:688; usethesis). The
  /// dropped body's `\def`-targets are defined EMPTY, the class's own default
  /// convention (:494-495), so the locked binding stays the single source.
  #[test]
  fn locked_setter_internals_are_defined_empty() {
    let tex = r"\documentclass{article}
\makeatletter
\def\author#1{\def\auth@r{#1}\gdef\auth@rtwo{#1}}
\makeatother
\author{First Author}
\begin{document}
\makeatletter[\auth@r][\auth@rtwo]\makeatother\maketitle
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[][]"), "{xml}");
    assert!(xml.contains("First Author"), "{xml}");
  }

  /// xcolor.sty:762-763 `\color` = `\@ifnextchar[\@undeclaredcolor\@declaredcolor`;
  /// fancyqr.sty:20-22 calls the named-color branch directly. Both engines
  /// bind `\color` monolithically and lacked the branches (KPE #177).
  #[test]
  fn color_switch_branches_are_defined() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\definecolor{tl}{HTML}{FF0000}\definecolor{br}{HTML}{3D3A38}
\begin{document}
\makeatletter{\@declaredcolor{tl!50!br}Hello} {\@undeclaredcolor[rgb]{0,0,1}Blue}\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r##"color="#9E1D1C">Hello"##), "{xml}");
    assert!(xml.contains(r##"color="#0000FF">Blue"##), "{xml}");
  }

  /// A raw biblatex style `.def` (biblatex-sbl.def:663) replaces
  /// `\printbibliography` with biblatex's real body, which reaches the
  /// `\blx@key@bibcheck` / `\blx@printbibliography` internals the binding
  /// stands in for (biblatex.sty:9643/:9820). Witness biblatex-sbl/sbl-paper.
  #[test]
  fn style_def_printbibliography_override_routes_to_binding() {
    let tex = r"\documentclass{article}
\usepackage[style=sbl,backend=biber]{biblatex}
\begin{document}
Text.
\printbibliography[heading=bibintoc]
\end{document}
";
    let (stderr, _xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// The siunitx `S`/`s` cell is read with expansion under LaTeX's
  /// `\protected@edef` context (`\protect` = `\@unexpandable@protect`,
  /// latex.ltx:1384): a raw class's size command then stays `\protect\small `
  /// instead of expanding — `\@setfontsize` (latex.ltx:14103) reaches
  /// `\@currsize` → `\normalsize` → itself under `\@typeset@protect`, the same
  /// overflow as pdflatex's `\edef\x{\small}`. The cell is emitted as ONE
  /// GROUP (LaTeX's column template wraps every entry in `{…}`) so the size
  /// stays scoped to the cell. Witness zugferd-invoice.sty:113 `\small\emph
  /// {Pos.}&…` in an `S` column under scrartcl (`PushbackLimit`; pdflatex
  /// clean). The number still parses; under article the size is applied.
  #[test]
  fn s_column_unbraced_size_command_is_scoped() {
    let tex = r"\documentclass{scrartcl}
\usepackage{siunitx}
\begin{document}
\begin{tabular}{S}
\small a \\
1.5 \\
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<Math mode="inline" tex="1.5""#), "{xml}");
    let article = tex.replace("scrartcl", "article");
    let (stderr, xml) = convert(&article, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<text fontsize="90%">a</text>"#), "{xml}");
    assert!(xml.contains(r#"<Math mode="inline" tex="1.5""#), "{xml}");
  }

  /// A pure size switch is LaTeX's `\@setfontsize` (latex.ltx:14103), whose
  /// first act is `\let\@currsize#1`; packages test the identity with
  /// `\ifx\@currsize\small` (ltugboat's `\SMC` cascade in
  /// latex-doc-ptr.sty:203-215, else `\TBWarning`). With the class
  /// binding's primitive alone `\@currsize` never matched any size.
  #[test]
  fn size_switch_lets_currsize() {
    let tex = r"\documentclass{article}
\makeatletter
\DeclareRobustCommand{\SMC}{\ifx\@currsize\normalsize\small\else
 \ifx\@currsize\small\footnotesize\else
  \ifx\@currsize\large\normalsize\else NOSIZE\fi\fi\fi}
\makeatother
\begin{document}
{\small A\SMC B}
{\large C\SMC D}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("NOSIZE"), "{xml}");
    // `\small`→`\footnotesize` (B), `\large`→`\normalsize` (D, base size,
    // no wrapper).
    assert!(xml.contains(r#"<text fontsize="89%">B</text>"#), "{xml}");
    assert!(
      xml.contains(r#"<text fontsize="120%">C</text>D</p>"#),
      "{xml}"
    );
  }

  /// caption3.sty:1595 `\providecommand*\caption@prepareslc{}` is an empty
  /// hook other packages extend (hep-bibliography.sty:108; 9 hep-* docs).
  #[test]
  fn caption_prepareslc_hook_is_defined() {
    let tex = r"\documentclass{article}
\usepackage{caption}
\makeatletter
\g@addto@macro\caption@prepareslc{\relax}
\makeatother
\begin{document}
Hello.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hello."), "{xml}");
  }

  /// titlesec.sty:1039-1041 `\newdimen\titlewidth…` (titlesec.tex:1780).
  #[test]
  fn titlesec_title_width_registers_exist() {
    let tex = r"\documentclass{article}
\usepackage{titlesec}
\titleformat{\section}[block]
  {\addtolength{\titlewidth}{2pc}\normalfont\sffamily}
  {\thesection}{1em}{}
\begin{document}
\section{Hello}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<title font=\"sansserif\">") && xml.contains("Hello</title>"),
      "{xml}"
    );
  }

  /// ntheorem.sty:714-715 `\newskip\thm@topsep`/`\thm@topsepadd`
  /// (dlfltxbcodetips.sty:102-106 copies ntheorem's code).
  #[test]
  fn ntheorem_topsep_registers_exist() {
    let tex = r"\documentclass{article}
\usepackage{amsmath,amssymb}
\usepackage[amsmath,thmmarks,framed]{ntheorem}
\makeatletter
\thm@topsepadd \theorempostskipamount
\advance\thm@topsepadd\partopsep
\makeatother
\begin{document}
Hi.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hi."), "{xml}");
  }

  /// mathtools.sty:1897-1907 `\xmathstrut` is a `\vphantom` strut
  /// (numerica.tex:3431 inside `\eval{\[\frac…\]}`).
  #[test]
  fn xmathstrut_is_a_vphantom() {
    let tex = r"\documentclass{article}
\usepackage{mathtools}
\begin{document}
\[ \frac{\xmathstrut{0.1} a}{\xmathstrut{0.4} b} \]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<XMApp"#) && xml.contains("phantom"),
      "{xml}"
    );
  }

  /// isorot's raw `\@xrotfloat` (isorot.sty:139-147) builds the sideways
  /// float as an lrbox + minipage capture, inside which `\caption`'s float-up
  /// finds no float ("`<ltx:caption>` isn't allowed in `<ltx:block>`";
  /// isorot/rotman, Perl identical, pdflatex clean). The binding gives the
  /// float environments rotating's shape, so the caption is the float's child.
  #[test]
  fn isorot_sideways_float_holds_its_caption() {
    let tex = r"\documentclass{article}
\usepackage{isorot}
\begin{document}
\begin{sidewaystable}
\centering
\caption{The rotation facilities}
\begin{tabular}{|l|l|}\hline A & B \\\hline\end{tabular}
\end{sidewaystable}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<table") && xml.contains("<caption class=\"ltx_centering\">"),
      "{xml}"
    );
    assert!(!xml.contains("<block>"), "{xml}");
  }

  /// adjmulticol.sty:151 raw-calls multicol.sty:172 `\mult@@cols`, the
  /// column balancer LaTeXML never emulates; bound, `adjmulticols` emits the
  /// same pagination markers as `multicols` (adjmulticol/sample).
  #[test]
  fn adjmulticols_are_pagination_markers() {
    let tex = r"\documentclass{book}
\usepackage{adjmulticol}
\begin{document}
\begin{adjmulticols}{2}{12pt}{-2in}
Some text flowing across two adjusted columns. More text here.
\end{adjmulticols}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"<pagination role="start_2_columns"/>"#),
      "{xml}"
    );
    assert!(
      xml.contains(r#"<pagination role="end_2_columns"/>"#),
      "{xml}"
    );
    assert!(xml.contains("Some text flowing"), "{xml}");
  }

  /// Raw biblatex style files reach the biblatex.sty internal/public surface
  /// at cite/bibliography time (windycity data-model declarations, sbl's
  /// `\citeshorthand` control flow, juradiss' `\AtDataInput`); the binding
  /// stands in for biblatex.sty and carries that surface.
  #[test]
  fn biblatex_style_internal_surface() {
    for (style, body) in [
      ("windycity", r"Text.\par \printbibliography"),
      ("sbl", r"See \citeshorthand{SBL} and \cite{SBLHS}."),
      ("biblatex-juradiss", r"Text.\par \printbibliography"),
    ] {
      let tex = format!(
        "\\documentclass{{article}}\n\\usepackage[style={style}]{{biblatex}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
      );
      let (stderr, xml) = convert(&tex, false);
      assert_eq!(error_count(&stderr), 0, "{style}: {stderr}");
      assert!(
        xml.contains("Text.") || xml.contains("<cite class="),
        "{style}: {xml}"
      );
    }
  }

  /// hyperref's low-level URL chain (`\hyper@normalise` :4604 → `\url@`
  /// :4802 → `\hyper@linkurl`/`\Hurl`) reached by biblatex.tex's `\fnurl`;
  /// the neutralised read keeps `#`/`%`/`~`.
  #[test]
  fn hyperref_normalise_chain_links_urls() {
    let tex = r"\documentclass{article}
\usepackage{hyperref}
\makeatletter
\newcommand\fnurl@[1]{\footnote{\url@{#1}}}
\DeclareRobustCommand\fnurl{\hyper@normalise\fnurl@}
\makeatother
\begin{document}
See the docs.\fnurl{https://ctan.org/pkg/biblatex#frag~x}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r##"href="https://ctan.org/pkg/biblatex#frag~x""##),
      "{xml}"
    );
    assert!(xml.contains("biblatex#frag~x</ref>"), "{xml}");
  }

  /// longtable.sty:135-137 redefine `\newpage`/`\pagebreak`/`\nopagebreak`
  /// inside the table to `\noalign{…}`, so tex.web §785 `align_peek` takes
  /// the no_align branch instead of starting a row (harmony: `\newpage`
  /// between `\hline` rows; `\clearpage` is NOT redefined and errors in
  /// pdflatex too).
  #[test]
  fn longtable_page_commands_between_rows_are_noalign() {
    for cmd in [r"\newpage", r"\nopagebreak", r"\pagebreak[2]"] {
      let tex = format!(
        "\\documentclass{{article}}\n\\usepackage{{longtable}}\n\\begin{{document}}\n\\begin{{longtable}}{{ll}}\n\\hline a & b \\\\ \\hline\n{cmd}\n\\hline c & d \\\\ \\hline\n\\end{{longtable}}\n\\end{{document}}\n"
      );
      let (stderr, xml) = convert(&tex, false);
      assert_eq!(error_count(&stderr), 0, "{cmd}: {stderr}");
      assert_eq!(xml.matches("<td").count(), 4, "{cmd}: {xml}");
    }
  }

  /// threeparttable.sty:110 (`\def\@captype{table}` if undefined) and :126
  /// (measuredfigure → `figure`) let `\caption` work outside a float; both
  /// bindings bound a bare `#body` and dropped it (threeparttablex;
  /// PERL-ORIGIN, threeparttable.sty.ltxml:31,36).
  #[test]
  fn threeparttable_sets_captype_outside_a_float() {
    let tex = r"\documentclass{article}
\usepackage{threeparttable}
\begin{document}
\begin{threeparttable}
\caption{A table}
\begin{tabular}{l} a \\ \end{tabular}
\end{threeparttable}
\begin{measuredfigure}
\caption{A figure}
\end{measuredfigure}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // Outside a float the kernel degrades the caption to inline text (guard
    // `caption_without_a_float_ancestor_degrades_to_text`); the point here is
    // that `\@captype` is defined, so no "outside any known float" error.
    assert_eq!(
      xml.matches(r#"<text class="ltx_caption">"#).count(),
      2,
      "{xml}"
    );
  }

  /// A block listing in a `p{}` cell: the listing's group must close with
  /// an implicit `\egroup` (tex.web §347: only `{`/`}` characters move
  /// `align_state`), else the cell's `&`/`\\` stop being column ends
  /// (pfdicons-doc, tikzcodeblocks-documentation, shipunov; pdflatex clean).
  /// The `\parbox` form errors in pdflatex too and stays an error.
  #[test]
  fn block_listing_in_a_paragraph_cell() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\begin{tabular}{p{4cm}l}
\begin{lstlisting}[numbers=none]
x=1;
\end{lstlisting} & b \\
c & d \\
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<td").count(), 4, "{xml}");
    assert!(xml.contains("<listing"), "{xml}");
    let control = tex
      .replace(
        r"\begin{tabular}{p{4cm}l}",
        r"\begin{tabular}{ll}\parbox{4cm}{",
      )
      .replace(r"\end{lstlisting} & b", r"\end{lstlisting}} & b");
    let (stderr, _xml) = convert(&control, false);
    assert!(
      error_count(&stderr) > 0,
      "CONTROL: pdflatex errors here too\n{stderr}"
    );
  }

  /// latex.ltx `\@tabarray` = `\m@th\@ifnextchar[\@array{\@array[c]}` — the
  /// full array setup; a package building its own array on it (t-angles.sty:491)
  /// nested in an outer array cell under `\begingroup` broke the outer cell's
  /// group (t-angles/t-manual, 101 errors; Perl identical, pdflatex clean).
  #[test]
  fn tabarray_is_the_full_array_setup() {
    let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{t-angles}
\def\SHOW#1#2{\begin{array}{c}\begin{tangle}#1\end{tangle}\\ \hbox{\tt\string#2}\end{array}}
\def\Show#1{\SHOW#1#1}
\begin{document}
$$ \Show\id \quad \Show\n $$
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.matches("<XMArray").count() >= 2, "{xml}");
  }

  /// colortbl's `\CT@*` internal surface for raw derivatives (tabu.sty:720
  /// assigns to and `\the`s `\CT@everycr`, colortbl.sty:116 `\let…\everycr`).
  #[test]
  fn colortbl_internal_surface_is_defined() {
    let tex = r"\documentclass{article}\usepackage{colortbl}
\makeatletter
\CT@everycr\expandafter{\expandafter\relax\the\CT@everycr}
\CT@arc@\CT@column@color\CT@row@color\CT@cell@color\CT@do@color
\makeatother
\begin{document}
x
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>x</p>"), "{xml}");
  }

  /// A box capture (`insert_block`'s `ltx:_CaptureBlock_`) is a completed
  /// box: its non-auto-closeable descendants (a `verbatim`, listing lines)
  /// are closed by the box, not reported (testnumberedblock; Perl emitted
  /// the same spurious error over the same tree).
  #[test]
  fn capture_box_closes_its_descendants() {
    let tex = r"\documentclass{article}
\usepackage{numberedblock}
\begin{document}
\begin{numVblock}
This is a labeled numVblock
program test
\end{numVblock}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("program test"), "{xml}");
    assert!(!xml.contains("_CaptureBlock_"), "{xml}");
  }

  /// physics2 `ab.braket`: the active `|` in `\braket<a|b>` is a
  /// `\middle\vert` without the `\egroup…\bgroup` atom split that
  /// LaTeXML's token-level `\left` capture cannot pair (physics2,
  /// physics2-legacy; lualatex clean).
  #[test]
  fn physics2_braket_active_bar_is_a_middle_fence() {
    let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{physics2}
\usephysicsmodule{ab,ab.braket}
\begin{document}
\[ \bra<\phi| \quad \ket|\psi> \quad \braket<\phi> \]
\[ \braket<\phi|\psi> \quad \braket<\phi|A|\psi> \quad \ketbra|\phi><\psi| \]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains(r#"role="MIDDLE""#) || xml.contains("∣"),
      "{xml}"
    );
  }

  /// physics2 + unicode-math: `physics2`'s `\vert` is redefined via `\Udelimiter`
  /// when `unicode-math` defines `\symrm`. Multi-dot dispatch loads
  /// `phy-ab.braket_sty.rs` and `\Udelimiter` constructor avoids mathcode 8000
  /// recursion on `\middle\vert` (witness: egroup_braket_physics2.tex).
  #[test]
  fn physics2_braket_with_unicode_math_delimiters() {
    let tex = r"\documentclass{article}
\usepackage{amsmath}\usepackage{unicode-math}\usepackage{physics2}
\usephysicsmodule{ab,ab.braket}
\begin{document}
\[ \braket< a | b > \]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"role="MIDDLE""#), "{xml}");
  }

  /// physics2 `\delopen` and `\delclose` paired with active pipe inside
  /// `\bgroup`..`\egroup` delimited math (witness: egroup_delopen_activepipe_reduced.tex).
  #[test]
  fn physics2_delopen_delclose_active_pipe_reduced() {
    let tex = r#"\documentclass{article}
\usepackage{amsmath}\usepackage{unicode-math}\usepackage{physics2}
\begingroup\catcode`\|=\active
\gdef\mytest{\begingroup\mathcode`\|="8000\def|{\egroup\vert\bgroup}%
  \delopen\langle\bgroup a|b\egroup\delclose\rangle\endgroup}
\endgroup
\begin{document}
\[ \mytest \]
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<XMWrap"), "{xml}");
  }

  /// tex.web §1206: a `\noalign` body is EXECUTED to the `}` closing its
  /// group; latex.ltx's `\hline` brace hack (`\noalign{\ifnum0=`}\fi…`) has a
  /// char-constant `}` a token pre-scan miscounted, leaking the rule into the
  /// alignment (boldline `\hlineB`, shipunov/boldline-ex-en; Perl identical).
  #[test]
  fn noalign_body_is_executed_to_its_group_end() {
    let tex = r"\documentclass{article}\usepackage{array}
\makeatletter
\def\myhline{\noalign{\ifnum0=`}\fi\hrule \@height \arrayrulewidth \futurelet\reserved@a\@xmyhline}
\def\@xmyhline{\ifx\reserved@a\myhline\fi\ifnum0=`{\fi}}
\makeatother
\begin{document}
\begin{tabular}{c}a\\\myhline b\\\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // the raw full-width `\hrule \@height…` is the next row's top border
    assert!(
      xml.contains(">a") && xml.contains(">b") && xml.contains(r#"border="t""#),
      "{xml}"
    );
    let boldline = r"\documentclass{article}\usepackage{boldline}
\begin{document}
\begin{tabular}{cc}\hlineB{2.5} a & b \\ \hlineB{2.5} c & d \\ \hlineB{2.5}\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(boldline, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    for cell in [">a<", ">b<", ">c<", ">d<"] {
      assert!(xml.contains(cell), "{cell}: {xml}");
    }
  }

  /// tabu's remaining user surface: `\everyrow`, `\rowfont`, and the
  /// `\extrarowsep` assignment syntax (tabu.sty:232) over
  /// `\extrarowheight`/`\extrarowdepth`.
  #[test]
  fn tabu_row_surface_is_covered() {
    let tex = r"\documentclass{article}\usepackage{tabu}
\begin{document}
\extrarowsep=2pt \extrarowsep^=3pt \extrarowsep=^1pt_2pt
\everyrow{\hline}
\begin{tabu}{ll}\rowfont[c]{\bfseries} a & b \\ c & d \\\end{tabu}
\the\extrarowheight/\the\extrarowdepth
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<td").count(), 4, "{xml}");
    assert!(xml.contains("1.0pt/2.0pt"), "{xml}");
  }

  /// latex.ltx:10005 `\@tabacckludge`: inside tabbing `\a=`/`\a<`/`\a>` reach
  /// the encoding-level accents although `\=`/`\<`/`\>` are tab operators,
  /// and an accent tabbing never rebinds (`\a"`) is the accent itself
  /// (encguide, greek-fontenc; Perl saved only `'` and `` ` ``).
  #[test]
  fn tabbing_accent_kludge_recovers_rebound_accents() {
    let tex = r#"\documentclass{article}
\begin{document}
\begin{tabbing}
xxx \= yyy \\
\a=o \> \a'e \a"u \\
\end{tabbing}
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("ō") && xml.contains("é") && xml.contains("ü"),
      "{xml}"
    );
  }

  /// beamerbasetemplates.sty:26 `\ifbeamertemplateempty` gates theme code on
  /// whether a template is set (beamerthemeAlbi; 43-error `\fi` cascade).
  #[test]
  fn beamer_template_empty_test_is_defined() {
    let tex = r"\documentclass{beamer}
\makeatletter
\ifbeamertemplateempty{logo}{EMPTY}{NONEMPTY}
\makeatother
\begin{document}
\begin{frame}Hi\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("EMPTY") && !xml.contains("NONEMPTY"), "{xml}");
  }

  /// amsart's raw `\maketitle` internals reached by a derivative class that
  /// redefines `\maketitle` over `\LoadClass{amsart}` (resphilosophica).
  #[test]
  fn amsart_maketitle_internals_are_defined() {
    let tex = r"\documentclass{resphilosophica}
\author{Alice}
\title{T}
\dedicatory{For X}
\begin{document}
\begin{abstract}Abs.\end{abstract}
\maketitle
Body.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<creator") && xml.contains("<abstract"),
      "{xml}"
    );
  }

  /// quantumview's `\renewcommand{\author}` cannot override the locked
  /// kernel `\author`, so its author-group list init never runs and the raw
  /// `\maketitle` loop meets an undefined `\@authorgroup`; the class
  /// binding initialises the lists (creators still captured).
  #[test]
  fn quantumview_author_group_lists_are_initialised() {
    let tex = r"\documentclass{quantumview}
\title{T}
\author{Alice}
\affiliation{Somewhere}
\begin{document}
\maketitle
Body.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<creator"), "{xml}");
  }

  /// A full-width `\hrule` with an explicit height inside `\noalign` is a
  /// horizontal rule → the next row's top border (it was silently dropped).
  #[test]
  fn noalign_rule_with_height_is_a_border() {
    let tex = r"\documentclass{article}
\begin{document}
\begin{tabular}{ll}
\noalign{\hrule height 1pt}
a & b \\
c & d \\
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<td").count(), 4, "{xml}");
    assert!(xml.contains(r#"border="t""#), "{xml}");
  }

  /// gauss `gmatrix`: the amsmath matrix its delimiter names plus the
  /// row/column operations as a math annotation (raw gauss measures the box
  /// with a `\lastbox` recursion whose termination is a physical width).
  #[test]
  fn gauss_gmatrix_renders_with_operation_lines() {
    let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{gauss}
\begin{document}
\[ \begin{gmatrix}[p] 1 & 2 \\ 3 & 4
\rowops \mult{0}{\cdot 2} \add[3]{0}{1} \swap{0}{1}
\end{gmatrix} \]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<XMArray"), "{xml}");
    assert!(
      xml.contains("←") || xml.contains("&#8592;") || xml.contains("leftarrow"),
      "{xml}"
    );
  }

  /// latex.ltx `\marginpar` is a macro (`\@ifnextchar[\@xmpar\@ympar`); a
  /// package prepending to it by expansion (marginfix.sty:91) must capture
  /// its body, not the bare token.
  #[test]
  fn marginpar_is_a_macro_over_its_constructor() {
    let tex = r"\documentclass{article}
\makeatletter
\edef\marginpar{\unexpanded{\typeout{pre}}\expandafter\unexpanded\expandafter{\marginpar}}
\makeatother
\begin{document}
Text\marginpar{Note}\marginpar[L]{R} more.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // `\marginpar[L]{R}` yields a left and a right note
    assert_eq!(xml.matches(r#"role="margin""#).count(), 3, "{xml}");
    assert!(xml.contains("ltx_marginpar_left"), "{xml}");
  }

  /// subfiles.sty:171 `\ifSubfilesClassLoaded{yes}{no}` (sshrc-insight).
  #[test]
  fn subfiles_class_loaded_test_is_defined() {
    let tex = r"\documentclass{article}\usepackage{subfiles}
\begin{document}
\ifSubfilesClassLoaded{CLASS}{PACKAGE}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("PACKAGE") && !xml.contains("CLASS"), "{xml}");
  }

  /// xparse `O{}` options nest brackets: nicematrix.tex:1364
  /// `[rules/color=[gray]{0.9},…]` was cut at the inner `]`, spilling the rest
  /// into the table (16 "Extra alignment tab" cascades in nicematrix).
  #[test]
  fn optional_balanced_nests_brackets() {
    let tex = r"\documentclass{article}\usepackage{nicematrix,xcolor}
\begin{document}
\begin{NiceTabular}{|ccc|}[rules/color=[gray]{0.9},rules/width=1pt,no-cell-nodes]
\hline
a & b & c \\
\hline
\end{NiceTabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<td").count(), 3, "{xml}");
  }

  /// fontspec's `\IfFontExistsTF` is a texmf-tree lookup, not a constant
  /// false (asmeconf's class-level font checks under the luatex profile).
  #[test]
  fn font_exists_test_consults_the_texmf_tree() {
    let tex = r"\documentclass{article}\usepackage{fontspec}
\begin{document}
\IfFontExistsTF{lmroman10-regular.otf}{YES1}{NO1}
\IfFontExistsTF{nonsense-font-xyz.otf}{YES2}{NO2}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("YES1") && xml.contains("NO2"), "{xml}");
  }

  /// Singleton internals reached by raw packages over the standing-in
  /// bindings: hyperref's driver sentinel (hrefhide.sty:154), doclicense's
  /// layout wrapper (beautynote), listings' `\lst@XConvert` consumer.
  #[test]
  fn singleton_internal_surface() {
    let tex = r"\documentclass{article}\usepackage{hyperref}\usepackage{doclicense}\usepackage{listings}
\makeatletter
\def\hrefhide@driver{hpdftex}
\begin{document}
\ifx\Hy@driver\hrefhide@driver DRIVER-OK\fi
\lst@XConvert{abc}\@nil
\doclicenseThis
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("DRIVER-OK"), "{xml}");
  }

  /// `\DeclareMathVersion{name}` registers a version `\mathversion{name}`
  /// may select (oz, askmaps, iwonamath, zed); an undeclared one still errors.
  #[test]
  fn declared_math_versions_are_selectable() {
    let tex = r"\documentclass{article}
\makeatletter
\DeclareMathVersion{oz}
\makeatother
\begin{document}
\mathversion{oz}$x=1$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<Math"), "{xml}");
    let (stderr, _xml) = convert(&tex.replace(r"\DeclareMathVersion{oz}", ""), false);
    assert_eq!(
      error_count(&stderr),
      1,
      "CONTROL: undeclared version errors\n{stderr}"
    );
  }

  /// array.sty's `\@mkpream` templates the cell as `\@sharp` (a cs `\let` to
  /// `#`); a package-assembled `\ialign` (sgame, tabularcalc, tabvar) is a real
  /// alignment once the raw `\halign` reader recognises the meaning (tex.web
  /// §783). A `\noalign` outside any alignment still errors.
  #[test]
  fn ialign_template_accepts_the_sharp_placeholder() {
    let tex = r"\documentclass{article}\makeatletter
\begin{document}
\let\@sharp=#
\ialign{\hfil\@sharp\hfil&&\hfil\@sharp\hfil\cr a&b\cr c&d\cr}
\makeatother\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<td").count(), 4, "{xml}");
    let control = r"\documentclass{article}\begin{document}
x \noalign{\hrule} y
\end{document}
";
    let (stderr, _xml) = convert(control, false);
    assert!(
      error_count(&stderr) >= 1,
      "CONTROL: \\noalign outside an alignment errors\n{stderr}"
    );
  }

  /// `\usetheme[opts]{name}` passes its options to the theme as package
  /// options and `\ProcessOptionsBeamer` applies them (beamerbasethemes.sty:
  /// 18, beamerbaseoptions.sty:15): Verona's `sidebar` option installs the
  /// real `\sidegraphics` instead of its "defined only with the 'sidebar'
  /// option" stub. A theme without options (Albi) loads as before.
  #[test]
  fn usetheme_options_reach_the_theme() {
    let tex = r"\documentclass{beamer}
\usetheme[sidebar]{Verona}
\begin{document}
\begin{frame}\sidegraphics<1>{plato}{scale=1.1}\end{frame}
\end{document}
";
    let (stderr, _xml) = convert(tex, true);
    assert!(
      !stderr.contains("defined only with the 'sidebar' option"),
      "{stderr}"
    );
    let albi = r"\documentclass{beamer}\usetheme{Albi}\begin{document}\begin{frame}Hi\end{frame}\end{document}
";
    let (stderr, xml) = convert(albi, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hi"), "{xml}");
  }

  /// tex.web §211: `\ifinner` is the box/inline-math interior sign — false
  /// at the main galley (paracol.sty:1996 `\ifinner\@parmoderr`; tidyres),
  /// true inside `\parbox`/`$…$`, false in display math.
  #[test]
  fn ifinner_is_the_box_frame_sign() {
    let tex = r"\documentclass{article}\usepackage{paracol}
\begin{document}
\par\ifinner INNER1\else OUTER1\fi
{\par\ifinner INNER2\else OUTER2\fi}
\parbox{5cm}{\par\ifinner INNER3\else OUTER3\fi}
$\ifinner I4\else O4\fi$ \[\ifinner I5\else O5\fi\]
\begin{paracol}{2}Left.\switchcolumn Right.\end{paracol}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    for want in ["OUTER1", "OUTER2", "INNER3", "I4", "O5"] {
      assert!(xml.contains(want), "{want}: {xml}");
    }
  }

  /// Under the `[luatex]` profile pgf takes its LuaTeX branch (keyed on
  /// `\directlua`) and expects Lua to define `\pgfutil@luaescapestring`; the
  /// binding supplies pgf's own TeX fallback (neoschool, beamerthemeCelestia).
  #[test]
  fn pgf_lua_entry_points_have_their_tex_fallback() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{graphdrawing,graphs}
\usegdlibrary{trees}
\begin{document}
\tikz \graph[tree layout] { a -> {b, c} };
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<svg:svg") || xml.contains("<svg"), "{xml}");
  }

  /// A float inside a Block container escapes to the enclosing `ltx:para`
  /// (the `^` float-up marker), as LaTeX floats escape their environment
  /// (isorot/rotman, bashful; Perl placed it in the quote).
  #[test]
  fn floats_escape_block_containers() {
    let tex = r"\documentclass{article}
\begin{document}
\begin{quote}
\begin{figure}
\caption{X}
\end{figure}
\end{quote}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<figure") && !xml.contains("<quote>\n    <figure"),
      "{xml}"
    );
    let plain = r"\documentclass{article}\begin{document}
Text.
\begin{figure}\caption{Y}\end{figure}
\end{document}
";
    let (stderr, xml) = convert(plain, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<figure"), "{xml}");
    // float.sty custom floats (bashful's `program`) go the same way.
    let custom = r"\documentclass{article}\usepackage{float}
\newfloat{program}{tbp}{lop}
\begin{document}
\begin{itemize}\item
\begin{program}\caption{Z}\end{program}
\end{itemize}
\end{document}
";
    let (stderr, xml) = convert(custom, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<float") && !xml.contains("<item>\n      <float"),
      "{xml}"
    );
  }

  /// achemso.cls:1022-1030 declares `scheme`/`chart`/`graph` floats through
  /// float.sty; the binding must too, or `\caption` inside `scheme` cascades
  /// (achemso-demo; RUST-ONLY, Perl raw-loads the class).
  #[test]
  fn achemso_declares_its_scheme_floats() {
    let tex = r"\documentclass{achemso}
\author{A}\title{T}
\begin{document}
\begin{scheme}
\caption{An example scheme}
\end{scheme}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("ltx_float_scheme") && xml.contains("<caption>"),
      "{xml}"
    );
  }

  /// Stub class bindings must issue the float-package requires of the real
  /// class: jmlr.cls:155 algorithm2e, oup.cls:137 rotating (pmlr-sample,
  /// oup-authoring-template; RUST-ONLY, Perl raw-loads both). jmlr's
  /// `\floatconts` keeps its caption (jmlrutils.sty:166).
  #[test]
  fn class_stubs_require_their_float_packages() {
    let jmlr = r"\documentclass[pmlr]{jmlr}
\title{T}\author{\Name{A}}
\begin{document}
\begin{algorithm2e}
\caption{Computing Net Activation}
\end{algorithm2e}
\begin{table}\floatconts{tab:a}{\caption{Cap A}}{\begin{tabular}{l}x\end{tabular}}\end{table}
\end{document}
";
    let (stderr, xml) = convert(jmlr, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Computing Net Activation") && xml.contains("Cap A"),
      "{xml}"
    );
    let oup = r"\documentclass[unnumsec,webpdf,contemporary,large]{oup-authoring-template}
\begin{document}
\begin{sidewaystable}
\caption{X\label{t3}}
\begin{tabular}{ll}a&b\end{tabular}
\end{sidewaystable}
\end{document}
";
    let (stderr, xml) = convert(oup, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<caption>"), "{xml}");
  }

  /// A misplaced `\omit` (tex.web §1128) is one error and nothing else: no
  /// group is left open to swallow the next `}` (nicematrix manual's
  /// `\multicolumn` off-alignment ran to a `\Body` EoF runaway), and `&`
  /// keeps working afterwards.
  #[test]
  fn misplaced_omit_does_not_open_a_group() {
    let tex = r"\documentclass{article}
\begin{document}
A{\multicolumn{1}{c}{B}}C

\begin{tabular}{ll}x&y\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert!(
      xml.contains(">C<") || xml.contains("BC") || xml.contains("C</p>"),
      "{xml}"
    );
    assert_eq!(xml.matches("<td").count(), 2, "{xml}");
  }

  /// latex.ltx:16576: `\@array` lets `\tabularnewline` to `\\` for `array`
  /// too, so a column template that re-lets `\\` inside a box it opened
  /// (tabvar's varwidth cells) still ends the row (tabvar demo).
  #[test]
  fn math_array_lets_tabularnewline_to_the_row_break() {
    let tex = r"\documentclass{article}
\usepackage{array,varwidth}
\newcolumntype{C}{>{\begin{varwidth}{3cm}\let\\=\tabularnewline$}c<{$\end{varwidth}}}
\begin{document}
\[\begin{array}{cC}a&b\\ c&d\end{array}\]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<XMRow").count(), 2, "{xml}");
  }

  /// varwidth.sty:308-314 defines the `V{width}` column when array is
  /// loaded; without it the template loses a column (numerica).
  #[test]
  fn varwidth_v_column_is_defined() {
    let tex = r"\documentclass{article}
\usepackage{array,varwidth,booktabs}
\begin{document}
\begin{tabular}{lccV{\linewidth}l}\toprule
env & rem & eq & vv & sep\tabularnewline\midrule
a & b & c & d & e\tabularnewline\bottomrule
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
    assert_eq!(xml.matches("<td").count(), 10, "{xml}");
  }

  /// algpseudocodex ends a line's varwidth box only at the next `\State`;
  /// `\Statex` (= `\item[]`) sets its text inside the open box, so it is a
  /// break within the open line, not a nested `listingline` (manual,
  /// coloredtheorem; pdflatex clean).
  #[test]
  fn statex_continues_the_open_line_box() {
    for wrap in [
      ("", ""),
      (r"\begin{minipage}[t]{0.45\textwidth}", r"\end{minipage}"),
    ] {
      let tex = format!(
        r"\documentclass{{article}}
\usepackage{{algpseudocodex}}
\begin{{document}}
{}
\begin{{algorithmic}}[1]
\State first line
\Statex continuing line
\State second line
\end{{algorithmic}}
{}
\end{{document}}
",
        wrap.0, wrap.1
      );
      let (stderr, xml) = convert(&tex, true);
      assert_eq!(error_count(&stderr), 0, "{stderr}");
      assert_eq!(xml.matches("<listingline").count(), 2, "{xml}");
      assert!(xml.contains("<break"), "{xml}");
    }
  }

  /// nicematrix's `\CodeBefore`/`\Body` must carry unique meanings: `\let` to
  /// `\relax`, `\@ifnextchar\CodeBefore` (meaning comparison) matched any
  /// `\relax` at a matrix start and the `Until:\Body` grab ran to EoF
  /// (nicematrix manual's Fatal). A genuine `\CodeBefore` still grabs.
  #[test]
  fn nicematrix_relax_at_matrix_start_is_not_codebefore() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
$\begin{bNiceMatrix}\relax 9 & 17 \\ -2 & 5\end{bNiceMatrix}$
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert_eq!(xml.matches("<XMArray").count(), 1, "{xml}");
    let control = r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
$\begin{bNiceMatrix}\CodeBefore \rowcolor{blue!15}{1} \Body 9 & 17 \\ -2 & 5\end{bNiceMatrix}$
\end{document}
";
    let (stderr, xml) = convert(control, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("backgroundcolor="), "{xml}");
  }

  /// achemso.cls:144-165 `\bibnote` (notes2bib) files a note into the
  /// bibliography; rendered as a numbered in-place note, and mciteplus's
  /// `\mciteSubRef` (mciteplus.sty:780-782) is defined (achemso-demo).
  #[test]
  fn achemso_bibnote_is_a_numbered_note() {
    let tex = r"\documentclass{achemso}
\author{A}\title{T}
\begin{document}
Text\bibnote{This is a note.} and ref.~\mciteSubRef{Key2005}.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("role=\"bibnote\"") && xml.contains("This is a note."),
      "{xml}"
    );
  }

  /// A nested `\halign …\bgroup` (oz.sty's `op` schema inside `class`) must
  /// not decrement the outer alignment's align_state at its end: `\bgroup`
  /// never incremented it (tex.web §347), so the outer's `\crcr\noalign`
  /// stayed recognizable (ozguide).
  #[test]
  fn nested_halign_bgroup_keeps_the_outer_align_state() {
    let tex = r"\documentclass{article}
\usepackage{oz}
\begin{document}
\begin{class}{Point}
\begin{op}{Translate}
dx? : \real
\ST
x' = x + dx?
\end{op}
\end{class}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Translate"), "{xml}");
  }

  /// amsmath.sty:52 `\newif\ifctagsplit@` exists for documents that poke it
  /// (testmath.tex:1796); SHARED, Perl's binding lacks it too.
  #[test]
  fn amsmath_ctagsplit_switch_exists() {
    let tex = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
{\makeatletter\ctagsplit@true
\begin{equation}\begin{split} a&=b\\ &=c \end{split}\end{equation}}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<equation"), "{xml}");
  }

  /// latex.ltx:13531 `\DeclareMathDelimiter` takes six arguments and defines
  /// a control-sequence symbol through `\DeclareMathSymbol` (oz.sty:261
  /// corner delimiters from the AMSa symbol font; ozguide).
  #[test]
  fn declare_math_delimiter_defines_the_symbol() {
    let tex = r#"\documentclass{article}
\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}
\DeclareMathDelimiter\ulcorner{4}{AMSa}{"70}{AMSa}{"70}
\DeclareMathDelimiter\urcorner{5}{AMSa}{"71}{AMSa}{"71}
\begin{document}
$\ulcorner a\urcorner$
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("\u{231C}") && xml.contains("\u{231D}"),
      "{xml}"
    );
    assert!(xml.contains("role=\"OPEN\""), "{xml}");
  }

  /// Long-tail singletons: aastex701.cls:13637 `\digitalasset`, amsbook.cls:1779
  /// `\markleft`, t5enc's `\textdotbelow` (aastex701-sample,
  /// Author_Handbook_Memo, amsldoc-vi; Perl's bindings lack all three).
  #[test]
  fn long_tail_class_and_encoding_singletons() {
    for (tex, needle) in [
      (
        r"\documentclass{aastex701}\begin{document}\digitalasset Text.\end{document}",
        "Text.",
      ),
      (
        r"\documentclass{amsbook}\begin{document}\markleft{RUNNING}Body.\end{document}",
        "Body.",
      ),
      (
        r"\documentclass{article}\usepackage[T5]{fontenc}\begin{document}\textdotbelow{a}\end{document}",
        "\u{1EA1}",
      ),
    ] {
      let (stderr, xml) = convert(tex, false);
      assert_eq!(error_count(&stderr), 0, "{stderr}");
      assert!(xml.contains(needle), "{xml}");
    }
  }

  /// e-TeX `\ifincsname` is true inside `\csname…\endcsname`, so utf8.def's
  /// guard keeps `§` literal in a name (clefval `\TheValue{a§b}`; Perl's
  /// constant-false shortcut expanded it to `\textsection` and errored).
  #[test]
  fn ifincsname_keeps_utf8_chars_literal_in_names() {
    let tex = r"\documentclass{article}
\begin{document}
\expandafter\def\csname V@a§b\endcsname{VALUE}%
[\csname V@a§b\endcsname][\ifincsname yes\else no\fi][§]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[VALUE][no][§]"), "{xml}");
    let clef = r"\documentclass{article}
\usepackage{clefval}
\begin{document}
\TheKey{a§b}{value-here}
\TheValue{a§b}
\end{document}
";
    // Single pass: clefval resolves values through the .aux file, so the
    // lookup prints `?? a§b ??` (pdflatex's first run does the same); the
    // point is that the key survived the `\csname` intact and no error fired.
    let (stderr, xml) = convert(clef, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("should not appear between"), "{stderr}");
    assert!(xml.contains("a§b"), "{xml}");
  }

  /// Long-tail singletons, batch 2: article.cls:585 `\@openbib@code`,
  /// lettrine.sty:143 `\LettrineTextFont`, hyperref.sty:229
  /// `\AfterBeginDocument` (mciteplus_doc, ijsra, iodhbwm).
  #[test]
  fn long_tail_bib_lettrine_hyperref_singletons() {
    for (tex, needle) in [
      (
        r"\documentclass{article}\begin{document}\makeatletter\@openbib@code Text.\end{document}",
        "Text.",
      ),
      (
        r"\documentclass{article}\usepackage{lettrine}\renewcommand*{\LettrineTextFont}{\itshape}\begin{document}\lettrine{A}{bc} def.\end{document}",
        "def.",
      ),
      (
        r"\documentclass{article}\usepackage{hyperref}\AfterBeginDocument{\def\x{Hooked.}}\begin{document}\x\end{document}",
        "Hooked.",
      ),
    ] {
      let (stderr, xml) = convert(tex, false);
      assert_eq!(error_count(&stderr), 0, "{stderr}");
      assert!(xml.contains(needle), "{xml}");
    }
  }

  /// afterpackage.sty's patched `\@popfilename` reads `\@currname` as the
  /// package being finished; a NESTED load (ncc.cls → ncclatex → nccsect) must
  /// still fire the `\AfterPackage{nccsect}` hook that defines
  /// `\openrightorany` (nccdefaults.sty:41; ncclatex manual).
  #[test]
  fn afterpackage_hook_fires_for_a_nested_load() {
    let tex = r"\documentclass[11pt]{ncc}
\begin{document}
\makeatletter\ifx\openrightorany\@undefined UNDEF\else DEF\fi\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("DEF") && !xml.contains("UNDEF"), "{xml}");
  }

  /// `\inputminted` inside a minipage/tcolorbox must not close the box: the
  /// listing's trailer already balances its own group (sweep-37 regression
  /// from 54x: algxpar-doc, tikzducks-doc, biblatex-oxref, tcolorbox posters).
  #[test]
  fn inputminted_inside_a_minipage_keeps_its_box() {
    let tex = r"\documentclass{article}
\usepackage{minted}
\begin{document}
\begin{figure}
\begin{minipage}{5cm}
\inputminted{tex}{no-such-file-for-this-guard.tex}
Still inside.
\end{minipage}
\end{figure}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Still inside.") && xml.contains("After."),
      "{xml}"
    );
  }

  /// Long-tail singletons, batch 3: lineno.sty:1445 `\linelabel`, verbatim's
  /// `\verbatim@in@stream`, hyperref.sty:3331 `\@baseurl` default (lineno
  /// manual, ltug notes-for-authors, cms-dates-intro).
  #[test]
  fn long_tail_lineno_verbatim_baseurl_singletons() {
    for (tex, needle) in [
      (
        r"\documentclass{article}\usepackage{lineno}\begin{document}\linenumbers x\linelabel{a} y (\lineref{a})\end{document}",
        "y",
      ),
      (
        r"\documentclass{article}\usepackage{verbatim}\begin{document}\makeatletter\ifx\verbatim@in@stream\@undefined NO\else OK\fi\makeatother\end{document}",
        "OK",
      ),
      (
        r"\documentclass{article}\usepackage{hyperref}\begin{document}\makeatletter[\@baseurl]\makeatother Text.\end{document}",
        "[]",
      ),
    ] {
      let (stderr, xml) = convert(tex, false);
      assert_eq!(error_count(&stderr), 0, "{stderr}");
      assert!(xml.contains(needle), "{xml}");
    }
  }

  /// A `\lstnewenvironment` cell leaves align_state balanced, so the `&`
  /// after it is the column end (lexref's ltxdockit `ltxcode` cells; 54x
  /// regression: `{` opener with an `\lx@hidden@egroup` closer).
  #[test]
  fn listings_environment_cell_ends_at_the_tab() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\lstnewenvironment{ltxcode}{}{}
\begin{document}
\begin{tabular}{llll}
\begin{ltxcode}
a
\end{ltxcode} & \begin{ltxcode}
b
\end{ltxcode} & \begin{ltxcode}
c
\end{ltxcode} & \begin{ltxcode}
d
\end{ltxcode} \\
1 & 2 & 3 & 4 \\
\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
    assert_eq!(xml.matches("<td").count(), 8, "{xml}");
  }

  /// `\marginpar[{…[1][1-4]…}]{…}`: the braced optional is re-passed braced,
  /// so its own brackets are not read as the optional's end (Test-flexipage,
  /// a 55a regression; latex.ltx:17591 uses `{#1}`).
  #[test]
  fn marginpar_optional_keeps_its_own_brackets() {
    let tex = r"\documentclass{article}
\usepackage{lipsum}
\begin{document}
Text\marginpar[{\lipsum[1][1-1]}]{Y} more.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("role=\"margin\"") || xml.contains("margin"),
      "{xml}"
    );
    assert!(xml.contains("more."), "{xml}");
  }

  /// A Semiverbatim read inertizes active shorthands as `\url`'s
  /// `\dospecials` loop does: babel-czech's active `-` in a hyperref url no
  /// longer runs its word scanner inside the attribute (csbulletin; a 55c
  /// regression once `\ifinner` became correct).
  #[test]
  fn semiverbatim_inertizes_babel_shorthands() {
    let tex = r"\documentclass{csbulletin}
\usepackage{hyperref}
\begin{document}
\nolinkurl{a-b}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert!(xml.contains("a-b"), "{xml}");
    let tilde =
      r"\documentclass{article}\usepackage{hyperref}\begin{document}\nolinkurl{a~b}\end{document}";
    let (stderr, xml) = convert(tilde, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("a~b"), "{xml}");
  }

  /// minted's displays use the `\begingroup`/`\endgroup` listing group like
  /// lstlisting, so inside a `p{}` cell their closer meets no mode-switch
  /// frame (kernel-alignment locus probe: the only red construct).
  #[test]
  fn minted_in_a_p_column_keeps_the_cell() {
    for body in [
      "\\begin{minted}{tex}\nzz\n\\end{minted}",
      "\\inputminted{tex}{no-such-file-for-this-guard.tex}",
    ] {
      let tex = format!(
        "\\documentclass{{article}}\n\\usepackage{{minted}}\n\\begin{{document}}\n\\begin{{tabular}}{{p{{4cm}}l}}\n{body} & next \\\\\nrow2 & x \\\\\n\\end{{tabular}}\n\\end{{document}}\n"
      );
      let (stderr, xml) = convert(&tex, false);
      assert_eq!(error_count(&stderr), 0, "{stderr}");
      assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
      assert_eq!(xml.matches("<td").count(), 4, "{xml}");
    }
  }

  /// Long-tail singletons, batch 4: physics.sty:23 `\vnabla`,
  /// quantumarticle.cls:22 `\quantumarticleversion`, latex.ltx:18349
  /// `\@normalsize`, graphics.sty:156-158 `\Ginput@path` (physics manual,
  /// quantum-template, UNAMThesis, upmethodology; Perl lacks all four).
  #[test]
  fn long_tail_physics_quantum_normalsize_ginput_singletons() {
    for (tex, needle) in [
      (
        r"\documentclass{article}\usepackage{physics}\begin{document}$\vnabla f$\end{document}",
        "<Math",
      ),
      (
        r"\documentclass{quantumarticle}\begin{document}v\quantumarticleversion.\end{document}",
        "v6.",
      ),
      (
        r"\documentclass{report}\begin{document}\makeatletter\@normalsize\makeatother x\end{document}",
        "x",
      ),
      (
        r"\documentclass{article}\usepackage{graphicx}\graphicspath{{figs/}}\begin{document}\makeatletter[\Ginput@path]\makeatother\end{document}",
        "[figs/]",
      ),
    ] {
      let (stderr, xml) = convert(tex, false);
      assert_eq!(error_count(&stderr), 0, "{stderr}");
      assert!(xml.contains(needle), "{xml}");
    }
  }

  /// tex.web §485-486: `\read` consumes the whole physical line, so a header
  /// read under `\ExplSyntaxOn` (space = IGNORE) followed by an
  /// `\ior_map_inline` under `\ExplSyntaxOff` yields no spurious empty row
  /// (l3prefixes' `Until:,` runaway; Perl shares the empty row).
  #[test]
  fn read_consumes_the_physical_line_across_catcode_regimes() {
    let tex = r"\documentclass{article}
\usepackage{expl3}
\begin{filecontents}[overwrite,noheader,nosearch]{guard-twocol.csv}
h1,h2,h3,h4
r1,r2,r3,r4
s1,s2,s3,s4
\end{filecontents}
\ExplSyntaxOn
\cs_new_protected:Npn \__guard_row:w #1 , #2 , #3 , #4 \q_stop { [#1/#2/#3/#4] }
\ior_new:N \g_guard_ior
\ior_open:Nn \g_guard_ior { guard-twocol.csv }
\ior_get:NN \g_guard_ior \l_tmpa_tl
\cs_new_protected:Npn \GuardTable
  { \ior_map_inline:Nn \g_guard_ior { \__guard_row:w ##1 \q_stop } }
\ExplSyntaxOff
\begin{document}
\GuardTable
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert!(
      xml.contains("[r1/r2/r3/r4") && xml.contains("[s1/s2/s3/s4"),
      "{xml}"
    );
  }

  /// mdwtab.sty:765 `\hlx{vhv}` ends the row and rules it (talkdoc);
  /// german.sty:375 `\def@dqmacro` exists for germkorr's patch.
  #[test]
  fn mdwtab_hlx_ends_the_row_and_rules() {
    let tex = r"\documentclass{article}\usepackage{mdwtab}
\begin{document}\begin{tabular}{cc}a&b\hlx{vhv}c&d\end{tabular}\end{document}";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
    let de =
      r"\documentclass{article}\usepackage{german,germkorr}\begin{document}Text.\end{document}";
    let (stderr, xml) = convert(de, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Text."), "{xml}");
  }

  /// jmlr.cls:246-247 `\titlebreak`/`\titletag` and the raw jmlrutils.sty
  /// surface (`\subfigure`, `\subfigref`, `\includeteximage`) reach the
  /// jmlr binding (pmlr-sample).
  #[test]
  fn jmlr_has_the_jmlrutils_surface() {
    let tex = r"\documentclass[pmlr]{jmlr}
\title[Short]{A Long\titlebreak Title \titletag{x}}\author{\Name{A}}
\begin{document}
\begin{figure}\floatconts{fig:a}{\caption{Two}}{\subfigure[one]{\rule{1cm}{1cm}}\subfigure[two]{\rule{1cm}{1cm}}}\end{figure}
See \subfigref{fig:a}{a}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<title") && xml.contains("<figure"), "{xml}");
  }

  /// microtype.sty:36 `\MT@MT` marks the package for typog.sty:68's
  /// `\ifdefined\MT@MT` (typog-example under `trackingttspacing`).
  #[test]
  fn microtype_marker_satisfies_typog() {
    let tex = r"\documentclass{article}\usepackage[activate=true]{microtype}\usepackage[trackingttspacing]{typog}
\begin{document}Text.\end{document}";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Text."), "{xml}");
  }

  /// An autoload stub must not satisfy a class-detection probe:
  /// projlib-author.sty:38 `\cs_if_exist:NT \subjclass {\endinput}` (homework).
  #[test]
  fn autoload_stubs_do_not_satisfy_class_probes() {
    let tex = r"\documentclass{article}\usepackage{expl3}\begin{document}
\ExplSyntaxOn[\cs_if_exist:NTF\subjclass{AMS}{NOAMS}]\ExplSyntaxOff
\end{document}";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[NOAMS]"), "{xml}");
    let ams = r"\documentclass{amsart}\begin{document}\subjclass{03B05}Text.\end{document}";
    let (stderr, _xml) = convert(ams, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// titlesec.sty:420 sets `\thetitle` per typeset title, so mla.cls:196's
  /// `\titleformat{\section}{}{\thetitle.\enspace}…` label expands.
  #[test]
  fn titlesec_thetitle_in_format_label() {
    let tex = r"\documentclass{article}\usepackage{titlesec}
\titleformat{\section}{}{\thetitle.\enspace}{0pt}{}
\begin{document}\section{Intro}Body.\end{document}";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Intro"), "{xml}");
  }

  /// latex.ltx:19172-19280 release rollback: `\RequirePackage{doc}[=v2]` loads
  /// doc-2021-06-01.sty, so dox.sty's v2-era `\let\SpecialMacroIndex
  /// \SpecialUsageIndex` is not a self-loop (testidx-manual and every
  /// nlctdoc manual; Perl hangs identically).
  #[test]
  fn package_release_rollback_loads_the_named_release() {
    let tex = r"\documentclass{article}
\RequirePackage{doc}[=v2]
\usepackage{dox}
\begin{document}
\makeatletter[\csname ver@doc.sty\endcsname]\makeatother
\DescribeMacro\foo Text.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("2021") && xml.contains("Text."), "{xml}");
  }

  /// article.cls's `\maketitle` disables `\title`/`\maketitle` after use; a
  /// class that `\renewcommand`s `\maketitle` without that cleanup
  /// (schooldocs.sty:136, `\correct` :168-178 chaining `\@title`) had the
  /// redefinition dropped by the lock and the kernel cleanup made later
  /// `\title`s no-ops, so the second `\correct` built a self-referential
  /// `\@originaltitle` (`PushbackLimit`; schooldocs-examples). The
  /// self-disabling half now yields when the class took `\maketitle` over.
  #[test]
  fn maketitle_cleanup_yields_to_a_class_redefinition() {
    let tex = r"\documentclass{article}
\usepackage{schooldocs}
\begin{document}
\schooldocstitles
\title{Standard}
\maketitle
\correct
\title{Exam}
\maketitle
\correct
\title{Small}
\schooldocstitles
\makesmalltitle
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Small"), "{xml}");
    // Without a class redefinition the standard cleanup still applies.
    let tex = r"\documentclass{article}
\title{T}\author{A}
\begin{document}
\maketitle
\title{Again}\makeatletter[\@title]\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[]"), "{xml}");
  }

  /// Conditionals inside dimension/glue arguments (e.g. \hspace, \raisebox)
  /// must cleanly expand remaining tokens (\else, \fi) when reparsed in a
  /// temporary mouth so they do not leak unclosed if-frames into enclosing
  /// macros like \parbox.
  /// Witness: typog-example / parbox_dimen_conditional_double.tex
  #[test]
  fn dimension_conditional_in_parbox_does_not_leak_or_duplicate() {
    let tex = r"\documentclass{article}
\makeatletter
\newlength{\Lreg}\newlength{\U}\setlength{\U}{.001em}\def\a{0}\def\b{*}
\makeatother
\begin{document}
\parbox[t]{0pt}{s\hspace{\ifx\a\b\Lreg\else\a\U\fi}e}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(
      xml.matches("class=\"ltx_parbox\"").count(),
      1,
      "Expected exactly one ltx_parbox: {xml}"
    );
    assert!(xml.contains("se"), "{xml}");

    // True branch case (\else ... \fi tail in mouth)
    let tex_true = r"\documentclass{article}
\begin{document}
\parbox[t]{0pt}{s\hspace{\iftrue 10pt\else 20pt\fi}e}
\end{document}
";
    let (stderr_true, xml_true) = convert(tex_true, false);
    assert_eq!(error_count(&stderr_true), 0, "{stderr_true}");
    assert_eq!(
      xml_true.matches("class=\"ltx_parbox\"").count(),
      1,
      "Expected exactly one ltx_parbox: {xml_true}"
    );

    // typog.sty \raisebox shape
    let tex_raisebox = r"\documentclass{article}
\begin{document}
\raisebox{\iftrue 5pt\else 10pt\fi}{test}
\end{document}
";
    let (stderr_raise, xml_raise) = convert(tex_raisebox, false);
    assert_eq!(error_count(&stderr_raise), 0, "{stderr_raise}");
    assert!(xml_raise.contains("test"), "{xml_raise}");
  }

  /// nicematrix environments with `name=...` or `create-cell-nodes` materialize
  /// coordinate nodes for PGF/TikZ overlays (`ma-matrice-2-2`), avoiding
  /// `Package pgf Error: No shape named '...' is known` (witness nicematrix-french:5986).
  #[test]
  fn nicematrix_cell_nodes_materialized_for_pgf_overlay() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\begin{document}
$\begin{pNiceMatrix}[name=ma-matrice]
1 & 2 & 3 \\ 4 & 5 & 6 \\ 7 & 8 & 9
\end{pNiceMatrix}$
\tikz[remember picture,overlay] \draw (ma-matrice-2-2) circle (2mm) ;
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<svg:path"),
      "Expected overlay svg path: {xml}"
    );
    assert!(
      xml.contains("matrix@(Array"),
      "Expected pNiceMatrix math: {xml}"
    );
  }

  /// P52: nicematrix + shortvrb verbatim footnotes. Under \VerbatimFootnotes,
  /// \footnote captures its body live so active verbatim tokens (e.g. `|` from
  /// shortvrb) and inner unescaped braces digest cleanly into <note>
  /// without triggering misplaced \omit or premature argument termination.
  #[test]
  fn nicematrix_shortvrb_verbatim_footnotes() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix,shortvrb,fancyvrb}
\MakeShortVerb{\|}
\VerbatimFootnotes
\begin{document}
Plain: |\multicolumn|.
X\footnote{Footnote with |\multicolumn| and a brace |}| here.}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"<note mark="1" role="footnote""#), "{xml}");
    assert!(xml.contains(r">\multicolumn<"), "{xml}");
  }

  /// P53: nicematrix AutoNiceMatrix, tabularnote, braces, and CodeBefore overlays.
  #[test]
  fn nicematrix_autonicematrix_delimiters_and_overlays() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\begin{document}
\[ C = \pAutoNiceMatrix{2-2}{C_{\arabic{iRow},\arabic{jCol}}} \]
\begin{NiceTabular}{cc}
A\tabularnote{A note} & B \\
\end{NiceTabular}
$\begin{NiceArray}{cc}[first-col]
\Hbrace{2}{top} \\
\Vbrace{2}{left} & 1 & 2 \\
& \Hspace{5mm} & \Vdotsfor{1}
\end{NiceArray}$
\[\begin{NiceArray}{cc}
\CodeBefore [create-cell-nodes]
  \chessboardcolors{red!15}{blue!15}
  \SubMatrix({1-1}{2-2})
  \tikz \draw (1-1) -- (2-2) ;
\Body
1 & 2 \\
3 & 4
\end{NiceArray}\]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("C_{1,1}"), "{xml}");
    assert!(xml.contains("C_{2,2}"), "{xml}");
    assert!(xml.contains(r#"role="footnote""#), "{xml}");
    assert!(xml.contains("A note"), "{xml}");
    assert!(xml.contains("top"), "{xml}");
    assert!(xml.contains("left"), "{xml}");
  }
}

mod perfect_kernel_batch55 {
  //! Red/green guards for perfect-kernel batch 55 (wave-5 root-causer
  //! reports over the CJK and LuaTeX oracle-clean doc residuals: circledtext,
  //! suanpan-l3, pascaltriangle, tikz-bagua, joinbox, jnuexam).
  use super::perfect_kernel_batch46::{convert, error_count};

  /// XeTeX/LuaTeX \Uchar <number> expands to a character token (Catcode 10 for space, 12 otherwise).
  /// expl3 binds \tex_Uchar:D to \Uchar.
  #[test]
  fn uchar_primitive_and_expl3_alias() {
    let tex = r"\documentclass{article}
\usepackage{expl3}
\begin{document}
[\Uchar 65][\Uchar 32][\ExplSyntaxOn\tex_Uchar:D 66\ExplSyntaxOff]
\end{document}
";
    // `\Uchar` is a Unicode-engine primitive: only the luatex profile has it.
    let (stderr, xml) =
      super::perfect_kernel_batch46::convert_with(tex, Some("[luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[A][ ][B]"), "{xml}");
  }

  /// xunicode.sty exports \UTFencname and macro declarations used by xunicode-addon.sty.
  #[test]
  fn xunicode_interface_for_addon() {
    let tex = r"\documentclass{article}
\usepackage{xunicode}
\DeclareUTFcharacter[\UTFencname]{x00A0}{\nobreakspace}
\ReloadXunicode
\begin{document}
UTF:\UTFencname
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("UTF:TU"), "{xml}");
  }

  /// \disable@package@load{pkg}{action} stores action in \@pkg-disable@pkg.ext and suppresses package load.
  #[test]
  fn kernel_disable_package_load() {
    let tex = r"\documentclass{article}
\makeatletter
\disable@package@load{nonexistentpkg}{\def\customdisabled{DISABLED}}
\makeatother
\usepackage{nonexistentpkg}
\begin{document}
[\customdisabled]
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[DISABLED]"), "{xml}");
  }

  /// fontspec family setters and ctex-engine-luatex lua commands
  #[test]
  fn fontspec_family_setters_and_ctex_luatex() {
    let tex = r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\tl_new:N \l_test_family_tl
\fontspec_gset_family:Nnn \l_test_family_tl {} {SimSun}
\fontspec_set_family:Nnn \l_test_family_tl {} {SimHei}
\ctex_ltj_add_kyenc:n { EU1 }
\ctex_ltj_zero_globaldefs:
\ExplSyntaxOff
\begin{document}
CJK fontspec ok
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("CJK fontspec ok"), "{xml}");
  }
}

mod perfect_kernel_batch56 {
  //! Red/green guards for perfect-kernel batch 56 (codebox manual:
  //! \SetCatcodeRange / \setcatcoderange / \@setrangecatcode, \lstloadaspects,
  //! \DeclareTCBListing nested inside \NewDocumentEnvironment with bare
  //! environment invocation and outer listing scanning, and unicode-math table loading).
  use super::perfect_kernel_batch46::{
    convert, convert_args, convert_files, convert_with, error_count,
  };

  /// Perl `State.pm:113-115` letters only ASCII and pdfTeX never letters a
  /// non-ASCII char (utf8.def makes the bytes active), so under the default
  /// profile `\xα` is `\x` followed by α, not one control sequence.
  #[test]
  fn non_ascii_letters_stay_other_under_pdftex() {
    let tex = "\\documentclass{article}\n\\def\\x{OK}\n\\begin{document}\n\\xα and \\x中.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("OKα and OK中."), "{xml}");
  }

  /// load-unicode-data.tex:134-135: the LuaTeX format letters every L/M code
  /// point, Latin-1 included (the dump pins U+0080-U+00FF OTHER, the profile
  /// re-letters them). Witnesses: circledtext, jnuexam, tikz-bagua.
  #[test]
  fn non_ascii_letters_are_letters_under_luatex() {
    let tex = "\\documentclass{article}\n\\begin{document}\n\\typeout{CC:\\the\\catcode`é:\\the\\catcode`α:\\the\\catcode`中:\\the\\catcode`Ⅳ}\n\\ifcat A中 ZH-LETTER\\else ZH-OTHER\\fi\n\\end{document}\n";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("CC:11:11:11:12"), "{stderr}");
    assert!(xml.contains("ZH-LETTER"), "{xml}");
  }

  /// A `\<type>name` that takes arguments (argumentation.sty:403 `\afname{…}`
  /// draws a tikz node) is not the counter's name noun: `\refstepcounter{af}`
  /// must format the tag from `\theaf` alone. KPE #194 (SHARED Perl failure;
  /// pdflatex clean).
  #[test]
  fn counter_name_command_is_not_a_name_noun() {
    let tex = r"\documentclass{article}
\newcounter{af}
\NewDocumentCommand{\afname}{m}{\node[caption](x){#1};}
\newcounter{gadget}
\newcommand{\gadgetname}{Gadget}
\newcounter{zero}
\NewDocumentCommand{\zeroname}{}{Zero}
\makeatletter
\begin{document}
\refstepcounter{af}\label{a}\refstepcounter{gadget}\label{g}\refstepcounter{zero}\label{z}
See \ref{a}, \ref{g} and \ref{z}.
\typeout{NOUN:\iflx@namenoun\gadgetname Y\else N\fi:\iflx@namenoun\afname Y\else N\fi:\iflx@namenoun\zeroname Y\else N\fi:\iflx@namenoun\figurename Y\else N\fi}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("ERROR"), "{xml}");
    // plain macro noun / m-taking xparse command / zero-arg xparse noun / kernel noun
    assert!(stderr.contains("NOUN:Y:N:Y:Y"), "{stderr}");
  }

  /// A `\lstnewenvironment` listing used as `\begin{name}` terminates only at its
  /// own `\end{name}`: a literal `\end{document}` in the body is verbatim content
  /// (listings.sty:2211-2215 compares against `\@currenvir` = name). The bare
  /// `\name` form (tcolorbox inside a wrapper environment) keeps terminating at
  /// the enclosing environment's `\end`.
  #[test]
  fn lstnewenvironment_begin_form_keeps_literal_end_document() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\lstnewenvironment{mycode}{}{}
\begin{document}
\begin{mycode}
line one of code
\end{document}
line three of code
\end{mycode}
Tail text survives.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Tail text survives."), "{xml}");
    // base64 of "line one of code\n\end{document}\nline three of code"
    assert!(
      xml.contains("bGluZSBvbmUgb2YgY29kZQpcZW5ke2RvY3VtZW50fQpsaW5lIHRocmVlIG9mIGNvZGU="),
      "{xml}"
    );
  }

  /// pict2e.sty:742-774 path interface (dvips mode under `\pdfoutput=0`):
  /// witnesses fancyqr-doc, curve2e-manual (RUST-ONLY: Perl raw-loads pict2e).
  #[test]
  fn pict2e_path_interface_strokes_a_polyline() {
    let tex = r"\documentclass{article}
\usepackage{pict2e}
\begin{document}
\setlength{\unitlength}{1mm}
\begin{picture}(40,40)
\moveto(0,0)
\lineto(40,0)
\lineto(40,40)
\closepath
\strokepath
\moveto(5,5)\curveto(10,20)(30,20)(35,5)\strokepath
\circlearc{20}{20}{10}{0}{90}\fillpath
\end{picture}
\newdimen\SIXR \SIXR=50pt
\begin{picture}(100,40)\moveto(\SIXR,20)\lineto(0,0)\strokepath\end{picture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<picture").count(), 2, "{xml}");
    // triangle + sampled curve + arc + the register-coordinate segment
    assert_eq!(xml.matches("<line ").count(), 4, "{xml}");
    // FramedSyntax.sty:189 shape: a dimen REGISTER coordinate is not 0
    assert!(!xml.contains("points=\"0,0 0,0\""), "{xml}");
  }

  /// hyperref.sty:3298-3311/3973-3979 storage macros and the :4092-4093
  /// driver link pair (witnesses movie15 overlay-example, hrefhide-example,
  /// ucalgmthesis sample-thesis; SHARED with Perl, pdflatex clean).
  #[test]
  fn hyperref_storage_and_driver_link_internals() {
    let tex = r"\documentclass{article}
\usepackage{hyperref}
\makeatletter
\begin{document}
\edef\z{/C [\@urlbordercolor] /H \@pdfhighlight (\@citebordercolor)(\@anchorcolor)}\typeout{Z:\z}
Text \hyper@linkstart{link}{target}anchor\hyper@linkend\ end.
\hyper@natlinkstart{k}cite\hyper@natlinkend.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // the space after `\@pdfhighlight` is a control-word space, eaten by the tokenizer
    assert!(
      stderr.contains("Z:/C [0 1 1] /H /I(0 1 0)(black)"),
      "{stderr}"
    );
    assert!(xml.contains("Text anchor end."), "{xml}");
    assert!(xml.contains("cite."), "{xml}");
  }

  /// showexpl.sty:58-61,115 switches: an undefined `\if@SX@…` inside a
  /// skipped branch desyncs the skip (tex.web §510). Witness pst-exa-doc
  /// (`\usepackage[tcb]{pst-exa}`), SHARED with Perl, pdflatex clean.
  #[test]
  fn showexpl_switches_balance_a_skipped_branch() {
    let tex = r"\documentclass{article}
\usepackage{showexpl}
\makeatletter
\newif\ifsw
\swfalse
\begin{document}
\ifsw
  \renewcommand*\Foo{%
    \ifx\a\@empty
      \if@SX@rangeaccept X\else Y\fi
    \else
      \begin{center}c\end{center}%
    \fi
  }%
\else
TCB-OK
\fi
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("TCB-OK"), "{xml}");
    assert!(!xml.contains(">c<"), "{xml}");
  }

  /// `\Umathcodenum` is an internal integer: `\the\Umathcodenum"2F` reads a
  /// number (fixdif.sty:38; physics2, physics2-legacy).
  #[test]
  fn umathcodenum_is_an_internal_integer() {
    let tex = r#"\documentclass{article}
\begin{document}
\count0=\numexpr(\the\Umathcodenum"2F-"2F)/16777216\relax X\typeout{UM:\the\count0}
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("UM:0"), "{stderr}");
    assert!(xml.contains("X"), "{xml}");
  }

  /// caesar_book.cls:106-115 counts title lines with a `\lastbox` loop;
  /// `\unpenalty` must not push a box per iteration (sidenotes caesar_example,
  /// an unbounded runaway in Perl too; pdflatex terminates).
  #[test]
  fn unpenalty_does_not_grow_the_box_list() {
    let tex = r"\documentclass{article}
\makeatletter
\begin{document}
\setbox0\vbox{A title line here\par
  \count@\z@
  \loop
  \unskip\unpenalty\unskip\unpenalty\unskip
  \setbox0\lastbox
  \ifvoid0 \xdef\numlines{\the\count@}\else \advance\count@\@ne \repeat}%
numlines=\numlines
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("numlines="), "{xml}");
  }

  /// latex.ltx:17570-17591 allocates `\@marbox` before dispatching to
  /// `\@ympar`; classes redefine `\@ympar` with the kernel idiom
  /// (caesar_book.cls:84-87), so `\marginpar` must dispatch to private targets
  /// (witness sidenotes caesar_example, 42 errors; RUST-ONLY, Perl and pdflatex clean).
  #[test]
  fn marginpar_ignores_a_class_redefined_ympar() {
    let tex = r"\documentclass{article}
\usepackage{graphicx}
\usepackage{sidenotes}
\makeatletter
\newcommand{\marginparstyle}{\footnotesize}
\long\def\@ympar#1{%
  \@savemarbox\@marbox{\marginparstyle#1}%
  \global\setbox\@currbox\copy\@marbox
  \@xympar}
\makeatother
\begin{document}
Text.
\begin{marginfigure}
  \includegraphics[width=\marginparwidth]{example-image-a}
  \caption{A margin figure.\label{f}}
\end{marginfigure}
More text~\ref{f}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("role=\"margin\""), "{xml}");
    assert!(xml.contains("<figure") && xml.contains("<caption"), "{xml}");
  }

  /// A `\lstnewenvironment` whose body is diverted to a file
  /// (`\lst@BeginWriteFile`) still runs its end code (pst-exa.sty:163-170
  /// closes a start-code `\hbox` there and reads the result back; pst-exa-doc,
  /// RUST-ONLY).
  #[test]
  fn lstnewenvironment_writefile_runs_the_end_code() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\makeatletter
\def\SX@put@code@result{RESULTMARK}
\lstnewenvironment{myex}[1][]
 {\setbox\@tempboxa=\hbox\bgroup\lst@BeginWriteFile{\jobname.swpl}}
 {\lst@EndWriteFile\egroup\SX@put@code@result}
\makeatother
\begin{document}
\begin{myex}[pos=t]
hello world
\end{myex}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("RESULTMARK"), "{xml}");
  }

  /// latex.ltx:9247-9266 cr chain entered directly by brief.cls:496
  /// `\@nobreakcr` (ntgclass brief-sample, RUST-ONLY: the raw `\@gnewline`
  /// body dereferences `\reserved@f` when a parbox body is re-expanded).
  #[test]
  fn raw_newline_chain_is_the_native_newline() {
    let tex = r"\documentclass{brief}
\name{WG}
\begin{document}
\begin{brief}{Jan}\end{brief}
\begin{brief}{Jan}\opening{Hallo,} \ondertekening{Victor}\afsluiting{doei}\end{brief}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Victor") && xml.contains("doei"), "{xml}");
  }

  /// Perl locks both `\tabular` and `\endtabular`; a class redefining both
  /// (jpsj2.cls:652,657) must have both dropped (witness jpsj injpsj2, RUST-ONLY).
  #[test]
  fn tabular_delegator_is_locked_with_endtabular() {
    let tex = r"\documentclass{article}
\makeatletter
\def\tabular{\begin{center}\let\@halignto\@empty\@tabular}
\def\endtabular{\crcr\egroup\egroup $\egroup\end{center}}
\makeatother
\begin{document}
\begin{center}
\begin{tabular}{cc} a & b \\ c & d \end{tabular}
\end{center}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<tabular"), "{xml}");
    for cell in ["a", "b", "c", "d"] {
      assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
    }
  }

  /// `\ifdefined\Uchar` (and `\primitive`) are Unicode-engine detection
  /// probes (ucharcat.sty, math-operator.sty); pdfTeX has neither, so the
  /// default profile must leave them undefined (sweep-38 regression).
  #[test]
  fn unicode_engine_primitives_stay_undefined_under_pdftex() {
    let tex = r#"\documentclass{article}
\begin{document}
\ifdefined\Uchar \Umathchar"0"0"0 \fi
\ifdefined\primitive \Umathchar"0"0"0 \fi
Done.
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Done."), "{xml}");
  }

  /// latex.ltx:16560 `\@tabular` is parameterless with a literal `$` that
  /// luababel.def:1895 `\bbl@replace\@tabular{$}{…}` rescans (lettrine-demo-arabic
  /// and every babel `bidi=basic` document; sweep-38 TokenLimit regression).
  #[test]
  fn at_tabular_is_parameterless_with_a_patchable_math_shift() {
    let tex = r"\documentclass{article}
\makeatletter
\long\def\bbl@afterfi#1\fi{\fi#1}
\def\bbl@replace#1#2#3{%
  \toks@{}%
  \def\bbl@replace@aux##1#2##2#2{%
    \ifx\bbl@nil##2\toks@\expandafter{\the\toks@##1}%
    \else\toks@\expandafter{\the\toks@##1#3}\bbl@afterfi\bbl@replace@aux##2#2\fi}%
  \expandafter\bbl@replace@aux#1#2\bbl@nil#2%
  \edef#1{\the\toks@}}
\def\PATCHED{}
\bbl@replace\@tabular{$}{$\def\PATCHED{patched}}%
\makeatother
\begin{document}
\begin{tabular}{cc} \PATCHED & b \\ c & d \end{tabular}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<tabular"), "{xml}");
    // `\@tabular` is locked, so the rescanned `\edef` is dropped (the patch only
    // sets bidi layout state); what matters is that the `$` scan balanced and
    // the alignment still works.
    for cell in ["b", "c", "d"] {
      assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
    }
  }

  /// curve2e.sty raw-loads over pict2e's driver-level path builders: vector
  /// algebra, `\Arc`, `\VectorARC`, `\Zbox`/`\Pbox`, `\xmultiput`, `\AutoGrid`
  /// (witness curve2e-manual, 32 undefined-command errors with the old stub).
  #[test]
  fn curve2e_raw_load_renders_arcs_and_vectors() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{curve2e}
\begin{document}
\setlength{\unitlength}{1mm}
\CopyVect 3,4 to\V \ModOfVect\V to\M Mod=\M.
\begin{picture}(40,40)
\Arc(20,20)(30,20){90}
\VectorARC(20,20)(30,20){60}
\Zbox(40,0)[l]{40,0}[1]
\Pbox(0,0)[r]{C}[0.75ex]
\xmultiput(0,0)(8,0){5}{\circle*{1}}
\AutoGrid
\end{picture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Mod=5"), "{xml}");
    // the arc, the vector arc and the grid all render as lines
    assert!(xml.matches("<line ").count() >= 3, "{xml}");
    assert!(xml.matches("<circle").count() >= 5, "{xml}");
  }

  /// booktabs.sty:53-118 rule machinery for documents that copy the real
  /// `\midrule` (l2kurz.tex:58-65): `\@BTendrule` closes the `\noalign{`
  /// that `\ifnum0=`}\fi` opened (witness lshort-german l2kurz, 41 errors).
  #[test]
  fn booktabs_rule_machinery_closes_its_noalign() {
    let tex = r"\documentclass{article}
\usepackage{array,longtable,tabularx,booktabs}
\makeatletter
\def\midrule{\noalign{\ifnum0=`}\fi\penalty\@M
  \@aboverulesep=\aboverulesep \global\@belowrulesep=\belowrulesep
  \global\@thisruleclass=\@ne
  \@ifnextchar[{\@BTrule}{\@BTrule[\lightrulewidth]}}
\makeatother
\begin{document}
\begin{tabular}[t]{rl}
\toprule A & B \\ \midrule 1 & 2 \\ \bottomrule
\end{tabular}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<tabular"), "{xml}");
    for cell in ["A", "B", "1", "2"] {
      assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
    }
    assert!(xml.contains("After."), "{xml}");
  }

  /// magyar.ldf:1882-1898 calls the removed caption3 internal
  /// `\caption@setdefaultlabelsep` only when `\caption@lsep@default` is
  /// undefined (witnesses elteikthesis ×3, elteiktdk ×2; RUST-ONLY).
  #[test]
  fn caption_lsep_default_keeps_magyar_off_the_removed_internal() {
    let tex = r"\documentclass{article}
\usepackage[hungarian]{babel}
\usepackage{caption}
\begin{document}
\begin{figure}
\centering Test
\caption{Teszt \'abra}
\end{figure}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<caption"), "{xml}");
    assert!(!xml.contains("ERROR"), "{xml}");
  }

  /// tcolorbox's `\dispExample` runs its body via `\tcbusetemp` = `\input`
  /// (tcolorbox.sty:2820), so a mid-body `\ExplSyntaxOn` applies to what
  /// follows (witness csvsimple-l3; RUST-ONLY: the body was eagerly tokenized).
  #[test]
  fn dispexample_body_runs_with_live_catcodes() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{documentation}
\begin{document}
\begin{dispExample}
\ExplSyntaxOn
\tl_new:N \l_test_tl
\tl_set:Nn \l_test_tl {LI\csname VE\endcsname}
\tl_use:N \l_test_tl \gdef\EXECUTED{yes}
\ExplSyntaxOff
\end{dispExample}
\ifdefined\EXECUTED RAN-\else NOT-\fi
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("tl_new"), "{xml}");
    // the executed body defines a macro the listing display cannot
    assert!(xml.contains("RAN-"), "{xml}");
  }

  /// `\tikzexternalize` without shell escape: tikz's `mode=graphics if exists`
  /// typesets the picture inline with no system-call error (witnesses
  /// tikzviolinplots 591, causets 106, tilings 80, tikz-feynhand 55; SHARED).
  #[test]
  fn tikz_externalize_typesets_inline_without_a_system_call() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{external}
\tikzexternalize[prefix=ext/]
\begin{document}
\begin{tikzpicture}
\draw (0,0) circle (1);
\end{tikzpicture}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<picture") || xml.contains("<svg"), "{xml}");
  }

  /// codehigh's non-LuaTeX parser is O(n²) on the l3regex VM; a whole package
  /// source through `\dochighinput` must still finish (fontscale-code and 6
  /// more manuals timed out; SHARED with Perl, pdflatex fast).
  #[test]
  fn codehigh_dochighinput_is_bounded() {
    let tex = r"\documentclass{article}
\usepackage{codehigh}
\begin{document}
\dochighinput[language=latex/latex3]{fontscale.sty}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("ProvidesExplPackage") || xml.contains("fontscale"),
      "{xml}"
    );
  }

  /// `DefToken` skips blanks before the token being defined (tex.web §1215):
  /// `\lstMakeShortInline [opts] {"}` must activate `"`, not the space
  /// (install-latex-guide-zh-cn:111 → a `\maketitle` recursion Fatal; SHARED).
  #[test]
  fn deftoken_skips_a_leading_space() {
    let tex = r#"\documentclass{article}
\usepackage{listings}
\lstMakeShortInline [ x = 1 ] {"}
\newcommand {\foo} {FOO}
\begin{document}
SP[\the\catcode`\ ]DQ[\the\catcode`\"] \foo
\end{document}
"#;
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("SP[10]DQ[13]"), "{xml}");
    assert!(xml.contains("FOO"), "{xml}");
  }

  /// dhucs.sty:44 `\ifx 가가` takes the native-Unicode branch here, whose
  /// `\dhucs@hu` lives behind LuaTeX/XeTeX probes; the engine-neutral subset
  /// is supplied after the raw load (kotex-oblivoir manuals; SHARED, pdflatex clean).
  #[test]
  fn dhucs_native_branch_defines_the_hangul_skip() {
    let tex = r"\documentclass{article}
\makeatletter
\RequirePackage{dhucs}
\newdimen\x@hu \x@hu=\dhucs@hu
\setInterHangulSkip{1pt}
\makeatother
\begin{document}
ok
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("ok"), "{xml}");
  }

  /// tcolorbox listings default to `listing and text`: the body is displayed
  /// AND executed (tcblistingscore.code.tex:429/:205); `listing only` is not
  /// (witnesses postit-doc-en/fr, 16 "No shape named" errors; RUST-ONLY).
  #[test]
  fn tcblisting_listing_and_text_executes_the_body() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newtcblisting{DemoCode}[1][]{listing options={commentstyle={\itshape}},#1}
\begin{document}
\begin{DemoCode}[]
\begin{tikzpicture}[remember picture]
  \coordinate (foo-N-W) at (0,0);
\end{tikzpicture}
\end{DemoCode}
\begin{tikzpicture}[remember picture,overlay]
  \draw (foo-N-W) circle[radius=2pt];
\end{tikzpicture}
\begin{DemoCode}[listing only]
\def\ONLYDISPLAYED{ran}
\end{DemoCode}
\ifdefined\ONLYDISPLAYED RAN\else NOTRUN\fi
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.matches("<svg").count() >= 2 || xml.matches("<picture").count() >= 2,
      "{xml}"
    );
    assert!(xml.contains("NOTRUN"), "{xml}");
  }

  /// beamerbasefont.sty:322-323 `\Tiny`/`\TINY` (font themes use them) and
  /// caption3.sty:701 `\DeclareCaptionFormat*{name}{code}` consumed whole
  /// (nostarch.cls:856 left `#1#2#3` in the stream). Both SHARED with Perl.
  #[test]
  fn beamer_tiny_sizes_and_starred_caption_format() {
    let tex = r"\documentclass{beamer}
\usepackage{caption}
\DeclareCaptionFormat*{myfmt}{\parbox{5cm}{#1#2#3}}
\DeclareCaptionFormat{plain2}[short]{#1#2#3\par}
\begin{document}
\begin{frame}
{\Tiny tiny text} {\TINY tinier text} Hello world.
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("tiny text") && xml.contains("tinier text") && xml.contains("Hello world."),
      "{xml}"
    );
    assert!(!xml.contains("#1"), "{xml}");
  }

  /// latex.ltx:15515-15521 `\@noligs` neutralises an active `<` (l3doc's
  /// `function` shorthand) inside fancyvrb verbatim (witnesses interface3,
  /// source3, source2e; SHARED with Perl, pdflatex clean).
  #[test]
  fn noligs_neutralises_active_chars_in_verbatim() {
    let tex = r"\documentclass{article}
\usepackage{fancyvrb}
\makeatletter
\catcode`\<=\active
\def<#1>{\textit{#1}}
\makeatother
\begin{document}
Meta <arg> outside.
\begin{Verbatim}
\dim_compare_p:n { #1 <= #2 }
next {line}
\end{Verbatim}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("dim_compare_p:n") && xml.contains("&lt;= #2") || xml.contains("<= #2"),
      "{xml}"
    );
    assert!(xml.contains("After."), "{xml}");
  }

  /// `\errmessage` counts as an error (tex.web §1283), so an expl3
  /// `\msg_error` loop is cut by the consecutive-error breaker instead of
  /// running to the token limit (csvsimple-l3 `sort by=` with no sorter).
  #[test]
  fn errmessage_counts_toward_the_error_breaker() {
    let tex = r"\documentclass{article}
\usepackage{csvsimple-l3}
\begin{filecontents*}[overwrite]{grade.csv}
name,givenname
Maier,Hans
\end{filecontents*}
\begin{filecontents*}[overwrite]{namesort.xml}
<sortconfig/>
\end{filecontents*}
\ExplSyntaxOn \tl_gclear_new:N \csvline \ExplSyntaxOff
\begin{document}
Before.
\csvreader[sort by=namesort.xml]{grade.csv}{}{X}
\end{document}
";
    let (stderr, _xml) = convert(tex, true);
    assert!(
      stderr.contains("Fatal:TooManyErrors") || stderr.contains("TooManyErrors"),
      "{stderr}"
    );
    assert!(!stderr.contains("Fatal:Timeout"), "{stderr}");
    assert!(
      stderr.matches("not existent").count() < 700,
      "{}",
      stderr.matches("not existent").count()
    );
  }

  /// A preload that itself pulls in the LaTeX pool + dump pushes with the
  /// native `\lx@pushfilename`; the pop must use the SAME decision (Perl's
  /// `$pushpop`, Package.pm:2578/2637), not the dump's `\@popfilename` —
  /// otherwise `\__hook_curr_name_pop:` underflows ("Extra \PopDefaultHookLabel").
  #[test]
  fn preload_that_pulls_in_the_format_pops_with_the_native_stack() {
    let tex = r"\documentclass{article}
\begin{document}
Plain text.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("PopDefaultHookLabel"), "{stderr}");
    assert!(xml.contains("Plain text."), "{xml}");
  }

  /// `\tcbuselibrary{listings}` raw-loads tcblistingscore.code.tex, whose
  /// `\NewDocumentCommand \newtcblisting` must not find our override already
  /// defined (ltcmd `command-already-defined` = counted `\errmessage`). The
  /// family is installed by the code.tex binding after the raw load.
  #[test]
  fn tcbuselibrary_listings_installs_the_family_once() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newtcblisting{mybox}{listing only}
\NewTCBListing{exbox}{ O{} }{listing only,#1}
\begin{document}
\begin{mybox}
int alpha = 1;
\end{mybox}
\begin{exbox}
int beta = 2;
\end{exbox}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("already defined"), "{stderr}");
    // listings bodies are base64 `data=` payloads.
    assert!(xml.contains("aW50IGFscGhhID0gMTs="), "{xml}");
    assert!(xml.contains("aW50IGJldGEgPSAyOw=="), "{xml}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// graphics.sty:189 `\Ginclude@graphics` (driver-level include, called
  /// directly by pagelayout.cls:1494) routes to the `\includegraphics`
  /// constructor.
  #[test]
  fn ginclude_graphics_internal_routes_to_the_constructor() {
    let tex = r"\documentclass{article}
\usepackage{graphicx}
\makeatletter
\begin{document}
\Ginclude@graphics{example-image}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<graphics ").count(), 1, "{xml}");
    assert!(xml.contains(r#"graphic="example-image""#), "{xml}");
  }

  /// A pgfmath string result carrying a control sequence stays executable
  /// (pgfmathparser.code.tex:392-396 keeps `"…"` operands as real tokens);
  /// braids.sty:276 sets its strand counter through `\pgfmathresult`.
  #[test]
  fn pgfmath_string_result_keeps_control_sequences() {
    let tex = r#"\documentclass{article}
\usepackage{tikz}
\newcounter{mytest}
\setcounter{mytest}{1}
\begin{document}
\pgfmathparse{\value{mytest} < 4 ? "\noexpand\setcounter{mytest}{4}" : ""}%
\pgfmathresult
\typeout{EXECVAL:\the\value{mytest}}
\pgfmathparse{2*3}\typeout{NUMVAL:\pgfmathresult}
\end{document}
"#;
    let (stderr, _xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("EXECVAL:4"), "{stderr}");
    assert!(stderr.contains("NUMVAL:6"), "{stderr}");
  }

  /// xkeyval.tex:248 `\XKV@cc` (beamerposter.sty:55 calls it directly) and
  /// xkvutils.tex:110-124 `\XKV@whilist` (powerdot) are verbatim ports.
  #[test]
  fn xkeyval_choice_check_and_whilist_internals() {
    let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\XKV@cc*+[\val\nr]{a1}{a0,a1,a2}{\typeout{XKVCC-OK val=\val\space nr=\nr}}{\typeout{XKVCC-BAD}}
\define@choicekey{fam}{shape}[\val\nr]{circle,square}{\typeout{shape=\val/\nr}}
\def\lst{alpha,beta,gamma}
\XKV@whilist\lst\itm\ifx\itm\@nnil\fi{\typeout{WH:\itm}}
\makeatother
\begin{document}
\setkeys{fam}{shape=square}
Done.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("XKVCC-OK val=a1 nr=1"), "{stderr}");
    assert!(stderr.contains("shape=square/1"), "{stderr}");
    assert!(xml.contains("Done."), "{xml}");
  }

  /// fonttext.ltx:57-68,93: a Unicode-engine format inputs tuenc.def and makes
  /// TU the default encoding; xunicode-addon.sty:59-113 checks `\T@TU` exists.
  #[test]
  fn tu_encoding_is_declared_under_luatex() {
    let tex = r#"\documentclass{article}
\usepackage{xunicode-addon}
\makeatletter
\typeout{TUENC:\UnicodeEncodingName:\encodingdefault:\ifcsname T@TU\endcsname yes\else no\fi}
\makeatother
\begin{document}
Caf\'e na\"ive \textdollar\ \S
\end{document}
"#;
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("TUENC:TU:TU:yes"), "{stderr}");
    assert!(xml.contains("Café naïve $ §"), "{xml}");
  }

  /// tex.web §1063/§1064 `off_save`: `\endgroup` against an open math frame
  /// inserts the missing `$`, closes the math and re-reads the `\endgroup`
  /// (Perl leaves the frame open and every later closer re-errors).
  #[test]
  fn endgroup_against_open_math_inserts_the_missing_dollar() {
    let tex = r"\documentclass{article}
\begin{document}
Before \begingroup $x+1\endgroup after.

Next paragraph.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(stderr.contains("Missing $ inserted"), "{stderr}");
    assert!(!stderr.contains("Attempt to close"), "{stderr}");
    assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
    assert!(xml.contains("after."), "{xml}");
    assert!(xml.contains("Next paragraph."), "{xml}");
    // The math closed inside the paragraph: no Math element after the last </para>.
    let last_para_end = xml.rfind("</para>").unwrap_or(0);
    assert!(!xml[last_para_end..].contains("<Math"), "{xml}");
  }

  /// `\newtcblisting{env}[1]{…,#1}`: `[1]` is one MANDATORY argument
  /// (tcblistingscore.code.tex:318-323), so `\begin{env}{listing only}` reaches
  /// the mode decision and the displayed preamble code is not executed.
  #[test]
  fn newtcblisting_mandatory_argument_stays_mandatory() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings,skins}
\newtcblisting{DemoCode}[1]{%
	enhanced,width=\linewidth,%
	listing options={breaklines=true,commentstyle={\itshape}},%
	#1
}
\newtcblisting{OptCode}[1][listing only]{#1}
\begin{document}
\begin{DemoCode}{listing only}
\usepackage{calculatoritems}
\end{DemoCode}
\begin{OptCode}
\usepackage{calculatoritems}
\end{OptCode}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(
      xml
        .matches("XHVzZXBhY2thZ2V7Y2FsY3VsYXRvcml0ZW1zfQ==")
        .count(),
      2,
      "{xml}"
    );
    assert!(xml.contains("After."), "{xml}");
  }

  /// nicematrix.sty:5772→5704: `\rowcolors`/`\rowlistcolors` absorb a trailing
  /// `[keys]` optional (manual :2412 `[cols=2-3,restart]`, :2446 `[respect-blocks]`).
  #[test]
  fn nicematrix_rowcolors_trailing_optional_is_absorbed() {
    let tex = r"\documentclass{article}
\usepackage{nicematrix}
\usepackage[table]{xcolor}
\begin{document}
\begin{NiceTabular}{lr}
\CodeBefore
  \rowcolors[gray]{2}{0.8}{}[cols=2-3,restart]
  \rowlistcolors{1}{blue!10}[respect-blocks]
\Body
a & 12 \\
b & 13 \\
\end{NiceTabular}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("cols=2-3"), "{xml}");
    assert!(!xml.contains("respect-blocks"), "{xml}");
    assert!(xml.contains("<tabular"), "{xml}");
    assert!(xml.contains(">13<") || xml.contains("13</"), "{xml}");
  }

  /// A pgfmath string result keeps its letters at catcode 11
  /// (pgfmathparser.code.tex:35-40), so `\pgfmathresult` = `arc[…]` dispatches
  /// in `\tikz@handle` (tikz.code.tex:2134-2163) instead of "Giving up".
  #[test]
  fn pgfmath_string_result_keeps_letter_catcodes() {
    let tex = r#"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
  \node (a) at (0,0) {A};
  \draw (a) \pgfextra{\pgfmathparse{"arc[start angle=90,end angle=180,radius=5pt]"}}%
    \pgfmathresult;
\end{tikzpicture}
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Giving up"), "{stderr}");
    assert!(xml.matches("<svg:path").count() >= 1, "{xml}");
  }

  /// tcblistingscore.code.tex:195-224: the listing mode is tcolorbox's resolved
  /// state; `listing only` inside a user `.style` (tutodoc.cls:1208) must not
  /// execute the body.
  #[test]
  fn tcb_listing_mode_hidden_in_a_style_is_honoured() {
    let tex = r#"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\tcbset{mystyle/.style={listing only}}
\NewTCBListing{mycode}{ m }{ mystyle }
\NewTCBListing{runcode}{ m }{ listing and text }
\begin{document}
\begin{mycode}{}
if ($name eq "") { print "hi $name"; }
\end{mycode}
\begin{runcode}{}
\gdef\RAN{yes}
\end{runcode}
\typeout{RAN:\ifdefined\RAN\RAN\else no\fi}
After.
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<XMath"), "{xml}");
    assert!(
      xml.contains("aWYgKCRuYW1lIGVxICIiKSB7IHByaW50ICJoaSAkbmFtZSI7IH0="),
      "{xml}"
    );
    assert!(stderr.contains("RAN:yes"), "{stderr}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// codebox.sty:268 sets `listing only` from the ENCLOSING environment before
  /// its `\DeclareTCBListing` box; the C body must stay a listing.
  #[test]
  fn tcb_listing_mode_set_by_the_enclosing_environment_is_honoured() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\DeclareTCBListing[]{codeviewaux}{m}{title={#1}}
\newenvironment{codeview}{\tcbset{listing only}\codeviewaux{X}}{\endcodeviewaux}
\begin{document}
\begin{codeview}{demo}
#include <stdio.h>
int main(){return 0;}
\end{codeview}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("misdefined"), "{stderr}");
    assert!(xml.contains("I2luY2x1ZGUgPHN0ZGlvLmg+"), "{xml}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// tcolorbox.sty:2726-2735 `tcbverbatimwrite` writes the body without the
  /// `\begin`-line remainder as an empty first line (csvsimple reads line 1).
  #[test]
  fn tcbverbatimwrite_has_no_leading_blank_line() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{documentation}
\usepackage{csvsimple-legacy}
\begin{document}
\begin{tcbverbatimwrite}{grade.csv}
name,givenname,matriculation,gender,grade
Maier,Hans,12345,m,1.0
\end{tcbverbatimwrite}
\csvautotabular{grade.csv}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("empty line"), "{stderr}");
    assert!(xml.contains("<tabular"), "{xml}");
    assert!(xml.contains("Maier"), "{xml}");
  }

  /// pdfpages.sty:205 `\includepdfset{…}` (tutodoc :1339) is absorbed.
  #[test]
  fn includepdfset_is_absorbed() {
    let tex = r"\documentclass{article}
\usepackage{pdfpages}
\includepdfset{pages=-,fitpaper=true}
\begin{document}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// tabularray.sty:2006/2008: `\hline[style]`/`\cline[style]` inside tblr
  /// absorb their optional (manual :547 `\hline[dashed]\hline`).
  #[test]
  fn tblr_hline_style_optional_is_absorbed() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{lcr}
One & Two & Three \\
\hline[dashed]\hline
Four & Five & Six \\
\cline[dotted]{1-2}
Seven & Eight & Nine \\
\end{tblr}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("dashed"), "{xml}");
    assert!(!xml.contains("dotted"), "{xml}");
    assert!(xml.contains("Nine"), "{xml}");
    assert!(
      xml.contains(r#"border="tt""#) || xml.contains(r#"border="bb""#),
      "{xml}"
    );
  }

  /// beamer.cls:343 requires geometry; beamerposter.sty:176 calls `\geometry`.
  #[test]
  fn beamer_requires_geometry() {
    let tex = r"\documentclass{beamer}
\geometry{paperwidth=84.1cm,paperheight=118.9cm,hmargin=1cm}
\begin{document}
\begin{frame}\frametitle{Poster}Body text.\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Body text."), "{xml}");
    assert!(xml.contains("Poster"), "{xml}");
  }

  /// A `\lstnewenvironment` end code that displays a listing itself
  /// (exsheets-listings.sty:89-112) runs once — it is not the postamble every
  /// nested display re-reads.
  #[test]
  fn lstnewenvironment_end_code_with_a_listing_does_not_recurse() {
    let tex = r"\documentclass{article}
\usepackage{listings}
\begin{filecontents*}[overwrite]{pre.lst}
preexisting line one
preexisting line two
\end{filecontents*}
\lstnewenvironment{myq}[1][]{}{\lstinputlisting{pre.lst}}
\begin{document}
\begin{myq}
hello listing
\end{myq}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert_eq!(xml.matches("<listing ").count(), 2, "{xml}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// tagpdf-base.sty declares the tagging API with `\cs_new_protected`; the
  /// no-op stubs for tagpdf-less documents are retracted before it loads.
  #[test]
  fn tagpdf_base_redeclares_the_stubbed_api_cleanly() {
    let tex = r"\RequirePackage{pdfmanagement}
\documentclass{article}
\begin{document}
\tagstructbegin{tag=P}\tagmcbegin{tag=P}Tagged.\tagmcend\tagstructend
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("already defined"), "{stderr}");
    assert!(xml.contains("Tagged."), "{xml}");
  }

  /// expl3-code.tex:34944-34966 stubs `\lua_*` on a non-Lua format; the luatex
  /// profile rebinds them to the bridge.
  #[test]
  fn lua_functions_are_live_under_the_luatex_profile() {
    let tex = r"\documentclass{article}
\ExplSyntaxOn
\lua_load_module:n { luaotfload-main }
\lua_now:n { tex.print('LUANOW') }
\ExplSyntaxOff
\begin{document}
Body.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("LuaTeX engine not in use"), "{stderr}");
    assert!(xml.contains("Body."), "{xml}");
  }

  /// tabularray's `\Set*` table commands are gobbled out of the cell
  /// (tabularray.sty:3770-3860) and the `booktabs` environment takes the tblr
  /// key-value spec through the same colspec extraction (:8163).
  #[test]
  fn tblr_table_commands_and_booktabs_env() {
    let tex = r"\documentclass{article}
\usepackage{tabularray}
\UseTblrLibrary{booktabs}
\begin{document}
\begin{tblr}{colspec={lcr}}
 \SetRow{c}  Alpha   & Beta  & Gamma  \\
 \SetHline[1]{1-3}{solid}
 \SetColumn{c} Epsilon & Zeta  & Eta    \\
\end{tblr}
\begin{booktabs}{row{2}={c}}
\toprule
 One & Two & Three & Four \\
 Five & Six & Seven & Eight \\
\bottomrule
\end{booktabs}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<ERROR"), "{xml}");
    assert!(xml.contains("Alpha"), "{xml}");
    assert!(xml.contains("Eight"), "{xml}");
    assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
  }

  /// tex.web §1069/§1047 for a box reader: a box whose body left inline math
  /// open (`\mbox{$x}`) closes the math into the box instead of running to the
  /// end of the document; a balanced `\hbox{$x$}` stays error-free.
  #[test]
  fn box_end_over_leaked_math_closes_it_into_the_box() {
    let tex = r"\documentclass{article}
\begin{document}
Before \mbox{$x} after.

Next paragraph.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert!(error_count(&stderr) <= 2, "{stderr}");
    assert!(!stderr.contains("malformed"), "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
    assert!(xml.contains("after."), "{xml}");
    assert!(xml.contains("Next paragraph."), "{xml}");
    let last_para_end = xml.rfind("</para>").unwrap_or(0);
    assert!(!xml[last_para_end..].contains("<Math"), "{xml}");
    let tex = r"\documentclass{article}
\begin{document}
Before \hbox{$x$} after.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  }

  /// tcolorbox.sty:712 `tikz lower` wraps the box's executed lower part in a
  /// `tikzpicture`; the executed listing body runs inside it.
  #[test]
  fn tcblisting_tikz_lower_wraps_the_executed_body() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage[most]{tcolorbox}
\tcbuselibrary{listings}
\newtcblisting{DemoCode}[1][]{#1}
\begin{document}
\begin{DemoCode}[tikz lower]
\draw (0,0) -- (2,1);
\coordinate (A) at (1,1);
\end{DemoCode}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<picture"), "{xml}");
    assert!(xml.contains("<svg:path"), "{xml}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// beamerbasecompatibility.sty:517 `\beamertemplatedotitem` (called by the
  /// miniframes outer theme) and beamerbasecolor.sty:149 `{beamercolorbox}`.
  #[test]
  fn beamer_theme_compat_aliases_and_colorbox() {
    let tex = r"\documentclass{beamer}
\usetheme[compress]{Singapore}
\begin{document}
\begin{frame}
\beamertemplatearticlebibitems
\begin{beamercolorbox}[wd=\textwidth,rounded=true]{block body}
Hello colored box.
\end{beamercolorbox}
\begin{itemize}\item One\end{itemize}
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hello colored box."), "{xml}");
    assert!(xml.contains("<item"), "{xml}");
  }

  /// latex.ltx:9670 `\IfFileExists@` re-`\def`s the selected branch, halving
  /// `##` once (chemexec.sty:274-289 defines `\react@##1` inside it).
  #[test]
  fn iffileexists_branch_halves_doubled_parameters() {
    let tex = r"\documentclass{article}
\begin{document}
\IfFileExists{article.cls}{%
  \long\def\reactx##1{[X ##1 Y]}%
}{}
\IfFileExists{no-such-file-xyz.sty}{}{\def\other##1{(O ##1)}}
\reactx{Z} \other{W}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[X Z Y]"), "{xml}");
    assert!(xml.contains("(O W)"), "{xml}");
  }

  /// xkeyval.tex:446-448: `\presetkeys` apply through `\ProcessOptionsX` and
  /// `\ExecuteOptionsX` (powerdot.cls:52-92 sets `mode=present` only by preset);
  /// head presets fill un-given keys, given keys win, tail presets follow.
  #[test]
  fn xkeyval_option_processing_applies_presetkeys() {
    let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\@namedef{opt@.}{size=12pt}
\define@choicekey*[pd]{class}{mode}[\pd@tempa\pd@mode]{present,print,handout}{}
\define@cmdkey[pd]{class}{size}{}
\define@cmdkey[pd]{class}{disp}{}
\presetkeys[pd]{class}{mode=present,size=10pt}{disp=tail}
\ProcessOptionsX[pd]<class>\relax
\typeout{PROBE:mode=\pd@mode:size=\cmdpd@class@size:disp=\cmdpd@class@disp}
\define@cmdkey[ex]{fam}{width}{}
\presetkeys[ex]{fam}{width=3cm}{}
\ExecuteOptionsX[ex]<fam>{}
\typeout{EXEC:width=\cmdex@fam@width}
\makeatother
\begin{document}
\makeatletter\ifnum\pd@mode>0 MODEGT\else MODELE\fi\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      stderr.contains("PROBE:mode=0:size=12pt:disp=tail"),
      "{stderr}"
    );
    assert!(stderr.contains("EXEC:width=3cm"), "{stderr}");
    assert!(xml.contains("MODELE"), "{xml}");
  }

  /// The raw `{tcblisting}` environment (tcblistingscore.code.tex:275-283)
  /// hands `\tcbverbatimwrite` the UNEXPANDED `\kvtcb@listingfile`; the body
  /// must be stored under the expanded `\jobname.listing` name that
  /// `\tcbinputlisting@core` reads back (sweep #40: 25 manuals regressed with
  /// `missing_file:<job>.listing`; witness cistercian-doc).
  #[test]
  fn raw_tcblisting_environment_round_trips_its_listing_file() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\begin{document}
\begin{tcblisting}{title={Font scaling}}
\textbf{bold} Text
\end{tcblisting}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("Can't find") && !stderr.contains("Can't read"),
      "{stderr}"
    );
    assert!(xml.contains("<listing"), "{xml}");
    assert!(xml.contains("font=\"bold\""), "{xml}");
  }

  /// latex.ltx:15504 `\verb@eol@error`: an unterminated `\verb` stops at the
  /// end of its line with ONE recoverable error instead of scanning across
  /// lines and swallowing a later `{verbatim}` (bigints manual; SHARED).
  #[test]
  fn verb_ended_by_end_of_line_recovers() {
    let tex = r"\documentclass{article}
\begin{document}
This package (\verb v1.1 ) helps you.

\begin{center}
\begin{verbatim}
\usepackage{bigints}
\end{verbatim}
\end{center}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(stderr.contains("ended by end of line"), "{stderr}");
    assert!(xml.contains("<verbatim"), "{xml}");
    assert!(xml.contains(r"\usepackage{bigints}"), "{xml}");
    assert!(xml.contains("After."), "{xml}");
  }

  /// pdfTeX `\pdfmatch` (dataref.sty:374 `\let\dref@strmatch\pdfmatch`, then
  /// `\ifnum\dref@strmatch{#1}{#2}=1`) expands to the match flag, and
  /// `\pdflastmatch` to `<pos>-><text>` (dataref-doc; SHARED, pdflatex clean).
  #[test]
  fn pdfmatch_expands_to_a_match_flag() {
    let tex = r"\documentclass{article}
\begin{document}
\ifnum\pdfmatch{b}{abc}=1 yes\else no\fi.
\ifnum\pdfmatch{z}{abc}=0 none\else some\fi.
\ifnum\pdfmatch icase {B(C)}{abc}=1 \pdflastmatch0/\pdflastmatch1\fi.
\ifnum\pdfmatch{(}{abc}=-1 bad\fi.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("yes."), "{xml}");
    assert!(xml.contains("none."), "{xml}");
    assert!(xml.contains("bc/2-") && xml.contains("c.\nbad."), "{xml}");
    assert!(xml.contains("bad."), "{xml}");
  }

  /// `\endlist` = `\endlx@list` = endMode('internal_vertical') (Perl
  /// latex_constructs.pool.ltxml:1651-1653) also closes an enumerate opened
  /// by its begin macro: nih/denselists.sty:16 `\newenvironment{Enumerate}
  /// {\Onumerate\Nospacing}{\endlist}` (example-biosketch, polydemo; RUST-ONLY).
  #[test]
  fn endlist_closes_an_enumerate_opened_by_its_begin_macro() {
    let tex = r"\documentclass{article}
\let\Onumerate=\enumerate
\newenvironment{Enumerate}{\Onumerate}{\endlist}
\begin{document}
\begin{Enumerate}
\item x
\end{Enumerate}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<enumerate") && xml.contains("<item "),
      "{xml}"
    );
    assert!(xml.contains("After."), "{xml}");
  }

  /// latex.ltx:18901 `\AtBeginDocument` = `\AddToHook{begindocument}`, so a
  /// `\RemoveFromHook{begindocument}[pkg]` cancels a package's
  /// `\AtBeginDocument{\MakeShortVerb\"}` (source2edoc.cls:12 vs
  /// l3doc.cls:511; base/source2e's ltoutenc macrocode leak; SHARED).
  #[test]
  fn atbegindocument_joins_the_l3_begindocument_hook() {
    let tex = r#"\documentclass{article}
\usepackage{doc}
\begin{filecontents}[overwrite,noheader,nosearch]{lxshortq.sty}
\AtBeginDocument{\MakeShortVerb\"}
\end{filecontents}
\usepackage{lxshortq}
\RemoveFromHook{begindocument}[lxshortq]
\AtBeginDocument{\def\lxhookran{ran}}
\begin{document}
Start "verb \textbf{bold" and more} done. \lxhookran
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("font=\"bold\""), "{xml}");
    assert!(!xml.contains("<verbatim"), "{xml}");
    assert!(xml.contains("ran"), "{xml}");
  }

  /// A listings example environment under `\DocInput` (forest-doc.sty:48
  /// `forestexample`, `gobble=2`, `\lst@BeginAlsoWriteFile` + re-input): the
  /// body's first line is read whole from column 0 so its `% ` survives like
  /// lines 2+, and the write-file tee is gobbled like real listings', so the
  /// re-input's `\end{forest}` is not commented out (forest-doc: 501
  /// `readBalanced ran out of input` + Fatal; RUST-ONLY, pdflatex clean).
  #[test]
  fn forest_docinput_lstenv_writefile_gobbles_doc_percent() {
    let tex = r"\documentclass{ltxdoc}
\begin{filecontents*}{fdtx.dtx}
% \iffalse
% \fi
% \section{T}
% \begin{forestexample}
%   \begin{forest}
%     [VP[DP][V]]
%   \end{forest}
% \end{forestexample}
% \endinput
\end{filecontents*}
\usepackage[external]{forest}
\usepackage{forest-doc}
\begin{document}
\DocInput{fdtx.dtx}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert!(!stderr.contains("Fatal:"), "{stderr}");
    assert!(!stderr.contains("ran out of input"), "{stderr}");
    assert!(xml.contains("<section"), "{xml}");
    // base64 of the gobbled first line "\begin{forest}" is what the listing
    // data starts with — line 1 lost neither its `\b` nor its indentation.
    assert!(xml.contains("<listing"), "{xml}");
  }

  /// polyglossia.sty:641 `\xpg_if_script:nTF` answers TRUE (there is no
  /// OpenType font model to ask; a lualatex-clean document loaded a
  /// script-capable font), so a non-Latin font switch no longer raises "The
  /// current main roman font, cmr10, does not contain the Greek script!"
  /// (fontsetup/fspsample ×2 11→0, greektonoi 16→0, latex-mr 98+Fatal→3).
  #[test]
  fn polyglossia_script_check_passes_without_font() {
    let tex = r"\documentclass{article}
\usepackage{polyglossia}
\setdefaultlanguage{english}
\setotherlanguage{greek}
\usepackage[default]{fontsetup}
\begin{document}
Hello \textgreek{ασδφ} world.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("ασδφ"), "{xml}");
  }

  /// `\DocumentMetadata{tagging=on}` also loads the block and minipage
  /// testphase modules (latex-lab-testphase-latest.sty:43,47): ltx-talk.cls
  /// :1860 `\EditInstance{item}{basic}`, tagpdfdocu-patches.sty:127
  /// `\DeclareInstance{blockenv}{docCommand}{display}` + `\UseInstance` +
  /// `\endblockenv`, and :146 `\AssignSocketPlug{tagsupport/minipage/before}
  /// {noop}` find their declarations (ltx-talk ×10, tagpdf manual 113 lines).
  #[test]
  fn testphase_tagging_sockets_and_block_templates_are_declared() {
    let tex = r"\DocumentMetadata{tagging = on}
\documentclass{article}
\ExplSyntaxOn
\EditInstance{item}{basic}{label-format = #1}
\DeclareInstance{blockenv}{docCommand}{display}{ name = docCommand, tag-name = Div, increment-level = false }
\AssignSocketPlug{tagsupport/minipage/before}{noop}
\ExplSyntaxOff
\begin{document}
A\UseTaggingSocket{minipage/before}B
\UseInstance{blockenv}{docCommand}{tag-name=Div,leftmargin=1pt,rightmargin=2pt}Hello\endblockenv
\begin{itemize}\item one\end{itemize}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("AB"), "{xml}");
    assert!(xml.contains("Hello"), "{xml}");
    assert!(xml.contains("<item "), "{xml}");
  }

  /// A `./`-prefixed name written by `\openout`/`\write` (fancyvrb
  /// `{VerbatimOut}{./foo.tex}`) is read back by `\VerbatimInput{./foo.tex}`:
  /// the VFS keys both sides without the `./` (xpicture-doc, checklistings;
  /// RUST-ONLY).
  #[test]
  fn verbatimout_dotslash_round_trips_through_the_vfs() {
    let tex = r"\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
\begin{VerbatimOut}{./foo.tex}
hello dotslash world
\end{VerbatimOut}
\VerbatimInput{./foo.tex}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("hello dotslash world"), "{xml}");
  }

  /// lthooks runs `top-level` chunks of `begindocument` AFTER package
  /// chunks: pgfmanual-en-macros.tex:35's document-level
  /// `\AtBeginDocument{\gdef|{\ifmmode…}}` must beat pgfmanual.pdflinks
  /// .code.tex:413-416's `\let|=\pgfmanual@verb` (registered later, under
  /// the `pgfmanual` label), so `\biggl|{r}\biggr|` in math is a fence, not
  /// a verbatim collector opening a group inside the box (tikz-ext-manual:
  /// ~950 of 1001 errors; SHARED). Follows from `\AtBeginDocument` joining
  /// the L3 hook.
  #[test]
  fn pgfmanual_toplevel_atbegindocument_runs_last() {
    let tex = r"\documentclass[a4paper,doc2,landscape]{ltxdoc}
\usepackage{tikz}
\input{pgfmanual-en-macros}
\begin{document}
\[ \biggl| {r} \biggr| \]
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("absolute-value"), "{xml}");
  }

  /// A standalone class's `\cs_new:Npn \thepage` (ltx-talk.cls:158) over the
  /// pool's `\thepage` (article.cls material in real LaTeX) is quiet and
  /// takes effect (ltx-talk ×10, 24 errors each; RUST-ONLY).
  #[test]
  fn l3_cs_new_over_a_pool_definition_is_quiet() {
    let tex = r"\begin{filecontents*}[overwrite]{poolc.cls}
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{poolc}[2026/01/01 pk-expl3]
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\ExplSyntaxOn
\cs_new:Npn \thepage { \@arabic \c@page }
\cs_new:Npn \figurename { Fig }
\ExplSyntaxOff
\normalsize
\end{filecontents*}
\documentclass{poolc}
\begin{document}
Page \thepage. \figurename.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Page 1. Fig."), "{xml}");
    assert!(!xml.contains("<ERROR"), "{xml}");
  }

  /// ltcmd's `\NewDocumentEnvironment{figure}` / `\NewDocumentCommand
  /// \section` (ltx-talk.cls:1016-1033, :1574-1580) over the pool's
  /// constructors: no error, and the pool's `<figure>`/`<section>` survive
  /// (ltcmd keeps the existing definition after its check).
  #[test]
  fn ltcmd_declarators_keep_pool_constructors_quietly() {
    let tex = r"\begin{filecontents*}[overwrite]{poolc.cls}
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{poolc}[2026/01/01 pk-expl3]
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\ExplSyntaxOn
\NewDocumentEnvironment { figure } { } { } { }
\NewDocumentCommand \section { m } { }
\ExplSyntaxOff
\normalsize
\end{filecontents*}
\documentclass{poolc}
\begin{document}
\section{Hi}
\begin{figure}Body\end{figure}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<section") && xml.contains("Hi</title>"),
      "{xml}"
    );
    assert!(xml.contains("<figure"), "{xml}");
  }

  /// The bindings' deferred begin-document code (cleveref_sty.rs `\let\label
  /// \lx@cleverref@label`) runs AFTER the raw packages' `begindocument` hook
  /// chunks, so raw cleveref.sty:66's `\def\label{\@ifnextchar[…}` does not
  /// shadow it (its `[#1][#2]` scan ran to EOF: crossreftools_driver,
  /// test-autonum fatal; RUST-ONLY).
  #[test]
  fn binding_begin_document_code_outranks_raw_hook() {
    let tex = r"\documentclass{article}
\usepackage{amsmath}
\usepackage{cleveref}
\begin{document}
\begin{equation}a^2+b^2=c^2\label[section]{pyth}\end{equation}
See \cref{pyth}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert!(xml.contains("labels=\"LABEL:pyth\""), "{xml}");
  }

  /// `\NewTCBListing{E}{ O{} D<>{} }` / `{ !O{} !s }` / `{ !G{1} !O{} }`
  /// (tutodoc.cls:1024, simplebnf-doc.tex:58, istgame-doc.tex:129): the
  /// begin-line arguments the `\lstnewenvironment` arity cannot express are
  /// absorbed, and the environment's own `\begin` line is never captured as
  /// body (it re-entered the environment on `\input`-back without bound:
  /// MemoryBudget fatal ×4, sweep #41; RUST-ONLY).
  #[test]
  fn tcb_listing_unmapped_begin_line_args_are_absorbed() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings,breakable}
\tcbset{listing engine=listings}
\NewTCBListing{mylst}{ O{} D<>{} }{ listing side text, #1 }
\NewTCBListing{example}{ !O{} !s }{ listing side text, #1 }
\DeclareTCBListing{doccode}{ !G{1} !O{} }{ listing only }
\begin{document}
\begin{mylst}<colback=red>
Some code line A
\end{mylst}
\begin{example}*
Some code line B
\end{example}
\begin{doccode}{colback=blue}
Some code line C
\end{doccode}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Fatal"), "{stderr}");
    assert_eq!(xml.matches("<listing ").count(), 3, "{xml}");
    assert!(!xml.contains("colback"), "{xml}");
    // `doccode` is `listing only`: its body is the base64 data, not text.
    assert!(
      xml.contains("Some code line A") && xml.contains("Some code line B"),
      "{xml}"
    );
    assert!(xml.contains("U29tZSBjb2RlIGxpbmUgQw=="), "{xml}");
  }

  /// `\mathitalicsmode` is a LuaTeX integer parameter (expl3-code.tex:996),
  /// set by lualatex classes (homework, jwjournal).
  #[test]
  fn mathitalicsmode_is_a_register() {
    let tex = r"\documentclass{article}
\begin{document}
\mathitalicsmode=1 Mode \the\mathitalicsmode.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Mode 1."), "{xml}");
  }

  /// beamerbasetranslator.sty:14 loads translator: `\uselanguage` from a
  /// language pack (ctex-scheme-chinese-beamer.def:71; mirage-beamer-zh).
  #[test]
  fn beamer_loads_translator() {
    let tex = r"\documentclass{beamer}
\uselanguage{English}\languagealias{en}{English}
\begin{document}
\begin{frame}Hi \translate{Theorem}\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hi"), "{xml}");
  }

  /// `\IfFontExistsTF` answers like luaotfload's case-insensitive database:
  /// asmeconf.cls:650 asks for `TexGyreTermesX-regular.otf` (TeX Live ships
  /// `TeXGyreTermesX-Regular.otf`), so the class must not take its
  /// missing-font `\ClassErrorNoLine` branch (asmeconf/asmejour templates).
  #[test]
  fn font_exists_test_is_case_insensitive() {
    let tex = r"\documentclass{article}
\usepackage{fontspec}
\begin{document}
\IfFontExistsTF{TexGyreTermesX-regular.otf}{found}{missing}.
\IfFontExistsTF{NoSuchFontXyz.otf}{found}{missing}.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("found.") && xml.contains("missing."), "{xml}");
  }

  /// `\usepackage[authordate]{biblatex-chicago}` selects the author-date
  /// family and chicago-dates-common.cbx:2966's `\gentextcite` renders as a
  /// text cite (cms-dates-intro, cms-dates-sample; lualatex clean).
  #[test]
  fn biblatex_chicago_authordate_has_gentextcite() {
    let tex = r"\documentclass{article}
\usepackage[authordate,backend=biber]{biblatex-chicago}
\begin{document}
As \gentextcite{k1} shows; \Gentextcite{k1}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<ERROR"), "{xml}");
    assert!(xml.contains("<cite"), "{xml}");
  }

  /// A `#`-bearing `\AtBeginDocument` chunk registered under a package's own
  /// label (pm-isomath.sty:150 `\providecommand\mathrmbf[1]{…}` and its
  /// `\NewDocumentCommand…{…#2…}` blocks) takes the private store: lthooks'
  /// labeled cleanup path is not yet reproduced by our gullet
  /// (euclideangeometry-man: 100× `\csname g__hook_` errors + Fatal, sweep
  /// #41). K3 correctness item; this guard pins the interim.
  #[test]
  fn hashful_begin_document_chunk_under_a_package_label() {
    let tex = r"\documentclass{article}
\begin{filecontents}[overwrite,noheader,nosearch]{lxhashpkg.sty}
\AtBeginDocument{\NewDocumentCommand\lxhashcmd{s m}{[#2]}\providecommand\lxhashplain[1]{(#1)}}
\end{filecontents}
\usepackage{lxhashpkg}
\begin{document}
\lxhashcmd{v} \lxhashplain{w}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("g__hook_"), "{stderr}");
    assert!(xml.contains("[v] (w)"), "{xml}");
  }

  /// The forest/diagrams discard stubs warn instead of erroring: the body is
  /// discarded cleanly (forest-quickstart, fragoli_doc, milsymb; pdflatex clean).
  #[test]
  fn forest_stub_is_a_warning() {
    let tex = r"\documentclass{article}
\usepackage{forest}
\begin{document}
Before.
\begin{forest}
[VP [V [sees]] [NP [DP [the]] [NP [dog]]]]
\end{forest}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Before.") && xml.contains("After."), "{xml}");
  }

  /// `{subeqnarray}` (subeqnarray.sty:33-41) is eqnarray with `\slabel`
  /// subnumbers: `&` aligns, rows get `1a`/`1b` (subeqnarray-sample).
  #[test]
  fn subeqnarray_aligns_with_subnumbers() {
    let tex = r"\documentclass{article}
\usepackage{subeqnarray}
\begin{document}
\begin{subeqnarray}
\slabel{a} x & = & a \\
\slabel{b}   & = & b
\end{subeqnarray}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<equationgroup"), "{xml}");
    assert!(xml.contains("1a") && xml.contains("1b"), "{xml}");
  }

  /// `\blendcolors*{!60!white}` then `\textcolor{black!75}`: the blend is a
  /// separate mix on the resolved color (gray .25 → .55 = #8C8C8C), not a
  /// string-concatenated `black!75!60!white` (iodhbwm via ydoc-desc.sty:125).
  #[test]
  fn xcolor_blend_applies_after_the_local_mix() {
    let tex = r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\blendcolors*{!60!white}\textcolor{black!75}{hello}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("#8C8C8C"), "{xml}");
  }

  /// siunitx `per-mode=power` (its default) renders per-units as negative
  /// exponents like our `reciprocal` (quantum-chemistry-bonn.sty:55).
  #[test]
  fn siunitx_per_mode_power_renders_reciprocal() {
    let tex = r"\documentclass{article}
\usepackage{siunitx}
\sisetup{per-mode=power}
\begin{document}
Energy: \qty{5}{\kilo\joule\per\mole}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("mol"), "{xml}");
    assert!(!xml.contains("<ERROR"), "{xml}");
  }

  /// `\documentclass[pdftex]` puts `pdfmode` in the backend request; naming
  /// `dvips` at the `\document` backend load avoids expl3's "Backend request
  /// inconsistent with engine" (elpres, scidoc), under both profiles.
  #[test]
  fn backend_load_names_the_dvi_backend() {
    let tex = r"\documentclass[pdftex]{article}
\usepackage{xcolor}
\begin{document}
Hello \textcolor{red}{world}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("world"), "{xml}");
    let (stderr, _) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// xpatch.sty:42 loads xparse, which restores ltcmd's legacy `g` argument
  /// type (prtec.cls:316 `\NewDocumentCommand\entry{m g}`).
  #[test]
  fn xpatch_loads_xparse_for_legacy_arg_types() {
    let tex = r"\documentclass{article}
\usepackage{xpatch}
\NewDocumentCommand{\entry}{m g}{[#1/#2]}
\begin{document}
\entry{A}{B} \entry{C}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[A/B]"), "{xml}");
  }

  /// Under `\DocumentMetadata`, `\definecolor` also registers the color with
  /// l3color (xcolor-patches-tmp-ltx.sty:56), so raw `\color_select:n`
  /// (ltx-talk.cls:201) finds it.
  #[test]
  fn xcolor_definecolor_bridges_to_l3color_under_documentmetadata() {
    let tex = r"\DocumentMetadata{}
\documentclass{article}
\usepackage{xcolor}
\definecolor{alert}{RGB}{200,0,0}
\begin{document}
\ExplSyntaxOn \color_select:n {alert} \ExplSyntaxOff text.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("text."), "{xml}");
  }

  /// K1 provenance: the l3/ltcmd leniency covers LaTeXML's OWN definitions
  /// only — a genuine double declaration between two raw files (or in the
  /// document) still reports "already defined", as pdflatex does.
  #[test]
  fn raw_double_declaration_still_errors() {
    let tex = r"\documentclass{article}
\begin{filecontents}[overwrite,noheader,nosearch]{lxdup.sty}
\ExplSyntaxOn
\cs_new:Npn \lxdupcmd { one }
\ExplSyntaxOff
\end{filecontents}
\usepackage{lxdup}
\ExplSyntaxOn
\cs_new:Npn \lxdupcmd { two }
\ExplSyntaxOff
\NewDocumentCommand\lxdupdoc{}{a}
\NewDocumentCommand\lxdupdoc{}{b}
\begin{document}
\lxdupcmd\ \lxdupdoc.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 2, "{stderr}");
    assert!(stderr.contains("already defined"), "{stderr}");
    assert!(xml.contains("two a."), "{xml}");
  }

  /// Perl Expandable.pm:35: an unbalanced expansion is FATAL. jarticle.cls's
  /// `\ds@tate` (ISO-2022-JP bytes whose `%` eats a brace) aborts in seconds
  /// instead of proceeding into a 250 s loop (platexcheat; RUST-ONLY).
  #[test]
  fn unbalanced_expansion_is_fatal() {
    let tex = r"\documentclass[12pt,a4j,dvipdfmx]{jarticle}
\begin{document}
Hello
\end{document}
";
    let start = std::time::Instant::now();
    let (stderr, _xml) = convert(tex, true);
    assert!(stderr.contains("Fatal:Stomach:Misdefined"), "{stderr}");
    assert!(start.elapsed().as_secs() < 60, "took {:?}", start.elapsed());
  }

  /// japanese-otf's ajmacros.sty (pTeX kanji token model, parked §D9) bails
  /// with an explicit Fatal in under a second instead of an aperiodic
  /// 250 s loop (platexsheet-jsclasses, wtref-ja, jpneduenumerate; SHARED).
  #[test]
  fn japanese_otf_kanji_scanners_bail_fast() {
    let tex = r"\documentclass{article}
\usepackage{otf}
\begin{document}
Hello
\end{document}
";
    let start = std::time::Instant::now();
    let (stderr, _xml) = convert(tex, true);
    assert!(
      stderr.contains("Fatal:") && stderr.contains("ajmacros"),
      "{stderr}"
    );
    assert!(start.elapsed().as_secs() < 60, "took {:?}", start.elapsed());
  }

  /// lthooks is FIFO within a label: a raw `#`-bearing `\AtBeginDocument`
  /// chunk registered first runs before a `#`-free one registered later
  /// (alphabeta.sty then hep-math-font.sty; hep-paper-documentation fatal).
  #[test]
  fn raw_hashful_begin_document_chunk_keeps_fifo_order() {
    let tex = r"\documentclass{article}
\AtBeginDocument{\def\dummy#1{#1}\def\WHO{FIRST-param}}
\AtBeginDocument{\def\WHO{SECOND-plain}}
\begin{document}
Who: \WHO.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Who: SECOND-plain."), "{xml}");
  }

  /// `\NewTCBListing{egcite}{D(){teal} o m !o}{colframe=#1,…}` (oxyear-doc
  /// .tex:216): the absorbed `D()` still owns `#1` (its default `teal`), so
  /// the mandatory citation text never reaches `colframe`.
  #[test]
  fn tcb_listing_absorbed_specifiers_keep_positional_numbers() {
    let tex = r"\documentclass{article}
\usepackage[most]{tcolorbox}
\tcbuselibrary{listings}
\definecolor{teal}{rgb}{0,0.5,0.5}
\NewTCBListing{egcite}{D(){teal} o m !o}%
  {colframe = #1 ,colback = #1!5!white ,listing side text}
\begin{document}
\begin{egcite}{(Marx 1867), (Clarke, n.d.).}
Some text.
\end{egcite}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Can't find color"), "{stderr}");
    assert!(xml.contains("Some text."), "{xml}");
  }

  /// 300 tcolorbox pictures stay bounded under `--streaming --max-memory=800`
  /// (fuse at 600 MB): each finished picture releases its node boxes, so a
  /// 1,000-box manual no longer climbs to the fuse with a 39-byte XML
  /// (glossaries-user, glossaries-extra-manual, datatool-user).
  #[test]
  fn tcolorbox_pictures_stay_memory_bounded() {
    let tex = r"\documentclass{article}
\usepackage[most]{tcolorbox}
\newtcolorbox{cb}{enhanced,breakable}
\newcount\ct \ct=0
\begin{document}
\loop\ifnum\ct<300
  \begin{cb}Sample code line \the\ct\end{cb}
  \advance\ct by 1
\repeat
\end{document}
";
    let (stderr, xml) = convert_args(tex, &["--streaming", "--max-memory=800"]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("MemoryBudget"), "{stderr}");
    assert_eq!(xml.matches("<picture").count(), 300, "{}", xml.len());
  }

  /// Under the luatex profile `\DeclareUnicodeCharacter` declares nothing
  /// (latex.ltx:22168/22203 — utf8.def is 8-bit-engine only), so a class's
  /// `\cs_new_protected:Npn ·` finds the native character free
  /// (einfart.cls:838-839; homework-demo-cn/-jp/-tc, jwjournal-demo-cn).
  /// Repro `unicode-catcodes/declareunicodechar_middot_luatex_einfart.tex`.
  #[test]
  fn unicode_engine_keeps_middle_dot_native() {
    let tex = "\\documentclass{article}
\\ExplSyntaxOn
\\char_set_catcode_active:n { `\\· }
\\cs_new_protected:Npn · { \\ensuremath\\cdot }
\\ExplSyntaxOff
\\begin{document}
Middle dot active: $a·b$.
\\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("already defined"), "{stderr}");
    assert!(xml.contains("\u{22c5}") || xml.contains("\\cdot"), "{xml}");
    // The 8-bit profile still activates the LICR mapping.
    let tex8 = "\\documentclass{article}
\\begin{document}
Middle dot: ·.
\\end{document}
";
    let (stderr, xml) = convert(tex8, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Middle dot: ·."), "{xml}");
  }

  /// expl3-code.tex:985-986 alias `\tex_luatexversion:D`/`\tex_luatexrevision:D`
  /// at format time; the luatex profile re-derives them from its own
  /// `\luatexversion` (lua-widow-control.sty:153 compares `\tex_luatexversion:D`).
  #[test]
  fn luatex_profile_aliases_expl3_version_primitives() {
    let tex = "\\documentclass{article}
\\ExplSyntaxOn
\\int_compare:nNnTF { \\tex_luatexversion:D } > { 200 } { \\def\\x{NEW} } { \\def\\x{OLD-\\tex_luatexversion:D} }
\\ExplSyntaxOff
\\begin{document}
Version: \\x.
\\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Version: OLD-121."), "{xml}");
  }

  /// dhucs's Unicode-native branch (dhucs.sty:44 `\ifx가가` true) skips the
  /// `\if@hangul` block that defines `\pdfstringdefPreHook` (:117) and
  /// `\dhucs@emph@raise`, which memhangul-ucs.sty:509/:451 then read; the
  /// overlay supplies them, and hyperref keeps an existing hook (Perl
  /// hyperref.sty.ltxml:413). Repro
  /// `loader/dhucs_native_pdfstringdefprehook_istgame.tex`.
  #[test]
  fn dhucs_native_branch_defines_pdfstringdefprehook() {
    let tex = r"\documentclass{article}
\usepackage{dhucs}
\makeatletter
\g@addto@macro\pdfstringdefPreHook{\def\lxprobe{kept}}
\makeatother
\usepackage{hyperref}
\begin{document}
\makeatletter\pdfstringdefPreHook
ok \lxprobe\ \the\dhucs@emph@raise\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("ok kept 0.0pt"), "{xml}");
  }

  /// The begin-document backend loader follows expl3.ltx:130: skip when a
  /// backend was already chosen, auto-select in PDF output, `dvips` in DVI.
  /// Repros `backend-persona/{pdfoutput_inconsistent,backend_already_set}.tex`.
  #[test]
  fn backend_load_follows_pdfoutput_and_prior_choice() {
    let pdf = r"\documentclass[11pt]{article}
\ifx\pdfoutput\undefined\else
  \pdfoutput=1
\fi
\begin{document}
Hello world.
\end{document}
";
    let (stderr, xml) = convert(pdf, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hello world."), "{xml}");
    let set = r"\documentclass{article}
\pdfoutput=1
\makeatletter\ExplSyntaxOn
\sys_load_backend:n {pdftex}
\ExplSyntaxOff\makeatother
\begin{document}
Hello.
\end{document}
";
    let (stderr, xml) = convert(set, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hello."), "{xml}");
  }

  /// biblatex.sty:12862 `\citefield`, :3649 `\mkcomprange`, and the
  /// :4371-4377 page-string family (`\pno`, `\psqq`) at document level
  /// (oxref manuals, biblatex-german-legal, biblatex-true-citepages-omit).
  /// Repro `index-bib/blx_toplevel_pagehelpers_oxref.tex`.
  #[test]
  fn biblatex_field_cites_and_page_strings() {
    let tex = r"\documentclass{article}
\usepackage[style=authoryear,backend=biber]{biblatex}
\begin{document}
Alpha \citefield{smith}{labelalpha}, range \mkcomprange{367-368}, first \mkfirstpage{367--368}.
See \cite[\pno~110]{smith}; also \cite[295 \psqq]{jones}.
Title \citefield{smith}{title}, editors \citename{smith}{editor}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("range 367-368, first 367."), "{xml}");
    assert!(
      xml.contains("p.\u{a0}110") || xml.contains("p.~110"),
      "{xml}"
    );
    assert!(xml.contains("sqq.</cite>"), "{xml}");
    assert!(xml.contains("show=\"Title\""), "{xml}");
    assert!(xml.contains("class=\"ltx_citemacro_citename\""), "{xml}");
  }

  /// `\usepackage[style = abnt]{biblatex}` with spaces around `=`
  /// (biblatex-abnt.tex:53) still selects the style, so `abnt.cbx` loads and
  /// its `\apud` exists.
  #[test]
  fn biblatex_style_option_tolerates_spaces() {
    let tex = r"\documentclass{article}
\usepackage[style = abnt, backend = biber]{biblatex}
\begin{document}
\makeatletter\typeout{[\meaning\apud]}\makeatother
Text.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("[undefined]"), "{stderr}");
    assert!(xml.contains("Text."), "{xml}");
  }

  /// newpxmath (uantwerpenexam-example2 `\square`) and MnSymbol (univie-ling
  /// `\blacktriangleright`, atableau `\bigcircle`) carry the AMS symbol set.
  #[test]
  fn font_symbol_packages_carry_amssymb() {
    for (pkg, sym) in [
      ("newpxmath", "\\square"),
      ("MnSymbol", "\\blacktriangleright"),
      ("MnSymbol", "\\bigcircle"),
    ] {
      let tex = format!(
        "\\documentclass{{article}}\n\\usepackage{{{pkg}}}\n\\begin{{document}}\n$a {sym} b$\n\\end{{document}}\n"
      );
      let (stderr, xml) = convert(&tex, true);
      assert_eq!(error_count(&stderr), 0, "{pkg} {sym}: {stderr}");
      assert!(!xml.contains("<ERROR"), "{pkg} {sym}: {xml}");
    }
  }

  /// mdframed.sty:591 `\newmdtheoremenv` defines a theorem environment
  /// (beautynote).
  #[test]
  fn mdframed_theorem_environments_are_theorems() {
    let tex = r"\documentclass{article}
\usepackage{amsthm}
\usepackage{mdframed}
\newmdtheoremenv[linewidth=1pt]{theorem}{Theorem}[section]
\newmdtheoremenv{lemma}[theorem]{Lemma}
\begin{document}
\section{One}
\begin{theorem}Thm body.\end{theorem}
\begin{lemma}Lemma body.\end{lemma}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<theorem") && xml.contains("Lemma body."),
      "{xml}"
    );
  }

  /// thm-restate.sty:191 `restatable*` (proof-at-the-end demo) and
  /// lineno.sty:2881 `bframe` (ulineno).
  #[test]
  fn restatable_star_and_lineno_bframe() {
    let tex = r"\documentclass{article}
\usepackage{amsthm}
\usepackage{thm-restate}
\usepackage{lineno}
\newtheorem{theorem}{Theorem}
\begin{document}
\begin{restatable*}[Main]{theorem}{mainthm}Restated body.\end{restatable*}
\begin{bframe}Framed text.\end{bframe}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Restated body.") && xml.contains("Framed text."),
      "{xml}"
    );
  }

  /// subeqn.sty:51 `subeqnarray` (subeqn-sample).
  #[test]
  fn subeqn_subeqnarray_environment() {
    let tex = r"\documentclass{article}
\usepackage{subeqn}
\begin{document}
\begin{subeqnarray}\label{main}
a &=& b \\
c &=& d
\end{subeqnarray}
Eq.~\ref{main}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<equationgroup"), "{xml}");
  }

  /// unicode-math-table.tex rows become math symbols (derivative `\coloneq`,
  /// rec-thy `\nvrightarrow`/`\mathhyphen`, shtthesis `\oiint`), while a
  /// kernel-defined name keeps its own definition.
  #[test]
  fn unicode_math_symbol_table_defines_names() {
    let tex = r"\documentclass{article}
\usepackage{unicode-math}
\removenolimits{\sum}
\begin{document}
$a \coloneq b \nvrightarrow c \mathhyphen d \oiint_S f \le g$
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("\u{2254}") && xml.contains("\u{21f8}") && xml.contains("\u{222f}"),
      "{xml}"
    );
    assert!(xml.contains("less-than-or-equals"), "{xml}");
  }

  /// `\DeclareMathOperator` stores its body unexpanded like TeX: iidef.sty:147
  /// names `\mathds` with dsfont unloaded and never uses the operator (ithw).
  #[test]
  fn declaremathoperator_body_stays_lazy() {
    let tex = r"\documentclass{article}
\usepackage{amsmath}
\DeclareMathOperator{\one}{\mathds{1}}
\DeclareMathOperator{\Tr}{{\rm Tr}}
\begin{document}
$\Tr A$
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Tr"), "{xml}");
  }

  /// biblatex-chicago's `notes` style (the default) loads chicago-notes
  /// bbx/cbx, whose `\DeclareCiteCommand`s define `\runcite` and
  /// `\headlessfullcite` (cms-legal-sample, cms-notes-sample); internals a
  /// document reaches directly (`\blx@opt@loccittracker@false`,
  /// biblatex-sbl-ibid.tex:200; `\blx@refpatch@sect`, cmsendnotes.sty:121)
  /// are consumed. Repros `index-bib/blx_chicago_cbx_citecmd_undefined.tex`,
  /// `blx_loccittracker_internal_sbl.tex`, `blx_refpatch_sect_cmsendnotes.tex`.
  #[test]
  fn biblatex_chicago_notes_loads_its_cbx() {
    let tex = r"\documentclass{article}
\usepackage[notes,backend=biber]{biblatex-chicago}
\begin{document}
Text.\footnote{See \runcite{smith}; \headlessfullcite{jones}.} \Citetitle{smith}.
\makeatletter\blx@opt@loccittracker@false\blx@refpatch@sect{section}{}{1}\makeatother
\printshorthands
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<note"), "{xml}");
  }

  /// biblatex-cv.sty is raw-input on top of the binding, so its own
  /// `\highlightname` (biblatex-cv.sty:565) exists. Repro
  /// `index-bib/blx_variant_own_macro_cv.tex`.
  #[test]
  fn biblatex_cv_variant_overlay() {
    let tex = r"\documentclass{article}
\usepackage{biblatex-cv}
\highlightname{Doe}{Jon}{}{}
\begin{document}
Body.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Body."), "{xml}");
  }

  /// tex.web §1131: `$` inside a group opened within math (`$\bm{\hat{m}$} b`,
  /// kblocks-doc.tex:207) closes the group first; the math ends when the
  /// bounded argument group does, and stays nested in the paragraph.
  /// Repro `boxes-groups/mal_math_bm_group_close.tex`.
  #[test]
  fn math_end_inside_open_group_defers_to_group_end() {
    let tex = r"\documentclass{article}
\usepackage{bm}
\begin{document}
a $\bm{\hat{m}$} b

c $\bm{x}$ d
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<Math ").count(), 2, "{xml}");
    assert!(xml.contains("</Math> b"), "{xml}");
    assert!(!xml.contains("</p>\n<Math"), "{xml}");
  }

  /// A `\NewTCBListing` option value built from a substituted argument keeps
  /// its control-word boundaries (`\dots ii` stayed `\dotsii`; oxnotes-doc).
  #[test]
  fn tcb_listing_option_tokens_keep_cs_boundaries() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\NewTCBListing{egcite}{m}{listing side text,before lower={#1\par}}
\begin{document}
\begin{egcite}{\dots ii (Brussels, 1867--88), 367--8}
\cite[367--368]{key}
\end{egcite}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("\u{2026} ii (Brussels") || xml.contains("\u{2026}ii (Brussels"),
      "{xml}"
    );
  }

  /// beamerfontthememetropolis.sty:278-308 `\patchcmd`s `\beamer@subsection`
  /// and `\beamer@@frametitle`; the binding carries both bodies (never
  /// invoked) so the patches apply. Repro `loader/beamer_metropolis_min.tex`.
  #[test]
  fn beamer_metropolis_font_theme_patches_apply() {
    let tex = r"\documentclass[10pt]{beamer}
\usetheme{metropolis}
\begin{document}
\section{S}
\subsection{Sub}
\begin{frame}{Title}x\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("Patching"), "{stderr}");
    assert!(xml.contains("<subsection"), "{xml}");
  }

  /// aguplus.cls:524 probes `\@ifundefined{chapter}` right after its
  /// `\LoadClass{article}`; the kernel `\chapter` is retracted at `\LoadClass`
  /// return, not only at `\documentclass`. Control: book keeps chapters.
  /// Repro `loader/aguplus_figcaps.tex`.
  #[test]
  fn loadclass_return_retracts_kernel_chapter() {
    let tex = r"\documentclass[twoside,agupp]{aguplus}
\begin{document}
ok
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("ok"), "{xml}");
    let book = r"\documentclass{book}
\begin{document}
\chapter{One}
Text.
\end{document}
";
    let (stderr, xml) = convert(book, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<chapter"), "{xml}");
  }

  /// Perl Mouth.pm:98-117: an `at_letter` mouth saves and restores `@`'s
  /// catcode LOCALLY, so a package that `\input`s a file inside
  /// `\bgroup\catcode`\@0 … \egroup` (CoverPage.sty:60-70) gets `@` back as a
  /// letter after the group. Repro
  /// `macro-state/atletter_group_input_catcode_leak_coverpage.tex`.
  #[test]
  fn at_letter_mouth_keeps_group_catcode_undo() {
    let sty = r"\NeedsTeXFormat{LaTeX2e}
\ProvidesPackage{lxcatleak}
\bgroup
  \catcode`\@0
  \bgroup
    \def\article##1{\xdef\CP@ParseArg{##1}}%
    \input{lxcatleak.txt}%
  \egroup
\egroup
\define@key{cover}{title}{\gdef\CP@Title{#1}}
\endinput
";
    let txt = "@article{k,\n title = {Some Title}}\n";
    let tex = r"\documentclass{article}
\usepackage{keyval}
\usepackage{lxcatleak}
\begin{document}
\makeatletter\setkeys{cover}{title=T}\CP@Title\makeatother
\end{document}
";
    let (stderr, xml) = convert_files(tex, &[("lxcatleak.sty", sty), ("lxcatleak.txt", txt)]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(">T<") || xml.contains("T</p>"), "{xml}");
  }

  /// datetime.sty:181-188 `\newdateformat{name}{format}` defines `\name`
  /// (chetdoc `\mydate`); jmlr.cls:593 `\abovestrut` (pmlr-sample); ejpecp.cls:156
  /// `\BEMAIL` (sample).
  #[test]
  fn class_and_datetime_definitions_exist() {
    let tex = r"\documentclass{article}
\usepackage{datetime}
\newdateformat{mydate}{\THEYEAR-\THEMONTH-\THEDAY}
\begin{document}
\mydate Date: \formatdate{5}{9}{2026}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Date: 2026-9-5."), "{xml}");
    let jmlr = r"\documentclass{jmlr}
\title{T}\author{\Name{A}\Email{a@b}}
\begin{document}
\maketitle
\begin{tabular}{c}\abovestrut{2ex}x\belowstrut{1ex}\end{tabular}
\end{document}
";
    let (stderr, xml) = convert(jmlr, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<tabular"), "{xml}");
  }

  /// Internals raw packages/documents reach that the replacing bindings
  /// omitted (one witness manual each): lastpage `\lastpage@lastpage`,
  /// geometry `\Gm@lmargin`, amsmath `\tag@true`, hyperref `\HyPsd@AMSclassfix`
  /// and the dvips `\pdfmark`, colortbl `\therownum`, beamer's
  /// `\pgfpagesuselayout`, siunitx v3 `\siunitx_number_format:nN`, fourier's
  /// `\lefthand`.
  #[test]
  fn binding_internals_reached_by_raw_code() {
    let tex = r"\documentclass{article}
\usepackage{lastpage}
\usepackage{geometry}
\usepackage{amsmath}
\usepackage{hyperref}
\usepackage{colortbl}
\usepackage{siunitx}
\usepackage{fourier}
\makeatletter
\tag@true
\pdfmark[/ANN]{pdfmark=/OBJ,Raw={/_objdef {x} /type /stream}}
\HyPsd@AMSclassfix
\ExplSyntaxOn
\siunitx_number_format:nN {12.50} \l_tmpa_tl
\tl_set_eq:NN \lxnum \l_tmpa_tl
\ExplSyntaxOff
\begin{document}
Last \lastpage@lastpage; margin \the\Gm@lmargin; row \therownum; number \lxnum; hand \lefthand.
\makeatother
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Last ??; margin 0.0pt; row 0; number 12.50; hand"),
      "{xml}"
    );
    let beamer = r"\documentclass{beamer}
\pgfpagesuselayout{2 on 1}[a4paper]
\begin{document}
\begin{frame}x\end{frame}
\end{document}
";
    let (stderr, _) = convert(beamer, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }

  /// listings stores an undefined-yet colour instead of digesting it
  /// (callouts: `\lstset{backgroundcolor=\color{…}}` before xcolor loads), and
  /// its character-conversion internals exist for add-on styles
  /// (lstfiracode `\lst@CCPutMacro`). Repros
  /// `graphics-tikz/{callouts_lstset_color_eager,listings_CCPutMacro}.tex`.
  #[test]
  fn listings_deferred_colour_and_conversion_internals() {
    let tex = r#"\documentclass{article}
\usepackage{listings}
\lstset{backgroundcolor=\color{cyan!10}}
\usepackage{xcolor}
\makeatletter
\lst@CCPutMacro\lst@ProcessOther {"2D}{\lst@ttfamily{-{}}{-{}}}\@empty\z@\@empty
\makeatother
\begin{document}
\begin{lstlisting}
x = 1
\end{lstlisting}
\end{document}
"#;
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("data=\"eCA9IDE=\""), "{xml}"); // base64 of `x = 1`
  }

  /// `\only<handout>{…}` is discarded in presentation mode (beamerswitch.cls:
  /// 226 runs `\pgfpagesuselayout` with pgfpages unloaded there); overlay
  /// specs and beamer-mode specs still apply. seminar.cls:760 probes
  /// `\ps@fancy` (semsamp1/2). Repro `beamer-stubs/beamer_only_modespec.tex`.
  #[test]
  fn beamer_only_discards_other_mode_specs() {
    let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}
\only<handout>{\undefinedhandoutonly}
\only<handout:0| trans:0>{\undefinedhandoutonly}
\only<2->{Overlay.}
\only<beamer>{Beamer.}
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Overlay.") && xml.contains("Beamer."), "{xml}");
    let fancy = r"\documentclass{article}
\usepackage{fancyhdr}
\pagestyle{fancy}
\makeatletter\ifx\ps@fancy\@undefined MISSING\fi\makeatother
\begin{document}
Text.
\end{document}
";
    let (stderr, xml) = convert(fancy, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("MISSING"), "{xml}");
  }

  /// verbatim.sty:210-217: `\verbatiminput` of a file that does not exist
  /// is `\typeout{No file …}`, not an error (msc.tex:287, lnosuppl.tex:89).
  /// Repro `parameter-conditional/verbatiminput_missing_msc.tex`.
  #[test]
  fn verbatiminput_missing_file_is_not_an_error() {
    let tex = r"\documentclass{article}
\usepackage{verbatim}
\begin{document}
Before.
\verbatiminput{COPYRIGHT}
After.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(stderr.contains("No file COPYRIGHT"), "{stderr}");
    assert!(
      xml.contains("Before.") && xml.contains("After.") && !xml.contains("<ERROR"),
      "{xml}"
    );
  }

  /// lua-widow-control's user surface under the luatex profile (its Lua half
  /// cannot run: homework-demo-*, jwjournal-demo-cn, abntexto).
  #[test]
  fn lua_widow_control_surface() {
    let tex = r"\documentclass{article}
\usepackage{lua-widow-control}
\lwcsetup{emergencystretch=1em, draft=false}
\begin{document}
\iflwc on\else off\fi; \lwcdisable\iflwc on\else off\fi.
\end{document}
";
    let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("on; off."), "{xml}");
  }

  /// pict2e.sty:791 `\cbezier` (halloweenmath-man) and ejpecp.cls:467
  /// `\realmathbb` (ejpecp sample).
  #[test]
  fn pict2e_cbezier_cubic() {
    let tex = r"\documentclass{article}
\usepackage{pict2e}
\begin{document}
\setlength{\unitlength}{1pt}
\begin{picture}(40,20)
\cbezier(0,0)(10,20)(30,20)(40,0)
\end{picture}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // four point pairs → the post-processor's SVG `C` segment
    let points = xml
      .split("<bezier points=\"")
      .nth(1)
      .and_then(|r| r.split('"').next())
      .unwrap_or("");
    assert_eq!(points.split(' ').count(), 4, "{xml}");
    let ej = r"\documentclass{ejpecp}
\title{T}\author{A}
\begin{document}
\maketitle
$\realmathbb{R}$
\end{document}
";
    let (stderr, xml) = convert(ej, true);
    assert!(!stderr.contains("undefined:\\realmathbb"), "{stderr}");
    assert!(
      xml.contains("\u{211d}") || xml.contains("mathbb") || xml.contains("R<"),
      "{xml}"
    );
  }

  /// `\pgfmathparse{\l_x_dim}` reads the expl3 register as one name; the
  /// alphabetic-only scanner split it at `_` and read `\l` (pgf-interference:
  /// 200k warnings, 412 s). Repro `expl3/pgfmath_expl3_register_split.tex`.
  #[test]
  fn pgfmath_reads_expl3_register_names() {
    let tex = r"\documentclass{article}
\usepackage{tikz}
\ExplSyntaxOn
\dim_new:N \l_x_dim
\dim_set:Nn \l_x_dim { 3cm }
\NewDocumentCommand \showit {} { \pgfmathparse { \l_x_dim } RESULT=[\pgfmathresult] }
\ExplSyntaxOff
\begin{document}
\showit
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!stderr.contains("is not a register"), "{stderr}");
    assert!(xml.contains("RESULT=[85.35826]"), "{xml}");
  }

  /// A deferred math end (#196) fires once at its group's end and never
  /// re-defers: nicefrac's text-mode denominator `\nicefrac{1}{2$^{x}$}` puts
  /// the inner `$` two groups below the math frame (egpeirce-doc.tex:1831);
  /// the ender must not escape past the math frame and leak `<ltx:Math>`.
  /// Repro `boxes-groups/math_defer_nicefrac_dollar_leak.tex`.
  #[test]
  fn deferred_math_end_never_escapes_the_math_frame() {
    let tex = r"\documentclass{article}
\usepackage{nicefrac}
\begin{document}
X \nicefrac{1}{2$^{\textrm{16}}$} Y

Z $a$ W
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert!(!stderr.contains("malformed"), "{stderr}");
    assert!(error_count(&stderr) <= 2, "{stderr}");
    assert!(xml.contains(" W</p>") || xml.contains(" W\n"), "{xml}");
    assert!(!xml.contains("</p>\n<Math"), "{xml}");
  }

  /// latex.ltx keeps one FIFO `\@begindocumenthook`: a `#`-bearing raw
  /// `\AtBeginDocument` chunk registered AFTER a `#`-free one runs after it
  /// (the italian.ldf/verifica.cls shape), and one registered BEFORE runs
  /// before (the hep-paper shape, guarded separately). Repro
  /// `macro-state/begindocument_hook_fifo.tex`.
  #[test]
  fn begin_document_hooks_run_in_registration_order() {
    let tex = r"\documentclass{article}
\makeatletter
\def\lxorder{}
\AtBeginDocument{\g@addto@macro\lxorder{A}}
\AtBeginDocument{\newcommand\lxhashed[1]{#1}\g@addto@macro\lxorder{B}}
\AtBeginDocument{\g@addto@macro\lxorder{C}}
\makeatother
\begin{document}
Order: \lxorder; \lxhashed{ok}.
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Order: ABC; ok."), "{xml}");
  }

  /// Sweep-44 singles: fontspec's `\latinencoding` (textalpha-doc), CJK.sty's
  /// `\CJKspace`/`\CJKencfamily` (cjk-ko-doc, bxcjkjatype beamer), beamer
  /// `\subject` (shipunov), pdfmanagement `\pdfmanagement_add:nee` under
  /// `\DocumentMetadata` (zugferd), MnSymbol `\rcurvearrowse` (biblatex-apa6),
  /// biblatex `\printorigdate` (cms-notes-sample).
  #[test]
  fn sweep44_single_name_gaps() {
    let luatex = r"\documentclass{article}
\usepackage{fontspec}
\usepackage{MnSymbol}
\begin{document}
Enc: \latinencoding; $a \rcurvearrowse b$.
\end{document}
";
    let (stderr, xml) = convert_with(luatex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Enc: TU;") && xml.contains("\u{21b7}"),
      "{xml}"
    );
    let cjk = r"\documentclass{article}
\usepackage{CJK}
\begin{document}
\CJKspace\CJKencfamily[UTF8]{mj}{}\CJKnospace Text.
\end{document}
";
    let (stderr, xml) = convert(cjk, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Text."), "{xml}");
    let beamer = r"\documentclass{beamer}
\subject{S}
\begin{document}
\begin{frame}x\end{frame}
\end{document}
";
    let (stderr, _) = convert(beamer, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    let meta = r"\DocumentMetadata{}
\documentclass{article}
\ExplSyntaxOn
\pdfmanagement_add:nee {Catalog/AF}{}{x}
\pdfmanagement_add:nnx {Catalog}{AF}{\pdf_object_ref:n{zugferd/rechnung}}
\ExplSyntaxOff
\begin{document}
Meta.
\end{document}
";
    let (stderr, xml) = convert(meta, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Meta."), "{xml}");
  }

  /// Package state: newtxtext keeps its xpatch dependency so ltcmd's legacy
  /// `g` type exists for `\NewDocumentCommand\entry{m g}` (prtec.cls:316);
  /// psfrag allocates its `\newwrite\pfg@temp` (psfrag.sty:151) so psfragx's
  /// read/write streams do not collide; verbatim's terminator reaches the
  /// current `\end` macro (knowledge's scope areas). Repros
  /// `macro-state/{newtxtext_drops_xpatch_xparse_g_prtec,psfrag_missing_newwrite_pfx_already_exists,knowledge_scope_verbatim_no_pop}.tex`.
  #[test]
  fn package_state_prtec_psfragx_knowledge() {
    let prtec = r"\documentclass{article}
\usepackage{newtxtext}
\NewDocumentCommand{\entry}{m g}{[#1/\IfNoValueTF{#2}{NO}{#2}]}
\begin{document}
\entry{A} \entry{B}{C}
\end{document}
";
    let (stderr, xml) = convert(prtec, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[A/NO]") && xml.contains("[B/C]"), "{xml}");
    let psfrag = r"\documentclass{article}
\usepackage{psfrag}
\usepackage{psfragx}
\begin{document}
\copypfxfromto{article.cls}{out.pfx}
Done.
\end{document}
";
    let (stderr, xml) = convert(psfrag, true);
    assert!(!stderr.contains("already exists"), "{stderr}");
    assert!(xml.contains("Done."), "{xml}");
    let knowledge = r"\documentclass{article}
\usepackage[scope,silent]{knowledge}
\begin{document}
Before.
\begin{verbatim}
x = 1
\end{verbatim}
After.
\end{document}
";
    let (stderr, xml) = convert(knowledge, true);
    assert!(!stderr.contains("Not allowed to close"), "{stderr}");
    assert!(xml.contains("<verbatim") && xml.contains("After."), "{xml}");
    // Control: the kernel `{verbatim}` hands its terminator to the CURRENT
    // `\end` (latex.ltx:15438 `\@xverbatim`), so a hooked `\end` sees it
    // once, after the verbatim, and the rest of the `\end` line still reads.
    let hooked = r"\documentclass{article}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{verbatim}
x = 1
\end{verbatim} tail
\begin{center}c\end{center}
\end{document}
";
    let (stderr, xml) = convert(hooked, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("[E:verbatim]").count(), 1, "{xml}");
    assert!(xml.contains("[E:center]") && xml.contains("tail"), "{xml}");
    let vpos = xml.find("</verbatim>").unwrap();
    assert!(xml[vpos..].contains("[E:verbatim]"), "{xml}");
  }

  /// Control for the Gemini G3 frame-body `#`-halving (DIVERGENCES #198): a
  /// lone `#1` inside a non-fragile frame stays `#1` (leniency: real beamer
  /// rejects it), `##1` and `####1` both reach `\newcommand` as `#1`, a
  /// `[fragile]` frame is not halved, and a frame after the frame is unaffected.
  #[test]
  fn beamer_frame_single_hash_control() {
    let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}{One}
\newcommand\ha[1]{(a:#1)}\ha{x}
\newcommand\hb[1]{(b:##1)}\hb{y}
\newcommand\hc[1]{(c:####1)}\hc{z}
\end{frame}
\begin{frame}[fragile]{Two}
\newcommand\hd[1]{(d:#1)}\hd{w}
\end{frame}
\begin{frame}{Three}
Plain text.
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    for s in ["(a:x)", "(b:y)", "(c:z)", "(d:w)", "Plain text."] {
      assert!(xml.contains(s), "missing {s}: {xml}");
    }
  }

  /// \SetCatcodeRange and \lstloadaspects support (witness codebox-doc-en).
  #[test]
  fn luatex_catcoderange_and_listings_aspects() {
    let tex = r"\documentclass{article}
\usepackage{luatexbase}
\SetCatcodeRange{65}{90}{11}
\usepackage{listings}
\lstloadaspects{comments}
\begin{document}
Listing test.
\SetCatcodeRange{`A}{`Z}{12}\typeout{CAT:\the\catcode`Q:\the\catcode`q}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Listing test."), "{xml}");
    // ctablestack.sty:18: a real `\catcode` loop over the range, nothing else.
    assert!(stderr.contains("CAT:12:11"), "{stderr}");
  }

  /// \DeclareTCBListing invoked via bare macros inside \NewDocumentEnvironment
  /// (witness codebox-doc-en \begin{codeview} calling \codeviewaux).
  #[test]
  fn declare_tcb_listing_nested_in_document_environment() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings,xparse}
\DeclareTCBListing{mycodeaux}{m}{title={Title #1},listing only}
\NewDocumentEnvironment{mycode}{O{} m}
  {\mycodeaux{#2}}
  {\endmycodeaux}
\begin{document}
\begin{mycode}{My Title}
#include <stdio.h>
int main() { return 0; }
\end{mycode}
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("include"), "{xml}");
    assert!(xml.contains("stdio"), "{xml}");
  }

  /// unicode-math symbol table loading and ctex LuaTeX math letter hooks
  #[test]
  fn unicode_math_table_loading_and_ctex_hooks() {
    let tex = r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\cs_if_exist:NTF \__um_input_math_symbol_table: { \__um_input_math_symbol_table: } {}
\cs_if_exist:NTF \um_input_math_symbol_table: { \um_input_math_symbol_table: } {}
\cs_if_exist:NTF \__um_load_symbols: { \__um_load_symbols: } {}
\cs_if_exist:NTF \__um_switchto_literal: { \__um_switchto_literal: } {}
\ExplSyntaxOff
\begin{document}
Table hooks ok.
\end{document}
";
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Table hooks ok."), "{xml}");
  }
}

mod perfect_kernel_gemini {
  use super::perfect_kernel_batch46::{convert, error_count};

  /// lineno.sty manual surface (witness lineno/ulineno): \linenumberwidth,
  /// \bframesep, \bframerule, \linerefp, \linerefr, and bare \internallinenumbers
  /// inside \parbox.
  #[test]
  fn lineno_manual_surface() {
    let tex = r"\documentclass{article}
\usepackage{lineno}
\begin{document}
\setlength\linenumberwidth{1cm}
\setlength\bframesep{10pt}
\setlength\bframerule{1pt}
\linenumbers
First line.\linelabel{l1}
Second line references \lineref{l1}, offset \lineref[+1]{l1}, \linerefp[+2]{l1}, \linerefr[+3]{l1}.
\begin{center}
\fbox{\parbox{0.8\textwidth}{
  \internallinenumbers \resetlinenumber[13]
  Internal linenumbers in a box.
}}
\end{center}
\begin{bframe}Framed text.\end{bframe}
\begin{internallinenumbers}Environment block.\end{internallinenumbers}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("First line.") && xml.contains("Internal linenumbers in a box."),
      "{xml}"
    );
  }

  /// caption hook surface for class and extension package patches
  /// (witness shtthesis/shtthesis-user-guide via raw bicaption.sty):
  /// \caption@beginhook, \caption@endhook, \caption@LT@setup, \caption@dblarg,
  /// \captionsetup[type][subtype], and faithful \caption@ifundefined.
  #[test]
  fn caption_hook_surface_for_class_patches() {
    let tex = r"\documentclass{article}
\usepackage{caption}
\makeatletter
\g@addto@macro\caption@beginhook{\def\hook@ran{1}}
\g@addto@macro\caption@endhook{\def\hook@ended{1}}
\g@addto@macro\caption@LT@setup{\relax}
\caption@ifundefined\undefined@cmd{\def\undef@branch{1}}{\def\undef@branch{0}}
\caption@ifundefined\caption@beginhook{\def\def@branch{0}}{\def\def@branch{1}}
\caption@dblarg{\def\test@dblarg[#1]#2{#1:#2}}
\test@dblarg{My Title}
\captionsetup[figure][bi-second]{name=Figure}
\captionsetup*[table][bi-second]{name=Table}
\makeatother
\begin{document}
\begin{figure}
  \caption{Test caption}
\end{figure}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Test caption"), "{xml}");
  }

  /// babel-italian ISO compliance unit definition via deferred \AtBeginDocument
  /// (witness: verifica/example4.tex, example5.tex).
  #[test]
  fn babel_italian_iso_compliance_unit() {
    let tex = r"\documentclass{article}
\usepackage[italian]{babel}
\AtBeginDocument{
  \setISOcompliance
}
\begin{document}
$25\unit{m}$
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("25"), "{xml}");
  }

  /// \openout, \write, \closeout, then \input within the same run via VFS,
  /// including filenames formed by protected macros (witness: proof-at-the-end/proof-at-the-end_demo).
  #[test]
  fn openout_then_input_same_run() {
    let tex = r"\documentclass{article}
\usepackage{xparse}
\NewDocumentCommand\prefixMacro{m}{#1-vfs}
\newwrite\testout
\begin{document}
\immediate\openout\testout=\prefixMacro{\jobname}out.tex
\immediate\write\testout{Hello from VFS with protected macro}
\immediate\closeout\testout
\input{\prefixMacro{\jobname}out.tex}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Hello from VFS with protected macro"), "{xml}");
  }

  fn convert_env_args(tex: &str, extra: &[&str], envs: &[(&str, &str)]) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(
      std::path::Path::new(bin).is_file(),
      "binary not staged at {bin}"
    );
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let mut args = vec![
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ];
    args.extend_from_slice(extra);
    let mut cmd = std::process::Command::new(bin);
    cmd.args(&args).current_dir(workdir.path());
    for (k, v) in envs {
      cmd.env(k, v);
    }
    let output = cmd.output().expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
    (stderr, xml)
  }

  /// Spill-gated node_boxes sweep (K8 memory lever): sweeps run after spills
  /// (runs_spilled > 0) to reclaim stale node_boxes, keeping the map bounded
  /// without futile full-DOM traversals when nothing spilled.
  #[test]
  fn spill_gated_node_boxes_stays_bounded() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\newtcolorbox{cb}{colback=red!5,colframe=red!75!black,title=Boxed}
\newcount\ct \ct=0
\begin{document}
\loop\ifnum\ct<300
  \ifnum\numexpr\ct/5*5=\ct
    \section{Section \the\ct}
  \fi
  \begin{cb}Box \the\ct\ with some text $x_{\the\ct}$.\end{cb}
  \advance\ct by 1
\repeat
\end{document}
";
    let (stderr, xml) = convert_env_args(tex, &["--streaming", "--max-memory=768"], &[(
      "LXML_TRACE_NODE_BOXES",
      "1",
    )]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      stderr.contains("node_boxes sweep"),
      "expected trace output from spill-gated sweep:\n{stderr}"
    );
    assert!(
      stderr.contains("dropped:"),
      "expected dropped entries in trace:\n{stderr}"
    );
    assert_eq!(xml.matches("<picture").count(), 300, "{}", xml.len());
  }

  /// Native ctable binding: \ctable with keyvals, captions, tabular/tabularx,
  /// rule macros (\NN, \FL, \ML, \LL), and footnotes block (\tnote, \tmark).
  /// Witnesses: proofread/example, arXiv:2011.04706.
  #[test]
  fn ctable_native_table_with_caption() {
    let tex = r"\documentclass{article}
\usepackage{ctable}
\begin{document}
\ctable[
  botcap,
  caption=Sample Table with Ctable,
  label=tab:sample,
  pos=htbp,
  width=80mm,
]{ccc}{
  \tnote[a]{First footnote.}
  \tnote[b]{Second footnote.}
}{
  \FL
  Col 1 & Col 2 & Col 3 \ML
  A\tmark[a] & B & C\tmark[b] \NN
  D & E & F \LL
}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Sample Table with Ctable"), "{xml}");
    assert!(xml.contains("<table"), "{xml}");
    assert!(xml.contains("<tabular"), "{xml}");
    assert!(xml.contains("First footnote."), "{xml}");
  }

  /// Beamer frame body parameter halving (\def-collect level halving):
  /// non-fragile beamer frames collect the body inside \loop ... \def\beamer@doifinframe ... \repeat,
  /// requiring two levels of parameter-hash halving so that ####1 becomes #1 at definition time.
  /// Witnesses: beamer-theme-albi/beamer-theme-albi-doc, tuda-ci/DEMO-TUDaBeamer.
  #[test]
  fn beamer_frame_hash_halving() {
    let tex = r"\documentclass{beamer}
\usepackage{etoolbox}
\begin{document}
\begin{frame}{Hash Halving Test}
  \renewcommand*{\do}[1]{[X ####1 Y]}
  \docsvlist{a,b,c}
\end{frame}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[X a Y][X b Y][X c Y]"), "{xml}");
  }

  /// mdframed with block-level content (e.g. \printbibliography / \thebibliography)
  /// mid-subsection followed by sectioning commands:
  /// mdframed breaks paragraph before opening, chooses logical-block outside floats,
  /// permits auto-closing so backmatter can place at section level, and auto-closes
  /// gracefully without error.
  /// Witness: biblatex-juradiss/biblatex-juradiss.
  #[test]
  fn mdframed_block_bibliography_juradiss() {
    let tex = r"\documentclass{article}
\usepackage{mdframed}
\begin{document}
\section{A}
Intro text before the frame.
\begin{mdframed}
\begin{thebibliography}{9}\bibitem{x}An entry.\end{thebibliography}
\end{mdframed}
\subsection{B}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<bibliography"), "{xml}");
    assert!(xml.contains("<subsection"), "{xml}");
    assert!(xml.contains("An entry."), "{xml}");
  }

  /// mdframed retains support for in-float frames (arXiv 1907.05772) and nested frames
  /// (arXiv 1712.00062).
  #[test]
  fn mdframed_in_float_and_nested() {
    let tex = r"\documentclass{article}
\usepackage{mdframed}
\begin{document}
\begin{figure}
\begin{mdframed}
Framed float.
\end{mdframed}
\end{figure}
\begin{mdframed}
\begin{mdframed}
Nested frame.
\end{mdframed}
\end{mdframed}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Framed float."), "{xml}");
    assert!(xml.contains("Nested frame."), "{xml}");
  }

  /// gauss.sty `gmatrix` inside outer alignment environments (e.g. `alignat*`):
  /// opens the amsmath matrix natively without gullet delimited-scan failures,
  /// with row and column operations closing the inner matrix alignment cleanly.
  /// Witness: tools/perfect_kernel/repros/beamer-stubs/gauss_in_alignat.tex.
  #[test]
  fn gauss_gmatrix_in_alignat() {
    let tex = r"\documentclass{article}
\usepackage{amsmath,gauss}
\begin{document}
\begin{alignat*}1
A=\begin{gmatrix}[p]
 1 & 1 \\
 t & 2t
\rowops
 \add[-t]{0}{1}
\end{gmatrix}&\\
\end{alignat*}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<XMArray"), "{xml}");
    assert!(
      xml.contains("←") || xml.contains("&#8592;") || xml.contains("leftarrow"),
      "{xml}"
    );
  }

  /// listings self-terminating environments hand \end{lstlisting} to the current \end macro
  /// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn listings_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{listings}
\AfterEndEnvironment{lstlisting}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{lstlisting}
x = 1
\end{lstlisting} tail
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:lstlisting]"), "{xml}");
    assert!(xml.contains("[AFTER]"), "{xml}");
    assert!(xml.contains("tail"), "{xml}");
    assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:lstlisting]").count(), 1, "{xml}");
  }

  /// fancyvrb self-terminating environments hand \end{Verbatim}, \end{BVerbatim}, \end{LVerbatim}
  /// to current \end macro (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn fancyvrb_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{fancyvrb}
\AfterEndEnvironment{Verbatim}{[AFTER-V]}
\AfterEndEnvironment{BVerbatim}{[AFTER-B]}
\AfterEndEnvironment{LVerbatim}{[AFTER-L]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{Verbatim}
v = 1
\end{Verbatim}
\begin{BVerbatim}
b = 1
\end{BVerbatim}
\begin{LVerbatim}
l = 1
\end{LVerbatim}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:Verbatim]"), "{xml}");
    assert!(xml.contains("[AFTER-V]"), "{xml}");
    assert!(xml.contains("[E:BVerbatim]"), "{xml}");
    assert!(xml.contains("[AFTER-B]"), "{xml}");
    assert!(xml.contains("[E:LVerbatim]"), "{xml}");
    assert!(xml.contains("[AFTER-L]"), "{xml}");
    assert_eq!(xml.matches("[AFTER-V]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:Verbatim]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[AFTER-B]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:BVerbatim]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[AFTER-L]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:LVerbatim]").count(), 1, "{xml}");
  }

  /// minted self-terminating environments hand \end{minted} to current \end macro
  /// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn minted_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{minted}
\AfterEndEnvironment{minted}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{minted}{python}
x = 1
\end{minted} tail
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:minted]"), "{xml}");
    assert!(xml.contains("[AFTER]"), "{xml}");
    assert!(xml.contains("tail"), "{xml}");
    assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:minted]").count(), 1, "{xml}");
  }

  /// comment.sty self-terminating environments hand \end{comment} to current \end macro
  /// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn comment_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{comment}
\AfterEndEnvironment{comment}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{comment}
ignored
\end{comment} tail
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:comment]"), "{xml}");
    assert!(xml.contains("[AFTER]"), "{xml}");
    assert!(xml.contains("tail"), "{xml}");
    assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:comment]").count(), 1, "{xml}");
  }

  /// verbatim.sty self-terminating environments hand \end{verbatim} to current \end macro
  /// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn verbatim_sty_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{verbatim}
\AfterEndEnvironment{verbatim}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{verbatim}
x = 1
\end{verbatim}
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:verbatim]"), "{xml}");
    assert!(xml.contains("[AFTER]"), "{xml}");
    assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:verbatim]").count(), 1, "{xml}");
  }

  /// alltt self-terminating environments hand \end{alltt} to current \end macro
  /// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn alltt_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{alltt}
\AfterEndEnvironment{alltt}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{alltt}
x = 1
\end{alltt} tail
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:alltt]"), "{xml}");
    assert!(xml.contains("[AFTER]"), "{xml}");
    assert!(xml.contains("tail"), "{xml}");
    assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:alltt]").count(), 1, "{xml}");
  }

  /// tcolorbox dispListing self-terminating environments hand \end{dispListing} to current \end macro
  /// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
  #[test]
  fn tcolorbox_self_terminating_hands_to_end() {
    let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\usepackage{etoolbox}
\AfterEndEnvironment{dispListing}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{dispListing}
x = 1
\end{dispListing} tail
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("[E:dispListing]"), "{xml}");
    assert!(xml.contains("[AFTER]"), "{xml}");
    assert!(xml.contains("tail"), "{xml}");
    assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
    assert_eq!(xml.matches("[E:dispListing]").count(), 1, "{xml}");
  }
}



