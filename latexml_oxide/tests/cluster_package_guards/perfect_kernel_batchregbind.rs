//! Guards for REGBIND: the post-cutoff sandbox reruns' new Errors of the bindings (CUTSBX clusters A, D, F).
//! - A binding defines the macros its real class or package defines, which documents and other packages extend with
//!   `\g@addto@macro`: an undefined one stays an Error, as in TeX (OXIDIZED_DESIGN_DIVERGENCES #1036), so the binding,
//!   not the Error, is what changes. One row per binding: each is its own code path (two classes' and four packages'
//!   bindings, revtex's shared helper, icml's version gate).
//! - A binding that defines a list its class prints with the title page prints it too (the text a document adds to it,
//!   dropped silently once the Error went): one row per print site and real paper's shape.
//! - cleveref's binding re-applies the kernel first aid's repair of the raw cleveref.sty's amsmath code (content: an
//!   explicit `\ltx@label` lost its key's first character to the text), for the release it repairs; amsmath's
//!   `begindocument/end` resets `\ltx@label` after an older copy's.
//! - pdfcomment's binding allocates the soulpos registers a document's copy of soulpos's end pass reads.
//!
//! Each converts its repro and asserts the exact Error and Warning counts, every creator's name and contacts in order
//! where the class has a frontmatter, and the elements the addition shows in, whole.

use latexml::util::test::{convert_with, error_count, warning_count};

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

fn run(tex: &str) -> (String, String) { convert_with(tex, Some("ar5iv.sty")) }

struct Row {
  name:     &'static str,
  tex:      &'static str,
  creators: &'static [&'static [&'static str]],
  elements: &'static [&'static str],
}

const BINDING_ROWS: &[Row] = &[
  // oup-authoring-template.cls:813 `\ps@opening` (2606.29739).
  Row {
    name:     "oup-authoring-template",
    tex:      repro!("class-bindings/oup_opening_page_style_is_the_class_macro.tex"),
    creators: &[&["Ann Able"]],
    elements: &[r#"<p xml:id="p1.1">Text.</p>"#],
  },
  // enumerate.sty:66 `\@enhook` (2606.12178, 2606.17008 through misccorr.sty).
  Row {
    name:     "enumerate",
    tex:      repro!("package-bindings/enumerate_enhook_is_the_package_macro.tex"),
    creators: &[],
    elements: &[
      r#"<p xml:id="S0.I1.i1.p1.1">First.</p>"#,
      r#"<p xml:id="p1.1">Hooked.</p>"#,
    ],
  },
  // threeparttable.sty:302-303 `\TPT@defaults` (2605.22976).
  Row {
    name:     "threeparttable",
    tex:      repro!("package-bindings/threeparttable_defaults_is_the_package_macro.tex"),
    creators: &[],
    elements: &[r#"<p xml:id="S0.I1.ix1.p1.1">Note.</p>"#],
  },
];

/// The lists the class or package prints with its title page, which a document extends past the binding's setters: each
/// is printed where the class prints it, each footnote text a note on the document (`\lx@frontmatter@footnotes`), the
/// rest typeset after the frontmatter. One row per binding's print site, and per real paper's shape where it differs.
const PRINTED_LIST_ROWS: &[Row] = &[
  // acmart.cls:1726, :2245-2247: a document's own `\authornote` (2605.28175, its footnote in a group); the footnote
  // count as it was, the abstract's footnote 1, the body's 2 (:2330, 2402.16516).
  Row {
    name:     "acmart",
    tex:      repro!("class-bindings/acmart_note_lists_are_the_class_macros.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A , C"]],
    elements: &[
      r#"<p xml:id="abstract1.1">Abstract.<note mark="1" role="footnote" xml:id="footnote1">"#,
      r#"<pubnote role="note">Corresponding author.</pubnote>"#,
      r#"<p xml:id="p1.1">Text.<note mark="2" role="footnote" xml:id="footnote2">"#,
    ],
  },
  // acmart: `\g@addto@macro\@authornotes` with `\footnotetext[2]` (2605.16813) and two `\footnotetext`s (2606.04448).
  Row {
    name:     "acmart (document footnotes)",
    tex:      repro!("class-bindings/acmart_document_footnotes_in_the_note_lists_print.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A , C"]],
    elements: &[
      r#"<pubnote role="note">Equal contribution.</pubnote>"#,
      r#"<pubnote role="note">Corresponding authors: ann@x.org.</pubnote>"#,
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
  },
  // elsarticle.cls:178-179, :294, printed at :766, :806 (`\end{frontmatter}`): the class's own `\cortext` shape, an
  // `\elsaddress` paragraph, the author's note (OD #1054).
  Row {
    name:     "elsarticle",
    tex:      repro!("class-bindings/elsarticle_note_and_address_lists_are_the_class_macros.tex"),
    creators: &[&["Ann Able", "address=Univ A"]],
    elements: &[
      r#"<note role="note" xml:id="id1">Preprint note.</note>"#,
      r#"<pubnote role="note">Corresponding author.</pubnote>"#,
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
  },
  // elsarticle: cnf.sty's `\cortext` text in `\@cornotes` (2605.08232, 2606.15628), a boxed preprint notice in
  // `\elsaddress` (2605.22682).
  Row {
    name:     "elsarticle (journal style)",
    tex:      repro!(
      "class-bindings/elsarticle_journal_style_lists_print_after_the_frontmatter.tex"
    ),
    creators: &[&["Ann Able", "address=Univ A", "mark=∗"]],
    elements: &[
      ELS_NOTICE,
      ELS_CORRESPONDENCE,
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
  },
  // revtex4-2.cls:2648-2652, :3020-3022: `\prep@absbox` text (2606.06621), `\mini@notes`, `\@FMN@list` (2605.00976).
  Row {
    name:     "revtex4-2",
    tex:      repro!("class-bindings/revtex_frontmatter_note_lists_are_the_class_macros.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A"]],
    elements: &[
      REVTEX_PREP_ABSBOX,
      r#"<pubnote role="note">Mini note.</pubnote>"#,
      r#"<pubnote role="note">Joint work.</pubnote>"#,
      r#"<p xml:id="p1.1">Text.</p>"#,
    ],
  },
  // amsaddr.sty:43, :102-113: a document's own `\email` with marks (2606.23853, 2606.25954, 2212.11488, 1801.06576,
  // 2307.06418), "E-mail addresses" for two `\author`s.
  Row {
    name:     "amsaddr",
    tex:      repro!("package-bindings/amsaddr_emails_is_the_package_macro.tex"),
    creators: &[&["Ann Able", "address=Univ A"], &[
      "Bob Baker",
      "address=Univ B",
    ]],
    elements: &[AMSADDR_EMAILS, r#"<p xml:id="p1.1">Text.</p>"#],
  },
  // icml2026.sty:490-499, :529-530: `\icmlcorrespondingauthor` builds `\icmlcorrespondingauthor@text` (2605.04712,
  // 2605.21190, 2606.01172) and still gives the e-mail contact; what the document added to it is a note.
  Row {
    name:     "icml2026",
    tex:      repro!("package-bindings/icml_corresponding_author_text_is_built.tex"),
    creators: &[&["Ann Able", "affiliation=Univ A", "email=ann@x.org"]],
    elements: &[
      r#"<pubnote role="note">Code at X</pubnote>"#,
      r#"<p xml:id="p1.1">Listed: Ann Able &lt;ann@x.org&gt;. Code at X.</p>"#,
    ],
  },
];

/// 2605.22682's notice, boxed into `\elsaddress`: the author block's note, a sole author's (OD #1054; a body paragraph
/// before).
const ELS_NOTICE: &str = concat!(
  r#"<note role="note" xml:id="id1"><p align="center" vattach="bottom" xml:id="id1.1">"#,
  r#"<text font="bold" fontsize="90%" xml:id="id1.1.1">Authors’ preprint version"#,
  r#"<break cssstyle="--ltx-break-space:4.6pt"/><text font="medium" xml:id="id1.1.1.1">"#,
  "\nThis is the authors’ preprint.</text></text></p></note>"
);
/// cnf.sty's `\cortext` text (2605.08232), a list of its own whose item sets the label `cor:c`: the note of the author
/// it goes to (none requests it: the sole author), marked ∗, its text without it (OD #1054; the block's note before,
/// with the notice).
const ELS_CORRESPONDENCE: &str = concat!(
  r#"<note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id2">"#,
  "Corresponding author: Univ A.</note>"
);
/// 2606.06621's `\prep@absbox` text: the note of a sole author whose name shows no ‡ (OD #1054; a body paragraph before).
const REVTEX_PREP_ABSBOX: &str = concat!(
  r#"<note role="note" xml:id="id1"><p align="center" xml:id="id1.1"><sup xml:id="id1.1.1">"#,
  r#"<text fontsize="90%" xml:id="id1.1.1.1">‡</text></sup>"#,
  r#"<text fontsize="90%" xml:id="id1.1.2"> These authors contributed equally.</text></p></note>"#
);
/// amsaddr's footnote, its marks and addresses as the document gave them (2606.23853).
const AMSADDR_EMAILS: &str = concat!(
  r#"<pubnote role="note"><text xml:id="id1"><text font="italic" xml:id="id1.1">E-mail addresses</text>: </text>("#,
  r#"<Math mode="inline" tex="\dagger" text="dagger" xml:id="m1">"#,
  "\n      <XMath xml:id=\"m1.1\">\n        <XMTok name=\"dagger\" role=\"MULOP\">†</XMTok>\n      </XMath>",
  "\n    </Math>)",
  r#"<text font="typewriter" xml:id="id2"> ann@x.org, </text>("#,
  r#"<Math mode="inline" tex="\ddagger" text="ddagger" xml:id="m2">"#,
  "\n      <XMath xml:id=\"m2.1\">\n        <XMTok name=\"ddagger\" role=\"MULOP\">‡</XMTok>\n      </XMath>",
  "\n    </Math>)",
  r#"<text font="typewriter" xml:id="id3"> bob@x.org</text>."#,
  "\n</pubnote>"
);

fn check(row: &Row) {
  let (log, xml) = run(row.tex);
  let name = row.name;
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 0),
    "{name}: {log}"
  );
  let want: Vec<Vec<String>> = row
    .creators
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{name}: {xml}");
  // (each once, in document order)
  let mut from = 0;
  for element in row.elements {
    assert_eq!(xml.matches(element).count(), 1, "{name}: {element}\n{xml}");
    let at = xml.find(element).unwrap_or_default();
    assert!(at >= from, "{name}: {element} out of order\n{xml}");
    from = at;
  }
}

/// Cluster A: each binding defines what its real class or package defines, so a document's `\g@addto@macro` on it is
/// clean (it was `Error:undefined` since OD #1036 made the addition to an undefined macro an Error, as TeX's).
#[test]
fn bindings_define_the_macros_their_files_define() {
  for row in BINDING_ROWS {
    check(row);
  }
}

/// Cluster A, the lists printed: a binding that defines a list its class prints at `\maketitle` prints it too, so the
/// text a document adds to it is in the output (it was dropped silently once the Error went: review B1).
#[test]
fn bindings_print_the_title_page_lists_their_files_print() {
  for row in PRINTED_LIST_ROWS {
    check(row);
  }
}

/// Cluster D (2605.07565): the first aid's `\ltx@label#1{\cref@label{#1}}` under amsmath 2025-05-18, re-applied after
/// the binding reads the raw cleveref.sty — the display's label whole and its math untouched ("a=b", not "a=b3a"
/// labelled `l`), the text's label taking nothing from the text ("XY", not "X4bY").
#[test]
fn cleveref_ltx_label_braces_its_key_under_amsmath() {
  let (log, xml) = run(repro!(
    "cross-references/cleveref_ltx_label_braces_its_key_under_amsmath.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for element in [
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" labels="LABEL:l4b" xml:id="id1">"#,
    r#"<equation labels="LABEL:l3a" xml:id="S0.E1">"#,
    r#"<Math mode="display" tex="a=b" text="a = b" xml:id="S0.E1.m1">"#,
    r#"<p xml:id="p1.1">XY</p>"#,
  ] {
    assert_eq!(xml.matches(element).count(), 1, "{element}\n{xml}");
  }
}

/// amsmath.sty:1228 lets `\ltx@label` be `\label` at `begindocument/end`, after every `begindocument` code: a v0.19
/// cleveref.sty's `\def\ltx@label#1{\cref@label#1}` there (2508.14614's copy), which the first aid leaves alone (it
/// repairs v0.21.4 only, review S2), is undone as in pdflatex ("XY", the display's math "a=b").
#[test]
fn amsmath_ltx_label_is_label_after_the_begin_document_code() {
  let (log, xml) = run(repro!(
    "cross-references/amsmath_ltx_label_is_label_after_begin_document_code.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for element in [
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" labels="LABEL:l4b" xml:id="id1">"#,
    r#"<equation labels="LABEL:l3a" xml:id="S0.E1">"#,
    r#"<Math mode="display" tex="a=b" text="a = b" xml:id="S0.E1.m1">"#,
    r#"<p xml:id="p1.1">XY</p>"#,
  ] {
    assert_eq!(xml.matches(element).count(), 1, "{element}\n{xml}");
  }
}

/// Cluster F (2606.00625): pdfcomment's binding allocates soulpos's `\ulp@cnt`, `\ulp@out` (soulpos.sty:12-16), which
/// a document's `enddocument/afterlastpage` copy of soulpos's end pass reads.
#[test]
fn pdfcomment_allocates_the_soulpos_registers() {
  let (log, xml) = run(repro!(
    "package-bindings/pdfcomment_allocates_the_soulpos_registers.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml.matches(r#"<p xml:id="p1.1">Text.</p>"#).count(),
    1,
    "{xml}"
  );
}
