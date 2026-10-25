//! Red/green guards for batch 80a (the hyphenation languages the format numbers keep their numbers) and 80b (a
//! listings style that is a control sequence runs, with the token in `\lst@token`) and 80c (import.sty's path model) and 80d
//! (ctable's options past a line end, subfig's `\subref`), 80e-80g (the bare-name `\import`, cleveref's braced comma,
//! subeqnarray's `\slabel`, a longtable's labels, autonum), 80h (`\newlabel` records, theoremref), 80i (acronym options,
//! caption3's `\captionbox`, nicematrix's `\tabularnote` label, autonum's reference patcher), 80j (`\pdfstrcmp` on shown
//! token lists, an ignored character keeps the blank skip), 80k (a binding's guessed theorem yields to the document's
//! `\newcommand`, a longtable's label key and a `\cref` list expanded, the conditional guard's Fatal salvaged), 80l
//! (siunitx's unit commands, aa's `\longtab`, an eqnarray's trailing label), 80m (a preloaded class's hook stack) and 80n
//! (a skipped branch ends at a column's v-part, `\maketitle` calls a redefined `\twocolumn`, acronym's protected
//! commands), 80o (iucr.cls's `\hcaption`), 80p (amsmath's `\collect@body`, `\everypar` ahead of a horizontal command, a
//! redefined `\newtheorem` start), 80q (a post-title amsart `\keywords` a brace short), 80r (dgruyter.sty a stand-in).
//! Each guard converts its repro under `tools/perfect_kernel/repros/`, asserts the
//! diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// latex_constructs declares the `\l@<language>` names babel checks for only where the format left them undefined
/// (sect13.rs), so TeX Live's numbering stands: `\l@english` 0, the language TeX starts in, and `\l@nohyphenation`
/// 2. Red (int94a): "other: 90, 94." — every name numbered again past the format's table (KPE #1168). Witness
/// 0802.3014 (languages.sty's "Inconsistent language numbering"). Repro babel-lang/language_numbers_follow_the_format.
#[test]
fn the_format_language_numbers_stand() {
  let (log, xml) = convert_files_with(
    repro!("babel-lang/language_numbers_follow_the_format.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">English: 0,\n2.</p>"),
    "{xml}"
  );
}

/// 80b: a listings style that is a control sequence runs as markup whatever its name ends in, and the token being
/// output is in `\lst@token` while it runs (listings.sty:481, :579-585). Red (int94a): `identifierstyle=\idstyle` was
/// taken for a class `\ids` and never ran, the default style's `}}` stayed after every identifier ("Attempt to close
/// boxing group"), and `\lst@token` was empty (KPE #1169, #1170). Witness 1907.13012. Repro
/// listings-algorithms/listings_identifier_style_reads_the_token.
#[test]
fn a_listings_identifier_style_reads_the_token() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/listings_identifier_style_reads_the_token.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (color, word) in [
    ("#0000FF", "Query"),
    ("#FF0000", "field"),
    ("#FF0000", "value"),
  ] {
    let element = format!(r#"color="{color}""#);
    let at = xml
      .find(&format!(">{word}</text>"))
      .unwrap_or_else(|| panic!("{word}: {xml}"));
    let open = xml[..at].rfind("<text ").unwrap();
    assert!(xml[open..at].contains(&element), "{word} {color}: {xml}");
  }
  assert!(xml.contains("After."), "{xml}");
}

/// 80c: import.sty's own path model (import_sty.rs) — `\import` resets `\import@path`, `\subimport` extends it, the
/// file read is `\import@path <file>` with the enclosing imports' directories still searched — and a directory is no
/// file (pathname.rs, kpathsea's `READABLE`). Red (int94b): the nested `\import{intro}{intro}` read the directory
/// importnest/intro ("Is a directory"), the section, its `\subimport{}{parts/part.tex}` part and the label lost
/// (KPE #1171). Witness 2408.09583. Repro loader/import_nested_absolute_import.
#[test]
fn a_nested_import_reads_its_files_as_import_sty_finds_them() {
  let (log, xml) = convert_files_with(
    repro!("loader/import_nested_absolute_import.tex"),
    &[
      (
        "importnest/chapters.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/importnest/chapters.tex"),
      ),
      (
        "importnest/intro/intro.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/importnest/intro/intro.tex"),
      ),
      (
        "importnest/intro/parts/part.tex",
        include_str!("../../../tools/perfect_kernel/repros/loader/importnest/intro/parts/part.tex"),
      ),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for text in [
    "The introduction text.",
    "A part of the introduction.",
    "Back in the chapters file.",
  ] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
  assert!(xml.contains(r#"labels="LABEL:sec:intro""#), "{xml}");
  assert!(
    xml.contains(r#"<ref labelref="LABEL:sec:intro"/>"#),
    "{xml}"
  );
}

/// 80d: ctable.sty:206's `\newcommand{\ctable}[4][]` finds its options past spaces (`\@ifnextchar[`), so
/// `\begin{ctable}` + a line end + `[caption=…,label=…]` keeps its caption and label. Red (int94c): the native
/// binding's `OptionalKeyVals` stopped at the space, read `[` as the column spec ("Extra alignment tab"), and the label
/// was lost. Witness 1706.01884. Repro package-bindings/ctable_env_options_after_a_line_end.
#[test]
fn a_ctable_environment_reads_its_options_after_a_line_end() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/ctable_env_options_after_a_line_end.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"labels="LABEL:T:res""#), "{xml}");
  assert!(xml.contains("</tag>Results</caption>"), "{xml}");
  assert!(xml.contains(r#"<ref labelref="LABEL:T:res"/>"#), "{xml}");
}

/// 80d: subfig's `\subref`/`\subref*` refer to the sub-float's own label, as subcaption's do. Red (int94c):
/// `\ref{sub@<label>}` (subfig.sty.ltxml:93-94) named the `sub@` entry subfig.sty's `\label` writes, which no sub-float
/// here carries, so every `\subref` dangled (KPE #1172). Witnesses 1802.05913, 2304.09615. Repro
/// cross-references/subfig_subref_names_the_subfloat.
#[test]
fn a_subfig_subref_names_the_subfloat() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/subfig_subref_names_the_subfloat.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for label in ["fig:l", "fig:r"] {
    assert!(
      xml.contains(&format!(r#"labels="LABEL:{label}""#)),
      "{label}: {xml}"
    );
    assert!(
      xml.contains(&format!(r#"<ref labelref="LABEL:{label}"/>"#)),
      "{label}: {xml}"
    );
  }
  assert!(!xml.contains("sub@"), "{xml}");
}

/// 80e: an `\import` whose `\import@path<file>` does not exist reads the bare name, as import.sty before v6 did
/// (`#1{#7}` with the import path first on `\input@path`), so an archive that ships the file beside the main file
/// keeps it. Red (int94c): 80c read only `pictures/importflat/part` (TL 2025's v6.2), "Can't find TeX file" and the
/// part lost. Witnesses 1409.7023, 1605.04692. Repro loader/import_missing_directory_reads_the_bare_name.
#[test]
fn an_import_from_a_missing_directory_reads_the_bare_name() {
  let (log, xml) = convert_files_with(
    repro!("loader/import_missing_directory_reads_the_bare_name.tex"),
    &[(
      "importflat/part.tex",
      include_str!("../../../tools/perfect_kernel/repros/loader/importflat/part.tex"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("A part shipped beside the main file."),
    "{xml}"
  );
}

/// 80f: a comma inside braces is part of a cleveref label: `\Cref{claim:copy_K_{2,4}}` names the one label (cleveref's
/// delimited `\@cref@stack@top#1,#2\@nil` matches only a comma at brace depth 0). Red (int95b): the list was split at
/// the braced comma too (Perl's `splitLabels`), the reference dangled (KPE #1173). Witness 2304.09155. Repro
/// cross-references/cleveref_label_with_a_braced_comma.
#[test]
fn a_cleveref_label_keeps_its_braced_comma() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/cleveref_label_with_a_braced_comma.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The label key drops the braces (`CleanLabel`): `\label{claim:copy_K_{2,4}}` is `LABEL:claim:copy_K_2,4`.
  assert!(
    xml.contains(r#"labels="LABEL:c1 LABEL:claim:copy_K_2,4""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<ref labelref="LABEL:claim:copy_K_2,4" show="creftypecap~creflabel"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<ref labelref="LABEL:c1" show="creftypeplural~creflabel"/>"#),
    "{xml}"
  );
}

/// 80f: subeqnarray's `\slabel` names its row: our rows are subnumbered equations, so it is `\label`. Red (int95b): the
/// raw `\slabel` (subeqnarray.sty:66-75) only writes `\newlabel` to the aux file, so every reference dangled (KPE
/// #1174). Witness 1612.09273. Repro cross-references/subeqnarray_slabel_names_the_row.
#[test]
fn a_subeqnarray_slabel_names_the_row() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/subeqnarray_slabel_names_the_row.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for label in ["bl", "bt"] {
    assert!(
      xml.contains(&format!(r#"labels="LABEL:{label}""#)),
      "{label}: {xml}"
    );
    assert!(
      xml.contains(&format!(r#"<ref labelref="LABEL:{label}"/>"#)),
      "{label}: {xml}"
    );
  }
}

/// 80g: every `\label` in a longtable names the table, which keeps them all. Red (int96a): one `LONGTABLE_LABEL`, each
/// label replacing the one before (longtable.sty.ltxml:134-135; KPE #1175). Witness 2211.06081. Repro
/// cross-references/longtable_keeps_every_label.
#[test]
fn a_longtable_keeps_every_label() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/longtable_keeps_every_label.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"labels="LABEL:a LABEL:b LABEL:c""#), "{xml}");
}

/// 80g: autonum's binding leaves the displays to amsmath's binding, so a `{split}` row in `{align}` does not leave the
/// display's `\label` behind it. Red (int96a): the raw package renewed `{split}` but not our locked `{align}`, whose
/// renewal restores `\label`; the next `\label` was undefined and lost (OXIDIZED_DESIGN_DIVERGENCES #895). Witness
/// 1905.00606. The repro's own `\autonum@generatePatchedReference{autoref}` (2211.09537) was undefined until 80i.
/// Repro cross-references/autonum_split_in_align_keeps_later_labels.
#[test]
fn autonum_leaves_the_displays_their_labels() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/autonum_split_in_align_keeps_later_labels.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for label in ["eq:one", "eq:two", "sec:b"] {
    assert!(
      xml.contains(&format!(r#"labels="LABEL:{label}""#)),
      "{label}: {xml}"
    );
  }
}

/// 80h: a `\newlabel` record names what its references print, whether the document makes it, writes it to the `.aux`
/// by `\protected@write` or by `\write` (LaTeX reads the file back at the next `\begin{document}`). Red (int97a): Perl's
/// aux stubs — `\newlabel` a no-op, `\protected@write` writing nothing, the written text never read — left all three
/// missing (OXIDIZED_DESIGN_DIVERGENCES #896). Witnesses 2503.03625, 1004.2872, 1608.05314. Repro
/// cross-references/newlabel_records_name_their_references.
#[test]
fn a_newlabel_record_names_its_references() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/newlabel_records_name_their_references.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (label, text) in [("tc:one", "TC-1"), ("eq:ph", "7"), ("ext:fig", "S3")] {
    assert!(
      xml.contains(&format!(
        r#"<ref class="ltx_ref_external" labelref="LABEL:{label}">{text}</ref>"#
      )),
      "{label}: {xml}"
    );
  }
  assert!(xml.contains(r#"<ref labelref="LABEL:sec:a"/>"#), "{xml}");
}

/// 80h: theoremref's `\thlabel` labels the theorem and `\thref` names it by its type and number. Red (int97a): raw,
/// `\thlabel` only wrote its `.aux` record, so every `\thref` printed "?THM?" before a missing label. Witness
/// 1211.4329. Repro cross-references/theoremref_thref_names_the_theorem.
#[test]
fn theoremref_thref_names_the_theorem() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/theoremref_thref_names_the_theorem.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("?THM?"), "{xml}");
  for label in ["main", "aux"] {
    assert!(
      xml.contains(&format!(r#"labels="LABEL:{label}""#)),
      "{label}: {xml}"
    );
  }
  assert!(
    xml.contains(
      r#"By <ref labelref="LABEL:main" show="typerefnum"/>, and <ref labelref="LABEL:aux" show="typerefnum"/> and <ref labelref="LABEL:main" show="typerefnum"/>."#
    ),
    "{xml}"
  );
}

/// 80i: acronym's `[smaller]` loads relsize, so the document's own `\smaller` prints, and `\acro` is the acronym
/// environment's alone. Red (int98a): no option read, `\acro` = `\acrodef` everywhere (KNOWN_PERL_ERRORS #1176).
/// Witness 1806.02261. Repro package-bindings/acronym_smaller_option_loads_relsize.
#[test]
fn acronym_smaller_loads_relsize() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/acronym_smaller_option_loads_relsize.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<text fontsize="83%" xml:id="p1.1.1">KLD</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<glossaryref inlist="acronym" key="BD" show="long"/>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"labels="LABEL:e""#), "{xml}");
}

/// 80i: the caption binding leaves a `\captionbox` the class defined alone, as caption.sty's `\newcommand*` does (and
/// caption3, which subfig `[caption=false]` and floatrow load, defines none). Red (int98a): the binding's macro replaced
/// ws-jmrr.cls's `\newbox\captionbox` ("Expected a relational token"). Witness 2303.09686. Repro
/// captions-floats/caption3_keeps_a_class_captionbox.
#[test]
fn caption3_keeps_a_class_captionbox() {
  let (log, xml) = convert_files_with(
    repro!("captions-floats/caption3_keeps_a_class_captionbox.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"<p xml:id="p1.1">yes</p>"#), "{xml}");
}

/// 80i: nicematrix's `\tabularnote[label]{text}` marks the note with its label. Red (int98a): the bracket-stripped
/// label was passed as `\footnote#1`, so `[x]` became the note's text and `[$\star$]` opened math that closed nowhere
/// (17 errors; 2602.00629 lost every section after the table). Repro
/// alignment-bindings/nicematrix_tabularnote_label_kept.
#[test]
fn nicematrix_tabularnote_label_kept() {
  let (log, xml) = convert_files_with(
    repro!("alignment-bindings/nicematrix_tabularnote_label_kept.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"B<note mark="x" role="footnote" xml:id="footnotex1">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"</tags>note one</note></td>"#), "{xml}");
  assert!(
    xml.contains(r#"(D<note mark="$\star$" role="footnote" xml:id="footnotex2">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"</tags>note two</note>)</td>"#), "{xml}");
  assert!(xml.contains(r#"<p xml:id="p2.1">After.</p>"#), "{xml}");
}

/// 80j: `\pdfstrcmp` compares its texts as `show_token_list` writes them — a control word with its trailing space —
/// so expl3's `\str_if_eq:nn` tells a compiled regex variable by its meaning and `\regex_replace_case_once:nN` uses it
/// (datatool's numeric parser, `\DTLmax`). Red (int98a): the plain token text ("NE"; the variable compiled as a
/// pattern, "Invalid character '_' following '\c' escape"; KNOWN_PERL_ERRORS #1178). Witness 2006.14937. Repro
/// expansion-primitives/pdfstrcmp_compares_shown_token_lists.
#[test]
fn pdfstrcmp_compares_shown_token_lists() {
  let (log, xml) = convert_files_with(
    repro!("expansion-primitives/pdfstrcmp_compares_shown_token_lists.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">A:EQ.\nB:EQ.[xby]\nMax: 58.18.</p>"),
    "{xml}"
  );
}

/// 80j: an ignored character after a control word leaves the tokenizer skipping blanks (tex.web §344), so under
/// `\ExplSyntaxOn` the `~` of `\foo ~` is skipped. Red (int98a): the skip ended at the ignored space and the `~` became a
/// space token ("A:19. B:NE."; KNOWN_PERL_ERRORS #1177). Repro
/// string-mouth/ignored_character_keeps_skipping_after_control_word.
#[test]
fn ignored_character_keeps_skipping_after_control_word() {
  let (log, xml) = convert_files_with(
    repro!("string-mouth/ignored_character_keeps_skipping_after_control_word.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">A:18.B:EQ.C:8.</p>"#),
    "{xml}"
  );
}

/// 80k: a binding's guessed theorem (`\lx@guess@newtheorem`, neurips' `\AtBeginDocument` set) is a fallback guess, so
/// the document's own `\newcommand` of the name defines it, as in LaTeX where the class never defined it. Red (int98a):
/// `\newcommand{\result}` was refused and every `\result{…}{…}` cell opened a theorem (6 errors; 2606.17122: 87, Fatal).
/// The guessed `{lemma}` stays a theorem. Repro class-bindings/neurips_guessed_theorem_yields_to_document_newcommand.
#[test]
fn neurips_guessed_theorem_yields_to_document_newcommand() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/neurips_guessed_theorem_yields_to_document_newcommand.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml
      .contains(r#"<td align="center" xml:id="p1.1.1.2">99.54<Math mode="inline" tex="\pm\,0.04""#),
    "{xml}"
  );
  assert!(xml.contains("ltx_theorem_lemma"), "{xml}");
}

/// 80k: a `\label` in a longtable's caption writes its key expanded, as LaTeX's `\label` does. Red (int99a): the key's
/// macros were kept (`\FloatPret:\ltbcp` on both tables; KNOWN_PERL_ERRORS #1179). Witness 1703.09144. Repro
/// cross-references/longtable_label_key_is_expanded.
#[test]
fn longtable_label_key_is_expanded() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/longtable_label_key_is_expanded.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for label in ["MCSt:A", "MCSt:B", "MCSt:BC"] {
    assert!(
      xml.contains(&format!(r#"labels="LABEL:{label}""#)),
      "{label}: {xml}"
    );
  }
  assert!(!xml.contains("FloatPre"), "{xml}");
}

/// 80k: `\cref` splits its label list once expanded, as cleveref's `\edef\@tempa{#2}` does. Red (int99a): a macro or an
/// expl3 sequence holding the list was one label (`LABEL:def:a,def:c`; KNOWN_PERL_ERRORS #1180). Witness 2108.04620.
/// Repro cross-references/cleveref_label_list_is_expanded.
#[test]
fn cleveref_label_list_is_expanded() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/cleveref_label_list_is_expanded.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let refs: Vec<&str> = xml
    .match_indices(r#"labelref="LABEL:"#)
    .map(|(i, _)| &xml[i + 16..i + 21])
    .collect();
  assert_eq!(
    refs,
    ["def:a", "def:c", "def:a", "def:b", "def:a", "def:b"],
    "{xml}"
  );
  assert!(!xml.contains("def:a,def:"), "{xml}");
}

/// 80k: the conditional guard's Fatal keeps what was digested before it, as Perl's `hardYankProcessing` keeps it for
/// every Fatal, and the kept document is built. Red (int99a): only the Stomach and TooManyErrors Fatals were salvaged,
/// so `Timeout:IfLimit` left a 39-byte document — the whole of 2006.14937, whose 24 matplotlib figures reach the limit
/// at the ninth; red (int100a): kept, the document was lost in the build, where the first formula the math parser could
/// not read took the latched Fatal for its own. Repro expansion-primitives/iflimit_fatal_keeps_the_document_before_it.
#[test]
fn iflimit_fatal_keeps_the_document_before_it() {
  let (log, xml) = convert_files_with(
    repro!("expansion-primitives/iflimit_fatal_keeps_the_document_before_it.tex"),
    &[],
    None,
  );
  assert!(log.contains("Fatal:Timeout:IfLimit"), "{log}");
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    xml.contains("Kept</title>") && xml.contains("Text before the runaway"),
    "{xml}"
  );
  assert_eq!(
    xml.matches("<Math ").count(),
    2,
    "both formulas kept: {xml}"
  );
  assert!(
    xml.contains("Inside"),
    "the level the loop ran in is kept: {xml}"
  );
}

/// 80l: siunitx's unit commands are its own business: a name siunitx never declared (`\yb`, version-1 only) stays
/// undefined, its own free ones are `\protected` `\ERROR` from `\begin{document}`. Red (int100c): `\DeclareSIUnit` let
/// every declared one `\relax` (KNOWN_PERL_ERRORS #1181), and tikz's math library read `\tikzmath{\yb = 0.6;}` as
/// the keyword `\yb` (1,001 errors). Witness 2409.01115. Repro
/// package-bindings/siunitx_unit_commands_stay_undefined_outside_units.
#[test]
fn siunitx_unit_commands_stay_undefined_outside_units() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/siunitx_unit_commands_stay_undefined_outside_units.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">B. D."#),
    "\\yb undefined, \\metre defined: {xml}"
  );
  assert!(xml.contains("<picture") && xml.contains("km"), "{xml}");
}

/// 80l: aa.cls v9.1's `\longtab[opt]{body}` keeps its body, where the older classes' `\longtab{N}{body}` drops the
/// number. Red (int100c): the binding gobbled the first argument, the v9.1 body, and the table with its label was lost
/// (1811.02197). Repro class-bindings/aa_v9_longtab_keeps_its_body.
#[test]
fn aa_v9_longtab_keeps_its_body() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/aa_v9_longtab_keeps_its_body.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for label in ["tab:new", "tab:old"] {
    assert!(
      xml.contains(&format!(r#"labels="LABEL:{label}""#)),
      "{label}: {xml}"
    );
  }
  for cell in ["a", "b", "c", "d"] {
    assert!(xml.contains(&format!(">{cell}</td>")), "{cell}: {xml}");
  }
}

/// 80l: a `\label` after an alignment's last row (`… \\ \label{c} \end{eqnarray*}`) labels that row, as `\eqnarray` set
/// `\@currentlabel` and pdflatex resolves it. Red (int100c): its `\lx@hidden@noalign` had no next cell to carry it and
/// was dropped (KNOWN_PERL_ERRORS #1182; 1206.1393). Repro cross-references/eqnarray_star_label_after_the_last_row.
#[test]
fn eqnarray_star_label_after_the_last_row() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/eqnarray_star_label_after_the_last_row.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"<equation labels="LABEL:c""#), "{xml}");
}

/// 80m (SYNC R4): a class preloaded (`--preload=article.cls`) leaves LaTeX's hook stack balanced — once "Extra
/// \PopDefaultHookLabel" (the class pushed with the pre-pool `\@pushfilename`); clean since, re-verified on int101a
/// for article, report, book, amsart and revtex4-2.
#[test]
fn preloaded_class_keeps_the_hook_stack() {
  for class in ["article", "amsart"] {
    let tex =
      format!("\\documentclass{{{class}}}\n\\begin{{document}}\nHello.\n\\end{{document}}\n");
    let (log, xml) = convert_files_with(&tex, &[], Some(&format!("{class}.cls")));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    assert!(xml.contains("Hello."), "{class}: {xml}");
  }
}

/// 80n: a false conditional that a column's u-part opens ends at its v-part's `\fi` — while TeX skips the cell's text
/// it meets the cell's `&` at align_state 0 and inserts the v-part there (tex.web §342 in `get_next`, which
/// `pass_text` reads through; §789). Red (int102a): the skip read past the `&` to the end of the table ("expected
/// `\fi`"), the rest of the document lost. Witness 1504.08100. Repro
/// kernel-alignment/iffalse_template_skips_to_the_v_part.
#[test]
fn a_false_conditional_in_a_template_ends_at_its_v_part() {
  let (log, xml) = convert_files_with(
    repro!("kernel-alignment/iffalse_template_skips_to_the_v_part.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for cell in ["A", "B", "C", "D", "E", "F"] {
    assert!(xml.contains(&format!(">{cell}</td>")), "{cell}: {xml}");
  }
  assert!(
    !xml.contains("hidden") && !xml.contains("gone") && !xml.contains("skipped"),
    "{xml}"
  );
  assert!(xml.contains("After the table."), "{xml}");
}

/// 80n: a two-column class's `\maketitle` calls `\twocolumn[\@maketitle]` (latex.ltx/article.cls, IEEEtran.cls,
/// ieeeconf.cls:3589-3605), so a document's redefinition of `\twocolumn` runs there and adds its full-width block under
/// the title — the ICRA/IROS teaser figure. Red (int102a): the frontmatter `\maketitle` called no `\twocolumn`, the
/// figure and its label were lost (`\ref` dangling). Witness 2305.16369. Repro
/// cross-references/maketitle_calls_a_redefined_twocolumn.
#[test]
fn maketitle_calls_a_redefined_twocolumn() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/maketitle_calls_a_redefined_twocolumn.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let title = xml.find("<title>The Title</title>").expect(&xml);
  let figure = xml.find(r#"labels="LABEL:fig:teaser""#).expect(&xml);
  assert!(title < figure, "{xml}");
  assert!(xml.contains("The teaser."), "{xml}");
  assert!(
    xml.contains(r#"<ref labelref="LABEL:fig:teaser"/>"#),
    "{xml}"
  );
}

/// 80n: acronym.sty's `\ac` & co. are `\protected` (their starred forms come from suffix.sty's `\WithSuffix`, which
/// `\protected\def`s the command, suffix.sty:53), so a siunitx `S` cell's expanding read keeps `\ac{MAE}` whole. Red
/// (int102a): the binding's plain macros expanded there — the star test's `\ifx` and the `\newif` switch — "Missing
/// `\fi`", the table left open and the rest of the document inside it. Witness 2607.15705. Repro
/// package-bindings/acronym_commands_pass_a_siunitx_column_whole.
#[test]
fn acronym_commands_pass_a_siunitx_column_whole() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/acronym_commands_pass_a_siunitx_column_whole.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let table = &xml[xml.find("<tabular").expect(&xml)..xml.find("</tabular>").expect(&xml)];
  assert_eq!(table.matches("<tr").count(), 2, "{xml}");
  assert!(
    table.contains(r#"<glossaryref inlist="acronym" key="MAE""#),
    "{xml}"
  );
  assert!(xml.contains("After the table."), "{xml}");
}

/// 80o: iucr.cls's `\hcaption` is the caption. The class (shipped by its papers, not in TeX Live) stores the caption
/// with `\gdef\captionbox` (iucr.cls:6645-6890) for its own float end code (:3828-3840), which the kernel's float
/// bindings own, so the caption and its `\label` were lost. Red (int103a): no caption, the `\ref` dangling. Witness
/// 1707.09045. The stand-in class keeps the real one's shape.
#[test]
fn iucr_hcaption_is_the_caption() {
  let (log, xml) = convert_files_with(
    r"\documentclass{iucr}
\begin{document}
\begin{table}
\begin{tabular}{ll} a & b \end{tabular}
\hcaption{The model.\label{model:direct}}
\end{table}
See Table~\ref{model:direct}.
\end{document}
",
    &[(
      "iucr.cls",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesClass{iucr}\LoadClass{article}
\newcommand{\hcaption}[1]{\gdef\captionbox{\par\noindent{\bfseries Table \thetable}\par\noindent #1}}
\renewenvironment{table}{\begin{center}\refstepcounter{table}}{\end{center}\captionbox}
",
    )],
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  let table = xml.find(r#"labels="LABEL:model:direct""#).expect(&xml);
  assert!(xml[table..].contains("The model."), "{xml}");
  assert!(xml.contains("<caption"), "{xml}");
}

/// 80p: amsmath's `\collect@body` (amsmath.sty:1410-1434) is defined, so a document's environment built on it — the
/// body up to the matching `\end`, nested environments counted, handed to a macro — typesets. Red (int104b): the binding
/// (as Perl's amsmath.sty.ltxml) reads its own displays' bodies another way and never defined it, "undefined
/// `\collect@body`", the body's `&` and `^` stray, 101 errors and a Fatal. Witness 2305.01517 (66 census papers call
/// it). Repro package-bindings/amsmath_collect_body_builds_an_environment.
#[test]
fn amsmath_collect_body_builds_an_environment() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/amsmath_collect_body_builds_an_environment.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"<equation labels="LABEL:eq:a""#), "{xml}");
  assert!(xml.contains("<XMArray"), "{xml}");
}

/// 80p: a horizontal command met in vertical mode (`\ `, `\leavevmode`, `\hskip`, …) is backed up while `\everypar` is
/// inserted in front of it, as a character is (tex.web §1090-1091), so a hook that opens a box for the line's material
/// holds it. Red (int104b): the hook was digested in place, its `\hbox\bgroup` under the paragraph's mode switch — one
/// "Attempt to close a group that switched to mode horizontal" per line (gr-qc9505014's Maple transcripts: 100 errors and
/// a Fatal). Repro paragraph-mode/everypar_box_holds_a_line_opened_by_a_control_space.
#[test]
fn everypar_box_holds_a_line_opened_by_a_control_space() {
  let (log, xml) = convert_files_with(
    repro!("paragraph-mode/everypar_box_holds_a_line_opened_by_a_control_space.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut at = 0;
  for line in ["A", "B", "C", "D", "After."] {
    at += xml[at..].find(&format!("{line}</p>")).expect(&xml);
  }
}

/// 80p: `\begin{X}` of an environment the document declared with `\newtheorem` runs a `\X` the document then redefined,
/// as LaTeX's `\begin` (`\begingroup\csname X\endcsname`, latex.ltx:15346-15362) does — `\renewenvironment` (environ's
/// `\RenewEnviron`) or a wrapper around the saved start (`\let\@lem\lem \def\lem{\@lem\label{…}}`); `\end{X}` follows
/// the same decision. Red (int104b): the theorem's constructor ran regardless, the renewal and the wrapper's `\label`
/// lost (2509.20478: `\makerestatable` never stored its restatements, three "No restatable environment" errors and five
/// theorem labels; 1502.00944). Repro cross-references/newtheorem_follows_a_redefined_start.
#[test]
fn newtheorem_follows_a_redefined_start() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/newtheorem_follows_a_redefined_start.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("[START] x [END]"), "{xml}");
  let lemma = xml.find(r#"labels="LABEL:lem:wrapped""#).expect(&xml);
  assert!(
    xml[..lemma].contains("<theorem") || xml[lemma..].contains("Wrapped."),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<ref labelref="LABEL:lem:wrapped"/>"#),
    "{xml}"
  );
}

/// 80q: after amsart's `\maketitle` its topmatter commands are `\relax` (`\@cleartopmattertags`, amsart.cls:620-629),
/// so pdflatex never reads a later `\keywords{…}` as an argument. A well-formed one keeps its keywords markup (ruling
/// 2026-10-25); one whose short argument would run into a `\par` (a brace short, 2204.07665) is read as pdflatex reads
/// it — an open group, no error. Red (int105a): "Paragraph ended before `\keywords` was complete", the labels inside
/// lost. Repro sectioning-frontmatter/amsart_post_title_keywords_one_brace_short.
#[test]
fn amsart_post_title_keywords_one_brace_short() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/amsart_post_title_keywords_one_brace_short.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let keywords = xml.find("<keywords").expect(&xml);
  assert!(xml[keywords..].contains("well, formed"), "{xml}");
  assert!(xml.contains("Key Words.") && xml.contains("a, b."), "{xml}");
  assert!(
    xml.contains(r#"labels="LABEL:eq:a""#) && xml.contains("Intro</title>"),
    "{xml}"
  );
}

/// 80r: dgruyter.sty (De Gruyter's style, shipped by its papers, not in TeX Live) is a stand-in: the paper's copy is
/// read after it, and its definitions of names the kernel's semantic layer owns — its copy of array.sty's `\@array`
/// (dgruyter.sty:2003-2029, which calls array.sty's `\@arrayleft`), its `\let\@@array\@array` (:2031) and wrapper (:2046) —
/// hold only inside its own code, while its extras stand. Red (int106a): read raw under the ar5iv preload with no binding, the copy's `\@array` took over every
/// `array` ("undefined `\@arrayleft`", the cells loose in math). Witness 2510.17333 (274 errors). The stand-in copy
/// keeps the real one's shape.
#[test]
fn dgruyter_copy_keeps_the_native_array() {
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\usepackage{dgruyter}
\begin{document}
\begin{equation}\label{eq:a}
\left(\begin{array}{c|c} a & b \\ \hline c & \dgextra \end{array}\right)
\end{equation}
See (\ref{eq:a}).
\end{document}
",
    &[(
      "dgruyter.sty",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesPackage{dgruyter}
\RequirePackage{array}
\newcommand\dgextra{d}
\def\@array[#1]#2{\@arrayleft\if #1t\vtop \else \if#1b\vbox \else \vcenter \fi \fi\bgroup\@preamble}
\let\@@array\@array
\let\orig@@array\@@array
\def\@@array[#1]#2{\orig@@array[#1]{#2}}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains(r#"<equation labels="LABEL:eq:a""#), "{xml}");
  assert!(xml.contains("<XMArray"), "{xml}");
  assert!(xml.contains(">d</XMTok>"), "{xml}");
}

/// 80r: the dgruyter stand-in declares the frontmatter dgruyter.sty stores for its title page (dgruyter.sty:2665-2760),
/// as the copy's `\maketitle` is the kernel's: the subtitle, keywords, classification, history dates, communication
/// and dedication notes are the frontmatter's elements; the running heads are page furniture. Red (int107a, the
/// stand-in without them): the copy's stores never printed, 13 of the 18 census dgruyter papers lost their subtitle and
/// keywords (2204.05056).
#[test]
fn dgruyter_frontmatter_is_kept() {
  let (log, xml) = convert_files_with(
    r"\documentclass{article}
\usepackage{dgruyter}
\begin{document}
\title{T}
\subtitle{The subtitle}
\runningtitle{Running}
\author{A}
\keywords{alpha, beta}
\classification[PACS]{01.01}
\received{May 1}
\accepted{June 2}
\maketitle
Body.
\end{document}
",
    &[(
      "dgruyter.sty",
      r"\NeedsTeXFormat{LaTeX2e}\ProvidesPackage{dgruyter}
\def\keywords#1{\gdef\@keywords{#1}}
\def\received#1{\gdef\@received{#1}}
\def\accepted#1{\gdef\@accepted{#1}}
\def\runningtitle#1{\gdef\@runningtitle{#1}}
\def\subtitle#1{\gdef\@subtitle{#1}}
\newcommand\classification[2][PACS]{\gdef\@classification{#2}}
",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  for element in [
    "<subtitle>The subtitle</subtitle>",
    ">alpha, beta</keywords>",
    "01.01</classification>",
    "<date name=\"Received\u{a0}\" role=\"received\">May 1</date>",
    "<date name=\"accepted\u{a0}\" role=\"accepted\">June 2</date>",
  ] {
    assert!(xml.contains(element), "{element}: {xml}");
  }
  assert!(!xml.contains("Running"), "{xml}");
}

/// revtex's group rule: every `\affiliation` after a group of `\author`s is each author's of the group (revtex4-2.cls's
/// `\affiliation`, `\move@AU\move@AF` filing the pending authors with each), `annotate=run` — the run of new authors,
/// and a second contact before any new author the same run's. Red (int109c): Perl's `annotate=new`
/// (Base_Utility.pool.ltxml:521) gave the group's second affiliation to its last author only — Ann lost Univ Y
/// (KNOWN_PERL_ERRORS #1190). Witness 1303.7097. Repro sectioning-frontmatter/author_revtex_group_affiliations_reach_every_name.
#[test]
fn revtex_group_affiliations_reach_every_name() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/author_revtex_group_affiliations_reach_every_name.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators: Vec<&str> = xml
    .split("<creator ")
    .skip(1)
    .map(|c| &c[..c.find("</creator>").expect("a closed creator")])
    .collect();
  let affiliations = |c: &str| -> Vec<String> {
    c.split("role=\"affiliation\">")
      .skip(1)
      .map(|a| {
        a[..a.find("</contact>").expect("a closed contact")]
          .trim()
          .to_string()
      })
      .collect()
  };
  let read: Vec<(bool, Vec<String>)> = creators
    .iter()
    .map(|c| (c.contains("<personname"), affiliations(c)))
    .collect();
  let named = |aff: &[&str]| (true, aff.iter().map(|a| a.to_string()).collect::<Vec<_>>());
  assert_eq!(
    read,
    [
      named(&["Univ X, City", "Univ Y, Town"]),
      named(&["Univ X, City", "Univ Y, Town"]),
      named(&["Univ Z, Village", "Univ W, Hamlet"]),
    ],
    "{xml}"
  );
}

/// revtex4's obsolete `\address` is `\affiliation` (revtex4-2.cls:7519, revtex4-1.cls:7498, revtex4.cls:5964
/// `\replace@command\address\affiliation`), as the binding's `\altaddress` is `\altaffiliation`: mark-led lines go to the
/// names showing their marks. Red (80u): Perl's `\lx@add@address` had no mark path, and under the group rule every line
/// was every name's. Witness 0806.3762. Repro sectioning-frontmatter/author_revtex_marked_address_by_mark.
#[test]
fn revtex_marked_address_goes_by_mark() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/author_revtex_marked_address_by_mark.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  let creators: Vec<&str> = xml
    .split("<creator ")
    .skip(1)
    .map(|c| &c[..c.find("</creator>").expect("a closed creator")])
    .collect();
  let places = |c: &str| -> Vec<&str> {
    ["CERN", "CCPP", "ITPP"]
      .into_iter()
      .filter(|p| c.contains(p))
      .collect()
  };
  assert_eq!(
    creators.iter().map(|c| places(c)).collect::<Vec<_>>(),
    [vec!["CERN", "CCPP"], vec!["CCPP"], vec!["ITPP"]],
    "{xml}"
  );
  assert!(
    creators.iter().all(|c| !c.contains("role=\"address\"")),
    "{xml}"
  );
}
