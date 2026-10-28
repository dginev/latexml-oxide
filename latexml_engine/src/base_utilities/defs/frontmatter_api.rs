//! `LoadDefinitions!` part 2: the frontmatter API (`\lx@personname`, the `Frontmatter` keyvals, `\lx@add@frontmatter*`,
//! `\lx@annotate@frontmatter*`, labels).

use super::*;

pub(in crate::base_utilities) fn load() -> Result<()> {
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // General support for Front Matter. (PR #2767 rework)
  // Not (yet) used by TeX (finish plain?)
  // But provides support for LaTeX (and other formats?) for handling frontmatter.
  //
  // The idea is to accumulate any frontmatter material (title, author,...)
  // rather than directly drop it into the digested stream.
  // When we begin constructing the document, all accumulated material is output.
  // See LaTeX.ltxml for usage.
  // Note: could be circumstances where you'd want modular frontmatter?
  // (ie. frontmatter for each sectional unit)

  // Perl: DebuggableFeature('frontmatter'); enable with `--debug frontmatter`.
  debuggable_feature("frontmatter");

  // Perl Base_Utility.pool.ltxml L219-222 (PR #2767): moved here from
  // latex_constructs (was \@personname). Perl's beforeDigest rebinds
  // `\thanks` → `\lx@add@thanks` so an author's `\thanks{...}` becomes a
  // role=thanks contact via \lx@annotate@frontmatter@now (which applies the
  // `\lx@contact@thanks@name` default, "Thanks: "). The earlier Rust port
  // mis-bound it to the now-removed `\person@thanks` constructor, which built
  // a bare <contact role=thanks> with no name; faithful binding restored.
  DefConstructor!("\\lx@personname{}", "<ltx:personname>#1</ltx:personname>",
    before_digest => { Let!("\\thanks", "\\lx@add@thanks"); },
    bounded => true,
    mode => "text",
    enter_horizontal => true
  );
  DefMacro!("\\lx@ignore@tabular[]{}", "");
  DefMacro!("\\lx@ignore@endtabular", "");

  // Sanitize person names for (obvious) punctuation abuse at start+end
  // (moved here from latex_constructs per PR #2767; the existing Rust
  // punctuation-strip port is carried over).
  Tag!("ltx:personname", after_close => sub[document, node] {
    // Normalize a whole-name bold BEFORE the punctuation strip, so the unwrapped
    // text is punctuation-cleaned like a plain name (html_feedback#61).
    unwrap_whole_name_bold(document, node)?;
    if let Some(mut first) = node.get_first_child() {
      if first.get_type() == Some(NodeType::TextNode) {
        let first_text = first.get_content();
        let mut first_text_iter = first_text.chars().peekable();
        while let Some(peeked) = first_text_iter.peek() {
          if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
            first_text_iter.next();
          } else {
            break;
          }
        }
        let new_text = first_text_iter.collect::<String>();
        if first_text != new_text {
          first.set_content(&new_text)?;
        }
      }
      if let Some(mut last) = node.get_last_child()
        && last.get_type() == Some(NodeType::TextNode) {
          let last_text = last.get_content();
          let mut last_text_iter  = last_text.chars().rev().peekable();
          while let Some(peeked) = last_text_iter.peek() {
            if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
              last_text_iter.next();
            } else {
              break;
            }
          }
          let mut new_text = last_text_iter.rev().collect::<String>();
          // Beyond Perl (KNOWN_PERL_ERRORS #523): the period of an initial or of a suffix written with one is the
          // name's (`Sridhar K.`, hep-ph9306209; `John Smith Jr.`)
          if last_text[new_text.len()..].starts_with('.') && ends_with_abbreviation(&new_text) {
            new_text.push('.');
          }
          if last_text != new_text {
            last.set_content(&new_text)?;
          }
        }
    }
  });

  //======================================================================
  // Perl Base_Utility.pool.ltxml L161
  AssignValue!(
    "frontmatter",
    Stored::HashTagData(HashMap::default()),
    Some(Scope::Global)
  );

  // Perl Base_Utility.pool.ltxml L163
  DefConditional!("\\if@in@preamble", { lookup_bool_sym(pin!("inPreamble")) });

  DefKeyVal!("Frontmatter", "role", "Semiverbatim");
  DefKeyVal!("Frontmatter", "class", "Semiverbatim");
  DefKeyVal!("Frontmatter", "graphic", "Semiverbatim");
  DefKeyVal!("Frontmatter", "annotations", "");
  DefKeyVal!("Frontmatter", "label", "");
  DefKeyVal!("Frontmatter", "labelref", "Semiverbatim");
  DefKeyVal!("Frontmatter", "labelseq", "");
  DefKeyVal!("Frontmatter", "annotate", "");
  // Rust-only: an entry of a replaceable tag that adds to the ones before instead of replacing them (acmart's
  // `\received` history, acmart.cls:1859-1872: several revised dates; 2307.05988), honoured by
  // `\lx@add@frontmatter@now` and `@until` ([`takes_accumulate`])
  DefKeyVal!("Frontmatter", "accumulate", "");
  DefKeyVal!("Frontmatter", "name", "");
  // Rust-only: the RDFa pair of an `ltx:rdf` annotation ([`\lx@add@creator@rdf`]; OXIDIZED_DESIGN #479). Semiverbatim, so an
  // address's `_` and `~` are characters, not a script and a tie.
  DefKeyVal!("Frontmatter", "property", "Semiverbatim");
  DefKeyVal!("Frontmatter", "content", "Semiverbatim");

  // \lx@clear@frontmatter{tag}[kv]
  // Remove all pending frontmatter element matching $tag, and role (if given) in keyvals
  DefPrimitive!("\\lx@clear@frontmatter {} OptionalKeyVals:Frontmatter", sub[(tag, kv)] {
    let role = kv.as_ref()
      .and_then(|kv| kv.get_value("role"))
      .map(|v| v.to_string())
      .filter(|r| !r.is_empty());
    match role {
      Some(ref role) => dequeue_front_matter(&tag.to_string(), &[("role", role)]),
      None => dequeue_front_matter(&tag.to_string(), &[]),
    }
  });

  // Remove all creators with given role (default author)
  DefMacro!(
    "\\lx@clear@creators []",
    "\\lx@clear@frontmatter{ltx:creator}[#1]"
  );

  // The various \lx@add@<frontmatter> commands
  //  (1) queue the command (appending @now) in the frontmatter_raw state variable
  //    to defer digestion (& possible replacement)
  //  (2) when the @now form is digested (see digest_front_matter)
  //    will add an entry to the frontmatter hash state variable.
  //    That hash is keyed by the tag, with values contains a list of
  //      [tag, {attr}, @content]
  //    to create an element <tag> with the given attributes and content.
  //    Each content item is either a Box (List,Whatsit)
  //    or recursively an array [tag,{attr},@content].

  // See clean_trailing_break for cleanup of misused \\.
  Tag!("ltx:personname", after_close => sub[document, node] {
    clean_trailing_break(document, node)?;
  });
  Tag!("ltx:contact", after_close => sub[document, node] {
    clean_trailing_break(document, node)?;
  });

  // Add a new frontmatter item that will be enclosed in <$tag %attr>...</$tag>
  // The content is the result of digesting $tokens.
  // \lx@add@frontmatter[keys]{tag}[attributes]{content}
  // keys can have
  //   replace (to replace the current entry, if any)
  //   ifnew   (only add if no previous entry)
  DefPrimitive!("\\lx@add@frontmatter OptionalKeyVals {} OptionalKeyVals {}",
    sub[(keys_opt,tag_tks,attrs_opt,tokens)] {
    // Beyond-Perl hardening (OXIDIZED_DESIGN #51; general frontmatter principle,
    // user-directed 2026-07-07): a frontmatter entry with an empty tag or empty
    // content is VOID — early-quit, emit nothing. Perl's `\lx@add@frontmatter`
    // queues unconditionally (L354-358), so a binding fed an empty argument
    // yields a stray empty element: e.g. icml's `\printAffiliationsAndNotice{}`
    // (empty braces = the ICML-sanctioned "no notice") -> an empty
    // `<ltx:note role="affiliationnotice">` rendering as a bare "affiliationnotice:"
    // marker (witness arXiv:2606.00309). The affiliation list itself survives —
    // it is fed separately via `\icmlaffiliation`.
    if tag_tks.to_string().trim().is_empty() || tokens.to_string().trim().is_empty() {
      return Ok(Vec::new());
    }
    queue_add_frontmatter_now(keys_opt.as_ref(), &tag_tks, attrs_opt.as_ref(), Some(&tokens))?;
  });

  // Beyond-Perl best-practice API (OXIDIZED_DESIGN #51): open an *empty*
  // frontmatter container element as a deliberate anchor for later annotations
  // — the intention-revealing counterpart to `\lx@add@frontmatter`, which is now
  // a no-op on empty content. A few bindings legitimately need an empty
  // container: moderncv's cv `<ltx:creator>` exists only to receive the
  // \firstname/\familyname/\email/... contacts that annotate the most-recent
  // creator. (Perl abuses `\lx@add@frontmatter{ltx:creator}[role=cv]{}` for
  // this; we name the intent instead of smuggling it through empty content.)
  // \lx@add@frontmatter@container[keys]{tag}[attributes]
  DefPrimitive!("\\lx@add@frontmatter@container OptionalKeyVals {} OptionalKeyVals",
    sub[(keys_opt,tag_tks,attrs_opt)] {
    queue_add_frontmatter_now(keys_opt.as_ref(), &tag_tks, attrs_opt.as_ref(), None)?;
  });

  DefPrimitive!("\\lx@add@frontmatter@now OptionalKeyVals {} OptionalKeyVals:Frontmatter {}",
    sub[(_obsoletekeys, tag_tks, kv, content)] {
    let tag = tag_tks.to_string();
    // %options = digested keyvals hash; role read from the UNdigested keyvals
    // (Careful! Multiple values — Perl getValue returns the last one.)
    let mut options = TagAttrs::default();
    let mut role = String::new();
    if let Some(kv) = kv {
      role = kv.get_value("role").map(|v| v.to_string()).unwrap_or_default();
      if let DigestedData::KeyVals(dkv) = kv.be_digested()?.data() {
        for key in dkv.get_keyvals().keys() {
          if let Some(v) = dkv.get_value_digested(key) {
            options.insert(key.clone(), v.to_string());
          }
        }
      }
    }
    let accumulate = takes_accumulate(&mut options);
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    // Replaceable tags (title/toctitle/subtitle/date) keep only one entry — a later
    // one replaces earlier ones. Ported from upstream `%ReplaceableFrontmatterTags`
    // (the vendored copy pushes unconditionally → duplicate <title> when a document
    // re-adds it, e.g. arXiv 2002.09766's appendix `\icmltitle`). OXIDIZED_DESIGN #154.
    // An `accumulate` entry adds to the ones before (acmart's `\received` history, 2307.05988).
    let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate;
    // …unless the document body began after the one it would replace: that one is the paper's, this a supplement's
    // ([`seal_digested_frontmatter`]), dropped — with the authors a supplement's frontmatter lists — before it is counted
    // (a later flush's last-author conjunction reads the count).
    let seal = frontmatter_seal_key(&tag, options.get("name").map(String::as_str), &role, replaceable);
    if let Some(ref key) = seal
      && frontmatter_sealed(key)
    {
      Info!("frontmatter", "sealed", s!("{tag} after the document body began is a supplement's; the paper's is kept"));
      // …and the annotations this flush attaches to it, which would otherwise land on the paper's.
      let sealed_off = lookup_string("lx_frontmatter_sealed_off");
      assign_value(
        "lx_frontmatter_sealed_off",
        Stored::String(pin(s!("{sealed_off}\u{1}{tag}"))),
        Some(Scope::Global),
      );
      return Ok(Vec::new());
    }
    // extract (possibly multiple!) labels
    let mut labels = clean_frontmatter_labels(
      options.get("annotations").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = lookup_mapping_int(&s!("num_{tag}"), &role) + 1;
      assign_mapping(&s!("num_{tag}"), &role, Some(Stored::Int(n)));
      options.insert("role".to_string(), role.clone());
      options.insert("_num".to_string(), n.to_string());
      // record sequence position as potential attachment label
      labels.push(clean_label(&n.to_string(), Some(&role)).into_owned());
    }
    options.insert("_annotations".to_string(), labels.join(","));
    let entry = TagData {
      tag: tag.clone(),
      attr: options,
      content: vec![TagContent::PlaceKeeper], // (in case embedded)
    };
    DebugFeature!("frontmatter", "FRONT Add {}\n   for: {}",
      show_frontmatter(&entry), content);
    if replaceable {
      frontmatter_clear_same_name(
        &tag,
        entry.attr.get("name").map(String::as_str),
        entry.attr.get("xml:lang").map(String::as_str),
        entry.attr.get("_store").map(String::as_str),
      );
    }
    let index = frontmatter_push(&tag, entry);
    // REPLACE only 'place_keeper'!!
    let digested = digest_frontmatter_item(&tag, content)?;
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
    if let Some(key) = seal {
      note_digested_frontmatter(&key);
    }
    Ok(Vec::new())
  }, bounded => true);

  // This is a variant of \lx@add@frontmatter which digests immediately
  // until a terminator token;
  // It is useful for frontmatter environments, like {abstract}
  // (expanding until \end{abstract} generally gets tangled by contents which expect
  // digestion and side effects).
  DefPrimitive!("\\lx@add@frontmatter@until {} OptionalKeyVals:Frontmatter DefToken",
    sub[(tag_tks, kv, end)] {
    let tag = tag_tks.to_string();
    let mut options = TagAttrs::default();
    let mut role = String::new();
    if let Some(kv) = kv {
      role = kv.get_value("role").map(|v| v.to_string()).unwrap_or_default();
      if let DigestedData::KeyVals(dkv) = kv.be_digested()?.data() {
        for key in dkv.get_keyvals().keys() {
          if let Some(v) = dkv.get_value_digested(key) {
            options.insert(key.clone(), v.to_string());
          }
        }
      }
    }
    let accumulate = takes_accumulate(&mut options);
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate;
    // (sealed whether or not the blocks accumulate: the IEEEtran and VTeX keyword blocks add to the ones before them,
    // and a supplement's after the body began is still its own — 2604.23289's `{IEEEkeywords}` under its "Supplemental
    // File" section, 77t)
    let seal = frontmatter_seal_key(
      &tag,
      options.get("name").map(String::as_str),
      &role,
      REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()),
    );
    // An abstract (or keywords) after the document body began, where the paper's own came before it, is a
    // supplement's: its text stays where it is, as body paragraphs (the schema admits `ltx:abstract` only in a
    // document, sidebar, bibliography or titlepage, LaTeXML-structure.rnc:668-673), and the paper's stays at the head
    // ([`seal_digested_frontmatter`]; 2301.10468, 2401.11672, 2405.10566, 2503.16707).
    if let Some(ref key) = seal
      && frontmatter_sealed(key)
    {
      Info!("frontmatter", "sealed", s!("{tag} after the document body began is a supplement's; kept in place"));
      // (read by what the body defines for itself: a class's in-abstract `\keywords` stays in place with it,
      // frontmatter_stores.rs `\lx@abstract@class@setters` — mtt.cls papers' English and Ukrainian abstracts at
      // their end: 1310.0585, 1310.0589, 1212.3890)
      assign_value("lx_frontmatter_kept_in_place", true, Some(Scope::Local));
      return digest_next_body(Some(end));
    }
    // extract (possibly multiple!) labels
    let mut labels = clean_frontmatter_labels(
      options.get("annotations").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = lookup_mapping_int(&s!("num_{tag}"), &role) + 1;
      assign_mapping(&s!("num_{tag}"), &role, Some(Stored::Int(n)));
      options.insert("role".to_string(), role.clone());
      options.insert("_num".to_string(), n.to_string());
      // record sequence position as potential attachment label
      labels.push(clean_label(&n.to_string(), Some(&role)).into_owned());
    }
    options.insert("_annotations".to_string(), labels.join(","));
    let entry = TagData {
      tag: tag.clone(),
      attr: options,
      content: vec![TagContent::PlaceKeeper], // (in case embedded)
    };
    // Replaceable tags (abstract/keywords) keep only one entry — but ONLY dedup when no
    // same-tag `@until` is already in progress (open PlaceKeeper), so a nested/malformed
    // `\begin{abstract}\begin{abstract}…` isn't corrupted by clearing a parent's still-
    // open entry. OXIDIZED_DESIGN #154. Witness 2511.21969 (nested abstract env).
    if replaceable && !frontmatter_has_open_placekeeper(&tag) {
      frontmatter_clear_same_name(
        &tag,
        entry.attr.get("name").map(String::as_str),
        entry.attr.get("xml:lang").map(String::as_str),
        entry.attr.get("_store").map(String::as_str),
      );
    }
    let index = frontmatter_push(&tag, entry);
    // the body's own paragraph breaks are the field's (`frontmatter_paragraph_break`); the enclosing notes come back
    // after it
    let enclosing = (lookup_value("lx_frontmatter_par_item"), lookup_value("lx_frontmatter_par_depth"));
    note_frontmatter_paragraphs(&tag);
    // A terminal reached INSIDE a nested group (`\section`'s
    // `\@startsection@hook` inside an unbalanced `\abstract{…`) ends the
    // body from there: `digest_next_body` records the terminal and depth on
    // this frame, `until_terminal_inside_group` (stomach.rs) acts.
    let body = digest_next_body(Some(end));
    assign_value("lx_frontmatter_par_item", enclosing.0.unwrap_or(Stored::None), Some(Scope::Local));
    assign_value("lx_frontmatter_par_depth", enclosing.1.unwrap_or(Stored::None), Some(Scope::Local));
    let digested = Digested::from(List::new(body?));
    // A body that typesets nothing adds no entry: an abstract (keywords, classification) with no authored content is
    // no frontmatter, whatever ended it — a heading that closed it before its first line (scrartcl's
    // `\begin{abstract}\section*{Abstract:} …`, 1507.08575; user ruling 2026-10-18, sect05.rs
    // `\lx@heading@ends@frontmatter`) or an environment left empty (pdflatex prints only its heading).
    if !typesets_content(&digested) {
      frontmatter_remove(&tag, index);
      return Ok(Vec::new());
    }
    DebugFeature!("frontmatter", "FRONT Add (until) {} for: {}", tag, digested);
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
    if let Some(key) = seal {
      note_digested_frontmatter(&key);
    }
    Ok(Vec::new())
  }, bounded => true);

  // Some frontmatter elements are "structured" in the sense of having a main bit of data
  // and several optional extra bits.  For example, LaTeX classes typically have markup
  // to define "creators" (authors, editors,etc) and a variety of markup strategies to
  // annotate them with "contacts" (affiliation, email, etc)
  // In the easy case, that markup is embedded within \author. Otherwise, it appears
  // separately and will be "attached" to the most recent creator,
  //
  // The \lx@annotate@frontmatter command is used to annotate some frontmatter elements,
  // with additional data. Several keywords support different attachment methods:
  //   label : $label; find the $parenttag with annotations containing $label.
  //   labelseq=$prefix : n-th $tag+$role attaches to $parenttag using label = prefix+n
  //   annotate=(all | new | <number> )
  //     all : attaches to all preceding $parenttag
  //     new : like all, but only those not yet having this type of annotation
  //     <number> : attaches to the <number>-th previous $parenttag
  //   <default> : attach to preceding $parenttag.

  // \lx@annotate@frontmatter{parenttag}{tag}[options]{content}
  // adds a tag element, containing content, to an appropriate parenttag,
  // according to the attachment criterion in options keyvals.
  DefPrimitive!("\\lx@annotate@frontmatter {} {} OptionalKeyVals:Frontmatter {}",
    sub[(parenttag, tag, kv, content)] {
    // Perl: queueFrontMatter($stomach, ToString($tag), $kv,
    //   Invocation(T_CS('\lx@annotate@frontmatter@now'), $parenttag, $tag, $kv, $content))
    let mut inv_tokens: Vec<Token> = vec![T_CS!("\\lx@annotate@frontmatter@now")];
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(parenttag.unlist_ref().iter().copied());
    inv_tokens.push(T_END!());
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(tag.unlist_ref().iter().copied());
    inv_tokens.push(T_END!());
    if let Some(ref kv) = kv {
      inv_tokens.push(T_OTHER!("["));
      inv_tokens.extend(kv.revert()?.unlist());
      inv_tokens.push(T_OTHER!("]"));
    }
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(content.unlist());
    inv_tokens.push(T_END!());
    queue_front_matter(&tag.to_string(), kv.as_ref(), Tokens::new(inv_tokens));
  });

  DefPrimitive!("\\lx@annotate@frontmatter@now {}{} OptionalKeyVals:Frontmatter {}",
    sub[(parenttag_tks, tag_tks, kv, content)] {
    let parenttag = parenttag_tks.to_string();
    let tag = tag_tks.to_string();
    // An annotation of an entry this flush dropped as a supplement's ([`seal_digested_frontmatter`]) goes with it,
    // instead of landing on the paper's (a supplement's `\address[A]` on the paper's author labelled A).
    if lookup_string("lx_frontmatter_sealed_off").split('\u{1}').any(|sealed| sealed == parenttag) {
      return Ok(Vec::new());
    }
    let preformatted = tag == "preformatted"; // Obsolete API? $content is constructor!
    // (the annotations made so far, this one counted: one made while its content is digested is content it moved out,
    // for the note below)
    let annotations = lookup_int("lx_frontmatter_annotations") + 1;
    assign_value("lx_frontmatter_annotations", Stored::Int(annotations), Some(Scope::Global));
    let mut options = TagAttrs::default();
    if let Some(kv) = kv
      && let DigestedData::KeyVals(dkv) = kv.be_digested()?.data() {
        for key in dkv.get_keyvals().keys() {
          if let Some(v) = dkv.get_value_digested(key) {
            options.insert(key.clone(), v.to_string());
          }
        }
      }
    // Perl: $role = $options{role} = ToString($options{role}) — digested value here
    let role = options.get("role").cloned().unwrap_or_default();
    options.insert("role".to_string(), role.clone());
    let mut labels = clean_frontmatter_labels(
      options.get("label").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = lookup_mapping_int(&s!("num_{tag}"), &role) + 1;
      assign_mapping(&s!("num_{tag}"), &role, Some(Stored::Int(n)));
      if let Some(labelseq) = options.get("labelseq")
        && !labelseq.is_empty() {
          labels.push(clean_label(&n.to_string(), Some(labelseq)).into_owned());
        }
    }
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #805): a contact of a sequenced list (`\lx@add@contact@placed[owner=sequence]`)
    // goes where this flush placed it ([`sequenced_contact_labels`]), else where `\lx@add@contact@placed` did.
    if options.remove("_sequence").is_some() {
      let at = lookup_int("lx_frontmatter_sequence_at");
      assign_value("lx_frontmatter_sequence_at", Stored::Int(at + 1), Some(Scope::Global));
      let placed = lookup_string("lx_frontmatter_sequence_labels");
      if let Some(label) = placed
        .split('\u{1}')
        .nth(usize::try_from(at).unwrap_or(usize::MAX))
        .filter(|label| !label.is_empty())
      {
        labels = vec![label.to_string()];
      }
    }
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    options.insert("_label".to_string(), labels.join(","));

    // Snapshot the parent entries (those not role=pending), inherit labels
    // from an enclosing pending entry, and push a tentative stub entry —
    // in case digestion changes labels!
    let (parent_indices, stub_idx) = with_value_mut("frontmatter", |val_opt| {
      if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
        let list = frnt.entry(parenttag.clone()).or_insert_with(Vec::new);
        let parent_indices: Vec<usize> = list.iter().enumerate()
          .filter(|(_, e)| e.attr.get("role").map(String::as_str).unwrap_or("") != "pending")
          .map(|(i, _)| i)
          .collect();
        // IF this item encountered WITHIN another frontmatter
        // we inherit that frontmatter's labels (even override!)
        if let Some(last) = list.last()
          && last.attr.get("role").map(String::as_str) == Some("pending")
            && matches!(last.content.first(), Some(TagContent::PlaceKeeper))
          {
            let inherited = last.attr.get("_annotations").cloned().unwrap_or_default();
            // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #901): an e-mail inside a contact every author shares (llncs's
            // one `\institute`, `share_row_contacts`) is no author's by that: it keeps the label its owner gave it,
            // else it is the author block's — never every author's (the 85b review's p19; 2410.04091's addresses had all gone to every author).
            let shares = inherited.split(',').any(|label| label == SHARED_ROW_LABEL);
            let label = match options.get("_label") {
              _ if !shares => inherited,
              Some(own) if role == "email" && !own.is_empty() => own.clone(),
              _ if role == "email" => "addresses:block".to_string(),
              _ => inherited,
            };
            options.insert("_label".to_string(), label);
          }
        let mut stub_attr = TagAttrs::default();
        stub_attr.insert("role".to_string(), "pending".to_string());
        stub_attr.insert("_annotations".to_string(),
          options.get("_label").cloned().unwrap_or_default());
        let stub = TagData {
          tag: parenttag.clone(),
          attr: stub_attr,
          content: vec![TagContent::PlaceKeeper],
        };
        DebugFeature!("frontmatter", "FRONT Add stub {}\n  for annotation {} [{}]",
          show_frontmatter(&stub), tag, options.get("_label").map(String::as_str).unwrap_or(""));
        list.push(stub);
        (parent_indices, list.len() - 1)
      } else {
        (Vec::new(), 0)
      }
    });
    let nparents = parent_indices.len();
    // (a contact led by a mark of its own, which labels no one — a legend, `$^\dagger$Corresponding author`, or a mark
    // no author carries — is no answer to an author's label request, nor passes over the authors one answers)
    let legend = marker_leads(&content);
    let xcontent = digest_frontmatter_item(&tag, content)?;
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #803; ruling 2026-10-21): a note whose whole content became contacts is
    // none. `\fnsep\thanks{\email{…}}` (webofc; aa's `\fnmsep\thanks{\email{…}}`) moves the e-mail into the author's
    // contact and left an empty `<note role="thanks"/>`, a visible mark opening a note with nothing in it (2409.01693;
    // 20 of 21 webofc papers). An author's own empty `\thanks{}` moved nothing out and stays a note, as Perl keeps it.
    if tag == "ltx:note" && lookup_int("lx_frontmatter_annotations") > annotations && !typesets_content(&xcontent) {
      DebugFeature!("frontmatter", "FRONT drop {tag}: its content became contacts");
      with_value_mut("frontmatter", |val_opt| {
        if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
          && let Some(list) = frnt.get_mut(&parenttag)
          && list.get(stub_idx).is_some_and(|e| e.attr.get("role").map(String::as_str) == Some("pending"))
        {
          list.remove(stub_idx);
        }
      });
      return Ok(Vec::new());
    }
    // Reset if changed (eg. by \lx@set@frontmatter@label during digestion)!
    let stub_label = with_value("frontmatter", |v| {
      if let Some(Stored::HashTagData(frnt)) = v {
        frnt.get(&parenttag)
          .and_then(|l| l.get(stub_idx))
          .and_then(|e| e.attr.get("_annotations"))
          .cloned()
      } else {
        None
      }
    }).unwrap_or_default();
    options.insert("_label".to_string(), stub_label.clone());
    let datum = if preformatted {
      TagContent::Box(xcontent)
    } else {
      TagContent::Entry(TagData {
        tag: tag.clone(),
        attr: options.clone(),
        content: vec![TagContent::Box(xcontent)],
      })
    };
    if !preformatted && (!stub_label.is_empty() || nparents == 0) {
      // deferred until we can compare labels
      DebugFeature!("frontmatter", "... deferring to label={stub_label}");
      frontmatter_set_first_content(&parenttag, stub_idx, datum);
    } else {
      let annotate = options.get("annotate").cloned().unwrap_or_default();
      // (`run:N`: the run, of the last N parents at most — authblk's `\AB@thenote` group, the authors since the last
      // `\affil`, `authblk_sty.rs`)
      let (annotate, cap) = match annotate.split_once(':') {
        Some((mode, cap)) => (mode.to_string(), cap.trim().parse::<i64>().ok()),
        None => (annotate, None),
      };
      let mut continued = false;
      // (the run's extent so far, which a contact continuing it keeps)
      let mut run_extent = 0i64;
      let (mut nprev, newonly): (i64, bool) = if annotate.is_empty() {
        (1, false)
      } else if annotate.chars().all(|c| c.is_ascii_digit()) {
        (annotate.parse().unwrap_or(1), false)
      } else if annotate == "all" {
        (nparents as i64, false)
      } else if annotate == "new" {
        (nparents as i64, true)
      } else if annotate == "run" {
        // `new`, and a second such contact before any new author (elsarticle's `\author{A}\author{B}\address{X}
        // \address{Y}`) the same run's as the one before it: past that one, the authors no longer read as new. The run is
        // keyed by the count of parents only, so a binding whose `\author` replaces the authors before it must not use it.
        let run_key = s!("{parenttag}|{role}|{nparents}");
        let last = match lookup_value("lx_frontmatter_new_run") {
          Some(Stored::String(last)) => with(last, |s| s.to_string()),
          _ => String::new(),
        };
        // (the run's record: the authors the contact before reached, and the ones Perl's `new` would have — those the
        // walk passed over as answered (below) included. A legend, `\affil{$^\dagger$Corresponding author}`, is the
        // whole run's: after `\affil{Univ Y}`, which an answered author's run passed over, it still reaches the author
        // showing its mark, a further `\affil{Univ W}` between them too (the extent is kept, below); 77n r5 review probe
        // m11, r6 review probe x1)
        let mut record = last.rsplitn(3, '|');
        match (record.next(), record.next(), record.next()) {
          (Some(extent), Some(n), Some(key)) if key == run_key => {
            continued = true;
            run_extent = extent.parse().unwrap_or(0);
            let n = if legend { extent } else { n };
            (n.parse().unwrap_or(nparents as i64), false)
          },
          _ => (nparents as i64, true),
        }
      } else {
        Info!("unexpected", &tag, s!("Frontmatter annotate '{annotate}' unrecognized"));
        (1, false)
      };
      // (a cap below 1 is none: a run of no authors would attach the contact to no one)
      if let Some(cap) = cap.filter(|cap| *cap >= 1) {
        nprev = nprev.min(cap);
      }
      DebugFeature!("frontmatter", "...adding to {nprev} previous{}",
        if newonly { " new" } else { "" });
      let (run_new, mut attached, mut passed) = (annotate == "run", 0i64, 0i64);
      let offered = if run_new && !legend {
        lookup_string("lx_frontmatter_offered_labels")
      } else {
        String::new()
      };
      with_value_mut("frontmatter", |val_opt| {
        if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
          && let Some(list) = frnt.get_mut(&parenttag) {
            // Remove unneeded (stub) entry.
            if list.get(stub_idx)
              .map(|e| e.attr.get("role").map(String::as_str) == Some("pending"))
              .unwrap_or(false)
            {
              list.remove(stub_idx);
            }
            let has_role_key = s!("_has{role}");
            // (a parent has the role already when it has such a contact; in a run, also when a mark of its own is
            // answered by a mark-picked contact of the role (`_bymark`, `\lx@affiliation@withsup`) — one deferred until
            // the labels are compared, or one this flush digests later (`lx_frontmatter_offered_labels`) — which has
            // not reached it yet: `\author{Ann$^{1}$}\affil{$^1$Univ X}\author{Bob \affil{Univ Y}}` gave Ann Y as well,
            // 77n r4 review probes e1, e4; a label nothing answers, `\author{Ann$^{1}$}\affil{$^{1-3}$Univ X}`, is none
            // (b3); a legend reads the contact only (`\affil{$^\dagger$Corresponding author}` after mark-picked ones
            // is every new author's, r3 review probe q04; 2605.11982's after `\maketitle`). A `label=` answers
            // nothing here: authblk's `\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}` prints Univ Y under
            // both (2605.25017, 77n r5 review probes q10, r07). Perl's `new` checks the contact only,
            // Base_Utility.pool.ltxml:530)
            let answering: Vec<String> = if run_new && !legend {
              list
                .iter()
                .filter(|e| {
                  e.attr.get("role").map(String::as_str) == Some("pending")
                    && e.attr.contains_key("_bymark")
                    && matches!(e.content.first(),
                      Some(TagContent::Entry(contact)) if contact.attr.get("role") == Some(&role))
                })
                .filter_map(|e| e.attr.get("_annotations"))
                .flat_map(|labels| labels.split(','))
                .map(str::to_string)
                .chain(offered.split('\u{1}').filter_map(|offer| {
                  offer
                    .split_once('\u{2}')
                    .filter(|(of, _)| *of == role)
                    .map(|(_, label)| label.to_string())
                }))
                .filter(|label| !label.is_empty())
                .collect()
            } else {
              Vec::new()
            };
            let answered = |parent: &TagData| {
              parent
                .attr
                .get("_annotations")
                .is_some_and(|labels| {
                  labels.split(',').any(|label| answering.iter().any(|a| a == label))
                })
            };
            // (the walk stops at an answered author only once it has attached the contact to an author no mark
            // answers: when the last author is answered too, every author of the run is, and the contact is the
            // whole run's as Perl's `_has{role}` test gives it — ptephy's `\affil{}{Univ Z}` after `\affil{1}{…}
            // \affil{2}{…}`, which the class prints under all of them; 77n r5 review probes t02, t04, o04)
            let mut reached_unanswered = false;
            let mut indices = parent_indices.clone();
            while let Some(pi) = indices.pop() {
              if nprev <= 0 {
                break;
              }
              nprev -= 1;
              if let Some(parent) = list.get_mut(pi) {
                attached += 1;
                parent.content.push(datum.clone());
                if !role.is_empty() {
                  reached_unanswered |= !answered(parent);
                  parent.attr.insert(has_role_key.clone(), "1".to_string());
                  let Some(&next_pi) = indices.last().filter(|_| newonly) else {
                    continue;
                  };
                  if list.get(next_pi).is_some_and(|next| next.attr.contains_key(&has_role_key)) {
                    break;
                  }
                  if reached_unanswered && list.get(next_pi).is_some_and(answered) {
                    // (the authors Perl's `new` reaches past here, for a legend of the run)
                    passed = indices
                      .iter()
                      .rev()
                      .take(nprev.max(0) as usize)
                      .take_while(|&&pi| {
                        list
                          .get(pi)
                          .is_some_and(|p| !p.attr.contains_key(&has_role_key))
                      })
                      .count() as i64;
                    break;
                  }
                }
              }
            }
          }
      });
      // (a contact that reached no one leaves the run as it was: `attached=0` would zero a later contact of the run; nor
      // does a legend within it, whose extent is not the authors the next contact continues with — one opening the run
      // starts it, `\affil{$^1$Univ X}` no author's mark answers before `\affil{Univ Y}`, r4 review probe d6. A contact
      // continuing the run keeps its extent: `\affil{Univ W}` after `\affil{Univ Y}`, which passed over the answered
      // author, reaches the unanswered ones only, and the legend after both still reaches the author showing its mark,
      // 77n r6 review probe x1)
      if run_new && attached > 0 && !(legend && continued) {
        let extent = if continued {
          run_extent.max(attached + passed)
        } else {
          attached + passed
        };
        assign_value(
          "lx_frontmatter_new_run",
          Stored::String(pin(s!("{parenttag}|{role}|{nparents}|{attached}|{extent}"))),
          Some(Scope::Global),
        );
      }
    }
  });

  // These next two are primitives executing during digestion (BEFORE XML);
  // they add to or replace labels to be used when building the frontmatter plan.

  // \lx@request@frontmatter@annotation adds additional (comma separated) labels to the
  // currently being digesting frontmatter;
  // Typically would \let\inst to this, and used within creator's content
  // to identify contacts to include.
  // Can provide the prefix to distinguish different sets of labels
  DefPrimitive!("\\lx@request@frontmatter@annotation[]{}", sub[(prefix, label)] {
    let prefix = prefix.as_ref().map(ToString::to_string).unwrap_or_default();
    let label = clean_frontmatter_labels(
      &label.to_string(),
      if prefix.is_empty() { "LABEL" } else { &prefix }).join(",");
    add_annotation_labels(label, false);
  });

  // \lx@set@frontmatter@label Internal to digesting annotation contents.
  // it sets (replaces) the _annotation labels on the currently being digesting
  // frontmatter so that the annotation inherits it as label (!!)
  // Typically would \let\label to this so that a \ref within a parent frontmatter
  // will get an annotation with corresponding \label will be attached.
  DefPrimitive!("\\lx@set@frontmatter@label [] Semiverbatim", sub[(prefix, label)] {
    // Optional prefix (default "LABEL"). The superscript heuristic passes the
    // role ("affiliation") so the label exactly matches the corresponding
    // \lx@request@frontmatter@annotation[affiliation] on the author; otherwise
    // relocate_annotations falls back to prefix-stripped matching and conflates
    // it with the creator's own "author:N" sequence label (double-attach).
    let prefix = prefix.as_ref().map(ToString::to_string)
      .filter(|p| !p.is_empty()).unwrap_or_else(|| "LABEL".to_string());
    let label = clean_frontmatter_labels(&label.to_string(), &prefix).into_iter().next();
    with_pending_entry_attr(move |attr| {
      match label {
        Some(label) => {
          DebugFeature!("frontmatter", "FRONT set label {label}");
          attr.insert("_annotations".to_string(), label);
        },
        None => {
          attr.remove("_annotations");
        },
      }
    });
  });
  Ok(())
}
