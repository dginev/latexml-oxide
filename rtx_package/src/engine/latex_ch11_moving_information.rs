use crate::prelude::*;

//**********************************************************************
// C.11 Moving Information Around
//**********************************************************************
LoadDefinitions!({
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
  DefConstructor!("\\label Semiverbatim", sub[document, _olabel, props] {
    if let Some(savenode) = document.float_to_label() {
      let mut labels : HashMap<String,bool> = HashMap::default();
      if let Some(label) = props.get("label") {
        labels.insert(label.to_string(), true);
      }
      for label in document.node_get_attribute("labels").unwrap_or_default().split_whitespace() {
        labels.insert(label.to_string(), true);
      }
      document.node_set_attribute("labels",
         &labels.keys().map(ToString::to_string).collect::<Vec<_>>().join(" "))?;
      document.set_node(&savenode);
    }
  },
  reversion => "", // TODO: implement for DUAL_BRANCH
  properties => {stored_map!("alignmentSkippable" => true, "alignmentPreserve" => true)},
  after_digest => sub[whatsit] {
    let label = match whatsit.get_arg(1) {
      Some(labeld) => clean_label(&labeld.to_string(), None).into_owned(),
      None => String::new()
    };
    let scope = label.replace("LABEL:","label:");
    let label_key = s!("LABEL@{}", label);
    whatsit.set_property("label", label);

    let ctr_key_opt = with_value("current_counter", |val_opt| val_opt
      .map(|ctr| s!("scopes_for_counter:{}", ctr)));
    if let Some(ctr_key) = ctr_key_opt {
      // TODO: we should probably improve the ergonomics here to avoid the vec![]
      state::unshift_value(&ctr_key, vec![scope.clone()]);
      state::activate_scope(arena::pin(scope));
      stomach::begin_mode("text")?;
      let current_label = stomach::digest(Tokens!(T_CS!("\\@currentlabel")))?;
      state::assign_value(&label_key, current_label, Some(Scope::Global));
      stomach::end_mode("text")?;
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
    properties => sub[args] {
      unpack_opt_ref!(args => _star, label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(arena::pin(clean_label(&label, None)))))
  });

  // "page" does not make sense in xml.  If the user really wants, they will need:
  // \usepackage{latexml} ... \iflatexml alternate\else page \pageref{label}\fi
  Let!("\\pageref", "\\ref");

  // ======================================================================
  //  C.11.3 Bibliography and Citation
  // ======================================================================

  // Note that it's called \refname in LaTeX's article, but \bibname in report & book.
  // And likewise, mixed up in various other classes!

  DefMacro!("\\thebibliography@ID", "");

  // Do this before digesting the body of a bibliography
  fn before_digest_bibliography() -> Result<()> {
    AssignValue!("inPreamble" => false);
    Digest!("\\@lx@inbibliographytrue")?;
    DefMacro!("\\bibliographystyle{}", "");
    DefMacro!("\\bibliography {}", "");
    // avoid \let-based redefinitions of the ending.
    Let!("\\endthebibliography", "\\saved@endthebibliography");
    ResetCounter!("@bibitem");
    Ok(())
  }

  DefMacro!("\\bibliography Semiverbatim",
    r#"\lx@ifusebbl{#1}{\input{\jobname.bbl}}{\lx@bibliography{#1}}"#);

  DefMacro!("\\lx@ifusebbl{}{}{}", sub[(bib_files_tks, bbl_clause, bib_clause)] {
    let bib_files = Expand!(bib_files_tks).to_string();
    if bib_files.is_empty() {
      return Ok(Tokens!());
    }
    let jobname = Expand!(T_CS!("\\jobname")).to_string();
    let bbl_path     = FindFile!(&jobname, type => "bbl");
    let mut missing_bibs = String::new();
    if lookup_bool("NO_BIBTEX") {
      if bbl_path.is_none() {
        Info!("expected", "bbl", "Couldn't find bbl file, bibliography may be empty.");
        Ok(Tokens!())
      } else {
        Ok(bbl_clause)
      }
    } else {
      for bf in bib_files.split(',') {
        let bib_path = FindFile!(bf, type => "bib");
        if bib_path.is_none() {
          if !missing_bibs.is_empty() {
            missing_bibs.push(',');
          }
          missing_bibs.push_str(bf);
        }
      }
      if missing_bibs.is_empty() || bbl_path.is_none() {
        Ok(bib_clause)
      } else {
        Info!("expected", missing_bibs, s!("Couldn't find all bib files, using {jobname}.bbl instead"));
        Ok(bbl_clause)
      }
    }
  });

  AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  AssignMapping!("BACKMATTER_ELEMENT", "ltx:index"        => "ltx:section");

  // DefConstructor('\lx@bibliography [] Semiverbatim',
  //   "<ltx:bibliography files='#2' xml:id='#id' "
  //     . "bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort' lists='#1'>"
  //     . "<ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>"
  //     . "</ltx:bibliography>",
  //   afterDigest => sub { $_[0]->begingroup;    # wrapped so redefns don't take effect!
  //     beginBibliography($_[1]);
  //     $_[0]->endgroup; },
  //   beforeConstruct => sub { adjustBackmatterElement($_[0], $_[1]); });

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

  // sub setBibstyle {
  //   my ($style) = @_;
  //   $style = ToString($style);
  //   AssignValue(BIBSTYLE => $style);
  //   if (my $parms = $$BIBSTYLES{$style}) {
  //     AssignValue(CITE_STYLE => $$parms{citestyle});
  //     AssignValue(CITE_SORT  => $$parms{sort}); }
  //   return; }

  // DefConstructor('\bibstyle{}', sub {
  //     my ($document, $style) = @_;
  //     setBibstyle($style);
  //     # Really ?
  //     if (my $bib = $document->findnode('//ltx:bibliography')) {
  //       $document->setAttribute($bib, bibstyle  => LookupValue('BIBSTYLE'));
  //       $document->setAttribute($bib, citestyle => LookupValue('CITE_STYLE'));
  //       $document->setAttribute($bib, sort      => LookupValue('CITE_SORT')); }
  //   },
  //   afterDigest => sub {
  //     my $style = ToString($_[1]->getArg(1));
  //     AssignValue(BIBSTYLE => $style, 'global');
  //     if (my $parms = $$BIBSTYLES{$style}) {
  //       AssignValue(CITE_STYLE => $$parms{citestyle}); }
  //     else {
  //       Info('unexpected', $style, $_[0], "Unknown bibstyle '$style', it will be ignored"); }
  //     return; });

  DefMacro!("\\bibliographystyle Semiverbatim", "\\bibstyle{#1}");

  DefConditional!("\\if@lx@inbibliography");
  // Should be an environment, but people seem to want to misuse it.
  DefConstructor!("\\thebibliography",
  "<ltx:bibliography xml:id='#id'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title><ltx:biblist>",
    before_digest => {
        before_digest_bibliography() },
    after_digest => sub[whatsit] {
      // NOTE that in some perverse situations (revtex?)
      // it seems to be allowable to omit the argument
      // It's ignorable for latexml anyway, so we'll just read it if its there.
      gullet::skip_spaces()?;
      if gullet::if_next(T_BEGIN!())? {
        gullet::read_arg(ExpansionLevel::Off)?;
      }
      begin_bibliography(whatsit)?;
    },
    before_construct => sub[doc,whatsit] { 
      adjust_backmatter_element(doc, whatsit)?;
    },
    locked => true
  );

  // Close the bibliography
  DefConstructor!("\\endthebibliography", sub[document,_whatsit,_props] {
    document.maybe_close_element("ltx:biblist")?;
    document.maybe_close_element("ltx:bibliography")?;
  }, locked=>true);
  Let!("\\saved@endthebibliography", "\\endthebibliography");
  // auto close the bibliography and contained biblist.
  Tag!("ltx:biblist",      auto_close => true);
  Tag!("ltx:bibliography", auto_close => true);

  DefMacro!("\\par@in@bibliography", {
    gullet::skip_spaces()?;
    if let Some(tok) = gullet::read_token()? {
      // If next token is another \par, or a REAL \bibitem,
      // then this \par expands into what followed
      // Else, put it back, and start a bibitem.
      if tok == T_CS!("\\par") || tok == T_CS!("\\bibitem") {
        Ok(Tokens!(tok))
      } else {
        gullet::unread_one(tok);
        Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
      }
    } else {
      Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
    }
  });
  DefMacro!("\\vskip@in@bibliography Glue", None);
  DefMacro!("\\item@in@bibliography", "\\save@bibitem{}");

  // If we hit a real \bibitem, put \par & \bibitem back to correct defn, and then \bibitem.
  // A bibitem with now key or label...
  //
  // Porting note: careful with the escaping rules. In perl we had a '\let\\\\\save@...'
  // but if we use the r## 'raw string literal' in Rust, the extra \\ escape is not needed.
  DefMacro!(
    "\\restoring@bibitem",
    r#"\let\bibitem\save@bibitem\let\par\save@par\let\\\save@backbackslash\bibitem"#
  );

  NewCounter!("@bibitem", "bibliography", idprefix => "bib");
  DefMacro!("\\the@bibitem", "\\arabic{@bibitem}");
  DefMacro!("\\@biblabel{}", "[#1]");
  DefMacro!("\\fnum@@bibitem", "{\\@biblabel{\\the@bibitem}}");
  // Hack for abused bibliographies; see below
  DefMacro!(
    "\\bibitem",
    r#"\if@lx@inbibliography\else\expandafter\lx@mung@bibliography\expandafter{\@currenvir}\fi\lx@bibitem"#,
    locked=>true);
  DefConstructor!("\\lx@bibitem[] Semiverbatim",
    "<ltx:bibitem key='#key' xml:id='#id'>#tags<ltx:bibblock>",
    after_digest => sub[whatsit] {
      let tag_opt = whatsit.get_arg(1);
      let key = if let Some(key) = whatsit.get_arg(2) {
        clean_bib_key(&key.to_string())
      } else { String::default() };
      if let Some(tag) = tag_opt {
        let mut properties = RefStepID!("@bibitem")?;
        properties.insert("key", key.into());
        let mut tag_tokens = vec![
            T_BEGIN!(), T_CS!("\\def"), T_CS!("\\the@bibitem"), T_BEGIN!()];
        tag_tokens.extend(Revert!(tag));
        tag_tokens.push(T_END!());
        tag_tokens.extend(
          Invocation!(T_CS!("\\lx@make@tags"), vec![T_OTHER!("@bibitem")]).unlist());
        tag_tokens.push(T_END!());
        properties.insert("tags",
          stomach::digest(tag_tokens)?.into());
        whatsit.set_properties(properties);
      } else {
        let mut properties = RefStepCounter!("@bibitem")?;
        properties.insert("key", key.into());
        whatsit.set_properties(properties);
      }
    }
  );

  // This attempts to handle the case where folks put \bibitem's within an enumerate or such.
  // We try to close the list and open the bibliography
  DefMacro!("\\lx@mung@bibliography{}", sub[(env)] {
    let tag = env.to_string();
    let mut tokens = Vec::new();
    // If we're in some sort of list environment, maybe we can recover
    if tag == "enumerate" || tag == "itemize" || tag == "description" {
      tokens.extend(Invocation!("\\end", vec![env]).unlist());
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
    //Now, try to open {thebibliography}
    tokens.push(T_CS!("\\lx@mung@bibliography@pre"));
    tokens.push(T_CS!("\\thebibliography"));
    Ok(Tokens::new(tokens))
  });
  DefConstructor!("\\lx@mung@bibliography@pre", sub[document] {
    let parent     = document.get_node();
    let tag_sym    = model::get_node_qname(parent);
    arena::with(tag_sym, |tag| if tag == "enumerate" || tag == "itemize" || tag == "description" {
      document.maybe_close_element(tag) } else { Ok(None) })?; // Or even remove (if empty)?
  });

  DefConstructor!("\\lx@bibnewblock", sub[document] {
    if document.is_openable("ltx:bibblock") {
      document.open_element("ltx:bibblock",None,None)?;
    }});
  Let!("\\newblock", "\\lx@bibnewblock");
  Tag!("ltx:bibitem",  auto_open => true, auto_close => true);
  Tag!("ltx:bibblock", auto_open => true, auto_close => true);

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
    "<ltx:bibref show='#1' bibrefs='#bibrefs' separator='#separator'
      yyseparator='#yyseparator'>#3#4</ltx:bibref>",
    properties => sub[args] {
      unref!(args => _show, keys, _phrase1, _phrase2);
      Ok(stored_map!("bibrefs" => clean_bib_key(&keys.to_string()),
        "separator" => match state::lookup_tokens("CITE_SEPARATOR") {
          Some(sep) => stomach::digest(sep)?.to_string(),
          None => String::new() },
        "yyseparator" => match state::lookup_tokens("CITE_YY_SEPARATOR") {
          Some(yysep) => stomach::digest(yysep)?.to_string(),
          None => String::new() }
      ))
    }
  );

  // Simple container for any phrases used in the bibref
  DefConstructor!("\\@@citephrase{}", "<ltx:bibrefphrase>#1</ltx:bibrefphrase>", mode => "text");

  DefMacro!("\\cite[] Semiverbatim", sub[(post_opt, keys)] {
    // let style = state::lookup_tokens("CITE_STYLE").unwrap_or_else(|| Tokens!());
    let open = state::lookup_tokens("CITE_OPEN");
    let open = open.unwrap_or_else(|| Tokens!());
    let close = state::lookup_tokens("CITE_CLOSE").unwrap_or_else(|| Tokens!());
    let mut post_tokens = match post_opt {
      Some(tks) => tks.unlist(),
      None => Vec::new()
    };
    if !post_tokens.is_empty() {
      let ns = state::lookup_tokens("CITE_NOTE_SEPARATOR").unwrap_or_else(|| Tokens!());
      let mut post_wrapped = ns.unlist();
      post_wrapped.push(T_SPACE!());
      post_wrapped.extend(post_tokens);
      post_tokens = post_wrapped;
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens!(), keys, Tokens!(), Tokens!()]);
    let mut arg_tokens = open.unlist();
    arg_tokens.extend(bibref.unlist());
    arg_tokens.extend(post_tokens);
    arg_tokens.extend(close.unlist());

    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("cite")), Tokens::new(arg_tokens)]))
  });

  // # NOTE: Eventually needs to be recognized by MakeBibliography
  // DefConstructor('\nocite Semiverbatim',
  //   "<ltx:cite><ltx:bibref show='nothing' bibrefs='#bibrefs'/></ltx:cite>",
  //   properties => sub { (bibrefs => CleanBibKey($_[1])) });

});

fn note_backmatter_element(whatsit: &mut Whatsit, backelement: &str) {
  if let Some(val) = state::lookup_mapping("BACKMATTER_ELEMENT", backelement) {
    whatsit.set_property("backmatterelement", val);
  }
}

fn adjust_backmatter_element(document: &mut Document, whatsit: &Whatsit) -> Result<()> {
  let asif_opt = if let Some(Stored::String(asif_sym)) = whatsit.get_property("backmatterelement").as_deref() {
    Some(arena::to_string(*asif_sym))
  } else {
    None
  };
  // Note: We allocate a string here, since
  // it looks like arena::with can deadlock with find_insertion_point
  // we may need a find_insertion_point_sym to avoid that...
  if let Some(asif) = asif_opt {
    let point = document.find_insertion_point(&asif, None)?;
    document.set_node(&point);
  }
  Ok(())
}

// Since SOME people seem to write bibliographies w/o \bibitem,
// just blank lines between apparent entries,
// Making \par do a \bibitem{} works, but screws up valid
// bibliographies with blank lines!
// So, let's do some redirection!
fn setup_pseudo_bibitem() -> Result<()> {
  Let!("\\save@bibitem","\\bibitem");
  Let!("\\save@par","\\par");
  Let!("\\save@backbackslash", "\\\\");
  Let!("\\bibitem","\\restoring@bibitem");
  Let!("\\par","\\par@in@bibliography");
  Let!("\\\\", "\\par@in@bibliography");
  Let!("\\vskip", "\\vskip@in@bibliography");
  // Moreover some people use \item instead of \bibitem
  Let!("\\item","\\item@in@bibliography");
  // And protect from redefinitions.
  Let!("\\newblock","\\lx@bibnewblock");
  // Risky, but when bibliography immediatesly starts with text (no implied \par)
  if let Some(token) = gullet::read_non_space()? {
    gullet::unread_one(token);
    if !token.is_executable() {
      gullet::unread_one(T_CS!("\\par"));
    }
  }
  Ok(())
}
// This sub does things that would commonly be needed when starting a bibliography
// setting the ID, etc...
fn begin_bibliography(
  whatsit: &mut Whatsit,
) -> Result<()> {
  begin_bibliography_clean( whatsit)?;
  // Fix for missing \bibitems!
  setup_pseudo_bibitem()
}

fn begin_bibliography_clean(
  whatsit: &mut Whatsit,
) -> Result<()> {
  // Check if \bibsection is defined and try to decipher it.
  // Expecting something like \section*{sometext}

  // TODO: Continue updating here...
  // let bs_opt       = lookup_definition(&T_CS!("\\bibsection"))?;

  note_backmatter_element(whatsit, "ltx:bibliography");
  // Try to compute a reasonable, but unique ID;
  // relative to the document's ID, if any.
  // But also, if there are multiple bibliographies,
  let bibnumber = 1 + lookup_int("n_bibliographies");
  assign_value("n_bibliographies", bibnumber, Some(Scope::Global));
  let mut docid: String = Expand!(T_CS!("\\thedocument@ID")).to_string();
  if !docid.is_empty() {
    docid += ".";
  }
  let bibid = s!("{}bib{}", docid, radix::radix_alpha(bibnumber - 1));
  DefMacro!(T_CS!("\\thebibliography@ID"), None, T_OTHER!(&bibid), scope => Some(Scope::Global));
  whatsit.set_property("id", bibid);
  let title_opt = match DigestIf!("\\refname")? {
    Some(v) => Some(v),
    None => DigestIf!("\\bibname")?,
  };
  if let Some(title) = title_opt {
    whatsit.set_property("titlefont", title.get_font()?.unwrap());
    whatsit.set_property("title", title);
  }
  {
    whatsit.set_property("bibstyle", lookup_value("BIBSTYLE"));
    whatsit.set_property("citestyle", lookup_value("CITE_STYLE"));
  }
  // And prepare for the likely nonsense that appears within bibliographies
  ResetCounter!("enumiv");
  Ok(())
}
