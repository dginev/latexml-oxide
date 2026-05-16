use latexml_package::prelude::*;

// === biblatex .bbl entry-pipeline helpers ===
// Mirror Perl ar5iv-bindings/biblatex.sty.ltxml entry/endentry/name/list/field
// closures (L127-340). The .bbl file emits \entry{key}{type}{} … \endentry per
// reference; between them \field/\strng/\list/\name records metadata into the
// `biblatex_entry` HashStored, and \endentry flushes a `\bibitem[label]{key}
// authors. \newblock title. \newblock In: journal. year, pages.` token stream
// onto `rebuilt_bibtex_variant`. The list-end macros (\enddatalist etc.) wrap
// the accumulated variant in `\thebibliography{count}…\endthebibliography`.

fn bib_entry_get() -> SymHashMap<Stored> {
  match state::lookup_value("biblatex_entry") {
    Some(Stored::HashStored(map)) => map.clone(),
    _ => SymHashMap::default(),
  }
}

fn bib_entry_save(map: SymHashMap<Stored>) {
  state::assign_value(
    "biblatex_entry",
    Stored::HashStored(map),
    Some(Scope::Global),
  );
}

fn bib_entry_set_tokens(name: &str, val: Tokens) {
  let mut entry = bib_entry_get();
  entry.insert(name, Stored::Tokens(val));
  bib_entry_save(entry);
}

fn bib_entry_get_tokens(map: &SymHashMap<Stored>, name: &str) -> Option<Tokens> {
  map.get(name).and_then(|s| match s {
    Stored::Tokens(t) => Some(t.clone()),
    _ => None,
  })
}

fn bib_state_int(key: &str) -> i64 {
  match state::lookup_value(key) {
    Some(Stored::Int(n)) => n,
    _ => 0,
  }
}

fn bib_state_set_int(key: &str, value: i64) {
  state::assign_value(key, Stored::Int(value), Some(Scope::Global));
}

fn bib_variant_push(toks: Vec<Token>) {
  let mut acc: Vec<Token> = match state::lookup_value("rebuilt_bibtex_variant") {
    Some(Stored::Tokens(t)) => t.clone().unlist(),
    _ => Vec::new(),
  };
  acc.extend(toks);
  state::assign_value(
    "rebuilt_bibtex_variant",
    Stored::Tokens(Tokens::new(acc)),
    Some(Scope::Global),
  );
}

fn bib_as_thebibliography() -> Tokens {
  let variant: Vec<Token> = match state::lookup_value("rebuilt_bibtex_variant") {
    Some(Stored::Tokens(t)) => t.clone().unlist(),
    _ => return Tokens::default(),
  };
  if variant.is_empty() {
    return Tokens::default();
  }
  // Reset variant and entry-count so re-invocation is idempotent (matches
  // Perl L113-115).
  state::assign_value(
    "rebuilt_bibtex_variant",
    Stored::Tokens(Tokens::default()),
    Some(Scope::Global),
  );
  let count = bib_state_int("biblatex_entry_count");
  bib_state_set_int("biblatex_entry_count", 0);
  let preamble: Vec<Token> = match state::lookup_value("biblatex_preamble") {
    Some(Stored::Tokens(t)) => t.clone().unlist(),
    _ => Vec::new(),
  };
  let mut result: Vec<Token> = Vec::with_capacity(variant.len() + 16);
  result.push(T_CS!("\\thebibliography"));
  result.push(T_BEGIN!());
  result.extend(preamble);
  result.extend(ExplodeText!(&count.to_string()));
  result.push(T_END!());
  result.extend(variant);
  result.push(T_CS!("\\endthebibliography"));
  Tokens::new(result)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Strict-Perl translation of ar5iv-bindings/biblatex.sty.ltxml
  // (803 lines). Most macro definitions, conditionals, registers,
  // and the trailing RawTeX toggle block are now line-by-line
  // mirrors. The deep-closure bibliography rebuilder remains
  // DEFERRED (Perl L110-263 / L270-340 / L367-397) — those are
  // stubbed as no-op `DefMacro` so documents compile but the
  // bibliography body is not assembled.
  //
  // Audit cycle 2: caught Rust-only bugs vs Perl source
  //   * duplicate `\newtoggle{blx@citation}` (etoolbox toggle redef)
  //   * missing 38 of 60 toggles in trailing RawTeX
  //   * missing `\addbibresource` / `\printbibliography` /
  //     `Let \bibliography → \addbibresource` chain
  //   * 60+ DefMacro/DefRegister/DefConditional declarations missing

  // Perl L14-15: Warn that biblatex.sty is only minimally stubbed.
  Warn!("missing_file", "biblatex.sty",
    "biblatex.sty is only minimally stubbed and will not be interpreted raw.");

  // Perl L19-22: option processing
  DefKeyVal!("biblatex", "maxbibnames", "Number", "4");
  // Perl `DeclareOption(undef, sub { })` — ignore unknown options.
  DeclareOption!(None, {});
  ProcessOptions!();

  // Perl L24-30: dependencies. (`#RequirePackage('natbib')` etc commented out in Perl.)
  RequirePackage!("hyperref");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("babel_support");

  // Perl L37-56: cite variants. Use `Let!` (not `DefMacro` with body=`\cite`)
  // for the simple aliases: a DefMacro body of literal `\cite` produces an
  // infinite loop when the user does `\let\cite\parencite` (a documented
  // pattern in driver 2402.09928), because both CSes then expand to the
  // token `\cite` which expands to `\cite` ad infinitum. `Let!` makes the
  // alias resolve to the SAME Definition object directly (no
  // expansion-then-relookup), so user redefinitions don't cycle.
  Let!("\\parencite",    "\\cite");
  Let!("\\Parencite",    "\\cite");
  Let!("\\Cite",         "\\cite");
  DefMacro!("\\citet OptionalMatch:* [][] Semiverbatim",   "\\cite[#2 ]{#4}", locked => true);
  DefMacro!("\\citep OptionalMatch:* [][] Semiverbatim",   "\\cite[#2]{#4}",  locked => true);
  DefMacro!("\\citealt OptionalMatch:* [][] Semiverbatim", "\\cite[#2]{#4}",  locked => true);
  DefMacro!("\\citealp OptionalMatch:* [][] Semiverbatim", "\\cite[#2]{#4}",  locked => true);
  Let!("\\citenum",      "\\cite");
  Let!("\\citem",        "\\cite");
  DefMacro!("\\autocite OptionalMatch:* [][]{}", "\\cite[#2]{#4}", locked => true);
  DefMacro!("\\Autocite OptionalMatch:* [][]{}", "\\cite[#2]{#4}", locked => true);
  Let!("\\fullcite",     "\\cite");
  Let!("\\footcite",     "\\cite");
  Let!("\\footcitetext", "\\cite");
  Let!("\\smartcite",    "\\cite");
  Let!("\\textcite",     "\\cite");
  Let!("\\Textcite",     "\\cite");
  Let!("\\supercite",    "\\cite");
  Let!("\\citeauthor",   "\\cite");
  Let!("\\citetitle",    "\\cite");

  // \parencites etc. — biblatex multi-cite variants. Real biblatex
  // accepts an arbitrary number of `[prenote][postnote]{key}` triples
  // which are individually rendered with parens around the whole list.
  // Stub: degrade to \cite of the first key. Driver: 1906.11485.
  DefMacro!("\\parencites OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\Parencites OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\citetexts  OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\autocites  OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\Autocites  OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\textcites  OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\Textcites  OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\smartcites OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\footcites  OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);
  DefMacro!("\\supercites OptionalMatch:* [][] Semiverbatim", "\\cite{#4}", locked => true);

  // \DeclareLabeldate — biblatex datacommands declaration. No-op stub.
  DefMacro!("\\DeclareLabeldate {}", "");

  // Perl L64-67: passthroughs
  DefMacro!("\\unspace", "\\relax");
  DefMacro!("\\blx@imc@resetpunctfont", "\\relax");
  DefMacro!("\\blx@postpunct", "\\@empty");
  DefRegister!("\\c@highnamepenalty" => Number(0));

  // Perl L69-72
  DefMacro!("\\addslash", "/\\hskip\\z@skip");
  DefMacro!("\\adddot", ".");
  DefMacro!("\\addcomma", ",");
  DefMacro!("\\autocap{}", "#1");

  // Perl L75-85
  DefMacro!("\\addspace",        "\\space");
  DefMacro!("\\addnbspace",      "\\space");
  DefMacro!("\\addthinspace",    "\\space");
  DefMacro!("\\addnbthinspace",  "\\space");
  DefMacro!("\\addlowpenspace",  "\\space");
  DefMacro!("\\addhighpenspace", "\\space");
  DefMacro!("\\addlpthinspace",  "\\space");
  DefMacro!("\\addhpthinspace",  "\\space");
  DefMacro!("\\addabbrvspace",   "\\space");
  DefMacro!("\\addabthinspace",  "\\space");
  DefMacro!("\\adddotspace",     "\\unspace\\adddot\\space");

  // Perl L87-91
  DefMacro!("\\noligature",   "\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphen",       "\\nobreak-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\nbhyphen",     "\\nobreak\\mbox{-}\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphenate",    "\\nobreak\\-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\allowhyphens", "\\nobreak\\hskip\\z@skip");

  // Perl L93-99
  DefMacro!("\\bibinitperiod",      "\\adddot");
  DefMacro!("\\bibinithyphendelim", ".\\mbox{-}");
  DefMacro!("\\bibnamedelima",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimb",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimc",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimd",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimi",      "\\addnbspace");

  // Perl L101-106: \datalist / \sortlist set the `biblatex_with_keyvals`
  // flag globally — Perl's `\name` closure (Cycle 9) reads it to choose
  // 3-arg vs 4-arg / keyval-vs-positional dispatch.
  DefMacro!("\\datalist[]{}", sub[_args] {
    state::assign_value("biblatex_with_keyvals", Stored::from(1),
      Some(Scope::Global));
    Ok(Tokens::new(vec![]))
  });
  DefMacro!("\\sortlist[]{}", sub[_args] {
    state::assign_value("biblatex_with_keyvals", Stored::from(1),
      Some(Scope::Global));
    Ok(Tokens::new(vec![]))
  });
  // Perl L107-108: \lossort / \refsection — empty stubs.
  DefMacro!("\\lossort", "", locked => true);
  DefMacro!("\\refsection{}", "", locked => true);

  // biblatex `.bbl` files emitted by biber include `\true{moreauthor}` /
  // `\true{morelabelname}` / `\false{...}` flags on multi-author entries.
  // Perl `ar5iv-bindings/bindings/biblatex.sty.ltxml:641-645` defines
  // `\blx@bbl@booltrue{}` / `\blx@bbl@boolfalse{}` as `\relax` stubs and
  // `\let\true\blx@bbl@booltrue` if `\true` is undefined.
  //
  // Rust never sets either, so the .bbl raw-load hits
  // `Error:undefined:\true` on every multi-author bibitem (witness:
  // arXiv:2509.15629 / 2509.21728 — biblatex `.bbl` v3.3 format with
  // multi-author entries).
  DefMacro!("\\blx@bbl@booltrue{}", "", locked => true);
  DefMacro!("\\blx@bbl@boolfalse{}", "", locked => true);
  Let!("\\true", "\\blx@bbl@booltrue");
  Let!("\\false", "\\blx@bbl@boolfalse");
  // Perl L122-125: \enddatalist / \endsortlist / \endlossort / \endrefsection
  // → biblatex_as_thebibliography rebuilder. Wraps the accumulated bibitems
  // emitted by repeated \endentry calls in `\thebibliography{count}…
  // \endthebibliography`.
  DefMacro!("\\enddatalist", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);
  DefMacro!("\\endsortlist", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);
  DefMacro!("\\endlossort", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);
  DefMacro!("\\endrefsection", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);

  // Perl L127-130: \entry{key}{type}{} initializes the entry hash so that the
  // following \field/\strng/\name/\list directives have a place to record
  // metadata. The 3rd arg is options (Perl ignores it).
  DefMacro!("\\entry{}{}{}", sub[(key, ty, _opts)] {
    let mut entry: SymHashMap<Stored> = SymHashMap::default();
    entry.insert("key", Stored::Tokens(key));
    entry.insert("type", Stored::Tokens(ty));
    bib_entry_save(entry);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L132-263: \endentry — flush the accumulated entry hash as a
  // `\bibitem[label]{key} authors. \newblock title. \newblock In: journal.
  // year, pages.` token stream onto rebuilt_bibtex_variant.
  // Simplified port: handles label-or-auto-label, key, authors (if collected
  // by \name as plain "fullname" tokens), title, journal/booktitle, year,
  // pages, doi/url/eprint. Pre-typeset name strings (the `{names}` array
  // form) are emitted comma-joined.
  DefMacro!("\\endentry", sub[_args] {
    let entry = bib_entry_get();
    state::assign_value("biblatex_entry", Stored::None, Some(Scope::Global));

    // label: Perl uses labelalpha if present, else label, else auto-counter.
    let label_toks = bib_entry_get_tokens(&entry, "labelalpha")
      .or_else(|| bib_entry_get_tokens(&entry, "label"));
    let label_str: String = match label_toks {
      Some(t) if !t.is_empty() => {
        // Perl: strip \word and braces from label for safety.
        let s = t.to_string();
        let mut cleaned = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(ch) = chars.next() {
          if ch == '\\' {
            // skip \word
            for c in chars.by_ref() { if !c.is_ascii_alphabetic() { break; } }
          } else if ch != '{' && ch != '}' {
            cleaned.push(ch);
          }
        }
        let cleaned = cleaned.trim().to_string();
        if cleaned.is_empty() {
          let n = bib_state_int("biblatex_auto_label") + 1;
          bib_state_set_int("biblatex_auto_label", n);
          n.to_string()
        } else {
          cleaned
        }
      },
      _ => {
        let n = bib_state_int("biblatex_auto_label") + 1;
        bib_state_set_int("biblatex_auto_label", n);
        n.to_string()
      },
    };

    // Bump entry count for thebibliography wrapper.
    bib_state_set_int("biblatex_entry_count", bib_state_int("biblatex_entry_count") + 1);

    let mut variant: Vec<Token> = Vec::with_capacity(64);
    let key_toks = bib_entry_get_tokens(&entry, "key").unwrap_or_default();
    variant.push(T_CS!("\\bibitem"));
    variant.push(T_OTHER!("["));
    variant.extend(ExplodeText!(&label_str));
    variant.push(T_OTHER!("]"));
    variant.push(T_BEGIN!());
    variant.extend(key_toks.unlist());
    variant.push(T_END!());

    // Authors: if \name stashed a comma-joined string under "authors_str",
    // emit it. Defer the et-al / per-author re-tokenization for now — most
    // .bbl files give us pre-formatted author tokens.
    let authors_toks = bib_entry_get_tokens(&entry, "authors_str");
    let mut have_authors = false;
    if let Some(toks) = authors_toks {
      if !toks.is_empty() {
        variant.extend(toks.unlist());
        have_authors = true;
      }
    }

    // Title
    if let Some(title) = bib_entry_get_tokens(&entry, "title") {
      if !title.is_empty() {
        if have_authors {
          variant.push(T_CS!("\\newblock"));
        }
        variant.push(T_OTHER!("`"));
        variant.push(T_OTHER!("`"));
        variant.extend(title.unlist());
        variant.push(T_OTHER!("'"));
        variant.push(T_OTHER!("'"));
      }
    }
    // Note
    if let Some(note) = bib_entry_get_tokens(&entry, "note") {
      if !note.is_empty() {
        variant.push(T_SPACE!());
        variant.extend(note.unlist());
      }
    }
    // Journal / booktitle
    let journal = bib_entry_get_tokens(&entry, "booktitle")
      .or_else(|| bib_entry_get_tokens(&entry, "journaltitle"))
      .or_else(|| bib_entry_get_tokens(&entry, "journal"));
    if let Some(j) = journal.as_ref() {
      if !j.is_empty() {
        variant.push(T_CS!("\\newblock"));
        variant.extend(ExplodeText!("In "));
        variant.push(T_CS!("\\emph"));
        variant.push(T_BEGIN!());
        variant.extend(j.clone().unlist());
        variant.push(T_END!());
      }
    }
    // Volume + (number)
    if let Some(volume) = bib_entry_get_tokens(&entry, "volume") {
      if journal.is_some() && !volume.is_empty() {
        variant.push(T_SPACE!());
        variant.push(T_CS!("\\textbf"));
        variant.push(T_BEGIN!());
        variant.extend(volume.unlist());
        if let Some(num) = bib_entry_get_tokens(&entry, "number") {
          if !num.is_empty() {
            variant.push(T_OTHER!("."));
            variant.extend(num.unlist());
          }
        }
        variant.push(T_END!());
      }
    }
    // Publisher / location
    if let Some(publisher) = bib_entry_get_tokens(&entry, "publisher") {
      if !publisher.is_empty() {
        variant.push(T_CS!("\\newblock"));
        if let Some(loc) = bib_entry_get_tokens(&entry, "location") {
          if !loc.is_empty() {
            variant.extend(loc.unlist());
            variant.push(T_OTHER!(":"));
            variant.push(T_SPACE!());
          }
        }
        variant.extend(publisher.unlist());
      }
    }
    // Year
    if let Some(year) = bib_entry_get_tokens(&entry, "year") {
      if !year.is_empty() {
        variant.push(T_OTHER!(","));
        variant.push(T_SPACE!());
        variant.extend(year.unlist());
      }
    }
    // Pages
    if let Some(pages) = bib_entry_get_tokens(&entry, "pages") {
      if !pages.is_empty() {
        variant.push(T_OTHER!(","));
        variant.push(T_SPACE!());
        variant.extend(ExplodeText!("pp. "));
        variant.extend(pages.unlist());
      }
    }
    // DOI / URL / eprint (URL-style)
    if let Some(doi) = bib_entry_get_tokens(&entry, "doi") {
      if !doi.is_empty() {
        variant.push(T_CS!("\\newblock"));
        variant.extend(ExplodeText!("DOI: "));
        variant.push(T_CS!("\\href"));
        variant.push(T_BEGIN!());
        variant.extend(ExplodeText!("https://dx.doi.org/"));
        variant.extend(doi.clone().unlist());
        variant.push(T_END!());
        variant.push(T_BEGIN!());
        variant.extend(doi.unlist());
        variant.push(T_END!());
      }
    } else if let Some(url) = bib_entry_get_tokens(&entry, "url") {
      if !url.is_empty() {
        variant.push(T_CS!("\\newblock"));
        variant.extend(ExplodeText!("URL: "));
        variant.push(T_CS!("\\url"));
        variant.push(T_BEGIN!());
        variant.extend(url.unlist());
        variant.push(T_END!());
      }
    } else if let Some(eprint) = bib_entry_get_tokens(&entry, "eprint") {
      if !eprint.is_empty() {
        variant.push(T_CS!("\\newblock"));
        let etype = bib_entry_get_tokens(&entry, "eprinttype")
          .map(|t| t.to_string().to_uppercase()).unwrap_or_default();
        let etype = if etype == "ARXIV" { "arXiv".to_string() }
          else if etype.is_empty() { "eprint".to_string() }
          else { etype };
        variant.extend(ExplodeText!(&format!("{etype}:")));
        variant.push(T_SPACE!());
        if etype == "arXiv" {
          variant.push(T_CS!("\\href"));
          variant.push(T_BEGIN!());
          variant.extend(ExplodeText!("https://arxiv.org/abs/"));
          variant.extend(eprint.clone().unlist());
          variant.push(T_END!());
          variant.push(T_BEGIN!());
          variant.extend(eprint.unlist());
          variant.push(T_END!());
        } else {
          variant.extend(eprint.unlist());
        }
      }
    }

    bib_variant_push(variant);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L265-268: BiblatexAuthor keyvals
  DefKeyVal!("BiblatexAuthor", "given",   "");
  DefKeyVal!("BiblatexAuthor", "giveni",  "");
  DefKeyVal!("BiblatexAuthor", "family",  "");
  DefKeyVal!("BiblatexAuthor", "familyi", "");

  // Perl L270-346: \name{type}{count}{maybe-content} — biblatex's author
  // record. The TeX-2.5+ .bbl shape is `\name{author}{N}{}{ {{}{Family}…} }`
  // where the 3rd arg is empty and the 4th is the author body. Older variants
  // pass 3 args. Simplified port: declare 4 mandatory args; the 4th-arg
  // capture covers the modern shape used by the vast majority of arxiv .bbl
  // files. The body holds N inner-author groups, each of the form
  // `{hash}{family}{familyi}{given}{giveni}{}{}{}{}` — we extract family +
  // given pairs into "Given Family" strings (no Perl-faithful keyval/hash
  // ordering yet) and stash them comma-joined under `authors_str` /
  // `editors_str` in the entry hash for `\endentry` to emit verbatim.
  DefMacro!("\\name{}{}{}{}", sub[(ty, _count, _maybe, body)] {
    let type_str = ty.to_string();
    // The body's tokens start with `{` `{}` (empty hash) or `{hash=…}` then
    // `{family}{familyi}{given}{giveni}{}{}{}{}` repeated, separated by
    // optional whitespace. Walk top-level groups.
    let body_toks: Vec<Token> = body.unlist();
    // Helper: scan one balanced {...} group, advancing index.
    fn read_group(tokens: &[Token], i: &mut usize) -> Option<Vec<Token>> {
      while *i < tokens.len() {
        let cc = tokens[*i].code;
        if cc == Catcode::SPACE { *i += 1; continue; }
        if cc == Catcode::BEGIN { break; }
        return None; // not a group
      }
      if *i >= tokens.len() { return None; }
      *i += 1; // consume BEGIN
      let mut depth = 1usize;
      let mut out = Vec::new();
      while *i < tokens.len() {
        let cc = tokens[*i].code;
        if cc == Catcode::BEGIN {
          depth += 1;
          out.push(tokens[*i]);
        } else if cc == Catcode::END {
          depth -= 1;
          if depth == 0 { *i += 1; return Some(out); }
          out.push(tokens[*i]);
        } else {
          out.push(tokens[*i]);
        }
        *i += 1;
      }
      Some(out) // unterminated: best effort
    }
    let mut names: Vec<String> = Vec::new();
    let mut idx = 0usize;
    while idx < body_toks.len() {
      // Skip space tokens between author groups.
      while idx < body_toks.len() &&
            body_toks[idx].code == Catcode::SPACE {
        idx += 1;
      }
      if idx >= body_toks.len() { break; }
      // Read the per-author group.
      let author_grp = match read_group(&body_toks, &mut idx) {
        Some(g) => g,
        None => break,
      };
      // Inside, read up to 5 sub-groups: hash, family, familyi, given, giveni
      let mut j = 0usize;
      let _hash = read_group(&author_grp, &mut j);
      let family = read_group(&author_grp, &mut j)
        .map(|g| Tokens::new(g).to_string()).unwrap_or_default();
      let _familyi = read_group(&author_grp, &mut j);
      let given = read_group(&author_grp, &mut j)
        .map(|g| Tokens::new(g).to_string()).unwrap_or_default();
      // Trim whitespace.
      let family = family.trim().to_string();
      let given = given.trim().to_string();
      let fullname = if !given.is_empty() && !family.is_empty() {
        format!("{given} {family}")
      } else if !family.is_empty() {
        family
      } else if !given.is_empty() {
        given
      } else {
        continue;
      };
      names.push(fullname);
    }
    // Format with et-al limit (default 4 per Perl L192).
    let etal_limit = bib_state_int("biblatex_maxbibnames");
    let etal_limit = if etal_limit > 0 { etal_limit as usize } else { 4 };
    let joined = if names.len() > etal_limit {
      format!("{} et al.", names[0])
    } else {
      let mut acc = String::new();
      let n = names.len();
      for (k, name) in names.iter().enumerate() {
        if k > 0 {
          if k + 1 == n {
            acc.push_str(" and ");
          } else {
            acc.push_str(", ");
          }
        }
        acc.push_str(name);
      }
      acc
    };
    // Stash under "authors_str" or "editors_str" depending on type.
    let key = if type_str.trim() == "editor" { "editors_str" } else { "authors_str" };
    if !joined.is_empty() {
      let toks = Tokens::new(ExplodeText!(&joined));
      bib_entry_set_tokens(key, toks);
    }
    Ok(Tokens::default())
  }, locked => true);

  // Perl L342-346: \list{name}{count}{value} — record value under `name` in
  // the entry hash. Perl ignores `count` for count==1 and notes "more
  // support needed" for >1. Same here.
  DefMacro!("\\list{}{}{}", sub[(name, _count, val)] {
    let name_s = name.to_string();
    bib_entry_set_tokens(name_s.trim(), val);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L355-363: \field{name}{value} / \strng{name}{value} — record value
  // under `name` in the entry hash. Perl uses DefPrimitive (immediate
  // side-effect, no expansion). DefMacro with sub gives equivalent behavior
  // at digestion time since the body is empty.
  DefMacro!("\\field{}{}", sub[(name, val)] {
    let name_s = name.to_string();
    bib_entry_set_tokens(name_s.trim(), val);
    Ok(Tokens::default())
  }, locked => true);
  DefMacro!("\\strng{}{}", sub[(name, val)] {
    let name_s = name.to_string();
    bib_entry_set_tokens(name_s.trim(), val);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L348-354
  DefMacro!("\\AtEveryBibitem{}",   "");
  DefMacro!("\\AtEveryCitekey{}",   "");
  DefMacro!("\\keyw{}",             "");
  DefMacro!("\\bibinitdelim",       "");
  // Note: \bibinithyphendelim re-defined here as just "-" per Perl L352
  // (overrides the L94 definition; Perl runs them in order).
  DefMacro!("\\bibinithyphendelim", "-");
  DefMacro!("\\bibrangedash", "\u{2013}");
  DefMacro!("\\bibnamedelimi", " ");

  // Perl L364
  DefMacro!("\\range{}{}", "");

  // Perl L367-369: \preamble{...} stashes the arg into biblatex_preamble
  // for the rebuilder (Cycle 7) and *also* re-emits the arg (Perl returns
  // $_[1]) so the preamble is digested in the current context too.
  DefMacro!("\\preamble{}", sub[(arg)] {
    state::assign_value("biblatex_preamble",
      Stored::Tokens(arg.clone()), Some(Scope::Global));
    Ok(arg.clone())
  });

  // Perl L371-397: \biblatex@verb{key}…\endverb captures a verbatim field
  // that biblatex's .bbl emits in the form
  //     \verb{key}
  //     \verb VALUE
  //     \endverb
  // The first `\verb` is `\let`'d to `\biblatex@verb` and reads `{key}`;
  // the second `\verb` then reads VALUE as a raw line; `\endverb` stores
  // VALUE under key. Perl uses gullet->readRawLine + dynamic re-bind. Rust
  // simulates the same effect with a single delimited macro that reads both
  // `{key}` and "Until:\\endverb" — the captured body is everything between
  // the first `\verb{key}` line and `\endverb`, including the second
  // `\verb` token plus the URL chars. We strip the inner `\verb` token and
  // surrounding whitespace before storing.
  // Without this, \verb LEAKS the URL into body text — and consumes the
  // first character (`h` of `http`) as a `{}` arg, producing the
  // characteristic `ttp://…` corruption on egpaper_final.tex.
  DefMacro!("\\biblatex@verb{} Until:\\endverb", sub[(key, body)] {
    let key_str = key.to_string();
    let body_toks = body.unlist();
    // Skip leading whitespace + the inner `\verb` token + one space.
    let mut start = 0usize;
    while start < body_toks.len() {
      let cc = body_toks[start].code;
      if cc == Catcode::SPACE || cc == Catcode::EOL { start += 1; continue; }
      break;
    }
    if start < body_toks.len() && body_toks[start].code == Catcode::CS {
      // Skip the `\verb` CS token (or whatever CS leads — should be \verb).
      start += 1;
      // Skip following space tokens.
      while start < body_toks.len() {
        let cc = body_toks[start].code;
        if cc == Catcode::SPACE || cc == Catcode::EOL { start += 1; continue; }
        break;
      }
    }
    // Strip trailing whitespace.
    let mut end = body_toks.len();
    while end > start {
      let cc = body_toks[end - 1].code;
      if cc == Catcode::SPACE || cc == Catcode::EOL { end -= 1; continue; }
      break;
    }
    // Sanitize: biblatex `\verb` is a verbatim primitive — its body is a
    // literal string. The mouth tokenized chars with their normal catcodes
    // (so `_` is SUB, `^` is SUPER, `#` is PARAM, etc.). When `\endentry`
    // later splices these tokens into `\href{URL}{text}` the SUB chars
    // trigger `Script _ can only appear in math mode` during horizontal
    // digestion. Reset structural catcodes to OTHER so the captured string
    // round-trips through the bibitem variant safely.
    // Witness cluster: ~29 papers/stage in next_warning_papers (Stages
    // 15-20 v3) hit this on biblatex bbl `\verb 10.1162/EVCO_a_00133`.
    let value_vec: Vec<Token> = body_toks[start..end].iter().map(|tok| {
      match tok.code {
        Catcode::SUB | Catcode::SUPER | Catcode::PARAM |
        Catcode::ALIGN | Catcode::MATH | Catcode::ACTIVE => {
          Token { text: tok.text, code: Catcode::OTHER }
        },
        _ => *tok,
      }
    }).collect();
    let value = Tokens::new(value_vec);
    bib_entry_set_tokens(key_str.trim(), value);
    Ok(Tokens::default())
  }, locked => true);
  // \biblatex@endverb is consumed by the Until: delimiter on \biblatex@verb,
  // but if it ever fires standalone (degenerate input), no-op it.
  DefMacro!("\\biblatex@endverb", "", locked => true);

  // Perl L400-408: \addbibresource{file,...} pushes onto biblatex_resources.
  // Then `\biblatex@saved@bibliography` is bound to whatever `\bibliography`
  // means at this point (classic LaTeX bibtex), and `\bibliography` is
  // re-let to `\addbibresource` so any classic `\bibliography{...}`
  // invocation in a biblatex doc just records resources.
  // see arXiv:1502.02314 for a paper that left in classic \bibliography
  // alongside biblatex; both forms must end up populating the resource list.
  DefPrimitive!("\\addbibresource{}", sub[(file_list_arg)] {
    // Perl: split(/\s*,\s*/, ToString($_[1])) — split on commas and
    // strip surrounding whitespace.
    let raw = file_list_arg.to_string();
    for part in raw.split(',') {
      let file = part.trim();
      if !file.is_empty() {
        push_value("biblatex_resources", Stored::String(arena::pin(file)))?;
      }
    }
  });
  Let!("\\biblatex@saved@bibliography", "\\bibliography");
  Let!("\\bibliography",                "\\addbibresource");

  // Perl L410-418: \printbibliography → \biblatex@printbibliography, which
  // emits the saved \biblatex@saved@bibliography call over popped resources.
  DefMacro!("\\printbibliography",
    "\\let\\verb\\biblatex@verb\\let\\endverb\\biblatex@endverb\\biblatex@printbibliography");
  DefMacro!("\\biblatex@printbibliography[]", sub[(_opts)] {
    let mut resources = Vec::new();
    while let Some(res) = state::pop_value("biblatex_resources")? {
      if !resources.is_empty() {
        resources.push(T_OTHER!(","));
        resources.push(T_SPACE!());
      }
      resources.push(T_OTHER!(res.to_string()));
    }
    Ok(Tokens!(
      T_CS!("\\biblatex@saved@bibliography"),
      T_BEGIN!(),
      Tokens::new(resources),
      T_END!()
    ))
  }, locked => true);

  // Perl L420-424
  DefMacro!("\\warn{}", "");
  DefMacro!("\\xref{}", "");
  DefMacro!("\\fakeset{}", "");

  // Perl L429-434: language API (no-ops)
  DefMacro!("\\DeclareLanguageMapping{}{}", "");
  DefMacro!("\\DeclareLanguageMappingSuffix{}", "");
  DefMacro!("\\DefineHyphenationExceptions{}{}", "");
  DefMacro!("\\DefineBibliographyExtras{}{}", "");
  DefMacro!("\\UndefineBibliographyExtras{}{}", "");
  DefMacro!("\\DefineBibliographyStrings{}{}", "");

  // Perl L436-438
  DefMacro!("\\DeclareNameFormat OptionalMatch:* []{}{}",  "");
  DefMacro!("\\DeclareListFormat OptionalMatch:* []{}{}",  "");
  DefMacro!("\\DeclareFieldFormat OptionalMatch:* []{}{}", "");

  // Perl L440-458
  DefMacro!("\\DeclareNameInputHandler{}{}", "");
  DefMacro!("\\DeclareListInputHandler{}{}", "");
  DefMacro!("\\DeclareFieldInputHandler{}{}", "");
  DefMacro!("\\DeclareSortingScheme[]{}", "");
  DefMacro!("\\DeclareSortingTemplate[]{}", "");
  DefMacro!("\\DeclareSortingNamekeyScheme[]{}", "");
  DefMacro!("\\namepart[]{}", "");
  DefMacro!("\\DeclareLabelalphaNameTemplate[]{}", "");
  DefMacro!("\\DeclareNameAlias{}{}", "");
  DefMacro!("\\DeclareIndexNameAlias{}{}", "");
  DefMacro!("\\DeclareListAlias{}{}", "");
  DefMacro!("\\DeclareIndexListAlias{}{}", "");
  DefMacro!("\\DeclareFieldAlias{}{}", "");
  DefMacro!("\\DeclareIndexFieldAlias{}{}", "");
  DefMacro!("\\DeclareNameWrapperAlias{}{}", "");
  DefMacro!("\\DeclareListWrapperAlias{}{}", "");
  DefMacro!("\\DeclareDelimcontextAlias{}{}", "");
  DefMacro!("\\UndeclareDelimcontextAlias{}", "");
  DefMacro!("\\DeclareCiteCommand OptionalMatch:* {}[]{}{}{}{}", "");

  // Perl L460-481
  DefMacro!("\\DeclareBibliographyExtras{}", "");
  DefMacro!("\\DeclareBibliographyStrings{}", "");
  DefMacro!("\\DeclareBibliographyDriver{}{}", "");
  DefMacro!("\\DeclareHyphenationExceptions{}", "");
  DefMacro!("\\InheritBibliographyExtras{}", "");
  DefMacro!("\\InheritBibliographyStrings{}", "");
  DefMacro!("\\UndeclareBibliographyExtras{}", "");
  DefMacro!("\\NewCount", "\\newcount");
  DefMacro!("\\ExecuteBibliographyOptions[]{}", "");
  DefMacro!("\\AtBeginBibliography{}", "");
  DefMacro!("\\AtEveryEntrykey{}{}{}", "");
  DefMacro!("\\UseBibitemHook", "");
  DefMacro!("\\UseUsedriverHook", "");
  DefMacro!("\\UseEveryCiteHook", "");
  DefMacro!("\\UseEveryCitekeyHook", "");
  DefMacro!("\\UseEveryMultiCiteHook", "");
  DefMacro!("\\UseNextCiteHook", "");
  DefMacro!("\\UseNextCitekeyHook", "");
  DefMacro!("\\UseNextMultiCiteHook", "");
  DefMacro!("\\UseVolciteHook", "");
  DefMacro!("\\DeferNextCitekeyHook", "");

  // Perl L483-491: bibmacro/heading/environment helpers
  DefMacro!("\\providebibmacro OptionalMatch:* {}[][]{}", "");
  DefMacro!("\\renewbibmacro OptionalMatch:* {}[][]{}", "");
  DefMacro!("\\newbibmacro OptionalMatch:* {}[][]{}", "");
  DefMacro!("\\restorebibmacro OptionalMatch:* {}", "");
  DefMacro!("\\savebibmacro OptionalMatch:* {}", "");
  DefMacro!("\\defbibheading OptionalMatch:* {}[]{}", "");
  DefMacro!("\\defbibenvironment OptionalMatch:* {}{}{}{}", "");
  DefMacro!("\\restorecommand OptionalMatch:* {}", "");
  DefMacro!("\\savecommand OptionalMatch:* {}", "");

  // Perl L493-500
  DefRegister!("\\labelnumberwidth" => Glue!("0pt"));
  DefRegister!("\\labelalphawidth" => Glue!("0pt"));
  DefRegister!("\\biblabelsep" => Glue!("0pt"));
  DefRegister!("\\bibnamesep" => Glue!("0pt"));
  DefRegister!("\\bibitemsep" => Glue!("0pt"));
  DefRegister!("\\bibinitsep" => Glue!("0pt"));
  DefRegister!("\\bibparsep" => Glue!("0pt"));
  DefRegister!("\\bibhang" => Glue!("0pt"));

  // Perl L553-604: 50 conditionals
  DefConditional!("\\ifandothers");
  DefConditional!("\\ifbibindex");
  DefConditional!("\\ifbibliography");
  DefConditional!("\\ifbibstring");
  DefConditional!("\\ifcapital");
  DefConditional!("\\ifcategory");
  DefConditional!("\\ifcitation");
  DefConditional!("\\ifciteibid");
  DefConditional!("\\ifciteidem");
  DefConditional!("\\ifciteindex");
  DefConditional!("\\ifciteseen");
  DefConditional!("\\ifcurrentfield");
  DefConditional!("\\ifcurrentlist");
  DefConditional!("\\ifcurrentname");
  DefConditional!("\\ifentrycategory");
  DefConditional!("\\ifentrykeyword");
  DefConditional!("\\ifentryseen");
  DefConditional!("\\ifentrytype");
  DefConditional!("\\iffieldbibstring");
  DefConditional!("\\iffieldequalcs");
  DefConditional!("\\iffieldequals");
  DefConditional!("\\iffieldequalstr");
  DefConditional!("\\iffieldint");
  DefConditional!("\\iffieldnum");
  DefConditional!("\\iffieldnums");
  DefConditional!("\\iffieldpages");
  DefConditional!("\\iffieldsequal");
  DefConditional!("\\iffieldundef");
  DefConditional!("\\iffirstinits");
  DefConditional!("\\iffirstonpage");
  DefConditional!("\\iffootnote");
  DefConditional!("\\ifhyperref");
  DefConditional!("\\ifinteger");
  DefConditional!("\\ifkeyword");
  DefConditional!("\\ifloccit");
  DefConditional!("\\ifmoreitems");
  DefConditional!("\\ifmorenames");
  DefConditional!("\\ifnameequalcs");
  DefConditional!("\\ifnameequals");
  DefConditional!("\\ifnamesequal");
  DefConditional!("\\ifnameundef");
  DefConditional!("\\ifnatbibmode");
  DefConditional!("\\ifnumeral");
  DefConditional!("\\ifnumerals");
  DefConditional!("\\ifopcit");
  DefConditional!("\\ifpages");
  DefConditional!("\\ifsamepage");
  DefConditional!("\\ifsingletitle");
  DefConditional!("\\ifuseauthor");
  DefConditional!("\\ifuseeditor");
  DefConditional!("\\ifuseprefix");
  DefConditional!("\\ifusetranslator");

  // Perl L608-610
  DefMacro!("\\key{}", "");
  // \keyw is already defined L348 (DefMacro empty, see above).
  DefMacro!("\\keyword{}", "");

  // Perl L632-635
  DefMacro!("\\ppspace", "\\addnbspace");
  DefMacro!("\\sqspace", "\\addnbspace");
  DefMacro!("\\labelalphaothers", "+");
  DefMacro!("\\sortalphaothers", "\\labelalphaothers");

  // Perl L638
  DefMacro!("\\sort[]{}", "");

  // Perl L641-645: bool stubs + AtBeginDocument-guarded \true/\false bind.
  // documents such as 1811.01740 conflict with unconditional binding.
  DefMacro!("\\blx@bbl@booltrue{}",  "\\relax", locked => true);
  DefMacro!("\\blx@bbl@boolfalse{}", "\\relax", locked => true);
  at_begin_document(TokenizeInternal!(
    r"\@ifundefined{true}{\let\true\blx@bbl@booltrue}{}\@ifundefined{false}{\let\false\blx@bbl@boolfalse}{}"
  ))?;

  // Perl L646-671: \the* counter-readouts (all empty)
  DefMacro!("\\type{}", "");
  DefMacro!("\\subtype{}", "");
  DefMacro!("\\theparenlevel", "");
  DefMacro!("\\therefsection", "");
  DefMacro!("\\therefsegment", "");
  DefMacro!("\\theuniquelist", "");
  DefMacro!("\\theuniquename", "");
  DefMacro!("\\themulticitecount", "");
  DefMacro!("\\themulticitetotal", "");
  DefMacro!("\\thelownamepenalty", "");
  DefMacro!("\\themaxextraalpha", "");
  DefMacro!("\\themaxextrayear", "");
  DefMacro!("\\themaxitems", "");
  DefMacro!("\\themaxnames", "");
  DefMacro!("\\themaxparens", "");
  DefMacro!("\\theminitems", "");
  DefMacro!("\\theminnames", "");
  DefMacro!("\\theabbrvpenalty", "");
  DefMacro!("\\thecitecount", "");
  DefMacro!("\\thecitetotal", "");
  DefMacro!("\\thehighnamepenalty", "");
  DefMacro!("\\theinstcount", "");
  DefMacro!("\\thelistcount", "");
  DefMacro!("\\theliststart", "");
  DefMacro!("\\theliststop", "");
  DefMacro!("\\thelisttotal", "");

  // Perl L673-688: print*/index*/entry* (all empty)
  DefMacro!("\\printtext[]{}", "");
  DefMacro!("\\printfield[]{}", "");
  DefMacro!("\\printlist[][]{}", "");
  DefMacro!("\\printnames[][]{}", "");
  DefMacro!("\\printtime", "");
  DefMacro!("\\printdate", "");
  DefMacro!("\\printdateextra", "");
  DefMacro!("\\printlabeldate", "");
  DefMacro!("\\printlabeldateextra", "");
  DefMacro!("\\printfile[]{}", "");
  DefMacro!("\\indexfield[]{}", "");
  DefMacro!("\\indexlist[][]{}", "");
  DefMacro!("\\indexnames[][]{}", "");
  DefMacro!("\\entrydata OptionalMatch:* {}{}", "");
  DefMacro!("\\entryset{}{}", "");
  DefMacro!("\\setunit OptionalMatch:* {}", "");

  // Perl L690-705
  DefMacro!("\\mkbibendnote{}", "");
  DefMacro!("\\mkbibendnotetext{}", "");
  DefMacro!("\\mkbibfootnote", "\\footnote");
  DefMacro!("\\mkbibfootnotetext", "\\footnotetext");
  DefMacro!("\\mkbibbrackets{}", "\\begingroup\\bibopenbracket#1\\bibclosebracket\\endgroup");
  DefMacro!("\\bibopenparen", "\\bibleftparen");
  DefMacro!("\\bibcloseparen", "\\bibrightparen");
  DefMacro!("\\bibopenbracket", "\\bibleftbracket");
  DefMacro!("\\bibclosebracket", "\\bibrightbracket");
  DefMacro!("\\bibleftparen", "\\blx@postpunct(");
  DefMacro!("\\bibrightparen", "\\blx@postpunct)\\midsentence");
  DefMacro!("\\bibleftbracket", "\\blx@postpunct[");
  DefMacro!("\\bibrightbracket", "\\blx@postpunct]\\midsentence");
  // Perl L704: redefine \blx@postpunct to \relax (overrides L66 \@empty).
  DefMacro!("\\blx@postpunct", "\\relax");
  DefMacro!("\\midsentence", "\\relax");

  // Perl L707-708
  DefMacro!("\\pagenote{}", "");
  DefMacro!("\\pagenotetext{}", "");

  // Perl L710-721
  DefMacro!("\\blx@uniquename", "false");
  DefMacro!("\\blx@uniquelist", "false");
  DefMacro!("\\blx@maxbibnames", "0");
  DefMacro!("\\blx@minbibnames", "0");
  DefMacro!("\\blx@maxcitenames", "0");
  DefMacro!("\\blx@mincitenames", "0");
  DefMacro!("\\blx@maxsortnames", "0");
  DefMacro!("\\blx@minsortnames", "0");
  DefMacro!("\\blx@maxalphanames", "0");
  DefMacro!("\\blx@minalphanames", "0");
  DefMacro!("\\blx@maxitems", "0");
  DefMacro!("\\blx@minitems", "0");

  // Perl L724-734: blx-internal counter registers
  DefRegister!("\\blx@tempcnta" => Number(0));
  DefRegister!("\\blx@tempcntb" => Number(0));
  DefRegister!("\\blx@tempcntc" => Number(0));
  DefRegister!("\\blx@maxsection" => Number(0));
  DefRegister!("\\blx@notetype" => Number(0));
  DefRegister!("\\blx@parenlevel@text" => Number(0));
  DefRegister!("\\blx@parenlevel@foot" => Number(0));
  // Note: `\blx@maxsegment@0` and `\blx@sectionciteorder@0` are CS names
  // with a trailing digit, which the prototype parser's CS regex
  // (`\\[a-zA-Z@]+`) cannot match — leftover `0` would then be parsed as
  // an unknown parameter type. Use the `(cs, None, value)` form so the
  // parser is skipped: a Token is built directly via `T_CS!` and no
  // parameter parsing occurs. Mirrors Perl's L731-732 register names exactly.
  DefRegister!(T_CS!("\\blx@maxsegment@0"), None, Number(0));
  DefRegister!(T_CS!("\\blx@sectionciteorder@0"), None, Number(0));
  DefRegister!("\\blx@entrysetcounter" => Number(0));
  DefRegister!("\\blx@biblioinstance" => Number(0));

  // Perl L736-801: trailing RawTeX with 9 \newbool + 60 \newtoggle
  // declarations. EXACT order and content from the Perl source.
  RawTeX!(r#"
\newbool{refcontextdefaults}
\booltrue{refcontextdefaults}%
\newbool{sourcemap}
\newbool{citetracker}
\newbool{pagetracker}
\newbool{backtracker}
\newbool{citerequest}
\booltrue{citerequest}
\newbool{sortcites}
\newtoggle{blx@bbldone}
\newtoggle{blx@tempa}
\newtoggle{blx@tempb}
\newtoggle{blx@runltx}
\newtoggle{blx@runbiber}
\newtoggle{blx@block}
\newtoggle{blx@unit}
\newtoggle{blx@skipentry}
\newtoggle{blx@insert}
\newtoggle{blx@lastins}
\newtoggle{blx@keepunit}
\newtoggle{blx@bibtex}
\newtoggle{blx@debug}
\newtoggle{blx@sortcase}
\newtoggle{blx@sortupper}
\newtoggle{blx@autolangbib}
\newtoggle{blx@autolangcite}
\newtoggle{blx@clearlang}
\newtoggle{blx@defernumbers}
\newtoggle{blx@omitnumbers}
\newtoggle{blx@footnote}
\newtoggle{blx@labelalpha}
\newtoggle{blx@labelnumber}
\newtoggle{blx@labeltitle}
\newtoggle{blx@labeltitleyear}
\newtoggle{blx@labeldateparts}
\newtoggle{blx@natbib}
\newtoggle{blx@mcite}
\newtoggle{blx@loadfiles}
\newtoggle{blx@sortsets}
\newtoggle{blx@crossrefsource}
\newtoggle{blx@xrefsource}
\newtoggle{blx@terseinits}
\newtoggle{blx@useprefix}
\newtoggle{blx@addset}
\newtoggle{blx@setonly}
\newtoggle{blx@dataonly}
\newtoggle{blx@skipbib}
\newtoggle{blx@skipbiblist}
\newtoggle{blx@skiplab}
\newtoggle{blx@citation}
\newtoggle{blx@volcite}
\newtoggle{blx@bibliography}
\newtoggle{blx@citeindex}
\newtoggle{blx@bibindex}
\newtoggle{blx@localnumber}
\newtoggle{blx@refcontext}
\newtoggle{blx@noroman}
\newtoggle{blx@nohashothers}
\newtoggle{blx@nosortothers}
\newtoggle{blx@singletitle}
\newtoggle{blx@uniquebaretitle}
\newtoggle{blx@uniqueprimaryauthor}
\newtoggle{blx@uniquetitle}
\newtoggle{blx@uniquework}
"#);
});
