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

/// Perl: CleanIndexKey — trim whitespace, remove trailing punctuation.
fn clean_index_key(key: &str) -> String {
  let key = key.trim();
  // Remove trailing punctuation
  key.trim_end_matches(['.', ',', ';']).to_string()
}

LoadDefinitions!({
  // #======================================================================
  // # C.11.5 Index and Glossary
  // #======================================================================

  // # ---- The index commands
  // # Format of Index entries:
  // #   \index{entry!entry}  gives multilevel index
  // # Each entry:
  // #   foo@bar  sorts on "foo" but prints "bar"
  // # The entries can end with a |expression:
  // #   \index{...|(}    this page starts a range for foo
  // #   \index{...|)}    this page ends a range
  // #           The last two aren't handled in any particular way.
  // #           We _could_ mark start & end, and then the postprocessor would
  // #           need to fill in all likely links... ???
  // #   \index{...|see{key}}  cross reference.
  // #   \index{...|seealso{key}}  cross reference.
  // #   \index{...|textbf}  (etc) causes the number to be printed in bold!
  // #
  // # I guess the formula is that
  // #    \index{foo|whatever{pi}{pa}{po}}  => \whatever{pi}{pa}{po}{page}
  // # How should this get interpreted??
  // our %index_style = (textbf => 'bold', bf => 'bold', textrm => '', rm => '',
  //   textit => 'italic', it => 'italic', emph => 'italic');    # What else?
  //     # A bit screwy, but....
  //     # Expand \index{a!b!...} into \@index{\@indexphrase{a}\@indexphrase{b}...}

  // sub process_index_phrases {
  //   my ($gullet, $phrases) = @_;
  //   my @expansion = ();
  //   # Split the text into phrases, separated by "!"
  //   my @tokens = $phrases->unlist;
  //   return unless @tokens;
  //   push(@tokens, T_OTHER('!')) unless $tokens[-1]->getString eq '!';    # Add terminal !
  //   my @phrase = ();
  //   my @sortas = ();
  //   my $style;
  //   while (@tokens) {
  //     my $tok    = shift(@tokens);
  //     my $string = $tok->getString;
  //     if ($string eq '"') {
  //       push(@phrase, shift(@tokens)); }
  //     elsif ($string eq '@') {
  //       while (@phrase && ($phrase[-1]->getString =~ /\s/)) { pop(@phrase); }    # Trim
  //       @sortas = @phrase; @phrase = (); }
  //     elsif (($string eq '!') || ($string eq '|')) {
  //       while (@phrase && ($phrase[-1]->getString =~ /\s/)) { pop(@phrase); }    # Trim
  //       push(@expansion, T_CS('\@indexphrase'),
  //         (@sortas ? (T_OTHER('['), @sortas, T_OTHER(']')) : ()),
  //         T_BEGIN, @phrase, T_END)
  //         if @phrase;
  //       @phrase = (); @sortas = ();
  //       if ($string eq '|') {
  //         pop(@tokens);    # Remove the extra "!" stopbit.
  //         my $extra = ToString(Tokens(@tokens));
  //         if ($extra =~ /^see\s*{/) { push(@expansion, T_CS('\@indexsee'), @tokens[3 ..
  // $#tokens]); }         elsif ($extra =~ /^seealso\s*\{/) { push(@expansion,
  // T_CS('\@indexseealso'), @tokens[7 .. $#tokens]); }         elsif ($extra eq '(') { $style =
  // 'rangestart'; }                     # ?         elsif ($extra eq ')') { $style = 'rangeend';
  // }                       # ?         else                  { $style = $index_style{$extra} ||
  // $extra; }         @tokens = (); } }
  //     elsif (!@phrase && ($string =~ /\s/)) { }                                # Skip leading
  // whitespace     else {
  //       push(@phrase, $tok); } }
  //   @expansion = (T_CS('\@index'),
  //     ($style ? (T_OTHER('['), T_OTHER($style), T_OTHER(']')) : ()),
  //     T_BEGIN, @expansion, T_END);
  //   return (T_BEGIN,T_CS('\normalfont'), @expansion, T_END); }

  // DefMacro('\index{}', \&process_index_phrases);

  Tag!("ltx:indexphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });
  Tag!("ltx:glossaryphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });
  // ltx:indexsee does NOT get a key (at this stage)

  // DefConstructor('\@index[]{}', "^<ltx:indexmark style='#1'>#2</ltx:indexmark>",
  //   mode => 'text', reversion => '', sizer => 0);

  // DefConstructor('\@indexphrase[]{}',
  //   "<ltx:indexphrase key='#key' _standalone_font='true'>#2</ltx:indexphrase>",
  //   properties => { key => sub { CleanIndexKey($_[1]); } });
  // DefConstructor('\@indexsee{}',
  //   "<ltx:indexsee key='#key' name='#name' _standalone_font='true'>#1</ltx:indexsee>",
  //   properties => { name => sub { DigestIf('\seename') } });

  // DefConstructor('\@indexseealso{}',
  //   "<ltx:indexsee key='#key' name='#name' _standalone_font='true'>#1</ltx:indexsee>",
  //   properties => { name => sub { DigestIf('\alsoname') } });

  // DefConstructor('\glossary{}',
  //   "<ltx:glossaryphrase role='glossary' key='#key'>#1</ltx:glossaryphrase>",
  //   properties => { key => sub { CleanIndexKey($_[1]); } },
  //   sizer => 0);

  // #======================================================================
  // # This converts an indexphrase node into a sortable string.
  // # Seems the XML nodes are the best place to handle it (rather than Boxes),
  // # although some of the special cases (see, @, may end up tricky)
  // sub indexify {
  //   my ($node, $document) = @_;
  //   my $type = $node->nodeType;
  //   if ($type == XML_TEXT_NODE) {
  //     my $string = $node->textContent;
  //     $string =~ s/\W//g;    # to be safe (if perhaps non-unique?)
  //     $string =~ s/\s//g;    # Or remove entirely? Eventually worry about many=>1 mapping???
  //     return $string; }
  //   elsif ($type == XML_ELEMENT_NODE) {
  //     if ($document->getModel->getNodeQName($node) eq 'ltx:Math') {
  //       return indexify_tex($node->getAttribute('tex')); }
  //     else {
  //       return join('', map { indexify($_, $document) } $node->childNodes); } }
  //   elsif ($type == XML_DOCUMENT_FRAG_NODE) {
  //     return join('', map { indexify($_, $document) } content_nodes($node)); }
  //   else {
  //     return ""; } }

  // # Try to clean up a TeX string into something
  // # Could walk the math tree and handle XMDual specially, but need to xref args.
  // # But also we'd have unicode showing up, which we'd like to latinize...
  // sub indexify_tex {
  //   my ($string) = @_;
  //   $string =~ s/(\\\@|\\,|\\:|\\;|\\!|\\ |\\\/|)//g;
  //   $string =~
  // s/(\\mathrm|\\mathit|\\mathbf|\\mathsf|\\mathtt|\\mathcal|\\mathscr|\\mbox|\\rm|\\it|\\bf|\\
  // tt|\\small|\\tiny)//g;   $string =~ s/\\left\b//g; $string =~ s/\\right\b//g;
  //   $string =~ s/(\\|\{|\})//g;
  //   $string =~ s/\W//g;    # to be safe (if perhaps non-unique?)
  //   $string =~ s/\s//g;    # Or remove entirely? Eventually worry about many=>1 mapping???
  //   return $string; }

  // # ---- Creating the index itself

  // AssignValue(INDEXLEVEL => 0);

  // Tag('ltx:indexentry', autoClose => 1);

  // sub closeIndexPhrase {
  //   my ($document) = @_;
  //   if ($document->isCloseable('ltx:indexphrase')) {
  //     $document->closeElement('ltx:indexphrase'); }
  //   return; }

  // sub doIndexItem {
  //   my ($document, $level) = @_;
  //   $document->closeElement('ltx:indexrefs') if $document->isCloseable('ltx:indexrefs');
  //   closeIndexPhrase($document);
  //   my $l = LookupValue('INDEXLEVEL');
  //   while ($l < $level) {
  //     $document->openElement('ltx:indexlist'); $l++; }
  //   while ($l > $level) {
  //     $document->closeElement('ltx:indexlist'); $l--; }
  //   AssignValue(INDEXLEVEL => $l);
  //   if ($level) {
  //     $document->openElement('ltx:indexentry');
  //     $document->openElement('ltx:indexphrase'); }
  //   return; }

  // DefConstructor('\index@dotfill', undef, sub {
  //     my ($document) = @_;
  //     closeIndexPhrase($document);
  //     $document->openElement('ltx:indexrefs'); });
  // DefConstructor('\index@item',       undef, sub { doIndexItem($_[0], 1); });
  // DefConstructor('\index@subitem',    undef, sub { doIndexItem($_[0], 2); });
  // DefConstructor('\index@subsubitem', undef, sub { doIndexItem($_[0], 3); });
  // DefConstructor('\index@done',       undef, sub { doIndexItem($_[0], 0); });

  DefMacro!("\\indexname", "Index");
  // Simplified {theindex} — Perl has complex index item handling
  DefEnvironment!("{theindex}",
    "<ltx:index xml:id='#id'>#body</ltx:index>");

  // Perl: latex_constructs.pool.ltxml L4587
  DefPrimitive!("\\indexspace", None);
  DefPrimitive!("\\makeindex", None);
  DefPrimitive!("\\makeglossary", None);

  // Stub for \index — just discard the argument for now.
  // Full process_index_phrases expansion is deferred.
  DefPrimitive!("\\index {}", None);
});
