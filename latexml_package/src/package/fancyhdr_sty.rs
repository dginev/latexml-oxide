// Thanks to Kim Philipp Jablonski <kpjkpjkpjkpjkpjkpj@gmail.com>
// of the arXMLiv group for initial implementation
//    http://arxmliv.kwarc.info/
// Released under the Gnu Public License
// Released to the Public Domain

use crate::prelude::*;

LoadDefinitions!({
  def_macro_noop("\\fancyhead[]{}")?;
  def_macro_noop("\\fancyfoot[]{}")?;
  def_macro_noop("\\fancyhf[]{}")?;

  def_macro_noop("\\fancyheadoffset[]{}")?;
  def_macro_noop("\\fancyfootoffset[]{}")?;
  def_macro_noop("\\fancyhfoffset[]{}")?;

  DefMacro!("\\headrulewidth", "0.4pt");
  DefMacro!("\\footrulewidth", "0pt");
  DefMacro!("\\headruleskip", "0pt"); // since 4.0
  DefMacro!("\\footruleskip", ".3\\normalbaselineskip");
  def_macro_noop("\\headrule")?;
  def_macro_noop("\\footrule")?;
  DefRegister!("\\headwidth" => Dimension(0)); // maybe need some other value here?

  def_macro_noop("\\fancyheadinit{}")?; // since 4.0
  def_macro_noop("\\fancyfootinit{}")?; // since 4.0
  def_macro_noop("\\fancyhfinit{}")?; // since 4.0

  // not implemented yet: \fancycenter[][]{}{}{}, since 4.0

  // always false as LaTeXML does not paginate
  DefMacro!("\\iffloatpage{}{}", "#2");
  DefMacro!("\\iftopfloat{}{}", "#2");
  DefMacro!("\\ifbotfloat{}{}", "#2");
  DefMacro!("\\iffootnote{}{}", "#2"); // since 3.8

  def_macro_noop("\\fancypagestyle{}[]{}")?;

  // extramarks.sty not implemented, as its commands can only be used in headers and footers

  // not defined outside of headers and footers
  // def_macro_noop("\\nouppercase")?;

  // deprecated commands
  def_macro_noop("\\lhead[]{}")?;
  def_macro_noop("\\chead[]{}")?;
  def_macro_noop("\\rhead[]{}")?;

  def_macro_noop("\\lfoot[]{}")?;
  def_macro_noop("\\cfoot[]{}")?;
  def_macro_noop("\\rfoot[]{}")?;

  def_macro_noop("\\fancyplain{}{}")?;

  DefMacro!("\\plainheadrulewidth", "0pt");
  DefMacro!("\\plainfootrulewidth", "0pt");
  // fancyhdr.sty:577-608 (v4): `\f@nch@initialise` sets the section marks and
  // the rule defaults, then the default head/foot fields; the package runs
  // it once at load. ctex's end-of-package hook (ctex-heading-article.def:
  // 672-686, ctexbook :800-814) patches it four times and EXECUTES it — with
  // the v3-era binding it was `undefined:\f@nch@initialise` (inkpaper,
  // sduthesis, shtthesis, caspervector; ctex+fancyhdr under any engine).
  // Kept faithful to the real body so ctexpatch finds the patterns it looks
  // for; the head/foot calls are the no-ops above. Guard:
  // `perfect_kernel_batch54::fancyhdr_initialise_is_defined`.
  RawTeX!(
    r"\newif\if@fancyplain
\@ifundefined{@chapapp}{\let\@chapapp\chaptername}{}%
\def\f@nch@initialise{%
  \@ifundefined{chapter}%
   {\def\sectionmark##1{\markboth{\MakeUppercase{\ifnum \c@secnumdepth>\z@
          \thesection\hskip 1em\relax
        \fi ##1}}{}}%
    \def\subsectionmark##1{\markright {\ifnum \c@secnumdepth >\@ne
      \thesubsection\hskip 1em\relax \fi ##1}}}%
   {\def\chaptermark##1{\markboth {\MakeUppercase{\ifnum
        \c@secnumdepth>\m@ne \@chapapp\ \thechapter. \ \fi ##1}}{}}%
    \def\sectionmark##1{\markright{\MakeUppercase{\ifnum \c@secnumdepth >\z@
        \thesection. \ \fi ##1}}}%
   }%
  \def\headrule{{\if@fancyplain\let\headrulewidth\plainheadrulewidth\fi
      \hrule\@height\headrulewidth\@width\headwidth
      \vskip-\headrulewidth}}%
  \def\footrule{{\if@fancyplain\let\footrulewidth\plainfootrulewidth\fi
      \hrule\@width\headwidth\@height\footrulewidth}}%
  \def\headrulewidth{0.4pt}%
  \def\footrulewidth{0pt}%
  \def\headruleskip{0pt}%
  \def\footruleskip{0.3\normalbaselineskip}%
  \fancyhf{}%
  \if@twoside
    \fancyhead[el,or]{\fancyplain{}{\slshape\rightmark}}%
    \fancyhead[er,ol]{\fancyplain{}{\slshape\leftmark}}%
  \else
    \fancyhead[l]{\fancyplain{}{\slshape\rightmark}}%
    \fancyhead[r]{\fancyplain{}{\slshape\leftmark}}%
  \fi
  \fancyfoot[c]{\rmfamily\thepage}%
}
\f@nch@initialise"
  );
});
