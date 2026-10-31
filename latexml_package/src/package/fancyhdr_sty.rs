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
  // fancyhdr.sty:668 `\f@nch@setoffs`: re-derives the running heads' offsets from `\headwidth`, which
  // a document re-runs after `\newgeometry` (the wheelchart manual, wheelchart.tex:2743). Running heads
  // are not converted, so there is nothing to size.
  def_macro_noop("\\f@nch@setoffs")?;

  // not implemented yet: \fancycenter[][]{}{}{}, since 4.0

  // always false as LaTeXML does not paginate
  DefMacro!("\\iffloatpage{}{}", "#2");
  DefMacro!("\\iftopfloat{}{}", "#2");
  DefMacro!("\\ifbotfloat{}{}", "#2");
  DefMacro!("\\iffootnote{}{}", "#2"); // since 3.8

  // fancyhdr.sty:609-619 `\ps@f@nch@fancyproto`, `\ps@fancy`/`\ps@fancyplain`: the page styles a class
  // probes (`seminar.cls:760 \@ifgoodps{fancy}`) or `\let`s — headers are not
  // modelled, so the styles are empty.
  // fancyhdr.sty:691-711 `\fancypagestyle[*]{name}[base]{defs}` stores the style as `\ps@name`, the base style then
  // the definitions, as TeX does (Perl fancyhdr.sty.ltxml:56 makes it a no-op). Selecting the style still sets no
  // head or foot (`\fancyfoot` and friends are no-ops), but the class's code is there to read: the frontmatter
  // stores read what a title page's page style prints (frontmatter_stores.rs, K11's toks stores: googlecloud.cls
  // :103-114 prints `\the\correspondingauthor` in `firststyle`'s foot, 2607.06973; cmu.cls:144-164, 2608.26053;
  // deepseek.cls:147-171, 2401.14196). The starred form's reset of the fields (`\f@nch@pagestyle@setup`) and the
  // base style's checks (an unknown or non-fancyhdr base is a `\PackageError`) are not modelled: the fields are not.
  RawTeX!(
    r"\def\ps@f@nch@fancyproto{}\@namedef{f@nch@ps@f@nch@fancyproto-is-fancyhdr}{}%
\def\ps@fancy{\ps@f@nch@fancyproto}\def\ps@fancyplain{\ps@f@nch@fancyproto}\@namedef{f@nch@ps@fancy-is-fancyhdr}{}%
\newif\iff@nch@pagestyle@star
\def\fancypagestyle{\@ifstar{\f@nch@pagestyle@startrue\f@nch@pagestyle}{\f@nch@pagestyle@starfalse\f@nch@pagestyle}}%
\def\f@nch@pagestyle#1{\@ifnextchar[{\f@nch@@pagestyle{#1}}{\f@nch@@pagestyle{#1}[f@nch@fancyproto]}}%
\long\def\f@nch@@pagestyle#1[#2]#3{\@namedef{ps@#1}{\@nameuse{ps@#2}#3\relax}\@namedef{f@nch@ps@#1-is-fancyhdr}{}}"
  );
  // fancyhdr.sty:849-851: under the newlfm class, `\ps@@empty` is fancyhdr's
  // version marker, which newlfm.cls:41 tests with `\@ifundefined` (class census
  // 2026-09-24). fancyhdr lets it to `\f@nch@ps@empty`, which is not modelled here:
  // `\ps@empty` stands in, as a `\let` to an undefined name would read undefined.
  RawTeX!(r"\@ifclassloaded{newlfm}{\let\ps@@empty\ps@empty}{}");

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
