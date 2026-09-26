use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// A kernel length is latex.ltx's allocated `\dimen<n>`, as the dump records it:
/// the post-dump pools' `DefRegister!` renamed it to its own CS name, so
/// etoolbox's `\ifdefdimen` (a `\meaning` test) said "not a length" — the
/// tudscrbase error of tudscrartcl/tudscrbook/tudscrposter/tudscrreprt.
#[test]
fn kernel_length_is_an_allocated_dimen() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/kernel_length_is_an_allocated_dimen.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("dimen124 yes yes 345.0pt</p>"), "{xml}");
}

/// The `xetex` profile: iftex, l3sys and the XeTeX primitives read XeTeX, so
/// classes gated on `\sys_if_engine_xetex:F{\msg_fatal…}` (fduthesis, njuthesis,
/// exam-zh, xtufte) convert.
#[test]
fn xetex_profile_engine_identity() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/xetex_profile_engine_identity.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,xetex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>[xetex][X][N][xelatex]yes yes yes no</p>"),
    "{xml}"
  );
  // The default persona is unchanged.
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>[pdftex][L][P][latex]no no no yes</p>"),
    "{xml}"
  );
}

/// Classes call the expl3 internals of fontspec, unicode-math, xeCJK and ulem,
/// whose bindings shadow the raw packages (fduthesis, njuthesis, exam-zh,
/// xdupgthesis, xduugtp, bitbeamer).
#[test]
fn xetex_class_calls_binding_internals() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/xetex_class_calls_binding_internals.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,xetex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Text.</p>"), "{xml}");
}

/// Raw classes call internals that our bindings reimplement without:
/// `\@chapapp`, `\env@matrix`, `\HyLang@addto`, `\linenumberdisplaymath`,
/// `\symAMSb`, `\NAT@find@eq`, `\vref@addto`, `\mdtheorem`, and book/report's
/// `\@titlepagetrue` default (utexasthesis, willowtreebook, unbtex, ascelike,
/// acmart-tagged, univie-ling-*, rbt-mathnotes*, jurabook, hausarbeit-jura).
#[test]
fn binding_internals_classes_call() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/binding_internals_classes_call.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Only mdframed's own "minimally stubbed" notice.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    xml.contains("<p>Chapter Appendix yes X a=b Y T</p>"),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<title class="ltx_runin"><tag><text font="bold">Theorem 1</text></tag></title>"#
    ),
    "{xml}"
  );
  // mdframed's unnumbered `thm*`.
  assert!(
    xml.contains(r#"<title class="ltx_runin"><tag><text font="bold">Theorem</text></tag></title>"#),
    "{xml}"
  );
}

/// latex.ltx:15362 `\UseHook{env/document/begin}` runs before `\document`: code
/// queued there (`\AddToHook`, or ctex's `\hook_gput_code:nnn` deferral of
/// `\RequirePackage{biblatex}` in xdupgthesis/xduugthesis) was dropped, since
/// the `\begin{document}` constructor is not a DefEnvironment.
#[test]
fn env_document_begin_hook_fires() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/env_document_begin_hook_fires.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[A][B][C]</p>"), "{xml}");
}

/// More binding internals raw classes call: biblatex
/// `\DeclarePrintbibliographyDefaults`, listings `\lst@InputCatcodes`/
/// `\lst@RestoreCatcodes`, caption `\AtCaptionPackage`, datetime's time
/// machinery (`\newtimeformat`, `\twodigit`, `\THEHOUR`).
#[test]
fn binding_internals_classes_call_2() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/binding_internals_classes_call_2.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Only datetime's own "partially ported" notice.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(xml.contains("<p>09:05 Text.</p>"), "{xml}");
}

/// tex.web §1224/§405: `\chardef`/`\mathchardef`/`\countdef`… read the optional
/// `=` with expansion. catoptions' `\chardef\cpt@optionstacklimit\reserved@a`
/// (`\reserved@a` = `=4\relax`) set its option-stack limit to 0 (cv4tw,
/// arabic-book: "Current option/key state is being pushed").
#[test]
fn chardef_optional_equals_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/chardef_optional_equals_expands.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[4][4]</p>"), "{xml}");
}

/// latex.ltx:18297-18300: `\pagestyle{#1}` runs `\ps@#1`, where classes define
/// macros (dccpaper-base.sty's `\TitleHead`: idcc, ijdc-v14, ijdc-v9).
#[test]
fn pagestyle_runs_its_ps_macro() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/pagestyle_runs_ps_macro.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>HEAD</p>"), "{xml}");
}

/// `\pdfcreationdate` is a conversion (tex.web §464 `str_toks`): catcode-12
/// characters, so novel-pdfx.sty's `D:`-delimited date parser matches it.
#[test]
fn pdfcreationdate_is_other_catcode() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/pdfcreationdate_other_catcode.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>year-ok</p>"), "{xml}");
}

/// `\defbeamertemplate`'s trailing `[action]{code}` and
/// `\defbeamertemplateparent`'s full argument list are consumed
/// (beamerbasetemplates.sty:54-89; univie-ling-poster).
#[test]
fn beamer_template_action_is_consumed() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/beamer_template_action.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>before after</p>"), "{xml}");
}

/// latex.ltx:18521-18525 `\@pass@ptions`: `\@raw@opt@<file>` holds the
/// argument tokens from pass time on (the first expanded once), so an expl3
/// value reaches `\ProcessKeyOptions` intact (fduthesis.cls:193-202 and
/// hustthesis.cls:197 → ctexbook.cls:336). pdflatex:
/// `[1.5][x] macro:->a=\myval ,b,c`.
#[test]
fn passed_options_keep_their_tokens() {
  let repro = include_str!(
    "../../../tools/perfect_kernel/repros/loader/passed_options_keep_their_tokens.tex"
  );
  // The sidecar package is the repro's `filecontents*` body; the guard writes
  // it into the conversion's own directory instead.
  let open = "\\begin{filecontents*}[overwrite]{rawoptpkg.sty}\n";
  let close = "\\end{filecontents*}\n";
  let start = repro.find(open).expect("filecontents start") + open.len();
  let end = repro.find(close).expect("filecontents end");
  let (sty, doc) = (&repro[start..end], &repro[end + close.len()..]);
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(doc, &[("rawoptpkg.sty", sty)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<p>[1.5][x] <text font="typewriter">macro:-&gt;a=\myval ,b,c</text></p>"#),
    "{xml}"
  );
}

/// latex.ltx:18405-18411 `\@ifpackagelater` compares the loaded file's
/// `\ver@<file>`, stored expanded (:18481-18483, :22454-22457): false for a
/// package not loaded yet (yathesis.cls:459 → babel `main=` passed twice).
#[test]
fn ifpackagelater_reads_the_loaded_version() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/ifpackagelater_reads_the_loaded_version.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[F][T][F][ok]</p>"), "{xml}");
}

/// titlesec keeps each title's spacing in a five-group `\ttls@<section>`
/// (titlesec.sty:640-658, filled at load) that ctex reads after titlesec
/// loads (mynsfc, qyxf-book, bjfuthesis), plus `\ttl@chapterout` (:402).
#[test]
fn titlesec_spacing_record() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_spacing_record.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[5][5][5][5][5][ok]</p>"), "{xml}");
}

/// Binding internals, third pass: biblatex loads xparse (its expl3 case
/// changer; nwejmart's `u` argument), mathtools' `\MT_options_name:` keyval
/// family (nwejmart), biblatex's `\bibbycategory` (gztarticle).
#[test]
fn binding_internals_classes_call_3() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/binding_internals_classes_call_3.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[a/b] [ok]</p>"), "{xml}");
}

/// `\tableofcontents` (and the `lof`/`lot` pair) carry the kernel's
/// `\@starttoc{…}` call, discarded, so etoolbox patches of it succeed
/// (exam-zh.cls:272) and the ToC still comes from post-processing.
#[test]
fn patchcmd_tableofcontents_starttoc() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/patchcmd_tableofcontents_starttoc.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches(r#"<TOC lists="toc""#).count(), 1, "{xml}");
  assert!(xml.contains("<p>[P-ok]</p>"), "{xml}");
}

/// listings resolves `rulecolor`/`backgroundcolor` when the listing is
/// drawn, so a colour defined after the `\lstset` applies (easybase.sty:2419
/// vs :2439; easybook).
#[test]
fn listings_color_resolved_at_listing() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/listings_color_resolved_at_listing.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      r##"<listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain" framecolor="#C00000" framed="rectangle">"##
    ),
    "{xml}"
  );
}

/// latex.ltx:18484-18486: a file served by the versioned-package fallback
/// takes the loaded file's version under its requested name, so floatrow's
/// `\@ifpackagelater{caption3}{…}` passes (iaria, iaria-lite, langscibook,
/// letgut).
#[test]
fn fallback_load_keeps_the_requested_version() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/fallback_load_keeps_the_requested_version.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[T]</p>"), "{xml}");
}

/// amsbook's running-head mark builder `\@secmark` (amsbook.cls:303-313),
/// which a class's `\ps@headings` marks call (my-thesis).
#[test]
fn amsbook_secmark() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsbook_secmark.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Text.</p>"), "{xml}");
}

/// biblatex.sty:15-21's version strings, which third-party style files echo
/// in their `\ProvidesFile` (dtk.cbx:12 `[\abx@cbxid]`; dtk).
#[test]
fn biblatex_style_version_strings() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/biblatex_style_version_strings.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Hello. [3.21]</p>"), "{xml}");
}

/// beamer loads scrlfile, as beamerbasefont → sansmathaccent does, so a
/// beamer-based class's `\AfterPackage` hooks run (univie-ling-poster).
#[test]
fn beamer_loads_scrlfile() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/beamer_loads_scrlfile.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>After: ok.</p>"), "{xml}");
}

/// The scrpage binding's page styles run their layout layers
/// (`\ps@@scrheadings`, `\ps@scrplain`), which `\pagestyle` reaches since
/// 56hz (l2picfaq, oscola and ~20 KOMA manuals, sweep 120).
#[test]
fn scrpage_pagestyle_layers() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/scrpage_pagestyle_layers.tex"
  );
  // The raw scrartcl (the sweep's persona) reaches `\pagestyle`; its KOMA
  // warnings (type area, `\@startsection`) are its own.
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Text.</p>"), "{xml}");
}

/// l3text case changes on accented text finish and map it: the dump's 8-bit
/// l3text read it as a UTF-8 lead byte and never finished (letgut's
/// `\text_lowercase:n {INSPÉ}`).
#[test]
fn l3text_case_change_accented() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/l3text_case_change_accented.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>[inspé][ÉÀÜ STRASSE][Élan][HELLO WORLD][HELLO wORLD]</p>"),
    "{xml}"
  );
}

/// mfirstuc's `\capitalisewords` over an arrayjob element reaches l3text's
/// own case changer (ftc-notebook's example-notebook.tex broke when the
/// changers took the native mapping, sweep #121).
#[test]
fn mfirstuc_capitalisewords_arrayjob() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/mfirstuc_capitalisewords_arrayjob.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<p><text font="bold">Task 1: robot drive.</text></p>"#),
    "{xml}"
  );
}

/// l3text changes the case of accented text held in a token list: expl3's
/// Unicode-engine codepoint layer reads one token as one code point (the
/// format's 8-bit layer took `é` as a UTF-8 lead byte).
#[test]
fn l3text_codepoint_layer_token_lists() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/l3text_codepoint_layer_token_lists.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>[HELLO ÉLAN][Élan De Vie][σας inspé]</p>"),
    "{xml}"
  );
}

/// Under the `luatex` profile `\pdffeedback creationdate` is the PDF date
/// string (luatex85's `\pdfcreationdate`; novel-pdfx's `D:` parser) and
/// `\savepos` is LuaTeX's `\pdfsavepos` (novel).
#[test]
fn luatex_pdffeedback_creationdate() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/luatex_pdffeedback_creationdate.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>year-ok pos-ok</p>"), "{xml}");
}

/// calc's `\calc@assign@count`/`@dimen`/`@skip`, which xifthen's
/// `\cnttest`/`\dimtest` call directly (novel's layout check).
#[test]
fn xifthen_calc_assign() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/xifthen_calc_assign.tex");
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[yes][yes][no]</p>"), "{xml}");
}

/// hyperref's `\Hy@DisableOption` (novel-pdfx.sty:487).
#[test]
fn binding_internals_classes_call_4() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/binding_internals_classes_call_4.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Text.</p>"), "{xml}");
}

/// `\DocumentMetadata` records pdfmanagement-testphase's version (which
/// documentmetadata-support.ltx:39 loads; the layer itself stays absorbed),
/// so packages' version tests on it pass (zugferd.sty:101).
#[test]
fn documentmetadata_loads_pdfmanagement() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/documentmetadata_loads_pdfmanagement.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[ok]</p>"), "{xml}");
}

/// `\pdfobj`/`\pdfannot` read every type spec form of the pdftex grammar,
/// keywords in sequence: `useobjnum <n> stream attr {…} file {…}` is
/// pdfmanagement's PDF/A colour profile (tuda-ci DEMOs).
#[test]
fn pdfobj_stream_file_spec() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/pdfobj_stream_file_spec.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>ABCDEFG</p>"), "{xml}");
}

/// `\verb` closes its group through `\verb@egroup` (latex.ltx:15501), so a
/// package's `\verb` wrapper closes too (accessibility's tagging).
#[test]
fn verb_egroup_closes_package_wrappers() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/verb_egroup_closes_package_wrappers.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  let bare = regex::Regex::new(r#" xml:id="[^"]*""#)
    .unwrap()
    .replace_all(&xml, "");
  assert!(
    bare.contains(r#"<td align="left"><verbatim font="typewriter">a_b</verbatim></td>"#),
    "{xml}"
  );
}

/// versonotes' `\versonote` is a margin note where it is written: the
/// package's own `.aux` round trip needs a second run (versonotes/sample).
#[test]
fn versonote_is_a_margin_note() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/versonote_is_a_margin_note.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let note = xml
    .split(r#"<note class="ltx_marginpar" role="margin""#)
    .nth(1)
    .expect("a margin note");
  let note = &note[..note.find("</note>").expect("closed note")];
  assert!(
    note.contains(r#"<emph font="italic">Aeneid</emph>: the epic of Vergil."#),
    "{xml}"
  );
  assert!(xml.contains("Body word alpha."), "{xml}");
}

/// K11: a rerouted title-page store the document never set keeps the class's
/// default, which reaches the frontmatter at `\maketitle` (lion-msc.cls:
/// 196-203 `\gdef\@affiliation{Huygens-Kamerlingh Onnes Laboratory, …}`).
#[test]
fn k11_class_default_store() {
  if !latexml::util::test::kpse_has("lion-msc.cls") {
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/k11_class_default_store.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // lion-msc `\RequirePackage{datetime}`: the one expected diagnostic.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    xml.contains(r##"<contact name="Affiliation: " role="affiliation"><text color="#000000">Huygens-Kamerlingh Onnes Laboratory, Leiden University</text></contact>"##),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<contact name="Address: " role="address"><text color="#000000">P.O. Box 9500, 2300 RA Leiden, The Netherlands</text></contact>"##),
    "{xml}"
  );
}

/// Two abstracts under different names are two abstracts: the replaceable-
/// frontmatter dedup (OXIDIZED_DESIGN #154) replaces only a same-name
/// re-emission (beamertheme-mirage-doc lost its English abstract).
#[test]
fn two_named_abstracts() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/two_named_abstracts.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let abstracts: Vec<String> = regex::Regex::new(r"(?s)<abstract [^>]*>.*?</abstract>")
    .unwrap()
    .find_iter(&xml)
    .map(|m| {
      let a = m.as_str();
      let head = &a[..a.find('>').unwrap() + 1];
      let text = regex::Regex::new(r"<[^>]+>")
        .unwrap()
        .replace_all(&a[head.len()..], " ");
      format!(
        "{head} {}",
        text.split_whitespace().collect::<Vec<_>>().join(" ")
      )
    })
    .collect();
  assert_eq!(
    abstracts,
    [
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> FIRSTABSTRACT english text here"#,
      r#"<abstract inlist="toc" name="Second" xml:id="abstract2"> SECONDABSTRACT chinese text here"#,
    ],
    "{xml}"
  );
}

/// geometry's length keys take a calc expression without touching its
/// operands (geometry.sty:369,461 `\Gm@setlength` → `\setlength`):
/// bookcover.cls:188 `paperwidth=2\marklength+…+\spinewidth` zeroed
/// `\spinewidth`, and the cover's spine and flaps vanished.
#[test]
fn geometry_length_expression() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/macro-state/geometry_length_expression.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>FLAPKEPTSPINEKEPT</p>"), "{xml}");
}

/// siunitx v3 keys: `locale` sets the decimal marker and products
/// (siunitx.sty:4956-5013), `drop-zero-decimal` / `minimum-decimal-digits` /
/// `uncertainty-mode` format the number, font-matching keys are accepted, and
/// an uncertainty is converted to the requested form (Perl
/// six_compute_separate_uncertainty / six_compute_relative_uncertainty):
/// `12.3(4)` separate is `12.3 ± 0.4`, not `12.3 ± 4`.
#[test]
fn siunitx_v3_keys() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/macro-state/siunitx_v3_keys.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let texs: Vec<String> = regex::Regex::new(r#"<Math [^>]*tex="([^"]*)""#)
    .unwrap()
    .captures_iter(&xml)
    .map(|c| c[1].to_string())
    .collect();
  assert_eq!(
    texs,
    [
      "3,14",
      r"1,5\text{\cdot}{10}^{3}",
      "3.14",
      "1",
      "2",
      "1.50",
      "1.50",
      "5.00",
      r"1.00\text{\times}{10}^{3}",
      r"12.3\pm 0.4",
      "12.0(4)",
      "5",
      "{10}^{3}",
      "12.3(4)",
      "12.30(45)",
      r"(1.23\pm 0.04)\text{\times}{10}^{3}",
    ],
    "{xml}"
  );
}

/// enumitem's `shortlabels`: a list's first key with no `=` that names no
/// enumitem key is a label template (enumitem.sty:660-681 `\enit@first`), not
/// an unknown key: `[(a)]` warned and kept arabic labels.
#[test]
fn enumitem_shortlabels() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/list-structure/enumitem_shortlabels.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let tags: Vec<&str> = regex::Regex::new(r#"<tag(?: role="refnum")?>[^<]*</tag>"#)
    .unwrap()
    .find_iter(&xml)
    .map(|m| m.as_str())
    .collect();
  assert_eq!(
    tags,
    [
      "<tag>(a)</tag>",
      r#"<tag role="refnum">a</tag>"#,
      "<tag>(b)</tag>",
      r#"<tag role="refnum">b</tag>"#,
      "<tag>(i)</tag>",
      r#"<tag role="refnum">i</tag>"#,
    ],
    "{xml}"
  );
}

/// A glossaries field may name an entry defined after it: the definitions of
/// preamble entries are emitted at `\begin{document}` (`\iflx@glossaries@defer`),
/// when every entry exists, as `\printglossary` typesets them (arXiv 2605.14032:
/// "Glossary entry `nr' has not been defined", and endc's long form borrowed
/// "Dual Connectivity"; Perl identical).
#[test]
fn glossaries_preamble_forward_reference() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/glossaries_preamble_forward_reference.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // Raw, not whitespace-normalized: the space between the two references is
  // part of the field (glossaries.sty stores `E-UTRAN-\gls {nr} \gls {dc}`).
  assert!(
    xml.contains(concat!(
      r#"<glossaryphrase key="endc" role="long">E-UTRAN-"#,
      r#"<glossaryref inlist="main" key="nr">NR</glossaryref> "#,
      r#"<glossaryref inlist="main" key="dc">DC</glossaryref> </glossaryphrase>"#
    )),
    "{xml}"
  );
}

/// nlctuserguide's `\nlctuserguidegls` entries are defined in the run, as
/// bib2gls's `.glstex` would define them, and their definitions are emitted
/// once every entry exists: the description's `\idx{exclusion}` names an entry
/// defined after it (Talbot's manuals: glossaries-extra-manual, datatool-user,
/// glossaries-user, mfirstuc-manual).
#[test]
fn nlctuserguide_entries_defined_in_run() {
  if !latexml::util::test::kpse_has("nlctuserguide.sty") {
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/nlctuserguide_entries_defined_in_run.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let flat = regex::Regex::new(r">\s+<")
    .unwrap()
    .replace_all(&xml, "><")
    .into_owned();
  assert!(
    flat.contains(concat!(
      r#"<glossarydefinition inlist="index" key="MFUexcl">"#,
      r#"<glossaryphrase key="MFUexcl" role="description">identifies an <glossaryref inlist="index" key="idx.exclusion">exclusion</glossaryref> command</glossaryphrase>"#,
      r#"<glossaryphrase key="MFUexcl" role="name"><text font="typewriter">\MFUexcl</text></glossaryphrase>"#,
      r#"<glossaryphrase key="MFUexcl" role="sort"><text font="typewriter">\MFUexcl</text></glossaryphrase>"#,
      r#"</glossarydefinition>"#
    )),
    "{flat}"
  );
  // The references (the description's `\idx` too) are `ltx:glossaryref`s since
  // batch 56jt: glossaries-extra replaces `\@gls@link` (glossaries-extra.sty:3465)
  // and the binding re-wraps it after the package loads (glossaries_sty.rs).
  assert!(
    flat.contains(concat!(
      r#"<p>Use <glossaryref font="typewriter" inlist="index" key="MFUexcl">\MFUexcl</glossaryref>"#,
      r#" for an <glossaryref inlist="index" key="idx.exclusion">exclusion</glossaryref>.</p>"#
    )),
    "{flat}"
  );
}

/// fontenc's `\usefont\encodingdefault` (fontenc.sty:116): the encoding in
/// force after `[LGR,TU]` is the default one, not the last mapped one
/// (greek-fontenc's test-tuenc-greek came out transliterated into Greek).
#[test]
fn fontenc_last_encoding_in_force() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/fontenc_last_encoding_in_force.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Hello world alphabet</p>"), "{xml}");
  let lgr_last = tex.replace("[LGR,TU]", "[T1,LGR]");
  let (_, xml) = convert_with(&lgr_last, None);
  assert!(xml.contains("<p>Ηελλο ωορλδ αλπηαβετ</p>"), "{xml}");
}

/// epstopdf loads grfext (epstopdf.sty:150): `\PrependGraphicsExtensions` and
/// its siblings are defined (arXiv 2606.05709).
#[test]
fn epstopdf_loads_grfext() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/epstopdf_loads_grfext.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Figure text.</p>"), "{xml}");
}

/// underscore's first aid (underscore-ltx): a file name keeps its `_`
/// because `\input` expands it as a `\csname` does (arXiv 2606.31852).
#[test]
fn underscore_first_aid_file_name() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/underscore_first_aid_file_name.tex"
  );
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files_with(
    tex,
    &[("sections/logic_T.tex", "Included text.\n")],
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The kernel's first aid loaded (the dump's `file/underscore.sty/after` hook).
  assert!(
    stderr.contains("First Aid for underscore.sty applied"),
    "{stderr}"
  );
  assert!(xml.contains("<p>Included text.</p>"), "{xml}");
  assert!(xml.contains("<p>Text a_b and <Math"), "{xml}");
  // `\InputIfFileExists` reads the same name.
  let tex = tex.replace(
    "\\input{sections/logic_T}",
    "\\InputIfFileExists{sections/logic_T}{}{}",
  );
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files_with(
    &tex,
    &[("sections/logic_T.tex", "Included text.\n")],
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Included text.</p>"), "{xml}");
}

/// NFSS size state: `\fontsize{…}{…}\selectfont` sizes the text, `\f@size`
/// follows the size switches, and a `\selectfont` with no pending
/// `\fontsize` (`\bfseries`) keeps the current size.
#[test]
fn fontsize_selectfont_size_state() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/fontsize_selectfont_size_state.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      r#"<p><text fontsize="120%">[12]</text> <text fontsize="240%">[24]</text> <text font="bold" fontsize="120%">c</text></p>"#
    ),
    "{xml}"
  );
}

/// A raw class's `\normalsize` (scrsize11pt.clo through `\@setfontsize`)
/// sizes the text and is the nominal size: body text carries no size,
/// `\large`/`\small` are relative to 10.95 pt, and typearea's good-width
/// measure no longer warns "Bad type area settings!".
#[test]
fn raw_class_normalsize_nominal() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/raw_class_normalsize_nominal.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Bad type area settings"), "{stderr}");
  // Only KOMA's two sectioning-identity warnings may remain (LEDGER 56ig).
  let other_warnings = stderr
    .lines()
    .filter(|l| l.contains("Warning:"))
    .filter(|l| !l.contains("has been") && !l.contains("Unexpected definition of"))
    .count();
  assert_eq!(other_warnings, 0, "{stderr}");
  assert!(
    xml
      .contains(r#"<p>Body <text fontsize="110%">L</text> and <text fontsize="91%">s</text>.</p>"#),
    "{xml}"
  );
}

/// `\PackageNote`/`\ClassNote` are infos (their text reads "Info:"), logged
/// at Info, not counted as warnings (typearea's classic-DIV note).
#[test]
fn package_note_is_info() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/package_note_is_info.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let plain = stderr.replace("\x1b[0m", "");
  assert!(
    plain.contains("Package demo Info: a package note"),
    "{stderr}"
  );
  assert!(plain.contains("Class demo Info: a class note"), "{stderr}");
  assert!(xml.contains("<p>Text.</p>"), "{xml}");
}

/// The K6 ruling (2026-09-24): pdflatex's `\pdfoutput=1` is the default
/// output mode and `\ifpdf` follows `\pdfoutput` (iftex.sty:290-291); a
/// document may still select DVI itself, and XeTeX has no PDF mode test.
#[test]
fn ifpdf_follows_pdfoutput() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/ifpdf_follows_pdfoutput.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[pdf][1]</p>"), "{xml}");
  let dvi = tex.replace(
    "\\documentclass{article}",
    "\\pdfoutput=0\n\\documentclass{article}",
  );
  let (stderr, xml) = convert_with(&dvi, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[dvi][0]</p>"), "{xml}");
}

/// datetime.sty:261-304's predefined time formats (arXiv 2605.13807,
/// 2606.01587 `\settimeformat{hhmmsstime}`); the time is the job's own.
#[test]
fn datetime_time_formats() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/datetime_time_formats.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Only datetime's own "partially ported" notice.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("datetime.sty is only partially ported"),
    "{stderr}"
  );
  let shape =
    regex::Regex::new(r"<p>\[\d\d:\d\d:\d\d\]\[\d{1,2}:\d\d(am| Noon|pm)\]\[\d\d:\d\d\]</p>")
      .unwrap();
  assert!(shape.is_match(&xml), "{xml}");
}

/// doc.sty:895-897 `\DocInput` reads the raw file as documentation (`%`
/// ignored), not the package binding that stands in for its definitions
/// (frankenstein/newclude: recall 1.1 % → 98.5 %).
#[test]
fn docinput_reads_the_file_not_its_binding() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/docinput_reads_the_file_not_its_binding.tex"
  );
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(tex, &[(
    "newclude.sty",
    "% Documented body.\n\\endinput\n",
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Documented body.</p>"), "{xml}");
  assert!(xml.contains("<p>END</p>"), "{xml}");
}

/// algorithm2e's block macros are the listing's structure, locked against a
/// document's box-drawing redefinition (arXiv 2605.20533).
#[test]
fn algorithm2e_block_macros_locked() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/algorithm2e_block_macros_locked.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("_CaptureBlock_"), "{xml}");
  // The loop body is a listing line of its own.
  let line = regex::Regex::new(r"(?s)<listingline[^>]*>(.*?)</listingline>").unwrap();
  assert!(
    line.captures_iter(&xml).any(|c| c[1].contains("update")),
    "{xml}"
  );
}

/// quotchap's `savequote` becomes an epigraph before the chapter it opens:
/// the package prints it from its own `\chapter`, which ours (locked) never
/// runs (quotchap manual; DIVERGENCES #287).
#[test]
fn quotchap_savequote_epigraph() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/quotchap_savequote_epigraph.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let flat = regex::Regex::new(r">\s+<").unwrap().replace_all(&xml, "><");
  assert!(
    flat.contains(
      "<quote class=\"ltx_epigraph ltx_quotchap\"><p>All changes have their melancholy.</p>\
         <block class=\"ltx_epigraph_source\"><p>Anatole France</p></block></quote>"
    ),
    "{xml}"
  );
}

/// `\pdfliteral` scans its spec keyword with expansion, as pdfTeX does
/// (accsupp's pdftex driver: `\pdfliteral\ACCSUPP@pdfliteral{…}`; arXiv
/// 2605.21262, 480 errors once PDF output became the default).
#[test]
fn pdfliteral_keyword_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/pdfliteral_keyword_expands.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>XY</p>"), "{xml}");
}

/// `\pdfxform` numbers a form in `\pdflastxform` and voids its box (pdfbase
/// in PDF output; arXiv 2605.10543).
#[test]
fn pdfxform_numbers_a_form() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/pdfxform_numbers_a_form.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Hello. [void](2)</p>"), "{xml}");
}
