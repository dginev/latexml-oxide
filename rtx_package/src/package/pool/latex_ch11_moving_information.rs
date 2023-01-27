use crate::package::*;

//**********************************************************************
// C.11 Moving Information Around
//**********************************************************************
LoadDefinitions!(outer_stomach, outer_state, {
  //======================================================================
  // C.11.1 Files
  //======================================================================
  DefPrimitive!("\\nofiles", None);

  //======================================================================
  // C.11.2 Cross-References
  //======================================================================

  // \label attaches a label to the nearest parent that can accept a labels attribute
  // but only those that have an xml:id (but should this require a refnum and/or title ???)
  // Note that latex essentially allows redundant labels, but we can record only one!!!
  DefConstructor!("\\label Semiverbatim", sub[document, olabel, props, state] {
    if let Some(savenode) = document.float_to_label(state) {
      let mut labels : HashMap<String,bool> = HashMap::new();
      if let Some(label) = props.get("label") {
        labels.insert(label.to_string(), true);
      }
      for label in document.node_get_attribute("labels").unwrap_or_default().split_whitespace() {
        labels.insert(label.to_string(), true);
      }
      document.node_set_attribute("labels",
         &labels.keys().map(ToString::to_string).collect::<Vec<_>>().join(" "),
         state)?;
      document.set_node(&savenode);
    }
  },
  reversion => "", // TODO: implement for DUAL_BRANCH
  properties => {stored_map!("alignmentSkippable" => true, "alignmentPreserve" => true)},
  after_digest => sub[stomach, whatsit, state] {
    let label = match whatsit.get_arg(1) {
      Some(labeld) => clean_label(&labeld.to_string(), None).into_owned(),
      None => String::new()
    };
    let mut scope = label.replace("LABEL:","label:");
    let label_key = s!("LABEL@{}", label);
    whatsit.set_property("label", label);
    let cc = state.lookup_value("current_counter");
    let ctr_key_opt = cc.map(|ctr| s!("scopes_for_counter:{}", ctr));
    if let Some(ctr_key) = ctr_key_opt {
      // TODO: we should probably improve the ergonomics here to avoid the vec![]
      state.unshift_value(&ctr_key, vec![scope.clone()]);
      state.activate_scope(&scope);
      stomach.begin_mode("text", state)?;
      let current_label = stomach.digest(Tokens!(T_CS!("\\@currentlabel")), state)?;
      state.assign_value(&label_key, current_label, Some(Scope::Global));
      stomach.end_mode("text", state)?;
    }
  }
  );

  // # If a node has been labeled, but still  hasn't yet got an id by afterClose:late,
  // # we'd better generate an id for it.
  // Tag('ltx:*', 'afterClose:late' => sub {
  //     my ($document, $node) = @_;
  //     if ($node->hasAttribute('labels') && !($node->hasAttribute('xml:id'))) {
  //       GenerateID($document, $node); } });

  // # These will get filled in during postprocessing.
  // # * is added to accommodate hyperref
  DefConstructor!("\\ref OptionalMatch:* Semiverbatim",
    "<ltx:ref labelref='#label' _force_font='true'/>",
    properties => sub[stomach, args, state] {
      unpack_opt_ref!(args => _star, label_opt);
      let label = label_opt.unwrap().as_ref().unwrap().to_string();
      Ok(map!("label" => Stored::String(clean_label(&label, None).into_owned())))
  });

  // DefConstructor('\pageref OptionalMatch:* Semiverbatim', "<ltx:ref labelref='#label'
  // _force_font='true'/>", # Same??   properties => sub { (label => CleanLabel($_[2])); });
  // #======================================================================
  // # C.11.3 Bibliography and Citation
  // #======================================================================

  // # Note that it's called \refname in LaTeX's article, but \bibname in report & book.
  // # And likewise, mixed up in various other classes!

  DefMacro!("\\thebibliography@ID", "");

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
     before_digest => sub[stomach, state] {
        AssignValue!("inPreamble", false);
        Ok(vec![stomach.digest(Tokens!(T_CS!("\\@lx@inbibliographytrue")), state)?])
    },
    after_digest => sub[stomach, whatsit, state] {
      // NOTE that in some perverse situations (revtex?)
      // it seems to be allowable to omit the argument
      // It's ignorable for latexml anyway, so we'll just read it if its there.
      let gullet = stomach.get_gullet_mut();
      gullet.skip_spaces(state);
      if gullet.if_next(T_BEGIN!(), state)? {
        gullet.read_arg(state)?;
      }
      begin_bibliography(stomach, whatsit, state)?;
    },
    locked => true
  );

  // Close the bibliography
  DefConstructor!("\\endthebibliography", sub[document,whatsit,props,state] {
    document.maybe_close_element("ltx:biblist", state)?;
    document.maybe_close_element("ltx:bibliography", state)?;
  }, locked=>true);
  // auto close the bibliography and contained biblist.
  Tag!("ltx:biblist",      auto_close => true);
  Tag!("ltx:bibliography", auto_close => true);

  DefMacro!("\\par@in@bibliography", sub[gullet, args, state] {
      gullet.skip_spaces(state);
      if let Some(tok) = gullet.read_token(state) {
        // If next token is another \par, or a REAL \bibitem,
        // then this \par expands into what followed
        // Else, put it back, and start a bibitem.
        if tok == T_CS!("\\par") || tok == T_CS!("\\bibitem") {
          Ok(Tokens!(tok))
        } else {
          gullet.unread_one(tok);
          Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
        }
      } else {
        Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
      }
  });
  DefMacro!("\\item@in@bibliography", "\\save@bibitem{}");

  // If we hit a real \bibitem, put \par & \bibitem back to correct defn, and then \bibitem.
  // A bibitem with now key or label...
  DefMacro!("\\restoring@bibitem", "\\let\\bibitem\\save@bibitem\\let\\par\\save@par\\bibitem");

  NewCounter!("@bibitem", "bibliography", idprefix => "bib");
  DefMacro!("\\the@bibitem", "\\arabic{@bibitem}");
  DefMacro!("\\@biblabel{}", "[#1]");
  DefMacro!("\\fnum@@bibitem", "{\\@biblabel{\\the@bibitem}}");
  // Hack for abused bibliographies; see below
  DefMacro!(
    "\\bibitem",
    "\\if@lx@inbibliography\\else\\expandafter\\lx@mung@bibliography\\expandafter{\\@currenvir}\\fi\\lx@bibitem", locked=>true);
  DefConstructor!("\\lx@bibitem[] Semiverbatim", "<ltx:bibitem key='#key' xml:id='#id'>#tags<ltx:bibblock>",
    after_digest => sub[stomach, whatsit, state] {
      let tag_opt = whatsit.get_arg(1);
      let key = if let Some(key) = whatsit.get_arg(2) {
        clean_bib_key(&key.to_string())
      } else { String::default() };
      if let Some(tag) = tag_opt {
        let mut properties = RefStepID!("@bibitem", stomach)?;
        properties.insert("key".to_string(), key.into());
        let gullet = stomach.get_gullet_mut();
        let mut tag_tokens = vec![
            T_BEGIN!(), T_CS!("\\def"), T_CS!("\\the@bibitem"), T_BEGIN!()];
        tag_tokens.extend(Revert!(tag));
        tag_tokens.push(T_END!());
        tag_tokens.extend(Invocation!(T_CS!("\\lx@make@tags"), vec![T_OTHER!("@bibitem")], gullet)?.unlist());
        tag_tokens.push(T_END!());
        properties.insert("tags".to_string(),
          stomach.digest(tag_tokens, state)?.into());
        whatsit.set_properties(properties);
      } else {
        let mut properties = RefStepCounter!("@bibitem", false, stomach)?;
        properties.insert("key".to_string(), key.into());
        whatsit.set_properties(properties);
      }
    }
  );

  // This attempts to handle the case where folks put \bibitem's within an enumerate or such.
  // We try to close the list and open the bibliography
  DefMacro!("\\lx@mung@bibliography{}", sub[gullet, (env), state] {
    let tag = env.to_string();
    let mut tokens = Vec::new();
    // If we're in some sort of list environment, maybe we can recover
    if tag == "enumerate" || tag == "itemize" || tag == "description" {
      Info!("\nDamn! We're in a list {}; try to close it!", tag);
      tokens.extend(Invocation!("\\end", vec![env], gullet)?.unlist());
      tokens.extend(vec![
        T_CS!("\\let"),
        T_CS!(format!("\\end{tag}")),
        T_CS!("\\endthebibliography"),
        T_CS!("\\let"),
        T_CS!(format!("\\end{{{tag}}}")),
        T_CS!("\\end{thebibliography}")
      ]);
    }
    // else ? it probably isn't going to work??
    Info!("Now, try to open {{thebibliography}}");
    tokens.extend(Invocation!("\\begin", vec![Tokenize!("thebibliography"), Tokens!()], gullet)?.unlist());
    let tokens = Tokens::new(tokens);
    Info!("PATCHING with {:?}", tokens.to_string());
    Ok(tokens)
  });

  DefConstructor!("\\lx@bibnewblock", "<ltx:bibblock>");
  Let!("\\newblock", "\\lx@bibnewblock");
  Tag!("ltx:bibitem",  auto_close => true);
  Tag!("ltx:bibblock", auto_close => true);

  //----------------------------------------------------------------------
  // We've got the same problem as LaTeX: Lather, Rinse, Repeat.
  // It would be nice to know the bib info at digestion time
  //  * whether author lists will collapse
  //  * whether there are "a","b".. extensions on the year.
  // We could process the bibliography first, (IF it is a separate *.bib!)
  // but won't know which entries are included (and so can't resolve the a/b/c..)
  // until we've finished looking at (all of) the source(s) that will refer to them!
  //
  // We can do this in 2 passes, however
  //  (1) convert (latexml) both the source document(s) and the bibliography
  //  (2) extract the required bibitems and integrate (latexmlpost) it into the documents.
  // [Note that for mult-document sites, step (2) becomes 2 stages: scan and integrate]
  //
  // Here's the general layout.
  //   <ltx:cite> contains everything that the citations produce,
  //     including parens, pre-note, punctunation that precede the <ltx:bibcite>
  //     and punctuation, post-note, parens, that follow it.
  //   <ltx:bibcite show="string" bibrefs="keys" sep="" yysep="">phrases</ltx:bibcite>
  //     encodes the actual citation.
  //
  //     bibrefs : lists the bibliographic keys that will be used
  //     show    : gives the pattern for formatting using data from the bibliography
  //       It can contain:
  //         authors or fullauthors
  //         year
  //         number
  //         phrase1,phrase2,... selects one of the phrases from the content of the <ltx:bibref>
  //     This format is used as follows:
  //       If author and year is present, and a subset of the citations share the same authors,
  //         then the format is used, but the year is repeated for each citation in the subset,
  //         as a link to the bib entry.
  //       Otherwise, the format is applied to each entry.
  //
  // The design is intended to support natbib, as well as plain LaTeX.

  AssignValue!("CITE_STYLE", "numbers");
  AssignValue!("CITE_OPEN", T_OTHER!("["));
  AssignValue!("CITE_CLOSE", T_OTHER!("]"));
  AssignValue!("CITE_SEPARATOR", T_OTHER!(","));
  AssignValue!("CITE_YY_SEPARATOR", T_OTHER!(","));
  AssignValue!("CITE_NOTE_SEPARATOR", T_OTHER!(","));

  DefConstructor!("\\@@cite[]{}", "<ltx:cite ?#1(class='ltx_citemacro_#1')>#2</ltx:cite>",
    mode => "text");

  // \@@bibref{what to show}{bibkeys}{phrase1}{phrase2}
  DefConstructor!("\\@@bibref Semiverbatim Semiverbatim {}{}",
    "<ltx:bibref show='#1' bibrefs='#bibrefs' separator='#separator' yyseparator='#yyseparator'>#3#4</ltx:bibref>",
    properties => sub[stomach, args, state] {
      unref!(args => show, keys, phrase1, phrase2);
      Ok(map!("bibrefs" => clean_bib_key(&keys.to_string()).into(),
        "separator" => match state.lookup_tokens("CITE_SEPARATOR") {
          Some(sep) => stomach.digest(sep, state)?.to_string().into(),
          None => String::new().into() },
        "yyseparator" => match state.lookup_tokens("CITE_YY_SEPARATOR") {
          Some(yysep) => stomach.digest(yysep, state)?.to_string().into(),
          None => String::new().into() }
      ))
    }
  );

  // Simple container for any phrases used in the bibref
  DefConstructor!("\\@@citephrase{}", "<ltx:bibrefphrase>#1</ltx:bibrefphrase>", mode => "text");

  DefMacro!("\\cite[] Semiverbatim", sub[gullet, (post_opt, keys), state] {
    let style = state.lookup_tokens("CITE_STYLE").unwrap_or_else(|| Tokens!());
    let open = state.lookup_tokens("CITE_OPEN");
    let open = open.unwrap_or_else(|| Tokens!());
    let close = state.lookup_tokens("CITE_CLOSE").unwrap_or_else(|| Tokens!());
    let mut post_tokens = match post_opt {
      Some(tks) => tks.unlist(),
      None => Vec::new()
    };
    if !post_tokens.is_empty() {
      let ns = state.lookup_tokens("CITE_NOTE_SEPARATOR").unwrap_or_else(|| Tokens!());
      let mut post_wrapped = ns.unlist();
      post_wrapped.push(T_SPACE!());
      post_wrapped.extend(post_tokens);
      post_tokens = post_wrapped;
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"), vec![Tokens!(), keys, Tokens!(), Tokens!()], gullet)?;
    let mut arg_tokens = open.unlist();
    arg_tokens.extend(bibref.unlist());
    arg_tokens.extend(post_tokens);
    arg_tokens.extend(close.unlist());

    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("cite")), Tokens::new(arg_tokens)], gullet)?)
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
  // DefConstructor(T_CS("\\begin{filecontents}"), "Semiverbatim",
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
  // DefConstructor(T_CS("\\begin{filecontents*}"), "Semiverbatim",
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

  // DefConstructor('\index@dotfill', undef, sub {
  //     my ($document) = @_;
  //     closeIndexPhrase($document);
  //     $document->openElement('ltx:indexrefs'); });
  // DefConstructor('\index@item',       undef, sub { doIndexItem($_[0], 1); });
  // DefConstructor('\index@subitem',    undef, sub { doIndexItem($_[0], 2); });
  // DefConstructor('\index@subsubitem', undef, sub { doIndexItem($_[0], 3); });
  // DefConstructor('\index@done',       undef, sub { doIndexItem($_[0], 0); });

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
});

// Since SOME people seem to write bibliographies w/o \bibitem,
// just blank lines between apparent entries,
// Making \par do a \bibitem{} works, but screws up valid
// bibliographies with blank lines!
// So, let's do some redirection!
fn setup_pseudo_bibitem(state: &mut State, gullet: &mut Gullet) {
  state.let_i(&T_CS!("\\save@bibitem"), T_CS!("\\bibitem"), None, gullet);
  state.let_i(&T_CS!("\\save@par"), T_CS!("\\par"), None, gullet);
  state.let_i(&T_CS!("\\bibitem"), T_CS!("\\restoring@bibitem"), None, gullet);
  state.let_i(&T_CS!("\\par"), T_CS!("\\par@in@bibliography"), None, gullet);
  // Moreover some people use \item instead of \bibitem
  state.let_i(&T_CS!("\\item"), T_CS!("\\item@in@bibliography"), None, gullet);
  // And protect from redefinitions.
  state.let_i(&T_CS!("\\newblock"), T_CS!("\\lx@bibnewblock"), None, gullet);
}
// This sub does things that would commonly be needed when starting a bibliography
// setting the ID, etc...
fn begin_bibliography(stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Result<()> {
  begin_bibliography_clean(stomach, whatsit, state)?;
  // Fix for missing \bibitems!
  setup_pseudo_bibitem(state, stomach.get_gullet_mut());
  Ok(())
}

fn begin_bibliography_clean(stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Result<()> {
  BindState!(stomach, state);
  // Try to compute a reasonable, but unique ID;
  // relative to the document's ID, if any.
  // But also, if there are multiple bibliographies,
  let bibnumber = 1 + state.lookup_int("n_bibliographies");
  state.assign_value("n_bibliographies", bibnumber, Some(Scope::Global));
  let mut gullet = stomach.get_gullet_mut();
  let mut docid: String = Expand!(T_CS!("\\thedocument@ID"), gullet, state).to_string();
  if !docid.is_empty() {
    docid += ".";
  }
  let bibid = s!("{}bib{}", docid, radix::radix_alpha(bibnumber - 1));
  DefMacro!(T_CS!("\\thebibliography@ID"), None, T_OTHER!(bibid), scope => Some(Scope::Global));
  whatsit.set_property("id", bibid);
  let title_opt = match DigestIf!("\\refname", stomach)? {
    Some(v) => Some(v),
    None => DigestIf!("\\bibname", stomach)?,
  };
  if let Some(title) = title_opt {
    whatsit.set_property("titlefont", title.get_font().unwrap());
    whatsit.set_property("title", title);
  }
  whatsit.set_property("bibstyle", LookupValue!("BIBSTYLE"));
  whatsit.set_property("citestyle", LookupValue!("CITE_STYLE"));
  // And prepare for the likely nonsense that appears within bibliographies
  ResetCounter!("enumiv");
  Ok(())
}
