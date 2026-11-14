use crate::cluster::convert_to_xml_contrib_clean;

/// `\mintinline{python}{lambda x: x + 1}` used to map to `\verb`, whose
/// delimiter is the first char `{` — it ran past the closing `}`, emitted two
/// unbalanced-group errors, and broke the `\begin{minted}` block that followed.
/// Routing to `\lstinline` (which accepts braces) keeps it inline and clean.
#[test]
fn mintinline_brace_form_does_not_error_or_swallow() {
  // The strict 0-error gate is itself the primary canary: the old `\verb`
  // mapping raised `unexpected:}` / `unexpected:\endgroup` here.
  let xml =
    convert_to_xml_contrib_clean("tests/cluster_regressions/minted_inline_and_highlighting.tex");
  // Text on both sides of the inline snippet survives (the brace form did not
  // run away), and the following block still renders.
  assert!(
    xml.contains("in a sentence"),
    "text after the inline \\mintinline was swallowed:\n{xml}"
  );
  assert!(
    xml.contains("Tail paragraph after the block"),
    "the \\begin{{minted}} block or its trailing text was swallowed:\n{xml}"
  );
  assert!(
    xml.contains("ltx_lstlisting"),
    "the \\begin{{minted}} block did not render as a listing:\n{xml}"
  );
}

/// `\begin{minted}{python}` now feeds the language to `\lstset`, so listings
/// tags `def`/`if`/`return` as keywords (was plain identifiers — no highlighting).
#[test]
fn minted_language_activates_keyword_highlighting() {
  let xml =
    convert_to_xml_contrib_clean("tests/cluster_regressions/minted_inline_and_highlighting.tex");
  assert!(
    xml.contains("ltx_lst_keyword"),
    "minted block produced no highlighted keywords (language not activated):\n{xml}"
  );
}
