use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svjour.cls.ltxml — covers svjour.cls, svjour1.cls, svjour2.cls & svjour3.cls

  // Option handling — declare all known options as no-ops
  for option in [
    "onecollarge", "runningheads", "smartrunhead", "nosmartrunhead",
    "referee", "instindent", "smartand", "nospthms", "deutsch", "francais",
    // numbering options
    "numbook", "envcountreset", "envcountsame", "envcountsect",
    // natbib
    "natbib",
    // journal style options
    "2epj", "epj", "glov2arxiv", "manmat-mod", "multphys", "aar",
    "epj-mine", "glov2", "matann2", "nummat",
    "ampa", "epjmod", "granma", "matann", "own", "arma",
    "epj_orig", "icps3", "matbio", "probth",
    "ceremade", "epj-spec", "ifip", "matprg", "publmath",
    "cmp", "gc10", "ijodl", "matzei", "tcfd",
    "cmt", "global", "invmat", "multph",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Other options get passed to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequireResource!("ltx-svjour.css");
  RequirePackage!("sv_support");
});
