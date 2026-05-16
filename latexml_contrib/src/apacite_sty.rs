//! Stub for apacite.sty (APA citation/bibliography style).
//!
//! Apacite defines a large family of APAref* macros driven by its .bbl
//! generator. We don't implement APA formatting; pass-through the
//! content-bearing args (authors, titles) and gobble the rest.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");

  // Core APAref* set (apacite.sty L1257-2243). Render as the
  // content-bearing argument so titles / authors survive in the XML.
  DefMacro!("\\APACinsertmetastar{}", "");
  DefMacro!("\\APACrefatitle{}{}", "#2");
  DefMacro!("\\APACrefbtitle{}{}", "#2");
  DefMacro!("\\APACrefYear{}", "(#1)");
  DefMacro!("\\APACrefYearMonthDay{}{}{}", "(#1)");
  DefMacro!("\\APACjournalVolNumPages{}{}{}{}", "#1 #2 #3 #4");
  DefMacro!("\\APAChowpublished{}", "#1");
  DefMacro!("\\APACaddressPublisher{}{}", "#1: #2");
  DefMacro!("\\APACaddressInstitution{}{}", "#1: #2");
  DefMacro!("\\APACexlab{}{}", "#2");
  DefMacro!("\\APACmonth{}", "");
  DefMacro!("\\APACrefnote{}", "#1");
  DefMacro!("\\APAhyperref{}{}", "#2");
  DefMacro!("\\PrintBackRefs{}", "");
  DefMacro!("\\CurrentBib", "");
  DefMacro!("\\bibcomputersoftwaremanual{}{}{}", "");

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
  DefMacro!("\\APACmetastar", "");
  DefMacro!("\\APACorigyearnote{}", "(#1)");
  DefMacro!("\\APACorigjournalnote{}", "#1");
  DefMacro!("\\APACorigbooknote{}", "#1");
  DefMacro!("\\APACorigED", "Ed.");
  DefMacro!("\\APACorigEDS", "Eds.");
  DefMacro!("\\APACstd{}", "#1");
  DefMacro!("\\APACSortNoop{}", "");
  DefMacro!("\\APACmetaprenote", "");
  DefMacro!("\\APACrefauthstyle{}", "");
  DefMacro!("\\APACbibcite{}", "");
  DefMacro!("\\APACrestorebibitem", "");
  DefMacro!("\\APACemindex{}", "");
  DefMacro!("\\APACltxemindex{}", "");
  DefMacro!("\\APACtocindex{}", "");
  DefMacro!("\\APACstdindex{}", "");
  DefMacro!("\\APACurlBreaks", "");

  // Short-form helpers (apacite L1300+: \BBA, \BCnt, \BPGS, etc.)
  DefMacro!("\\BBA", "&");
  DefMacro!("\\BBCQ", "");
  DefMacro!("\\BBOQ", "");
  DefMacro!("\\BPBI", ".");
  DefMacro!("\\BHBI", "-");
  DefMacro!("\\BDBL", "");
  DefMacro!("\\BCBT", "");
  DefMacro!("\\BCBL", "");
  DefMacro!("\\BCnt{}", "#1");
  DefMacro!("\\BPGS{}", "#1");
  DefMacro!("\\BVOL{}", "#1");
  DefMacro!("\\BOthers{}", "et al.");
  DefMacro!("\\BEDS", "Eds.");
  DefMacro!("\\BIn", "In");
});
