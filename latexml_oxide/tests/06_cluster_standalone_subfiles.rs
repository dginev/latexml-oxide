//! Cluster regressions in `standalone` / `subfiles` / `import` multi-file
//! documents — the child-preamble and search-path seams.
//!
//! Split out of `06_cluster_regressions`; shares its helpers via
//! [`mod cluster`](cluster).

mod cluster;
use cluster::{convert_log, convert_log_includestyles, convert_to_xml, convert_xml_includestyles};

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
    // … `\subimport*` in the PREAMBLE of a plain article, where import.sty's
    // `{…}` is the ONLY bracket — the arm that makes `import_sty.rs`'s own
    // `activate_scope` falsifiable (every other route also crosses
    // standalone_sty's bracket, so deleting import's line stayed green) …
    "tests/cluster_regressions/subimport/index_import_preamble.tex",
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

  // The package half: two sibling children load DIFFERENT packages that define
  // the same macro. Hoisting a package's ordinary macros to global made the
  // second `\newcommand` a silent no-op, so sibling B rendered sibling A's body
  // — worse than Perl, which scopes both. Only conditionals are hoisted.
  let xml_pkg =
    convert_xml_includestyles("tests/cluster_regressions/subimport/index_pkg_siblings.tex");
  assert!(
    xml_pkg.contains("kidA FROM-A"),
    "#311: first sibling must use its own package's macro:\n{xml_pkg}"
  );
  assert!(
    xml_pkg.contains("kidB FROM-B"),
    "#311: second sibling must use ITS OWN package's macro, not the first \
     child's hoisted one:\n{xml_pkg}"
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
/// `import.sty`'s search-path scoping. `SEARCHPATHS` is now a group-scoped value
/// (Perl-faithful: default-local `AssignValue`), so the `{…}` wrapper each
/// `\subimport` opens reverts the path change at `}` — exactly as Perl does, with
/// no explicit save/restore stack. Without group-local paths, the second sibling
/// `\subimport{Chapter/}{…}` would concatenate `Chapter/` onto the first call's
/// still-mutated lead and search `Chapter/Chapter/…`. Witnesses: arXiv:2604.09744,
/// 2603.04457.
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

/// Issue #500: `\usepackage{standalone}` before `\usepackage{fancyvrb}` made
/// `\DefineVerbatimEnvironment` report `Error:undefined:\KV@do`; swapping the
/// two `\usepackage` lines made it go away.
///
/// `\KV@do` lives ONLY in raw `keyval.sty` (L31) — no LaTeXML binding defines
/// it; both engines get it from `InputDefinitions('keyval', noltxml => 1)`. The
/// `xkeyval` binding used to only *pretend* keyval was loaded
/// (`AssignValue('keyval.sty_loaded' => 1)`, Perl `xkeyval.sty.ltxml` L23),
/// which turns every later `\RequirePackage{keyval}` into a no-op — so raw
/// `fancyvrb.sty`'s `\FV@UseKeyValues` (L112-117) called a `\KV@do` that never
/// got defined. Rust reached it first because `standalone_sty.rs` carries the
/// beyond-Perl `RequirePackage!("xkeyval")` of real `standalone.sty` L107.
/// Fixed at the root: xkeyval now really loads keyval, as real `xkeyval.sty`
/// L39 does via the bundle's `keyval.tex` (OXIDIZED_DESIGN #95).
///
/// The options argument must be NON-empty — `{\n  }` tokenizes to one space —
/// or `\ifx\FV@KeyValues\@empty` short-circuits and `\KV@do` is never reached.
#[test]
fn keyval_internals_survive_xkeyval_preloading_it() {
  // The issue's MWE, verbatim: standalone (→ xkeyval) before fancyvrb.
  let log = convert_log("tests/cluster_regressions/standalone_fancyvrb_keyval.tex");
  assert!(
    !log.contains("KV@do"),
    "#500: \\DefineVerbatimEnvironment after `standalone` must not lose \
     keyval's \\KV@do:\n{log}"
  );
  assert!(
    convert_to_xml("tests/cluster_regressions/standalone_fancyvrb_keyval.tex")
      .contains("text in a custom verbatim"),
    "#500: the custom verbatim environment lost its body"
  );

  // The root, reachable without standalone — this half Perl 0.8.8 also gets
  // wrong (KNOWN_PERL_ERRORS #73), so it is an intentional surpass. Its options
  // are VALUED, which additionally drives keyval's key-lookup branch
  // (`\KV@split` → `\csname KV@FV@fontsize\endcsname`) rather than only
  // `\KV@do`'s existence.
  assert!(
    convert_to_xml("tests/cluster_regressions/xkeyval_fancyvrb_keyval.tex")
      .contains("text in a custom verbatim"),
    "#500: explicit \\usepackage{{xkeyval}} before fancyvrb must keep \\KV@do"
  );

  // The order the reporter found working must stay working: loading keyval
  // first and xkeyval second must still leave xkeyval's extended \setkeys in
  // charge (real xkeyval overrides keyval, never the reverse).
  assert!(
    convert_to_xml("tests/cluster_regressions/fancyvrb_standalone_keyval.tex")
      .contains("text in a custom verbatim"),
    "#500: the already-working `fancyvrb` then `standalone` order regressed"
  );
}

/// Issue #698: a `\includegraphics` inside a `\subimport*`-ed child whose file
/// lives in a SIBLING directory (reached via `../`) must record its graphic
/// `candidates` RELATIVE to the main file's directory — never as an absolute
/// filesystem path. `util::image::image_candidates` relativizes each hit
/// against `SOURCEDIRECTORY`; it used `strip_prefix`, which can only strip a
/// *descendant* prefix, so a sibling-dir graphic fell back to the raw absolute
/// path (`/mnt/g/…` in the report) and leaked verbatim into the HTML `data=`
/// URL, breaking every deployment. Perl stores `../gfx_asset/images/pic.svg`
/// (Util/Image.pm `pathname_relative` → `File::Spec->abs2rel`); the lexical
/// `abs2rel` helper already used for the kpsewhich branch produces the same.
#[test]
fn subimport_sibling_graphic_candidate_is_relative() {
  let xml = convert_to_xml("tests/cluster_regressions/subimport/gfx_main/index_gfx_sibling.tex");
  // The exact Perl-parity relative candidate for a sibling-dir asset.
  assert!(
    xml.contains("candidates=\"../gfx_asset/images/pic.svg\""),
    "#698: sibling-dir graphic must record a relative candidate \
     `../gfx_asset/images/pic.svg`:\n{xml}"
  );
  // The canary: an absolute candidate is the bug, machine-independent.
  assert!(
    !xml.contains("candidates=\"/"),
    "#698: graphic candidate must never be an absolute filesystem path \
     (it leaks into the output `data=`/`src=` URL):\n{xml}"
  );
}

/// Issue #697 / OXIDIZED_DESIGN #137 (surpass Perl): `\subimport*` with an
/// ABSOLUTE directory argument resolves in real LaTeX (pdflatex, verified) but
/// failed in BOTH LaTeXML engines — `\lx@append@path` concatenated the
/// absolute arg onto the lead search path (`<lead>//abs/…`), which never
/// resolved. An absolute arg is now used verbatim (mirroring `\lx@set@path`),
/// matching pdflatex. The path is generated at runtime, so this is driven
/// through a tempdir rather than a committed fixture.
#[test]
fn subimport_absolute_path_resolves_like_real_latex() {
  let dir = tempfile::tempdir().expect("tempdir");
  let child = dir.path().join("child");
  std::fs::create_dir_all(&child).expect("mk child");
  std::fs::write(
    child.join("index.tex"),
    "\\documentclass{article}\\begin{document}ABSOLUTEsubimportBODY\\end{document}",
  )
  .expect("write child");
  let main = dir.path().join("main.tex");
  std::fs::write(
    &main,
    format!(
      "\\documentclass{{article}}\\usepackage{{import}}\\usepackage{{standalone}}\
       \\begin{{document}}\\subimport*{{{}/}}{{index.tex}}\\end{{document}}",
      // Forward slashes: a Windows abs path (C:\Users\…) in TeX source would
      // tokenize \U, \d, … as control sequences (`\` is catcode 0). kpathsea
      // accepts `/` on Windows; no-op on Unix.
      child.to_string_lossy().replace('\\', "/")
    ),
  )
  .expect("write main");
  let src = main.to_str().expect("utf8 path");

  let log = convert_log(src);
  assert!(
    !log.contains("missing_file"),
    "#697: absolute-path \\subimport* must resolve (matches pdflatex):\n{log}"
  );
  assert!(
    convert_to_xml(src).contains("ABSOLUTEsubimportBODY"),
    "#697: the absolutely-subimported child body was lost"
  );
}
