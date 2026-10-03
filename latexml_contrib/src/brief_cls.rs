//! brief.cls (ntgclass, Dutch business letters): the class is loaded raw, and the letter's sender — which the class
//! prints only in its first-page head and foot (`\ps@firstpage`, brief.cls:285-294: `\briefhoofd` from
//! `\maakbriefhoofd{<name>}{<address>}`, :437-458, and the `\voetitem{<label>}{<value>}` foot, :459-468), a page style
//! LaTeXML never typesets — becomes the letter's frontmatter at the class's own print site, the start of `{brief}`
//! (:321): a `creator` of role `sender` named by the head's name, with the address and each foot item as a `contact`
//! (the item's label as its `name`). As the g-brief bindings (OXIDIZED_DESIGN_DIVERGENCES #412, user ruling 2026-10-01:
//! a letter's sender is frontmatter, the rest of the page furniture). SHARED: Perl with raw classes drops it alike. Only
//! a document's first letter is given (one frontmatter a document). Witness ntgclass/brief-sample. Guard
//! `perfect_kernel_batch61::brief_letter_sender_is_frontmatter`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("brief", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // A foot label may break its line (`\voetitem{telefoon \\ priv\'e:}`, brief-sample.tex:16); a contact's name is
  // one line, so the break is a space there.
  DefMacro!("\\lx@brief@label{}", sub[(label)] {
    let mut out: Vec<Token> = Vec::new();
    for t in label.unlist() {
      let t = if t == T_CS!("\\\\") { T_SPACE!() } else { t };
      let space = t.get_catcode() == Catcode::SPACE;
      if space && out.last().is_some_and(|l| l.get_catcode() == Catcode::SPACE) {
        continue;
      }
      out.push(t);
    }
    Ok(Tokens::new(out))
  });
  RawTeX!(
    r"\let\lx@brief@maak\@maakbriefhoofd
\def\@maakbriefhoofd#1#2{\lx@brief@maak{#1}{#2}\gdef\lx@brief@name{#1}\gdef\lx@brief@address{#2}}
\gdef\lx@brief@contacts{}
\let\lx@brief@voetitem\voetitem
\def\voetitem#1#2{\lx@brief@voetitem{#1}{#2}\g@addto@macro\lx@brief@contacts{\lx@add@contact[name={\lx@brief@label{#1}}]{#2}}}
\let\footitem\voetitem
\def\lx@brief@frontmatter{\global\let\lx@brief@frontmatter\relax
  \ifx\lx@brief@name\@undefined\else
    \lx@add@creator[role=sender]{\lx@brief@name}%
    \lx@add@contact[role=address]{\lx@brief@address}\fi
  \lx@brief@contacts}
\let\lx@brief@begin\brief
\def\brief{\lx@brief@frontmatter\lx@brief@begin}
\def\lx@contact@address@name{}"
  );
});
