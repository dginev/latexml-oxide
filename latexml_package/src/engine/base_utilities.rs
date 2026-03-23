//! Base Utilities
//!
//! Core TeX Implementation for LaTeXML

use rustc_hash::FxHashSet as HashSet;
const FRONTMATTER_ELEMENTS: &[&str] = &[
  "ltx:title",
  "ltx:toctitle",
  "ltx:subtitle",
  "ltx:creator",
  "ltx:date",
  "ltx:abstract",
  "ltx:keywords",
  "ltx:classification",
  "ltx:acknowledgements",
];
use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // LaTeX has a very particular, but useful, notion of "Undefined",
  //    so let's get that squared away at the outset; it's useful for TeX, too!
  //
  // Naturally, it uses \csname to check, which ends up DEFINING the possibly undefined macro as
  // \relax
  DefMacro!("\\lx@ifundefined{}{}{}", sub[(name, if_token, else_token)] {
    let cs = T_CS!(s!("\\{}", Expand!(name).to_string()));
    if IsDefined!(&cs) {
      Ok(else_token)
    } else {
      state::assign_meaning(&cs, state::lookup_meaning(&TOKEN_RELAX), None);  // Let w/o AfterAssign
      Ok(if_token)
    }
  }, locked=>true);

  DefPrimitive!("\\lx@ignorehardspaces", {
    let mut boxes = Vec::new();
    while let Some(token) = gullet::read_x_token(None, false, None)? {
      boxes = stomach::invoke_token(&token)?;
      if boxes.is_empty() {
        break;
      }
      while !boxes.is_empty() {
        if match boxes[0].get_property("isSpace") {
          Some(Cow::Borrowed(Stored::Bool(space_bool))) => *space_bool,
          Some(Cow::Owned(Stored::Bool(ref space_bool))) => *space_bool,
          _ => false,
        } {
          boxes = boxes[1..].to_vec();
        } else {
          break;
        }
      }
      if !boxes.is_empty() {
        break;
      }
    }
    Ok(boxes)
  });

  DefConstructor!("\\@ADDCLASS Semiverbatim", sub[document,args] {
      document.add_class(&mut document.get_element().unwrap(), 
        &args[0].as_ref().unwrap().to_string())?; 
    }, sizer => 0);

  //======================================================================
  // General support for Front Matter.
  // Not (yet) used by TeX (finish plain?)
  // But provides support for LaTeX (and other formats?) for handling frontmatter.
  //
  // The idea is to accumulate any frontmatter material (title, author,...)
  // rather than directly drop it into the digested stream.
  // When we begin constructing the document, all accumulated material is output.
  // See LaTeX.ltxml for usage.
  // Note: could be circumstances where you'd want modular frontmatter?
  // (ie. frontmatter for each sectional unit)
  AssignValue!(
    "frontmatter",
    Stored::HashTagData(HashMap::default()),
    Some(Scope::Global)
  );

  // Dash and space primitives used by ligatures and other mechanisms
  DefPrimitive!("\\lx@endash", {
    Tbox::new(
      arena::pin_static("\u{2013}"),
      None, None,
      Tokens!(T_CS!("\\lx@endash")),
      SymHashMap::default(),
    )
  });
  DefPrimitive!("\\lx@emdash", {
    Tbox::new(
      arena::pin_static("\u{2014}"),
      None, None,
      Tokens!(T_CS!("\\lx@emdash")),
      SymHashMap::default(),
    )
  });
  // Perl: Box(UTF(0xA0), undef, undef, T_ACTIVE("~"), ...);
  DefPrimitive!("\\lx@NBSP", {
    Tbox::new(
      arena::pin_static("\u{00A0}"),
      None, None,
      Tokens!(T_ACTIVE!('~')),
      stored_map!("isSpace" => true, "width" => Dimension::from_str("0.333em")?),
    )
  });
  DefPrimitive!("\\lx@nobreakspace", {
    Tbox::new(
      arena::pin_static("\u{00A0}"),
      None, None,
      Tokens!(T_CS!("\\lx@nobreakspace")),
      stored_map!("isSpace" => true, "width" => Dimension::from_str("0.333em")?),
    )
  });

  DefConditional!("\\if@in@preamble", { lookup_bool("inPreamble") });

  // Add a new frontmatter item that will be enclosed in <$tag %attr>...</$tag>
  // The content is the result of digesting $tokens.
  // \\@add@frontmatter[keys]{tag}[attributes]{content}
  // Perl: DEFERS processing by pushing \@add@frontmatter@now invocation
  // into @at@begin@maketitle, which is digested at \maketitle time.
  // This is critical for correct ordering: \author creates entry BEFORE
  // \address/\email append to it.
  DefPrimitive!("\\@add@frontmatter OptionalKeyVals {} OptionalKeyVals {}",
    sub[(keys_opt,tag_tks,attrs_opt,tokens)] {
    // Build invocation: \@add@frontmatter@now[keys]{tag}[attrs]{tokens}
    let mut inv_tokens: Vec<Token> = vec![T_CS!("\\@add@frontmatter@now")];
    // Pass keys if present
    if let Some(ref keys) = keys_opt {
      inv_tokens.push(T_OTHER!("["));
      inv_tokens.extend(keys.revert()?.unlist());
      inv_tokens.push(T_OTHER!("]"));
    }
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(tag_tks.unlist());
    inv_tokens.push(T_END!());
    // Pass attrs if present
    if let Some(ref attrs) = attrs_opt {
      inv_tokens.push(T_OTHER!("["));
      inv_tokens.extend(attrs.revert()?.unlist());
      inv_tokens.push(T_OTHER!("]"));
    }
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(tokens.unlist());
    inv_tokens.push(T_END!());
    let _ = state::push_value("@at@begin@maketitle", Stored::Tokens(Tokens::new(inv_tokens)));
  });

  // \@add@frontmatter@now — actually processes the frontmatter entry
  // keys can have: replace (to replace the current entry, if any),
  //                ifnew (only add if no previous entry)
  DefPrimitive!("\\@add@frontmatter@now OptionalKeyVals {} OptionalKeyVals {}",
    sub[(keys_opt,tag_tks,attrs_opt,tokens)] {
    // Digest this as if we're already in the document body!
    let inpreamble = lookup_bool("inPreamble");
    assign_value("inPreamble", false, None);
    // Be careful since the contents may also want to add frontmatter
    // (which should be inside or after this one!)
    // So, we append this entry before digesting (Perl comment from Base_Utility.pool.ltxml)
    let tag = tag_tks.to_string();
    if let Some(keys) = keys_opt {
      let known_key = if let Some(Stored::HashTagData(ref mut frnt)) = lookup_value("frontmatter") {
        frnt.contains_key(&tag)
      } else { false };
      if known_key && keys.has_key("replace") {
        state::with_value_mut("frontmatter", |val_opt| {
          if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
            frnt.insert(tag.clone(), Vec::new());
          }
        });
      }
      if known_key && keys.has_key("ifnew") {
        return Ok(Vec::new());
      }
    }

    let attrs_digested = if let Some(attr_kvs) = attrs_opt {
      if let DigestedData::KeyVals(digested) = attr_kvs.be_digested()?.data() {
        Some(digested.get_hash_digested())
      } else {
        None
      }
    } else {
      None
    };
    // Perl: Create entry FIRST, then digest content (so nested @add@to@ appends to this entry)
    let placeholder = Digested::from(List::new(Vec::new()));
    let entry = (tag.clone(), attrs_digested, placeholder);
    state::with_value_mut("frontmatter", |val_opt| {
      let frontmatter = match val_opt {
        Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
        _ => fatal!(TexPool, Expected, "Global TeX Frontmatter hash was not available, should never happen"),
      };
      let f_entry = frontmatter.entry(tag.clone()).or_insert_with(Vec::new);
      f_entry.push(entry);
      Ok(())
    })?;
    // NOW digest the content — nested @add@to@frontmatter calls will see this entry as "last"
    let mut wrapped_tokens = vec![T_BEGIN!()];
    wrapped_tokens.extend(tokens.unlist());
    wrapped_tokens.push(T_END!());
    let digested_tokens = stomach::digest(Tokens::new(wrapped_tokens))?;
    // Fill in the placeholder
    state::with_value_mut("frontmatter", |val_opt| {
      let frontmatter = match val_opt {
        Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
        _ => return Ok::<(), latexml_core::Error>(()),
      };
      if let Some(list) = frontmatter.get_mut(&tag) {
        if let Some(last) = list.last_mut() {
          last.2 = digested_tokens;
        }
      }
      Ok(())
    })?;
    AssignValue!("inPreamble", inpreamble);
  },
  before_digest => { stomach::bgroup(); },
  after_digest => { let _ = stomach::egroup(); });

  // Append a piece of data to an existing frontmatter item that is contained in <$tag>
  // If $label is given, look for an item which has label=>$label,
  // otherwise, just append to the last item in $tag.

  // \@add@to@frontmatter{tag}[label]{content}
  // Perl: defers processing by pushing \@add@to@frontmatter@now invocation
  // into @at@begin@maketitle, which is digested at \maketitle time.
  DefPrimitive!("\\@add@to@frontmatter {} [] {}", sub[(tag, label, tokens)] {
    // Build invocation: \@add@to@frontmatter@now{tag}[label]{tokens}
    let mut inv_tokens = vec![T_CS!("\\@add@to@frontmatter@now"), T_BEGIN!()];
    inv_tokens.extend(tag.unwrap_or_default().unlist());
    inv_tokens.push(T_END!());
    if let Some(ref lbl) = label {
      inv_tokens.push(T_OTHER!("["));
      inv_tokens.extend(lbl.clone().unlist());
      inv_tokens.push(T_OTHER!("]"));
    }
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(tokens.unwrap_or_default().unlist());
    inv_tokens.push(T_END!());
    let _ = state::push_value("@at@begin@maketitle", Stored::Tokens(Tokens::new(inv_tokens)));
  });

  // \@add@to@frontmatter@now{tag}[label]{content}
  // Actually processes content into the frontmatter hash.
  DefPrimitive!("\\@add@to@frontmatter@now {} [] {}",
    sub[(tag_tks, label_opt, tokens)] {
    let tag = tag_tks.unwrap().to_string();
    let label = label_opt.as_ref().map(|l| l.to_string());
    let inpreamble = lookup_bool("inPreamble");
    assign_value("inPreamble", false, None);

    // Digest the content tokens
    let mut wrapped = vec![T_BEGIN!()];
    wrapped.extend(tokens.unwrap_or_default().unlist());
    wrapped.push(T_END!());
    let datum = stomach::digest(Tokens::new(wrapped))?;

    assign_value("inPreamble", inpreamble, None);

    state::with_value_mut("frontmatter", |val_opt| {
      let frontmatter = match val_opt {
        Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
        _ => fatal!(TexPool, Expected, "Frontmatter hash missing"),
      };
      if let Some(ref lbl) = label {
        // Look for existing item with matching label
        if let Some(list) = frontmatter.get_mut(&tag) {
          for item in list.iter_mut() {
            let (_, ref iattr, _) = item;
            if let Some(ref attrs) = iattr {
              if attrs.get("label").map(|s| s.as_str()) == Some(lbl.as_str()) {
                // Append datum to existing item
                let (_, _, ref mut existing_content) = item;
                let items = vec![existing_content.clone(), datum.clone()];
                *existing_content = Digested::from(List::new(items));
                return Ok(());
              }
            }
          }
        }
      } else if let Some(list) = frontmatter.get_mut(&tag) {
        // No label: append datum to last item in tag
        if let Some(last) = list.last_mut() {
          let (_, _, ref mut existing_content) = last;
          let items = vec![existing_content.clone(), datum.clone()];
          *existing_content = Digested::from(List::new(items));
          return Ok(());
        }
      }
      // New entry
      let attrs = label.map(|l| {
        let mut m = HashMap::default();
        m.insert("label".to_string(), l);
        m
      });
      let entry = (tag.clone(), attrs, datum);
      frontmatter.entry(tag).or_insert_with(Vec::new).push(entry);
      Ok(())
    })?;
  },
  before_digest => { stomach::bgroup(); },
  after_digest => { let _ = stomach::egroup(); });

  // Add FrontMatter at document begin, unless deferred to a better position.
  Tag!("ltx:document", after_open_late => sub[document,_root] {
    if !lookup_bool("frontmatter_deferred") {
      insert_frontmatter(document)?;
    }
  });

  // Request Frontmatter to appear HERE (if not already done),
  // deferring it from document begin.
  DefConstructor!("\\lx@frontmatterhere", sub[doc,_args] { insert_frontmatter(doc)? },
    after_digest => {
      // Perl: digest @at@begin@maketitle tokens (which runs \@add@to@frontmatter@now
      // for each deferred frontmatter entry, populating the frontmatter hash)
      if let Some(Stored::VecDequeStored(ref tks_list)) = state::lookup_value("@at@begin@maketitle") {
        // Collect all token lists into one
        let mut all_tokens = Vec::new();
        for stored_item in tks_list.iter() {
          if let Stored::Tokens(ref tks) = stored_item {
            all_tokens.extend(tks.clone().unlist());
          }
        }
        if !all_tokens.is_empty() {
          let _ = stomach::digest(Tokens::new(all_tokens));
        }
        state::assign_value("@at@begin@maketitle", Stored::None, Some(Scope::Global));
      }
      state::assign_value("frontmatter_deferred", true, Some(Scope::Global));
    });

  // Fallback: if \maketitle wasn't used, this still triggers frontmatter placement.
  // Perl: processes @at@begin@maketitle tokens.
  DefPrimitive!("\\lx@frontmatter@fallback", None,
    after_digest => {
      if let Some(Stored::VecDequeStored(ref tks_list)) = state::lookup_value("@at@begin@maketitle") {
        let mut all_tokens = Vec::new();
        for stored_item in tks_list.iter() {
          if let Stored::Tokens(ref tks) = stored_item {
            all_tokens.extend(tks.clone().unlist());
          }
        }
        if !all_tokens.is_empty() {
          let _ = stomach::digest(Tokens::new(all_tokens));
        }
        state::assign_value("@at@begin@maketitle", Stored::None, Some(Scope::Global));
      }
    });

  // Maintain a list of classes that apply to the document root.
  // This might involve global style options, like leqno.
  Tag!("ltx:document", after_open_late => sub[document, root] {
    let classes = with_mapping_keys("DOCUMENT_CLASSES", |keys| arena::join(&keys," "));
    if !classes.is_empty()  {
      document.add_class(root, &classes)?;
    }
  });

  //======================================================================
  // Tags & Titles
  // The reference numbers, titles, captions etc, for various objects have
  // different styling conventions, and the styling various depending on context.
  // We'll use ltx:tags as a container for the various forms of ltx:tag with different @role's.
  // The role=refnum form is simply formatted by \the<counter> and used by \ref;
  // An ltx:tag w/o @role are for the numbers, often formatted differently, which
  // appear alongside the object; Such a tag also may be embedded within the title or caption.
  // Cross-references automatically generated by LaTeXML benefit from a bit more context:
  // these are the role=typerefnum forms.
  // Additional forms are needed for bibliographies, hyperref's autoref, etc.
  // An additional complication is that while the "type" determines the formatting
  // of the various forms, some types (eg. theorems) share the same counter.
  // LaTeX defines this handling on an adhoc basis; defines \fnum@table, \fnum@figure for some types
  // but \labelenumi, etc for others.

  // This section synthesizes a more uniform support for reference numbers,
  // references to reference numbers, title formatting etc.
  // It allows you to customize each of the forms for each type encountered.
  // The design reflects LaTeX needs, more than TeX, but support starts here!

  // This collects up the various declared ltx:tag's into an ltx:tags
  DefMacro!("\\lx@make@tags {}", sub[(ttype)] {
    let mut tags = Vec::new();
    if let Some(Stored::HashStored(formatters)) =
      lookup_value("type_tag_formatter") {
      let keys_sym : Vec<_> = formatters.keys().copied().collect();
      let mut sorted_keys : Vec<String> = arena::with_many(&keys_sym, |keys| {
        keys.into_iter().map(str::to_owned).collect()
      });
      sorted_keys.sort();
      for role in sorted_keys.iter() {
        let formatter_opt = match formatters.get(role) {
          Some(Stored::Token(t)) => Some(*t),
          Some(Stored::String(sym)) => Some(Token { text: *sym, code: Catcode::CS }),
          _ => None
        };
        if let Some(formatter_token) = formatter_opt {
          tags.push(Invocation!(T_CS!("\\lx@tag@intags"),
            vec![
              Tokens!(T_OTHER!(role)),
              build_invocation(formatter_token, vec![Some(ttype.clone())])?
            ])
          );
        }
      }
    }

    let mut lx_tags = vec![T_CS!("\\lx@tags"), T_BEGIN!()];
    for invoked_tag in tags {
      lx_tags.append(&mut invoked_tag.unlist());
    }
    lx_tags.push(T_END!());
    Ok(Tokens::new(lx_tags))
  });

  // Remove the last closed node, if it's empty.
  let remove_empty_element: Vec<ConstructionClosure> = construct!(document, _whatsit, {
    if let Some(node) = document.get_node().get_last_child() {
      // This should be the wrapper just added.
      if node.get_child_nodes().is_empty() {
        document.remove_node(node);
      }
    }
  });

  // \lx@tag[open][close]{stuff}
  let remove_empty_element_1 = remove_empty_element.clone();
  DefConstructor!("\\lx@tag[][][]{}", "<ltx:tag open='#1' close='#2'>#4</ltx:tag>",
    mode => "restricted_horizontal",
    after_construct => remove_empty_element_1
  );

  // \lx@tag@intags{role}{stuff}
  let remove_empty_element_2 = remove_empty_element.clone();
  DefConstructor!("\\lx@tag@intags[]{}", "<ltx:tag role='#1'>#2</ltx:tag>",
    mode => "restricted_horizontal",
    before_digest => sub { neutralize_font(); },
    after_construct => remove_empty_element_2
  );
  DefConstructor!("\\lx@tags{}","<ltx:tags>#1</ltx:tags>",
    after_construct => remove_empty_element
  );

  //----------------------------------------------------------------------
  // "refnum" is the lowest level reference number for an object is typically \the<counter>
  // but be sure to use the right counter!  This is how \ref will show the number.
  // You'll typically customize this by defining \the<counter> (and \p@<counter) as in LaTeX.
  DefMacro!("\\lx@counterfor{}", sub[(ctr_type)] {
    with_mapping("counter_for_type", &ctr_type.to_string(), |ctr_opt|
    if let Some(ctr) = ctr_opt {
      Tokens!(T_OTHER!(ctr.to_string()))
    } else {
      ctr_type
    })
  });
  DefMacro!(
    "\\lx@the@@{}",
    "\\expandafter\\lx@@the@@\\expandafter{\\lx@counterfor{#1}}"
  );
  DefMacro!("\\lx@@the@@{}", "\\csname the#1\\endcsname");

  DefMacro!(
    "\\lx@therefnum@@{}",
    "\\expandafter\\lx@@therefnum@@\\expandafter{\\lx@counterfor{#1}}"
  );
  DefMacro!(
    "\\lx@@therefnum@@{}",
    "{\\normalfont\\csname p@#1\\endcsname\\csname the#1\\endcsname}"
  );

  AssignMapping!("type_tag_formatter", "refnum" => "\\lx@therefnum@@");

  //----------------------------------------------------------------------
  // \lx@fnum@@{type}  Gets the formatted form of the refnum, as part of the object, (no @role).
  // Customize by defining \fnum@<type> or \<type>name and \fnum@font@<type>
  // Default uses \fnum@font@<type> \<type>name prefix + space (if any) and \the<counter>.
  // When using the "name", uses \<type>name in preference to fallback \lx@name@<type>
  DefMacro!(
    r"\lx@refnum@compose{}{}",
    r"\expandafter\lx@refnum@compose@\expandafter{#2}{#1}"
  );
  DefMacro!(r"\lx@refnum@compose@{}{}", r"\if.#1.#2\else#2\space#1\fi");

  DefMacro!(
    r"\lx@fnum@@{}",
    r"{\normalfont\@ifundefined{fnum@font@#1}{}{\csname fnum@font@#1\endcsname}\@ifundefined{fnum@#1}{\lx@@fnum@@{#1}}{\csname fnum@#1\endcsname}}"
  );

  // Really seems like <type>name should take precedence over \lx@name@<type>,
  // since users might define it.
  // BUT amsthm defines \thmname{}!
  DefMacro!(
    "\\lx@@fnum@@ {}",
    r"\@ifundefined{lx@name@#1}{\@ifundefined{#1name}{\lx@the@@{#1}}{\lx@refnum@compose{\csname #1name\endcsname}{\lx@the@@{#1}}}}{\lx@refnum@compose{\csname lx@name@#1\endcsname}{\lx@the@@{#1}}}"
  );

  AssignMapping!("type_tag_formatter", "" => "\\lx@fnum@@"); // Default!

  //----------------------------------------------------------------------
  // \\lx@fnum@toc@{type} is similar, but formats the number for use within \\toctitle
  // Customize by defining \\fnum@toc@<type> or \\fnum@tocfont@<type>
  // Default uses just \\the<counter>, else composes using \\lx@@fnum@@{type}
  DefMacro!(
    r"\lx@fnum@toc@@{}",
    r"{\normalfont\@ifundefined{fnum@tocfont@#1}{}{\csname fnum@tocfont@#1\endcsname}\@ifundefined{fnum@toc@#1}{\lx@the@@{#1}}{\csname fnum@toc@#1\endcsname}}"
  );

  //----------------------------------------------------------------------
  // "typerefnum" form is used by automatic cross-references, typically "type number" or similar.
  // Customize by defining \typerefnum@<type> or \typerefnum@font@<type>
  // Default uses either \<type>typerefname or \<type>name (if any, followed by space, then
  // \\the<counter>
  DefMacro!(
    "\\lx@typerefnum@@{}",
    r"{\normalfont\@ifundefined{typerefnum@font@#1}{}{\csname typerefnum@font@#1\endcsname}\@ifundefined{typerefnum@#1}{\lx@@typerefnum@@{#1}}{\csname typerefnum@#1\endcsname}}"
  );

  DefMacro!(
    "\\lx@@typerefnum@@{}",
    r"\@ifundefined{#1typerefname}{\@ifundefined{lx@name@#1}{\@ifundefined{#1name}{}{\lx@refnum@compose{\csname #1name\endcsname}{\csname p@#1\endcsname\lx@the@@{#1}}}}{\lx@refnum@compose{\csname lx@name@#1\endcsname}{\csname p@#1\endcsname\lx@the@@{#1}}}}{\lx@refnum@compose{\csname #1typerefname\endcsname}{\csname p@#1\endcsname\lx@the@@{#1}}}"
  );

  AssignMapping!("type_tag_formatter", "typerefnum" => "\\lx@typerefnum@@");

  //----------------------------------------------------------------------
  // The following macros provide similar customization for titles & toctitles
  // in particular for supporting localization for different languages.
  // Redefine these if you want to assemble the name (eg. \chaptername), refnum and titles
  // differently
  //----------------------------------------------------------------------
  // \lx@format@title@@{type}{title}
  // Format a title (or caption) appropriately for type.
  // Customize by defining \format@title@type{title}
  // Default composes \lx@fnum@@{type} space title.
  DefMacro!(
    "\\lx@format@title@@{}{}",
    r"\lx@@format@title@@{#1}{{\lx@format@title@font@@{#1}#2}}"
  );
  DefMacro!(
    "\\lx@@format@title@@{}{}",
    r"{\@ifundefined{format@title@#1}{\lx@@compose@title{\lx@fnum@@{#1}}{#2}}{\csname format@title@#1\endcsname{#2}}}"
  );

  // \\lx@format@toctitle@@{type}{toctitle}
  // Similar for toctitle, typically briefer
  // Customize by defining \\format@toctitle@type{title}
  // Default composes \\lx@fnum@toc@@{type} space title.
  DefMacro!(
    "\\lx@format@toctitle@@{}{}",
    r"\lx@@format@toctitle@@{#1}{{\lx@format@toctitle@font@@{#1}#2}}"
  );

  DefMacro!(
    "\\lx@@format@toctitle@@{}{}",
    r"{\@ifundefined{format@toctitle@#1}{\lx@@compose@title{\lx@fnum@toc@@{#1}}{#2}}{\csname format@toctitle@#1\endcsname{#2}}}"
  );

  DefMacro!("\\lx@@compose@title{}{}", r"\lx@tag[][ ]{#1}#2");

  DefMacro!(
    r"\lx@format@title@font@@{}",
    r"\@ifundefined{format@title@font@#1}{}{\csname format@title@font@#1\endcsname}"
  );
  DefMacro!(
    r"\lx@format@toctitle@font@@{}",
    r"\@ifundefined{format@toctitle@font@#1}{}{\csname format@toctitle@font@#1\endcsname}"
  );

  // NOTE that a 3rd form seems desirable: an concise form that cannot rely on context for the type.
  // This would be useful for the titles in links; thus can be plain (unicode) text.

  //======================================================================
  // Normally definitions disappear; the macros are expanded or have their expected effect.
  // But in a few cases (eg tabular column definitions, or LaTeX \Declarexxxx)
  // they will need declarations in the (La)TeX preamble to allow (La)TeX to process snippets
  // (eg. math) in order to create images.
  // Returning a call to this utility from Primitives will add a preamble Processing Instruction

  // TODO
  // sub AddToPreamble {
  //   my ($cs, @args) = @_;
  //   return Digest(Invocation(T_CS('\lx@add@Preamble@PI'), Invocation((ref $cs ? $cs : T_CS($cs)),
  // @args))); }

  // Perl: DefConstructor('\lx@add@Preamble@PI Undigested', "<?latexml preamble='#1'?>");
  // PI syntax not supported in constructor templates, so use procedural body.
  DefConstructor!("\\lx@add@Preamble@PI Undigested",
    sub[document, args, _props] {
      if let Some(Some(preamble_arg)) = args.first() {
        let preamble_text = preamble_arg.untex()?;
        if !preamble_text.is_empty() {
          let mut attrs = HashMap::default();
          attrs.insert(String::from("preamble"), preamble_text);
          document.insert_pi("latexml", Some(attrs))?;
        }
      }
    }
  );
});

/// Insert FrontMatter into document, if not already added
pub fn insert_frontmatter(document: &mut Document) -> Result<()> {
  if lookup_bool("frontmatter_done") {
    return Ok(());
  }
  let frontmatter_elements_set: HashSet<String> =
    FRONTMATTER_ELEMENTS.iter().map(ToString::to_string).collect();

  // Get frontmatter hash
  let frontmatter_ref = match state::lookup_value("frontmatter") {
    Some(Stored::HashTagData(frnt)) => frnt,
    _ => return Ok(()),
  };
  let set_keys: Vec<String> = frontmatter_ref.keys().cloned().collect();

  // If doc ONLY has abstract as frontmatter, defer until abstract's document location
  if set_keys.len() == 1
    && set_keys[0] == "ltx:abstract"
    && !lookup_bool("frontmatter_deferred")
  {
    state::assign_value("frontmatter_deferred", true, Some(Scope::Global));
    return Ok(());
  }

  state::assign_value("frontmatter_done", true, Some(Scope::Global));

  // Remove frontmatter and replace with empty
  let mut frontmatter = match state::remove_value("frontmatter") {
    Some(Stored::HashTagData(frnt)) => frnt,
    _ => return Ok(()),
  };
  state::assign_value(
    "frontmatter",
    Stored::HashTagData(HashMap::default()),
    Some(Scope::Global),
  );

  // Order: first go through frontmatter_elements order, then any custom keys
  let custom_keys: Vec<String> = frontmatter
    .keys()
    .filter(|key| !frontmatter_elements_set.contains(key.as_str()))
    .map(ToString::to_string)
    .collect();
  let mut all_keys: Vec<String> =
    FRONTMATTER_ELEMENTS.iter().map(ToString::to_string).collect();
  all_keys.extend(custom_keys);

  for key in &all_keys {
    if let Some(list) = frontmatter.remove(key) {
      // Dubious, but assures that frontmatter appears in text mode...
      document.set_box_to_absorb(
        Tbox::new(
          *EMPTY_SYM,
          lookup_font(),
          None,
          Tokens!(T_SPACE!()),
          SymHashMap::default(),
        )
        .into(),
      );
      for (tag, attr, stuff) in list {
        // Add a dedicated class for frontmatter notes
        let attr = if tag == "ltx:note" {
          let mut a = attr.unwrap_or_default();
          let existing = a.get("class").cloned().unwrap_or_default();
          let new_class = if existing.is_empty() {
            "ltx_note_frontmatter".to_string()
          } else {
            s!("{existing} ltx_note_frontmatter")
          };
          a.insert("class".to_string(), new_class);
          Some(a)
        } else {
          attr
        };
        document.open_element(&tag, attr, None)?;
        document.absorb(&stuff, None)?;
        let completed_node = document.close_element(&tag)?;
        // Prune empty frontmatter elements (except ltx:rdf)
        if tag != "ltx:rdf" {
          if let Some(ref node) = completed_node {
            if node.get_child_nodes().is_empty() {
              document.remove_node(node.clone());
            }
          }
        }
      }
      document.expire_box_to_absorb();
    }
  }
  Ok(())
}
