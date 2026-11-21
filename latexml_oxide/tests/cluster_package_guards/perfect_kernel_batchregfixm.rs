//! Red/green guards for batch REGFIXM, four regressions of the v0.8.0 per-paper check: a renamed cvpr/iccv copy read
//! raw by its binding, and names a binding's real class or package defines, which documents extend
//! (`\g@addto@macro`, an Error on an undefined name since OD #1036) or name.
use latexml::util::test::{assert_element, convert_files_with, convert_with, kpse_has};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex` with ar5iv.sty preloaded and checks it converts clean; returns the XML.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// stand-in-copies/provides_route/cvpr_renamed_copy_keeps_its_hooks: a shipped style declaring
/// `\ProvidesPackage{cvpr}` (2412.03517's style.sty, 2311.13194, 2501.05472, …), `{iccv}` (2507.06125, 2504.13405) or
/// `{wacv}` (2609.04603) is read raw by its binding, as a self-named copy is: its `\AtEndPreamble` cleveref load runs
/// and `\cref` resolves. Red: `\cref` undefined (the copy read in provides mode, its hooks deferred). One case per
/// binding file (cvpr_sty.rs serves cvpr and wacv, iccv_sty.rs iccv).
#[test]
fn renamed_cvpr_and_iccv_copies_keep_their_hooks() {
  let tex = repro!(
    "stand-in-copies/provides_route",
    "cvpr_renamed_copy_keeps_its_hooks.tex"
  );
  let copy = repro!("stand-in-copies/provides_route", "cvprcopy.sty");
  for declared in ["cvpr", "iccv", "wacv"] {
    let copy = copy.replace(
      r"\ProvidesPackage{cvpr}",
      &format!(r"\ProvidesPackage{{{declared}}}"),
    );
    let (log, xml) = convert_files_with(tex, &[("cvprcopy.sty", &copy)], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{declared}: {log}"
    );
    assert_element(
      &xml,
      "ref",
      &[r#"labelref="LABEL:sec:intro""#],
      r#"<ref labelref="LABEL:sec:intro" show="creftype~creflabel"/>"#,
    );
  }
}

/// class-bindings/elsarticle_fnotes_document_fntext: a document's own `\fntext` adds to `\@fnotes` (elsarticle.cls:167-171;
/// 2301.05821, 2311.15389, 2312.12429, …), which the binding prints with its title-page lists: the note is kept. Red: 1
/// Error (`\@fnotes` undefined) and the note text lost. Distinct from the define-only names below: the binding prints
/// the list.
#[test]
fn elsarticle_document_fntext_note_is_kept() {
  let xml = clean(repro!(
    "class-bindings",
    "elsarticle_fnotes_document_fntext.tex"
  ));
  assert_eq!(creators(&xml), vec![vec!["A. Author".to_string()]], "{xml}");
  assert_element(
    &xml,
    "pubnote",
    &[r#"role="note""#],
    r#"<pubnote role="note">These authors contributed equally.</pubnote>"#,
  );
  assert_eq!(xml.matches("contributed equally").count(), 1, "{xml}");
}

/// class-bindings/elsarticle_endfrontmatter_extended: `\end{frontmatter}` runs `\endfrontmatter` (elsarticle.cls:1303),
/// which documents extend (1609.08567, 2308.12736, 1609.08586, 2304.14608, 1910.03866): the addition is legal and runs
/// after the frontmatter, which is unchanged. Red: 1 Error (`\endfrontmatter` undefined). Distinct: the binding's own
/// end is reached through the name.
#[test]
fn elsarticle_endfrontmatter_addition_runs() {
  let tex = repro!("class-bindings", "elsarticle_endfrontmatter_extended.tex");
  let xml = clean(tex);
  assert_eq!(creators(&xml), vec![vec!["A. Author".to_string()]], "{xml}");
  assert!(xml.contains("<title>T</title>"), "{xml}");
  let marked = tex
    .replace(
      r"\enlargethispage{-2\baselineskip}",
      r"\gdef\afterfm{AFTER}",
    )
    .replace("Body.", r"Body \afterfm.");
  assert_ne!(marked, tex);
  let xml = clean(&marked);
  assert_eq!(creators(&xml), vec![vec!["A. Author".to_string()]], "{xml}");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">Body AFTER.</p>"#);
}

/// The names a real file defines and documents extend or name, which the binding defines as it does (red: 1 Error each,
/// the name undefined): newtxmath's `\ntxmath@scaled` (newtxmath.sty:46-51, `s*[\tx@scale]`, the `scaled` key;
/// 2205.08971, 2510.18441, …); amsrefs' ifoption load (amsrefs.sty:159, `\IfOption`; 1707.00398, 1806.01802, …), skipped
/// visibly where the amsrefs bundle is absent. amsart's `\@setauthors` is `amsart_setauthors_additions_are_printed`.
#[test]
fn bindings_define_names_documents_extend() {
  let tex = repro!("package-bindings", "newtxmath_scaled_font_shape.tex");
  clean(tex);
  // (the option sets the scale as the key does: pdflatex `\meaning` "macro:->s*[1.05]")
  let probe = tex.replace(
    r"\makeatother",
    r"\typeout{[[\meaning\ntxmath@scaled]]}\makeatother",
  );
  let (log, _) = convert_with(&probe, Some("ar5iv.sty"));
  assert!(log.contains("[[macro:->s*[1.05]]]"), "{log}");
  if kpse_has("amsrefs.sty") && kpse_has("ifoption.sty") {
    let xml = clean(repro!("package-bindings", "amsrefs_loads_ifoption.tex"));
    assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">Initials off.</p>"#);
  } else {
    eprintln!("SKIP amsrefs_loads_ifoption: amsrefs.sty or ifoption.sty is not in this TeX Live");
  }
}

/// class-bindings/amsart_setauthors_*: amsart's and amsproc's `\maketitle` run `\@setauthors` after the title
/// (amsart.cls:638, :808; amsproc.cls:613, :746), a store documents extend. The binding places the names and the
/// addresses and runs it with neither: an appended `\@setaddresses` (2607.28440) places each address once; appended text
/// (the author's only affiliation and e-mail, 2610.08257; 2610.02630, 2610.25104) is a note at the title block; a
/// document's own `\renewcommand\@setauthors` (amsproc), a copy of the class's (1202.1250, 2310.04922), prints its text,
/// its `\authors` not twice, its `\contribs` test reaching no undefined setter. Red (REGFIXM): the text lost, 0
/// diagnostics.
#[test]
fn amsart_setauthors_additions_are_printed() {
  let xml = clean(repro!("class-bindings", "amsart_setauthors_extended.tex"));
  assert_eq!(
    creators(&xml),
    vec![
      vec![
        "Ann Able".to_string(),
        "address=Univ X, Town".to_string(),
        "email=ann@x.edu".to_string()
      ],
      vec!["Bob Baker".to_string(), "address=Univ Y, City".to_string()]
    ],
    "{xml}"
  );
  for address in ["Univ X, Town", "Univ Y, City", "ann@x.edu"] {
    assert_eq!(xml.matches(address).count(), 1, "{address}: {xml}");
  }
  assert!(!xml.contains("<note"), "{xml}");
  let xml = clean(repro!(
    "class-bindings",
    "amsart_setauthors_prints_additions.tex"
  ));
  assert_eq!(creators(&xml), vec![vec!["Ann Able".to_string()]], "{xml}");
  assert_element(
    &xml,
    "note",
    &[r#"role="note""#],
    r#"<note role="note" xml:id="id1"><text align="center" font="italic" fontsize="80%" xml:id="id1.1">Graduate School of AI, UNIST<break/>dms6721@unist.ac.kr</text></note>"#,
  );
  assert_eq!(xml.matches("UNIST").count(), 1, "{xml}");
  let xml = clean(repro!("class-bindings", "amsproc_setauthors_renewed.tex"));
  assert_eq!(creators(&xml), vec![vec!["Ann Able".to_string()]], "{xml}");
  assert_eq!(xml.matches("Ann Able").count(), 1, "{xml}");
  assert_element(
    &xml,
    "note",
    &[r#"role="note""#],
    r#"<note role="note" xml:id="id1"><itemize class="ltx_trivlist" xml:id="S0.I1a">
        <item class="ltx_centering" xml:id="S0.I1.ix1a">
          <tags>
            <tag/>
          </tags>
          <para xml:id="S0.I1.ix1.p1a">
            <p xml:id="S0.I1.ix1.p1a.1"><text font="italic" xml:id="S0.I1.ix1.p1a.1.1">Institute Z, Somewhere</text></p>
          </para>
        </item>
      </itemize></note>"#,
  );
}
