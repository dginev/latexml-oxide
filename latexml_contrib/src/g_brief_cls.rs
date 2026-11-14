//! g-brief.cls (German business letters, from g-brief.dtx): the class is loaded raw, and the letter's sender and
//! addressee — which the class prints only in its first-page head and foot (`\ps@firstpage`, g-brief.cls:306-360),
//! a page style LaTeXML never typesets (`\thispagestyle` is a no-op, Perl latex_constructs.pool.ltxml:998, Rust
//! latex_constructs/sect05.rs) — become the letter's frontmatter at the class's own print site, the start of
//! `{g-brief}` (g-brief.cls:250-251 `\thispagestyle{firstpage}`): a `creator` of role `sender` (the name, then a
//! `contact` per block the foot prints: address, phone, fax, telex, e-mail, web, bank) and one of role `addressee`
//! (the postal note and the address). The rest of the first page — fold, punch and window marks, rules — stays
//! dropped as page furniture; the letter's sender and recipient are its frontmatter (user ruling 2026-10-01).
//! A binding, justified as the README's exceptions are: raw interpretation cannot reach a page style that is never
//! typeset, and typesetting it would draw the `picture` layout the furniture ruling drops. Without it both engines
//! lose every field (SHARED: Perl with raw classes, 0 errors, the same drop). The fields' values are taken at the
//! letter; only a serial document's first letter is given (one frontmatter a document), and fields set inside
//! `{g-brief}` are missed. OXIDIZED_DESIGN_DIVERGENCES #412.
//! Witnesses g-brief/beispiel (g-brief.cls), g-brief/beispiel2 and cv/ApplicationLetter (g-brief2.cls).
use latexml_package::prelude::*;

/// The value of the store `\<name>` (a parameterless macro, `\def\name{#1}`, g-brief.cls:192-211): its expansion,
/// read when the letter starts — the frontmatter queue digests later, after a later letter's setters
/// (`\Name`, `\Adresse`) may have replaced it.
fn store_value(name: &str) -> Result<Option<Tokens>> {
  Ok(match lookup_definition(&T_CS!(s!("\\{name}")))? {
    Some(defn) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
      _ => None,
    },
    None => None,
  })
}

/// A store that prints nothing: only spaces and `\empty`s. The binding's rule; the classes test `\ifx\x\empty`
/// (g-brief.cls) or `\equal{\x}{\empty}` (g-brief2.cls).
fn is_blank(value: &Tokens) -> bool {
  value
    .unlist_ref()
    .iter()
    .all(|t| t.get_catcode() == Catcode::SPACE || *t == T_CS!("\\empty"))
}

/// The set values among `stores` (a comma list of store names).
fn set_values(stores: &Tokens) -> Result<Vec<Tokens>> {
  let mut values = Vec::new();
  for name in stores
    .to_string()
    .split(',')
    .map(str::trim)
    .filter(|n| !n.is_empty())
  {
    if let Some(value) = store_value(name)?.filter(|v| !is_blank(v)) {
      values.push(value);
    }
  }
  Ok(values)
}

/// `\lx@gbrief@contact{role}{label}{store,…}` and `\lx@gbrief@sender{name store}{store,…}`.
/// The contact: the set stores' values joined by `\\`, a `contact` of the current creator with the class's label
/// (`\adresstext`, a constant of the class's language option) as its `name`; nothing when every store is blank. The sender: a `creator` of role
/// `sender` named by the name store, or, without a name, an empty one for the contacts — none when nothing is set.
pub fn define_contact() -> Result<()> {
  DefMacro!("\\lx@gbrief@contact{}{}{}", sub[(role, label, stores)] {
    let values = set_values(&stores)?;
    if values.is_empty() {
      return Ok(Tokens!());
    }
    let mut out = TokenizeInternal!(TeXString::assembled(format!("\\lx@add@contact[role={role},name={{")))
      .unlist();
    out.extend(label.unlist());
    out.extend(TokenizeInternal!("}]{").unlist());
    for (i, value) in values.into_iter().enumerate() {
      if i > 0 {
        out.push(T_CS!("\\\\"));
      }
      out.extend(value.unlist());
    }
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  DefMacro!("\\lx@gbrief@sender{}{}", sub[(name, stores)] {
    let name = store_value(&name.to_string())?.filter(|v| !is_blank(v));
    Ok(match name {
      Some(name) => {
        let mut out = TokenizeInternal!("\\lx@add@creator[role=sender]{").unlist();
        out.extend(name.unlist());
        out.push(T_END!());
        Tokens::new(out)
      },
      None if !set_values(&stores)?.is_empty() => {
        TokenizeInternal!("\\lx@add@frontmatter@container{ltx:creator}[role=sender]")
      },
      None => Tokens!(),
    })
  });
  // Neither class labels an address (the head's street and town, g-brief.cls:315-321; the recipient, :332-333,
  // g-brief2.cls:321-322): no kernel default "Address:" either (`\lx@contact@address@name`, Base_Utility).
  RawTeX!(r"\def\lx@contact@address@name{}");
  Ok(())
}

LoadDefinitions!({
  InputDefinitions!("g-brief", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  define_contact()?;
  // The foot's blocks in its order (g-brief.cls:342-358); the bank block only when bank, BLZ and account are all
  // set, the class's own condition; the return address only when set (its default repeats name, street and town).
  RawTeX!(
    r"\def\lx@gbrief@frontmatter{\global\let\lx@gbrief@frontmatter\relax
\lx@gbrief@sender{name}{strasse,zusatz,ort,land,telefon,telefax,telex,email,http,bank,retouradresse}%
\lx@gbrief@contact{address}{}{strasse,zusatz,ort,land}%
\lx@gbrief@contact{phone}{\telefontex}{telefon}%
\lx@gbrief@contact{fax}{\telefaxtext}{telefax}%
\lx@gbrief@contact{telex}{\telextext}{telex}%
\lx@gbrief@contact{email}{\emailtext}{email}%
\lx@gbrief@contact{http}{\httptext}{http}%
\ifx\bank\empty\else\ifx\blz\empty\else\ifx\konto\empty\else
\edef\lx@gbrief@blzline{\unexpanded\expandafter{\blztext}\space\unexpanded\expandafter{\blz}}%
\edef\lx@gbrief@kontoline{\unexpanded\expandafter{\kontotext}\space\unexpanded\expandafter{\konto}}%
\lx@gbrief@contact{bank}{\banktext}{bank,lx@gbrief@blzline,lx@gbrief@kontoline}\fi\fi\fi
\lx@gbrief@contact{return_address}{}{retouradresse}%
\ifx\postvermerk\empty\ifx\adresse\empty\else\lx@gbrief@addressee\fi\else\lx@gbrief@addressee\fi}
\def\lx@gbrief@addressee{\lx@add@frontmatter@container{ltx:creator}[role=addressee]%
\lx@gbrief@contact{postal_note}{}{postvermerk}\lx@gbrief@contact{address}{}{adresse}}
\expandafter\let\expandafter\lx@gbrief@begin\csname g-brief\endcsname
\expandafter\def\csname g-brief\endcsname{\lx@gbrief@frontmatter\lx@gbrief@begin}"
  );
});
