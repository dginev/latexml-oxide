use crate::package::*;

//**********************************************************************
// C.11 Moving Information Around
//**********************************************************************
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //======================================================================
  // C.11.1 Files
  //======================================================================
  DefPrimitiveI!("\\nofiles", noprimitive!());

  //======================================================================
  // C.11.2 Cross-References
  //======================================================================

  // \label attaches a label to the nearest parent that can accept a labels attribute
  // but only those that have an xml:id (but should this require a refnum and/or title ???)
  // Note that latex essentially allows redundant labels, but we can record only one!!!
  DefConstructor!("\\label Semiverbatim", sub[document, olabel, props, state] {
    if let Some(savenode) = document.float_to_label() {
      let mut node = document.get_node();
      let mut labels : HashMap<String,bool> = HashMap::new();
      if let Some(label) = props.get("label") {
        labels.insert(label.to_string(), true);
      }
      let labels_iter = node.get_attribute("labels").unwrap_or_default().split_whitespace();
      for label in labels_iter {
        labels.insert(label.to_string(), true);
      }
      document.set_attribute(&mut node, "labels", labels.keys().collect().join(' '));
      document.set_node(&savenode);
    }
  }
  //   reversion   => '',
  //   properties  => { alignmentSkippable => 1, alignmentPreserve => 1 },
  //   afterDigest => sub {
  //     my $label = CleanLabel(ToString($_[1]->getArg(1)));
  //     $_[1]->setProperty(label => $label);
  //     my $scope = $label; $scope =~ s/^LABEL:/label:/;
  //     if (my $ctr = LookupValue('current_counter')) {
  //       unshift(@{ LookupValue('scopes_for_counter:' . $ctr) }, $scope);
  //       $STATE->activateScope($scope);
  //       $_[0]->beginMode('text');
  //       AssignValue('LABEL@' . $label, Digest(T_CS('\@currentlabel')), 'global');
  //       $_[0]->endMode('text'); }
  );

  // # If a node has been labeled, but still  hasn't yet got an id by afterClose:late,
  // # we'd better generate an id for it.
  // Tag('ltx:*', 'afterClose:late' => sub {
  //     my ($document, $node) = @_;
  //     if ($node->hasAttribute('labels') && !($node->hasAttribute('xml:id'))) {
  //       GenerateID($document, $node); } });

  // # These will get filled in during postprocessing.
  // # * is added to accommodate hyperref
  DefConstructor!(
    "\\ref OptionalMatch:* Semiverbatim",
    "<ltx:ref labelref='#label' _force_font='true'/>"
  );
  // TODO
  // properties => sub { (label => CleanLabel($_[2])); });

  // DefConstructor('\pageref OptionalMatch:* Semiverbatim', "<ltx:ref labelref='#label'
  // _force_font='true'/>", # Same??   properties => sub { (label => CleanLabel($_[2])); });
  // #======================================================================
  // # C.11.3 Bibliography and Citation
  // #======================================================================

  // # Note that it's called \refname in LaTeX's article, but \bibname in report & book.
  // # And likewise, mixed up in various other classes!

  DefMacro!("\\thebibliography@ID", "");

  // # This sub does things that would commonly be needed when starting a bibliography
  // # setting the ID, etc...
  // sub beginBibliography {
  //   my ($whatsit) = @_;
  //   beginBibliography_clean($whatsit);
  //   # Fix for missing \bibitems!
  //   setupPseudoBibitem();
  //   return; }

  // sub beginBibliography_clean {
  //   my ($whatsit) = @_;
  //   # Try to compute a reasonable, but unique ID;
  //   # relative to the document's ID, if any.
  //   # But also, if there are multiple bibliographies,
  //   my $bibnumber = LookupValue('n_bibliographies') || 0;
  //   AssignValue(n_bibliographies => ++$bibnumber, 'global');
  //   my $docid = ToString(Expand(T_CS('\thedocument@ID')));
  //   my $bibid = ($docid ? $docid . '.' : '') . 'bib' . radix_alpha($bibnumber - 1);
  //   DefMacroI(T_CS('\thebibliography@ID'), undef, T_OTHER($bibid), scope => 'global');
  //   #  $whatsit->setProperty(id=>ToString(Expand(T_CS('\thebibliography@ID'))));
  //   $whatsit->setProperty(id => $bibid);
  //   my $title = DigestIf('\refname') || DigestIf('\bibname');
  //   $whatsit->setProperty(title     => $title)          if $title;
  //   $whatsit->setProperty(titlefont => $title->getFont) if $title;
  //   $whatsit->setProperty(bibstyle  => LookupValue('BIBSTYLE'));
  //   $whatsit->setProperty(citestyle => LookupValue('CITE_STYLE'));
  //   #  $whatsit->setProperty(sort=> ???
  //   # And prepare for the likely nonsense that appears within bibliographies
  //   ResetCounter('enumiv');
  //   return; }

  // DefMacro('\bibliography Semiverbatim',
  // '\lx@ifusebbl{#1}{\input{\jobname.bbl}}{\@bibliography{#1}}'); DefMacro('\lx@ifusebbl{}{}{}',
  // sub {     my ($gullet, $bib_files, $bbl_clause, $bib_clause) = @_;
  //     $bib_files = ToString(Expand($bib_files));
  //     return unless $bib_files;
  //     my $jobname = ToString(Expand(T_CS('\jobname')));

  //     my $bbl_path = FindFile($jobname, type => 'bbl');
  //     my $missing_bibs = '';
  //     for my $bf (split(',', $bib_files)) {
  //       my $bib_path = FindFile($bf, type => 'bib');
  //       if (not $bib_path) {
  //         $missing_bibs .= ',' unless length($missing_bibs) == 0;
  //         $missing_bibs .= $bf; } }

  //     if (length($missing_bibs) == 0 or not $bbl_path) {
  //       return $bib_clause->unlist; }
  //     else {
  //       Info('expected', $missing_bibs, $_[0], "Couldn't find all bib files, using " . $jobname .
  // ".bbl instead");       return $bbl_clause->unlist; } });
  // DefConstructor('\@bibliography Semiverbatim',
  //   "<ltx:bibliography files='#1' xml:id='#id' "
  //     . "bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>"
  //     . "<ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>"
  //     . "</ltx:bibliography>",
  //   afterDigest => sub { $_[0]->begingroup;    # wrapped so redefns don't take effect!
  //     beginBibliography($_[1]);
  //     $_[0]->endgroup; });

  // # NOTE: This totally needs to be made extensible (parsing *.bst!?!? OMG!)
  // our $BIBSTYLES = {
  //   plain    => { citestyle => 'numbers', sort => 'true' },
  //   unsrt    => { citestyle => 'numbers', sort => 'false' },
  //   alpha    => { citestyle => 'AY',      sort => 'true' },
  //   abbrv    => { citestyle => 'numbers', sort => 'true' },
  //   plainnat => { citestyle => 'numbers', sort => 'true' },
  //   unsrtnat => { citestyle => 'numbers', sort => 'false' },
  //   alphanat => { citestyle => 'AY',      sort => 'true' },
  //   abbrvnat => { citestyle => 'numbers', sort => 'true' } };

  // DefConstructor('\bibstyle{}', sub {
  //     my ($document, $style) = @_;
  //     $style = ToString($style);
  //     if (my $bib = $document->findnode('//ltx:bibliography')) {
  //       $document->setAttribute($bib, bibstyle => $style);
  //       if (my $parms = $$BIBSTYLES{$style}) {
  //         $document->setAttribute($bib, citestyle => $$parms{citestyle});
  //         $document->setAttribute($bib, sort      => $$parms{sort});
  //   } } },
  //   afterDigest => sub {
  //     my $style = ToString($_[1]->getArg(1));
  //     AssignValue(BIBSTYLE => $style, 'global');
  //     if (my $parms = $$BIBSTYLES{$style}) {
  //       AssignValue(CITE_STYLE => $$parms{citestyle}); }
  //     else {
  //       Info('unexpected', $style, $_[0], "Unknown bibstyle '$style', it will be ignored"); }
  //     return; });

  // DefMacro('\bibliographystyle Semiverbatim', '\bibstyle{#1}');

  DefConditional!("\\if@lx@inbibliography");
  // Should be an environment, but people seem to want to misuse it.
  DefConstructor!("\\thebibliography",
    "<ltx:bibliography xml:id='#id'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title><ltx:biblist>",
     before_digest => beforesub!(stomach, state, {
        state.assign_value("inPreamble", false, None);
        Ok(vec![stomach.digest(Tokens!(T_CS!("\\@lx@inbibliographytrue")), state)?])
    }),
    after_digest => afterproc!(stomach, whatsit, state, {
      // NOTE that in some perverse situations (revtex?)
      // it seems to be allowable to omit the argument
      // It's ignorable for latexml anyway, so we'll just read it if its there.
      let gullet = stomach.get_gullet_mut();
      gullet.skip_spaces(state);
      if gullet.if_next(T_BEGIN!(), state)? {
        gullet.read_arg(state)?;
      }
      // TODO:
      // beginBibliography(whatsit); },
    })
  //   locked => 1);
  );

  // Close the bibliography
  DefConstructor!("\\endthebibliography", "</ltx:biblist></ltx:bibliography>"); //, TODO:
                                                                                //   afterDigest => sub { my $t = T_CS('\@appendix');
                                                                                //     Digest($t) if IsDefined($t);
                                                                                //     return; },
                                                                                //   locked => 1);
                                                                                // # auto close the bibliography and contained biblist.
                                                                                // Tag('ltx:biblist',      autoClose => 1);
                                                                                // Tag('ltx:bibliography', autoClose => 1);

  // # Since SOME people seem to write bibliographies w/o \bibitem,
  // # just blank lines between apparent entries,
  // # Making \par do a \bibitem{} works, but screws up valid
  // # bibliographies with blank lines!
  // # So, let's do some redirection!
  // sub setupPseudoBibitem {
  //   Let('\save@bibitem', '\bibitem');
  //   Let('\save@par',     '\par');
  //   Let('\bibitem',      '\restoring@bibitem');
  //   Let('\par',          '\par@in@bibliography');
  //   # Moreover, some people use \item instead of \bibitem
  //   Let('\item', '\item@in@bibliography');
  //   # And protect from redefinitions.
  //   Let('\newblock', '\lx@bibnewblock');
  //   return; }

  // DefMacroI('\par@in@bibliography', undef, sub {
  //     my ($gullet) = @_;
  //     $gullet->skipSpaces;
  //     my $tok = $gullet->readToken;
  //     # If next token is another \par, or a REAL \bibitem,
  //     if (Equals($tok, T_CS('\par')) || Equals($tok, T_CS('\bibitem'))) {
  //       ($tok); }    # then this \par expands into what followed
  //     else {         # Else, put it back, and start a bibitem.
  //       $gullet->unread($tok);
  //       (T_CS('\save@bibitem'), T_BEGIN, T_END); } });

  // DefMacroI('\item@in@bibliography', undef, '\save@bibitem{}');

  // # If we hit a real \bibitem, put \par & \bibitem back to correct defn, and then \bibitem.
  // # A bibitem with now key or label...
  // DefMacro('\restoring@bibitem',
  //   '\let\bibitem\save@bibitem\let\par\save@par\bibitem');

  // NewCounter('@bibitem', 'bibliography', idprefix => 'bib');
  // DefMacroI('\the@bibitem', undef, '\arabic{@bibitem}');
  // DefMacro('\@biblabel{}', '[#1]');
  // DefMacroI('\fnum@@bibitem', undef, '{\@biblabel{\the@bibitem}}');
  // # Hack for abused bibliographies; see below
  DefMacro!("\\bibitem",
    "\\if@lx@inbibliography\\else\\expandafter\\lx@mung@bibliography\\expandafter{\\@currenvir}\\fi\\lx@bibitem");
  DefConstructor!("\\lx@bibitem[] Semiverbatim", "<ltx:bibitem key='#key' xml:id='#id'>#tags<ltx:bibblock>",
    after_digest => afterproc!(stomach, whatsit, state, {
      let tag_opt = whatsit.get_arg(1);
      let key = clean_bib_key(&match whatsit.get_arg(2) {
        None => String::new(),
        Some(key) => key.to_string(),
      });
      // if let Some(tag) = tag_opt {
      //   whatsit.set_properties(
      //     key => $key,
      //     RefStepID('@bibitem'),
      //     tags => Digest(T_BEGIN,
      //       T_CS('\def'), T_CS('\the@bibitem'), T_BEGIN, Revert($tag), T_END,
      //       Invocation(T_CS('\lx@make@tags'), T_OTHER('@bibitem')),
      //       T_END)); }
      // else {
      //   whatsit.set_properties(key => $key, RefStepCounter('@bibitem'));
      // }
    })
  );

  // This attempts to handle the case where folks put \bibitem's within an enumerate or such.
  // We try to close the list and open the bibliography
  DefMacro!("\\lx@mung@bibliography{}", sub[gullet, args, state] {
    unpack!(args => env);
    let tag = env.to_string();
    let mut tokens = Vec::new();
    // If we're in some sort of list environment, maybe we can recover
    if tag == "enumerate" || tag == "itemize" || tag == "description" {
      info!("\nDamn! We're in a list {}; try to close it!", tag);
      tokens.extend(Invocation!("\\end", env.unlist(), gullet, state)?.unlist());
      tokens.extend(vec![
        T_CS!("\\let"),
        T_CS!(&format!("\\end{}", tag)),
        T_CS!("\\endthebibliography"),
        T_CS!("\\let"),
        T_CS!(&format!("\\end{{{}}}", tag)),
        T_CS!("\\end{thebibliography}")
      ]);
    }
    // else ? it probably isn't going to work??
    info!("Now, try to open {{thebibliography}}");
    tokens.extend(Invocation!("\\begin", vec![Tokenize!("thebibliography"), Tokens!()], gullet, state)?.unlist());
    let tokens = Tokens::new(tokens);
    info!("PATCHING with {:?}", tokens.to_string());
    Ok(tokens)
  });

  // DefConstructorI('\lx@bibnewblock', undef, "<ltx:bibblock>");
  // Let('\newblock', '\lx@bibnewblock');
  // Tag('ltx:bibitem',  autoClose => 1);
  // Tag('ltx:bibblock', autoClose => 1);

  // #----------------------------------------------------------------------
  // # We've got the same problem as LaTeX: Lather, Rinse, Repeat.
  // # It would be nice to know the bib info at digestion time
  // #  * whether author lists will collapse
  // #  * whether there are "a","b".. extensions on the year.
  // # We could process the bibliography first, (IF it is a separate *.bib!)
  // # but won't know which entries are included (and so can't resolve the a/b/c..)
  // # until we've finished looking at (all of) the source(s) that will refer to them!
  // #
  // # We can do this in 2 passes, however
  // #  (1) convert (latexml) both the source document(s) and the bibliography
  // #  (2) extract the required bibitems and integrate (latexmlpost) it into the documents.
  // # [Note that for mult-document sites, step (2) becomes 2 stages: scan and integrate]
  // #
  // # Here's the general layout.
  // #   <ltx:cite> contains everything that the citations produce,
  // #     including parens, pre-note, punctunation that precede the <ltx:bibcite>
  // #     and punctuation, post-note, parens, that follow it.
  // #   <ltx:bibcite show="string" bibrefs="keys" sep="" yysep="">phrases</ltx:bibcite>
  // #     encodes the actual citation.
  // #
  // #     bibrefs : lists the bibliographic keys that will be used
  // #     show    : gives the pattern for formatting using data from the bibliography
  // #       It can contain:
  // #         authors or fullauthors
  // #         year
  // #         number
  // #         phrase1,phrase2,... selects one of the phrases from the content of the <ltx:bibref>
  // #     This format is used as follows:
  // #       If author and year is present, and a subset of the citations share the same authors,
  // #         then the format is used, but the year is repeated for each citation in the subset,
  // #         as a link to the bib entry.
  // #       Otherwise, the format is applied to each entry.
  // #
  // # The design is intended to support natbib, as well as plain LaTeX.

  // AssignValue(CITE_STYLE          => 'numbers');
  // AssignValue(CITE_OPEN           => T_OTHER('['));
  // AssignValue(CITE_CLOSE          => T_OTHER(']'));
  // AssignValue(CITE_SEPARATOR      => T_OTHER(','));
  // AssignValue(CITE_YY_SEPARATOR   => T_OTHER(','));
  // AssignValue(CITE_NOTE_SEPARATOR => T_OTHER(','));

  // DefConstructor('\@@cite []{}', "<ltx:cite ?#1(class='ltx_citemacro_#1')>#2</ltx:cite>",
  //   mode => 'text');

  // # \@@bibref{what to show}{bibkeys}{phrase1}{phrase2}
  // DefConstructor('\@@bibref Semiverbatim Semiverbatim {}{}',
  //   "<ltx:bibref show='#1' bibrefs='#bibrefs'"
  //     . " separator='#separator' yyseparator='#yyseparator'>#3#4</ltx:bibref>",
  //   properties => sub { (bibrefs => CleanBibKey($_[2]),
  //       separator   => ToString(Digest(LookupValue('CITE_SEPARATOR'))),
  //       yyseparator => ToString(Digest(LookupValue('CITE_YY_SEPARATOR')))); });

  // # Simple container for any phrases used in the bibref
  // DefConstructor('\@@citephrase{}', "<ltx:bibrefphrase>#1</ltx:bibrefphrase>",
  //   mode => 'text');

  DefMacro!("\\cite[] Semiverbatim", sub[gullet, args, state] {
    unpack!(args => post, keys);
  //     my ($style, $open, $close, $ns)
  //       = map { LookupValue($_) } qw(CITE_STYLE CITE_OPEN CITE_CLOSE CITE_NOTE_SEPARATOR);
  //     $post = undef unless $post && $post->unlist;
  //     Invocation(T_CS('\@@cite'),
  //       Tokens(Explode('cite')),
  //       Tokens($open,
  //         Invocation(T_CS('\@@bibref'), undef, $keys, undef, undef),
  //         ($post ? ($ns, T_SPACE, $post) : ()), $close)); });
    Ok(Tokens!())
  });

  // # NOTE: Eventually needs to be recognized by MakeBibliography
  // DefConstructor('\nocite Semiverbatim',
  //   "<ltx:cite><ltx:bibref show='nothing' bibrefs='#bibrefs'/></ltx:cite>",
  //   properties => sub { (bibrefs => CleanBibKey($_[1])) });

  // #======================================================================
  // # C.11.4 Splitting the input
  // #======================================================================
  // Let('\@@input', '\input');    # Save TeX's version.
  //                               # LaTeX's \input is a bit different...
  // DefMacroI('\input', undef, '\@ifnextchar\bgroup\@iinput\@@input');
  // DefPrimitive('\@iinput {}', sub { Input(Expand($_[1])); });

  // # Note that even excluded files SHOULD have the effects of their inclusion
  // # simulated by having read the corresponding aux file;
  // # But we're not bothering with that.
  // DefPrimitive('\include{}', sub {
  //     my ($stomach, $path) = @_;
  //     $path = ToString($path);
  //     my $table = LookupValue('including@only');
  //     if (!$table || $$table{$path}) {
  //       Input($path); }
  //     return; });

  // # [note, this will load name.tex, if it exists, else name]
  // DefPrimitive('\includeonly{}', sub {
  //     my ($stomach, $paths) = @_;
  //     $paths = ToString($paths);
  //     my $table = LookupValue('including@only');
  //     AssignValue('including@only', $table = {}, 'global') unless $table;
  //     map { $$table{$_} = 1 } map { /^\s*(.*?)\s*$/ && $1; } split(/,/, $paths);
  //     return; });

  // # NOTE: In the long run, we want to SAVE the contents and associate them with the given file
  // name #  AND, arrange so that when a file is read, we'll use the contents!
  // DefConstructorI(T_CS("\\begin{filecontents}"), "Semiverbatim",
  //   '',
  //   reversion   => '',
  //   afterDigest => [sub {
  //       my ($stomach, $whatsit) = @_;
  //       my $filename = ToString($whatsit->getArg(1));
  //       my @lines    = ();
  //       my $gullet   = $stomach->getGullet;
  //       my $line;
  //       while (defined($line = $gullet->readRawLine) && ($line ne '\end{filecontents}')) {
  //         push(@lines, $line); }
  //       AssignValue($filename . '_contents' => join("\n", @lines), 'global');
  //       NoteProgress("[Cached filecontents for $filename (" . scalar(@lines) . " lines)]"); }]);
  // DefConstructorI(T_CS("\\begin{filecontents*}"), "Semiverbatim",
  //   '',
  //   reversion   => '',
  //   afterDigest => [sub {
  //       my ($stomach, $whatsit) = @_;
  //       my $filename = ToString($whatsit->getArg(1));
  //       my @lines    = ();
  //       my $gullet   = $stomach->getGullet;
  //       my $line;
  //       while (defined($line = $gullet->readRawLine) && ($line ne '\end{filecontents*}')) {
  //         push(@lines, $line); }
  //       AssignValue($filename . '_contents' => join("\n", @lines), 'global');
  //       NoteProgress("[Cached filecontents* for $filename (" . scalar(@lines) . " lines)]"); }]);
  // DefMacro('\endfilecontents', '');
  // DefPrimitive('\listfiles', undef);

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

  // Tag('ltx:indexphrase',    afterClose => \&addIndexPhraseKey);
  // Tag('ltx:glossaryphrase', afterClose => \&addIndexPhraseKey);
  // ### ltx:indexsee does NOT get a key (at this stage)!

  // sub addIndexPhraseKey {
  //   my ($document, $node) = @_;
  //   if (!$node->getAttribute('key')) {
  //     $node->setAttribute(key => CleanIndexKey($node->textContent)); }
  //   return; }

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
  // s/(\\mathrm|\\mathit|\\mathbf|\\mathsf|\\mathtt|\\mathcal|\\mathscr|\\mbox|\\rm|\\it|\\bf|\\tt|\\small|\\tiny)//g;
  //   $string =~ s/\\left\b//g; $string =~ s/\\right\b//g;
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

  // DefConstructorI('\index@dotfill', undef, sub {
  //     my ($document) = @_;
  //     closeIndexPhrase($document);
  //     $document->openElement('ltx:indexrefs'); });
  // DefConstructorI('\index@item',       undef, sub { doIndexItem($_[0], 1); });
  // DefConstructorI('\index@subitem',    undef, sub { doIndexItem($_[0], 2); });
  // DefConstructorI('\index@subsubitem', undef, sub { doIndexItem($_[0], 3); });
  // DefConstructorI('\index@done',       undef, sub { doIndexItem($_[0], 0); });

  // DefMacroI('\indexname', undef, 'Index');
  // DefEnvironment('{theindex}',
  //   "<ltx:index xml:id='#id'>"
  //     . "<ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>"
  //     . "#body"
  //     . "</ltx:index>",
  //   beforeDigest => sub {
  //     Let('\item',       '\index@item');
  //     Let('\subitem',    '\index@subitem');
  //     Let('\subsubitem', '\index@subsubitem');
  //     Let('\dotfill',    '\index@dotfill'); },
  //   beforeDigestEnd => sub { Digest(T_CS('\index@done')); },
  //   afterDigestBegin => sub {
  //     my $docid = ToString(Expand(T_CS('\thedocument@ID')));
  //     my $title = DigestIf('\indexname');
  //     $_[1]->setProperties(id => ($docid ? "$docid.idx" : 'idx'),
  //       title     => $title,
  //       titlefont => $title->getFont); });

  // DefPrimitiveI('\indexspace',   undef, undef);
  // DefPrimitiveI('\makeindex',    undef, undef);
  // DefPrimitiveI('\makeglossary', undef, undef);

  // #======================================================================
  // # C.11.6 Terminal Input and Output
  // #======================================================================

  // DefPrimitive('\typeout{}', sub {
  //     my ($stomach, $stuff) = @_;
  //     print STDERR ToString(Expand($stuff)) . "\n" if LookupValue('VERBOSITY') > -1;
  //     return; });

  // DefPrimitive('\typein[]{}', undef);

  Ok(())
}
