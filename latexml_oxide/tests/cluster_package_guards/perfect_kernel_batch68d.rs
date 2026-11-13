//! Red/green guards for batch 68d: loader fixes from the `missing_file` study of 2605+2606 — a path-prefixed class is
//! its basename's binding, a braced package name loses its braces, a binding standing in for an author's file does
//! not answer `\IfFileExists` for it, and jfm's `lineno` option follows the paper's copy.
use latexml::util::test::{assert_element, convert_files_with, convert_files_with_status};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// `\documentclass{lib-acm/acmart}` with the class shipped in lib-acm/ loads the acmart binding as the request's own
/// file, as Perl does (Package.pm:2125-2128): no `missing_file` warning, no "missing file" in the verdict, no
/// dependency scan of the shipped copy (its xstring), and the binding's frontmatter (2605.00109, 2605.25901,
/// 2606.20651). Repro loader/subdir_class_is_its_binding.
#[test]
fn subdir_class_is_its_binding() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/subdir_class_is_its_binding.tex");
  let class = include_str!("../../../tools/perfect_kernel/repros/loader/lib-acm/acmart.cls");
  let (log, xml, status, code) =
    convert_files_with_status(tex, &[("lib-acm/acmart.cls", class)], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!((status.as_str(), code), ("No obvious problems", 0), "{log}");
  assert!(
    !log.contains("xstring"),
    "the shipped copy's xstring was scanned in: {log}"
  );
  assert!(
    xml.contains("<?latexml class=\"lib-acm/acmart\" options=\"sigconf\"?>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\">\n    <personname>A</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text class=\"ltx_affiliation_institution\" xml:id=\"id1\">U</text></contact>\n  </creator>",
  );
}

/// The same with the shipped copy failing if it is read raw, under each spelling of the basename: the binding is
/// loaded, not the file (`misc/ieeetran` → IEEEtran, 2105.02087; the batch 61 alternate guard's case, now silent).
#[test]
fn subdir_class_binding_is_not_read_raw() {
  const READ_RAW: &str =
    "\\ProvidesClass{stub}\\errmessage{the shipped class was read raw}\\LoadClass{article}\n";
  for (class, file) in [
    ("misc/ieeetran", "misc/ieeetran.cls"),
    ("sub/IEEEtran", "sub/IEEEtran.cls"),
  ] {
    let (log, xml, status, _) = convert_files_with_status(
      &format!(
        "\\documentclass{{{class}}}\n\\begin{{document}}\n\\begin{{IEEEkeywords}}\nstars\n\\end{{IEEEkeywords}}\n\\end{{document}}\n"
      ),
      &[(file, READ_RAW)],
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    assert_eq!(status, "No obvious problems", "{class}: {log}");
    assert_element(
      &xml,
      "keywords",
      &[],
      "<keywords name=\"Index Terms:\u{a0}\">stars</keywords>",
    );
  }
}

/// `\usepackage{{makecell}}`: latex.ltx strips a lone brace group at each delimited argument the name passes —
/// the list as `\@fileswith@pti@ns`'s #3 (latex.ltx:18714) and `\zap@space`'s #1 (:18695), each item as
/// `\reserved@b`'s ##1 (:18727-18734) and `\@onefilewithoptions`'s #1 (:18740), four in all (`{{{{{xcolor}}}}}`) —
/// and a class name two (`\documentclass{{article}}`); the PIs are spelled as Perl's, the text without its braces
/// and with its whitespace (2605.12897, 2606.09700). Repro loader/braced_package_name.
#[test]
fn braced_package_name_loses_its_braces() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/braced_package_name.tex");
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for pi in [
    "<?latexml class=\"article\"?>",
    "<?latexml package=\"amsmath,  amssymb \"?>",
    "<?latexml package=\"makecell\"?>",
    "<?latexml package=\"xcolor\"?>",
  ] {
    assert!(xml.contains(pi), "{pi}: {xml}");
  }
  assert_element(
    &xml,
    "text",
    &["color=\"#FF0000\""],
    "<text color=\"#FF0000\" xml:id=\"p1.1.1\">C</text>",
  );
  assert_element(
    &xml,
    "td",
    &["xml:id=\"p1.1.2.1.1.2.1.1.1\""],
    "<td align=\"center\" class=\"ltx_nopad_r\" xml:id=\"p1.1.2.1.1.2.1.1.1\">a</td>",
  );
  assert_element(
    &xml,
    "td",
    &["xml:id=\"p1.1.2.1.1.2.1.2.1\""],
    "<td align=\"center\" class=\"ltx_nopad_r\" xml:id=\"p1.1.2.1.1.2.1.2.1\">b</td>",
  );
}

/// A binding for a file TeX Live once distributed (`FORMER_TEX_LIVE`: html.sty, aastex.cls, psfig.sty, scrpage2.sty,
/// cropmark.sty) answers `\IfFileExists` as that file, which TeX in the paper's era found (ruling 2026-10-15;
/// 1710.04900's harvard.sty asks for html.sty); a stand-in for an author's file does not (jmlr2e.sty, colt2025.cls,
/// agujournal2019.cls, colm2025_conference.sty, arxiv.sty). Repro loader/iffileexists_former_tex_live.
#[test]
fn iffileexists_finds_former_tex_live_files() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/iffileexists_former_tex_live.tex");
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">FFFFF\nAAAAA</p>");
}

/// `\IfFileExists{arxiv.sty}` is false when the paper ships no arxiv.sty, as is a versioned conference style's; a TeX
/// Live file is found (2606.27281). Repro loader/iffileexists_author_file_stand_in.
#[test]
fn iffileexists_skips_author_file_stand_ins() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/iffileexists_author_file_stand_in.tex"
  );
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">ABSENT ABSENT FOUND</p>");
  // A shipped copy is found, and `\usepackage` then reads it under the binding (`localrawstyles`).
  let shipped = tex.replace("\\resa\\ \\resb\\ \\resc", "\\resa");
  let (log, xml) = convert_files_with(
    &shipped,
    &[("arxiv.sty", "\\ProvidesPackage{arxiv}\n")],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">FOUND</p>");
}

/// The registered `.cls`/`.sty`/`.tex` bindings, by file name (`arxiv.sty`).
fn registered_bindings() -> Vec<String> {
  latexml_package::binding_names()
    .iter()
    .chain(latexml_contrib::binding_names())
    .map(|(name, ext)| format!("{name}.{ext}"))
    .collect()
}

/// kpathsea's path for `file`, empty when it finds none.
fn kpsewhich(file: &str) -> String {
  std::process::Command::new("kpsewhich")
    .arg(file)
    .output()
    .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
    .unwrap_or_default()
}

/// Every stand-in for an author's file, every former TeX Live file and every LaTeXML-own file
/// (`latexml_core::binding::author_files`) is a registered binding TeX Live 2025 does not distribute (kpathsea finds
/// none of them; it finds `article.cls`, so it is there to ask), and no name is on two lists.
#[test]
fn author_file_stand_ins_are_bindings_outside_tex_live() {
  use latexml_core::binding::author_files::{AUTHOR_FILE_STAND_INS, FORMER_TEX_LIVE, LATEXML_OWN};
  let registered = registered_bindings();
  assert!(
    !kpsewhich("article.cls").is_empty(),
    "kpsewhich must be on PATH, with a TeX Live tree"
  );
  let lists = [
    ("AUTHOR_FILE_STAND_INS", AUTHOR_FILE_STAND_INS.to_vec()),
    (
      "FORMER_TEX_LIVE",
      FORMER_TEX_LIVE.iter().map(|(file, _)| *file).collect(),
    ),
    ("LATEXML_OWN", LATEXML_OWN.to_vec()),
  ];
  let mut seen = std::collections::HashMap::new();
  for (list, files) in &lists {
    for file in files {
      assert!(
        registered.iter().any(|r| r == file),
        "{list}: {file} is no registered binding"
      );
      assert_eq!(kpsewhich(file), "", "{list}: {file} is a TeX Live file");
      if let Some(other) = seen.insert(file.to_string(), *list) {
        panic!("{file} is on both {other} and {list}");
      }
    }
  }
  // A former TeX Live file answers `\IfFileExists` as one; a stand-in does not.
  assert!(latexml_core::binding::author_files::stands_in_for_an_author_file("arxiv.sty"));
  assert!(!latexml_core::binding::author_files::stands_in_for_an_author_file("html.sty"));
}

/// The reverse: every registered `.cls`/`.sty`/`.tex` binding kpathsea does not find is a stand-in, a former TeX Live
/// file or LaTeXML's own, so a new binding for an author's file cannot answer `\IfFileExists` unlisted. Runs on a full
/// TeX Live tree only (`aastex701.cls` is found); a trimmed one (CI's) lacks files the list takes as TeX Live's, and
/// the test says it skipped.
#[test]
fn every_binding_outside_tex_live_is_listed() {
  use latexml_core::binding::author_files::{AUTHOR_FILE_STAND_INS, FORMER_TEX_LIVE, LATEXML_OWN};
  if kpsewhich("aastex701.cls").is_empty() {
    eprintln!(
      "SKIPPED every_binding_outside_tex_live_is_listed: kpathsea finds no aastex701.cls, not a full TeX Live tree"
    );
    return;
  }
  let listed = |file: &str| {
    AUTHOR_FILE_STAND_INS.contains(&file)
      || FORMER_TEX_LIVE.iter().any(|(former, _)| *former == file)
      || LATEXML_OWN.contains(&file)
  };
  let unlisted: Vec<String> = registered_bindings()
    .into_iter()
    .filter(|file| file.ends_with(".cls") || file.ends_with(".sty") || file.ends_with(".tex"))
    .filter(|file| !listed(file) && kpsewhich(file).is_empty())
    .collect();
  assert!(
    unlisted.is_empty(),
    "bindings outside TeX Live on no list (latexml_core::binding::author_files): {unlisted:?}"
  );
}

/// jfm's `lineno` option (lineno-FLM.sty at the start of the document) is the newer copies'; with the v1.25 copy
/// 2605.03843 ships it is an unused option, and nothing asks for a lineno-FLM.sty the paper does not have. Repro
/// loader/jfm_v125/jfm_old_copy_has_no_lineno_option.
#[test]
fn jfm_lineno_option_follows_the_shipped_copy() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/jfm_v125/jfm_old_copy_has_no_lineno_option.tex"
  );
  let old = include_str!("../../../tools/perfect_kernel/repros/loader/jfm_v125/jfm.cls");
  let (log, xml) = convert_files_with(tex, &[("jfm.cls", old)], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Text no-lineno-FLM."), "{xml}");
  // A copy that declares the option loads the lineno-FLM.sty it comes with.
  let newer = old.replace(
    "\\DeclareOption{prodtf}",
    "\\newif\\iflineno\\DeclareOption{lineno}{\\linenotrue}\\DeclareOption{prodtf}",
  );
  let (log, xml) = convert_files_with(
    tex,
    &[
      ("jfm.cls", &newer),
      (
        "lineno-FLM.sty",
        "\\ProvidesPackage{lineno-FLM}\\newcommand\\flmmark{FLM}\n",
      ),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Text FLM."), "{xml}");
}
