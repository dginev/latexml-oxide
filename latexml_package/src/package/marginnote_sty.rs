use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: marginnote.sty.ltxml
  DefConditional!("\\if@mn@verbose");

  // not documented, but in the implementation
  DeclareOption!("quiet", {
    Let!("\\if@mn@verbose", "\\iffalse");
  });
  DeclareOption!("verbose", {
    Let!("\\if@mn@verbose", "\\iftrue");
  });

  DeclareOption!("parboxrestore", {
    DefMacro!("\\mn@parboxrestore", "\\@parboxrestore");
  });
  DeclareOption!("noparboxrestore", {
    def_macro_noop("\\mn@parboxrestore")?;
  });

  for option in ["fulladjust", "heightadjust", "depthadjust", "noadjust"] {
    DeclareOption!(option, None);
  }
  Digest!("\\ExecuteOptions{verbose,fulladjust,parboxrestore}")?;
  ProcessOptions!();

  DefMacro!("\\marginfont", "\\normalcolor");
  DefMacro!("\\raggedleftmarginnote", "\\raggedleft");
  DefMacro!("\\raggedrightmarginnote", "\\raggedright");

  // marginnote.sty:319-343: `\marginnote` = `\@dblarg\@mn@marginnote`, then
  // `\@mn@marginnote[#1]#2` → `\@ifnextchar[` → `\@mn@@marginnote[#1]#2[#3]`
  // → `\@mn@@@marginnote[#1]#2[#3]`, i.e. the note body rides through three
  // macro-argument layers before it is set. Perl marginnote.sty.ltxml:37-40
  // (and the former port) expanded `\marginnote` straight to `\marginpar`:
  // one layer short, so a body calibrated for the real depth —
  // skdoc.cls:631 `\marginnote{\clist_map_inline:Nn…{\index@option*{####1}}}`
  // — left a literal `#1` (`misdefined:#` ×48) and defined glossary entries
  // under the leaked key (`Glossary entry 'index-1-opt'…` ×96; iodhbwm
  // 146 errors, KPE #166). The terminal macro sets the note as `\marginpar`
  // (the `[left]` text only when it differs from the right one — `\@dblarg`
  // copies the body into `#1`); the page-position machinery is skipped.
  // Guard: `perfect_kernel_batch54::marginnote_body_rides_three_argument_layers`.
  RawTeX!(r"\newcommand*{\marginnote}{\@dblarg\@mn@marginnote}
\newcommand{\@mn@marginnote}[2][]{\begingroup\@ifnextchar[{\@mn@@marginnote[{#1}]{#2}}{\@mn@@marginnote[{#1}]{#2}[\z@]}}
\newcommand{\@mn@@marginnote}{}
\long\def\@mn@@marginnote[#1]#2[#3]{\endgroup\@mn@@@marginnote[{#1}]{#2}[{#3}]}
\newcommand{\@mn@@@marginnote}{}
\long\def\@mn@@@marginnote[#1]#2[#3]{\def\mn@tempa{#1}\def\mn@tempb{#2}\ifx\mn@tempa\mn@tempb\marginpar{\mn@parboxrestore\marginfont\raggedrightmarginnote #2}\else\marginpar[\mn@parboxrestore\marginfont\raggedleftmarginnote #1]{\mn@parboxrestore\marginfont\raggedrightmarginnote #2}\fi}");

  // Perl marginnote.sty.ltxml L42-46: \@mn@if@RTL dispatches at
  // expansion time — if \if@RTL is defined (LookupValue) AND currently
  // true (IfCondition), return \@firstoftwo; otherwise \@secondoftwo.
  DefMacro!("\\@mn@if@RTL", sub[_args] {
    let rtl_cs = T_CS!("\\if@RTL");
    let is_rtl = lookup_definition(&rtl_cs)?.is_some()
      && if_condition(&rtl_cs)?.unwrap_or(false);
    Ok(Tokens!(if is_rtl { T_CS!("\\@firstoftwo") } else { T_CS!("\\@secondoftwo") }))
  });

  // stubs that could do something but do not
  DefRegister!("\\marginnotevadjust" => Dimension!("0pt"));
  // Note: Perl uses LookupRegister('\textwidth') but we use 0pt as a safe default
  DefRegister!("\\marginnotetextwidth" => Dimension!("0pt"));
  Let!("\\newmarginnote", "\\newlabel");
  Let!("\\mn@lastxpos", "\\lastxpos");
  Let!("\\mn@savepos", "\\savepos");
  Let!("\\mn@pagewidth", "\\pagewidth");
  Let!("\\mn@strut", "\\strut");
  Let!("\\mn@vadjust", "\\vadjust");

  // stubs that do nothing
  def_macro_noop("\\@mn@margintest")?;
  def_macro_noop("\\@mn@thispage")?;
  def_macro_noop("\\@mn@atthispage")?;
  def_macro_noop("\\@mn@currpage")?;
  def_macro_noop("\\@mn@currxpos")?;
  def_macro_noop("\\mn@vlap {}")?;
  def_macro_noop("\\mn@zbox {}")?;

  NewCounter!("mn@abspage");
});
