//! Red/green guards for batch 92g: a frontmatter note whose class prints no label has none (FIDELITY_CHARTER §4, the
//! user's rulings 2026-10-30: "keep every note with the label the class prints, which here is none", the note's role
//! kept as semantic markup, `ltx_role_<role>` in the HTML; OXIDIZED_DESIGN_DIVERGENCES #921).
//!
//! The API: an explicit empty `name=` (`name={}`, or one that digests to nothing) is no label — the role's default
//! (`\lx@pubnote@<role>@name`) no longer stands in, as Perl's getFrontmatterName has it. Each binding passes the label
//! its class prints: none for the kernel's title `\thanks`, the dedications, the report numbers set top right, the
//! head and slug notes, the journal lines, the conference lines, the bare DOIs, volumes and IDs, the article types;
//! the class's own where it prints one (aa's "DOI: ", unchanged). A call that forwards an optional label
//! (`\email[]{…}`, `name={#1}`) keeps the role's label when none is given. Each guard converts its repro under
//! `tools/perfect_kernel/repros/class-bindings/` from the binary, eager and `--streaming`, with no Error and no
//! Warning, and asserts every note (or contact) whole; two read the HTML: the note's role class, no name span.
use std::{path::Path, process::Command};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns it.
#[track_caller]
fn clean_both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(eager, streamed);
  eager
}

/// Every element `<tag …>…</tag>` of `xml`, whole, in document order (`tag` holds no element of its own name).
fn elements(xml: &str, tag: &str) -> Vec<String> {
  let (open, close) = (format!("<{tag} "), format!("</{tag}>"));
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find(&open) {
    let tail = &rest[at..];
    let end = tail
      .find(&close)
      .map_or(tail.len(), |end| end + close.len());
    found.push(tail[..end].to_string());
    rest = &tail[end..];
  }
  found
}

/// Every start tag `<tag …>` of `xml`, in document order.
fn start_tags(xml: &str, tag: &str) -> Vec<String> {
  let open = format!("<{tag} ");
  xml
    .match_indices(&open)
    .map(|(at, _)| {
      xml[at..]
        .split_inclusive('>')
        .next()
        .unwrap_or_default()
        .to_string()
    })
    .collect()
}

/// Converts `tex` to HTML with the binary (`--dest t.html`, the post stage and XSLT, ar5iv preloaded), checks it has
/// no Error and no Warning, and returns the HTML.
#[track_caller]
fn clean_html(tex: &str) -> String {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  std::fs::read_to_string(workdir.path().join("t.html"))
    .unwrap_or_else(|_| panic!("no t.html: {log}"))
}

/// The kernel's title `\thanks` (`\lx@add@pubnote@thanks`): LaTeX prints it as a marked footnote with no label, so
/// the note has none — its role stays, in the XML and as the HTML class.
#[test]
fn kernel_title_thanks_note_has_no_label() {
  let tex = repro!("kernel_title_thanks_note_has_no_label");
  let xml = clean_both_ways(tex);
  assert_eq!(
    elements(&xml, "pubnote"),
    vec![r#"<pubnote role="thanks">Funded by NSF grant 123.</pubnote>"#.to_string()],
    "{xml}"
  );
  assert_eq!(creators(&xml), vec![vec!["Ann Able".to_string()]], "{xml}");
  let html = clean_html(tex);
  assert!(
    html.contains(r#"<span class="ltx_pubnote ltx_role_thanks">Funded by NSF grant 123.</span>"#),
    "{html}"
  );
  assert!(!html.contains("ltx_note_name"), "{html}");
}

/// A binding's notes (aa's): the journal's name, the head note and the dedication print with no label, so they have
/// none; the DOI keeps "DOI: ", the label the class prints (aa.cls:430) — the role's default, unchanged.
#[test]
fn aa_notes_read_as_aa_prints_them() {
  let tex = repro!("aa_notes_read_as_aa_prints_them");
  let xml = clean_both_ways(tex);
  assert_eq!(
    elements(&xml, "pubnote"),
    vec![
      r#"<pubnote role="journal">Astronomy &amp; Astrophysics</pubnote>"#.to_string(),
      r#"<pubnote role="note">Research Note</pubnote>"#.to_string(),
      r#"<pubnote role="dedication">To the memory of B. Baker</pubnote>"#.to_string(),
      r#"<pubnote name="DOI: " role="doi">10.1051/x</pubnote>"#.to_string(),
    ],
    "{xml}"
  );
  let html = clean_html(tex);
  assert!(
    html.contains(
      r#"<span class="ltx_pubnote ltx_role_dedication">To the memory of B. Baker</span>"#
    ),
    "{html}"
  );
  assert_eq!(html.matches("ltx_note_name").count(), 1, "{html}");
  assert!(
    html.contains(r#"<span class="ltx_pubnote ltx_role_doi"><span class="ltx_note_name">"#),
    "{html}"
  );
}

/// IEEEtran's `\IEEEpubid`: the ID alone (IEEEtran.cls:6153), neither the role's "PubID: " nor Perl's content prefix
/// "pubid: ".
#[test]
fn ieeetran_pubid_note_is_the_id_alone() {
  let xml = clean_both_ways(repro!("ieeetran_pubid_note_is_the_id_alone"));
  assert_eq!(
    elements(&xml, "pubnote"),
    vec!["<pubnote role=\"pubid\">0000–0000/24$31.00\u{a0}©\u{a0}2024 IEEE</pubnote>".to_string()],
    "{xml}"
  );
}

/// REVTeX's contacts forward their optional label (`name={#1}`), the role's when none is given (the option's default):
/// they keep it then, a given one stays, and an empty one (`\email[]{…}`, revtex4-2.cls:2404-2408 `#1\href…`) is none.
#[test]
fn revtex_contact_without_its_label_keeps_the_roles() {
  let xml = clean_both_ways(repro!("revtex_contact_without_its_label_keeps_the_roles"));
  assert_eq!(
    start_tags(&xml, "contact"),
    vec![
      "<contact name=\"Email:\u{a0}\" role=\"email\">",
      "<contact name=\"URL:\u{a0}\" role=\"url\">",
      "<contact name=\"Thanks:\u{a0}\" role=\"thanks\">",
      "<contact name=\"Alternate Affiliation:\u{a0}\" role=\"altaffiliation\">",
      "<contact role=\"email\">",
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">",
      "<contact name=\"Electronic mail: \" role=\"email\">",
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">",
    ],
    "{xml}"
  );
  assert_eq!(
    creators(&xml),
    vec![
      vec![
        "Ann Able".to_string(),
        "email=a@x.org".to_string(),
        "url=http://x.org/~a".to_string(),
        "thanks=Funded.".to_string(),
        "altaffiliation=Also at Univ Y.".to_string(),
        "email=a2@x.org".to_string(),
        "affiliation=Univ X".to_string(),
      ],
      vec![
        "Bob Baker".to_string(),
        "email=b@x.org".to_string(),
        "affiliation=Univ X".to_string(),
      ],
    ],
    "{xml}"
  );
}
