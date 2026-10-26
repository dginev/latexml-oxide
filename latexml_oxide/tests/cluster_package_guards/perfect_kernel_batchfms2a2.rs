//! Guards for batch FM S2a.2: the frontmatter observer's resolver and policy table, in shadow mode
//! (OXIDIZED_DESIGN_DIVERGENCES #837; KERNEL_CAPABILITIES K23). Under the shadow switch (here the thread's override)
//! the ledger's blocks and API calls are resolved into the tuples `ab_audit fmshadow` compares with the XML — each
//! creator's name without its marks, its affiliations, contacts and notes, the author block's own — each with the rule
//! that decided it; the guards assert those tuples and rules for the design probes (symbol lines, a user mark macro),
//! the gretsi shared-mark repro, 2401.16957's `,\inst`, the SYNC frontmatter rows' shapes, the 78f placements
//! (jheppub/iopart e-mail by spelling else call order, a `\label` in `\thanks`), an A&A `\inst{\ref{…}}` mark, the r2
//! defect classes of review revfms2a2 (whole note texts, whole fuzzy labels, `author:N` placements, the prefix
//! fallback, legends by their leading Mark, institution lines, contact roles, marks mid-line and trailing, authblk's
//! `\AB@thenote`, report's title-page marks, a replacing `\author`, the amsart contact grid), the r3 classes (marked
//! entries of one call, glued labels, fuzzy labels by surname and with their macros, e-mails by evidence, the per-author
//! names line, ORCID links, `\cite` marks, environment arguments, title notes, a sole author's orphans, author-order
//! address lists, collaborations), and that the XML is still byte-identical with the switch on and off. Every probe
//! compiles clean with pdflatex (TL 2025; the aa and svjour3 probes with 1809.08354's aa.cls and 0807.3898's
//! svjour3.cls, which TeX Live lacks) and converts with no Error or Warning.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

/// Convert `tex` (with `preload`) under the shadow switch, asserting a clean conversion: the dump's `resolved`.
fn resolved(tex: &str, preload: Option<&str>) -> Value {
  let (log, _, json) = convert_with_setup_then(
    tex,
    preload,
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let json: Value = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}: {json}"));
  json["resolved"].clone()
}

/// The resolved tuples compactly, one line per item: `C <name> [<requests>] <rule>`, then its `a <text> <rule>`
/// (affiliations), `c <role> <text> <rule>` (contacts), `n <text> <rule>` (notes); `B <role> <text> <rule>` the block's,
/// `D <role> <text> <rule>` what a ruling dropped.
fn tuples(resolved: &Value) -> Vec<String> {
  let mut out = Vec::new();
  let item = |prefix: &str, a: &Value, role: bool| {
    let rule = a["rule"].as_str().unwrap_or("");
    let text = a["text"].as_str().unwrap_or("");
    if role {
      format!(
        "{prefix} {} {text} | {rule}",
        a["role"].as_str().unwrap_or("")
      )
    } else {
      format!("{prefix} {text} | {rule}")
    }
  };
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

/// An article document holding `author` (and `preamble`): its resolved tuples.
fn article(preamble: &str, author: &str) -> Vec<String> {
  tuples(&resolved(
    &format!(
      "\\documentclass{{article}}\n{preamble}\n\\begin{{document}}\n\\title{{T}}\n{author}\n\\maketitle\nBody.\n\\end{{document}}\n"
    ),
    Some("ar5iv.sty"),
  ))
}

/// Ruling 2 (the design's `dag`, `dagger`, `dagger2` probes): a line led by a footnote symbol labels that line for
/// every name carrying the same character, whatever command printed it — Ann Able and Cat Cole `†Univ A`, Bob Baker
/// `‡Univ B` (today all of it is one contact on Cat Cole).
#[test]
fn symbol_lines_go_to_their_carriers() {
  let names_line = r"\author{Ann Able$^{\dag}$ \and Bob Baker$^{\ddag}$ \and Cat Cole$^{\dag}$\\";
  for lines in [
    r"\dag Univ A\\ \ddag Univ B}",
    r"$^{\dagger}$Univ A\\ $^{\ddagger}$Univ B}",
    r"$\dagger$ Univ A\\ $\ddagger$ Univ B}",
  ] {
    assert_eq!(
      article("", &format!("{names_line}\n{lines}")),
      [
        "C Ann Able [†] | line:names",
        "  a Univ A | ruling2:symbol-line",
        "C Bob Baker [‡] | line:names sep:column",
        "  a Univ B | ruling2:symbol-line",
        "C Cat Cole [†] | line:names sep:column",
        "  a Univ A | ruling2:symbol-line",
      ],
      "{lines}"
    );
  }
}

/// The design's `mac` probe (2304.13998's shape): a user macro printing `\textsuperscript{#1}` marks the names, its
/// `1,2` two requests (ruling 1); `~~ ` between the names a gap, so Bob Baker is a name of his own and no junk "2]"
/// creator is made (today's token reading).
#[test]
fn user_mark_macro_marks_request_affiliations() {
  assert_eq!(
    article(
      r"\newcommand*{\affmark}[1][*]{\textsuperscript{#1}}",
      r"\author{Ann Able\affmark[1]~~ Bob Baker\affmark[1,2]\\ \affmark[1]Univ A\\ \affmark[2]Univ B}"
    ),
    [
      "C Ann Able [1] | line:names",
      "  a Univ A | ruling1:request",
      "C Bob Baker [1,2] | line:names sep:gap",
      "  a Univ A | ruling1:request",
      "  a Univ B | ruling1:request",
    ]
  );
}

/// Ruling 1 on the gretsi v1.3 repro (2508.03934; class-bindings/gretsi_v13_shared_mark_affiliation, RED for its
/// output): the marks shown on each `\auteur`'s name are its requests and the name is its text alone, so the stray
/// "2" of `Lepage1,2` is no label — Simon Lepage holds CRITEO and LIGM, Jeremie Mary CRITEO, David Picard LIGM (the
/// PDF's), each with the e-mail its call gave.
#[test]
fn gretsi_shown_marks_are_the_requests() {
  assert_eq!(
    tuples(&resolved(
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/gretsi_v13_shared_mark_affiliation.tex"
      ),
      Some("ar5iv.sty"),
    )),
    [
      "C Simon Lepage [1,2] | line:names",
      "  a CRITEO AI Lab, Paris, France | api:leading-mark",
      "  a LIGM, Ecole des Ponts, France | api:leading-mark",
      "  c email s.lepage@x.com | api:annotate:last",
      "C Jeremie Mary [1] | line:names",
      "  a CRITEO AI Lab, Paris, France | api:leading-mark",
      "  c email j.mary@x.com | api:annotate:last",
      "C David Picard [2] | line:names",
      "  a LIGM, Ecole des Ponts, France | api:leading-mark",
      "  c email d.picard@y.fr | api:annotate:last",
    ]
  );
}

/// 2401.16957's `,\inst` (repro llncs_comma_before_inst_is_the_name_before; design §5 default 9, OD #807): under the
/// llncs policy an `\inst` after a comma is the name's before it, the `\fnmsep` comma between marks too, the `\thanks`
/// Ann's; `\institute{A \and B \and C}` numbered by its `\and`s (`labelseq`, OD #791) — the PDF's "Ann Able,1,2⋆ Bob
/// Baker,3,2 and Cat Cole1".
#[test]
fn llncs_inst_after_a_comma_is_the_name_before() {
  assert_eq!(
    tuples(&resolved(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_comma_before_inst_is_the_name_before.tex"
      ),
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [1,2] | line:names mark:inst-after-comma",
      "  a Inst A | api:labelseq",
      "  a Inst B | api:labelseq",
      "  n Email: ann@x.org | note:adjacent",
      "C Bob Baker [3,2] | line:names sep:comma mark:inst-after-comma",
      "  a Inst B | api:labelseq",
      "  a Inst C | api:labelseq",
      "C Cat Cole [1] | line:names",
      "  a Inst A | api:labelseq",
    ]
  );
}

/// aa's `,\inst` (aa.cls:759-760, the same `\unskip` convention; OD #807; the probe compiles clean with pdflatex and
/// 1809.08354's aa.cls, which TeX Live lacks): aa's binding loads inst_support.sty, whose
/// `\author` the policy key names (entry `inst_support`), which reads it as llncs's.
#[test]
fn aa_inst_after_a_comma_is_the_name_before() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{aa}\n\\begin{document}\n\\title{T}\n\\author{Ann Able,\\inst{1}\\fnmsep\\inst{2} Bob Baker,\\inst{2}}\n\\institute{Inst A \\and Inst B}\n\\maketitle\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [1,2] | line:names mark:inst-after-comma",
      "  a Inst A | api:labelseq",
      "  a Inst B | api:labelseq",
      "C Bob Baker [2] | line:names sep:comma mark:inst-after-comma",
      "  a Inst B | api:labelseq",
    ]
  );
}

/// The 78f placements (design §5 default 9): jheppub's `\emailAdd` list in author order, one call per author —
/// spelled addresses on their authors, the unspelled `zq1@` by its place, the third call's two addresses both Cat
/// Cole's (OD #805; repro jheppub_emailadd_in_author_order).
#[test]
fn jheppub_emails_by_spelling_else_call_order() {
  let out = tuples(&resolved(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jheppub_emailadd_in_author_order.tex"
    ),
    Some("ar5iv.sty"),
  ));
  assert_eq!(out, [
    "C Ann Able [a] | line:names",
    "  a Univ A | api:label",
    "  c email zq1@x.org | api:sequence owner:order",
    "C Bob Baker [a] | line:names",
    "  a Univ A | api:label",
    "  c email bbaker@x.org | api:sequence owner:spelled",
    "C Cat Cole [b] | line:names",
    "  a Univ B | api:label",
    "  c email wy7@y.org | api:sequence owner:order",
    "  c email cat2@y.org | api:sequence owner:spelled",
  ]);
}

/// iopart's `\ead` by its spelling (OD #806, 2311.12551's shape; repro iopart_ead_goes_to_the_author_it_spells),
/// `\address{$^1$ …}` entries by their leading marks.
#[test]
fn iopart_ead_goes_to_the_author_it_spells() {
  assert_eq!(
    tuples(&resolved(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/iopart_ead_goes_to_the_author_it_spells.tex"
      ),
      Some("ar5iv.sty"),
    )),
    [
      "C A. Bal [1] | line:names",
      "  a KIT, Karlsruhe | api:leading-mark",
      "C B. Maier [1,2] | line:names sep:comma",
      "  a KIT, Karlsruhe | api:leading-mark",
      "  a ICL, London | api:leading-mark",
      "  c email benedikt.maier@cern.ch | api:sequence owner:spelled",
      "C C. Cole [2] | line:names sep:word",
      "  a ICL, London | api:leading-mark",
    ]
  );
}

/// SYNC rows 46, 53, 66's shapes: spacing-run names over symbol lines and the unmarked rows after them (each line its
/// own contact, the names row above's: R12); `$^{*1}$` two requests, `*` and `1`; a mark with a trailing comma;
/// `\hspace{1em}and\hspace{1em}` a separator.
#[test]
fn sync_row_shapes_46_53_66() {
  assert_eq!(
    article(
      "",
      r"\author{Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics}"
    ),
    [
      "C Ann Able [†] | line:names",
      "  a Univ A | ruling2:symbol-line",
      "C Bob Baker [‡] | line:names sep:gap",
      "  a Univ B | ruling2:symbol-line",
      "C Cat Cole [] | line:names",
      "  a Physics | line:unmarked R12:row-above",
    ]
  );
  assert_eq!(
    article(
      "",
      r"\author{Ann Able$^{*1}$, Bob Baker$^{1}$ \\ $^1$Univ A}"
    ),
    [
      "C Ann Able [*,1] | line:names",
      "  a Univ A | ruling1:request",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A | ruling1:request",
    ]
  );
  assert_eq!(article("", r"\author{Ann Able$^{1,}$ Bob Baker$^{2}$}"), [
    "C Ann Able [1] | line:names",
    "C Bob Baker [2] | line:names"
  ]);
  assert_eq!(
    article("", r"\author{Ann Able\hspace{1em}and\hspace{1em}Bob Baker}"),
    [
      "C Ann Able [] | line:names",
      "C Bob Baker [] | line:names sep:gap"
    ]
  );
}

/// SYNC rows 47 and 48's shapes: a link under a name is that name's URL contact (the row above, R12), a bare address a
/// contact by the owner chain (the group's only author); an organization link and a braced line under the kernel
/// `\author`'s columns, each column's own.
#[test]
fn sync_row_shapes_47_48_links() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{amsart}\n\\usepackage{hyperref}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\\\ \\href{https://x.org}{Project Page} \\\\ Univ B \\\\ contact@x.org}\n\\maketitle\nBody.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [] | line:names",
      "  a Univ B | line:unmarked R12:row-above",
      "  c url Project Page | line:contact R12:row-above",
      "  c email contact@x.org | owner:only-author ruling3:one-per-print",
    ]
  );
  assert_eq!(
    article(
      r"\usepackage{hyperref}",
      r"\author{Hua Wei \\ \href{https://www.huawei.com}{Huawei} \and Hua Wu \\ {HW}}"
    ),
    [
      "C Hua Wei [] | line:names",
      "  c url Huawei | line:contact R12:row-above",
      "C Hua Wu [] | line:names sep:column",
      "  a HW | line:unmarked R12:row-above",
    ]
  );
}

/// SYNC rows 62 and 64's shapes (authblk): with a label given the marks show, so Bob shows `\AB@thenote`, "1", and the
/// unlabelled `\affil{Univ Y}` is labelled "1" too (authblk.sty:114-118, 148-151; ruling 1, review revfms2a2's answer
/// to question 6b): "Ann¹ and Bob¹", both under Univ X and Univ Y, as pdflatex prints it (2605.25017);
/// `\author{Ann$^1$ and Bob$^2$}` with `\affil{$^1$X}` by its leading mark, `\affil{$^3$Z}` requested by no one, the
/// block's.
#[test]
fn sync_row_shapes_62_64_authblk() {
  assert_eq!(
    article(
      r"\usepackage{authblk}",
      "\\author[1]{Ann}\n\\author{Bob}\n\\affil[1]{Univ X}\n\\affil{Univ Y}"
    ),
    [
      "C Ann [1] | line:names",
      "  a Univ X | api:label",
      "  a Univ Y | api:thenote ruling1:request",
      "C Bob [1] | api:creator",
      "  a Univ X | api:label ruling1:thenote",
      "  a Univ Y | api:thenote ruling1:request",
    ]
  );
  assert_eq!(
    article(
      r"\usepackage{authblk}",
      "\\author{Ann$^1$ and Bob$^2$}\n\\affil{$^1$X}\n\\affil{$^3$Z}"
    ),
    [
      "C Ann [1] | line:names",
      "  a X | api:leading-mark",
      "C Bob [2] | line:names sep:word",
      "B affiliation Z | api:leading-mark label:unrequested owner:block",
    ]
  );
}

/// A `\label` in a `\thanks` labels the note (78f item 4, design §5 default 9); a `\thanks` whose content became
/// contacts is dropped (ruling 4), an empty `\thanks{}` kept; an e-mail line no name spells goes to the one author a
/// note marks as the corresponding author (FIDELITY_CHARTER §4, 2026-10-21). (FM S2a.2 r8, main-session decision 1 on
/// r7: a url is an author's only as the page it spells — `https://b.org` spells no one, so it stays in Bob's note, which
/// is kept.)
#[test]
fn notes_labels_and_contacts() {
  let (log, _, json) = convert_with_setup_then(
    "\\documentclass{article}\n\\usepackage{url}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{Corresponding author.\\protect\\label{fn:a}} \\and Bob Baker\\thanks{\\protect\\url{https://b.org}} \\and Cat Cole\\thanks{}\\\\ office@x.org}\n\\maketitle\nSee \\ref{fn:a}.\n\\end{document}\n",
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let json: Value = serde_json::from_str(&json).expect("json");
  let creators = json["resolved"]["creators"].as_array().expect("creators");
  assert_eq!(creators[0]["notes"][0]["label"], "fn:a");
  // (the address under Cat Cole's column spells no one and the block has three authors: the one whose note names them
  // the corresponding author owns it)
  assert_eq!(tuples(&json["resolved"]), [
    "C Ann Able [] | line:names",
    "  c email office@x.org | owner:corresponding ruling3:one-per-print",
    "  n Corresponding author. | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  n https://b.org | note:adjacent",
    "C Cat Cole [] | line:names sep:column",
    "  n  | note:adjacent ruling4:empty-kept",
  ]);
}

/// An `\inst{\ref{key}}` mark (A&A, 1809.08354 — the probe compiles clean with its aa.cls, "Ann Able1 and Bob Baker2";
/// S2a.1 r2's open question (a)) prints nothing at digestion: its label is
/// the `\ref`'s key, matched against the `\label` inside the `\institute` entry.
#[test]
fn ref_mark_takes_the_label_of_its_institute() {
  // (the `\label` keys request nothing of their own here: each entry is also the `\and` list's numbered piece, and
  // the key, `ref:ia`, is what the mark requests — Inst A is Ann's alone, Inst B Bob's alone)
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{aa}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\inst{\\ref{ia}} \\and Bob Baker\\inst{\\ref{ib}}}\n\\institute{Inst A\\label{ia} \\and Inst B\\label{ib}}\n\\maketitle\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [ref:ia] | line:names",
      "  a Inst A | api:labelseq",
      "C Bob Baker [ref:ib] | line:names",
      "  a Inst B | api:labelseq",
    ]
  );
}

/// revtex's `\firstname{…} \surname{…}` is one name (a layout container between two runs never separates, decision
/// 6d; 1303.6104); the revtex policy entry runs (its `\affiliation`'s `annotate=new` the call's own).
#[test]
fn revtex_name_parts_are_one_name() {
  let json_out = resolved(
    "\\documentclass{revtex4-2}\n\\begin{document}\n\\title{T}\n\\author{\\firstname{Francesco} \\surname{Parisen Toldin}}\n\\author{Ann Able}\n\\affiliation{Univ A}\n\\maketitle\nText.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(tuples(&json_out), [
    "C Francesco Parisen Toldin [] | line:names name:layout-joined",
    "  a Univ A | api:annotate:run",
    "C Ann Able [] | line:names",
    "  a Univ A | api:annotate:run",
  ]);
  assert_eq!(json_out["policies"][0][1], "revtex");
}

/// A `\footnotemark[<n>]` is read with the footnote counter the block's notes stepped before it (review rev78o2
/// finding 3; 2201.11183, 2111.13260): `\footnotemark[\arabic{footnote}]` after one `\thanks` prints the first
/// footnote symbol, as in LaTeX, so Bob Baker shows (and requests) the mark of Ann Able's note — not the empty mark of
/// the unstepped counter.
#[test]
fn footnotemark_number_reads_the_stepped_counter() {
  assert_eq!(
    article(
      "",
      r"\author{Ann Able\thanks{Equal.} \and Bob Baker\footnotemark[\arabic{footnote}]}"
    ),
    [
      "C Ann Able [] | line:names",
      "  n Equal. | note:adjacent",
      "C Bob Baker [*] | line:names sep:column",
    ]
  );
}

/// The switch still decides nothing with the resolver in: the XML is the same on and off over the shapes above.
#[test]
fn resolver_leaves_the_xml_unchanged() {
  for tex in [
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{\\dag}$ \\and Bob Baker$^{\\ddag}$\\\\ \\dag Univ A\\\\ \\ddag Univ B}\n\\maketitle\nBody.\n\\end{document}\n",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jheppub_emailadd_in_author_order.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_comma_before_inst_is_the_name_before.tex"
    ),
    NOTE_URL,
    IEEE_LEGENDS,
    AUTHBLK_THENOTE,
    AMS_GRID,
  ] {
    let run = |on: bool| {
      convert_with_setup_then(
        tex,
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

// ---- FM S2a.2 r2: the defect classes of review revfms2a2 (each RED on r1 eac47a244e) ----

/// A note with a link: "Code is available at \url{…}." (2401.17399, 1607.05792, 2603.01292).
const NOTE_URL: &str = "\\documentclass{article}\n\\usepackage{url}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{Code is available at \\protect\\url{https://x.org/c}.} \\and Bob Baker\\thanks{\\protect\\url{https://b.org}}}\n\\maketitle\nBody.\n\\end{document}\n";

/// IEEEtran's numbered `\thanks` legends (1909.02288, 2608.06122, 1903.10543, 2407.19826).
const IEEE_LEGENDS: &str = "\\documentclass[journal]{IEEEtran}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1}$ and Bob Baker$^{2}$\\thanks{$^{1}$Dept A, Univ X.}\\thanks{$^{2}$Dept B, Univ Y.}}\n\\maketitle\nText.\n\\end{document}\n";

/// authblk with a letter label: Bob shows `\AB@thenote` "1" (2605.25017's shape).
const AUTHBLK_THENOTE: &str = "\\documentclass{article}\n\\usepackage{authblk}\n\\begin{document}\n\\title{T}\n\\author[a]{Ann}\n\\author{Bob}\n\\affil[a]{Univ X}\n\\affil{Univ Y}\n\\maketitle\nBody.\n\\end{document}\n";

/// amsart's `\author{A and B}` with the contacts after (2205.02163).
const AMS_GRID: &str = "\\documentclass{amsart}\n\\begin{document}\n\\title{T}\n\\author{Ann Able and Bob Baker}\n\\address{Univ A}\n\\email{aable@x.org}\n\\address{Univ B}\n\\email{bbaker@y.org}\n\\maketitle\nText.\n\\end{document}\n";

/// Item 1: a note keeps its whole text (2401.17399, 1607.05792, 2603.01292). (FM S2a.2 r8, main-session decision 1 on
/// r7: its url is no one's contact unless its page spells an author — neither `https://x.org/c` nor `https://b.org`
/// does, so both stay in their notes, and Bob's is not emptied; r2-r7 made each its note's owner's as the only link.)
#[test]
fn notes_keep_their_whole_text() {
  assert_eq!(tuples(&resolved(NOTE_URL, Some("ar5iv.sty"))), [
    "C Ann Able [] | line:names",
    "  n Code is available at https://x.org/c . | note:adjacent",
    "C Bob Baker [] | line:names sep:column",
    "  n https://b.org | note:adjacent",
  ]);
}

/// Item 2: svjour's `\institute{<name> \at <affiliation>}` labels its entry `fuzzy:<name>`, one label with the name's
/// spaces (0807.3898, 2403.09642: split at its spaces, every entry went to the block) — matched by the name's letters.
#[test]
fn fuzzy_labels_stay_whole() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{svjour3}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\and Bob M. Baker}\n\\institute{Ann Able \\at Univ A, Dept X \\and Bob M. Baker \\at Univ B}\n\\maketitle\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [] | line:names",
      "  a Univ A, Dept X | api:label ruling1:fuzzy-label",
      "C Bob M. Baker [] | line:names",
      "  a Univ B | api:label ruling1:fuzzy-label",
    ]
  );
}

/// Item 3: a contact placed by its author's role-sequence label `author:N` (place.rs: an IEEE e-mail line in author
/// order, 1804.09680; revtex astro-ph0611539) goes to the N-th author exactly; a shared affiliation's number never reaches
/// an author by it.
#[test]
fn author_sequence_labels_place_their_contacts() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass[conference]{IEEEtran}\n\\begin{document}\n\\title{T}\n\\author{\\IEEEauthorblockN{Ann Able$^1$, Bob Baker$^{1,2}$ and Cat Cole$^1$}\n\\IEEEauthorblockA{$^1$Univ A, City \\\\ $^2$Univ B, Town \\\\ \\{aable, bbaker, ccole\\}@x.edu}}\n\\maketitle\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [1] | line:names",
      "  a Univ A, City | api:leading-mark",
      "  c email aable@x.edu | api:label",
      "C Bob Baker [1,2] | line:names sep:comma",
      "  a Univ A, City | api:leading-mark",
      "  a Univ B, Town | api:leading-mark",
      "  c email bbaker@x.edu | api:label",
      "C Cat Cole [1] | line:names sep:word",
      "  a Univ A, City | api:leading-mark",
      "  c email ccole@x.edu | api:label",
    ]
  );
}

/// Item 4: Perl's prefix fallback (Base_Utility.pool.ltxml:891-900): elsarticle's `\fnref{l1}` requests `fn:l1`,
/// which answers `\address[l1]` past the prefix (1712.00945; pdflatex prints footnote 1 there and the address apart,
/// today's XML and the shadow read the misuse as Perl does).
#[test]
fn prefix_fallback_answers_a_mismatched_label() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{elsarticle}\n\\begin{document}\n\\begin{frontmatter}\n\\title{T}\n\\author{Ann Able\\fnref{l1}}\n\\author{Bob Baker}\n\\address[l1]{Univ A}\n\\address{Univ B}\n\\end{frontmatter}\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [fn:l1] | line:names",
      "  a Univ B | api:annotate:run",
      "  a Univ A | api:label",
      "C Bob Baker [] | line:names",
      "  a Univ B | api:annotate:run",
    ]
  );
}

/// Item 5: a `\thanks` opening with a Mark is the legend of the label it prints — read from its leading Mark event,
/// not the flattened text, where `$^{1}$Dept` is the word "1Dept" (1909.02288).
#[test]
fn note_legends_by_their_leading_mark() {
  assert_eq!(tuples(&resolved(IEEE_LEGENDS, Some("ar5iv.sty"))), [
    "C Ann Able [1] | line:names",
    "  a Dept A, Univ X. | note:adjacent note:legend ruling1:request",
    "C Bob Baker [2] | line:names sep:word",
    "  a Dept B, Univ Y. | note:adjacent note:legend ruling1:request",
  ]);
}

/// Item 6: a later line of a column is names only with evidence; "Capital Fund Management" and "Google DeepMind" under
/// a name are its affiliation lines (2606.07382, 2609.09054), the e-mail its own.
#[test]
fn institution_lines_are_not_names() {
  assert_eq!(
    article(
      "",
      r"\author{Ann Able \\ Capital Fund Management \\ 23 Rue X, Paris \and Bob Baker \\ Google DeepMind \\ bob@g.com}"
    ),
    [
      "C Ann Able [] | line:names",
      "  a Capital Fund Management | line:unmarked R12:row-above",
      "  a 23 Rue X, Paris | line:unmarked R12:row-above",
      "C Bob Baker [] | line:names sep:column",
      "  a Google DeepMind | line:unmarked R12:row-above",
      "  c email bob@g.com | owner:spelled ruling3:one-per-print",
    ]
  );
}

/// Item 8: the call's declared role wins (amsart's `\email{\href{mailto:…}{…}}`, 2506.17875), and a url printing one
/// e-mail address is an e-mail (1409.0496).
#[test]
fn contact_roles_follow_the_call_and_the_address() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{amsart}\n\\usepackage{url}\n\\usepackage{hyperref}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\address{Univ A}\n\\email{\\href{mailto:ann@x.org}{ann@x.org}}\n\\maketitle\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [] | line:names",
      "  a Univ A | api:annotate:last",
      "  c email ann@x.org | api:annotate:last",
    ]
  );
  assert_eq!(
    article(
      r"\usepackage{url}",
      r"\author{Ann Able \\ Univ A \\ \url{ann@x.org}}"
    ),
    [
      "C Ann Able [] | line:names",
      "  a Univ A | line:unmarked R12:row-above",
      "  c email ann@x.org | line:contact R12:row-above",
    ]
  );
}

/// Item 9: marks inside a line: `($^1$)` leads its line (1004.4809); a mark after a space opens an entry of its own
/// (`$^1$Inst A $^2$Inst B`, 2601.20069); marks trailing their text, each shown by a name elsewhere, label them
/// (2507.05750).
#[test]
fn marks_mid_line_and_trailing_label_entries() {
  assert_eq!(
    article(
      "",
      r"\author{Ann Able$^{1}$, Bob Baker$^{2}$ \\ ($^1$) Univ A \\ $^1$Inst A $^2$Inst B \\ Lab C$^{1}$, Lab D$^{2}$}"
    ),
    [
      "C Ann Able [1] | line:names",
      "  a Univ A | ruling1:request",
      "  a Inst A | ruling1:request",
      "  a Lab C | ruling1:request mark:trailing",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Inst B | ruling1:request",
      "  a Lab D | ruling1:request mark:trailing",
    ]
  );
}

/// Item 11: authblk's notes — with a letter label given the marks show, Bob shows `\AB@thenote` "1" and the unlabelled
/// `\affil{Univ Y}` is "1": Bob's alone ("Anna and Bob1", review revfms2a2's answer to question 6b; authblk.sty:114-118,
/// 148-151).
#[test]
fn authblk_unlabelled_affil_is_its_thenote() {
  let json = resolved(AUTHBLK_THENOTE, Some("ar5iv.sty"));
  assert_eq!(tuples(&json), [
    "C Ann [a] | line:names",
    "  a Univ X | api:label",
    "C Bob [1] | api:creator",
    "  a Univ Y | api:thenote ruling1:request",
  ]);
  assert_eq!(json["policies"][0][1], "authblk");
}

/// Item 11: the policy key falls back to the document class's binding when `\author` is the kernel's — report's title
/// page prints its notes in arabic, so Bob's `\footnotemark` after one `\thanks` is "2" (report.cls:211).
#[test]
fn report_title_page_marks_are_arabic() {
  let json = resolved(
    "\\documentclass{report}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{First.} \\and Bob Baker\\footnotemark}\n\\maketitle\nBody.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(tuples(&json), [
    "C Ann Able [] | line:names",
    "  n First. | note:adjacent",
    "C Bob Baker [2] | line:names sep:column",
  ]);
  assert_eq!(json["policies"][0][1], "report");
}

/// A second kernel `\author` replaces the authors before it in the queue (2502.11323, 2509.11381): the latest's alone.
#[test]
fn a_replacing_author_call_keeps_the_latest() {
  assert_eq!(
    article(
      "",
      "\\author{Ann Able \\and Bob Baker}\n\\author{Ann Able\\thanks{Funded.} \\and Bob Baker}"
    ),
    [
      "C Ann Able [] | line:names",
      "  n Funded. | note:adjacent",
      "C Bob Baker [] | line:names sep:column",
    ]
  );
}

/// Today's amsart contact grid (insert.rs `distribute_upfront_contacts`, html_feedback#46): the j-th address and
/// e-mail pair is the j-th author's (2205.02163).
#[test]
fn ams_contact_grid_is_distributed() {
  assert_eq!(tuples(&resolved(AMS_GRID, Some("ar5iv.sty"))), [
    "C Ann Able [] | line:names",
    "  a Univ A | api:annotate:last owner:upfront-grid",
    "  c email aable@x.org | api:annotate:last owner:upfront-grid",
    "C Bob Baker [] | line:names sep:word",
    "  a Univ B | api:annotate:last",
    "  c email bbaker@y.org | api:annotate:last",
  ]);
}

// ---- FM S2a.2 r3: the classified bucket's wrong classes and the pilot families' residuals (each RED on r2 eb506a0d28) ----

/// A document of class `class` (and `preamble`) with the front matter `front` and an abstract when the class wants
/// one: its resolved tuples.
fn document(class: &str, preamble: &str, front: &str) -> Vec<String> {
  tuples(&resolved(
    &format!(
      "\\documentclass{{{class}}}\n{preamble}\n\\begin{{document}}\n\\title{{T}}\n{front}\n\\maketitle\nText.\n\\end{{document}}\n"
    ),
    Some("ar5iv.sty"),
  ))
}

/// Item 1a: an API affiliation holding several marked entries is parted by its marks, each entry its own leading
/// mark's (cond-mat0510562, 1612.09425, 1809.03738, hep-ph0408045, 2509.08362, 1506.00848: the whole went to the first
/// mark's carriers) — the marks at a line's start, after a break, or after a separator.
#[test]
fn api_affiliation_parts_by_its_marks() {
  assert_eq!(
    document(
      "revtex4-2",
      "",
      "\\author{Ann Able$^{*}$, Bob Baker$^{+}$, and Cat Cole$^{*}$}\n\\affiliation{$^{*}$Univ A, City,\\\\$^{+}$Univ B, Town}"
    ),
    [
      "C Ann Able [*] | line:names",
      "  a Univ A, City | api:leading-mark api:split-by-marks",
      "C Bob Baker [+] | line:names sep:comma",
      "  a Univ B, Town | api:leading-mark api:split-by-marks",
      "C Cat Cole [*] | line:names sep:word",
      "  a Univ A, City | api:leading-mark api:split-by-marks",
    ]
  );
  assert_eq!(
    document(
      "revtex4-2",
      "",
      "\\author{Ann Able$^{1}$, Bob Baker$^{2}$, and Cat Cole$^{1,2}$}\n\\affiliation{$^{1}$Univ A, $^{2}$Univ B}"
    ),
    [
      "C Ann Able [1] | line:names",
      "  a Univ A | api:leading-mark",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ B | api:leading-mark",
      "C Cat Cole [1,2] | line:names sep:word",
      "  a Univ A | api:leading-mark",
      "  a Univ B | api:leading-mark",
    ]
  );
}

/// Item 1b: a `fuzzy:` label that spells no name whole answers the creators its names' surnames name (today's OD #159
/// reading: `A. Able \at …`, `Corresponding author: B. M. Baker \at …`; 2011.14024, 2403.19676, 2106.12864), and a
/// fuzzy label holding a macro is read with the macro written out — aastex's `\email` after `\correspondingauthor`
/// labels it `fuzzy:\lx@aas@correspondent` (2506.07978: it spelled no one and went to the block).
#[test]
fn fuzzy_labels_by_surname_and_with_their_macros() {
  assert_eq!(
    tuples(&resolved(
      "\\documentclass{svjour3}\n\\begin{document}\n\\title{T}\n\\author{Ann Able \\and Bob M. Baker}\n\\institute{A. Able \\at Univ A \\\\ \\email{ann@x.org} \\and Corresponding author: B. M. Baker \\at Univ B}\n\\maketitle\nText.\n\\end{document}\n",
      Some("ar5iv.sty"),
    )),
    [
      "C Ann Able [] | line:names",
      "  a Univ A | api:label ruling1:fuzzy-label fuzzy:surname",
      "  c email ann@x.org | api:label api:inner-contact ruling1:fuzzy-label fuzzy:surname",
      "C Bob M. Baker [] | line:names",
      "  a Univ B | api:label ruling1:fuzzy-label fuzzy:surname",
    ]
  );
  assert_eq!(tuples(&resolved(AAS_CORRESPONDENT, Some("ar5iv.sty"))), [
    "C Ann Able [] | line:names",
    "  a Univ A | api:annotate:new",
    "C Bob Baker [] | line:names",
    "  a Univ B | api:annotate:new",
    "  c correspondent Bob Baker | api:label ruling1:fuzzy-label",
    "  c email bbaker@y.org | api:label ruling1:fuzzy-label",
    "C Sanhueza, Patricio [] | line:names",
    "  a Univ C | api:annotate:new",
  ]);
}

/// AASTeX 7: a corresponding author's `\email` after the authors (2506.07978's shape), and an `\author{Surname,
/// Given}` (2604.19097).
const AAS_CORRESPONDENT: &str = "\\documentclass{aastex701}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\affiliation{Univ A}\n\\email{ann@x.org}\n\\author{Bob Baker}\n\\affiliation{Univ B}\n\\email{bbaker@y.org}\n\\author{Sanhueza, Patricio}\n\\affiliation{Univ C}\n\\email{ps@z.org}\n\\correspondingauthor{Bob Baker}\n\\email{bbaker@y.org}\n\\begin{abstract}\nAbs.\n\\end{abstract}\n\\section{S}\nText.\n\\end{document}\n";

/// Item 1c: glued labels part by the marks the other side shows — `12` under entries `1` and `2` (2103.10685), `1⋆`
/// (2606.01911) — each found owner tagged `ruling1:parted`; and a mark glued to the mark before it goes where that one
/// went (`Oh,$^{1}$$^{,2}$ Nordlander`, mn2e, 2306.15492). (FM S2a.2 r7: the `⋆` line's text is a legend, a note of
/// the name showing the mark — a legend's mark is a symbol, a mark glyph of no `\@fnsymbol` kind too, 2401.00056.)
#[test]
fn glued_marks_part_by_the_marks_shown() {
  assert_eq!(
    article(
      "",
      "\\author{Ann Able$^{12}$, Bob Baker$^{1\\star}$, Cat Cole$^{2}$\\\\ $^1$Univ A\\\\ $^2$Univ B\\\\ $^\\star$Equal contribution.}"
    ),
    [
      "C Ann Able [1,2] | line:names mark:parted",
      "  a Univ A | ruling1:request ruling1:parted",
      "  a Univ B | ruling1:request ruling1:parted",
      "C Bob Baker [1,⋆] | line:names sep:comma mark:parted",
      "  a Univ A | ruling1:request ruling1:parted",
      "  n Equal contribution. | ruling1:request ruling1:parted",
      "C Cat Cole [2] | line:names sep:comma",
      "  a Univ B | ruling1:request",
    ]
  );
  assert_eq!(
    article(
      "",
      "\\author{A. Able,$^{1}$$^{,2}$ B. Baker,$^{1}$$^{,2}$ C. Cole$^{1}$\\\\ $^1$Univ A\\\\ $^2$Univ B}"
    ),
    [
      "C A. Able [1,2] | line:names mark:after-comma",
      "  a Univ A | ruling1:request",
      "  a Univ B | ruling1:request",
      "C B. Baker [1,2] | line:names sep:comma mark:after-comma",
      "  a Univ A | ruling1:request",
      "  a Univ B | ruling1:request",
      "C C. Cole [1] | line:names sep:comma",
      "  a Univ A | ruling1:request",
    ]
  );
}

/// Item 1d: a note's e-mail goes to the author it spells before the one the note is on (FIDELITY_CHARTER §4:
/// ownership by evidence; 1908.04020, 2304.00435), else stays with the note's author.
#[test]
fn a_notes_email_goes_by_spelling() {
  assert_eq!(
    article(
      "\\usepackage{hyperref}",
      "\\author{Ann Able\\thanks{Univ A. \\href{mailto:ann.able@x.org}{ann.able@x.org}, \\href{mailto:cat.cole@x.org}{cat.cole@x.org}} \\and Bob Baker\\thanks{Univ B. \\href{mailto:bob.baker@y.org}{bob.baker@y.org}} \\and Cat Cole}"
    ),
    [
      "C Ann Able [] | line:names",
      "  c email ann.able@x.org | note:adjacent ruling4:inner-contact",
      "  n Univ A. ann.able@x.org, cat.cole@x.org | note:adjacent",
      "C Bob Baker [] | line:names sep:column",
      "  c email bob.baker@y.org | note:adjacent ruling4:inner-contact",
      "  n Univ B. bob.baker@y.org | note:adjacent",
      "C Cat Cole [] | line:names sep:column",
      "  c email cat.cole@x.org | note:adjacent ruling4:inner-contact owner:spelled",
    ]
  );
}

/// Item 1e: the e-mail addresses a names line prints as text are no names (jmlr2e's `\email` a font switch,
/// 2507.03885: the second address was a creator) — each an e-mail of the line's names by the owner chain.
#[test]
fn addresses_in_a_names_line_are_emails() {
  assert_eq!(
    article(
      "\\def\\email{\\hfill\\small\\sc}\n\\def\\name{\\normalsize\\bf}",
      "\\author{\\name Ann Able \\email {ann@x.org, aable@y.org}\\\\ Univ A\\\\ City}"
    ),
    [
      "C Ann Able [] | line:names",
      "  a Univ A | line:unmarked R12:row-above",
      "  a City | line:unmarked R12:row-above",
      "  c email ann@x.org | owner:spelled ruling3:one-per-print",
      "  c email aable@y.org | owner:spelled ruling3:one-per-print",
    ]
  );
}

/// Item 2 (the pilot families' lexer residuals): a link to an ORCID record is the creator's ORCID (2307.02633,
/// 2304.11780, 1910.01653); a link inside an affiliation is its text (2609.39586); a `\cite` in a name is a mark of it
/// (astro-ph9907221, cond-mat0305028); an environment's arguments are no ink (`\begin{CJK*}{UTF8}{gbsn}`, 2607.05550);
/// a note made before any author is the title's (1603.01892).
#[test]
fn orcid_links_cites_environments_and_title_notes() {
  assert_eq!(
    document(
      "revtex4-2",
      "\\usepackage{hyperref}\n\\newcommand{\\orcidicon}[1]{\\href{https://orcid.org/#1}{\\rule{1ex}{1ex}}}",
      "\\thanks{Supported by the X Foundation.}\n\\author{Ann Able\\orcidicon{0000-0001-7461-3342}}\n\\affiliation{\\href{https://x.org}{Univ A}, City}\n\\author{Bob Baker\\cite{byline}}\n\\affiliation{Univ B, Town}"
    ),
    [
      "C Ann Able [] | line:names",
      "  a Univ A, City | api:annotate:run",
      "  c orcid 0000-0001-7461-3342 | owner:adjacent",
      "C Bob Baker [] | line:names",
      "  a Univ B, Town | api:annotate:run",
      "B thanks Supported by the X Foundation. | api:owner:block api:title-note",
    ]
  );
  assert_eq!(
    document(
      "revtex4-2",
      "\\usepackage{CJKutf8}",
      "\\author{Jiayi Zhang (\\begin{CJK*}{UTF8}{gbsn}张嘉懿\\end{CJK*})}\n\\affiliation{Univ A}"
    ),
    [
      "C Jiayi Zhang ( 张嘉懿) [] | line:names name:layout-joined",
      "  a Univ A | api:annotate:run",
    ]
  );
}

/// Item 2: revtex names with their marks before them on the block's only line, the affiliations its calls
/// (1805.08625, 1405.2400, 2001.00788, 0710.4115: no creator was made); a sole author's orphans are that author's, as
/// today's (OD #159: amsart's `\address` and `\dedicatory` before the `\author`, 0911.5235, 0712.4133) — in source order
/// (FM S2a.2 r8, review revfms2a2_7 finding 1: the `\dedicatory` before the `\email`).
#[test]
fn prefix_marks_on_a_sole_line_and_a_sole_authors_orphans() {
  assert_eq!(
    document(
      "revtex4-2",
      "",
      "\\author{$^{1}$Ann Able and $^{2,3}$Bob Baker}\n\\affiliation{$^{1}$Univ A}\n\\affiliation{$^{2}$Univ B}\n\\affiliation{$^{3}$Univ C}"
    ),
    [
      "C Ann Able [1] | line:names mark:prefix",
      "  a Univ A | api:leading-mark",
      "C Bob Baker [2,3] | line:names mark:prefix",
      "  a Univ B | api:leading-mark",
      "  a Univ C | api:leading-mark",
    ]
  );
  assert_eq!(
    document(
      "amsart",
      "",
      "\\address{Dept of Math, Univ A, City}\n\\dedicatory{To Cat.}\n\\author{Ann Able}\n\\email{aable@x.org}"
    ),
    [
      "C Ann Able [] | line:names",
      "  a Dept of Math, Univ A, City | api:owner:block owner:sole-author",
      "  c dedicatory To Cat. | api:owner:block owner:sole-author",
      "  c email aable@x.org | api:annotate:last",
    ]
  );
}

/// Item 3 (the largest buckets' resolver defects): a list with one address per author that every spelled address
/// agrees with is in author order (FIDELITY_CHARTER §4, 2026-10-21; `{qkv, riven, …}@zju.edu.cn`, 2603.19610,
/// 2512.22629); "the MAST Team" among the names is an author (1609.09534, astro-ph0509500).
#[test]
fn address_lists_in_author_order_and_collaborations() {
  assert_eq!(
    article(
      "",
      "\\author{Quan Kong, Yuhao Shen, Yicheng Ji, Huan Li\\\\ Zhejiang University\\\\ \\{qkv, riven, jiyicheng.cs, lihuan.cs\\}@zju.edu.cn}"
    ),
    [
      "C Quan Kong [] | line:names",
      "  a Zhejiang University | line:unmarked R12:row-above",
      "  c email qkv@zju.edu.cn | owner:order ruling3:one-per-print",
      "C Yuhao Shen [] | line:names sep:comma",
      "  a Zhejiang University | line:unmarked R12:row-above",
      "  c email riven@zju.edu.cn | owner:order ruling3:one-per-print",
      "C Yicheng Ji [] | line:names sep:comma",
      "  a Zhejiang University | line:unmarked R12:row-above",
      "  c email jiyicheng.cs@zju.edu.cn | owner:spelled ruling3:one-per-print",
      "C Huan Li [] | line:names sep:comma",
      "  a Zhejiang University | line:unmarked R12:row-above",
      "  c email lihuan.cs@zju.edu.cn | owner:spelled ruling3:one-per-print",
    ]
  );
  assert_eq!(
    article(
      "",
      "\\author{Ann Able$^{1}$, Bob Baker$^{1}$, and the MAST Team\\\\ $^{1}$Univ A}"
    ),
    [
      "C Ann Able [1] | line:names",
      "  a Univ A | ruling1:request",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A | ruling1:request",
      "C the MAST Team [] | line:names sep:word",
    ]
  );
}

/// The r3 shapes leave the XML byte-identical and clean with the switch on and off.
#[test]
fn r3_shapes_leave_the_xml_unchanged() {
  for tex in [
    AAS_CORRESPONDENT,
    "\\documentclass{revtex4-2}\n\\begin{document}\n\\title{T}\n\\thanks{Supported.}\n\\author{$^{1}$Ann Able and $^{2}$Bob Baker}\n\\affiliation{$^{1}$Univ A, $^{2}$Univ B}\n\\maketitle\nText.\n\\end{document}\n",
  ] {
    let run = |on: bool| {
      convert_with_setup_then(
        tex,
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

/// The creators of a reading: each `C` line's name.
fn reading_names(tuples: &[String]) -> Vec<String> {
  tuples
    .iter()
    .filter_map(|t| t.strip_prefix("C "))
    .map(|t| t.split(" [").next().unwrap_or(t).to_string())
    .collect()
}

/// FM S2b ams gate (2026-10-25): a byline's "by" opening a names line is no name, as today's split reads it
/// (`without_leading_by`). Red: the reading's first name was "by David Harari" (0808.2556).
#[test]
fn a_bylines_by_is_no_name() {
  let names = reading_names(&document(
    "amsart",
    "",
    "\\author{by Ann Able and Bob Baker}",
  ));
  assert_eq!(names, ["Ann Able", "Bob Baker"]);
}

/// FM S2b ams gate: a lowercase superscript glued inside a word is the word's letters, not a mark between two names —
/// `M\textsuperscript{c}Kernan` (math0610203). Red: the reading parted "James M" from "Kernan".
#[test]
fn a_superscript_inside_a_word_is_the_name() {
  let names = reading_names(&document(
    "amsart",
    "",
    "\\author{Ann Able}\n\\author{James M\\textsuperscript{c}Kernan}",
  ));
  assert_eq!(names, ["Ann Able", "James McKernan"]);
}

/// Each creator of the reading with its affiliations: `<name>: <affiliation>; <affiliation>`.
fn reading_affiliations(tuples: &[String]) -> Vec<String> {
  let mut out: Vec<String> = Vec::new();
  for t in tuples {
    if let Some(c) = t.strip_prefix("C ") {
      out.push(format!("{}:", c.split(" [").next().unwrap_or(c)));
    } else if let Some(a) = t.strip_prefix("  a ")
      && let Some(last) = out.last_mut()
    {
      let text = a.split(" | ").next().unwrap_or(a);
      last.push_str(if last.ends_with(':') { " " } else { "; " });
      last.push_str(text);
    }
  }
  out
}

/// FM S2b ams precision sample (2026-10-25): a list of symbol marks leading an address, `$\dagger,\;\ast$`, requests
/// each of them, the address text starting after the list. Red: only `†` was read, the address kept ", Univ A" and
/// Cat Cole (∗) got nothing (math-ph0404036).
#[test]
fn a_leading_list_of_symbol_marks_requests_each() {
  assert_eq!(
    reading_affiliations(&document(
      "amsart",
      "",
      "\\author{Ann Able$^{\\dagger}$}\n\\author{Bob Baker$^{\\ddagger}$}\n\\author{Cat Cole$^{\\ast}$}\n\\address{$\\dagger,\\;\\ast$\\;Univ A, City}\n\\address{$\\ddagger$ Univ B, Town}"
    )),
    [
      "Ann Able: Univ A, City",
      "Bob Baker: Univ B, Town",
      "Cat Cole: Univ A, City"
    ]
  );
}

/// FM S2b ams precision sample: a range of marks set with an en-dash (`$^{1\text{--}3}$`, `\textsuperscript{1--3}`)
/// requests every number in it, as a hyphen's does (80v `spelled_ranges`). Red: only its ends, 1 and 3, were requested
/// and 2 went to the other name alone (2507.06232's `$^{1\text{--}5}$`).
#[test]
fn an_en_dash_range_requests_every_number_in_it() {
  for marks in ["$^{1\\text{--}3}$", "\\textsuperscript{1--3}"] {
    assert_eq!(
      reading_affiliations(&document(
        "amsart",
        "",
        &format!(
          "\\author{{Ann Able{marks} and Bob Baker$^{{2}}$}}\n\\address{{$^1$Univ A\\\\ $^2$Univ B\\\\ $^3$Univ C}}"
        )
      )),
      ["Ann Able: Univ A; Univ B; Univ C", "Bob Baker: Univ B"],
      "{marks}"
    );
  }
}

/// FM S2b ams precision sample: amsart's `\address[<author>]{…}` names its author, printed in parentheses before the
/// address ("(MARTIN BRÜCKERHOFF) UNIVERSITÄT MÜNSTER"), so the address is that author's wherever it is written — here
/// inside the `\author` argument after each name. Red: the optional argument was read as a label no mark requests and
/// both addresses went to the block (2103.03620).
#[test]
fn an_ams_address_naming_its_author_is_that_authors() {
  assert_eq!(
    reading_affiliations(&document(
      "amsart",
      "",
      "\\author{Ann Able \\address[Ann Able]{Univ A, City} \\email{ann@a.edu} \\hspace*{0.5cm} Bob Baker \\address[Bob Baker]{Univ B, Town} \\email{bob@b.edu}}"
    )),
    ["Ann Able: Univ A, City", "Bob Baker: Univ B, Town"]
  );
}

/// FM S2b ams gate: a note mark of another kind on a name — endnotes.sty's `\endnotemark[1]`, the kernel's
/// `\lx@notemark{endnote}[1]` — is a mark showing its printed tag, its type argument no ink. Red: the names read "Ann
/// Able endnote" and a lone "endnote" name (1703.08733).
#[test]
fn an_endnote_mark_on_a_name_is_no_name_text() {
  let names = reading_names(&document(
    "amsart",
    "\\usepackage{endnotes}",
    "\\author{Ann Able\\protect\\endnotemark[1], Bob Baker\\protect\\endnotemark[1]$^,$\\protect\\endnotemark[2]}",
  ));
  assert_eq!(names, ["Ann Able", "Bob Baker"]);
}

/// FM S2b ams precision sample: a labelled address no name requests that opens with an author's whole name (`$^1$Lior
/// Alon, School of Mathematics, …` under unmarked names, 2106.06096) is that author's, as the page reads it. Red: both
/// went to the block (`label:unrequested`), where today's output gives each its author.
#[test]
fn an_unrequested_address_opening_with_a_name_is_that_authors() {
  assert_eq!(
    reading_affiliations(&document(
      "amsart",
      "",
      "\\author{Ann Able}\n\\author{Bob Baker}\n\\address{$^1$Ann Able, Univ A, City}\n\\address{$^2$Bob Baker, Univ B, Town}"
    )),
    [
      "Ann Able: Ann Able, Univ A, City",
      "Bob Baker: Bob Baker, Univ B, Town"
    ]
  );
}

/// FM S2b ams written: a contact call led by a footnote symbol no name carries stays where its call puts it (ruling 2
/// labels affiliation and legend lines, user ruling 2026-10-21 #2). Red (ams written): amsart's `\email{$^\dagger$
/// ann@a.edu}` after Ann's `\author` went to the block, which the writer printed as a nameless creator
/// (`perfect_kernel_batch64zc::typewriter_address_split_keeps_its_link`).
#[test]
fn an_uncarried_symbol_on_a_contact_call_keeps_its_place() {
  let tuples = document(
    "amsart",
    "",
    "\\author{Ann Able}\n\\email{$^\\dagger$ ann@a.edu}\n\\author{Bob Baker}\n\\email{bob@b.edu}",
  );
  let ann = tuples
    .iter()
    .position(|t| t.starts_with("C Ann Able"))
    .expect("Ann");
  let bob = tuples
    .iter()
    .position(|t| t.starts_with("C Bob Baker"))
    .expect("Bob");
  assert!(
    tuples[ann..bob]
      .iter()
      .any(|t| t.starts_with("  c email") && t.contains("ann@a.edu")),
    "{tuples:#?}"
  );
  assert!(!tuples.iter().any(|t| t.starts_with("B ")), "{tuples:#?}");
}

/// FM S2b ams written: a footnote-symbol line under names that show no symbol mark at all is a mark of a kind no name
/// shows (`label:unshown`), so it keeps its call's placement, as an unshown number does — ruling 2 gives a symbol line to
/// the names carrying it, and none does. Red (ams written, census int110m): 1610.06222's `\address{$^*$Also
/// affiliated with …}` and `\address{$\dag$Also affiliated with FAMNIT …}` left Verret for a nameless creator.
#[test]
fn a_symbol_line_under_names_showing_no_symbols_keeps_its_place() {
  let tuples = document(
    "amsart",
    "",
    "\\author{Ann Able}\n\\author{Bob Baker}\n\\address{Univ A}\n\\address{$^*$Also at X}\n\\address{$\\dag$Also at Y}",
  );
  assert!(!tuples.iter().any(|t| t.starts_with("B ")), "{tuples:#?}");
  assert_eq!(reading_affiliations(&tuples), [
    "Ann Able:",
    "Bob Baker: Univ A; Also at X; Also at Y"
  ]);
}
