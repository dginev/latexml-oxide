//! g-brief2.cls (the newer g-brief letter class, from g-brief.dtx): as `g_brief_cls`, the class is loaded raw and
//! the sender and addressee its first-page head and foot print (`\ps@firstpage`, g-brief2.cls:304-426, never
//! typeset by LaTeXML) become the letter's frontmatter at `\begin{g-brief}` (g-brief2.cls:252-253): the sender's
//! name, then a `contact` per foot block — the name lines, address, phone, internet and bank lines, each up to six
//! (`\NameZeileA`…, :176-231), labelled as the class labels them (`\adresstext`, :57-60) — and the addressee's postal
//! note and address. The internet block mixes mail addresses and URLs, so its role is `internet`, not `url` (whose
//! XSLT makes the text a link). User ruling 2026-10-01: the sender's contact block is kept. OXIDIZED_DESIGN_DIVERGENCES
//! #412. Witnesses g-brief/beispiel2 (recall 59.4 % at sweep #135), cv/ApplicationLetter.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("g-brief2", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  crate::g_brief_cls::define_contact()?;
  RawTeX!(
    r"\def\lx@gbrief@frontmatter{\global\let\lx@gbrief@frontmatter\relax
\lx@gbrief@sender{name}{namezeilea,namezeileb,namezeilec,namezeiled,namezeilee,namezeilef,namezeileg,adresszeilea,adresszeileb,adresszeilec,adresszeiled,adresszeilee,adresszeilef,telefonzeilea,telefonzeileb,telefonzeilec,telefonzeiled,telefonzeilee,telefonzeilef,internetzeilea,internetzeileb,internetzeilec,internetzeiled,internetzeilee,internetzeilef,bankzeilea,bankzeileb,bankzeilec,bankzeiled,bankzeilee,bankzeilef,retouradresse}%
\lx@gbrief@contact{name}{}{namezeilea,namezeileb,namezeilec,namezeiled,namezeilee,namezeilef,namezeileg}%
\lx@gbrief@contact{address}{\adresstext}{adresszeilea,adresszeileb,adresszeilec,adresszeiled,adresszeilee,adresszeilef}%
\lx@gbrief@contact{phone}{\telefontext}{telefonzeilea,telefonzeileb,telefonzeilec,telefonzeiled,telefonzeilee,telefonzeilef}%
\lx@gbrief@contact{internet}{\internettext}{internetzeilea,internetzeileb,internetzeilec,internetzeiled,internetzeilee,internetzeilef}%
\lx@gbrief@contact{bank}{\banktext}{bankzeilea,bankzeileb,bankzeilec,bankzeiled,bankzeilee,bankzeilef}%
\lx@gbrief@contact{return_address}{}{retouradresse}%
\ifx\postvermerk\empty\ifx\adresse\empty\else\lx@gbrief@addressee\fi\else\lx@gbrief@addressee\fi}
\def\lx@gbrief@addressee{\lx@add@frontmatter@container{ltx:creator}[role=addressee]%
\lx@gbrief@contact{postal_note}{}{postvermerk}\lx@gbrief@contact{address}{}{adresse}}
\expandafter\let\expandafter\lx@gbrief@begin\csname g-brief\endcsname
\expandafter\def\csname g-brief\endcsname{\lx@gbrief@frontmatter\lx@gbrief@begin}"
  );
});
