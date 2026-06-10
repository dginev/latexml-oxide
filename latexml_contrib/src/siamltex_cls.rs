use latexml_package::prelude::*;

/// Push an `{AMS}`/`{AM}`/`{PII}` classification environment's digested body
/// into the document `frontmatter` as `<ltx:classification scheme=…>`, instead
/// of constructing it inline where the env was invoked. Mirrors Perl
/// siamltex.cls.ltxml (`classification_tokens_for_env` → `\@add@frontmatter`)
/// and OmniBus's `push_keyword_body_to_frontmatter`, just scheme-parameterized.
/// Floating is REQUIRED because these envs are routinely used INSIDE
/// `\begin{abstract}…\end{abstract}` (SIAM house style), where an inline
/// `<ltx:classification>` is a content-model violation
/// ("ltx:classification isn't allowed in ltx:abstract"). Witness 2009.00379.
fn push_classification_to_frontmatter(
  whatsit: &mut latexml_core::whatsit::Whatsit,
  scheme: &str,
) -> latexml_core::Result<Vec<latexml_core::digested::Digested>> {
  use latexml_core::BoxOps;
  use latexml_core::common::store::Stored;
  if let Some(body) = whatsit.get_body()? {
    let mut attrs: rustc_hash::FxHashMap<String, String> = rustc_hash::FxHashMap::default();
    attrs.insert("scheme".to_string(), scheme.to_string());
    let entry = latexml_core::document::tag::TagData {
      tag: "ltx:classification".to_string(),
      attr: attrs,
      content: vec![latexml_core::document::tag::TagContent::Box(body)],
    };
    latexml_core::state::with_value_mut("frontmatter", |val_opt| {
      if let Some(Stored::HashTagData(ref mut frnt)) = val_opt {
        frnt
          .entry("ltx:classification".to_string())
          .or_insert_with(Vec::new)
          .push(entry);
      }
    });
  }
  Ok(Vec::new())
}

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsthm");
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}"
  );
  Let!("\\Appendix", "\\appendix");
  // Perl siamltex.cls.ltxml L27-40: `classification_tokens_for_env` unreads
  // `\@add@frontmatter{ltx:classification}[scheme=<TYPE>]{body}` via
  // `afterDigestBody` — i.e. it FLOATS the classification into the document
  // frontmatter rather than emitting it inline. We must do the same (not the
  // earlier direct-inline-XML form): SIAM papers put `\begin{AMS}…\end{AMS}`
  // INSIDE `\begin{abstract}`, and an inline `<ltx:classification>` there is a
  // content-model violation. Witness 2009.00379 (`{keywords}`+`{AMS}` inside
  // `{abstract}`; `{keywords}` already floats via OmniBus, `{AMS}` did not).
  DefEnvironment!("{AMS}", "",
    after_digest_body => sub[whatsit] { push_classification_to_frontmatter(whatsit, "AMS") });
  DefEnvironment!("{AM}", "",
    after_digest_body => sub[whatsit] { push_classification_to_frontmatter(whatsit, "AM") });
  DefEnvironment!("{PII}", "",
    after_digest_body => sub[whatsit] { push_classification_to_frontmatter(whatsit, "PII") });
  DefMacro!(T_CS!("\\begin{romannum}"), None, "\\begin{enumerate}");
  DefMacro!(T_CS!("\\end{romannum}"), None, "\\end{enumerate}");
  DefMacro!(T_CS!("\\begin{remunerate}"), None, "\\begin{enumerate}");
  DefMacro!(T_CS!("\\end{remunerate}"), None, "\\end{enumerate}");
  DefMacro!("\\sixptsize", "\\@setfontsize\\sixptsize{6}{8}");
  DefMacro!("\\fiveptsize", "\\@setfontsize\\fiveptsize{5}{7}");
  DefMacro!(
    "\\simac",
    "SIAM J{\\fiveptsize OURNAL} M{\\fiveptsize ACRO}"
  );
  DefMacro!(
    "\\siap",
    "SIAM J.\\ A{\\fiveptsize PPL.} M{\\fiveptsize ATH}"
  );
  DefMacro!("\\sicomp", "SIAM J.\\ C{\\fiveptsize OMPUT}");
  DefMacro!(
    "\\sicon",
    "SIAM J.\\ C{\\fiveptsize ONTROL}  O{\\fiveptsize PTIM}"
  );
  DefMacro!(
    "\\sidma",
    "SIAM J.\\ D{\\fiveptsize ISCRETE} M{\\fiveptsize ATH}"
  );
  DefMacro!(
    "\\sima",
    "SIAM J.\\ M{\\fiveptsize ATH.} A{\\fiveptsize NAL}"
  );
  DefMacro!(
    "\\simax",
    "SIAM J.\\ M{\\fiveptsize ATRIX} A{\\fiveptsize NAL.} A{\\fiveptsize PPL}"
  );
  DefMacro!(
    "\\sinum",
    "SIAM J.\\ N{\\fiveptsize UMER.} A{\\fiveptsize NAL}"
  );
  DefMacro!("\\siopt", "SIAM J.\\ O{\\fiveptsize PTIM}");
  DefMacro!(
    "\\sisc",
    "SIAM J.\\ S{\\fiveptsize CI.} C{\\fiveptsize OMPUT}"
  );
  DefMacro!("\\sirev", "SIAM R{\\fiveptsize EV}");
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\keywordsname", "Key words");
  DefMacro!("\\AMSname", "AMS subject classifications");
  DefMacro!("\\AMname", "AMS subject classification");
  DefMacro!("\\PIIname", "PII");
  DefMacro!(
    "\\URL",
    "\\protect\\\\ \\hspace*{15.37pt}http://www.siam.org/journals/"
  );
  DefMacro!("\\sameauthor", "\\relax");
  DefMacro!("\\const", "\\mathop{\\operator@font const}\\nolimits");
  DefMacro!("\\diag", "\\mathop{\\operator@font diag}\\nolimits");
  DefMacro!("\\grad", "\\mathop{\\operator@font grad}\\nolimits");
  DefMacro!("\\Range", "\\mathop{\\operator@font Range}\\nolimits");
  DefMacro!("\\rank", "\\mathop{\\operator@font rank}\\nolimits");
  DefMacro!("\\supp", "\\mathop{\\operator@font supp}\\nolimits");
});
