use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("ifthen");
  RequirePackage!("eso-pic");
  RequirePackage!("fancyheadings");
  // Perl-parity stubs: every \svn* CS is a `Tokens()` no-op in
  // ar5iv-bindings/svninfo.sty.ltxml L23-44 (matching upstream SVN
  // keyword placeholders that have no print-time equivalent in LaTeXML).
  DefMacro!("\\svnInfo SkipSpaces Match:$ Until:$", "");
  DefMacro!("\\svnKeyword SkipSpaces Match:$ Until:$", "");
  DefMacro!("\\svnId", "");
  DefMacro!("\\svnInfoDate", "");
  DefMacro!("\\svnInfoDay", "");
  DefMacro!("\\svnInfoFile", "");
  DefMacro!("\\svnInfoHeadURL", "");
  DefMacro!("\\svnInfoLongDate", "");
  DefMacro!("\\svnInfoMaxDay", "");
  DefMacro!("\\svnInfoMaxMonth", "");
  DefMacro!("\\svnInfoMaxRevision", "");
  DefMacro!("\\svnInfoMaxToday", "");
  DefMacro!("\\svnInfoMaxYear", "");
  DefMacro!("\\svnInfoMinRevision", "");
  DefMacro!("\\svnInfoMonth", "");
  DefMacro!("\\svnInfoOwner", "");
  DefMacro!("\\svnInfoRevision", "");
  DefMacro!("\\svnInfoTime", "");
  DefMacro!("\\svnInfoYear", "");
  DefMacro!("\\svnKeywordempty", "\\relax");
  DefMacro!("\\svnMaxToday", "");
  DefMacro!("\\svnToday", "");
});
