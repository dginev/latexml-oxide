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

use package::*;
use rtx_core::document::tag::TagConstructionClosure;
use std::collections::HashSet;
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  AssignValue!(
    "frontmatter",
    Stored::HashTagData(HashMap::new()),
    Some(Scope::Global)
  );

  // // Add a new frontmatter item that will be enclosed in <$tag %attr>...</$tag>
  // // The content is the result of digesting $tokens.
  // // \\@add@frontmatter[keys]{tag}[attributes]{content}
  // // keys can have
  // //   replace (to replace the current entry, if any)
  // //   ifnew   (only add if no previous entry)//

  // TODO: Real signature when we have KeyVals
  // DefPrimitiveI!("\\@add@frontmatter OptionalKeyVals {} OptionalKeyVals {}",
  DefPrimitiveI!(
    "\\@add@frontmatter{}{}",
    primitiveproc!(stomach, args, state, {
      unpack!(args => tag, tokens);
      // TODO: Real args when we have KeyVals
      // unpack!(args => keys, tag, attr, tokens);

      // Digest this as if we're already in the document body!
      let inpreamble = state.lookup_bool("inPreamble");
      state.assign_value("inPreamble", false, None);
      {
        // Be careful since the contents may also want to add frontmatter
        // (which should be inside or after this one!)
        // So, we append this entry before digesting
        // if ($keys && $keys->hasKey('replace') && $$frontmatter{$tag}) {    // if replace and
        // previous entries $$frontmatter{$tag} = []; }
        // // Remove previous entries if ($keys && $keys->hasKey('ifnew') &&
        // $$frontmatter{$tag}) {      // if ifnew and previous entries return; }
        // // Skip this one.   if ($attr) {
        //     $$entry[1] = { $attr->beDigested($stomach)->getHash }; }
        //   $$entry[2] = Digest(Tokens(T_BEGIN, $tokens, T_END));
        let mut wrapped_tokens = vec![T_BEGIN!()];
        wrapped_tokens.extend(tokens.clone().unlist());
        wrapped_tokens.push(T_END!());
        let digested_tokens = stomach.digest(Tokens::new(wrapped_tokens), state)?;
        let entry = (tag.to_string(), None, digested_tokens);
        let frontmatter = match state.lookup_value_mut("frontmatter") {
          Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
          _ => fatal!(
            TexPool,
            Expected,
            "Global TeX Frontmatter hash was not available, should never happen"
          ),
        };
        let f_entry = frontmatter.entry(tag.to_string()).or_insert_with(Vec::new);
        f_entry.push(entry);
      }
      state.assign_value("inPreamble", inpreamble, None);
    })
  );

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

  let insert_frontmatter: Vec<TagConstructionClosure> = tagsub!(document, node, state, {
    let frontmatter_elements: HashSet<String> = [
      "ltx:title",
      "ltx:toctitle",
      "ltx:subtitle",
      "ltx:creator",
      "ltx:date",
      "ltx:abstract",
      "ltx:keywords",
      "ltx:classification",
      "ltx:acknowledgements",
    ]
      .iter()
      .map(|s| s.to_string())
      .collect();

    let mut frontmatter = match state.remove_value("frontmatter") {
      Some(Stored::HashTagData(frnt)) => frnt,
      _ => fatal!(
        TexPool,
        Expected,
        "Global TeX Frontmatter hash was not available, should never happen"
      ),
    };
    state.assign_value(
      "frontmatter",
      Stored::HashTagData(HashMap::new()),
      Some(Scope::Global),
    );
    let state_keys: HashSet<String> = frontmatter.keys().cloned().collect();
    let mut all_keys: HashSet<String> = frontmatter_elements.union(&state_keys).cloned().collect();
    for key in &all_keys {
      if let Some(list) = frontmatter.remove(key) {
        // Dubious, but assures that frontmatter appears in text mode...
        // TODO:
        //local $LaTeXML::BOX = Box('', $STATE->lookupValue('font'), '', T_SPACE);
        document.box_to_absorb = Some(Digested::TBox(Box::new(Tbox::new(
          String::new(),
          state.lookup_font(),
          None,
          Tokens!(T_SPACE!()),
          HashMap::new(),
          state,
        ))));
        for (tag, attr, stuff) in list {
          document.open_element(&tag, attr, None, state)?; // TODO:  (scalar(@stuff) && $document->canHaveAttribute($tag, 'font')
                                                           //        ? (font => $stuff[0]->getFont, _force_font => 'true') : ()));
          document.absorb(stuff, state)?;

          document.close_element(&tag, state)?;
        }
      }
    }
  });

  Tag!("ltx:document", after_open_late => insert_frontmatter);

  // // Maintain a list of classes that apply to the document root.
  // // This might involve global style options, like leqno.
  // Tag('ltx:document', 'afterOpen:late' => sub {
  //     my ($document, $root) = @_;
  //     if (my $classes = join(' ', LookupMappingKeys('DOCUMENT_CLASSES'))) {
  //       $document->addClass($root, $classes); } });

  // DefConstructor('\beginsection Until:\par',
  //   "<ltx:section><ltx:title>#1</ltx:title>");

  // // POSSIBLY #1 is a name or reference number and  #2 is the theoremm TITLE
  // //  If so, how do know when the theorem ends?
  // DefConstructorI('\proclaim', parseDefParameters('\proclaim', Tokenize('#1. #2\par')),
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
        let formatter = formatters.get(*role).unwrap();

        tags.push(Invocation!(T_CS!("\\lx@tag@intags"), 
          vec![
            Tokens!(T_OTHER!(role)),
            Invocation!(formatter, vec![ttype.clone()], gullet, state)?
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
  // let remove_empty_element = 
  //   my ($document, $whatsit) = @_;
  //   my $node = $document->getNode->lastChild;    # This should be the wrapper just added.
  //   if (!$node->childNodes) {
  //     $document->removeNode($node); }
  //   return; }

  // \lx@tag[open][close]{stuff}
  DefConstructor!("\\lx@tag[][][]{}", "<ltx:tag open='#1' close='#2'>#4</ltx:tag>",
    bounded => true, mode => Some(s!("text"))
  );
  // // afterConstruct => \&remove_empty_element);

  // \lx@tag@intags{role}{stuff}
  DefConstructor!("\\lx@tag@intags[]{}", "<ltx:tag role='#1'>#2</ltx:tag>",
    bounded => true, mode => Some(s!("text"))
  );
  // // afterConstruct => \&remove_empty_element);

  DefConstructor!("\\lx@tags{}","<ltx:tags>#1</ltx:tags>");
  //afterConstruct => \&remove_empty_element);

  //----------------------------------------------------------------------
  // "refnum" is the lowest level reference number for an object is typically \the<counter>
  // but be sure to use the right counter!  This is how \ref will show the number.
  // You'll typically customize this by defining \the<counter> (and \p@<counter) as in LaTeX.
  DefMacro!("\\lx@counterfor{}", sub[gullet, args, state] {
    unpack!(args => ctr_type);
    let ctr_opt = LookupMapping!("counter_for_type", &ctr_type.to_string(), state);
    if let Some(ctr) = ctr_opt {
      T_OTHER!(ctr).into()
    } else {
      ctr_type.into()
    }
  });
  DefMacro!("\\lx@the@@{}",  "\\expandafter\\lx@@the@@\\expandafter{\\lx@counterfor{#1}}");
  DefMacro!("\\lx@@the@@{}", "\\csname the#1\\endcsname");

  DefMacro!("\\lx@therefnum@@{}", "\\expandafter\\lx@@therefnum@@\\expandafter{\\lx@counterfor{#1}}");
  DefMacro!("\\lx@@therefnum@@{}",
    "{\\normalfont\\csname p@#1\\endcsname\\csname the#1\\endcsname}");

  AssignMapping!("type_tag_formatter", "refnum" => "\\lx@therefnum@@");

  //----------------------------------------------------------------------
  // \lx@fnum@@{type}  Gets the formatted form of the refnum, as part of the object, (no @role).
  // Customize by defining \fnum@<type> or \<type>name and \fnum@font@<type>
  // Default uses \fnum@font@<type> \<type>name prefix + space (if any) and \the<counter>.

  DefMacro!("\\lx@refnum@compose{}{}",  "\\expandafter\\lx@refnum@compose@\\expandafter{#2}{#1}");
  DefMacro!("\\lx@refnum@compose@{}{}", "\\if.#1.#2\\else#2\\space#1\\fi");
  //###DefMacro!("\lx@refnum@compose@{}{}", "\if.#1.#2\else#2~#1\fi");

  DefMacro!("\\lx@fnum@@{}",
    "{\\normalfont\\@ifundefined{fnum@font@#1}{}{\\csname fnum@font@#1\\endcsname}\\@ifundefined{fnum@#1}{\\lx@@fnum@@{#1}}{\\csname fnum@#1\\endcsname}}");

  DefMacro!("\\lx@@fnum@@ {}",
  "\\@ifundefined{#1name}{\\lx@the@@{#1}}{\\lx@refnum@compose{\\csname #1name\\endcsname}{\\lx@the@@{#1}}}");

  AssignMapping!("type_tag_formatter", "" => "\\lx@fnum@@");  // Default!

  //----------------------------------------------------------------------
  // \\lx@fnum@toc@{type} is similar, but formats the number for use within \\toctitle
  // Customize by defining \\fnum@toc@<type> or \\fnum@tocfont@<type>
  // Default uses just \\the<counter>, else composes using \\lx@@fnum@@{type}
  DefMacro!("\\lx@fnum@toc@@{}",
    "{\\normalfont\\@ifundefined{fnum@tocfont@#1}{}{\\csname fnum@tocfont@#1\\endcsname}\\@ifundefined{fnum@toc@#1}{\\lx@the@@{#1}}{\\csname fnum@toc@#1\\endcsname}}");

  //----------------------------------------------------------------------
  // "typerefnum" form is used by automatic cross-references, typically "type number" or similar.
  // Customize by defining \\typerefnum@<type> or \\typerefnum@font@<type>
  // Default uses either \\<type>typerefname or \\<type>name (if any, followed by space, then \\the<counter>
  DefMacro!("\\lx@typerefnum@@{}",
    "{\\normalfont\\@ifundefined{typerefnum@font@#1}{}{\\csname typerefnum@font@#1\\endcsname}\\@ifundefined{typerefnum@#1}{\\lx@@typerefnum@@{#1}}{\\csname typerefnum@#1\\endcsname}}");

  DefMacro!("\\lx@@typerefnum@@{}",
    "\\@ifundefined{#1typerefname}{\\@ifundefined{#1name}{}{\\lx@refnum@compose{\\csname #1name\\endcsname}{\\lx@the@@{#1}}}}{\\lx@refnum@compose{\\csname #1typerefname\\endcsname}{\\lx@the@@{#1}}}");

  AssignMapping!("type_tag_formatter", "typerefnum" => "\\lx@typerefnum@@");


  Ok(())
}
