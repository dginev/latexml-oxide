//! svn-multi.sty — Subversion keyword macros. There is no Perl binding (Perl reports the
//! package's macros undefined). The stub this replaced read the package wrongly: TeX
//! conditionals for `\ifsvnmodified`/`\ifsvnfilemodified`, which are `\@secondoftwo`-style
//! two-argument choosers (svn-multi.sty:256, :277), an argument for the storage macro `\svnurl`
//! (:269), and no-op `\svnid`/`\svnidlong`, so a document's revision line took `Error:expected:\fi`
//! and its keyword `$…$` groups became math. Loaded raw, the real package converts cleanly with
//! pdflatex's first-pass text; the document keywords come back from the .aux on pass 2, which a
//! single-pass conversion does not read (K13 stage-2 audit row svn-multi, 57h; repro
//! singletons/svn_multi_loads_raw).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("svn-multi", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
