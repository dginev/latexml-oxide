//! Cluster-regression integration tests.
//!
//! Pins the surpass-Perl wins from the post-100k cluster work
//! (NBSP, @ifundefined, setdec/dec, \CITE) as 0-error.
//! If a future change re-introduces the cluster errors, CI fails
//! before the PR can land.
use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

fn convert_clean(source: &str) {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    // Same contrib dispatcher the binaries install — without it,
    // contrib-provided bindings (mhchem, chemformula, …) resolve to
    // nothing in the test environment while working in production.
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  // Shared lax `Error:<class>:` counter — see util::test::error_count
  // (single source of truth for the signal-integrity pattern).
  let n_errors = latexml::util::test::error_count(&r.log);
  assert_eq!(
    n_errors, 0,
    "{source}: expected 0 errors but log contained {n_errors} Error:<class>: markers (status_code={})",
    r.status_code
  );
  assert!(
    r.status_code <= 1,
    "{source}: status_code {} (expected 0/1), status={:?}",
    r.status_code,
    r.status
  );
}

/// Convert and return the serialized XML (for structural assertions that the
/// 0-error `convert_clean` cannot express). STRICT: like `convert_clean`, this
/// asserts the conversion logged **zero** `Error:` markers — a structural test
/// that silently tolerates a conversion error is exactly the false-negative the
/// project's signal-integrity rule forbids. An input that is *supposed* to error
/// (a malformed / EOF-truncated specimen, parity with Perl) must use
/// `convert_expecting_errors`, which asserts the exact intended count.
fn convert_to_xml(source: &str) -> String { convert_expecting_errors(source, 0) }

/// Convert an input EXPECTED to emit exactly `n` soft `Error:` markers, returning
/// the serialized XML. `n == 0` is the strict/clean case (`convert_to_xml`); a
/// nonzero `n` is for an intentionally-malformed specimen whose error is the
/// correct, Perl-parity outcome. Mirrors `util::test`'s `INTENTIONALLY_FAILING`
/// contract: drift fails BOTH ways — *more* errors = a handling regression,
/// *fewer* = we silently stopped detecting the bad input — and a `Fatal:`
/// (status_code 3) is always a regression, since the point is graceful recovery.
fn convert_expecting_errors(source: &str, n: usize) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  // Shared lax `Error:<class>:` counter — see util::test::error_count.
  let n_errors = latexml::util::test::error_count(&r.log);
  assert_eq!(
    n_errors, n,
    "{source}: expected {n} error(s) but log contained {n_errors} Error:<class>: markers (status_code={})",
    r.status_code
  );
  assert!(
    r.status_code < 3,
    "{source}: conversion hit a Fatal (status_code={}) — must degrade gracefully",
    r.status_code
  );
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}

/// Convert AND run the post-processing pipeline, returning the post-processed
/// XML. `convert_to_xml` stops at the engine, so it cannot see anything
/// MakeBibliography/CrossRef do — a `<bibitem>` in its output came straight from
/// `\begin{thebibliography}`, not from an `ltx:bibentry` conversion. Use this
/// helper for post-stage regressions.
fn convert_and_post(source: &str) -> String { convert_and_post_opts(source, None) }

/// Like `convert_and_post` but with the `context` navigation TOC enabled — for
/// the upstream LaTeXML#2316 / arXiv-fork behavior where frontmatter
/// (abstract/acknowledgements/bibliography) joins the navigation TOC.
fn convert_and_post_navtoc(source: &str) -> String {
  convert_and_post_opts(source, Some("context"))
}

fn convert_and_post_opts(source: &str, navigationtoc: Option<&str>) -> String {
  let xml = convert_to_xml(source);
  // No `stylesheet`: the assertions are about MakeBibliography, so stop at the
  // post-processed ltx XML rather than running XSLT into HTML.
  let opts = latexml::post::PostOptions {
    pmml: false,
    cmml: false,
    keep_xmath: false,
    stylesheet: None,
    destination: None,
    source_directory: Some("tests/cluster_regressions"),
    site_directory: None,
    search_paths: &[],
    nodefaultresources: true,
    css_files: &[],
    js_files: &[],
    noinvisibletimes: false,
    mathtex: false,
    navigationtoc,
    schemadocs: false,
    split: false,
    split_xpath: None,
    split_naming: None,
    xslt_parameters: &[],
    graphics_svg_threshold_kb: 0,
    graphicimages: false,
    timestamp: None,
    icon: None,
    whatsout: latexml_post::extract::Whatsout::default(),
  };
  latexml::post::run_post_processing(&xml, &opts)
}

/// Convert and return the conversion log (for asserting the ABSENCE of a
/// Rust-only warning that `convert_clean` — which only counts `Error:` — misses).
fn convert_log(source: &str) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  r.log
}

#[test]
fn cluster_nbsp_csname() { convert_clean("tests/cluster_regressions/nbsp_csname.tex"); }

#[test]
fn cluster_at_ifundefined() { convert_clean("tests/cluster_regressions/at_ifundefined.tex"); }

#[test]
fn cluster_setdec_dec() { convert_clean("tests/cluster_regressions/setdec_dec.tex"); }

#[test]
fn cluster_cite_uppercase() { convert_clean("tests/cluster_regressions/cite_uppercase.tex"); }

/// `\let\cline\cmidrule` (a common booktabs idiom) must NOT create a
/// `\cmidrule`->`\cline`->`\cmidrule` infinite expansion. LaTeXML's booktabs
/// binding defines `\cmidrule` via `\cline`, so the `\let` would loop until the
/// 8M-conditional IfLimit fatal unless `\cmidrule` routes through a private
/// saved `\cline` (`booktabs_sty.rs` `\ltx@saved@cline`). Shared with Perl
/// LaTeXML (which hangs); Rust surpasses. Witnesses: arXiv 2506.23179, 2511.17056.
#[test]
fn cluster_cmidrule_cline_let() {
  convert_clean("tests/cluster_regressions/cmidrule_cline_let.tex");
}

/// fvextra's `breakanywhere=true` installs a recursive char-by-char break
/// scanner that measures every character by boxing a line-prefix. In our
/// engine that recursed through `predigest_box_contents_in_mode` and grew the
/// gullet pushback until the 650000 `Timeout/PushbackLimit` Fatal — where Perl
/// converts cleanly. The `fvextra_sty` binding routes the breaking
/// line-processor to the non-breaking one (line wrapping is a PDF-visual
/// concern with no HTML semantics), so the verbatim completes with the
/// `font="typewriter"` styling preserved. Drove 119/121 fatal papers in the
/// sandbox-arxiv-2605 corpus (witness arXiv 2605.01024).
#[test]
fn cluster_fvextra_breakanywhere() {
  convert_clean("tests/cluster_regressions/fvextra_breakanywhere.tex");
}

/// An unbound class (->OmniBus) whose `.bbl` `\bibitem[\protect\citeauthoryear…]`
/// side-loads natbib must not leave a body `\citep` looping. The side-load runs
/// inside the `thebibliography` group, so natbib's `\citep` would be popped on
/// `\end{thebibliography}` and revert to its (now `sty_loaded`) `def_autoload`
/// trigger, whose already-loaded re-emit then loops to the token limit. Fixed by
/// hoisting the side-loaded package's defs to global (`\lx@late@usepackage`,
/// omnibus_cls.rs). Witness: arXiv 2209.11799 (200s TokenLimit fatal -> 1s/0err).
#[test]
fn cluster_omnibus_natbib_bbl_sideload() {
  convert_clean("tests/cluster_regressions/omnibus_natbib_bbl_sideload.tex");
}

/// A bare `\url` at end-of-input previously panicked: `\url`'s reader did
/// `read_token()?.unwrap()` and the `None` (input exhausted) hit the `.unwrap()`.
/// Real TeX raises a clean "Emergency stop" ("File ended while scanning use of
/// \url"); now `read_token_required` emits that parity Error and the macro
/// degrades (closes its group) instead of crashing. Guards the whole
/// `read_token_required` family (hyperref/url.sty `\url`, `\path`, amscd `\cd@`,
/// `\textfont`). Witnesses: 1401.5000, 1502.05051, 2204.10457. The specimen
/// truncates `\url` at EOF, so the ONE intentional `expected:` Error (input
/// ended while scanning use of `\url`) is the correct outcome — Perl emits the
/// same. `convert_expecting_errors(…, 1)` asserts EXACTLY that error (not merely
/// "non-empty output"): a drift to 0 means we stopped detecting the truncation,
/// >1 or a Fatal means the graceful recovery regressed.
#[test]
fn cluster_url_at_eof_no_panic() {
  let xml = convert_expecting_errors("tests/cluster_regressions/url_eof_no_panic.tex", 1);
  assert!(
    !xml.is_empty(),
    "url-at-EOF conversion produced empty output"
  );
}

/// Twemoji-style csname construction with accent macros (`\'`, `\^`, `\~`)
/// and `\textquoteright` apostrophe — must produce 0 errors after the
/// csname-stream soft-substitute fixes for `\lx@applyaccent`, the canonical
/// `\text…` primitives, and the NFSS `\<encoding>\i`/`\j` glyphs.
/// Pinned by stage-1..3 of the 100k warning corpus (arXiv:2603.22193,
/// 2603.23433, 2604.20621 — twemoji St. Barthélemy / Côte d'Ivoire / São Tomé).
#[test]
fn cluster_csname_accent() { convert_clean("tests/cluster_regressions/csname_accent.tex"); }

/// Legacy `\documentstyle[…]{amsart}` (LaTeX 2.09 compat) must auto-load
/// the AmS-TeX `\Sb` / `\Sp` substack environments via
/// `RequirePackage('amstex') if LookupValue('2.09_COMPATIBILITY')`.
/// Witnesses: arXiv:alg-geom9208004, arXiv:alg-geom9202004.
#[test]
fn cluster_amstex_2_09_sb() { convert_clean("tests/cluster_regressions/amstex_2_09_sb.tex"); }

/// AmSTeX `\input amstex` + `\documentstyle{amsppt}` papers must
/// stub `\vspace` / `\hspace` / `\scriptsize` / other LaTeX2e
/// typesetting CSes as no-ops (the AmSTeX pool path doesn't load
/// latex_constructs.rs). Witnesses: arXiv:funct-an9211012,
/// funct-an9211013, funct-an9211011, funct-an9312004.
#[test]
fn cluster_amsppt_vspace() { convert_clean("tests/cluster_regressions/amsppt_vspace.tex"); }

/// Picture-environment `\multiput(x,{y})` with the second coordinate
/// braced. Pair parameter reader must look through BEGIN…END groups
/// before reading the float. Witnesses: arXiv:hep-th9610147,
/// hep-th9703142.
#[test]
fn cluster_multiput_braced_pair() {
  convert_clean("tests/cluster_regressions/multiput_braced_pair.tex");
}

/// `\thechapter` autoload from `omnibus_cls.rs` must autoload the
/// `book.cls` BINDING, not `book.sty`. The obsolete `book.sty` shim
/// in TeXLive fires `\LoadClass{book}` immediately — by the time
/// `\thechapter` triggers (inside the document body), we're past
/// the preamble and `\LoadClass`'s preamble guard errors. Perl
/// avoids this by using `DefAutoload('thechapter', 'book.cls.ltxml')`
/// (cls extension, not sty). Witness: arXiv:2602.10407.
#[test]
fn cluster_omnibus_chapter_book_autoload() {
  convert_clean("tests/cluster_regressions/omnibus_chapter_book_autoload.tex");
}

/// Tolerant `Pair` parameter reader: malformed `(3.2,3,8)` (three
/// comma-separated values where Pair expects two) must consume the
/// trailing `,8` silently so the next Pair argument can read its `(`.
/// Mirrors Perl `ReadPair`'s `readUntil(',')`/`readUntil(')')`.
/// Witness: arXiv:physics/9709007.
#[test]
fn cluster_pair_tolerant_trailing() {
  convert_clean("tests/cluster_regressions/pair_tolerant_trailing.tex");
}

/// `\newpsobject{name}{old}{keyval}` must dynamically define
/// `\<name>` as a forwarder to `\<old>[<keyval>]`. Earlier stub
/// no-op'd, leaving the defined CS undefined. Mirrors Perl
/// `pstricks_support.sty.ltxml` L849-861. Witness:
/// arXiv:physics/9710028 (10 errors → 0 with this fix).
#[test]
fn cluster_newpsobject_forward() {
  convert_clean("tests/cluster_regressions/newpsobject_forward.tex");
}

/// JHEP.cls override of `\href` must use `Semiverbatim Semiverbatim`
/// (NOT hyperref's `HyperVerbatim {}`) so the BODY arg's `^`/`_`
/// are neutralized to OTHER catcode and don't fire `script_handler`
/// when digested in math mode. Affects all `\@spires`-style journal
/// citation macros (`\am`, `\ap`, `\np`, `\pl`, …). Mirrors Perl
/// `JHEP.cls.ltxml` L133-136. Witness: arXiv:2602.22473.
#[test]
fn cluster_jhep_href_semiverbatim() {
  convert_clean("tests/cluster_regressions/jhep_href_semiverbatim.tex");
}

/// The broad `^S\d+` prune sweep (`Document::prune_dangling_split_xmrefs`)
/// must NOT drop a `\Pr` (`\lx@dual` content-arm) ARGUMENT ref for
/// section-numbered aligned equations — that emitted a malformed
/// `apply(probability)` with no operand (silent content-MathML corruption).
/// The operand-protection guard keeps the ref (dangling rather than dropped,
/// closer to Perl which resolves it). See
/// docs/parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md (2026-06-26m/o).
/// A comma-list LEFT of a conditional bar parses with `|` binding to the LAST
/// item (Perl): `a,b|c` → `list@(a, conditional@(b, c))`, `a,b,c|d` →
/// `list@(a, b, conditional@(c, d))`, `x|y,z` → `conditional@(x, list@(y, z))`.
/// Previously `a,b|c` was UNPARSED — the root of the Class-B dangling-XMRef
/// witness (aligned `\Pr(s_A,s_B|\Omega)` arg failed to parse). The grammar rule
/// `statements punct statement vertbar statements => vertbar_modifier_listlhs`
/// fixes it; this asserts the exact Perl-matching tree shapes.
#[test]
fn cluster_comma_list_conditional() {
  let xml = convert_to_xml("tests/cluster_regressions/comma_list_conditional.tex");
  for expected in [
    "list@(a, conditional@(b, c))",
    "list@(a, b, conditional@(c, d))",
    "conditional@(x, list@(y, z))",
  ] {
    assert!(
      xml.contains(expected),
      "expected math text {expected:?} not found (comma-list conditional regressed)"
    );
  }
}

/// A `\quad`-separated formulae sequence whose first item is a
/// comma-list-left-of-relation (built by `distribute_list_relation`, which makes
/// a dual with a relation-`Apply` presentation, not an `XMWrap`) must NOT strand a
/// keyless bare `<XMRef/>` when a further `\quad` formula extends it. This was the
/// dominant `expected:id` "Missing idref" cluster (~370 papers). The Wrap-
/// presentation guard on the formulae/list extend paths fixes it. See
/// docs/parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md (2026-06-26v).
#[test]
fn cluster_formulae_distribute_no_bare_ref() {
  let xml = convert_to_xml("tests/cluster_regressions/formulae_distribute_no_bare_ref.tex");
  // A bare `<XMRef/>` (no idref) is the "Missing idref" symptom.
  let collapsed: String = xml.split_whitespace().collect::<Vec<_>>().join("");
  assert!(
    !collapsed.contains("<XMRef/>"),
    "keyless bare <XMRef/> present — distribute/formulae extend stranded a ref"
  );
}

/// A bare bigop as a `/`-fraction numerator (`\partial/\partial t`, Leibniz
/// partial-derivative notation) must PARSE — previously `ltx_math_unparsed`
/// (Rust-only; Perl: `partial-differential / partial-differential@(t)`). The
/// divide-scoped grammar rule `any_bigop divide term` fixes it without disturbing
/// the apply case (`\partial t`) or `\partial \times B`. See SYNC_STATUS.
#[test]
fn cluster_partial_over_partial() {
  let xml = convert_to_xml("tests/cluster_regressions/partial_over_partial.tex");
  // The \partial/\partial t formula must parse (no unparsed marker) and match Perl.
  assert!(
    !xml.contains("ltx_math_unparsed"),
    "\\partial/\\partial t left unparsed (bare-bigop fraction regressed)"
  );
  assert!(
    xml.contains("partial-differential / partial-differential"),
    "expected Perl-matching content text for \\partial/\\partial t not found"
  );
}

#[test]
fn cluster_xmref_pr_arg_not_dropped() {
  let xml = convert_to_xml("tests/cluster_regressions/xmref_pr_arg_not_dropped.tex");
  assert!(
    xml.contains(r#"meaning="probability""#),
    "probability operator missing from output"
  );
  // The probability XMApp must retain an operand: a bare
  // `<XMTok meaning="probability"/>` immediately followed by `</XMApp>`
  // (whitespace-insensitive) is the malformed/corrupted form we guard against.
  let collapsed: String = xml.split_whitespace().collect::<Vec<_>>().join("");
  assert!(
    !collapsed.contains(r#"meaning="probability"/></XMApp>"#),
    "malformed apply(probability) with no operand — content-arm arg ref was dropped"
  );
}

/// An eqnarray reading a `\def`-ized `\arraycolsep` (a plain macro, not a length
/// register) must NOT emit the Rust-only `expected:register` warning — Perl's
/// `LookupDimension` reads the macro body silently (verified same-host: Perl
/// 0.8.8 is silent; Rust used to warn 1×). Fixed by `state::lookup_dimension_cs`.
/// See docs/SYNC_STATUS.md.
#[test]
fn cluster_eqnarray_arraycolsep_macro_no_register_warning() {
  let log = convert_log("tests/cluster_regressions/eqnarray_arraycolsep_macro.tex");
  assert!(
    !log.contains("is not a register"),
    "spurious expected:register warning on a \\def-ized \\arraycolsep (LookupDimension regressed):\n{log}"
  );
}

/// Same as above for the `cases` package `numcases` environment (Perl
/// cases.sty.ltxml L82 also reads `\arraycolsep` via `LookupDimension`). A
/// `\def`-ized `\arraycolsep` must not produce the Rust-only `expected:register`
/// warning. See docs/SYNC_STATUS.md.
#[test]
fn cluster_numcases_arraycolsep_macro_no_register_warning() {
  let log = convert_log("tests/cluster_regressions/numcases_arraycolsep_macro.tex");
  assert!(
    !log.contains("is not a register"),
    "spurious expected:register warning on a \\def-ized \\arraycolsep in numcases:\n{log}"
  );
}

/// floatflt `floatingfigure` must compute the `width` percentage from its
/// `{Dimension}` arg (Perl `toPercent`: `int(100*dim/\textwidth)`). The args are
/// only on the BEGIN whatsit (after_digest_begin); the prior code read them in
/// `after_digest` (args=None) → `width="0%"`. Default \textwidth=345pt + a 3cm
/// figure → `width="24%"` (matches Perl 0.8.8). See docs/SYNC_STATUS.md.
#[test]
fn cluster_floatflt_pctwidth() {
  let xml = convert_to_xml("tests/cluster_regressions/floatflt_pctwidth.tex");
  assert!(
    xml.contains(r#"width="24%""#),
    "floatflt floatingfigure width != 24% (pctwidth/args regressed)"
  );
  assert!(
    !xml.contains(r#"width="0%""#),
    "floatflt floatingfigure width=\"0%\" — Dimension arg not read (after_digest args=None)"
  );
}

/// Same fix for the `floatfig` package: a 4cm figure → `width="32%"`.
#[test]
fn cluster_floatfig_pctwidth() {
  let xml = convert_to_xml("tests/cluster_regressions/floatfig_pctwidth.tex");
  assert!(
    xml.contains(r#"width="32%""#),
    "floatfig floatingfigure width != 32% (pctwidth/args regressed)"
  );
}

/// The arXiv IMS journal class (`arximspdf`/`arxstspdf`, used by Annals of
/// Probability/Statistics — aop/aos) must convert with 0 errors AND preserve
/// frontmatter metadata via the standard `\lx@add@*` API. Neither Perl LaTeXML nor
/// Rust bound this self-contained ~3000-line class, so papers cascaded into dozens
/// of undefined errors (`\b*` structured bib, `{barticle}`, `\operatorname`/`\tfrac`,
/// plain-TeX `\matrix`); the binding loads `article` + defines the IMS macros.
/// Surpasses Perl (which fails outright — both engines lack the class). Witness
/// cluster: 0910.0069 + 15 aop/aos papers. See docs/SYNC_STATUS.md.
#[test]
fn cluster_arximspdf_imsart() {
  convert_clean("tests/cluster_regressions/arximspdf_imsart.tex");
  let xml = convert_to_xml("tests/cluster_regressions/arximspdf_imsart.tex");
  // Frontmatter metadata preserved (standard frontmatter API).
  assert!(xml.contains("A Sample IMS Paper"), "title metadata missing");
  assert!(
    xml.contains("Doe"),
    "author (creator/personname) metadata missing"
  );
  assert!(xml.contains("probability"), "keywords metadata missing");
  // Structured \b* bibliography passes through as readable text.
  assert!(
    xml.contains("Smith") && xml.contains("On examples"),
    "structured \\b* bibliography content missing"
  );
}

/// A plain DefMath symbol (`\rightarrowfill`, a DefMath ARROW) used in TEXT mode
/// must NOT emit the Rust-only `unexpected:mode` "should only appear in math mode"
/// warning. Perl (Package.pm:1304) adds the requireMath beforeDigest only for
/// `requireMath => 1` bindings; Rust's `transfer_common_constructor_options` added
/// it unconditionally for every DefMath (broad over-emission; 0802.3360 Rust 3 /
/// Perl 0). See docs/SYNC_STATUS.md.
#[test]
fn cluster_defmath_textmode_no_mode_warning() {
  let log = convert_log("tests/cluster_regressions/defmath_textmode_no_mode_warning.tex");
  assert!(
    !log.contains("should only appear in math mode"),
    "spurious unexpected:mode warning for a DefMath symbol in text mode (requireMath over-applied):\n{log}"
  );
}

/// A `feynmp` (Feynman-diagram, MetaPost) document must convert with 0 errors —
/// feynmp shares feynmf's macros but had no Rust binding, so `\fmf{...label=$$}`
/// cascaded into `expected:$` display-math errors and `{fmfgraph*}`/`\fmfleft`/…
/// were undefined (witness 1003.1620: Rust 28 / Perl 0). The feynmp binding +
/// shared diagram-macro stubs absorb them. See docs/SYNC_STATUS.md.
#[test]
fn cluster_feynmp_fmf() { convert_clean("tests/cluster_regressions/feynmp_fmf.tex"); }

/// An UNBOUND journal class (`sn-jnl`, `wlpeerj`, `sagej`, Wiley, …) falls back
/// to the OmniBus class, whose lazy natbib autoload triggers (`\citep`/`\citet`/
/// `\citeyear`/…) must load natbib EXACTLY ONCE and resolve to natbib's real
/// definition. The hand-rolled OmniBus autoload (require_package → re-emit, no
/// clear) re-fired its own stub on every re-emit — fully RE-loading natbib each
/// iteration until the wall-clock watchdog (~60s+ digest hang). This was the
/// dominant slow/timeout cluster in the arXiv perf testbed (~50 sn-jnl + Wiley/
/// sagej/wlpeerj papers; witness 2603.06884: 90s digest → fatal timeout). Routing
/// through the canonical loop-safe `def_autoload` (clear trigger globally BEFORE
/// the load, hoist natbib's fresh defs to global, then re-emit) fixes the hang
/// while keeping `\citep` defined — the 1403.6801 (wlpeerj) regression that the
/// clear-AFTER-load attempt broke. See docs/performance/ARXIV_PERFORMANCE.md.
#[test]
fn cluster_omnibus_natbib_autoload_no_reload_loop() {
  let src = "tests/cluster_regressions/omnibus_natbib_autoload.tex";
  // Completes (no hang/timeout) and renders the citations — natbib's real
  // \citep/\citet resolved, producing the `ltx_cite` citation groups.
  let html = convert_to_xml(src);
  assert!(
    html.contains("ltx_cite"),
    "OmniBus natbib autoload: citations did not resolve to natbib's \\citep/\\citet \
     (expected an ltx_cite group in the output):\n{html}"
  );
  // The cite trigger must NOT have reverted to undefined after natbib loaded
  // (the clear-after-load failure mode, 1403.6801).
  let log = convert_log(src);
  let undef_cite = log
    .lines()
    .any(|l| l.contains("undefined") && l.contains("cite"));
  assert!(
    !undef_cite,
    "OmniBus natbib autoload: a cite trigger reverted to undefined after the load:\n{log}"
  );
}

/// The mhchem stub must NOT clobber an author's own `\cf` ("cf.") macro.
/// `\cf`/`\cee` are mhchem LEGACY (`version < 4`) commands; real mhchem
/// resolves the default version to 4 and leaves them undefined, so Perl
/// (raw-load) lets `\newcommand{\cf}` succeed as text. Defining them
/// unconditionally made `\newcommand` error "already defined" and left
/// `\cf` an `\ensuremath` math macro, so "cf." text leaked into math mode
/// ("Script _ can only appear in math mode" → `<ltx:XMTok>` cascade).
/// Mirrors mhchem.sty L3430 `\int_compare:nT { version < 4 }`. Witness:
/// arXiv:1901.08894 (chemformula + revtex4-1): 1002 errors / Fatal → 0.
#[test]
fn cluster_mhchem_cf_author_macro() {
  convert_clean("tests/cluster_regressions/mhchem_cf_author_macro.tex");
}

/// The flagship raw-load guard: \ce{H2O}/\ce{SO4^2-} must convert cleanly
/// through the real mhchem.sty + expl3 pipeline (PR_READINESS review — the
/// chemistry corpus had no fixture at all).
#[test]
fn cluster_mhchem_ce_subscripts() {
  convert_clean("tests/cluster_regressions/mhchem_ce_subscripts.tex");
}

/// Multi-level `theindex` (`\item`/`\subitem`/`\subsubitem`) must build nested
/// `<ltx:indexlist>`/`<ltx:indexentry>` cleanly. Requires (1) `Tag('ltx:indexentry',
/// autoClose=>1)` — Perl `latex_constructs.pool.ltxml` L4477 — so a new entry
/// auto-closes its open sibling and indexlist unwinds its entry children; and (2)
/// the theindex `beforeDigestEnd` must RETURN the digested `\index@done` whatsit so
/// it is constructed and unwinds the trailing indexphrase/indexlist. Without these
/// the builder errors "indexentry isn't allowed in indexentry" / "Closing ltx:index
/// whose descendents do not auto-close". Witness: arXiv:1205.0533 (102 errors /
/// Fatal → 1, the residual `\hyperpage` shared with Perl).
#[test]
fn cluster_theindex_nested_autoclose() {
  convert_clean("tests/cluster_regressions/theindex_nested_autoclose.tex");
}

/// Convert with the ar5iv profile preloaded — the production route that sets
/// `bibconfig=bbl,bib` PROGRAMMATICALLY (`ar5iv_sty.rs`). It cannot be set
/// from TeX source: `\usepackage[bibconfig={bbl,bib}]{latexml}` naive-splits
/// at the comma in BOTH engines (Perl `TrimmedCommaList` is not brace-aware),
/// leaving `['bbl']`.
fn convert_to_xml_ar5iv(source: &str) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    preload: Some(vec!["ar5iv.sty".to_string()]),
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}

/// bbl/bib precedence matrix for `\lx@ifusebbl` (latex_constructs.rs) — the
/// decision seam behind `\bibliography`. The clauses are arbitrary tokens, so
/// marker text pins WHICH phase was chosen without running the full BibTeX
/// pipeline. Covers the cb8b648784 fallback (bbl-first config + no .bbl on
/// disk → use the real .bib) and Perl's first-phase-only rule.
#[test]
fn cluster_bbl_bib_precedence() {
  // Default config ['bib','bbl']: refs.bib AND <jobname>.bbl both exist —
  // the bib phase is first and all bibs exist → BIB wins.
  let x = convert_to_xml("tests/cluster_regressions/bblbib/both.tex");
  assert!(
    x.contains("BIBCHOSEN") && !x.contains("BBLCHOSEN"),
    "default config with both files should choose bib, got:\n{x}"
  );
  // Default config, requested norefs.bib is MISSING but <jobname>.bbl exists
  // → falls to the bbl clause (Perl: "Couldn't find all bib files").
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bblwins.tex");
  assert!(
    x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "default config with missing .bib should choose bbl, got:\n{x}"
  );
  // nobibtex config ['bbl'] with <jobname>.bbl on disk → BBL wins,
  // even though refs.bib also exists.
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bblfirst.tex");
  assert!(
    x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "nobibtex config with .bbl present should choose bbl, got:\n{x}"
  );
  // nobibtex config ['bbl'] and NO <jobname>.bbl: Perl's first-phase-only
  // rule — no 'bib' phase configured, so NEITHER clause fires (empty +
  // Info:expected:bbl), not a spurious empty bibliography.
  let x = convert_to_xml("tests/cluster_regressions/bblbib/bblnone.tex");
  assert!(
    !x.contains("BBLCHOSEN") && !x.contains("BIBCHOSEN"),
    "nobibtex config without .bbl should choose neither, got:\n{x}"
  );
  // ar5iv profile (bibconfig=bbl,bib) but NO <jobname>.bbl: falls through to
  // the configured bib phase because refs.bib exists (cb8b648784; witness
  // 2605.16562 — refs.bib and no .bbl under the ar5iv fleet profile).
  let x = convert_to_xml_ar5iv("tests/cluster_regressions/bblbib/bblfallback.tex");
  assert!(
    x.contains("BIBCHOSEN") && !x.contains("BBLCHOSEN"),
    "ar5iv bbl-first config without .bbl should fall back to bib, got:\n{x}"
  );
}

/// Convert with the contrib bindings dispatched (biblatex lives in
/// latexml_contrib) and return the serialized XML.
fn convert_to_xml_contrib(source: &str) -> String {
  // Raise the RSS fuse to the harness cap (9 GB): these hand-written helpers
  // drive `Converter` directly, bypassing `latexml_test_single`, so without
  // this they run under the low production default and a full-file
  // `--test-threads=2` run trips a false `MemoryBudget` cascade once enough
  // conversions are in flight. See util::test::init_test_rss_cap.
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}

/// biblatex author-year support (ar5iv-bindings PRs #20/#21 + repair
/// 0911aec): style=apa documents with a biber .bbl get "Surname, Year"
/// labels, one schema-valid role-tagged <ltx:tags> per bibitem, and the
/// three citation families; style=numeric documents keep sequential
/// labels, core [ ] brackets, and plain-\cite fallbacks (multicite keys
/// comma-joined).
#[test]
fn cluster_biblatex_authoryear() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/ay.tex");
  // Structured tags with author/year roles (single-author, 2-author "&",
  // 3+-author "et al." short form vs full list, prefix-name surname).
  assert!(
    x.contains(r#"<tag role="year">2020</tag>"#),
    "year tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Smith</tag>"#),
    "authors tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="refnum">Smith (2020)</tag>"#),
    "refnum tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Jones &amp; Brown</tag>"#),
    "2-author tag missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Adams et al.</tag>"#),
    "et-al short form missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="fullauthors">Adams, Baker &amp; Clark</tag>"#),
    "fullauthors missing:\n{x}"
  );
  assert!(
    x.contains(r#"<tag role="authors">Berg</tag>"#),
    "prefix-name surname missing:\n{x}"
  );
  // Citation families: parenthetical vs textual vs bare, with show= specs.
  assert!(
    x.contains("citemacro_citep"),
    "parenthetical cite class missing:\n{x}"
  );
  assert!(
    x.contains("citemacro_citet"),
    "textual cite class missing:\n{x}"
  );
  assert!(
    x.contains(r#"show="Authors Phrase1YearPhrase2""#),
    "textual show spec missing:\n{x}"
  );
  assert!(
    x.contains(r#"show="FullAuthorsPhrase1Year""#),
    "starred full-author show missing:\n{x}"
  );
  // Multicite: two bibrefs inside one cite, "; "-joined.
  assert!(
    x.contains(r#"bibrefs="smith2020""#) && x.contains(r#"bibrefs="jones2019""#),
    "multicite per-group bibrefs missing:\n{x}"
  );
  // arxiv-readability#10 / ar5iv-bindings#4: \parencite[see][]{key} — a
  // present-but-EMPTY second optional must NOT demote the prenote to a
  // postnote ("(see Smith, 2020)", never "(Smith, 2020, see)").
  assert!(
    x.matches("(see ").count() >= 2,
    "issue-4 prenote missing:\n{x}"
  );
  assert!(
    !x.contains(", see)"),
    "issue-4 prenote demoted to postnote:\n{x}"
  );

  let x = convert_to_xml_contrib("tests/cluster_regressions/biblatex_ay/num.tex");
  // Numeric style: sequential labels, NO author-year relabeling, and the
  // fallback \cite path (keys preserved; multicite keys comma-joined).
  assert!(
    x.contains(r#"bibrefs="smith2020""#),
    "numeric fallback lost keys:\n{x}"
  );
  assert!(
    x.contains(r#"bibrefs="smith2020,jones2019""#),
    "numeric multicite keys not comma-joined:\n{x}"
  );
  assert!(
    !x.contains("Smith, 2020"),
    "numeric doc must not get author-year labels:\n{x}"
  );
  assert!(
    !x.contains(r#"role="fullauthors""#),
    "numeric doc must not get author-year tags:\n{x}"
  );
}

/// Upstream LaTeXML #2837: `\hdotsfor[]{N}` spans N alignment columns (the
/// dots row gets N cells, `\hdots & … & \hdots`), instead of piling N
/// `\hdots` into one cell. 3+3+3 cells in the first matrix + 2+2 in the
/// second = 13 mtds, 5 of them dots. The optional spacing arg is consumed
/// and ignored, matching upstream.
#[test]
fn cluster_hdotsfor_columns() {
  let x = convert_to_xml("tests/cluster_regressions/hdotsfor.tex");
  // The harness returns the pre-XSLT XML, so count XMath cells.
  let cells = x.matches("<XMCell").count() + x.matches("<mtd").count();
  assert_eq!(
    cells, 13,
    "\\hdotsfor must span its column count (9 + 4 cells), got:\n{x}"
  );
  assert_eq!(
    x.matches('\u{2026}').count(),
    5,
    "expected 3 + 2 dots cells, got:\n{x}"
  );
}

// ── Frontmatter class-binding fixtures ──────────────────────────────────────
// Structured, well-rendered author blocks across conference/journal classes.
// Witnesses are open arXiv HTML "front matter" reports; each fix is described
// in its binding. `<personname>` counts use the default-namespace serialization
// (bare tag names).

/// acmart `\author[F. Poli]{Federico Poli}`: the real class is `\author[2][]`
/// (optional running-head short name + full name). The name must render, and
/// the `[F. Poli]` optarg must NOT leak as a `[` creator. Witness 2405.08372.
#[test]
fn frontmatter_acmart_author_optarg() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_acmart_author_optarg.tex");
  assert!(
    x.contains("Federico Poli"),
    "acmart author name missing:\n{x}"
  );
  assert!(
    !x.contains("<personname>[") && !x.contains("<personname> ["),
    "acmart `[short]` optarg leaked as a bracket creator:\n{x}"
  );
}

/// IEEEtran `\author{\IEEEauthorblockN{…}\IEEEauthorblockA{…}\and …}`: each
/// block is one creator; the `1\textsuperscript{st}` ordinals must not be
/// misread as affiliation markers and drop every author. Witness 2602.05517.
#[test]
fn frontmatter_ieee_authorblock() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_authorblock.tex");
  assert!(
    x.contains("Alice Smith"),
    "IEEE authorblock author 1 missing:\n{x}"
  );
  assert!(
    x.contains("Bob Jones"),
    "IEEE authorblock author 2 missing:\n{x}"
  );
  assert!(
    x.matches("<personname>").count() >= 2,
    "IEEE authorblock must yield >=2 creators, got {}:\n{x}",
    x.matches("<personname>").count()
  );
}

/// IEEEtran `\IEEEmembership{Senior Member, IEEE}` inside a flat comma author
/// list must not become a phantom "Senior Member, IEEE" creator. Witness
/// 2508.00603.
#[test]
fn frontmatter_ieee_membership_no_phantom() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_membership.tex");
  assert!(
    x.contains("Alice Smith") && x.contains("Bob Jones"),
    "IEEE authors missing:\n{x}"
  );
  assert!(
    !x.contains("<personname>Senior Member") && !x.contains("<personname>Member, IEEE"),
    "IEEEmembership leaked as a phantom creator:\n{x}"
  );
}

/// Modern Interspeech.cls `\name[affiliation={1,*}]{First}{Last}` (2-arg): the
/// author renders as "First Last"; the `[affiliation=…]` optarg must not leak a
/// `[` creator or `\name`. Interspeech2024 resolves here by version-stripping.
/// Witness 2406.11727.
#[test]
fn frontmatter_interspeech2024_name() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_interspeech2024_name.tex");
  assert!(
    x.contains("Alice Smith"),
    "Interspeech author 1 missing:\n{x}"
  );
  assert!(
    x.contains("Bob Jones"),
    "Interspeech author 2 missing:\n{x}"
  );
  assert!(!x.contains("\\name"), "Interspeech `\\name` leaked:\n{x}");
  assert!(
    !x.contains("<personname>["),
    "Interspeech optarg leaked as bracket:\n{x}"
  );
}

/// czipreprint `\author[1]{…}` / `\author*[1,2]{…}` (starred = corresponding):
/// the star must be peeked via `\@ifstar`, not baked into the signature (which
/// would break the plain form → `]Name` leak). Witness 2508.00826.
#[test]
fn frontmatter_czipreprint_author_star() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_czipreprint_author.tex");
  assert!(
    x.contains("Alice Smith"),
    "czipreprint plain author missing:\n{x}"
  );
  assert!(
    x.contains("Bob Jones"),
    "czipreprint starred author missing:\n{x}"
  );
  assert!(
    !x.contains("<personname>]"),
    "czipreprint `[n]` optarg leaked a `]`:\n{x}"
  );
}

/// spconf.sty / INTERSPEECH2021.sty single-arg `\name{Author1$^1$, Author2$^2$}`
/// on `\documentclass{article}`: the name list becomes structured creators
/// rather than being stashed and dropped. Witness 2309.14838, 2405.13379.
#[test]
fn frontmatter_spconf_name() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_spconf_name.tex");
  assert!(x.contains("Alice Smith"), "spconf author 1 missing:\n{x}");
  assert!(x.contains("Bob Jones"), "spconf author 2 missing:\n{x}");
}

/// atlasdoc `\AtlasTitle{…}` / `\AtlasAbstract{…}` / `\AtlasOrcid[orcid]{Name}`:
/// the frontmatter macros of the (very large, unbound) ATLAS class must not leak
/// as literal text — the title/abstract render and the collaboration author
/// names show. Witness 2508.20929. (Full author-list-as-creators is out of scope
/// for this minimal frontmatter binding — the list is `\input` in the body.)
#[test]
fn frontmatter_atlasdoc_title() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_atlasdoc_title.tex");
  assert!(
    x.contains("heavy neutral leptons"),
    "AtlasTitle text missing:\n{x}"
  );
  assert!(
    !x.contains("\\AtlasTitle") && !x.contains("\\AtlasAbstract") && !x.contains("\\AtlasOrcid"),
    "Atlas frontmatter macro leaked as raw text:\n{x}"
  );
  assert!(x.contains("Aad"), "AtlasOrcid author name missing:\n{x}");
}

/// jmlr.cls `\author{ \Name{N} \Email{E} \\ ... \addr Affiliation }`: the
/// structured sub-macros must build one clean creator per `\Name` (name →
/// personname, `\Email` → contact[email], the trailing `\addr` block →
/// contact[affiliation]), not cram everything into one personname or split the
/// affiliation's commas into phantom "Foo"/"FL" authors. `\nametag` must not
/// leak. Witness 2410.16138.
#[test]
fn frontmatter_jmlr_structured_author() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_jmlr_name.tex");
  assert!(
    x.contains("<personname>Alice Smith</personname>"),
    "jmlr author 1 not a clean personname:\n{x}"
  );
  assert!(
    x.contains("<personname>Bob Jones</personname>"),
    "jmlr author 2 not a clean personname:\n{x}"
  );
  assert!(
    !x.contains("\\Name") && !x.contains("\\nametag") && !x.contains("\\addr"),
    "jmlr author sub-macro leaked as raw text:\n{x}"
  );
  assert!(
    x.contains("role=\"email\"") && x.contains("alice@example.edu"),
    "jmlr email not structured:\n{x}"
  );
  assert!(
    x.contains("role=\"affiliation\"") && x.contains("Department of Computer Science"),
    "jmlr affiliation not structured:\n{x}"
  );
  assert!(
    !x.contains("<personname>Foo") && !x.contains("<personname>FL"),
    "jmlr affiliation commas mis-split into phantom authors:\n{x}"
  );
}

/// MRM.cls (Wiley `\author[idx]{name}{orcid}` family): the author name renders,
/// the ORCID becomes a linked contact, `\address`/`\state`/`\country` don't leak
/// (`\state` is deliberately absent from OmniBus), and `\corres`/`\finfo` are
/// preserved as notes. Witness 2509.13644.
#[test]
fn frontmatter_mrm_author() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_mrm_author.tex");
  assert!(
    x.contains("<personname>Jakob Asslander*</personname>"),
    "MRM author name missing/unstructured:\n{x}"
  );
  assert!(
    !x.contains("\\state")
      && !x.contains("\\orcid")
      && !x.contains("\\corres")
      && !x.contains("\\authormark"),
    "MRM frontmatter macro leaked as raw text:\n{x}"
  );
  assert!(
    x.contains("role=\"orcid\"") && x.contains("0000-0003-2288-038X"),
    "MRM ORCID not a structured contact:\n{x}"
  );
  assert!(
    x.contains("Center for Biomedical Imaging"),
    "MRM affiliation content missing:\n{x}"
  );
}

/// subcaption loaded AFTER subfigure.sty must not clobber subfigure.sty's
/// self-contained `\subfigure[][]{}` macro with its own `{subfigure}[]{Dimension}`
/// environment. The two have incompatible contracts: the macro consumes a
/// balanced body and closes itself; the environment reads a `{Dimension}` and
/// opens a group closed only by `\end{subfigure}`. A document using the macro
/// form (`\subfigure[]{\includegraphics{...}}`) would then reparse it as an
/// environment — the `{\includegraphics{...}}` misread as a Dimension and the
/// group left open — swallowing the rest of the document (figures, sections,
/// bibliography). Real LaTeX's `\newenvironment` refuses to redefine an existing
/// `\subfigure`; we mirror that guard. Witness 2507.21938 (Perl times out on it).
#[test]
fn subcaption_does_not_clobber_subfigure_macro() {
  let x = convert_to_xml("tests/cluster_regressions/subcaption_subfigure_conflict.tex");
  // Content after the figure survived => no leaked, unclosed group.
  assert!(
    x.contains("must survive"),
    "subcaption clobbered subfigure.sty's \\subfigure; content after the figure was lost:\n{x}"
  );
  // The bibliography (document tail) is present => no truncation.
  assert!(
    x.contains("<bibitem") && x.contains("representative title"),
    "bibliography lost — the subfigure/subcaption clash leaked a group and truncated the document:\n{x}"
  );
}

/// Brace-less `\hphantom` immediately followed by `\endminipage` (the low-level
/// minipage primitive, no braces): upstream #2783's `\hphantom{}` grabs `#1`
/// unconditionally, so it would swallow `\endminipage` into the phantom's
/// `restricted_horizontal` frame — the minipage never closes and every element
/// after it (the "After" section and the bibliography) is absorbed and LOST.
/// The brace-guard (`\@ifnextchar\bgroup`) emits an empty phantom that consumes
/// nothing, so `\endminipage` closes its minipage in the ambient mode.
/// Witness 2004.10048 (`\minipage…\hphantom\endminipage`).
#[test]
fn hphantom_braceless_minipage_does_not_swallow_endminipage() {
  let x = convert_to_xml("tests/cluster_regressions/hphantom_braceless_minipage.tex");
  // Content after the figure survived => the minipage closed.
  assert!(
    x.contains("must survive"),
    "brace-less \\hphantom swallowed \\endminipage; content after the minipage was lost:\n{x}"
  );
  // The bibliography (last thing in the document) is present => no truncation.
  assert!(
    x.contains("<bibitem") && x.contains("representative title"),
    "bibliography lost — the minipage leaked and truncated the document:\n{x}"
  );
}

/// apacite spells its citation pre-note in ANGLE brackets:
/// `\cite<pre-note>[post-note]{key-list}` (apacite.sty L259-311 dispatch
/// `\@ifnextchar< {\@cite} {\@cite<>}`, L313-327 `\def\@cite<#1>`). Without that
/// form the kernel/natbib `\cite` takes the single token `<` as its whole key
/// list: the citation renders as a dangling `[<]`, the REAL keys are never cited
/// (so they are silently absent from the References) and `see>` leaks into the
/// body text. Witness 2605.10951 (`\cite<see>{Gangopadhyay02,Ferris25}`,
/// agujournal2019), 2606.16518, 2606.19048, 2606.21531, 2606.24563.
///
/// Guards BOTH halves of the fix: the pre-note form resolves its keys, AND the
/// pre-note-ABSENT case does not swallow a later `>`. The latter is why this
/// uses the real `OptionalAngled` parameter type rather than
/// `OptionalMatch:< OptionalUntil:>` — `Until` never checks for the OPENING
/// delimiter, so with no `<` it scanned to the next `>` anywhere downstream and
/// `\citeA{Gangopadhyay02} and $a > b$` reported the key as `b`.
#[test]
fn apacite_angled_prenote_cites_keys_and_does_not_swallow_gt() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/cite_angled_prenote/ap.tex");
  // `\cite<see>{Gangopadhyay02,Ferris25}` cites BOTH real keys, not `<`.
  assert!(
    x.contains("Gangopadhyay02,Ferris25"),
    "\\cite<see>{{...}} lost its keys (apacite angle-bracket pre-note):\n{x}"
  );
  assert!(
    !x.contains(r#"bibrefs="&lt;""#) && !x.contains(r#"bibrefs="<""#),
    "`<` was parsed as the citation key list:\n{x}"
  );
  // Pre-note absent + a later `>`: the cite keeps its key and the math survives.
  assert!(
    !x.contains(r#"bibrefs="b""#),
    "an absent angle pre-note swallowed the cite and the following `$a > b$`:\n{x}"
  );
}

/// Real sn-jnl.cls loads natbib for EVERY reference style (L1649/1652/1662/…:
/// `\usepackage[numbers,sort&compress]{natbib}` / `\usepackage[authoryear]{natbib}`),
/// but our binding `LoadClass!("OmniBus")`es — which short-circuits the
/// unbound-class dependency scan — and OmniBus only `def_autoload`s natbib off
/// `\citet`/`\citep`/`\citeyear`/…, deliberately NOT off `\cite` (the kernel
/// already defines it). So a paper citing solely via natbib's TWO-optional
/// `\cite[pre][post]{keys}` never triggered the autoload and the kernel's
/// single-optional `\cite[] Semiverbatim` read `[` as the whole key list — the
/// real keys were dropped (silently absent from the References) and `]{keys}`
/// leaked as body text. Witness 2605.23484 (sn-mathphys-num), 2606.10002
/// (sn-basic), 2606.10215, 2606.11534.
#[test]
fn sn_jnl_natbib_two_optional_cite_keeps_its_keys() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/sn_jnl_cite/sn.tex");
  assert!(
    x.contains("Melrose1980"),
    "sn-jnl `\\cite[e.g.][]{{Melrose1980}}` lost its key (natbib not loaded):\n{x}"
  );
  assert!(
    !x.contains(r#"bibrefs="[""#) && !x.contains(r#"bibrefs="&#91;""#),
    "`[` was parsed as the citation key list — natbib's two-optional \
     \\cite[pre][post]{{keys}} did not parse:\n{x}"
  );
  assert!(
    x.contains("Zhang2021"),
    "`\\cite[see][chap.~2]{{Zhang2021}}` lost its key:\n{x}"
  );
}

/// amsrefs writes the bibliography INTO the document —
/// `\begin{bibdiv}\begin{biblist}\bib{key}{article}{...}` — instead of into an
/// external `.bib`. The engine digests that correctly into
/// `ltx:biblist`/`ltx:bibentry` (see the `amsrefs_basic` structure test), but
/// upstream `MakeBibliography::getBibEntries` collects entries ONLY from
/// `getBibliographies()`, which resolves `//ltx:bibliography/@files` — an
/// amsrefs bibliography has no `@files`, so nothing is collected, and `process`
/// then executes its unconditional `removeNodes(//ltx:bibentry)`, deleting every
/// entry it never converted. The whole bibliography vanishes with ZERO errors:
/// empty References plus every `\cite` dangling.
///
/// PARITY with installed AND vendored Perl 0.8.8 (rev 51fea96a) — fixed here
/// rather than reproduced (OXIDIZED_DESIGN #57, KNOWN_PERL_ERRORS #49).
/// Witness 2605.01646 (AIPFa.tex; Perl: 0 bibitems / 81 dangling citations,
/// Rust now 23 / 0), 2605.00783, 2605.03852.
///
/// NOTE the structure test `amsrefs_basic` asserts only on the ENGINE's XML and
/// so never exercised MakeBibliography — which is exactly how this stayed
/// silent. This test runs the full pipeline.
#[test]
fn amsrefs_inline_bibliography_is_not_dropped() {
  let x = convert_and_post("tests/cluster_regressions/amsrefs_inline_bibliography.tex");
  // The inline entries became real bibitems (post ran and collected them).
  assert!(
    x.contains("<bibitem"),
    "amsrefs inline bibliography was dropped whole — no bibitem survived:\n{x}"
  );
  // Both entries, with their content, are present. NB amsrefs sentence-cases
  // titles ("On Examples" -> "On examples"), as `amsrefs_basic.xml` records.
  for needle in ["Beilinson", "Height pairing", "On examples", "Smith"] {
    assert!(
      x.contains(needle),
      "amsrefs entry content `{needle}` missing from the References:\n{x}"
    );
  }
  // No leftover uncollected bibentry (they were converted, not deleted).
  assert!(
    !x.contains("<bibentry"),
    "an ltx:bibentry survived unconverted:\n{x}"
  );
}

/// Loading `bibunits` — even without ever opening a `bibunit` environment —
/// made EVERY citation dangle. `\cite` runs bibunits' `\lx@bibunits@resetglobal`,
/// stamping `CITE_UNIT=bu0`, so the bibref asks for `BIBLABEL:bu0:<key>`; the
/// document's one `\bibliography` registers its bibitems under the default
/// `bibliography` list, and CrossRef searched the unit list ONLY. Witness
/// 2303.06077 (revtex4-2 + bibunits): 93 bibitems rendered, 93 keys dangling,
/// 0 links. Deleting the single `\usepackage{bibunits}` line resolves the cite,
/// which is the whole defect in one bisect.
#[test]
fn bibunits_cite_resolves_against_the_main_bibliography() {
  let x = convert_and_post("tests/cluster_regressions/bibunits_cite.tex");
  // The entry reaches the References either way — the defect is the LINK.
  assert!(
    x.contains("<bibitem"),
    "bibunits: the bibliography itself is missing:\n{x}"
  );
  assert!(
    !x.contains("ltx_missing_citation"),
    "bibunits: \\cite{{Smith2020}} dangles — CrossRef only searched the `bu0` \
     unit list and never fell back to the main `bibliography` list:\n{x}"
  );
}

/// Witness 2605.00490: a JabRef `.bib` self-declaring `% Encoding: Cp1252`.
/// MakeBibliography read it with `read_to_string`, which hard-errors on the
/// first non-UTF-8 byte, so the whole bibliography was dropped and the paper
/// rendered an empty References section with NO `Error:` — a silent, total
/// loss. Real `bibtex` 0.99d is 8-bit clean and Perl passes raw bytes through
/// (`Mouth.pm` L75-80).
///
/// This exercises the POST path (`convert_bib_file_to_xml`), which is where
/// the production failure actually happened; `pre_bibtex`'s own
/// `non_utf8_bib_file_is_read_not_rejected` covers the engine-side reader.
#[test]
fn non_utf8_bib_file_still_yields_a_bibliography() {
  let x = convert_and_post("tests/cluster_regressions/cp1252_bib.tex");
  assert!(
    x.contains("<bibitem"),
    "cp1252 .bib: the whole bibliography was dropped on a non-UTF-8 byte:\n{x}"
  );
  // The Latin-1 fallback is lossless byte -> char, so the accent survives to
  // the rendered entry rather than collapsing to U+FFFD. Only the SURNAME is
  // asserted: the fixture's `author = {Café, André}` is BibTeX's `Last, First`
  // form, so the style abbreviates the given name to `A.` ("A. Café").
  assert!(
    x.contains("Café"),
    "cp1252 .bib: the accented surname did not survive the decode:\n{x}"
  );
}

/// Witness 2605.11619: `\end{lstlisting}` preceded by content on the same line
/// (`</body></html> \end{lstlisting}`). Perl anchors the terminator regex at the
/// line start (listings.sty.ltxml L316), so the reader ran to EOF and swallowed
/// the rest of the document — Conclusion, `\bibliography` and appendix — with NO
/// error at all. Real `listings` terminates there (pdflatex renders the leading
/// text as the final listing line and continues), so both LaTeXML engines were
/// wrong vs the PDF. OXIDIZED_DESIGN #61 / KNOWN_PERL_ERRORS #51.
#[test]
fn inline_end_lstlisting_does_not_swallow_the_document() {
  let x = convert_to_xml("tests/cluster_regressions/lstlisting_inline_end.tex");
  assert!(
    x.contains("AFTER-THE-LISTING-MARKER"),
    "inline \\end{{lstlisting}}: the rest of the document was swallowed:\n{x}"
  );
  // The text before the terminator is still the listing's last line (pdflatex
  // renders exactly "hello world" there).
  assert!(
    x.contains("hello") && x.contains("world"),
    "inline \\end{{lstlisting}}: the listing body was lost:\n{x}"
  );
}

/// Issue #291: `\setcounter{tocdepth}{0}` in a `book` must restrict the
/// `\tableofcontents` to chapters only. The `\tableofcontents` constructor
/// already computes the correct `select="ltx:part | ltx:chapter | ..."`
/// attribute from `tocdepth`; the defect was purely in POST — CrossRef's
/// `gen_toc` ignored `select` (and the TOC's `lists`), hardcoding
/// `NORMAL_TOC_TYPES` + `inlist=="toc"`, so every level leaked into the ToC.
/// Faithful Perl: `CrossRef.pm::gentoc` L246-261 filters by the `select`-derived
/// `$types` and `inlist_match($lists, ...)`. Witness = the issue's MWE.
#[test]
fn tocdepth_select_restricts_the_toc() {
  // tocdepth=0 ⇒ chapters only; sections/subsubsections must be filtered out.
  let x = convert_and_post("tests/cluster_regressions/tocdepth0.tex");
  assert!(
    x.contains("ltx_tocentry_chapter"),
    "#291: chapters must appear in the ToC:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_section"),
    "#291: \\setcounter{{tocdepth}}{{0}} must drop sections from the ToC, but a \
     section tocentry is present (CrossRef gen_toc ignored the `select` attr):\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_subsubsection"),
    "#291: subsubsections must be dropped from a tocdepth=0 ToC:\n{x}"
  );

  // Guard against over-filtering: with the book default (tocdepth=2), sections
  // MUST still appear.
  let y = convert_and_post("tests/cluster_regressions/tocdepth_default.tex");
  assert!(
    y.contains("ltx_tocentry_chapter") && y.contains("ltx_tocentry_section"),
    "#291 guard: default tocdepth must still list chapters AND sections:\n{y}"
  );

  // Connected behavior — upstream LaTeXML#2316 / arXiv-fork: the abstract carries
  // inlist="toc" (so it shows in the navigation TOC), but the user
  // `\tableofcontents` emits a `select` that omits ltx:abstract, so the abstract
  // must be EXEMPT here. Honoring `select` (the #291 fix) is exactly what keeps
  // it out; before the fix `gen_toc` ignored `select`, so the abstract LEAKED
  // into `\tableofcontents` (`ltx_tocentry_abstract` present). `convert_and_post`
  // runs no navigation TOC, so the only TOC here is the user's.
  let z = convert_and_post("tests/cluster_regressions/toc_abstract_exempt.tex");
  assert!(
    z.contains("ltx_tocentry_section"),
    "#2316 guard: sections must appear in \\tableofcontents:\n{z}"
  );
  assert!(
    !z.contains("ltx_tocentry_abstract"),
    "#2316/#291 guard: the abstract (inlist=toc, for the nav TOC) must stay \
     EXEMPT from the user \\tableofcontents, whose `select` omits ltx:abstract:\n{z}"
  );
}

/// Upstream LaTeXML#2316 / arXiv-fork, the *inclusion* half: with the `context`
/// navigation TOC enabled, the abstract MUST appear in the nav TOC (screenreader
/// accessibility) — and, because its `select`-less nav TOC accepts all types
/// while the user `\tableofcontents` omits `ltx:abstract`, it must appear
/// **exactly once** (nav TOC only, not the inline one). This is the companion of
/// `tocdepth_select_restricts_the_toc`'s exempt half: both rely on `gen_toc`
/// honoring `select` (the #291 fix). Before that fix the abstract appeared twice
/// (leaked into `\tableofcontents`). Witness = the issue's frontmatter shape.
///
/// The navigation TOC now runs Perl's `format="context"` path
/// (`gen_toc_context`); on a single page that reduces to the same downward tree
/// as a normal TOC, so the count == 1 invariant holds. The multi-page breadcrumb
/// shape is covered by `context_toc_breadcrumb_across_split_pages`.
#[test]
fn nav_toc_includes_abstract_issue_2316() {
  let x = convert_and_post_navtoc("tests/cluster_regressions/toc_abstract_exempt.tex");
  let n = x.matches("ltx_tocentry_abstract").count();
  assert_eq!(
    n, 1,
    "#2316: the abstract must appear in the navigation TOC exactly once (nav \
     only — present for accessibility, exempt from \\tableofcontents), got {n}:\n{x}"
  );
  // Sections still populate both TOCs — sanity that the nav TOC was built at all.
  assert!(
    x.contains("ltx_tocentry_section"),
    "#2316: sections missing — navigation TOC was not generated:\n{x}"
  );
}

/// Issue #291 hardening: a *negative* `\setcounter{tocdepth}` must not panic.
/// `\tableofcontents` builds `select` from `tocdepth` by taking the first
/// `tocdepth + 1` section types; the old code cast that through `as usize`, so
/// `tocdepth = -1` (parts only) overflowed — a debug panic, and in release a
/// silently over-full ToC (`{-2}` listed everything). Faithful Perl
/// (`latex_constructs.pool.ltxml` L727-733) computes `0 .. $td` in signed space,
/// an empty range for negative `$td`. `tocdepth = -1` ⇒ the part stays, chapters
/// and sections are dropped. The conversion completing at all is the no-panic
/// guard (the fixture converts under the debug profile's overflow-checks).
#[test]
fn tocdepth_negative_is_parts_only_no_panic() {
  let x = convert_and_post("tests/cluster_regressions/tocdepth_negative.tex");
  assert!(
    x.contains("ltx_tocentry_part"),
    "#291: tocdepth=-1 must still list the part:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_chapter"),
    "#291: tocdepth=-1 (parts only) must drop chapters from the ToC:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_section"),
    "#291: tocdepth=-1 must drop sections from the ToC:\n{x}"
  );
}

/// Issue #291 latent fix: honoring the `<ltx:TOC>` `lists` attribute (`lof`)
/// also repairs `\listoffigures`/`\listoftables`, which the old hardcoded `"toc"`
/// bucket broke outright — `\listoffigures` listed a document *section* (an
/// `inlist="toc"` entry) instead of the figures (`inlist="lof"`). It must now
/// list exactly the figures and no section. Faithful Perl: `\listoffigures`
/// emits `<ltx:TOC lists='lof'>` and CrossRef draws only from that `inlist`
/// bucket. Guards a fix the #291 change delivered but did not otherwise cover.
#[test]
fn listoffigures_lists_figures_not_toc_sections() {
  let x = convert_and_post("tests/cluster_regressions/listoffigures.tex");
  let n = x.matches("ltx_tocentry_figure").count();
  assert_eq!(
    n, 2,
    "#291: \\listoffigures must list both figures (inlist=lof), got {n}:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_section"),
    "#291: \\listoffigures must NOT list document sections (inlist=toc) — the old \
     hardcoded `toc` bucket did exactly that:\n{x}"
  );
}

/// Issues #293 and #309: neither argument of a subimported child's
/// `\documentclass` is a package list, but the `\@standalone@documentclass[]{}`
/// intercept used to RequirePackage both in turn — the mandatory class name
/// (#293: `\documentclass{article}` → `missing_file:article`) and then the
/// optional class options (#309: `\documentclass[12pt]{article}` →
/// `missing_file:12pt`). Both are spurious; the child body always rendered.
///
/// The optional list is required only for a `{standalone}` child, and only for
/// options that `standalone.cls` itself turns into a package load — see
/// OXIDIZED_DESIGN #63 for why this diverges from Perl, which requires every
/// option of every class.
#[test]
fn standalone_subimport_documentclass_no_spurious_require() {
  // No optional args ⇒ nothing required (#293).
  let log = convert_log("tests/cluster_regressions/subimport/index.tex");
  assert!(
    !log.contains("missing_file") && !log.contains("Can't find binding or file for 'article"),
    "#293: \\documentclass{{article}} in a standalone child must NOT require the \
     class as a package (article.sty). Log:\n{log}"
  );
  let xml = convert_to_xml("tests/cluster_regressions/subimport/index.tex");
  assert!(
    xml.contains("this is a test in child document"),
    "#293: the subimported child body was lost:\n{xml}"
  );

  // #309's witness: `[12pt]{article}`. The class is not `standalone`, so its
  // options are ordinary class options and none of them is a package.
  let log_opt = convert_log("tests/cluster_regressions/subimport/index_opt.tex");
  assert!(
    !log_opt.contains("missing_file"),
    "#309: class options of a non-standalone child must NOT be RequirePackage'd \
     (`[12pt]` is a size option, not 12pt.sty):\n{log_opt}"
  );
  assert!(
    convert_to_xml("tests/cluster_regressions/subimport/index_opt.tex")
      .contains("child with class options"),
    "#309: the subimported child body was lost"
  );

  // Guard the other half — the reason the RequirePackage loop exists at all
  // (upstream LaTeXML#1432 wanted `\documentclass[tikz]{standalone}` to load
  // tikz). For a `{standalone}` child the package-loading options must still
  // load, while its non-package options stay quiet. The child *uses* varwidth,
  // so the load is observable: drop `varwidth` from CLASS_OPTION_PACKAGES and
  // this reports `Error:undefined:{varwidth}`.
  let log_sa = convert_log("tests/cluster_regressions/subimport/index_sa.tex");
  assert!(
    !log_sa.contains("undefined"),
    "#309 guard: a `{{standalone}}` child's package-loading options (here \
     `varwidth`, as `tikz` in LaTeXML#1432) must still be required:\n{log_sa}"
  );
  assert!(
    !log_sa.contains("missing_file"),
    "#309 guard: `border=2pt` is handled by standalone.cls, not a package:\n{log_sa}"
  );

  // Every one of these options also has a VALUED form — `\sa@boolorvalue`
  // takes `varwidth=5cm` and `tikz=true` just like the bare words
  // (standalone.sty L815-824) — and values may be brace groups containing
  // commas. Matching whole comma-split items missed all of that:
  // `[varwidth=5cm]` reported `Error:undefined:{varwidth}` while pdflatex was
  // clean. Reading the argument as `OptionalKeyVals` and matching on the KEY
  // is what makes these equivalent, so guard the valued form explicitly.
  let log_saval = convert_log("tests/cluster_regressions/subimport/index_saval.tex");
  assert!(
    !log_saval.contains("undefined"),
    "#309: a VALUED package option (`varwidth=5cm`) must load its package just \
     like the bare `varwidth`:\n{log_saval}"
  );
  assert!(
    !log_saval.contains("missing_file"),
    "#309: `border={{1pt 2pt}}` — a brace group with a space — is not a package:\n{log_saval}"
  );
  assert!(
    convert_to_xml("tests/cluster_regressions/subimport/index_saval.tex")
      .contains("VALUED class options"),
    "#309: the subimported child body was lost"
  );
}

/// Convert with `INCLUDE_STYLES` (the `--includestyles` / ar5iv mode) and return
/// the log: the only way to exercise a *raw-loaded* `.sty`, which is where the
/// TeX-local-vs-global split of issue #311 actually lives. A package with a Rust
/// binding installs its definitions globally already, so a bound package cannot
/// reproduce the bug.
fn convert_log_includestyles(source: &str) -> String {
  latexml::util::test::init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    include_styles: Some(true),
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  r.log
}

/// Issue #311: a package loaded while a group is open must still be defined
/// after that group closes.
///
/// A standalone subfile's preamble runs inside the group `standalone.sty.ltxml`
/// opens at the child's `\documentclass`, and LaTeXML — unlike the real package,
/// which *gobbles* the child preamble — actually executes it, so packages
/// genuinely load in there. A package is then split in half: its definitions are
/// frame-local, while the document-level hooks it registers are global. The
/// witness is `\documentclass[tikz]{standalone}`, where
/// `pgfcoreexternal.code.tex` L152 `\newif\ifpgf@external@grabshipout` is popped
/// with the child's group but its L171-179 `\AtEndDocument` survives to the
/// *parent's* `\end{document}` → `Error:undefined:\ifpgf@external@grabshipout`
/// at the very end of an otherwise complete conversion. Perl 0.8.8 emits the
/// identical error (KNOWN_PERL_ERRORS #55).
///
/// Fixed at the package-load seam (`content.rs::require_package` hoists the
/// load's meaning-delta past the enclosing group) rather than by removing the
/// group, so it holds for *every* way of ending up inside one — see
/// OXIDIZED_DESIGN #65 and the companion
/// `standalone_child_preamble_definitions_stay_scoped`, which pins the half that
/// must NOT leak.
///
/// `lx311demo.sty` is the mechanism in three lines and needs no host texmf tree,
/// so this arm runs everywhere; the tikz arms below are the real witness and are
/// gated on TeX Live.
#[test]
fn standalone_child_preamble_package_survives_the_subfile_group() {
  for index in [
    // plain \input …
    "tests/cluster_regressions/subimport/index_rawsty.tex",
    // … via import.sty, which adds a second group of its own …
    "tests/cluster_regressions/subimport/index_rawsty_subimport.tex",
    // … inside a group in the parent body …
    "tests/cluster_regressions/subimport/index_rawsty_grouped.tex",
    // … and a standalone child nested inside another standalone child, where
    // the load sits two brackets deep. Removing the two bindings' own groups
    // (the first fix tried for #311) left both of these last two broken — the
    // enclosing group was then simply somebody else's.
    "tests/cluster_regressions/subimport/index_rawsty_nested.tex",
  ] {
    let log = convert_log_includestyles(index);
    assert!(
      !log.contains("iflx@demo@flag"),
      "#311: a \\newif from a package loaded in the child's preamble must \
       survive to the parent's \\end{{document}}, where the package's \
       \\AtEndDocument hook reads it ({index}):\n{log}"
    );
    assert!(
      !log.contains("Error:") && !log.contains("Fatal:"),
      "#311: {index} must convert cleanly:\n{log}"
    );
  }
}

/// The other half of #311: hoisting a package load past the enclosing group must
/// NOT hoist the child's OWN preamble, which stays scoped to the child.
///
/// This is the regression the first attempt at #311 caused — it dropped the
/// groups instead of fixing the load — and every case here is silent wrong
/// content, not an error, so nothing else would have caught it. Multi-figure
/// papers are exactly the shape at risk: a directory of `standalone` figures
/// whose preambles reuse the same macro names with different bodies.
#[test]
fn standalone_child_preamble_definitions_stay_scoped() {
  // Two sibling children define \sharedmac and {sharedenv} differently; each
  // must render with its own.
  let xml = convert_to_xml("tests/cluster_regressions/subimport/index_macro_siblings.tex");
  assert!(
    xml.contains("SHAREDA") && xml.contains("SHAREDB"),
    "#311: each sibling child must use its OWN \\newcommand, not the first \
     child's leaked definition:\n{xml}"
  );
  assert!(
    xml.contains("[Aone A]") || xml.contains("[AoneA]"),
    "#311: first child's environment body:\n{xml}"
  );
  assert!(
    xml.contains("[Btwo B]") || xml.contains("[BtwoB]"),
    "#311: second child must use its OWN \\newenvironment:\n{xml}"
  );

  // A conditional the child flips in its preamble must not flip the parent's.
  let xml_flag = convert_to_xml("tests/cluster_regressions/subimport/index_flag.tex");
  assert!(
    xml_flag.contains("CHILDTRUE"),
    "#311: the child's own \\dupflagtrue must hold inside the child:\n{xml_flag}"
  );
  assert!(
    xml_flag.contains("PARENTFALSE"),
    "#311: the child's \\dupflagtrue must NOT leak into the parent's \
     same-named conditional:\n{xml_flag}"
  );
}

/// The #311 witness itself, and the second entry path. Gated: raw-loads
/// `pgfcoreexternal.code.tex` from the host texmf tree.
#[cfg_attr(
  not(building_with_texlive),
  ignore = "raw-loads pgfcoreexternal.code.tex from the host texmf tree"
)]
#[test]
fn standalone_child_tikz_survives_the_subfile_group() {
  for (index, how) in [
    // the reported witness: `\subimport*` + the `tikz` CLASS OPTION …
    (
      "tests/cluster_regressions/subimport/index_tikz.tex",
      "[tikz] class option",
    ),
    // … and plain `\input` + the child's OWN `\usepackage`. The two half-fixes
    // tried in the ticket each covered only one of these.
    (
      "tests/cluster_regressions/subimport/index_tikzpkg.tex",
      "child \\usepackage",
    ),
  ] {
    let log = convert_log(index);
    assert!(
      !log.contains("ifpgf@external@grabshipout"),
      "#311 ({how}): pgf's \\newif must survive the child's group:\n{log}"
    );
    assert!(
      !log.contains("Error:") && !log.contains("Fatal:"),
      "#311 ({how}): must convert cleanly:\n{log}"
    );
    // The error fired *after* the picture was built, so "no error" alone would
    // also pass on a fix that simply lost the child. Pin the content too.
    let xml = convert_to_xml(index);
    assert!(
      xml.contains("ltx:picture") || xml.contains("<svg"),
      "#311 ({how}): the child's tikzpicture must still render:\n{xml}"
    );
  }
}

/// `import.sty`'s search-path save/restore, which the #311 work nearly removed.
/// Perl's `{…}` wrapper around the input is what scopes `SEARCHPATHS` there;
/// Rust needs the explicit `\lx@save@paths`/`\lx@restore@paths` stack on top
/// because `set_search_paths` mutates a global. Without one or the other, the
/// second sibling `\subimport{Chapter/}{…}` concatenates `Chapter/` onto the
/// still-mutated lead and searches `Chapter/Chapter/…`. Witnesses:
/// arXiv:2604.09744, 2603.04457.
#[test]
fn subimport_sibling_calls_do_not_accumulate_search_paths() {
  let xml = convert_to_xml("tests/cluster_regressions/subimport/index_siblings.tex");
  assert!(
    xml.contains("first sibling body"),
    "first \\subimport lost:\n{xml}"
  );
  assert!(
    xml.contains("second sibling body"),
    "second \\subimport lost — the lead search path accumulated:\n{xml}"
  );
}

/// The boundary of the #311 hoist: a group the AUTHOR wrote is real, and real
/// LaTeX's verdict on it stands. `{\usepackage{amsthm}}` errors twice in
/// pdflatex — "Loading a class or package in a group", then "Undefined control
/// sequence" for `\theoremstyle` — and same-host Perl LaTeXML reports the
/// matching `Error:undefined:\theoremstyle`. Hoisting there would rescue an
/// authoring mistake and emit FEWER errors than Perl, which is a downgrade, not
/// a fix; only LaTeXML's own subfile brackets are hoisted past. The wall-clock
/// half of this (the stale-autoload runaway) is
/// `tests/100_stale_autoload_no_runaway.rs`.
#[test]
fn author_written_group_around_usepackage_still_loses_the_package() {
  // (b) the harder half: an author's group written INSIDE a subfile preamble.
  // The region is active there, so a "am I in a subfile?" test alone hoists it
  // too and Rust drops below Perl by an error. The scope name carries the
  // bracket's frame depth precisely to confine the region to its own level.
  let log_in_child = convert_log_includestyles(
    "tests/cluster_regressions/subimport/index_author_group_in_child.tex",
  );
  assert!(
    log_in_child.contains("iflx@demo@flag"),
    "#311: an author's group nested inside a subfile preamble must keep real \
     LaTeX's verdict — the package is lost, as in pdflatex and Perl:\n{log_in_child}"
  );

  let log = convert_log("tests/cluster_regressions/subimport/index_author_group.tex");
  assert!(
    log.contains("Error:undefined:\\theoremstyle"),
    "#311: the hoist must not reach a group the author wrote — Perl and \
     pdflatex both leave \\theoremstyle undefined here:\n{log}"
  );
}

/// KNOWN_PERL_ERRORS #56: `\includefrom`/`\subincludefrom` take a directory AND
/// a file name, but Perl's prototypes declare only one argument while their
/// bodies use `#3` — so the file is silently dropped: no error, no warning, no
/// content. Real `import.sty` routes all four through the same `\@doimport`.
#[test]
fn includefrom_takes_directory_and_file() {
  let xml = convert_to_xml("tests/cluster_regressions/subimport/index_includefrom.tex");
  assert!(
    xml.contains("includefrom body"),
    "\\includefrom{{dir}}{{file}} silently dropped the included file:\n{xml}"
  );
  // Both variants carry the typo, so both need pinning.
  assert!(
    xml.contains("subincludefrom body"),
    "\\subincludefrom{{dir}}{{file}} silently dropped the included file:\n{xml}"
  );
}
