//! Red/green guards for batch 68r: AMS-TeX (amstex.tex + amsppt.sty) completeness — amsppt's author list as one creator
//! a name with its numbered addresses, plain's `\+` under the AMS-TeX format, `\subjclassyear`, `\transl` and the
//! reference parts after it, `$$\gather` without LaTeX, and the material before `\document`. Run-353 class-less
//! Error papers (math0002054, math0103164, math0002213, cs0201007, math0005042). Repros
//! `tools/perfect_kernel/repros/amstex/*`, each checked against TL 2025's `amstex` format (its header holds the oracle).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

fn assert_clean(log: &str) {
  assert_eq!((error_count(log), warning_count(log)), (0, 0), "{log}");
}

/// amsppt's `\author … \endauthor` names the paper's authors in one list: one creator a name (no `merged_creators`
/// error), and, after a block naming several, amsppt's numbered addresses and the emails paired with them go to the
/// authors in order when there are as many (math0002054, math0002116, 1611.05173).
#[test]
fn amsppt_author_list_is_one_creator_a_name() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_author_list_pairs_addresses.tex"
  ));
  assert_clean(&log);
  assert!(
    xml.contains(
      "<creator role=\"author\">\n    <personname>Nobuo Hara</personname>\n    <contact name=\"Address:\u{a0}\" \
       role=\"address\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">a@x.jp</contact>\n  \
       </creator>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<creator before=\" and \" role=\"author\">\n    <personname>Kei-ichi Watanabe</personname>\n    <contact \
       name=\"Address:\u{a0}\" role=\"address\">Univ B</contact>\n    <contact name=\"Email:\u{a0}\" \
       role=\"email\">b@y.jp</contact>\n  </creator>"
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
}

/// More addresses than names (math0002213: Kharlamov's two): amsppt ties none to a name, so they stay together, in
/// order, on the shared creator below the authors, the current address after the address it joins.
#[test]
fn amsppt_extra_address_is_shared() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_author_list_extra_address_shared.tex"
  ));
  assert_clean(&log);
  assert!(
    xml.contains(
      "<creator role=\"author\">\n    <personname>S.Yu.Orevkov</personname>\n  </creator>\n  <creator before=\" and \
       \" role=\"author\">\n    <personname>V.M.Kharlamov</personname>\n  </creator>\n  <creator \
       role=\"author\">\n    <contact name=\"Address:\u{a0}\" role=\"address\">Steklov Math. Inst.</contact>\n    \
       <contact name=\"Address:\u{a0}\" role=\"address\">Univ B</contact>\n    <contact \
       role=\"currentaddress\">Univ C</contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ D</contact>\n  \
       </creator>"
    ),
    "{xml}"
  );
}

/// An `\input amstex` + `\documentstyle{amsppt}` document runs on the LaTeX format; its `\+` is still plain's
/// (math0103164, math0105110, math0106202, math0109193).
#[test]
fn amstex_plain_tabbing_rows() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_plain_tabbing_rows.tex"
  ));
  assert_clean(&log);
  let re = regex::Regex::new(r"<[^>]+>").unwrap();
  let cells: Vec<String> = xml
    .split("<td ")
    .skip(1)
    .map(|cell| {
      re.replace_all(
        &format!("<{}", cell.split("</td>").next().unwrap_or("")),
        "",
      )
      .into_owned()
    })
    .collect();
  assert_eq!(cells, ["a", "b", "c"], "{xml}");
}

/// A LaTeX document's stray `\+` (cond-mat0001412's `\!+\+`) stays one undefined-macro error, also when an AMS-TeX
/// command there autoloads the AmSTeX pool.
#[test]
fn latex_stray_plus_stays_undefined() {
  for tex in [
    "\\documentclass{article}\n\\begin{document}\n$a\\!+\\+b$\n\\end{document}\n",
    "\\documentclass{article}\n\\begin{document}\n\\NoBlackBoxes\n$a\\!+\\+b$\n\\end{document}\n",
  ] {
    let (log, _xml) = convert_counted(tex);
    assert_eq!(error_count(&log), 1, "{log}");
    assert_eq!(log.matches("Error:undefined:\\+ ").count(), 1, "{log}");
  }
}

/// `\subjclassyear{2000}` (amsppt.sty:391-401) names the edition `\subjclass` uses: the classification's scheme and
/// label (cs0201007, math0205233, math0209036).
#[test]
fn amsppt_subjclassyear_names_the_scheme() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_subjclassyear.tex"
  ));
  assert_clean(&log);
  assert!(
    xml.contains(
      "<classification name=\"2000 Mathematics Subject Classification. \" scheme=\"2000 Mathematics Subject \
       Classification\">05E05, 13A50</classification>"
    ),
    "{xml}"
  );
}

/// `\transl` is amsppt's `\moreref` (amsppt.sty:1088): the translation is a second part of the reference, its text
/// first and its journal, volume, year and pages after it; a book's translation is printed as a book (its
/// `\bookinfo`), and `\publaddr` is printed (math0002213, math-ph0003008, math0003031, math0111124).
#[test]
fn amsppt_transl_continues_the_reference() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amsppt_transl_reference.tex"
  ));
  assert_clean(&log);
  let text: String = {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    xml
      .lines()
      .filter(|l| l.contains("<bibblock"))
      .map(|l| re.replace_all(l.trim(), "").into_owned())
      .collect::<Vec<_>>()
      .join("\n")
  };
  assert_eq!(
    text,
    "V.I.\u{a0}Arnold, Statistics of integral convex polygons, Funkt. Anal. i Prilozhen. 14 (1980), no.\u{a0}2, 1–3.\n\
     English translation:, Funct. Anal. and Appl. 14 (1980), 79–81.\n\
     N. K. Nikolski, Treatise on the shift operator, Nauka, Moscow, 1980.\n\
     English transl., Grundlehren der mathematischen Wissenschaften 273, Springer-Verlag, Berlin etc., 1986.",
    "{xml}"
  );
}

/// `$$\gather … \endgather$$` in an `\input amsppt.sty` document (no LaTeX format): amsmath's `gathered` opens
/// without `\@empty` (math9412232, math0005042, math0006111, math0009215).
#[test]
fn amstex_gather_without_latex() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_plain_gather_return_opt_arg.tex"
  ));
  assert_clean(&log);
  assert!(xml.contains("<XMArray name=\"gathered\""), "{xml}");
  for row in ["a", "b"] {
    assert!(
      xml.contains(&format!(
        "<XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"id{}\">{row}</XMTok>",
        if row == "a" { 1 } else { 2 }
      )),
      "{xml}"
    );
  }
}

/// What the file typesets before `\document` (math0002213's epigraph) is the paper's, and the topmatter stays: the
/// document element it opened is the document.
#[test]
fn amstex_material_before_document_is_kept() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_material_before_document.tex"
  ));
  assert_clean(&log);
  assert!(xml.contains("<title>Title Text</title>"), "{xml}");
  assert!(
    xml.contains("<creator role=\"author\">\n    <personname>A. One</personname>\n  </creator>"),
    "{xml}"
  );
  assert!(xml.contains(">Epigraph words.\nBody text.</p>"), "{xml}");
}
