use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsrefs.sty.ltxml — Leverage the BibTeX implementation

  // Perl: LoadPool('BibTeX');
  LoadPool!("BibTeX");

  // Perl: DefParameterType('BibURL', ...) — semiverbatim URL reading
  // Perl: DefKeyVal('amsrefs', 'url', 'BibURL');
  // TODO: BibURL parameter type and amsrefs keyval not yet ported

  // \bib{key}{type}{keyval-pairs}
  // Perl: DefMacro('\bib{}{} RequiredKeyVals:amsrefs', sub { ... });
  //
  // Phase 2 transition stub (2026-05-15): wires the TeX-level
  // \bib invocation to the Phase 1 BibEntry registry. We read the
  // key + entry-type as strings and call `register_entry`, which
  // sets it as the current entry. Field extraction from the
  // keyval-pairs slot still needs a `RequiredKeyVals:amsrefs`
  // parameter type port (TODO Phase 2-3), so the keyvals are
  // captured raw via `add_raw_field("_raw_keyvals", ...)` for now.
  // Returning empty Tokens — no XML emit until Phase 3 lands the
  // bibAddToContainer / `\bib@@field` constructors.
  DefMacro!("\\bib{}{}{}", sub[args] {
    // ArgWrap::None's Display impl writes the literal "None", so
    // guard each slot with is_some() before calling to_string().
    let key = if args[0].is_some() { args[0].to_string() } else { String::new() };
    let entry_type = if args[1].is_some() { args[1].to_string() } else { String::new() };
    let raw_kv = if args[2].is_some() { args[2].to_string() } else { String::new() };
    let mut entry = latexml_engine::bibtex::BibEntry::new(
      key.clone(), entry_type);
    if !raw_kv.is_empty() {
      entry.add_raw_field("_raw_keyvals", raw_kv);
    }
    latexml_engine::bibtex::register_entry(&key, entry);
    Ok(Tokens!())
  });

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
