//----------------------------------------------------------------------
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

use crate::package::*;
use std::collections::HashSet;
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

LoadDefinitions!(state, {
  AssignValue!("frontmatter", Stored::HashTagData(HashMap::new()), Some(Scope::Global));

  // // Add a new frontmatter item that will be enclosed in <$tag %attr>...</$tag>
  // // The content is the result of digesting $tokens.
  // // \\@add@frontmatter[keys]{tag}[attributes]{content}
  // // keys can have
  // //   replace (to replace the current entry, if any)
  // //   ifnew   (only add if no previous entry)//

  DefPrimitive!("\\@add@frontmatter OptionalKeyVals {} OptionalKeyVals {}", sub[stomach, args, state] {
    unpack!(args => keys, tag, attrs, tokens);
    // Digest this as if we're already in the document body!
    let inpreamble = LookupBool!("inPreamble");
    AssignValue!("inPreamble", false);

    // Be careful since the contents may also want to add frontmatter
    // (which should be inside or after this one!)
    // So, we append this entry before digesting

    // TODO: Port over keys handling from TeX.pool
    let attrs_digested = if attrs.is_empty() {
      None
    } else {
      // WAS: $$entry[1] = { $attr->beDigested($stomach)->getHash };
      let attr_kvs = attrs.to_keyvals(state);
      if let Digested::KeyVals(digested) = attr_kvs.be_digested(stomach, state)? {
        Some(digested.get_hash())
      } else {
        None
      }
    };
    // WAS:  $$entry[2] = Digest(Tokens(T_BEGIN, $tokens, T_END));
    let mut wrapped_tokens = vec![T_BEGIN!()];
    wrapped_tokens.extend(tokens.unlist());
    wrapped_tokens.push(T_END!());
    let digested_tokens = stomach.digest(Tokens::new(wrapped_tokens), state)?;
    let entry = (tag.to_string(), attrs_digested, digested_tokens);
    let frontmatter = match state.lookup_value_mut("frontmatter") {
      Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
      _ => fatal!(TexPool, Expected, "Global TeX Frontmatter hash was not available, should never happen"),
    };
    let f_entry = frontmatter.entry(tag.to_string()).or_insert_with(Vec::new);
    f_entry.push(entry);

    AssignValue!("inPreamble", inpreamble);
  });

  // // Append a piece of data to an existing frontmatter item that is contained in <$tag>
  // // If $label is given, look for an item which has label=>$label,
  // // otherwise, just append to the last item in $tag.

  // // \@add@to@frontmatter{tag}[label]{content}
  // DefPrimitive('\@add@to@frontmatter {} [] {}', sub {
  //     my ($stomach, $tag, $label, $tokens) = @_;
  //     $tag = ToString($tag);
  //     $label = ToString($label) if $label;
  //     my $frontmatter = LookupValue('frontmatter');

  //     my $inpreamble = LookupValue('inPreamble');
  //     AssignValue(inPreamble => 0);
  //     my $datum = Digest(Tokens(T_BEGIN, $tokens, T_END));
  //     AssignValue(inPreamble => $inpreamble);
  //     if ($label) {
  //       my $entry;
  //       foreach my $item (@{ $$frontmatter{$tag} || [] }) {
  //         my ($itag, $iattr, @stuff) = @$item;
  //         if ($label eq ($$iattr{label} || '')) {
  //           push(@$item, $datum);
  //           return; } } }
  //     elsif (my $list = $$frontmatter{$tag}) {
  //       push(@{ $$list[-1] }, $datum);
  //       return; }
  //     push(@{ $$frontmatter{$tag} }, [$tag, ($label ? { label => $label } : undef), $datum]);
  //     return; });

  // This is called by afterOpen (by default on <ltx:document>) to
  // output any frontmatter that was accumulated.

  Tag!("ltx:document", after_open_late => sub[document, node, state] {
    // this happens only once, not a big deal to skip the lazy_static! and keep it in the closure
    let frontmatter_elements_set: HashSet<String> = FRONTMATTER_ELEMENTS.iter().map(ToString::to_string).collect();

    let mut frontmatter = match state.remove_value("frontmatter") {
      Some(Stored::HashTagData(frnt)) => frnt,
      _ => fatal!(TexPool, Expected, "Global TeX Frontmatter hash was not available, should never happen"),
    };
    state.assign_value("frontmatter", Stored::HashTagData(HashMap::new()), Some(Scope::Global));

    // order is important here, first go through frontmatter_elements, then any leftover keys.
    let custom_keys: Vec<String> = frontmatter
      .keys()
      .filter(|key| !frontmatter_elements_set.contains(key.as_str()))
      .map(ToString::to_string)
      .collect();
    let mut all_keys: Vec<String> = FRONTMATTER_ELEMENTS.iter().map(ToString::to_string).collect();
    all_keys.extend(custom_keys);

    for key in &all_keys {
      if let Some(list) = frontmatter.remove(key) {
        // Dubious, but assures that frontmatter appears in text mode...
        // TODO:
        //local $LaTeXML::BOX = Box('', $STATE->lookupValue('font'), '', T_SPACE);
        document.set_box_to_absorb(Tbox::new(String::new(), state.lookup_font(), None, Tokens!(T_SPACE!()), HashMap::new(), state).into());
        for (tag, attr, stuff) in list {
          document.open_element(&tag, attr, None, state)?; // TODO:  (scalar(@stuff) && $document->canHaveAttribute($tag, 'font')
                                                           //        ? (font => $stuff[0]->getFont, _force_font => 'true') : ()));
          document.absorb(&stuff, None, state)?;

          document.close_element(&tag, state)?;
        }
        document.localize_box_to_absorb();
      }
    }
  });

  // Maintain a list of classes that apply to the document root.
  // This might involve global style options, like leqno.
  Tag!("ltx:document", after_open_late => sub[document, root, state] {
    let classes = LookupMappingKeys!("DOCUMENT_CLASSES").join(" ");
    if !classes.is_empty()  {
      document.add_class(root, &classes)?;
    }
  });

  DefConstructor!("\\beginsection Until:\\par", "<ltx:section><ltx:title>#1</ltx:title>");

  // // POSSIBLY #1 is a name or reference number and  #2 is the theoremm TITLE
  // //  If so, how do know when the theorem ends?
  // DefConstructor('\proclaim', parseDefParameters('\proclaim', Tokenize('#1. #2\par')),
  //   "<ltx:theorem>"
  //     . "<ltx:title font='#titlefont' _force_font='true' >#title</ltx:title>"
  //     . "#2"
  //     . "</ltx:theorem>",
  //   properties => sub {
  //     my $title = $_[1];
  //     (title => $title, titlefont => $title->getFont); });

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
  DefMacro!("\\lx@make@tags {}", sub[gullet, args, state] {
    unpack!(args => ttype);
    let formatters = if let Some(Stored::HashStored(formatters)) = state.lookup_value("type_tag_formatter") {
      Some(formatters.clone())
    } else {
      None
    };

    let mut tags = Vec::new();
    if let Some(formatters) = formatters {
      let mut sorted_keys : Vec<&String> = formatters.keys().collect();
      sorted_keys.sort();
      for role in sorted_keys.iter() {
        let formatter = &formatters[*role];
        // Note: Another curious mutability issue here if we leave ",state" out of the Invocation!()
        // call. We'd need to assign each invocation piece in a separate variable, to avoid Rust getting
        // confused about mutability conflicts in borrowing. The explicit invocation seems clear enough.
        tags.push(Invocation!(T_CS!("\\lx@tag@intags"),
          vec![
            Tokens!(T_OTHER!(role)),
            build_invocation(formatter, vec![Some(ttype.clone())], gullet, state)?
          ], gullet, state)?
        );
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
  let remove_empty_element: Vec<ConstructionClosure> = construct!(document, whatsit, state, {
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
    bounded => true,
    mode => "text",
    after_construct => remove_empty_element_1
  );

  // \lx@tag@intags{role}{stuff}
  let remove_empty_element_2 = remove_empty_element.clone();
  DefConstructor!("\\lx@tag@intags[]{}", "<ltx:tag role='#1'>#2</ltx:tag>",
    bounded => true, mode => "text",
    after_construct => remove_empty_element_2
  );
  DefConstructor!("\\lx@tags{}","<ltx:tags>#1</ltx:tags>",
    after_construct => remove_empty_element
  );

  //----------------------------------------------------------------------
  // "refnum" is the lowest level reference number for an object is typically \the<counter>
  // but be sure to use the right counter!  This is how \ref will show the number.
  // You'll typically customize this by defining \the<counter> (and \p@<counter) as in LaTeX.
  DefMacro!("\\lx@counterfor{}", sub[gullet, args, state] {
    unpack!(args => ctr_type);
    if let Some(ctr) = LookupMapping!("counter_for_type", &ctr_type.to_string()) {
      Tokens!(T_OTHER!(ctr))
    } else {
      ctr_type
    }
  });
  DefMacro!("\\lx@the@@{}", "\\expandafter\\lx@@the@@\\expandafter{\\lx@counterfor{#1}}");
  DefMacro!("\\lx@@the@@{}", "\\csname the#1\\endcsname");

  DefMacro!("\\lx@therefnum@@{}", "\\expandafter\\lx@@therefnum@@\\expandafter{\\lx@counterfor{#1}}");
  DefMacro!("\\lx@@therefnum@@{}", "{\\normalfont\\csname p@#1\\endcsname\\csname the#1\\endcsname}");

  AssignMapping!("type_tag_formatter", "refnum" => "\\lx@therefnum@@");

  //----------------------------------------------------------------------
  // \lx@fnum@@{type}  Gets the formatted form of the refnum, as part of the object, (no @role).
  // Customize by defining \fnum@<type> or \<type>name and \fnum@font@<type>
  // Default uses \fnum@font@<type> \<type>name prefix + space (if any) and \the<counter>.
  // When using the "name", uses \<type>name in preference to fallback \lx@name@<type>
  DefMacro!(r"\lx@refnum@compose{}{}", r"\expandafter\lx@refnum@compose@\expandafter{#2}{#1}");
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
    r"\@ifundefined{lx@name@#1}{\@ifundefined{#1name}{\lx@the@@{#1}}{\lx@refnum@compose{\csname #1name\endcsname}{\lx@the@@{#1}}}}
    {\lx@refnum@compose{\csname lx@name@#1\endcsname}{\lx@the@@{#1}}}"
  );

  AssignMapping!("type_tag_formatter", "" => "\\lx@fnum@@"); // Default!

  //----------------------------------------------------------------------
  // \\lx@fnum@toc@{type} is similar, but formats the number for use within \\toctitle
  // Customize by defining \\fnum@toc@<type> or \\fnum@tocfont@<type>
  // Default uses just \\the<counter>, else composes using \\lx@@fnum@@{type}
  DefMacro!(
    r"\lx@fnum@toc@@{}",
    r"{\normalfont\@ifundefined{fnum@tocfont@#1}{}
      {\csname fnum@tocfont@#1\endcsname}\@ifundefined{fnum@toc@#1}{\lx@the@@{#1}}{\csname fnum@toc@#1\endcsname}}"
  );

  //----------------------------------------------------------------------
  // "typerefnum" form is used by automatic cross-references, typically "type number" or similar.
  // Customize by defining \\typerefnum@<type> or \\typerefnum@font@<type>
  // Default uses either \\<type>typerefname or \\<type>name (if any, followed by space, then
  // \\the<counter>
  DefMacro!(
    "\\lx@typerefnum@@{}",
    "{\\normalfont\\@ifundefined{typerefnum@font@#1}{}\
     {\\csname typerefnum@font@#1\\endcsname}\\@ifundefined{typerefnum@#1}\
     {\\lx@@typerefnum@@{#1}}{\\csname typerefnum@#1\\endcsname}}"
  );

  DefMacro!(
    "\\lx@@typerefnum@@{}",
    "\\@ifundefined{#1typerefname}{\\@ifundefined{#1name}{}{\
     \\lx@refnum@compose{\\csname #1name\\endcsname}{\\lx@the@@{#1}}}}\
     {\\lx@refnum@compose{\\csname #1typerefname\\endcsname}{\\lx@the@@{#1}}}"
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
    "\\lx@@format@title@@{#1}{{\\@ifundefined{format@title@font@#1}{}{\\csname format@title@font@#1\\endcsname}#2}}"
  );
  DefMacro!(
    "\\lx@@format@title@@{}{}",
    "{\\@ifundefined{format@title@#1}{\\lx@@compose@title{\\lx@fnum@@{#1}}{#2}}{\\csname format@title@#1\\endcsname{#2}}}"
  );

  // \\lx@format@toctitle@@{type}{toctitle}
  // Similar for toctitle, typically briefer
  // Customize by defining \\format@toctitle@type{title}
  // Default composes \\lx@fnum@toc@@{type} space title.
  DefMacro!(
    "\\lx@format@toctitle@@{}{}",
    "\\lx@@format@toctitle@@{#1}\
     {{\\@ifundefined{format@toctitle@font@#1}{}{\\csname format@toctitle@font@#1\\endcsname}#2}}"
  );

  DefMacro!(
    "\\lx@@format@toctitle@@{}{}",
    "{\\@ifundefined{format@toctitle@#1}\
     {\\lx@@compose@title{\\lx@fnum@toc@@{#1}}{#2}}\
     {\\csname format@toctitle@#1\\endcsname{#2}}}"
  );

  DefMacro!("\\lx@@compose@title{}{}", "\\lx@tag[][ ]{#1}#2");

  // NOTE that a 3rd form seems desirable: an concise form that cannot rely on context for the type.
  // This would be useful for the titles in links; thus can be plain (unicode) text.
});
