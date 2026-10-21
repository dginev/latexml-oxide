//! vmsta.cls (Modern Stochastics: Theory and Applications, VTeX): the papers ship the class (it is not in TeX Live) and
//! it is read raw, with the author name markup its title page prints. A paper whose source lacks the class falls to
//! OmniBus, with the loader's missing-file Warning.
use latexml_package::prelude::*;

const VMSTA: crate::shipped_class::RawRead = crate::shipped_class::RawRead {
  shipped: true,
  // vmsta.cls:278-291 defines the author's name parts only where its `\@@author` prints the author list: `\name@m@front`
  // stores `\fnm{…}` and `\snm{…}`, drops `\inits` and `\suffix`, and prints `\@fnm~\@snm` after them — "Yuliya
  // Mishura" for `\author[a]{\inits{Yu.}\fnm{Yuliya}\snm{Mishura}\corref{cor1}}` (the initials serve its e-mail line,
  // `\name@m@ead`, "(Yu. Mishura)"). The kernel's `\author` replaces the class's, so the author content takes them as
  // its title page does (`\lx@author@markup@class`, for the author content only; was "Yu.YuliyaMishura", the
  // class's `\name@m` identities that #647 found and OmniBus's `\inits`): 71 papers, 1508.02842, 1411.4499, 1601.01109.
  // A first name no `\snm` follows is printed after the content, as `\@fnm~\@snm` prints it there (`\fnm{Ivan}~Voronov`
  // "VoronovIvan", where vmsta.cls errs on its undefined `\@snm`; the rev77t review's finding 7): flushed with the
  // markup's restore list, which runs after the author content.
  after: r"\let\lx@vmsta@fnm\@empty
\def\lx@vmsta@first#1{\def\lx@vmsta@fnm{#1}}
\def\lx@vmsta@sur#1{\ifx\lx@vmsta@fnm\@empty\else\lx@vmsta@fnm~\fi#1\let\lx@vmsta@fnm\@empty}
\def\lx@vmsta@flush{\lx@vmsta@fnm\let\lx@vmsta@fnm\@empty}
\def\lx@author@markup@class{\lx@author@markup@provide\inits\@gobble\lx@author@markup@provide\suffix\@gobble\lx@author@markup@provide\fnm\lx@vmsta@first\lx@author@markup@provide\snm\lx@vmsta@sur\expandafter\def\expandafter\lx@author@markup@unprovide\expandafter{\expandafter\lx@vmsta@flush\lx@author@markup@unprovide}}",
  ..crate::shipped_class::RawRead::new("vmsta", "cls")
};

LoadDefinitions!({
  if !crate::shipped_class::read_raw(&VMSTA)? {
    return Ok(());
  }
  // The e-mails and addresses are the authors' contacts. The class prints them only from its title page: an e-mail
  // reaches it through the `.aux` file, for a later run (vmsta.cls:242-256 `\@email` writes `\emailauthor{…}` to the
  // `.aux`; the 2018+ copies' `\ead`, :1272-1297, adds it to `\email@author@toks`), and an address is appended to
  // `\@@@address` (:381-390; the 2018+ copies' `\address@optarg`/`\address@noptarg`, :877-920, to `\address@list`).
  // Read raw, the e-mails were lost (`myus@univ.kiev.ua` in none of 1508.02842's output) and the address was an
  // italic paragraph of the replayed title page. An `\address[a]` is the affiliation of the authors labelled `a`
  // (`\author[a,b]{…}`, 1801.00977, 1411.4499, 2001.02487), one without a label of the authors before it with none
  // (1508.02842, 1601.01109, 1912.07948); an e-mail is the contact of the author it follows (the 2014-2018 copies:
  // `\author{…}\email{…}`) or is inside (the 2018+ copies: `\author[a]{…\ead[label=e1]{…}}`). So `\author` makes
  // the creator with its labels as annotations, after the class's own `\author` (refused by the kernel's lock),
  // replayed for the counters and lists its `{frontmatter}` checks and title page read (vmsta.cls:753-792; a class
  // error "`\articletype` is missing" without them, 1805.02511).
  DefMacro!(
    "\\author[]{}",
    "\\lx@replay@class@setter{author}{#1}{#2}\\lx@add@authors@append[annotations={#1}]{#2}"
  );
  if IsDefined!(&T_CS!("\\emailauthor")) {
    RawTeX!(
      r"\def\@email#1{\lx@add@email{#1}}
\long\def\@@address[#1]#2{\lx@add@affiliation[label={#1}]{#2}}
\long\def\@address#1{\lx@add@affiliation[annotate=run]{#1}}"
    );
  }
  if IsDefined!(&T_CS!("\\address@optarg")) {
    RawTeX!(
      r"\def\address@optarg[#1]#2{\lx@add@affiliation[label={#1}]{#2}\ignorespaces}
\def\address@noptarg#1{\lx@add@affiliation[annotate=run]{#1}\ignorespaces}"
    );
    // (where the class's code runs `\ead` — its `\author`, replayed for its counters and lists, and its title page,
    // deposited — it counts the address, as the class's `\ead@hook` does, :1264-1265, for the check at
    // `\end{frontmatter}`, "At least one email address is required!" (:561-563; the copies that have it, not
    // 1810.09681's): the e-mail is the frontmatter's once, not again in the title page's e-mail line, whose block the
    // title page still opens, `\footnotesize` for the history line after it, :385-397, which it set bold without)
    DefMacro!("\\ead[]{}", sub[(_opt, address)] {
      let class_code = ["lx_replaying_class_setter", "lx_depositing_class_maketitle", "lx_depositing_frontmatter_fields"]
        .iter()
        .any(|flag| lookup_bool(flag));
      Ok(if class_code {
        TokenizeInternal!(
          r"\ifdefined\c@countead\stepcounter{countead}\fi\ifdefined\email@author@toks\global\email@author@toks{\relax}\fi"
        )
      } else {
        Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(address)])
      })
    });
  }
});
