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
  // Count inline `Error:<class>:` markers (parity_check.sh's lax pattern,
  // see feedback_strict_vs_lax_error_grep.md). Errors are emitted INLINE
  // within `(Building...Error:..)` envelopes, not at line starts.
  let n_errors = r
    .log
    .match_indices("Error:")
    .filter(|(i, _)| {
      let tail = &r.log.as_bytes()[*i + 6..];
      let n_class = tail.iter().take_while(|b| b.is_ascii_lowercase()).count();
      n_class > 0 && tail.get(n_class) == Some(&b':')
    })
    .count();
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

#[test]
fn cluster_nbsp_csname() { convert_clean("tests/cluster_regressions/nbsp_csname.tex"); }

#[test]
fn cluster_at_ifundefined() { convert_clean("tests/cluster_regressions/at_ifundefined.tex"); }

#[test]
fn cluster_setdec_dec() { convert_clean("tests/cluster_regressions/setdec_dec.tex"); }

#[test]
fn cluster_cite_uppercase() { convert_clean("tests/cluster_regressions/cite_uppercase.tex"); }

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
fn cluster_multiput_braced_pair() { convert_clean("tests/cluster_regressions/multiput_braced_pair.tex"); }

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
