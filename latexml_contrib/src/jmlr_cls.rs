//! Stub for jmlr.cls and clear2025.cls family.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Do NOT eager-load xcolor (Perl ships no jmlr binding → OmniBus, no
  // preload). A preloaded xcolor makes a later `\usepackage[table]{xcolor}`
  // a no-op → colortbl/array never load → array `m{}`/`b{}` columns are
  // "Unrecognized tabular template" → "Extra alignment tab". The document
  // loads xcolor with its own options; `\color`/`\definecolor` stay
  // available via hyperref→color. See ifacconf_cls.rs / SYNC_STATUS.
  RequirePackage!("hyperref");
  RequirePackage!("natbib");

  // Author block. jmlr.cls L259/374-449:
  //   \author[short]{ \Name{N1} \Email{E1} \\ \Name{N2} \Email{E2}
  //                    \\ \addr Affiliation }
  // The `\Name`/`\Email`/`\addr` sub-macros carry the structure. The generic
  // \and/comma author splitter mis-reads it — it crams every \Name into a
  // single <personname> and splits the affiliation's commas into phantom
  // authors ("Oxford", "UK"). So when the body uses \Name, digest it directly
  // into structured creators (each \Name → ltx:creator/ltx:personname, \Email →
  // ltx:contact[role=email], the trailing \addr block → ltx:contact
  // [role=affiliation]); otherwise fall back to the standard \author splitter.
  // Beyond-Perl (Perl ships no jmlr binding). Witness 2410.16138, 2409.07012.
  let_i(&T_CS!("\\lx@jmlr@core@author"), &T_CS!("\\author"), None);
  DefMacro!("\\author[]{}", sub[(short, body)] {
    if body.to_string().contains("\\Name") {
      Ok(Invocation!(T_CS!("\\lx@jmlr@structauthor"), vec![Some(body)]))
    } else {
      Ok(Invocation!(T_CS!("\\lx@jmlr@core@author"), vec![short, Some(body)]))
    }
  });
  // Digest the structured body in a group so the row separators (\\, \and, \AND)
  // collapse to spaces; a sentinel is appended so a trailing \addr can capture
  // the (shared) affiliation up to it.
  DefMacro!(
    "\\lx@jmlr@structauthor{}",
    "\\begingroup\\def\\\\{ }\\def\\and{ }\\def\\AND{ }#1\\lx@jmlr@endaddr\\endgroup"
  );
  DefMacro!("\\Name[]{}", "\\lx@add@creator[role=author]{#2}");
  DefMacro!("\\Email{}", "\\lx@add@email{#1}");
  DefMacro!("\\addr Until:\\lx@jmlr@endaddr", "\\lx@add@affiliation{#1}");
  def_macro_noop("\\lx@jmlr@endaddr")?;
  // jmlr.cls \nametag{tag} appends a marker (typically a \thanks) next to an
  // author name inside \Name{... \nametag{\thanks{...}}}. Render its content
  // inline so the wrapped \thanks becomes a note; without this the control
  // word leaks as literal `\nametag`. Witness 2410.16138.
  DefMacro!("\\nametag{}", "#1");
  // \IncludeName{firstname}{lastname} — author name parts in JMLR
  // bibliography. Preserve as ltx:note (rare in main paper body).
  DefMacro!(
    "\\IncludeName{}{}",
    "\\@add@frontmatter{ltx:note}[role=name]{#1 #2}"
  );
  DefMacro!("\\And", " ");
  // \acks{text} — JMLR Acknowledgments-and-Disclosure-of-Funding
  // section. Author body; emit as structural ltx:acknowledgements
  // (matches jmlr2e \acks treatment from commit 78bd49f1e2).
  DefConstructor!(
    "\\acks{}",
    "<ltx:acknowledgements name='acknowledgments-disclosure-of-funding'>#1</ltx:acknowledgements>"
  );
  DefMacro!("\\clearauthor{}", "\\author{#1}");

  // Frontmatter / pagination ceremony — preserve as ltx:note.
  DefMacro!(
    "\\jmlrheading{}{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4 #5 #6}"
  );
  DefMacro!(
    "\\jmlrvolume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  // JMLR frontmatter — preserve author-typed metadata as ltx:note so
  // it reaches the XML (content-preserving). Year/page/workshop/dates
  // are short scalars but the editor list is real prose authors care
  // about; gobbling drops attribution.
  DefMacro!(
    "\\jmlryear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\jmlrworkshop{}",
    "\\@add@frontmatter{ltx:note}[role=workshop]{#1}"
  );
  DefMacro!(
    "\\jmlrsubmitted{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}"
  );
  DefMacro!(
    "\\jmlrpublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}"
  );
  DefMacro!(
    "\\jmlrproceedings{}{}",
    "\\@add@frontmatter{ltx:note}[role=proceedings]{#1: #2}"
  );
  DefMacro!(
    "\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}"
  );
  DefMacro!(
    "\\editors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}"
  );
  def_macro_noop("\\firstpageno{}")?;

  // {keywords} env.
  DefEnvironment!(
    "{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );

  // jmlrcombine helpers used in tables / floats.
  DefMacro!("\\floatconts{}{}{}", "#3");
  DefMacro!("\\tableref{}", "#1");
  DefMacro!("\\figureref{}", "#1");
  DefMacro!("\\algorithmref{}", "#1");
  // jmlrutils.sty L86-140: reference helpers. Stub as the LaTeX
  // \ref expansion so cross-refs still resolve. Witness 2409.07012.
  DefMacro!("\\sectionref{}", "\\ref{#1}");
  DefMacro!("\\appendixref{}", "\\ref{#1}");
  DefMacro!("\\equationref{}", "(\\ref{#1})");
  DefMacro!("\\theoremref{}", "\\ref{#1}");
  DefMacro!("\\lemmaref{}", "\\ref{#1}");
  DefMacro!("\\corollaryref{}", "\\ref{#1}");
  DefMacro!("\\propositionref{}", "\\ref{#1}");
  DefMacro!("\\definitionref{}", "\\ref{#1}");
  DefMacro!("\\exampleref{}", "\\ref{#1}");
  DefMacro!("\\remarkref{}", "\\ref{#1}");

  // jmlrutils theorem-style configuration helpers (gobble silently —
  // we don't replicate the punctuation/spacing). Witness: 2502.19625
  // (\theorempostheader{:}).
  def_macro_noop("\\theorempostheader{}")?;
  def_macro_noop("\\theoremheader{}")?;
  def_macro_noop("\\theoremsep{}")?;
  def_macro_noop("\\theoremprework{}")?;
  def_macro_noop("\\theorempostwork{}")?;
  def_macro_noop("\\theorembodyfont{}")?;
  def_macro_noop("\\theoremheaderfont{}")?;
  def_macro_noop("\\definetheoremstyle{}{}")?;
  def_macro_noop("\\settheoremtag{}")?;

  // Theorem-likes.
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{example}[theorem]{Example}
\newtheorem{remark}[theorem]{Remark}"
  );
});
