use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "svn-multi.sty",
    "svn-multi.sty is not implemented and will not be interpreted raw."
  );
  RequirePackage!("graphics");
  DefConditional!("\\ifsvnfilemodified");
  DefConditional!("\\ifsvnmodified");
  DefConditional!("\\ifsvnsubgroups");
  DefMacro!("\\svnFullAuthor OptionalMatch:* {}", "");
  DefMacro!("\\svnRegisterAuthor{}{}", "");
  DefMacro!("\\svnRegisterRevision OptionalMatch:* {}{}", "");
  DefMacro!("\\svncgtime", "");
  DefMacro!("\\svncgtimezone", "");
  DefMacro!("\\svncgtoday", "");
  DefMacro!("\\svndate", "");
  DefMacro!("\\svnday", "");
  DefMacro!("\\svnfileauthor", "");
  DefMacro!("\\svnfiledate", "");
  DefMacro!("\\svnfileday", "");
  DefMacro!("\\svnfiledir", "");
  DefMacro!("\\svnfilefname", "");
  DefMacro!("\\svnfilehour", "");
  DefMacro!("\\svnfileminute", "");
  DefMacro!("\\svnfilemonth", "");
  DefMacro!("\\svnfilerev", "");
  DefMacro!("\\svnfilesecond", "");
  DefMacro!("\\svnfiletime", "");
  DefMacro!("\\svnfiletimezone", "");
  DefMacro!("\\svnfiletimezonehour", "");
  DefMacro!("\\svnfiletimezoneminute", "");
  DefMacro!("\\svnfiletoday", "");
  DefMacro!("\\svnfileyear", "");
  DefMacro!("\\svnhour", "");
  DefMacro!("\\svnid{}", "");
  DefMacro!("\\svnidlong", "");
  DefMacro!("\\svnminute", "");
  DefMacro!("\\svnnolinkurl", "#1");
  DefMacro!("\\svnsecond", "");
  DefMacro!("\\svntime", "");
  DefMacro!("\\svntimezone", "");
  DefMacro!("\\svntimezonehour", "");
  DefMacro!("\\svntimezoneminute", "");
  DefMacro!("\\svntoday", "");
  DefMacro!("\\svnurl{}", "");
  DefMacro!("\\svnyear", "");
  DefMacro!("\\tableofrevisions", "");
  DefEnvironment!("{svnfilerow}", "");
  DefEnvironment!("{svnglobalrow}", "");
  DefEnvironment!("{svngrouprow}", "");
  DefEnvironment!("{svnsubgrouprow}", "");
  DefEnvironment!("{svntable}", "");
});
