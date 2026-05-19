use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


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

  // Perl marginnote.sty.ltxml L37-40: \marginnote[left]{right}[vshift]
  // expands to \marginpar — with the left-margin text (#1) included as the
  // optional [left] argument when present, and the vshift (#3) ignored.
  // Prior Rust stub dropped #1 entirely; this port preserves the left branch.
  DefMacro!("\\marginnote []{}[]", sub[(left, right, _vadjust)] {
    let mut out: Vec<Token> = Vec::new();
    out.push(T_CS!("\\marginpar"));
    if let Some(l) = left {
      // [\mn@parboxrestore\marginfont\raggedleftmarginnote <left>]
      out.push(T_OTHER!("["));
      out.push(T_CS!("\\mn@parboxrestore"));
      out.push(T_CS!("\\marginfont"));
      out.push(T_CS!("\\raggedleftmarginnote"));
      out.push(T_SPACE!());
      out.extend(l.unlist());
      out.push(T_OTHER!("]"));
    }
    // {\mn@parboxrestore\marginfont\raggedrightmarginnote <right>}
    out.push(T_BEGIN!());
    out.push(T_CS!("\\mn@parboxrestore"));
    out.push(T_CS!("\\marginfont"));
    out.push(T_CS!("\\raggedrightmarginnote"));
    out.push(T_SPACE!());
    out.extend(right.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });

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
  def_macro_noop("\\@mn@marginnote []{}")?;
  def_macro_noop("\\@mn@@marginnote []{}[]")?;
  def_macro_noop("\\@mn@@@marginnote []{}[]")?;
  def_macro_noop("\\@mn@margintest")?;
  def_macro_noop("\\@mn@thispage")?;
  def_macro_noop("\\@mn@atthispage")?;
  def_macro_noop("\\@mn@currpage")?;
  def_macro_noop("\\@mn@currxpos")?;
  def_macro_noop("\\mn@vlap {}")?;
  def_macro_noop("\\mn@zbox {}")?;

  NewCounter!("mn@abspage");
});
