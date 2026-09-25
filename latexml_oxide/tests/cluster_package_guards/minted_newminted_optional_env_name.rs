use crate::cluster::convert_to_xml_contrib_clean;

#[test]
fn newminted_optional_name_does_not_clobber_display_math() {
  let xml =
    convert_to_xml_contrib_clean("tests/cluster_regressions/minted_newminted_optarg_comment.tex");
  // The runaway listing is the whole bug: with `\[` clobbered, a single
  // `<listing>` swallows everything after the first display equation. The
  // `leancode` block is inside `\begin{comment}` (discarded) and `\[` is math, so
  // a healthy conversion emits NO listing at all.
  assert!(
    !xml.contains("listingline"),
    "a runaway listing swallowed the document (\\[ clobbered by \\newminted):\n{xml}"
  );
  // Content on BOTH sides of the display equation must survive into the flow.
  assert!(
    xml.contains("Body text after the display equation"),
    "body after the display equation was swallowed:\n{xml}"
  );
  assert!(
    xml.contains("Tail paragraph immediately before the bibliography"),
    "tail before the bibliography was swallowed:\n{xml}"
  );
  // `\bibliography` executed (the element exists at the core stage) rather than
  // being consumed as listing text.
  assert!(
    xml.contains("<bibliography "),
    "\\bibliography never executed — swallowed by the runaway listing:\n{xml}"
  );
}

/// The title's literal claim: a `\newminted`-created env used the normal way
/// (outside a comment) must close at its own `\end{name}` and NOT read on to the
/// literal `\end{lstlisting}`. The created env now delegates to
/// `\lstnewenvironment{name}`, whose verbatim reader stops at `\end{name}`.
#[test]
fn newminted_env_closes_at_its_own_end_marker() {
  let xml =
    convert_to_xml_contrib_clean("tests/cluster_regressions/minted_newminted_direct_env.tex");
  // The env body IS captured as a listing (the code is tokenized into
  // `<text class="ltx_lst_*">` spans, so the raw string is not contiguous — key
  // off the listing element instead).
  assert!(
    xml.contains("ltx_lstlisting"),
    "the \\newminted env body was not captured as a listing:\n{xml}"
  );
  // …and it closes at `\end{leancode}`, so text after it stays in the document
  // flow instead of being swallowed to EOF.
  assert!(
    xml.contains("After the code listing"),
    "content after the \\newminted env was swallowed (ran past \\end{{name}}):\n{xml}"
  );
}
