//! Guards for LISTINLINE: an inline listing runs listings.sty's `\lst@Init\relax` … `\lst@DeInit\egroup` bracket
//! (listings.sty:1941-1989), so a hook a document adds to `\lst@DeInit` runs where TeX runs it (witness 2411.07667:
//! realboxes' `\Colorbox{…}\bgroup\appto\lst@DeInit{\egroup}` nested a group per listing, `Fatal:Stomach:Recursion`),
//! and realboxes' `\Colorbox`/`\Fcolorbox` reach the colour internals they call.
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{convert_with, error_count, kpse_has, warning_count};

/// The raw packages the hooked shapes load; a TeX Live without them skips those rows visibly.
const RAW: &[&str] = &["realboxes.sty", "collectbox.sty", "xpatch.sty"];

/// One row per inline entry point: the repro, and the paragraph it converts to, whole.
const INLINE_ROWS: &[(&str, &str, &str)] = &[
  // The `\lst@DeInit` hook under a `|` delimiter (the witness's shape). Before: 6 Errors, only "A" kept.
  (
    "lstinline_deinit_hook_closes_a_colorbox",
    include_str!(
      "../../../tools/perfect_kernel/repros/listings-algorithms/lstinline_deinit_hook_closes_a_colorbox.tex"
    ),
    concat!(
      r#"<p>A <text class="ltx_lstlisting"><text class="ltx_lst_identifier">x</text>.<text class="ltx_lst_identifier">a</text></text>"#,
      r#" and <text class="ltx_lstlisting"><text class="ltx_lst_identifier">y</text>.<text class="ltx_lst_identifier">b</text></text>"#,
      r#" and <text class="ltx_lstlisting"><text class="ltx_lst_identifier">z</text>.<text class="ltx_lst_identifier">c</text></text> end.</p>"#,
    ),
  ),
  // The hook under every other entry point: `{…}`, `[language=C]`, `\lstMakeShortInline`, `\lstinline` direct.
  // Before: 10 Errors, the text after "A" lost.
  (
    "lstinline_deinit_hook_every_entry_point",
    include_str!(
      "../../../tools/perfect_kernel/repros/listings-algorithms/lstinline_deinit_hook_every_entry_point.tex"
    ),
    concat!(
      r#"<p>A <text class="ltx_lstlisting"><text class="ltx_lst_identifier">p</text>.<text class="ltx_lst_identifier">q</text></text>"#,
      r#" B <text class="ltx_lstlisting"><text class="ltx_lst_identifier">r</text>.<text class="ltx_lst_identifier">s</text></text>"#,
      r#" C <text class="ltx_lst_language_C ltx_lstlisting"><text class="ltx_lst_keyword" font="bold">int</text>"#,
      r#"<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">t</text>;</text>"#,
      r#" D <text class="ltx_lstlisting"><text class="ltx_lst_identifier">u</text>.<text class="ltx_lst_identifier">v</text></text>"#,
      r#" E <text class="ltx_lstlisting"><text class="ltx_lst_identifier">w</text>.<text class="ltx_lst_identifier">x</text></text> end.</p>"#,
    ),
  ),
  // The safe twins, no hook: the bracket adds nothing (byte-identical to rc2's XML).
  (
    "lstinline_plain_shapes_stay",
    include_str!(
      "../../../tools/perfect_kernel/repros/listings-algorithms/lstinline_plain_shapes_stay.tex"
    ),
    concat!(
      r#"<p>Text <text class="ltx_lstlisting"><text class="ltx_lst_identifier">a</text>.<text class="ltx_lst_identifier">b</text></text>"#,
      r#" mid. A <text class="ltx_lst_language_C ltx_lstlisting"><text class="ltx_lst_keyword" font="bold">int</text>"#,
      r#"<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">x</text>;</text> B.</p>"#,
    ),
  ),
];

/// listings-algorithms/lstinline_*: each row converts with 0 Errors and 0 Warnings to exactly its paragraph, every
/// inline text kept in order as the binding's inline code element; the twin's display listing is unchanged.
#[test]
fn lstinline_runs_the_init_deinit_bracket() {
  for (name, tex, para) in INLINE_ROWS {
    if let Some(missing) = RAW
      .iter()
      .find(|p| tex.contains("realboxes") && !kpse_has(p))
    {
      eprintln!("SKIP {name}: {missing} is not in this TeX Live");
      continue;
    }
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{name}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{name}: {stderr}");
    assert_eq!(xml.matches("<p>").count(), 1, "{name}: {xml}");
    assert!(xml.contains(para), "{name}: {xml}");
    if *name == "lstinline_plain_shapes_stay" {
      assert!(
        xml.contains(concat!(
          r#"<listing class="ltx_lstlisting" data="eSA9IDE7" dataencoding="base64" datamimetype="text/plain">"#,
          "\n      ",
          r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">y</text><text class="ltx_lst_space"> </text>"#,
          r#"=<text class="ltx_lst_space"> </text>1;</listingline>"#,
          "\n    </listing>"
        )),
        "{name}: {xml}"
      );
    }
  }
}

/// boxes-groups/realboxes_colorbox_and_fcolorbox_call_the_color_internals: realboxes' `\Colorbox` and `\Fcolorbox`
/// call xcolor's `\color@box`/`\color@fb@x` (realboxes.sty:264-279), bound as the user-level boxes. Before: an
/// undefined-macro Error each, the colour names spilled as text.
#[test]
fn realboxes_colorbox_and_fcolorbox_call_the_color_internals() {
  if let Some(missing) = RAW[..2].iter().find(|p| !kpse_has(p)) {
    eprintln!("SKIP: {missing} is not in this TeX Live");
    return;
  }
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/realboxes_colorbox_and_fcolorbox_call_the_color_internals.tex"
    ),
    Some("[rawstyles,rawclasses]latexml.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(concat!(
      r##"<p>A x B <text cssstyle="padding:3.0pt" framecolor="#FF0000" framed="rectangle">y</text>"##,
      r##" C <text cssstyle="padding:3.0pt" framecolor="#FF0000" framed="rectangle">z</text> end.</p>"##
    )),
    "{xml}"
  );
}
