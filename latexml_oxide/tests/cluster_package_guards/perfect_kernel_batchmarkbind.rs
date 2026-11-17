//! Red/green guards for MARKBIND (DG 2026-11-16, "bindings compute, the XSLT invents"): a class binding computes the
//! glyph its class prints and the counter its commands step. mnras's and REVTeX's society marks are rows of
//! `perfect_kernel_batchfmmarks`; here lni's (a TeX Live class that changed its `\author` between versions, so each guard
//! is skipped, visibly, where TeX Live's lni.cls is not the v2.0 one built on authblk), mnras's footnote counting, and
//! the Frontmatter keys the engine declares (witnesses 2407.04435, 2412.04178).
use latexml::util::test::{convert_with_setup_then, error_count, kpse_has, warning_count};

use super::{perfect_kernel_batch84b::text, perfect_kernel_batchshmark::notes_in};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Whether TeX Live's lni.cls is v2.0's, built on authblk (`\RequirePackage{authblk}`, lni.cls:362), which lni_cls.rs
/// binds; an older one keeps LaTeX's `\author`.
fn lni_on_authblk() -> bool {
  let found = kpse_has("lni.cls")
    && std::process::Command::new("kpsewhich")
      .arg("lni.cls")
      .output()
      .ok()
      .and_then(|out| std::fs::read_to_string(String::from_utf8_lossy(&out.stdout).trim()).ok())
      .is_some_and(|cls| cls.contains("\\RequirePackage{authblk}"));
  if !found {
    eprintln!("SKIP: TeX Live's lni.cls is not v2.0's (built on authblk)");
  }
  found
}

/// lni's `\affil[n]` sets the footnote counter to n (lni.cls:463), so Ann's title `\thanks` is 3 after the labels 1 and
/// 2, where authblk's `\affil` alone left it 1, Lab One's label (review revMARKSAFEc `lni_aff`, DG CASE 1). Distinct
/// from `perfect_kernel_batchshmark::lni_prints_its_own_arabic_title_marks` (the glyph): this is the counter.
#[test]
fn lni_affil_sets_the_footnote_counter() {
  if !lni_on_authblk() {
    return;
  }
  let (log, xml, ()) = convert_with_setup_then(
    repro!("fmmarks_lni_affil_sets_the_footnote_counter"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(notes_in(&xml), [
    vec!["Ann Able".to_string(), "thanks[3]=Note A.".to_string()],
    vec!["Bob Baker".to_string()],
  ]);
  // (the `\thanks` steps the counter, which lni's `\maketitle` does not set back: the body's note is 4, review revMARKBIND)
  assert_eq!(all_notes(&xml), [
    "thanks[3]=Note A.",
    "footnote[4]=Body note."
  ]);
}

/// Every note of the document in order, `role[mark]=text`, its `<tags>` left out.
fn all_notes(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find("<note ") {
    let note = &rest[at..];
    let open_end = note.find('>').expect("note tag");
    let attribute = |name: &str| {
      note[..open_end]
        .split(&format!(" {name}=\""))
        .nth(1)
        .and_then(|m| m.split('"').next())
        .unwrap_or("")
        .to_string()
    };
    let close = note.find("</note>").expect("closed note");
    let mut body = note[open_end + 1..close].to_string();
    if let (Some(from), Some(to)) = (body.find("<tags>"), body.find("</tags>")) {
      body.replace_range(from..to + "</tags>".len(), "");
    }
    out.push(format!(
      "{}[{}]={}",
      attribute("role"),
      attribute("mark"),
      text(&body)
    ));
    rest = &note[close..];
  }
  out
}

/// mnras's `\thanks` steps the footnote counter and its `\maketitle` does not set it back: the notes before the first
/// `\section` count on in the class's symbols, ⋆ † ‡, and that `\section` (starred) numbers them in arabic from 1, a later
/// `\section` and the appendix counting on (review revMARKBIND p/mn_ids.tex, p/mn_nosec.tex; mnras.cls:736-737,
/// :1056-1071, :1517-1523). Distinct from `perfect_kernel_batchfmmarks`'s mnras rows (the title's marks): the body's
/// notes, on both sides of the switch.
#[test]
fn mnras_title_notes_step_the_counter() {
  let (log, xml, ()) = convert_with_setup_then(
    repro!("fmmarks_mnras_title_notes_step_the_counter"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(all_notes(&xml), [
    "thanks[⋆]=E-mail: ann@x.org",
    "footnote[†]=Abs note.",
    "footnote[‡]=Pre note.",
    "footnote[1]=Body note A.",
    "footnote[2]=Body note B.",
    "footnote[3]=Body note C.",
    "footnote[4]=App note.",
  ]);
}

/// An `\author[running names]{names}` written for lni v1 is the kernel's `\author[short]{names}`: two creators and no
/// stray text, where v2.0's `[4][]` reading took the next tokens (`\startpage`) for its e-mail and ORCID and left
/// `\startpage`'s argument "1" printed in the body (2407.04435, whose running names authblk then read as 8 labels).
#[test]
fn lni_v1_author_is_the_kernels() {
  if !lni_on_authblk() {
    return;
  }
  let (log, xml, ()) = convert_with_setup_then(
    repro!("fmmarks_lni_v1_author_running_names"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(notes_in(&xml), [vec!["Ann Able".to_string()], vec![
    "Bob Baker".to_string()
  ]]);
  assert!(!xml.contains("<contact"), "{xml}");
  let body = xml.split("</creator>").last().expect("a creator");
  assert_eq!(text(body), "Body.", "{xml}");
}

/// The engine's own Frontmatter keys are declared, in one list (`frontmatter_api::FRONTMATTER_INTERNAL_KEYS`): a block
/// passing seven of them (`_cutlist`, `_group`, `_markintext`, `_markshown`, `_marks`, `_sequence`, `_unmarked`)
/// reports no unknown key, while a key the list does not hold still does (a typo, `_markshwon`).
#[test]
fn internal_frontmatter_keys_are_declared() {
  let unknown = |log: &str| {
    log
      .lines()
      .filter(|line| line.contains("unknown KeyVals key '_") && line.contains("'Frontmatter'"))
      .map(ToString::to_string)
      .collect::<Vec<_>>()
  };
  let (log, ..) = convert_with_setup_then(
    repro!("iopart_ead_corresponding_author_legend"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(unknown(&log), Vec::<String>::new());
  let (log, ..) = convert_with_setup_then(
    "\\documentclass{article}\n\\makeatletter\n\\title{T}\\author{Ann Able}\n\
     \\lx@add@contact[role=note,_markshwon=1]{A note.}\n\\begin{document}\\maketitle\nBody.\n\\end{document}\n",
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!(unknown(&log).len(), 1, "{log}");
  assert!(unknown(&log)[0].contains("'_markshwon'"), "{log}");
}
