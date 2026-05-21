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
  // Reads the key + entry-type as strings and parses the keyval slot
  // via `parse_amsrefs_keyvals` into individual `BibEntry::add_raw_field`
  // calls. Mirrors Perl `amsrefs.sty.ltxml:42-51`: `lc()` on each key,
  // `UnTeX()` on each value (we use the raw source — close enough for
  // the downstream `\bib@field@<type>@<field>` dispatch since the
  // handlers themselves do the digestion).
  //
  // Emits `\ProcessBibTeXEntry{<key>}` (Perl L51) so the bibtex.rs
  // orchestration drives field dispatch + entry-type prepare/complete
  // and builds the `<ltx:bibentry>` XML.
  DefMacro!("\\bib{}{}{}", sub[args] {
    use latexml_engine::bibtex::{BibEntry, register_entry, parse_amsrefs_keyvals};
    let key = if args[0].is_some() { args[0].to_string() } else { String::new() };
    let entry_type = if args[1].is_some() { args[1].to_string() } else { String::new() };
    let raw_kv = if args[2].is_some() { args[2].to_string() } else { String::new() };
    let mut entry = BibEntry::new(key.clone(), entry_type);
    for (field, value) in parse_amsrefs_keyvals(&raw_kv) {
      entry.add_raw_field(field, value);
    }
    register_entry(&key, entry);
    // Emit `\ProcessBibTeXEntry{<key>}` to drive bibtex.rs orchestration.
    Ok(Invocation!(T_CS!("\\ProcessBibTeXEntry"),
      vec![Tokens::new(Explode!(&key))]))
  });

  // \BibSpec — ignore
  def_macro_noop("\\BibSpec{}{}")?;

  // \cites = \cite
  Let!("\\cites", "\\cite");
  // amsrefs.sty L1467: `\citelist{ \cite{key1} \cite{key2} ... }` —
  // grouped multi-citation where each `\cite` may carry `*{prenote}`.
  // Degrade to passing the body through; each inner `\cite` renders
  // independently. Witness 2404.11319.
  DefMacro!("\\citelist{}", "#1");

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
