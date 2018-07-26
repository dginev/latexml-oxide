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
    ObjectStore::HashTagData(HashMap::new()),
    Some(Scope::Global)
  );

  // // Add a new frontmatter item that will be enclosed in <$tag %attr>...</$tag>
  // // The content is the result of digesting $tokens.
  // // \@add@frontmatter[keys]{tag}[attributes]{content}
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
      state.assign_value("inPreamble", ObjectStore::Bool(false), None);
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
          Some(&mut ObjectStore::HashTagData(ref mut frnt)) => frnt,
          _ => fatal!(
            TexPool,
            Expected,
            "Global TeX Frontmatter hash was not available, should never happen"
          ),
        };
        let f_entry = frontmatter.entry(tag.to_string()).or_insert(Vec::new());
        f_entry.push(entry);
      }
      state.assign_value("inPreamble", ObjectStore::Bool(inpreamble), None);
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
    ].iter()
      .map(|s| s.to_string())
      .collect();

    let mut frontmatter = match state.remove_value("frontmatter") {
      Some(ObjectStore::HashTagData(frnt)) => frnt,
      _ => fatal!(
        TexPool,
        Expected,
        "Global TeX Frontmatter hash was not available, should never happen"
      ),
    };
    state.assign_value(
      "frontmatter",
      ObjectStore::HashTagData(HashMap::new()),
      Some(Scope::Global),
    );
    let state_keys: HashSet<String> = frontmatter.keys().cloned().collect();
    let mut all_keys: HashSet<String> = frontmatter_elements.union(&state_keys).cloned().collect();
    for key in all_keys.iter() {
      if let Some(list) = frontmatter.remove(key) {
        // Dubious, but assures that frontmatter appears in text mode...
        // TODO:
        //local $LaTeXML::BOX = Box('', $STATE->lookupValue('font'), '', T_SPACE);
        document.box_to_absorb = Some(Digested::Box(Tbox::new(
          String::new(),
          state.lookup_font(),
          None,
          Tokens!(T_SPACE!()),
          HashMap::new(),
          state,
        )));
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
  Ok(())
}
