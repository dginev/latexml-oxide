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
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
    let mut args = vec!["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=110"];
    if raw {
      args.push("--preload=[rawstyles,rawclasses]latexml.sty");
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
    assert!(stderr.contains("Stray alignment"), "{stderr}");
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
    // Perl's shape: exactly one "Attempt to end mode" per list, no pop, no
    // cascade; the items are plain paragraphs until P38 gives raw
    // `\list`/`\@trivlist` list semantics.
    assert!(!stderr.contains("Attempt to close"), "{stderr}");
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    assert!(
      stderr.contains("Attempt to end mode internal_vertical"),
      "{stderr}"
    );
    assert!(xml.contains("<p>one</p>") && xml.contains("two"), "{xml}");
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
  fn convert_with_sty(tex: &str, sty_name: &str, sty_body: &str) -> (String, String) {
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
  use super::perfect_kernel_batch46::{convert, error_count};

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
    assert!(!xml.contains("ltx:ERROR") && !xml.contains("<ERROR"), "{xml}");
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
    assert!(!stderr.contains("hobby.code.tex is not implemented"), "{stderr}");
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
    assert!(xml.contains("G\n0</p>"), "{xml}");
  }

}
