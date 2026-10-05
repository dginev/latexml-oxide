//! Stub for INFORMS journal classes (informs, informs3).
//!
//! The informs* classes are used by Operations Research, Management Science,
//! and other INFORMS journals. They define a large frontmatter API
//! (\TITLE, \ARTICLEAUTHORS, \ABSTRACT, \KEYWORDS, etc.) inside the cls,
//! which we never raw-load. Provide gobble stubs so papers using this
//! class convert without "undefined" cascades.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");

  // Frontmatter / paper metadata — preserve author content.
  // \TITLE / \ARTICLETITLE → \title to populate the document title.
  DefMacro!("\\TITLE{}", "\\title{#1}");
  DefMacro!("\\ARTICLETITLE{}", "\\title{#1}");
  // Running header variants — short title for header; preserve as note.
  DefMacro!(
    "\\RUNAUTHOR{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}"
  );
  DefMacro!(
    "\\RUNTITLE{}",
    "\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}"
  );
  DefMacro!(
    "\\ECRUNAUTHOR{}",
    "\\@add@frontmatter{ltx:note}[role=ec-runningauthor]{#1}"
  );
  DefMacro!(
    "\\ECRUNTITLE{}",
    "\\@add@frontmatter{ltx:note}[role=ec-runningtitle]{#1}"
  );
  // \AUTHOR{name}{affiliation} — emit name as author, affiliation as note.
  DefMacro!(
    "\\AUTHOR{}{}",
    "\\author{#1}\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  DefMacro!(
    "\\AFF[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  // \ABSTRACT → abstract env so the text is preserved as document abstract.
  DefMacro!("\\ABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\ARTICLEABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!(
    "\\KEYWORDS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\MANUSCRIPTNO{}",
    "\\@add@frontmatter{ltx:note}[role=manuscriptno]{#1}"
  );
  DefMacro!(
    "\\HISTORY{}",
    "\\@add@frontmatter{ltx:note}[role=history]{#1}"
  );
  DefMacro!(
    "\\ARTICLEAUTHORS{}",
    "\\@add@frontmatter{ltx:note}[role=authors]{#1}"
  );
  DefMacro!(
    "\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}"
  );
  def_macro_noop("\\thetitle")?;

  // Layout / structure switches — no visual effect in XML.
  // informs3.cls:1511-1527, 2089, 2117-2124, 2442-2523, 2727-2738: the theorem sets a paper picks (numbered through
  // or by section; the class's theorem.sty styles TH and EX, :1809-1856, are amsthm's plain and definition: an italic
  // and an upright body), the equation numbering, and the e-companion switch that renumbers sections,
  // equations, theorems, figures and tables `EC.n` — `{assumption}` and `\ECSwitch` were undefined (2609.08001).
  RawTeX!(
    r"\def\TheoremsNumberedThrough{\theoremstyle{plain}\newtheorem{theorem}{Theorem}\newtheorem{lemma}{Lemma}\newtheorem{proposition}{Proposition}\newtheorem{corollary}{Corollary}\newtheorem{claim}{Claim}\newtheorem{conjecture}{Conjecture}\newtheorem{hypothesis}{Hypothesis}\newtheorem{assumption}{Assumption}\theoremstyle{definition}\newtheorem{remark}{Remark}\newtheorem{example}{Example}\newtheorem{problem}{Problem}\newtheorem{definition}{Definition}\newtheorem{question}{Question}\newtheorem{answer}{Answer}\newtheorem{exercise}{Exercise}\def\ECHowTheorems{\setcounter{theorem}{0}\expandafter\let\csname oldthetheorem\expandafter\endcsname\csname thetheorem\endcsname\expandafter\def\csname thetheorem\endcsname{EC.\csname oldthetheorem\endcsname}\setcounter{lemma}{0}\expandafter\let\csname oldthelemma\expandafter\endcsname\csname thelemma\endcsname\expandafter\def\csname thelemma\endcsname{EC.\csname oldthelemma\endcsname}\setcounter{proposition}{0}\expandafter\let\csname oldtheproposition\expandafter\endcsname\csname theproposition\endcsname\expandafter\def\csname theproposition\endcsname{EC.\csname oldtheproposition\endcsname}\setcounter{corollary}{0}\expandafter\let\csname oldthecorollary\expandafter\endcsname\csname thecorollary\endcsname\expandafter\def\csname thecorollary\endcsname{EC.\csname oldthecorollary\endcsname}\setcounter{claim}{0}\expandafter\let\csname oldtheclaim\expandafter\endcsname\csname theclaim\endcsname\expandafter\def\csname theclaim\endcsname{EC.\csname oldtheclaim\endcsname}\setcounter{conjecture}{0}\expandafter\let\csname oldtheconjecture\expandafter\endcsname\csname theconjecture\endcsname\expandafter\def\csname theconjecture\endcsname{EC.\csname oldtheconjecture\endcsname}\setcounter{hypothesis}{0}\expandafter\let\csname oldthehypothesis\expandafter\endcsname\csname thehypothesis\endcsname\expandafter\def\csname thehypothesis\endcsname{EC.\csname oldthehypothesis\endcsname}\setcounter{assumption}{0}\expandafter\let\csname oldtheassumption\expandafter\endcsname\csname theassumption\endcsname\expandafter\def\csname theassumption\endcsname{EC.\csname oldtheassumption\endcsname}\setcounter{remark}{0}\expandafter\let\csname oldtheremark\expandafter\endcsname\csname theremark\endcsname\expandafter\def\csname theremark\endcsname{EC.\csname oldtheremark\endcsname}\setcounter{example}{0}\expandafter\let\csname oldtheexample\expandafter\endcsname\csname theexample\endcsname\expandafter\def\csname theexample\endcsname{EC.\csname oldtheexample\endcsname}\setcounter{problem}{0}\expandafter\let\csname oldtheproblem\expandafter\endcsname\csname theproblem\endcsname\expandafter\def\csname theproblem\endcsname{EC.\csname oldtheproblem\endcsname}\setcounter{definition}{0}\expandafter\let\csname oldthedefinition\expandafter\endcsname\csname thedefinition\endcsname\expandafter\def\csname thedefinition\endcsname{EC.\csname oldthedefinition\endcsname}\setcounter{question}{0}\expandafter\let\csname oldthequestion\expandafter\endcsname\csname thequestion\endcsname\expandafter\def\csname thequestion\endcsname{EC.\csname oldthequestion\endcsname}\setcounter{answer}{0}\expandafter\let\csname oldtheanswer\expandafter\endcsname\csname theanswer\endcsname\expandafter\def\csname theanswer\endcsname{EC.\csname oldtheanswer\endcsname}\setcounter{exercise}{0}\expandafter\let\csname oldtheexercise\expandafter\endcsname\csname theexercise\endcsname\expandafter\def\csname theexercise\endcsname{EC.\csname oldtheexercise\endcsname}}}
\def\TheoremsNumberedBySection{\theoremstyle{plain}\newtheorem{theorem}{Theorem}[section]\newtheorem{lemma}{Lemma}[section]\newtheorem{proposition}{Proposition}[section]\newtheorem{corollary}{Corollary}[section]\newtheorem{claim}{Claim}[section]\newtheorem{conjecture}{Conjecture}[section]\newtheorem{hypothesis}{Hypothesis}[section]\newtheorem{assumption}{Assumption}[section]\theoremstyle{definition}\newtheorem{remark}{Remark}[section]\newtheorem{example}{Example}[section]\newtheorem{problem}{Problem}[section]\newtheorem{definition}{Definition}[section]\newtheorem{question}{Question}[section]\newtheorem{answer}{Answer}[section]\newtheorem{exercise}{Exercise}[section]\def\ECHowTheorems{\setcounter{theorem}{0}\expandafter\let\csname oldthetheorem\expandafter\endcsname\csname thetheorem\endcsname\expandafter\def\csname thetheorem\endcsname{\csname oldthetheorem\endcsname}\setcounter{lemma}{0}\expandafter\let\csname oldthelemma\expandafter\endcsname\csname thelemma\endcsname\expandafter\def\csname thelemma\endcsname{\csname oldthelemma\endcsname}\setcounter{proposition}{0}\expandafter\let\csname oldtheproposition\expandafter\endcsname\csname theproposition\endcsname\expandafter\def\csname theproposition\endcsname{\csname oldtheproposition\endcsname}\setcounter{corollary}{0}\expandafter\let\csname oldthecorollary\expandafter\endcsname\csname thecorollary\endcsname\expandafter\def\csname thecorollary\endcsname{\csname oldthecorollary\endcsname}\setcounter{claim}{0}\expandafter\let\csname oldtheclaim\expandafter\endcsname\csname theclaim\endcsname\expandafter\def\csname theclaim\endcsname{\csname oldtheclaim\endcsname}\setcounter{conjecture}{0}\expandafter\let\csname oldtheconjecture\expandafter\endcsname\csname theconjecture\endcsname\expandafter\def\csname theconjecture\endcsname{\csname oldtheconjecture\endcsname}\setcounter{hypothesis}{0}\expandafter\let\csname oldthehypothesis\expandafter\endcsname\csname thehypothesis\endcsname\expandafter\def\csname thehypothesis\endcsname{\csname oldthehypothesis\endcsname}\setcounter{assumption}{0}\expandafter\let\csname oldtheassumption\expandafter\endcsname\csname theassumption\endcsname\expandafter\def\csname theassumption\endcsname{\csname oldtheassumption\endcsname}\setcounter{remark}{0}\expandafter\let\csname oldtheremark\expandafter\endcsname\csname theremark\endcsname\expandafter\def\csname theremark\endcsname{\csname oldtheremark\endcsname}\setcounter{example}{0}\expandafter\let\csname oldtheexample\expandafter\endcsname\csname theexample\endcsname\expandafter\def\csname theexample\endcsname{\csname oldtheexample\endcsname}\setcounter{problem}{0}\expandafter\let\csname oldtheproblem\expandafter\endcsname\csname theproblem\endcsname\expandafter\def\csname theproblem\endcsname{\csname oldtheproblem\endcsname}\setcounter{definition}{0}\expandafter\let\csname oldthedefinition\expandafter\endcsname\csname thedefinition\endcsname\expandafter\def\csname thedefinition\endcsname{\csname oldthedefinition\endcsname}\setcounter{question}{0}\expandafter\let\csname oldthequestion\expandafter\endcsname\csname thequestion\endcsname\expandafter\def\csname thequestion\endcsname{\csname oldthequestion\endcsname}\setcounter{answer}{0}\expandafter\let\csname oldtheanswer\expandafter\endcsname\csname theanswer\endcsname\expandafter\def\csname theanswer\endcsname{\csname oldtheanswer\endcsname}\setcounter{exercise}{0}\expandafter\let\csname oldtheexercise\expandafter\endcsname\csname theexercise\endcsname\expandafter\def\csname theexercise\endcsname{\csname oldtheexercise\endcsname}}}
\def\EquationsNumberedThrough{\def\theequation{\arabic{equation}}\def\ECHowEquations{\setcounter{equation}{0}\let\oldtheequation\theequation\def\theequation{EC.\oldtheequation}}}
\def\EquationsNumberedBySection{\@addtoreset{equation}{section}\def\theequation{\thesection.\arabic{equation}}\def\ECHowEquations{\setcounter{equation}{0}\let\oldtheequation\theequation\def\theequation{EC.\oldtheequation}}}
\EquationsNumberedThrough
\def\ECHowSections{\setcounter{section}{0}\renewcommand\thesection{EC.\@arabic\c@section}\renewcommand\thesubsection{\thesection.\@arabic\c@subsection}\renewcommand\thesubsubsection{\thesubsection.\@arabic\c@subsubsection}\renewcommand\theparagraph{\thesubsubsection.\@arabic\c@paragraph}\renewcommand\thesubparagraph{\theparagraph.\@arabic\c@subparagraph}}
\def\ECSwitch{\ECHowTheorems\ECHowEquations\ECHowSections\setcounter{figure}{0}\renewcommand\thefigure{EC.\@arabic\c@figure}\setcounter{table}{0}\renewcommand\thetable{EC.\@arabic\c@table}}
\def\ECHead#1{\par\noindent{\raggedright\bfseries #1\par}}"
  );
  def_macro_noop("\\ECRepeatTheorems")?;
  def_macro_noop("\\OneAndAHalfSpacedXI")?;
  def_macro_noop("\\OneAndAHalfSpacedXII")?;
  def_macro_noop("\\DoubleSpacedXI")?;
  def_macro_noop("\\DoubleSpacedXII")?;
  def_macro_noop("\\SingleSpacedXI")?;

  // {APPENDICES} env — render contents as appendix section.
  DefMacro!(T_CS!("\\begin{APPENDICES}"), None, "\\appendix");
  DefMacro!(T_CS!("\\end{APPENDICES}"), None, "");

  // informs3.cls L932: `\def\Halmos{\mbox{\quad$\square$}}` — proof-end
  // QED box. Render the square in math mode.
  DefMacro!("\\Halmos", "\\ensuremath{\\square}");
  // informs3.cls L1231: `\def\EMAIL#1{#1}` — used within \AFF; plain
  // passthrough of the email text.
  DefMacro!("\\EMAIL{}", "#1");
  // informs3.cls L1273: `\long\def\ACKNOWLEDGMENT#1{\section*{\bf
  // \theACKname.}{#1}}` (\theACKname defaults to "Acknowledgments").
  // Route the body to a structural acknowledgements block (see
  // feedback: prefer ltx:acknowledgements over a flattened \section*).
  DefConstructor!(
    "\\ACKNOWLEDGMENT{}",
    "<ltx:acknowledgements name='Acknowledgments'>#1</ltx:acknowledgements>"
  );
  // \ACKname{name} sets the acknowledgements heading name.
  def_macro_noop("\\ACKname{}")?;
  // informs3.cls L2642: `\newenvironment{APPENDIX}[1]{…appendix with
  // title #1…}`. The singular env wraps a single titled appendix.
  // Begin enters appendix mode + emits the title as a section; the arg
  // is read by the helper. Mirrors the {APPENDICES} handling.
  DefMacro!(
    T_CS!("\\begin{APPENDIX}"),
    None,
    "\\appendix\\lx@informs@appendixhead"
  );
  DefMacro!("\\lx@informs@appendixhead{}", "\\section{#1}");
  DefMacro!(T_CS!("\\end{APPENDIX}"), None, "");
});
