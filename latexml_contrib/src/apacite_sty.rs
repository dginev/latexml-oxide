//! Stub for apacite.sty (APA citation/bibliography style).
//!
//! Apacite defines a large family of APAref* macros driven by its .bbl
//! generator. We don't implement APA formatting; pass-through the
//! content-bearing args (authors, titles) and gobble the rest.
use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
/// Routes inline macro expansion (each ~960 B of .text) through one
/// runtime call. Engine bootstrap pays parse_prototype once per entry.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


/// DEP-19 helper for identity-1 `DefMacro!("\\cs{}", "#1")` macros — the
/// CS takes one mandatory arg and expands to it unchanged. Routes
/// inline macro expansion through a single runtime call.
fn def_macro_identity(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("#1");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  RequirePackage!("natbib");

  // Core APAref* set (apacite.sty L1257-2243). Render as the
  // content-bearing argument so titles / authors survive in the XML.
  def_macro_noop("\\APACinsertmetastar{}")?;
  DefMacro!("\\APACrefatitle{}{}", "#2");
  DefMacro!("\\APACrefbtitle{}{}", "#2");
  DefMacro!("\\APACrefYear{}", "(#1)");
  DefMacro!("\\APACrefYearMonthDay{}{}{}", "(#1)");
  DefMacro!("\\APACjournalVolNumPages{}{}{}{}", "#1 #2 #3 #4");
  def_macro_identity("\\APAChowpublished{}")?;
  DefMacro!("\\APACaddressPublisher{}{}", "#1: #2");
  DefMacro!("\\APACaddressInstitution{}{}", "#1: #2");
  DefMacro!("\\APACexlab{}{}", "#2");
  // \APACmonth{name} — month text (was gobbled). Pass through inline.
  def_macro_identity("\\APACmonth{}")?;
  def_macro_identity("\\APACrefnote{}")?;
  DefMacro!("\\APAhyperref{}{}", "#2");
  def_macro_noop("\\PrintBackRefs{}")?;
  def_macro_noop("\\CurrentBib")?;
  def_macro_noop("\\bibcomputersoftwaremanual{}{}{}")?;

  // APAref* environments
  DefEnvironment!("{APACrefauthors}", "#body");
  DefEnvironment!("{APACrefURL}", "#body");
  DefEnvironment!("{APACrefDOI}", "#body");

  // Additional APAC* macros (apacite ships many).
  DefMacro!("\\APACyear{}", "(#1)");
  DefMacro!("\\APACciteatitle{}{}", "#2");
  DefMacro!("\\APACcitebtitle{}{}", "#2");
  DefMacro!("\\APACrefaetitle{}{}", "#2");
  DefMacro!("\\APACrefbetitle{}{}", "#2");
  DefMacro!("\\APACbVolEdTR{}{}", "#2");
  DefMacro!("\\APACbVolEdTRpgs{}{}{}", "#3");
  DefMacro!("\\APACaddressInstitutionEqAuth{}{}", "#1: #2");
  DefMacro!("\\APACaddressPublisherEqAuth{}{}", "#1: #2");
  DefMacro!("\\APACaddressSchool{}{}", "#1: #2");
  DefMacro!("\\APACtypeAddressSchool{}{}{}", "#3");
  def_macro_noop("\\APACmetastar")?;
  DefMacro!("\\APACorigyearnote{}", "(#1)");
  def_macro_identity("\\APACorigjournalnote{}")?;
  def_macro_identity("\\APACorigbooknote{}")?;
  DefMacro!("\\APACorigED", "Ed.");
  DefMacro!("\\APACorigEDS", "Eds.");
  def_macro_identity("\\APACstd{}")?;
  def_macro_noop("\\APACSortNoop{}")?;
  def_macro_noop("\\APACmetaprenote")?;
  def_macro_noop("\\APACrefauthstyle{}")?;
  def_macro_noop("\\APACbibcite{}")?;

  // apacite citation forms (apacite.sty L328+). Delegate to natbib's
  // \cite which we wrapped in natbib_sty.rs. Forms:
  //   \citeA[pre][post]{key} — author-only ("Smith")
  //   \citeauthor[pre][post]{key} — author-only (alternate spelling)
  //   \citeNP[pre][post]{key} — citation without parens
  //   \citeyearNP[pre][post]{key} — year-only without parens
  // Witness 2407.14158, 2407.18402, 2407.16770 (apacite-using papers).
  DefMacro!("\\citeA[][] Semiverbatim", "\\citet[#1][#2]{#3}");
  DefMacro!("\\citeNP[][] Semiverbatim", "\\citealp[#1][#2]{#3}");
  DefMacro!("\\citeyearNP[][] Semiverbatim", "\\citeyear[#1][#2]{#3}");
  def_macro_noop("\\APACrestorebibitem")?;
  def_macro_noop("\\APACemindex{}")?;
  def_macro_noop("\\APACltxemindex{}")?;
  def_macro_noop("\\APACtocindex{}")?;
  def_macro_noop("\\APACstdindex{}")?;
  def_macro_noop("\\APACurlBreaks")?;

  // Short-form helpers (apacite L1300+: \BBA, \BCnt, \BPGS, etc.)
  DefMacro!("\\BBA", "&");
  def_macro_noop("\\BBCQ")?;
  def_macro_noop("\\BBOQ")?;
  DefMacro!("\\BPBI", ".");
  DefMacro!("\\BHBI", "-");
  def_macro_noop("\\BDBL")?;
  def_macro_noop("\\BCBT")?;
  def_macro_noop("\\BCBL")?;
  def_macro_identity("\\BCnt{}")?;
  def_macro_identity("\\BPGS{}")?;
  def_macro_identity("\\BVOL{}")?;
  DefMacro!("\\BOthers{}", "et al.");
  DefMacro!("\\BEDS", "Eds.");
  DefMacro!("\\BIn", "In");
});
