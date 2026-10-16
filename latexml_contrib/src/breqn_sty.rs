use latexml_package::{
  engine::latex_constructs::{after_equation, before_equation, prepare_equation_counter},
  prelude::*,
};

// breqn.sty (TeX Live 2025, v0.98l): automatic line breaking of displayed equations. The binding keeps breqn's
// display environments as LaTeXML equations and equation groups and does not read the package raw: breqn.sty
// typesets each display by measuring and re-breaking boxes (:441-1700), and the flexisym.sty it loads replaces the
// math characters' mathcodes by macros (flexisym.sty:178-400), so a raw load would leave the formulas without their
// symbols' meaning. The line breaks are the renderer's to make. Witnesses (68b, binding gaps G3): 0903.2087,
// 1008.1366, 1107.4667, 1111.2451, 1201.6197, 1302.6047, 1303.0187, 1305.0701, 1306.0788 (`{dgroup}`/`{dgroup*}`
// undefined), 1606.07330 (`{dsuspend}`), 1201.6197, 1607.06055, 1706.02708, 1811.11912 (`{dseries}`), 2008.13194
// (`\eqinterlinepenalty`, `\setkeys{breqn}`), 1805.03313 (`\eqlinespacing`, `\eqlineskip`, `\intereqskip`).
LoadDefinitions!({
  // breqn.sty:47-48 `\RequirePackage{amsmath}` and `{graphicx}`, :56 `{keyval,calc}`: a document can rely on them
  // (OXIDIZED_DESIGN #317).
  RequirePackage!("amsmath");
  RequirePackage!("graphicx");
  RequirePackage!("keyval");
  RequirePackage!("calc");
  Warn!(
    "missing_file",
    "breqn.sty",
    "breqn.sty is minimally stubbed — its displays are equations and equation groups, without breqn's automatic line \
     breaking; it will not be interpreted raw."
  );
  // Forbid loading this package, even locally, until we can implement it natively
  DefMacro!("\\condition", "\\text");
  DefMacro!("\\hiderel{}", "#1");
  // breqn.sty:441-472 `{dmath}`/`{dmath*}`: one displayed equation, numbered or not, its options set with
  // `\setkeys{breqn}` (:448). A `label` names the equation (:383-385), a `number` is its tag (:389-391).
  DefMacro!(
    T_CS!("\\begin{dmath}"),
    "OptionalUndigested",
    "\\begin{equation}\\setkeys{breqn}{#1}"
  );
  DefMacro!(T_CS!("\\end{dmath}"), None, "\\end{equation}");
  DefMacro!(
    T_CS!("\\begin{dmath*}"),
    "OptionalUndigested",
    "\\begin{equation*}\\setkeys{breqn}{#1}"
  );
  DefMacro!(T_CS!("\\end{dmath*}"), None, "\\end{equation*}");
  // breqn.sty:383-436: the option keys of the display environments and of `\breqnsetup` (:57). A `label` takes the
  // pending `labelprefix` and clears it (:383-388); breqn keeps the `\label` in `\next@label` until the display
  // is set, the binding labels at once. The layout keys (breaking, spacing, framing, alignment, style) shape the
  // printed display only.
  RawTeX!(
    r"\newcommand{\breqnsetup}[1]{\setkeys{breqn}{#1}}
\define@key{breqn}{label}{\edef\lx@breqn@label{\noexpand\label{\next@label@pre#1}}\lx@breqn@label
  \let\next@label@pre\@empty}
\define@key{breqn}{labelprefix}{\def\next@label@pre{#1}}
\global\let\next@label@pre\@empty
\define@key{breqn}{number}{\tag{#1}}
\define@key{breqn}{shiftnumber}[]{}\define@key{breqn}{holdnumber}[]{}
\define@key{breqn}{density}{}\define@key{breqn}{indentstep}{}\define@key{breqn}{compact}[]{}
\define@key{breqn}{layout}[]{}\define@key{breqn}{spread}{}\define@key{breqn}{sidespace}{}
\define@key{breqn}{style}{}\define@key{breqn}{shortskiplimit}{}\define@key{breqn}{frame}[]{}
\define@key{breqn}{fullframe}[]{}\define@key{breqn}{framesep}[]{}\define@key{breqn}{background}{}
\define@key{breqn}{color}{}\define@key{breqn}{center}[]{}\define@key{breqn}{nocenter}[]{}
\define@key{breqn}{noalign}[]{}\define@key{breqn}{brace}[]{}\define@key{breqn}{breakdepth}{}
\define@key{breqn}{cols}{}"
  );
  // breqn.sty:179-198: the display parameters a document sets (2008.13194 `\eqinterlinepenalty=0`, 1805.03313
  // `\eqlinespacing=0pt plus2pt`).
  RawTeX!(
    r"\newdimen\eqnumsep \eqnumsep=10pt
\newdimen\eqmargin \eqmargin=8pt
\def\eqindent{C}
\newlength\eqlinespacing \eqlinespacing=14pt plus2pt
\newlength\eqlineskip \eqlineskip=3pt plus2pt
\newdimen\eqlineskiplimit \eqlineskiplimit=2pt
\newmuskip\eqbinoffset \eqbinoffset=15mu minus-3mu
\newmuskip\eqdelimoffset \eqdelimoffset=2mu
\newdimen\eqindentstep \eqindentstep=8pt
\newtoks\eqstyle
\newcount\eqbreakdepth \eqbreakdepth=2
\newcount\eqinterlinepenalty \eqinterlinepenalty=10000
\newcount\intereqpenalty \intereqpenalty=1000
\newlength\intereqskip \intereqskip=3pt plus2pt
\newcount\prerelpenalty \prerelpenalty=-\@M
\newcount\prebinoppenalty \prebinoppenalty=888
\newmuskip\Dmedmuskip \Dmedmuskip=4mu minus 3mu
\newmuskip\Dthickmuskip \Dthickmuskip=5mu minus 2mu"
  );
  // breqn.sty:1771-1784 `{dgroup}`: the equations of a group numbered as `(3a)`, `(3b)`, … from the group's
  // number, as amsmath's `{subequations}` numbers them (the subequation equation group); `{dgroup*}` keeps each
  // equation's own number and gives the group none. The options are the breqn keys (:1793 `\setkeys{breqn}{#1}`),
  // a `label` the group's (:1808; 2607.02409 `\begin{dgroup}[label=…]`).
  DefMacro!(
    T_CS!("\\begin{dgroup}"),
    "OptionalUndigested",
    "\\begingroup\\lx@equationgroup@subnumbering@begin\\setkeys{breqn}{#1}"
  );
  DefMacro!(
    T_CS!("\\end{dgroup}"),
    None,
    "\\lx@equationgroup@subnumbering@end\\endgroup"
  );
  DefEnvironment!(
    "{dgroup*} OptionalUndigested",
    "<ltx:equationgroup xml:id='#id'>#body</ltx:equationgroup>",
    properties => { ref_step_id("@equationgroup") }
  );
  // breqn.sty:2027-2041 `{dsuspend}`: text set between the equations of a group (`\intertext`, :2042); it stays
  // a paragraph of the group.
  DefMacro!(T_CS!("\\begin{dsuspend}"), None, "\\begingroup");
  DefMacro!(T_CS!("\\end{dsuspend}"), None, "\\endgroup");
  // breqn.sty:1724-1748 `{dseries}`/`{dseries*}`: one display holding a series of in-line formulas
  // (`\begin{math}…\end{math}` or `$…$`) and the text between them, numbered or not.
  DefEnvironment!(
    "{dseries} OptionalUndigested",
    "<ltx:equation xml:id='#id'>#tags#body</ltx:equation>",
    mode => "restricted_horizontal",
    before_digest => {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true));
      before_equation()?;
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    }
  );
  DefEnvironment!(
    "{dseries*} OptionalUndigested",
    "<ltx:equation xml:id='#id'>#tags#body</ltx:equation>",
    mode => "restricted_horizontal",
    before_digest => {
      prepare_equation_counter(stored_map!("preset" => true));
      before_equation()?;
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    }
  );
  // (`\Biggg`/`\biggg`, breqn.sty:1523-1528, exist only where `\bBigg@` does not, and the amsmath breqn loads defines
  // it: 1907.10288's `\Biggg` is undefined under pdflatex too.)
  // breqn.sty loads mathstyle.sty, whose default option `mathactivechars` (mathstyle.sty:233-236, :250-261) makes `_`
  // and `^` math-active (mathcode "8000, the active forms `\sb`/`\sp`) and, from `\begin{document}`, catcode 12
  // ("other") in text: a bare URL `https://…/boiler_room` prints its `_` (2402.04396, 2403.03720; was an error). breqn's
  // `mathstyleoff` (breqn.sty:42-44) reaches mathstyle as `noactivechars` (flexisym.sty:401-402): catcodes 7 and 8
  // again. Read raw, mathstyle.sty errs ("\over no longer primitive").
  DeclareOption!(
    "mathstyleoff",
    r"\def\lx@breqn@catcodes{\catcode`\^=7\relax \catcode`\_=8\relax}"
  );
  ProcessOptions!();
  RawTeX!(
    r#"\mathcode`\_="8000 \mathcode`\^="8000
\begingroup\catcode`\_=\active\catcode`\^=\active \global\let_=\sb \global\let^=\sp \endgroup
\providecommand\lx@breqn@catcodes{\catcode`\^=12\relax \catcode`\_=12\relax}
\AtBeginDocument{\lx@breqn@catcodes}"#
  );
});
