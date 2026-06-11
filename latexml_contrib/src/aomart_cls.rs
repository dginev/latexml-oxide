//! Stub for aomart.cls (Annals of Mathematics).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");

  // Author metadata (aomart.cls L222+) — preserve as ltx:note
  // frontmatter so author-supplied values reach the XML output.
  // Name parts emit inline.
  DefMacro!("\\givenname{}", "#1");
  DefMacro!("\\surname{}", "#1");
  DefMacro!(
    "\\subject{}{}{}",
    "\\@add@frontmatter{ltx:classification}[scheme=#1]{#3}"
  );
  DefMacro!(
    "\\published{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}"
  );
  DefMacro!(
    "\\publishedonline{}",
    "\\@add@frontmatter{ltx:note}[role=published-online]{#1}"
  );
  DefMacro!(
    "\\publicationyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\volumenumber{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!(
    "\\issuenumber{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}"
  );
  DefMacro!(
    "\\papernumber{}",
    "\\@add@frontmatter{ltx:note}[role=papernumber]{#1}"
  );
  DefMacro!(
    "\\startpage{}",
    "\\@add@frontmatter{ltx:note}[role=startpage]{#1}"
  );
  DefMacro!(
    "\\endpage{}",
    "\\@add@frontmatter{ltx:note}[role=endpage]{#1}"
  );
  DefMacro!(
    "\\doinumber{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}"
  );
  DefMacro!("\\mrnumber{}", "\\@add@frontmatter{ltx:note}[role=mr]{#1}");
  DefMacro!(
    "\\zblnumber{}",
    "\\@add@frontmatter{ltx:note}[role=zbl]{#1}"
  );
  DefMacro!(
    "\\arxivnumber{}",
    "\\@add@frontmatter{ltx:note}[role=arxiv]{#1}"
  );
  DefMacro!(
    "\\version{}",
    "\\@add@frontmatter{ltx:note}[role=version]{#1}"
  );
  DefMacro!(
    "\\copyrightnote{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  DefMacro!("\\formatdate{}", "#1");
});
