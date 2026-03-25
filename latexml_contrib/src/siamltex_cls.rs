use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsthm");
  RawTeX!(r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}");
  Let!("\\Appendix", "\\appendix");
  // TODO: Perl has classification_tokens_for_env sub and DefEnvironment for
  // {AMS}, {AM}, {PII} that emit ltx:classification elements.
  // Stubbed as empty environments for now.
  DefEnvironment!("{AMS}", "#body");
  DefEnvironment!("{AM}", "#body");
  DefEnvironment!("{PII}", "#body");
  DefMacro!("\\begin{romannum}", "\\begin{enumerate}");
  DefMacro!("\\end{romannum}", "\\end{enumerate}");
  DefMacro!("\\begin{remunerate}", "\\begin{enumerate}");
  DefMacro!("\\end{remunerate}", "\\end{enumerate}");
  DefMacro!("\\sixptsize", "\\@setfontsize\\sixptsize{6}{8}");
  DefMacro!("\\fiveptsize", "\\@setfontsize\\fiveptsize{5}{7}");
  DefMacro!("\\simac", "SIAM J{\\fiveptsize OURNAL} M{\\fiveptsize ACRO}");
  DefMacro!("\\siap", "SIAM J.\\ A{\\fiveptsize PPL.} M{\\fiveptsize ATH}");
  DefMacro!("\\sicomp", "SIAM J.\\ C{\\fiveptsize OMPUT}");
  DefMacro!("\\sicon", "SIAM J.\\ C{\\fiveptsize ONTROL}  O{\\fiveptsize PTIM}");
  DefMacro!("\\sidma", "SIAM J.\\ D{\\fiveptsize ISCRETE} M{\\fiveptsize ATH}");
  DefMacro!("\\sima", "SIAM J.\\ M{\\fiveptsize ATH.} A{\\fiveptsize NAL}");
  DefMacro!("\\simax", "SIAM J.\\ M{\\fiveptsize ATRIX} A{\\fiveptsize NAL.} A{\\fiveptsize PPL}");
  DefMacro!("\\sinum", "SIAM J.\\ N{\\fiveptsize UMER.} A{\\fiveptsize NAL}");
  DefMacro!("\\siopt", "SIAM J.\\ O{\\fiveptsize PTIM}");
  DefMacro!("\\sisc", "SIAM J.\\ S{\\fiveptsize CI.} C{\\fiveptsize OMPUT}");
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
  DefMacro!("\\URL", "\\protect\\\\ \\hspace*{15.37pt}http://www.siam.org/journals/");
  DefMacro!("\\sameauthor", "\\relax");
  DefMacro!("\\const", "\\mathop{\\operator@font const}\\nolimits");
  DefMacro!("\\diag", "\\mathop{\\operator@font diag}\\nolimits");
  DefMacro!("\\grad", "\\mathop{\\operator@font grad}\\nolimits");
  DefMacro!("\\Range", "\\mathop{\\operator@font Range}\\nolimits");
  DefMacro!("\\rank", "\\mathop{\\operator@font rank}\\nolimits");
  DefMacro!("\\supp", "\\mathop{\\operator@font supp}\\nolimits");
});
