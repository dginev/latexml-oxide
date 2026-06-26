use crate::prelude::*;

/// Perl `dirtytalk.sty.ltxml` `setDirtytalkSymbol` (#2806): register a `dirtytalk`
/// keyval whose `code` redefines the quote-symbol macro `symbol` to the keyval's
/// (undigested) value — but only when that value is non-empty (Perl
/// `return if IsEmpty($tokens)`). Mirrors
/// `DefKeyVal('dirtytalk', $key, 'UndigestedKey', '', code => sub { … })`.
fn def_dirtytalk_symbol_key(key: &str, symbol: &'static str) -> Result<()> {
  keyval::define(KeyvalConfig {
    prefix: "KV",
    keyset: "dirtytalk",
    key,
    vtype: "", // Perl 'UndigestedKey' → a plain `{}` undigested value
    default: Some(""),
    code: Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        if let Some(value) = args.into_iter().next() {
          let tokens = value.revert()?;
          if !tokens.is_empty() {
            def_macro(T_CS!(symbol), None, ExpansionBody::Tokens(tokens), None)?;
          }
        }
        Ok(Tokens!())
      },
    ))),
    ..KeyvalConfig::default()
  })
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl dirtytalk.sty.ltxml (#2806, 51fea96a). `\say{…}` produces context-aware
  // quotation marks via a nesting-depth counter: the outer level uses double
  // quotes, a nested `\say` uses single quotes.
  DefMacro!("\\dirtytalk@lqq", "\\textquotedblleft");
  DefMacro!("\\dirtytalk@rqq", "\\textquotedblright");
  DefMacro!("\\dirtytalk@lq",  "\\textquoteleft");
  DefMacro!("\\dirtytalk@rq",  "\\textquoteright");

  // Package options let the author override each symbol (UndigestedKey + code).
  def_dirtytalk_symbol_key("left",     "\\dirtytalk@lqq")?;
  def_dirtytalk_symbol_key("right",    "\\dirtytalk@rqq")?;
  def_dirtytalk_symbol_key("leftsub",  "\\dirtytalk@lq")?;
  def_dirtytalk_symbol_key("rightsub", "\\dirtytalk@rq")?;

  ProcessOptions!(keysets => ["dirtytalk"]);

  RawTeX!(r"\newcounter{dirtytalk@qdepth}
\newcommand{\dirtytalk@lsymb}{%
  \ifnum\value{dirtytalk@qdepth}>1
    \dirtytalk@lq
  \else
    \dirtytalk@lqq
  \fi}
\newcommand{\dirtytalk@rsymb}{%
  \ifnum\value{dirtytalk@qdepth}>1
    \dirtytalk@rq
  \else
    \dirtytalk@rqq
  \fi}
\providecommand{\say}[1]{%
  \addtocounter{dirtytalk@qdepth}{1}%
  \dirtytalk@lsymb #1\dirtytalk@rsymb%
  \addtocounter{dirtytalk@qdepth}{-1}}");
});
