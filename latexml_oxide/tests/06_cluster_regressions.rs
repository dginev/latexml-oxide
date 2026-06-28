//! Cluster-regression integration tests.
//!
//! Pins the surpass-Perl wins from the post-100k cluster work
//! (NBSP, @ifundefined, setdec/dec, \CITE) as 0-error.
//! If a future change re-introduces the cluster errors, CI fails
//! before the PR can land.
use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

fn convert_clean(source: &str) {
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
/// 0-error `convert_clean` cannot express).
fn convert_to_xml(source: &str) -> String {
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  r.result
    .unwrap_or_else(|| panic!("{source}: conversion produced no result"))
}

/// Convert and return the conversion log (for asserting the ABSENCE of a
/// Rust-only warning that `convert_clean` — which only counts `Error:` — misses).
fn convert_log(source: &str) -> String {
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
fn cluster_cmidrule_cline_let() { convert_clean("tests/cluster_regressions/cmidrule_cline_let.tex"); }

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
/// docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md (2026-06-26m/o).
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
/// docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md (2026-06-26v).
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
/// clear-AFTER-load attempt broke. See docs/ARXIV_PERFORMANCE.md.
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
