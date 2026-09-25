//! Binding singletons (round 12, P4/P5): each binding lacked one piece of
//! the package it stands in for — a `\RequirePackage`, a definition, a
//! parameter shape or a routing — and one corpus manual errored on it. The
//! repros live in `tools/perfect_kernel/repros/singletons/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, convert_files, error_count, warning_count};

/// hvfloat.sty:45 `\RequirePackage{atbegshi}`: the binding omitted it, so
/// `\AtBeginShipoutNext` was undefined (hvfloat fullpage1s1c/fullpage1s2c).
/// Control: a document that loads atbegshi itself stays clean.
#[test]
fn hvfloat_requires_atbegshi() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/singletons/hvfloat_atbegshi.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>Textmore.</p></para>"#,
  );

  let control = tex.replace(
    "\\usepackage{hvfloat}",
    "\\usepackage{atbegshi}\n\\usepackage{hvfloat}",
  );
  let (stderr, xml) = convert(&control, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>Textmore.</p></para>"#,
  );
}

/// `\includepdf`'s file name is Semiverbatim (Perl graphicx.sty.ltxml:52 reads
/// `\includegraphics`'s so): `a_b.pdf` errored "Script _ can only appear in
/// math mode" (latex4wp, `pandoc_template.pdf`; Perl pdfpages.sty.ltxml:30
/// shares the `{}`). Control: the `pages` keyval still prefixes the link.
#[test]
fn includepdf_file_name_is_semiverbatim() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/pdfpages_includepdf_semiverbatim.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "resource",
    &[r#"src="a_b.pdf""#],
    r#"<resource src="a_b.pdf" type="application/pdf"/>"#,
  );
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>See the template.<resource src="a_b.pdf" type="application/pdf"/>See <ref href="a_b.pdf">a_b.pdf</ref></p></para>"#,
  );

  let control = tex.replace("[fitpaper]{a_b.pdf}", "[pages=1-3]{doc.pdf}");
  let (stderr, xml) = convert(&control, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>See the template.<resource src="doc.pdf" type="application/pdf"/>See pages 1-3 of <ref href="doc.pdf">doc.pdf</ref></p></para>"#,
  );
}

/// lstmisc.sty:1236-1239 `\theHlstnumber` (SASnRdisplay.sty:265-275
/// `\SnRHrefNumber` expands it once): the uncaptioned branch, since the
/// binding never sets `\lst@@caption`. Control: defining `\lst@@caption`
/// would claim the binding's `caption` key handler name, so both captions
/// (keyval and `\lstset`) and a `\begingroup\endgroup` in the body must survive.
#[test]
fn listings_the_h_lstnumber_is_defined() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/singletons/listings_theHlstnumber.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>[<text font="typewriter">R-\lst@neglisting .\thelstnumber </text>]</p></para>"#,
  );

  let control = "\\documentclass{article}\n\\usepackage{listings}\n\\begin{document}\n\
    \\begingroup\\endgroup\n\
    \\begin{lstlisting}[caption={Cap A}]\nx = 1\n\\end{lstlisting}\n\
    \\lstset{caption={Cap B}}\n\\begin{lstlisting}\ny = 2\n\\end{lstlisting}\n\\end{document}\n";
  let (stderr, xml) = convert(control, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\": \">Listing\u{a0}1</tag>Cap A</caption>",
  );
  assert_eq!(xml.matches("<caption>").count(), 2, "{xml}");
  assert!(
    xml.contains("<tag close=\": \">Listing\u{a0}2</tag>Cap B</caption>"),
    "{xml}"
  );
}

/// varioref.sty:799-802 `\newif\if@vrefhandlespace`, true by default
/// (`space`); zref-vario.sty:486 clears it (tikz-cookingsymbols-doc `\zvref`).
#[test]
fn varioref_handlespace_switch_exists() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/singletons/varioref_vrefhandlespace.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>[yes][no]</p></para>"#,
  );
}

/// biblatex.def:401-404 `\mkbibacro` (socialscienceshuberlin) and
/// standard.bbx:4-8's toggles, set true by :21 `\ExecuteBibliographyOptions`,
/// which a skipped native style must still allocate (biblatex-ext: 29
/// `Toggle 'bbx:doi' undefined`). Controls: a load-time `doi=false` clears
/// its toggle, the default style (biblatex.sty:16387 `style=numeric`)
/// allocates them too, and so does a raw third-party style that chains
/// standard (ext-standard.bbx `\RequireBibliographyStyle{standard}`).
#[test]
fn biblatex_mkbibacro_and_standard_toggles() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/biblatex_mkbibacro_standard_toggles.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>[doi][isbn][<text font="smallcaps">nasa</text>]</p></para>"#,
  );

  for (options, expected) in [
    ("style=authoryear,doi=false", "[nodoi][isbn]"),
    ("backend=biber", "[doi][isbn]"),
    ("style=ext-authoryear", "[doi][isbn]"),
  ] {
    if options.contains("ext-") && !latexml::util::test::kpse_has("ext-authoryear.bbx") {
      continue;
    }
    let control = tex
      .replace("[style=authoryear]", &format!("[{options}]"))
      .replace("[\\mkbibacro{NASA}]", "");
    let (stderr, xml) = convert(&control, true);
    assert_eq!(error_count(&stderr), 0, "{options}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{options}: {stderr}");
    assert_element(
      &xml,
      "para",
      &[],
      &format!(r#"<para xml:id="p1"><p>{expected}</p></para>"#),
    );
  }
}

/// catchfile.sty:264-306's protocol, which catchfilebetweentags drives by
/// redefining `\CatchFile@Do` and `\everyeof` in `\CatchFileDef`'s setup
/// (factura-ejemplo-prefactura: `undefined:\CatchFile@EOF`, and the whole
/// source file dumped where the tagged text belonged). Controls, each as
/// pdflatex has it: a caught file keeps its final end-of-line space and its
/// parameter characters (`\def\z#1{[#1]}` still defines), and a missing file
/// defines the macro empty with one package error (catchfile.sty:240-245).
#[test]
fn catchfilebetweentags_uses_the_eof_protocol() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/singletons/catchfile_betweentags_eof.tex");
  let doc = tex.replace("\\jobname.tex", "tags.tex");
  let (stderr, xml) = convert_files(&doc, &[("tags.tex", tex)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>[Hello tagged]</p></para>"#,
  );

  let control = "\\documentclass{article}\n\\usepackage{catchfile}\n\\begin{document}\n\
    \\CatchFileDef\\y{defs.tex}{}\\y\\z{a}\n\
    \\CatchFileDef\\w{nonexistent-file.tex}{}[\\meaning\\w]\n\\end{document}\n";
  let (stderr, xml) = convert_files(control, &[("defs.tex", "\\def\\z#1{[#1]}%\nCaught text\n")]);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("Package catchfile Error: File `nonexistent-file.tex' not found"),
    "{stderr}"
  );
  // `\meaning`'s `>` renders through OT1 as `¿`.
  assert_element(
    &xml,
    "para",
    &[],
    "<para xml:id=\"p1\"><p>Caught text\n[a]\n[macro:-¿]</p></para>",
  );
}

/// catchfile.sty:224-238 builds the name through `\IfFileExists`, which
/// expands it, and :251-261 reads it as `\input` does: makron.sty:61's
/// `\CatchFileEdef\tmp{\jobname.runs}{…}` (arXiv 1611.01359) finds the job's
/// file, a filecontents file is caught by both commands, and the `\space`
/// after the name is `\input`'s terminator rather than content. Controls: a
/// missing file is the package's error (catchfile.sty:240-245), and a caught
/// file with an unbalanced brace stops at the file's end — pdflatex's "File
/// ended while scanning use of `\CatchFile@Do`" — so the document after it
/// survives (the scan used to cross into it: `Fatal:Mouth:EoF`).
#[test]
fn catchfile_expands_the_name_and_reads_filecontents() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/singletons/catchfile_jobname_vfs.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>[43][42 ][AFOOB ]</p></para>"#,
  );

  let control = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/catchfile_missing_file_control.tex"
  );
  let (stderr, xml) = convert(control, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("Package catchfile Error: File `nonexistent-file.txt' not found"),
    "{stderr}"
  );
  // `\meaning`'s `>` renders through OT1 as `¿`.
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>[macro:-¿]</p></para>"#,
  );

  let control = "\\documentclass{article}\n\\usepackage{catchfile}\n\
    \\begin{filecontents*}{unbalanced.txt}\nA{B\n\\end{filecontents*}\n\\begin{document}\n\
    Before. \\CatchFileDef\\cu{unbalanced.txt}{}After.\n\nNext paragraph.\n\\end{document}\n";
  let (stderr, xml) = convert(control, true);
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  // The runaway at the file's end, then the recovery from the half-read
  // argument (pdflatex reports five). The warning is the unclosed
  // `\begingroup` of `\CatchFileDef` at `\end{document}` — pdflatex's
  // "(\end occurred inside a group at level 1)".
  assert_eq!(error_count(&stderr), 4, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Error:expected:} Gullet->readBalanced ran out of input"),
    "{stderr}"
  );
  // NOT pdflatex's "Before. After.": tex.web §396 discards the runaway
  // argument, while the kernel's delimited read unreads it (gullet.rs
  // `read_until` "Ran out! Unread", Perl Gullet.pm:683-685 `readUntil`), so the
  // file's `A{B` and the `\CatchFile@EOF` delimiter (`@@`, catcodes 8 and 3:
  // the `_`) are typeset. Pinned as it is; the paragraph after survives.
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>Before. AB\n_After.</p></para>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Next paragraph.</p></para>"#,
  );
}

/// accessibility.sty:1566-1577 (`pdf` option) redefines `\verb` to open an
/// environment and `\verb@egroup` to close it; newverbs.sty:56-67 calls the
/// `\verb` in force and appends to the `\verb@egroup` in force, so both halves
/// run for `\qverb` and a `\newverbcommand` (the binding read the verbatim
/// argument itself but ran `\verb@egroup`: an unmatched `\end`).
#[test]
fn qverb_keeps_a_redefined_verb_pair() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/newverbs_redefined_verb_pair.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>Say <text font="typewriter">‘‘[<verbatim>abc</verbatim>]’’</text> and <text font="typewriter">&lt;[<verbatim>x_y</verbatim>]&gt;</text> here.</p></para>"#,
  );
}

/// ceurart.cls:1477 `\address[label]{text}` and :640 `\ead` (an email, or a
/// URL given an option), the author keyval's email/url keys and :665/:689
/// `\urlauthor`/`\emailauthor` become `<contact>`s inside their `<creator>`
/// (Perl's pattern, aas_support.sty.ltxml:118,122), the affiliation matched by
/// label as in elsarticle — not document-level `<note>`s (witness arXiv
/// 2511.11770).
#[test]
fn ceurart_contacts_attach_to_creators() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/ceurart_email_address_contacts.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Alice Smith</personname><contact name=\"Email:\u{a0}\" role=\"email\">alice@example.org</contact><contact name=\"URL:\u{a0}\" role=\"url\">https://alice.example.org</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Alpha University</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before="],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Jones</personname><contact name=\"Email:\u{a0}\" role=\"email\">bob@example.org</contact><contact name=\"URL:\u{a0}\" role=\"url\">https://bob.example.org</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Beta Institute</contact></creator>",
  );
  assert!(!xml.contains("<note"), "{xml}");
}

/// newverbs.sty:52-69 selects `\verbatim@font` before the `before` code, so
/// both of `\qverb`'s quotes (:100-107) are typewriter (macros2e `"…"`
/// short verb); the opening one was roman. They print in the document's
/// encoding, as pdflatex's cmtt ‘‘abc’’: the verbatim body's ASCII encoding
/// (OXIDIZED_DESIGN #144) covers neither `before` nor `after`.
#[test]
fn qverb_quotes_share_the_verbatim_font() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/singletons/newverbs_qverb_font.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>Say <text font="typewriter">‘‘<verbatim>abc</verbatim>’’</text> here.</p></para>"#,
  );
}

/// feynmf.sty:180 `\def\fmfgraph(#1,#2)`: the `(w,h)` pair is one delimited
/// argument pair, not two single tokens (the note read
/// "(Feynman diagram, (x3)"). feynmp shares the environments (witness arXiv
/// 2309.07343 loads feynmp).
#[test]
fn fmfgraph_reads_its_size_pair() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/singletons/feynmf_graph_pair.tex");
  for tex in [tex.to_string(), tex.replace("{feynmf}", "{feynmp}")] {
    let (stderr, xml) = convert(&tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    assert_element(
      &xml,
      "note",
      &[],
      r#"<note role="feynman-diagram">(Feynman diagram, 30x20)</note>"#,
    );
    assert!(
      xml.contains(r#"<note role="feynman-diagram">(Feynman diagram, 40x25)</note>"#),
      "{xml}"
    );
  }
}
