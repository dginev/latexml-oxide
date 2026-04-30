use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsrefs.sty.ltxml — Leverage the BibTeX implementation

  // Perl: LoadPool('BibTeX');
  // TODO: BibTeX pool not yet ported to Rust

  // Perl: DefParameterType('BibURL', ...) — semiverbatim URL reading
  // Perl: DefKeyVal('amsrefs', 'url', 'BibURL');
  // TODO: BibURL parameter type and amsrefs keyval not yet ported

  // \bib{key}{type}{keyval-pairs}
  // Perl: DefMacro('\bib{}{} RequiredKeyVals:amsrefs', sub { ... });
  // TODO: \bib requires BibTeX pool (CleanBibKey, NormalizeBibKey, ProcessBibTeXEntry)
  DefMacro!("\\bib{}{}{}", "");

  // \BibSpec — ignore
  DefMacro!("\\BibSpec{}{}", "");

  // \cites = \cite
  Let!("\\cites", "\\cite");

  // {bibdiv} environment — amsrefs.sty.ltxml L60-68.
  // beforeDigest: beforeDigestBibliography (preamble/counter/guard setup).
  // afterDigestBegin: beginBibliography_clean + Let('\par','\relax'). The
  // `_clean` variant skips setup_pseudo_bibitem because amsrefs bibliographies
  // always use explicit `\bibitem`; the pseudo-bibitem machinery rebinds
  // `\bibitem` and would break amsrefs' own `\bib{...}{...}{...}` entries.
  // The Let('\par','\relax') silences the implicit paragraph breaks between
  // entries (amsrefs items are sibling elements, not paragraphs).
  DefEnvironment!("{bibdiv}",
    "<ltx:bibliography xml:id='#id' \
     bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
     <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>\
     #body\
     </ltx:bibliography>",
    before_digest => {
      crate::engine::latex_constructs::before_digest_bibliography()?;
    },
    after_digest_begin => sub[whatsit] {
      crate::engine::latex_constructs::begin_bibliography_clean(whatsit)?;
      Let!("\\par", "\\relax");
    });

  // {biblist} environment
  DefEnvironment!("{biblist}", "<ltx:biblist>#body</ltx:biblist>");

  // \MR{...} — MathReviews link. Perl amsrefs.sty.ltxml L75-82:
  // properties closure patches old-style "12345 \# 67" → "12345:67" and
  // emits both mr= and href= AMS lookup URL. Ported directly with
  // a regex for the \# substitution.
  DefConstructor!("\\MR{}",
    "<ltx:ref href='#href' class='ltx_mathreviews'>MathReviews</ltx:ref>",
    enter_horizontal => true,
    properties => sub[args] {
      let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
      let re = RE.get_or_init(|| regex::Regex::new(r"\s+\\#\s*").unwrap());
      let mr = re.replace(&raw, ":").to_string();
      let href = format!("http://www.ams.org/mathscinet-getitem?mr={}", mr);
      Ok(stored_map!("mr" => mr, "href" => href))
    });

  // \ndash, \mdash
  DefConstructor!("\\ndash", "\u{2013}"); // EN DASH
  DefConstructor!("\\mdash", "\u{2014}"); // EM DASH
});
