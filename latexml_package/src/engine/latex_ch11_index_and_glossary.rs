use crate::prelude::*;

/// Perl: addIndexPhraseKey — sets the `key` attribute on index/glossary phrase
/// nodes from their text content, applying CleanIndexKey normalization.
fn add_index_phrase_key(node: &mut Node) -> Result<()> {
  if node.get_attribute("key").is_none() {
    let text = node.get_content();
    let key = clean_index_key(&text);
    if !key.is_empty() {
      node.set_attribute("key", &key)?;
    }
  }
  Ok(())
}

/// Perl: doIndexItem — open/close index list levels.
fn do_index_item(document: &mut Document, level: i64) -> Result<()> {
  if document.is_closeable("ltx:indexrefs").is_some() {
    document.close_element("ltx:indexrefs")?;
  }
  // closeIndexPhrase
  if document.is_closeable("ltx:indexphrase").is_some() {
    document.close_element("ltx:indexphrase")?;
  }
  let current_level = state::lookup_int("INDEXLEVEL");
  let mut l = current_level;
  while l < level {
    document.open_element("ltx:indexlist", None, None)?;
    l += 1;
  }
  while l > level {
    document.close_element("ltx:indexlist")?;
    l -= 1;
  }
  state::assign_value("INDEXLEVEL", Stored::Int(l), Some(Scope::Local));
  if level > 0 {
    document.open_element("ltx:indexentry", None, None)?;
    document.open_element("ltx:indexphrase", None, None)?;
  }
  Ok(())
}

/// Perl: CleanIndexKey — trim whitespace, remove trailing punctuation.
fn clean_index_key(key: &str) -> String {
  let key = key.trim();
  key.trim_end_matches(['.', ',', ';']).to_string()
}

/// Perl: process_index_phrases — expand \index{a!b@c|see{d}} into
/// \@index{\@indexphrase{a}\@indexphrase[c]{b}} etc.
///
/// Port of latex_constructs.pool.ltxml L4528-4591
fn process_index_phrases(tokens: Tokens) -> Result<Tokens> {
  let token_list = tokens.unlist();
  if token_list.is_empty() {
    return Ok(Tokens::new(vec![]));
  }

  // Add terminal ! if not present
  let mut toks = token_list;
  if toks.last().map(|t| t.with_str(|s| s != "!")).unwrap_or(true) {
    toks.push(T_OTHER!("!"));
  }

  let mut expansion: Vec<Token> = Vec::new();
  let mut phrase: Vec<Token> = Vec::new();
  let mut sortas: Vec<Token> = Vec::new();
  let mut style: Option<String> = None;
  let mut i = 0;

  while i < toks.len() {
    let tok = toks[i];
    let s = tok.with_str(|s| s.to_string());
    i += 1;

    if s == "\"" && i < toks.len() {
      // Escaped character: take next token literally
      phrase.push(toks[i]);
      i += 1;
    } else if s == "@" {
      // Sort key: everything before @ is the sort key
      while phrase.last().map(|t| t.with_str(|s| s.trim().is_empty())).unwrap_or(false) {
        phrase.pop();
      }
      sortas = phrase;
      phrase = Vec::new();
    } else if s == "!" || s == "|" {
      // End of phrase
      while phrase.last().map(|t| t.with_str(|s| s.trim().is_empty())).unwrap_or(false) {
        phrase.pop();
      }
      if !phrase.is_empty() {
        expansion.push(T_CS!("\\@indexphrase"));
        if !sortas.is_empty() {
          expansion.push(T_OTHER!("["));
          expansion.append(&mut sortas);
          expansion.push(T_OTHER!("]"));
        }
        expansion.push(T_BEGIN!());
        expansion.append(&mut phrase);
        expansion.push(T_END!());
      }
      sortas.clear();

      if s == "|" {
        // Collect remaining tokens as style/see/seealso
        if i < toks.len() && toks.last().map(|t| t.with_str(|s| s == "!")).unwrap_or(false) {
          // Remove terminal ! stopbit
          toks.pop();
        }
        let extra: String = toks[i..].iter().map(|t| t.with_str(|s| s.to_string())).collect();
        if extra.starts_with("see{") || extra.starts_with("see {") {
          // \@indexsee{content}
          // Skip "see{", collect until "}"
          expansion.push(T_CS!("\\@indexsee"));
          // Find the content between { and }
          let content = extra.trim_start_matches("see").trim();
          let content = content.strip_prefix('{').unwrap_or(content);
          let content = content.strip_suffix('}').unwrap_or(content);
          expansion.push(T_BEGIN!());
          expansion.extend(Explode!(content));
          expansion.push(T_END!());
        } else if extra.starts_with("seealso{") || extra.starts_with("seealso {") {
          expansion.push(T_CS!("\\@indexseealso"));
          let content = extra.trim_start_matches("seealso").trim();
          let content = content.strip_prefix('{').unwrap_or(content);
          let content = content.strip_suffix('}').unwrap_or(content);
          expansion.push(T_BEGIN!());
          expansion.extend(Explode!(content));
          expansion.push(T_END!());
        } else if extra == "(" {
          style = Some("rangestart".to_string());
        } else if extra == ")" {
          style = Some("rangeend".to_string());
        } else if !extra.is_empty() {
          // Style name (e.g., textbf → bold)
          style = Some(match extra.as_str() {
            "textbf" | "bf" => "bold".to_string(),
            "textit" | "it" | "emph" => "italic".to_string(),
            "textrm" | "rm" => String::new(),
            other => other.to_string(),
          });
        }
        break; // Consumed everything after |
      }
    } else if phrase.is_empty() && s.trim().is_empty() {
      // Skip leading whitespace
    } else {
      phrase.push(tok);
    }
  }

  // Wrap in \@index[style]{...}
  let mut result = vec![T_BEGIN!(), T_CS!("\\normalfont"), T_CS!("\\@index")];
  if let Some(ref sty) = style {
    if !sty.is_empty() {
      result.push(T_OTHER!("["));
      result.extend(Explode!(sty));
      result.push(T_OTHER!("]"));
    }
  }
  result.push(T_BEGIN!());
  result.extend(expansion);
  result.push(T_END!());
  result.push(T_END!());
  Ok(Tokens::new(result))
}

LoadDefinitions!({
  Tag!("ltx:indexphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });
  Tag!("ltx:glossaryphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });

  // \@index[style][inlist]{phrases} → <ltx:indexmark>
  DefConstructor!("\\@index[][]{}", "^<ltx:indexmark style='#1' inlist='#2'>#3</ltx:indexmark>",
    bounded => true,
    mode => "restricted_horizontal",
    sizer => 0
  );

  // \@indexphrase[sortkey]{phrase} → <ltx:indexphrase>
  DefConstructor!("\\@indexphrase[]{}", "<ltx:indexphrase key='#key'>#2</ltx:indexphrase>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      if key.is_empty() {
        Ok(stored_map!())
      } else {
        Ok(stored_map!("key" => key))
      }
    }
  );

  // \@indexsee{key} → <ltx:indexsee>
  DefConstructor!("\\@indexsee{}", "<ltx:indexsee key='#key'>#1</ltx:indexsee>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    }
  );

  // \@indexseealso{key} → <ltx:indexsee>
  DefConstructor!("\\@indexseealso{}", "<ltx:indexsee key='#key'>#1</ltx:indexsee>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    }
  );

  // \index{phrases} — expand to \@index via process_index_phrases
  DefMacro!("\\index {}", sub[(phrases)] {
    process_index_phrases(Tokens::new(phrases.revert()))
  });

  DefMacro!("\\indexname", "Index");
  DefEnvironment!("{theindex}",
    "<ltx:index xml:id='#id'>#body</ltx:index>");

  DefPrimitive!("\\indexspace", None);
  DefPrimitive!("\\makeindex", None);
  DefPrimitive!("\\makeglossary", None);
  // Perl: DefMacro('\printindex', '\@printindex');
  // \printindex and \@printindex produce a stub index TOC
  DefConstructor!("\\printindex", "<ltx:index xml:id='#id' lists='idx'><ltx:title>#name</ltx:title></ltx:index>",
    properties => {
      Ok(stored_map!("name" => stomach::digest(T_CS!("\\indexname"))?))
    }
  );

  // Perl: \glossary{} — simplified glossary entry
  DefConstructor!("\\glossary{}", "<ltx:glossaryphrase role='glossary' key='#key'>#1</ltx:glossaryphrase>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    },
    sizer => 0
  );

  DefMacro!("\\glossaryname", "Glossary");
  DefConstructor!("\\printglossary",
    "<ltx:glossary xml:id='#id'><ltx:title>#name</ltx:title></ltx:glossary>",
    properties => {
      Ok(stored_map!("name" => stomach::digest(T_CS!("\\glossaryname"))?))
    }
  );

  DefMacro!("\\seename", "see");
  DefMacro!("\\alsoname", "see also");

  //======================================================================
  // Perl: latex_constructs.pool.ltxml L4536-4564 — index constructors

  // Helper: close an open indexphrase element
  // closeIndexPhrase + doIndexItem
  DefConstructor!("\\index@dotfill", sub[document] {
    if document.is_closeable("ltx:indexphrase").is_some() {
      document.close_element("ltx:indexphrase")?;
    }
    document.open_element("ltx:indexrefs", None, None)?;
  });

  DefConstructor!("\\index@item", sub[document] {
    do_index_item(document, 1)?;
  });
  DefConstructor!("\\index@subitem", sub[document] {
    do_index_item(document, 2)?;
  });
  DefConstructor!("\\index@subsubitem", sub[document] {
    do_index_item(document, 3)?;
  });
  DefConstructor!("\\index@done", sub[document] {
    do_index_item(document, 0)?;
  });
});
