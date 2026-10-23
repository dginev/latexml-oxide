//! Guards for FM S2a.2 r9: the frontmatter observer on the r8 landing review's findings (revfms2a2_8) and the
//! rehearsal's isolation (OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r9"), in shadow mode. Each shape was red on r8
//! (6604bc49cf): a link contact held the anchor it printed, not where it points, and a `mailto:` link was a url read by
//! its anchor (finding 1: 2210.15991, 2309.10007); an API entry naming an equal contribution became a legend note with
//! the institutions it holds (finding 2: 1903.06934); and the switch itself changed the XML — a box register the
//! author block prints (2405.14627, 2603.26792) and a fallback meaning the rehearsal `\let` (siamart's `\AND`,
//! 2409.10200). Every probe compiles clean with pdflatex (TL 2025; the siamart one with 2409.10200's class) and
//! converts with no Error or Warning, and the XML is byte-identical with the switch on and off.
use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

/// Convert `tex` under the shadow switch, asserting a clean conversion: the whole dump.
fn dump_with(tex: &str) -> Value {
  let (log, _, json) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"))
}

/// A dump's resolved tuples, as `perfect_kernel_batchfms2a2r8::dump_tuples` writes them, a contact's target after its
/// text (`→ <target>`): `C <name> [<requests>] | <rule>`, then its `a` affiliations, `c <role>` contacts, `n` notes;
/// `B <role>` the block's, `D <role>` what was dropped.
fn tuples(tex: &str) -> Vec<String> {
  let dump = dump_with(tex);
  let resolved = &dump["resolved"];
  let item = |prefix: &str, a: &Value, role: bool| {
    let target = a["target"]
      .as_str()
      .map(|t| format!(" → {t}"))
      .unwrap_or_default();
    let role = if role {
      format!("{} ", a["role"].as_str().unwrap_or(""))
    } else {
      String::new()
    };
    format!(
      "{prefix} {role}{}{target} | {}",
      a["text"].as_str().unwrap_or(""),
      a["rule"].as_str().unwrap_or("")
    )
  };
  let mut out = Vec::new();
  for c in resolved["creators"].as_array().expect("creators") {
    let requests: Vec<&str> = c["requests"]
      .as_array()
      .expect("requests")
      .iter()
      .filter_map(Value::as_str)
      .collect();
    out.push(format!(
      "C {} [{}] | {}",
      c["name"].as_str().unwrap_or(""),
      requests.join(","),
      c["rule"].as_str().unwrap_or("")
    ));
    for a in c["affiliations"].as_array().expect("affiliations") {
      out.push(item("  a", a, false));
    }
    for a in c["contacts"].as_array().expect("contacts") {
      out.push(item("  c", a, true));
    }
    for a in c["notes"].as_array().expect("notes") {
      out.push(item("  n", a, false));
    }
  }
  for a in resolved["block"].as_array().expect("block") {
    out.push(item("B", a, true));
  }
  for a in resolved["dropped"].as_array().expect("dropped") {
    out.push(item("D", a, true));
  }
  out
}

const LINKS: &str = r#"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\title{T}
\author{Robin Hankin\thanks{\href{https://academics.aut.ac.nz/robin.hankin}{work}, play} \and Tanmay Samak\thanks{\texttt{\{\href{mailto:tsamak@clemson.edu}{tsamak}, \href{mailto:csamak@clemson.edu?subject=Hi}{csamak}\}@clemson.edu}} \and Chinmay Samak}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_8 finding 1: a link contact carries where it points (`Assoc::target`, the dump's `target`), its text
/// the anchor it prints — Hankin's `\href{https://academics.aut.ac.nz/robin.hankin}{work}` was a url contact "work"
/// with the url lost (2210.15991); a `mailto:` link is an e-mail, its target the address (a `?subject=` aside), placed by
/// the address — `\href{mailto:tsamak@clemson.edu}{tsamak}` was a url "tsamak" (2309.10007). The notes keep their whole
/// text (review revfms2a2 finding 1).
#[test]
fn a_link_contact_carries_its_target() {
  assert_eq!(tuples(LINKS), [
    "C Robin Hankin [] | line:names",
    "  c url work → https://academics.aut.ac.nz/robin.hankin | note:adjacent ruling4:inner-contact",
    "  n work, play | note:adjacent",
    "C Tanmay Samak [] | line:names sep:column",
    "  c email tsamak → tsamak@clemson.edu | note:adjacent ruling4:inner-contact",
    "  n { tsamak, csamak }@clemson.edu | note:adjacent",
    "C Chinmay Samak [] | line:names sep:column",
    "  c email csamak → csamak@clemson.edu | note:adjacent ruling4:inner-contact owner:spelled",
  ]);
}

const IEEE_LEGEND: &str = r#"\documentclass[conference]{IEEEtran}
\begin{document}
\title{T}
\author{\IEEEauthorblockN{Ann Able*, Bob Baker*}
\IEEEauthorblockA{{*Equal Contribution} \\
\IEEEauthorblockA{\textit{Univ A}}
\IEEEauthorblockA{\textit{National Center in Town}}
\{ann, bob\}@x.org}
\and
\IEEEauthorblockN{Cat Cole$^\dagger$}
\IEEEauthorblockA{{$^\dagger$These authors contributed equally} \\ E-mail: cat@x.org}}
\maketitle
Text.
\end{document}
"#;

/// Review revfms2a2_8 finding 2: an API entry a symbol leads is a legend note (r8, 0812.1531) only when no piece of it
/// reads as an affiliation — IEEEtran's `\IEEEauthorblockA{{*Equal Contribution} \\ \IEEEauthorblockA{Univ A}
/// \IEEEauthorblockA{National Center …} …}` holds two institutions, and r8 made the whole a legend note of the starred
/// names (1903.06934); it is their affiliation again, as r7 read it. An entry printing the legend and an address only
/// stays a legend (1902.02631's `These authors contributed equally to this work \\ E-mail: …`), the role change tagged
/// `api:legend`.
#[test]
fn an_api_legend_holding_an_institution_is_an_affiliation() {
  // (the whole tuple list, Bob's and the institutions' text included — review revfms2a2_9 finding 2: a substring test
  // passed with an institution or the addresses out of the affiliation, and never read Bob)
  assert_eq!(tuples(IEEE_LEGEND), [
    "C Ann Able [*] | line:names",
    "  a Equal Contribution Univ A National Center in Town {ann, bob}@x.org | api:leading-mark",
    "C Bob Baker [*] | line:names sep:comma",
    "  a Equal Contribution Univ A National Center in Town {ann, bob}@x.org | api:leading-mark",
    "C Cat Cole [†] | line:names",
    "  n These authors contributed equally E-mail: cat@x.org | api:leading-mark api:legend",
  ]);
}

/// The rehearsal leaves the conversion as it found it (the SYNC row "FM shadow rehearsal leaks into the conversion"):
/// the XML of each shape byte-identical with the switch on and off, and what it prints as pdflatex does.
/// - a box register the author block prints with `\usebox`: the rehearsal's print stamped the register's box itself
///   with a field provenance its scope then dropped, and the conversion's print, finding it stamped, wrapped it in an
///   `ltx:text` with an id (core `token::FieldLedgerScope` now unstamps the boxes it stamped) — 2405.14627's one
///   `\ORCIDicon`, 2603.26792's one box printed by four authors' links and once in the body;
/// - a meaning's standing as a fallback guess (OmniBus's `\AND` under the siamart binding): the rehearsal's group-local
///   `\let\AND` cleared it, which no group end puts back, and algorithmic.sty:161's `\newcommand{\AND}` was refused —
///   `\AND` printed a plain "and", not `\textbf{and}` (2409.10200; core `state::AssignmentJournal` now saves the standing
///   with the bindings and its rollback restores it).
#[test]
fn the_rehearsal_leaves_the_conversion_unchanged() {
  let shapes: [(&str, &str, &[&str], &[&str]); 3] = [
    (
      "used box",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_rehearsal_leaves_a_used_box_unchanged.tex"
      ),
      &[
        r#"<ref class="ltx_href" href="https://orcid.org/0000-0002-1828-8183"><rule height="9.0pt" width="9.0pt"/></ref>"#,
      ],
      &["<text xml:id="],
    ),
    (
      "shared box",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_rehearsal_leaves_a_shared_box_unchanged.tex"
      ),
      &[
        r#"<ref class="ltx_href" href="https://orcid.org/0000-0003-2623-5206"><rule height="6.0pt" width="6.0pt"/></ref>"#,
        r#"<ref class="ltx_href" href="https://orcid.org/0000-0001-5109-3700"><rule height="6.0pt" width="6.0pt"/></ref>"#,
      ],
      &["<text xml:id="],
    ),
    (
      "fallback meaning",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/observer_rehearsal_keeps_a_fallback_meaning.tex"
      ),
      &[r#"<text font="bold" xml:id="algx1.l1.2">and</text>"#],
      &["</Math>  and <Math"],
    ),
  ];
  for (shape, tex, present, absent) in shapes {
    let run = |on: bool| {
      convert_files_with_setup_then(
        tex,
        &[],
        Some("ar5iv.sty"),
        move || set_fm_shadow_override(Some(on)),
        |_| (),
      )
    };
    let (on_log, on_xml, ()) = run(true);
    let (off_log, off_xml, ()) = run(false);
    assert_eq!(on_xml, off_xml, "{shape}");
    for (log, side) in [(&on_log, "on"), (&off_log, "off")] {
      assert_eq!(
        (error_count(log), warning_count(log)),
        (0, 0),
        "{shape} {side}: {log}"
      );
    }
    for needle in present {
      assert!(on_xml.contains(needle), "{shape}: {needle}\n{on_xml}");
    }
    // (in the frontmatter: the body's own `\usebox` is a box of the paragraph, an `ltx:text` with its id either way)
    let front = on_xml.split("<para").next().unwrap_or("");
    for needle in absent {
      assert!(!front.contains(needle), "{shape}: {needle}\n{on_xml}");
    }
  }
}

/// The r9 shapes leave the XML byte-identical with the switch on and off (shadow only).
#[test]
fn r9_shapes_leave_the_xml_unchanged() {
  for tex in [LINKS, IEEE_LEGEND] {
    let run = |on: bool| {
      convert_files_with_setup_then(
        tex,
        &[],
        Some("ar5iv.sty"),
        move || set_fm_shadow_override(Some(on)),
        |_| (),
      )
    };
    let (on_log, on_xml, ()) = run(true);
    let (off_log, off_xml, ()) = run(false);
    assert_eq!(on_xml, off_xml, "{tex}");
    assert_eq!(
      (error_count(&on_log), warning_count(&on_log)),
      (0, 0),
      "{on_log}"
    );
    assert_eq!(
      (error_count(&off_log), warning_count(&off_log)),
      (0, 0),
      "{off_log}"
    );
  }
}
