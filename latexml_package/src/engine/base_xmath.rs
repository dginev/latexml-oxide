//! Base XMath
//!
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
use std::collections::hash_map::Entry;

static NAMED_SPACE_CHARS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
  static_map!("negthinspace" => "", "thinspace" => "\u{2009}",
    "medspace" => "\u{2005}", "thickspace" => "\u{2004}", "space" => " ")
});
static DECIMAL_SEP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(
  || static_map!("en" => ".", "de" => ",", "fr" => ",", "nl" => ",", "pt" => ",", "es" => ","),
);
static THOUSANDS_SEP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(
  || static_map!("en" => ",", "de" => ".", "fr" => ".", "nl" => ".", "pt" => ".", "es" => "."),
);

/// Perl: XMath_copy_keyvals — copy key-value pairs from OptionalKeyVals:XMath arg to whatsit props
fn xmath_copy_keyvals(whatsit: &mut Whatsit) -> Result<Vec<Digested>> {
  // Get pairs first, then set properties (avoids borrow conflict)
  // Use get_hash_digested() since after digestion values are in cached_hash_digested
  let pairs: Vec<(String, String)> = if let Some(arg1) = whatsit.get_arg(1) {
    match arg1.data() {
      DigestedData::KeyVals(ref kv) => kv.get_hash_digested().into_iter().collect(),
      _ => Vec::new(),
    }
  } else {
    Vec::new()
  };
  for (key, val) in pairs {
    whatsit.set_property(&key, Stored::from(val));
  }
  Ok(Vec::new())
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // LaTeXML Enhancemens to Math Representation to preserve Semantics
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Some of this stuff is more semantic versions of declarations in
  // plain or latex. Is this the right place for them?

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Normally, the content branch contains the pure structure and meaning of a construct,
  // and the presentation is generated from lower level TeX macros that only concern
  // themselves with how to display the object.
  // Nevertheless, it is sometimes useful to know where the tokens in the presentation branch
  // came from;  particularly what their presumed "meaning" is.
  // For example, when search-indexing pmml, or providing links to definitions from the pmml.
  //
  // The following constructor (see how it's used in DefMath), adds meaning attributes
  // whereever it seems sensible on the presentation branch, after it has been generated.
  // This appears to be obsolete/no-longer-used, but keep for future reference.
  DefConstructor!("\\lx@assert@meaning{}{}", "#2",
  reversion      => "#2",
  after_construct => sub[document,whatsit] {
    let node    = document.get_node().clone(); // This should be the wrapper just added.
    let meaning = whatsit.get_arg(1).unwrap().to_string();
    add_meaning_rec(document, node, &meaning)?;
  });
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Support for constructing mathematical expressions
  // # Common XMath pattern for assigning attributes from Whatsit properties.
  // our $XMath_attributes =
  //   " role='#role' name='#name' meaning='#meaning' omcd='#omcd'"
  //   . " width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset'"
  //   . " lpadding='#lpadding' rpadding='#rpadding'";

  // sub XMath_copy_keyvals {
  //   my ($stomach, $whatsit) = @_;
  //   my $kv = $whatsit->getArg(1);
  //   $whatsit->setProperties($kv->getPairs) if $kv;
  //   return; }

  // Build an ltx:XMApp, application of function/operator to arguments
  DefConstructor!("\\lx@apply OptionalKeyVals:XMath {}{}",
    "<ltx:XMApp role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'>#2#3</ltx:XMApp>",
    reversion => "#2#3",
    after_digest => sub[whatsit] { xmath_copy_keyvals(whatsit) });

  // Build an ltx:XMTok, a mathematical symbol, with given attributes
  DefConstructor!("\\lx@symbol OptionalKeyVals:XMath {}",
    "<ltx:XMTok role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'>#2</ltx:XMTok>",
    reversion => "#2",
    after_digest => sub[whatsit] {
      // Copy font from arg 2 to whatsit
      if let Some(arg2) = whatsit.get_arg(2) {
        if let Ok(Some(font)) = arg2.get_font() {
          whatsit.set_font(Rc::new(font.into_owned()));
        }
      }
      xmath_copy_keyvals(whatsit) });

  // Wrap the contents in an ltx:XMWrap, to stand as a single subtree & providing attributes
  DefConstructor!("\\lx@wrap OptionalKeyVals:XMath {}",
    "<ltx:XMWrap role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'>#2</ltx:XMWrap>",
    reversion => "#2",
    after_digest => sub[whatsit] { xmath_copy_keyvals(whatsit) });

  // # Convert a hashref into a list of tokens of the form key=value,...
  // sub I_keyvals {
  //   my ($keyvals) = @_;
  //   my @options = ();
  //   if ($keyvals) {
  //     while (my ($key, $value) = each %$keyvals) {
  //       $value = TokenizeInternal($value) if defined $value && !ref $value;
  //       push(@options, T_OTHER(',')) if @options;
  //       push(@options, T_OTHER($key), T_OTHER('='), T_BEGIN, $value, T_END); } }
  //   return (@options ? Tokens(T_OTHER('['), @options, T_OTHER(']')) : ()); }

  // sub I_apply {
  //   my ($kv, $op, @args) = @_;
  //   return Tokens(T_CS('\lx@apply'), I_keyvals($kv),
  //     T_BEGIN, T_CS('\lx@wrap'), T_BEGIN, $op, T_END, T_END,
  //     T_BEGIN, (map { (T_CS('\lx@wrap'), T_BEGIN, $_, T_END); } @args), T_END); }

  // sub I_symbol {
  //   my ($kv, $text) = @_;
  //   return Tokens(T_CS('\lx@symbol'), I_keyvals($kv), T_BEGIN, (defined $text ? $text : ()),
  // T_END); }

  // sub I_wrap {
  //   my ($kv, @stuff) = @_;
  //   return Tokens(T_CS('\lx@wrap'), I_keyvals($kv), T_BEGIN, @stuff, T_END); }

  // Superscript with optional keyvals (operator_meaning, etc.)
  DefConstructor!("\\lx@superscript OptionalKeyVals:XMath {} InScriptStyle",
  "<ltx:XMApp role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'><ltx:XMTok role='SUPERSCRIPTOP' meaning='#operator_meaning' omcd='#operator_omcd' scriptpos='#scriptpos'/><ltx:XMArg>#2</ltx:XMArg><ltx:XMArg rule='Superscript'>#3</ltx:XMArg></ltx:XMApp>",
  after_digest => sub[whatsit] {
    xmath_copy_keyvals(whatsit)?;
    // Compute scriptpos = "post" + script_level
    let scriptpos = s!("post{}", stomach::get_script_level());
    whatsit.set_property("scriptpos", Stored::from(scriptpos));
    Ok(Vec::new()) },
  reversion => sub[_whatsit, args] {
    // Always wrap base in braces (bump=1)
    let base = &args[1];
    let sup = &args[2];
    let base_rev = match base { Some(inner) => inner.revert()?, None => Tokens!() };
    let sup_rev = match sup { Some(inner) => inner.revert()?, None => Tokens!() };
    if sup_rev.is_empty() {
      Ok(base_rev)
    } else {
      let mut tks = vec![T_BEGIN!()];
      tks.extend(base_rev.unlist());
      tks.push(T_END!());
      tks.push(T_SUPER!());
      tks.push(T_BEGIN!());
      tks.extend(sup_rev.unlist());
      tks.push(T_END!());
      Ok(Tokens::new(tks))
    }});

  // Subscript with optional keyvals (operator_meaning, etc.)
  DefConstructor!("\\lx@subscript OptionalKeyVals:XMath {} InScriptStyle",
  "<ltx:XMApp role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'><ltx:XMTok role='SUBSCRIPTOP' meaning='#operator_meaning' omcd='#operator_omcd' scriptpos='#scriptpos'/><ltx:XMArg>#2</ltx:XMArg><ltx:XMArg rule='Subscript'>#3</ltx:XMArg></ltx:XMApp>",
  after_digest => sub[whatsit] {
    xmath_copy_keyvals(whatsit)?;
    // Compute scriptpos = "post" + script_level
    let scriptpos = s!("post{}", stomach::get_script_level());
    whatsit.set_property("scriptpos", Stored::from(scriptpos));
    Ok(Vec::new()) },
  reversion => sub[_whatsit, args] {
    // Always wrap base in braces (bump=1)
    let base = &args[1];
    let sub_arg = &args[2];
    let base_rev = match base { Some(inner) => inner.revert()?, None => Tokens!() };
    let sub_rev = match sub_arg { Some(inner) => inner.revert()?, None => Tokens!() };
    if sub_rev.is_empty() {
      Ok(base_rev)
    } else {
      let mut tks = vec![T_BEGIN!()];
      tks.extend(base_rev.unlist());
      tks.push(T_END!());
      tks.push(T_SUB!());
      tks.push(T_BEGIN!());
      tks.extend(sub_rev.unlist());
      tks.push(T_END!());
      Ok(Tokens::new(tks))
    }});

  // # Ignore $kv for the moment?????
  // sub I_subscript {
  //   my ($kv, $base, $script) = @_;
  //   return Tokens(T_CS('\lx@subscript'), I_keyvals($kv), T_BEGIN, $base, T_END, T_BEGIN, $script,
  // T_END); }

  // sub I_superscript {
  //   my ($kv, $base, $script) = @_;
  //   return Tokens(T_CS('\lx@superscript'), I_keyvals($kv), T_BEGIN, $base, T_END, T_BEGIN,
  // $script, T_END); }

  // Superscript meaning power
  DefMacro!(
    "\\lx@power{}{}",
    "\\lx@superscript[operator_meaning=power]{#1}{#2}"
  );
  // Superscript meaning functional (or applicative) power; iterated function/operator application
  DefMacro!(
    "\\lx@functionalpower{}{}",
    "\\lx@superscript[operator_meaning=functional-power]{#1}{#2}"
  );

  // These to be used in presentation side
  DefMath!("\\lx@ApplyFunction", None, "\u{2061}", reversion => "", name => "", role =>"APPLYOP");
  // Perl Base_Deprecated: deprecated aliases for invisible math operators
  Let!("\\@APPLYFUNCTION", "\\lx@ApplyFunction");
  Let!("\\@INVISIBLETIMES", "\\lx@InvisibleTimes");
  Let!("\\@INVISIBLECOMMA", "\\lx@InvisibleComma");
  Let!("\\@INVISIBLEPLUS", "\\lx@InvisiblePlus");
  DefMath!("\\lx@InvisibleTimes", None, "\u{2062}", reversion => "", name => "",
    meaning => "times", role => "MULOP");
  DefMath!("\\lx@InvisibleComma", None, "\u{2063}", reversion => "", name => "", role => "PUNCT");
  DefMath!("\\lx@InvisiblePlus", None, "\u{2064}", reversion => "", name => "", meaning => "plus", role => "ADDOP");
  // Perl: beforeDigest => sub { $_[0]->enterHorizontal; }
  DefConstructor!("\\lx@kludged{}",
    "?#isMath(<ltx:XMWrap rule='kludge'>#1</ltx:XMWrap>)(#1)",
    enter_horizontal => true,
    reversion => "#1");
  // Perl L197-209: \lx@padded sets lpadding/rpadding on the absorbed content.
  // afterConstruct sets attributes on the last child (skipping XMDual wrapper).
  // The lpadding/rpadding values are MuDimensions, but the XML attribute is
  // pt-typed; do mu→pt conversion here (1mu = font_size/18) instead of
  // emitting raw "3.0mu" strings.
  fn mudimension_to_pt_attr(d: &Digested) -> String {
    if let DigestedData::RegisterValue(rv) = d.data() {
      let mu_val = match rv {
        RegisterValue::MuDimension(md) => Some(md.value_of()),
        RegisterValue::MuGlue(mg) => Some(mg.value_of()),
        _ => None,
      };
      if let Some(mu) = mu_val {
        let fs = state::lookup_font().and_then(|f| f.get_size()).unwrap_or(10.0);
        let unity = latexml_core::common::numeric_ops::UNITY_F64;
        let muwidth = (fs * unity / 18.0) as i64;
        let pt_scaled = (mu as f64 * muwidth as f64 / unity).trunc() as i64;
        return Dimension::new(pt_scaled).to_string();
      }
    }
    d.to_string()
  }
  DefConstructor!("\\lx@padded[MuDimension]{MuDimension}{}",
    "#3",
    after_construct => sub[document, whatsit] {
      let node = document.get_node();
      if let Some(mut last) = node.get_last_child() {
        // If last child is XMDual, use its second child (presentation)
        if document::with_node_qname(&last, |qn| qn == "ltx:XMDual") {
          if let Some(ch2) = last.get_child_nodes().into_iter().nth(1) {
            last = ch2;
          }
        }
        if let Some(lpad) = whatsit.get_arg(0) {
          let val = mudimension_to_pt_attr(lpad);
          if !val.is_empty() {
            document.set_attribute(&mut last, "lpadding", &val)?;
          }
        }
        if let Some(rpad) = whatsit.get_arg(1) {
          let val = mudimension_to_pt_attr(rpad);
          if !val.is_empty() {
            document.set_attribute(&mut last, "rpadding", &val)?;
          }
        }
      }
    },
    reversion => "#3");

  // #======================================================================
  // # Building XMDuals for Mathematical Parallel markup
  // # Used when the content and presentation forms have different structure.

  DefKeyVal!("XMath", "reversion", "UndigestedDefKey");
  DefKeyVal!("XMath", "content_reversion", "UndigestedDefKey");
  DefKeyVal!("XMath", "presentation_reversion", "UndigestedDefKey");
  // Common XMath attribute keys used in templates
  DefKeyVal!("XMath", "role", "");
  DefKeyVal!("XMath", "name", "");
  DefKeyVal!("XMath", "meaning", "");
  DefKeyVal!("XMath", "omcd", "");
  DefKeyVal!("XMath", "width", "");
  DefKeyVal!("XMath", "height", "");
  DefKeyVal!("XMath", "xoffset", "");
  DefKeyVal!("XMath", "yoffset", "");
  DefKeyVal!("XMath", "lpadding", "");
  DefKeyVal!("XMath", "rpadding", "");
  DefKeyVal!("XMath", "operator_meaning", "");
  DefKeyVal!("XMath", "operator_omcd", "");
  DefKeyVal!("XMath", "scriptpos", "");
  DefKeyVal!("XMath", "revert_as", "");

  DefConstructor!("\\lx@dual OptionalKeyVals:XMath {}{}",
  "<ltx:XMDual role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'>#2<ltx:XMWrap>#3</ltx:XMWrap></ltx:XMDual>",
  before_digest => {
    push_value("PENDING_DUAL_XMARGS", Stored::HashStored(SymHashMap::default()))
  },
  after_digest => sub[whatsit] {
    // let kv     = whatsit.get_arg(1);
    if let Some(Stored::HashStored(xmargs)) = pop_value("PENDING_DUAL_XMARGS")? { // Really SHOULD be a hash
      whatsit.set_properties(xmargs);  // Hopefully no name class with XM<digits>
    }
    // Perl: whatsit.set_properties($kv->getPairs) if $kv;
    // Extract key-value pairs from the OptionalKeyVals argument and set as properties.
    // This makes #role, #name, #meaning etc. available in the constructor template.
    if let Some(kv_arg) = whatsit.get_arg(1) {
      if let DigestedData::KeyVals(kv) = kv_arg.data() {
        for (k, v) in kv.get_hash() {
          whatsit.set_property(&k, Stored::from(v));
        }
      }
    }
    // Pop reversion from state if set by i_dual (preserves ARG catcodes)
    if let Some(Stored::Tokens(rev_tks)) = pop_value("PENDING_DUAL_REVERSION")? {
      whatsit.set_property("reversion", Stored::Tokens(rev_tks));
    }

    let props = whatsit.get_properties();
    let cr    = props.get("content_reversion").cloned();
    let pr    = props.get("presentation_reversion").cloned();
    let r     = match props.get("revert_as") {
      Some(v) => v.to_string(),
      None => String::from("content")
    };    // ?????
    if whatsit.get_property("reversion").is_none() {
      let reversion_closure = Reversion::Closure(Rc::new(move |wself, args| {
        // TODO: The data manamgement here is far from final.
        // Can we avoid clones? Can we consolidate the reversion variants?
        // let kvs = &args[0];
        let c = &args[1];
        let p = &args[2];
        let reverted = match r.as_str() {
          "content" => match &cr {
          Some(Stored::Reversion(Reversion::Tokens(cr_tks))) => cr_tks.clone(),
          Some(Stored::Reversion(Reversion::Closure(cr_closure))) => cr_closure(wself,args)?,
          Some(Stored::Tokens(cr_tks)) => cr_tks.clone(),
          _ => match c {
            Some(inner) => inner.revert()?,
            None => Tokens!()
          }},
          "presentation" => match &pr {
          Some(Stored::Reversion(Reversion::Tokens(pr_tks))) => pr_tks.clone(),
          Some(Stored::Reversion(Reversion::Closure(pr_closure))) => pr_closure(wself,args)?,
          Some(Stored::Tokens(pr_tks)) => pr_tks.clone(),
          _ => match p {
            Some(inner) => inner.revert()?,
            None => Tokens!()
          }},
          "dual" => {
            // Perl: Tokens(T_CS('\lx@dual'), I_keyvals($kvs),
            //   T_BEGIN, ($cr || Revert($c)), T_END,
            //   T_BEGIN, ($pr || Revert($p)), T_END)
            let c_rev = match &cr {
              Some(Stored::Reversion(Reversion::Tokens(tks))) => tks.clone(),
              Some(Stored::Reversion(Reversion::Closure(cl))) => cl(wself,args)?,
              Some(Stored::Tokens(tks)) => tks.clone(),
              _ => match c { Some(inner) => inner.revert()?, None => Tokens!() },
            };
            let p_rev = match &pr {
              Some(Stored::Reversion(Reversion::Tokens(tks))) => tks.clone(),
              Some(Stored::Reversion(Reversion::Closure(cl))) => cl(wself,args)?,
              Some(Stored::Tokens(tks)) => tks.clone(),
              _ => match p { Some(inner) => inner.revert()?, None => Tokens!() },
            };
            let mut tks = vec![T_CS!("\\lx@dual"), T_BEGIN!()];
            tks.extend(c_rev.unlist());
            tks.push(T_END!());
            tks.push(T_BEGIN!());
            tks.extend(p_rev.unlist());
            tks.push(T_END!());
            Tokens::new(tks)
          },
          _other => {
            // Context-dependent reversion: use presentation if DUAL_BRANCH
            // is "presentation", otherwise use content. with_value avoids
            // the Stored::clone + full to_string we previously paid just
            // to compare against a single literal.
            let is_presentation = state::with_value("DUAL_BRANCH", |v| {
              v.map(|s| s.eq_text("presentation")).unwrap_or(false)
            });
            if is_presentation {
              match &pr {
                Some(Stored::Reversion(Reversion::Tokens(tks))) => tks.clone(),
                Some(Stored::Reversion(Reversion::Closure(cl))) => cl(wself,args)?,
                Some(Stored::Tokens(tks)) => tks.clone(),
                _ => match p { Some(inner) => inner.revert()?, None => Tokens!() },
              }
            } else {
              match &cr {
                Some(Stored::Reversion(Reversion::Tokens(tks))) => tks.clone(),
                Some(Stored::Reversion(Reversion::Closure(cl))) => cl(wself,args)?,
                Some(Stored::Tokens(tks)) => tks.clone(),
                _ => match c { Some(inner) => inner.revert()?, None => Tokens!() },
              }
            }
          }
        };
        Ok(reverted)
      }));
      whatsit.set_property("reversion", reversion_closure);
    }
    Ok(Vec::new())
  },
  sizer => "#3"); // size according to presentation

  // These are used within XMDual
  // The XMDual represents both a content & presentation representation of some
  // possibly exotic structure ("Transfix notation"),
  // or just a somewhat complex presentation that corresponds (often) to a simpler
  // applicative content structure.
  // Invoking such a mathematical object to "arguments" requires that both the
  // content & presentation branches contain those arguments.
  // There will be an XMArg, with an ID, containing the actual markup, and an XMRef that referrs to
  // it. The XMArg will usually be in the presentation branch (so that it inherits appropriate
  // style), unless the arg is "hidden" (ie. semantic, but not displayed).
  // This means that we don't know which one appears first! (See Package's dualize_arglist)
  //
  // To get a "proper id", we'll use a temporary label-like attribute (_xmkey)
  // and establish an id and idref later.
  DefConstructor!("\\lx@xmarg{}{}", "<ltx:XMArg _xmkey='#1'>#2</ltx:XMArg>",
    reversion   => "#2",
    after_digest => sub[whatsit] {
      let xmid = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      let arg = whatsit.get_arg(2);
      let reversion_key = s!("xref:{}@reversion", xmid);
      with_value_mut("PENDING_DUAL_XMARGS", |pending_opt|
        if let Some(Stored::HashStored(ref mut pending)) = pending_opt  {
          pending.insert(&xmid, arg.into());
        });
      // TODO: Must we store the (currently &mut) Whatsit?
      // let whatsit_stored = Stored::Digested(whatsit.into());
      state::assign_value(&reversion_key, Stored::Tokens(whatsit.revert()?),
        Some(Scope::Global));
      // state::assign_value(&s!("xref:{}@size", xmid),
      //   whatsit.get_size(None), Some(Scope::Global));
  });

  DefConstructor!("\\lx@xmref{}", "<ltx:XMRef _xmkey='#1'/>",
    // TODO: Must we store and lookup the Whatsit?
    reversion => sub[_whatsit,args] {
      let xmid = args[0].as_ref().unwrap().to_string();
      Ok( state::lookup_tokens(&s!("xref:{xmid}@reversion")).unwrap_or_default() )}
    // sizer => sub { LookupValue('xref:' . ToString($_[0]->getArg(1)))->getSize; }
  );

  // Connect up the XMRef/XMArg pairs (actually can be multiple XMRef's)
  // We want to set the idref of the XMRef's to point to the id of the XMArg (or other XM element),
  // but usually the XMRef is created first, and we want to let the referred to element
  // get it's id computed by whatever means it prefers.
  // so we have to work both ways (use state::to record associations, to avoid expensive xpath)
  // Set id's on any non-XMRef nodes that have an _xmkey
  // This gets a more natural ordering
  Tag!("ltx:*", after_open_late => sub[document,node] {
    if node.has_attribute("_xmkey") {
      let qname = document::get_node_qname(node);
      if (qname != arena::pin_static("ltx:XMRef")) &&
        arena::with(qname, |qstr| qstr.starts_with("ltx:XM")) && !node.has_attribute("xml:id") {
        document.generate_id(node, "")?;
      }
    }
  });

  Tag!("ltx:XMDual", after_close_late => sub[document,node] {
    let mut ids  = HashMap::default();
    let mut refs = Vec::new();
    // Collect all children with _xmkey attribute
    for mut n in document.findnodes("descendant::*[@_xmkey]", Some(node)) {
      if document::with_node_qname(&n, |qname| qname == "ltx:XMRef")
          && !n.has_attribute("idref") {
        refs.push(n);    // we'll fill these in next
      } else { // generate & record ids for all referenced noces
        let key = n.get_attribute("_xmkey").unwrap();
        if let Entry::Vacant(e) = ids.entry(key) {
          document.generate_id(&mut n, "")?; // Generate id if none already.
          e.insert(n.get_attribute_ns("id",XML_NS).unwrap_or_default());
        }
      }
    }
    for mut r in refs {                        // Now fill in the references
      let r_xmkey = r.get_attribute("_xmkey").unwrap();
      if let Some(idref) = ids.get(&r_xmkey) {
        document.set_attribute(&mut r, "idref", idref)?;
      } else {
        // xmkey not resolved — may happen with parser-generated nested structures
        Warn!("expected", "id", s!("Unresolved _xmkey '{}' in createXMRefs", r_xmkey));
      }
      r.remove_attribute("_xmkey")?;
    }
  });

  // # Construction aids
  // # Build an XMDual (via \lx@dual) given the content & presentation forms.
  // # These forms are provided as Tokens, invoking the appropriate constructor macros,
  // # and refering to any arguments using #1, #2.... (see T_XMArg for syntactic sugar)
  // # The arguments (if any) are given separately; within the content & presentation
  // # they are replaced by \lx@xmref and \lx@xmarg, appropriately,
  // # so that they will be linked/shared in the XML tree.
  // # The keyvals argument is a hash containing any properties of the construct,
  // # along with reversion, content_reversion  & presentation_reversion, which are
  // # substituted for arguments as well.
  // sub I_dual {
  //   my ($keyvals, $content, $presentation, @args) = @_;
  //   $content      = TokenizeInternal($content)      if $content      && !ref $content;
  //   $presentation = TokenizeInternal($presentation) if $presentation && !ref $presentation;
  //   my (@revargs, @pargs, @cargs);
  //   foreach my $arg (@args) {
  //     my $id = LaTeXML::Package::getXMArgID();
  //     push(@revargs, Tokens(I_arg(ToString($id))));
  //     push(@pargs,   Invocation(T_CS('\lx@xmarg'), $id, $arg));
  //     push(@cargs,   Invocation(T_CS('\lx@xmref'), $id)); }
  //   my $optional = undef;
  //   if ($keyvals) {
  //     my @options = ();
  //     while (my ($key, $value) = each %$keyvals) {
  //       $value = TokenizeInternal($value) if $value && !ref $value;
  //       if ($key =~ /^(?:presentation_|content_|)reversion$/) {
  //         $value = $value->substituteParameters(@revargs); }
  //       push(@options, T_OTHER(',')) if @options;
  //       push(@options, T_OTHER($key), T_OTHER('='), T_BEGIN, $value, T_END); }
  //     $optional = Tokens(@options); }
  //   return
  //     Invocation(T_CS('\lx@dual'), $optional,
  //     $content->substituteParameters(@cargs),
  //     I_wrap({}, $presentation->substituteParameters(@pargs))); }

  // # A little helper to shorten things up a bit; simply generates #1 (or whatever)
  // sub I_arg {    # uncoditionally create an arg token
  //   return bless ["$_[0]", CC_ARG], 'LaTeXML::Core::Token'; }

  // sub I_xmarg {
  //   my ($id, $arg) = @_;
  //   return Tokens(T_CS('\lx@xmarg'),
  //     T_BEGIN, (ref $id ? $id : T_OTHER($id)), T_END, T_BEGIN, $arg, T_END); }

  // sub I_xmref {
  //   my ($id) = @_;
  //   return Tokens(T_CS('\lx@xmref'), T_BEGIN, (ref $id ? $id : T_OTHER($id)), T_END); }

  //======================================================================

  // We OUGHT to be able to do this using \llap,\rlap,\hss...
  DefMacro!(
    "\\lx@tweaked{}{}",
    r"\ifmmode\lx@math@tweaked{#1}{#2}\else\lx@text@tweaked{#1}{#2}\fi"
  );
  // Perl: DefConstructor('\lx@math@tweaked RequiredKeyVals {}',
  //   "<ltx:XMWrap $XMath_attributes>#2</ltx:XMWrap>", ...);
  DefConstructor!("\\lx@math@tweaked RequiredKeyVals {}",
    "<ltx:XMWrap role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'>#2</ltx:XMWrap>",
    reversion => "#2",
    after_digest => sub[whatsit] { xmath_copy_keyvals(whatsit) }
  );

  // Perl: DefConstructor('\lx@text@tweaked RequiredKeyVals {}',
  //   "<ltx:text _noautoclose='1' %&GetKeyVals(#1)>#2</ltx:text>", ...);
  // Properties from keyvals are copied by after_digest, then referenced as #prop in template.
  DefConstructor!("\\lx@text@tweaked RequiredKeyVals {}",
    "<ltx:text width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset'>#2</ltx:text>",
    after_digest => sub[whatsit] { xmath_copy_keyvals(whatsit) }
  );

  DefConstructor!(T_CS!("\\lx@ldots"), None,
  "?#isMath(<ltx:XMTok name='ldots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})",
  sizer      => "\u{2026}",
  reversion  => "\\ldots",
  properties => {
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      Ok(stored_map!("font" => lookup_font().unwrap().merge(
        fontmap!(family => "serif", series => "medium", shape => "upright")
          .specialize("\u{2026}"))))
    } else {
          // Since not DefMath!
          // And so can \vdots
      Ok(SymHashMap::default())
    }});
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Support for rewrite rules
  //**********************************************************************
  DefConstructor!("\\WildCard[]", "<_WildCard_>#1</_WildCard_>");
  DefConstructor!("\\WildCardA", "<_WildCard_/>");
  DefConstructor!("\\WildCardB", "<_WildCard_/>");
  DefConstructor!("\\WildCardC", "<_WildCard_/>");

  //======================================================================
  // Properties for plain characters.
  // These are allowed in plain text, but need to act a bit special in math.
  DefMath!('=', None, '=', role => "RELOP",   meaning  => "equals");
  DefMath!('+', None, '+', role => "ADDOP",   meaning  => "plus");
  DefMath!('-', None, '-', role => "ADDOP",   meaning  => "minus");
  DefMath!('*', None, "\u{2217}", role => "MULOP", meaning => "times"); // U+2217 ASTERISK OPERATOR
  DefMath!('/', None, '/', role => "MULOP",   meaning  => "divide");
  DefMath!('!', None, '!', role => "POSTFIX", meaning  => "factorial");
  DefMath!(',', None, ',', role => "PUNCT");
  DefMath!('.', None, '.', role => "PERIOD");
  DefMath!(';', None, ';', role => "PUNCT");
  DefMath!('(', None, '(', role => "OPEN",    stretchy => false);
  DefMath!(')', None, ')', role => "CLOSE",   stretchy => false);
  DefMath!('[', None, '[', role => "OPEN",    stretchy => false);
  DefMath!(']', None, ']', role => "CLOSE",   stretchy => false);
  DefMath!('|', None, '|', role => "VERTBAR", stretchy => false);
  DefMath!(':', None, ':', role => "METARELOP", name => "colon"); // Seems like good default role
  DefMath!('<', None, '<', role => "RELOP", meaning => "less-than");
  DefMath!('>', None, '>', role => "RELOP", meaning => "greater-than");

  // NOTE: Need to evolve Ligatures to be easier to write.
  // rough draft of tool to make ligatures more sane to write...
  // It is tempting to handle these with macros,
  // But that tends to run afoul of tricky packages like babel that make : active as well!
  // Even using mathactive doesn't help.
  // sub TestNode {
  //   my ($node, $qname, $content, %attrib) = @_;
  //   return $node
  //     && ($LaTeXML::DOCUMENT->getModel->getNodeQName($node) eq $qname)
  //     && ((!defined $content) || (($node->textContent || '') eq $content))
  //     && !grep { $node->getAttribute($_) ne $attrib{$_} } keys %attrib; }

  // Recognize !!
  DefMathLigature!("!!", "!!", role => "POSTFIX", meaning => "double-factorial");
  // Recognize :=
  DefMathLigature!(":=", ":=", role => "RELOP", meaning => "assign");

  //======================================================================
  // Combine letters, when the fonts are right. (sorta related to mathcode)
  // well, maybe a letter followed by letters & digits?
  DefMathLigature!(matcher => sub [document,node_opt] {
    //  let mut chars :Vec<char> = Vec::new();
     let font  = document.get_node_font(node_opt);
     let mut this_node;
     let mut node_mut = node_opt;
     if font.is_sticky() {
       let mut n      = 0;
       let mut text = String::new();
       loop {
         if model::with_node_qname(node_mut, |qname| qname != "ltx:XMTok")
          || document.get_node_font(node_mut) != font
          || node_mut.has_attribute("name") {
            break;
          }
         match node_mut.get_attribute("role") {
           Some(role) if role != "UNKNOWN" && role != "NUMBER" => break,
           _ => {}
         };
         let node_text = node_mut.get_content();
         if !node_text.chars().all(|c| c.is_alphanumeric()) {
           break;
         }
         n+=1;
         text = node_text + &text;
         if let Some(sibling) = node_mut.get_prev_sibling() {
           this_node = sibling;
           node_mut = &mut this_node;
         } else {
           break;
         }
       }
       let has_leading_letter = match text.chars().next() {
         Some(fc) => fc.is_alphabetic(),
         None => false
       };
       if has_leading_letter && n > 1 {
         Ok(Some((n, text, MathLigatureOptions {
           role: Some("UNKNOWN".to_string()),
           meaning: None,
           name: None }))) }
       else {
         Ok(None)
       }
     } else {
       Ok(None)
     }
  });

  //======================================================================
  // Combine digits in math.
  DefMath!('0', None, '0', role => "NUMBER", meaning => "0");
  DefMath!('1', None, '1', role => "NUMBER", meaning => "1");
  DefMath!('2', None, '2', role => "NUMBER", meaning => "2");
  DefMath!('3', None, '3', role => "NUMBER", meaning => "3");
  DefMath!('4', None, '4', role => "NUMBER", meaning => "4");
  DefMath!('5', None, '5', role => "NUMBER", meaning => "5");
  DefMath!('6', None, '6', role => "NUMBER", meaning => "6");
  DefMath!('7', None, '7', role => "NUMBER", meaning => "7");
  DefMath!('8', None, '8', role => "NUMBER", meaning => "8");
  DefMath!('9', None, '9', role => "NUMBER", meaning => "9");

  // This is getting out-of-hand;
  // (1) this gets done after document build, so we query the document/node for language
  // rather than using something specified during digestion (eg. macros, roles...)
  // (2) the way we've specified the decimal & thousands separators (language dependent)
  // is completely insufficient; should leverage numprint or babel or ... ?
  // (3) the way we're detecting the chars is a mess: a mix of string content & role!
  // If we could accommodate multiple roles, maybe a separate role could be set on the tokens
  // (a period could be a PERIOD or a DECIMAL_SEPARATOR, eg)

  DefMathLigature!(matcher => sub[document, node] {
  let lang = document.get_node_language(node);
  let lang = lang.split('-').next().unwrap(); // strip off region code, if any.
  let dec     = DECIMAL_SEP.get(lang).unwrap_or(&".");
  let thou    = THOUSANDS_SEP.get(lang).unwrap_or(&",");
  let decrole = if dec == &"." { "PERIOD" } else { "" };
  // let mut chars : Vec<char> = Vec::new();
  let (mut n, mut combined, mut number, _w, mut font) = (0, String::new(), String::new(), 0, None);
  //     NOTE: We're scanning chars from END!
  let mut node_ref = node;
  let mut current;
  loop {
    let qn = model::get_node_qname(node_ref);
    if qn == arena::pin_static("ltx:XMTok") || qn == arena::pin_static("ltx:XMWrap") {
      let r = node_ref.get_attribute("role").unwrap_or_default();
      let f    = document.get_node_font(node_ref);
      let text = node_ref.get_content();
      //  A number in same font?
      if r=="NUMBER" && (font.is_none() || font.as_ref().unwrap() == &f) {
        font = Some(f);
        combined = text + &combined;
        if let Some(m) = node_ref.get_attribute("meaning") {
          number = m + &number;
        }
      } else if n == 0 { // any following cases are not allowed as LAST char
        break;
      }

      // if thousands separator (but NOT simultaneously PUNCT!!!! Be paranoid about lists)
      else if text.as_str() == *thou && r != "PUNCT" {
        combined = text + &combined; // Add to string, but omit from number
      } else if text.as_str() == *dec || r == decrole {
        // if decimal separator, turn it into "standard" "."
        combined = node_ref.get_content() + &combined;
        number = String::from('.') + &number;
      } else {
        break;
      }
    // OR if XMHint with 0 <= width <= thickmuskip (5mu == ?)
    } else if qn == arena::pin_static("ltx:XMHint") {
      if let Some(s_name) = node_ref.get_attribute("name") {
        if let Some(s_char) = NAMED_SPACE_CHARS.get(s_name.as_str()) {
          combined = s_char.to_string() + &combined;
        } else {
          break;
        }
      } else {
         break;
       }
    } else {
      break;
    }
    n+=1;
    if let Some(sibling) = node_ref.get_prev_sibling() {
      current = sibling;
      node_ref = &mut current;
    } else {
      break;
    }
  }
  if n > 1 && (number.chars().any(|c| c.is_numeric())) {
    Ok(Some((n, combined, MathLigatureOptions {
      meaning: Some(number),
      role: Some("NUMBER".to_string()), .. MathLigatureOptions::default()})))
    } else {
      Ok(None)
    }
  });

  // This needs to be applied AFTER numbers have been resolved!
  // If we have a non-negative integer (no signs, decimals,...)
  // followed by a fraction dividing two non-negative integers,
  // Figure it's a mixed fraction --- ADDING the fraction to the number, not multiplying!
  // DefRewrite(select => ['descendant-or-self::ltx:XMTok[@role="NUMBER" and
  // translate(@meaning,"0123456789","")=""]'       . '[ following-sibling::*[1][self::ltx:XMApp]'
  //       . ' [child::*[1][self::ltx:XMTok[@meaning="divide"]]]'
  //       . ' [child::*[2]['
  //       . 'self::ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . 'or self::ltx:XMArg[count(child::*)=1]/ltx:XMTok[@role="NUMBER" and
  // translate(@meaning,"0123456789","")=""]'       . ']]'
  //       . ' [child::*[3]['
  //       . 'self::ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . 'or self::ltx:XMArg[count(child::*)=1]/ltx:XMTok[@role="NUMBER" and
  // translate(@meaning,"0123456789","")=""]'       . ']]'
  //       . ']',
  //     2],
  //   replace => sub { my ($document, $number, $frac) = @_;
  //     my $box = $document->getNodeBox($number);
  //     $document->openElement('ltx:XMApp', _box => $box);
  //     $document->insertMathToken("\x{2064}",    # Invisible Plus!
  //       meaning => 'plus', role => "ADDOP", _box => $box);
  //     $document->getNode->appendChild($number);
  //     $document->getNode->appendChild($frac);
  //     $document->closeElement('ltx:XMApp'); });

  //----------------------------------------------------------------------
  // Matrices;  Generalized

  // The delimiters around a matrix may simply be notational, or for readability,
  // and don't affect the "meaning" of the array structure as a matrix.
  // In that case, we'll use an XMDual to indidate the content is simply the matrix,
  // but the presentation includes the delimiters.
  // HOWEVER, the delimiters may also signify an OPERATION on the matrix
  // in which case the application & meaning of that operator must be supplied.

  // keys are
  //  name  : the name of the environment (for reversion)
  //  datameaning: the (presumed) meaning of the array construct (typically 'matrix')
  //  delimitermeaning  : the operator meaning due to delimiters (eg. norm)(as applied to the array)
  //  style : typically \displaystyle, \textstyle...
  //  left  : TeX code for left of matrix
  //  right  : TeX code for right
  //  ncolumns : the number of columns (default is not limited)
  DefKeyVal!("lx@GEN", "style", "UndigestedKey");
  // Perl uses these keys via getValue/constructor templates without formal DefKeyVal.
  // Declare them to suppress spurious Info messages about unknown keys.
  DefKeyVal!("lx@GEN", "name", "");
  DefKeyVal!("lx@GEN", "meaning", "");
  DefKeyVal!("lx@GEN", "datameaning", "");
  DefKeyVal!("lx@GEN", "delimitermeaning", "");
  DefKeyVal!("lx@GEN", "left", "");
  DefKeyVal!("lx@GEN", "right", "");
  DefKeyVal!("lx@GEN", "ncolumns", "");
  DefKeyVal!("lx@GEN", "alignment", "");
  DefKeyVal!("lx@GEN", "rowsep", "");
  DefKeyVal!("lx@GEN", "conditionmode", "");

  // Perl: Base_XMath.pool.ltxml line 575
  DefPrimitive!("\\lx@gen@matrix@bindings RequiredKeyVals:lx@GEN", sub[(kv)] {
    use latexml_core::alignment::cell::Cell;
    use latexml_core::alignment::template::TemplateConfig;
    use crate::engine::tex_tables::alignment_bindings;

    bgroup();
    // style defaults to \textstyle
    let style_tok = kv.get_value("style")
      .map(|a| {
        let s = a.to_string();
        if s.starts_with('\\') {
          T_CS!(&s)
        } else {
          T_CS!("\\textstyle")
        }
      })
      .unwrap_or_else(|| T_CS!("\\textstyle"));
    let align = kv.get_value("alignment")
      .map(ToString::to_string)
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| String::from("c"));
    let ncols_str = kv.get_value("ncolumns").map(ToString::to_string).unwrap_or_default();
    let ncols: usize = ncols_str.parse().unwrap_or(0);

    // Build column spec: before = hfil? + style, after = hfil?
    let mut before_toks = Vec::new();
    if align.starts_with('c') || align.starts_with('r') {
      before_toks.push(T_CS!("\\hfil"));
    }
    before_toks.push(style_tok);

    let mut after_toks = Vec::new();
    if align.starts_with('c') || align.starts_with('l') {
      after_toks.push(T_CS!("\\hfil"));
    }

    // Set explicit alignment on cell so it propagates to XMCell align= attribute
    let cell_align = match align.as_str() {
      "l" => Some(Align::Left),
      "r" => Some(Align::Right),
      _ => Some(Align::Center), // default "c"
    };
    let col = Cell {
      align: cell_align,
      before: Some(Tokens::new(before_toks)),
      after: if after_toks.is_empty() { None } else { Some(Tokens::new(after_toks)) },
      empty: true,
      ..Cell::default()
    };

    let template = if ncols > 0 {
      Template::new(TemplateConfig {
        columns: Some((0..ncols).map(|_| col.clone()).collect()),
        ..TemplateConfig::default()
      })
    } else {
      Template::new(TemplateConfig {
        repeated: vec![col],
        ..TemplateConfig::default()
      })
    };

    // Collect xml attributes (e.g. rowsep)
    let mut xml_attributes = HashMap::default();
    if let Some(rowsep) = kv.get_value("rowsep") {
      xml_attributes.insert(String::from("rowsep"), rowsep.to_string());
    }

    let properties = SymHashMap::default();
    alignment_bindings(template, String::from("math"), properties, xml_attributes);
    state::let_i(&T_CS!("\\\\"), &T_CS!("\\lx@alignment@newline"), None);
    state::let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@math@intercol"), None);
    // Disable special row treatment (eg. numbering) unless requested
    state::let_i(&T_CS!("\\lx@alignment@row@before"), &T_CS!("\\lx@empty"), None);
    state::let_i(&T_CS!("\\lx@alignment@row@after"), &T_CS!("\\lx@empty"), None);
  });

  DefPrimitive!("\\lx@end@gen@matrix", {
    egroup()?;
  });

  DefMacro!(
    "\\lx@gen@plain@matrix{}{}",
    "\\lx@gen@matrix@bindings{#1}\
      \\lx@gen@plain@matrix@{#1}{\\lx@begin@alignment#2\\lx@end@alignment}\\lx@end@gen@matrix"
  );

  // Perl: Base_XMath.pool.ltxml line 610 — \lx@gen@plain@matrix@
  // The delimiters on a matrix are presumably just for notation or readability (not an operator);
  // the array data itself is the matrix.
  DefConstructor!("\\lx@gen@plain@matrix@ RequiredKeyVals:lx@GEN {}",
    "?#needXMDual(\
       <ltx:XMDual>\
         ?#delimitermeaning(<ltx:XMApp><ltx:XMTok meaning='#delimitermeaning'/>)()\
         ?#datameaning(<ltx:XMApp><ltx:XMTok meaning='#datameaning'/>)()\
         <ltx:XMRef _xmkey='#xmkey'/>\
         ?#delimitermeaning(</ltx:XMApp>)()\
         ?#datameaning(</ltx:XMApp>)()\
         <ltx:XMWrap>#left<ltx:XMArg _xmkey='#xmkey'>#2</ltx:XMArg>#right</ltx:XMWrap>\
       </ltx:XMDual>\
     )(\
       #2\
     )",
    properties => sub[args] {
      // Perl: properties => sub { %{ $_[1]->getKeyVals }; }
      // Pass all keyval pairs as properties
      let mut props = stored_map!();
      if let Some(d) = &args[0] {
        if let DigestedData::KeyVals(ref kv) = d.data() {
          // Store left/right as Digested directly from digested keyvals.
          for prop_key in &["left", "right"] {
            if let Some(digested) = kv.get_value_digested(prop_key) {
              props.insert(prop_key, Stored::Digested(digested.clone()));
            }
          }
          for (k, v) in kv.get_pairs() {
            if k != "left" && k != "right" {
              props.insert(k, Stored::String(arena::pin(v.to_string())));
            }
          }
        }
      }
      Ok(props)
    },
    after_digest => sub[whatsit] {
      // Perl: afterDigest — check if XMDual is needed, store alignment
      // Check if datameaning or delimitermeaning is set
      let has_datameaning = whatsit.get_property("datameaning")
        .is_some_and(|v| !v.to_string().is_empty());
      let has_delimmeaning = whatsit.get_property("delimitermeaning")
        .is_some_and(|v| !v.to_string().is_empty());
      if has_datameaning || has_delimmeaning {
        whatsit.set_property("needXMDual", "1");
        whatsit.set_property("xmkey", get_xmarg_id()?);
      }
      // Perl: $whatsit->setProperties(alignment => LookupValue('Alignment'));
      if let Some(alignment) = state::lookup_alignment() {
        whatsit.set_property("alignment", Stored::Digested(alignment));
      }
      Ok(Vec::new())
    },
    reversion => sub[_whatsit, args] {
      // Perl: reversion => sub { my ($whatsit, $kv, $body) = @_;
      //   my $name = ToString($kv->getValue('name'));
      //   my $alignment = $whatsit->getProperty('alignment');
      //   (T_CS('\\' . $name), T_BEGIN, $alignment->revert, T_END); }
      let mut name = String::new();
      if let Some(d) = &args[0] {
        if let DigestedData::KeyVals(ref kv) = d.data() {
          name = kv.get_value("name").map(|v| v.to_string()).unwrap_or_default();
        }
      }
      // Get alignment reversion from the whatsit property
      let alignment_rev = {
        let prop = _whatsit.get_property("alignment");
        let mut rev = None;
        if let Some(cow) = prop.as_ref() {
          if let Stored::Digested(ref alignment) = &**cow {
            if let DigestedData::Alignment(ref al) = alignment.data() {
              rev = Some(al.borrow().revert()?);
            }
          }
        }
        rev.unwrap_or(match &args[1] { Some(inner) => inner.revert()?, None => Tokens!() })
      };
      let cs_name = format!("\\{}", name);
      let mut tks = vec![T_CS!(&cs_name), T_BEGIN!()];
      tks.extend(alignment_rev.unlist());
      tks.push(T_END!());
      Ok(Tokens::new(tks))
    }
  );

  // Perl: Base_XMath.pool.ltxml line 644 — \lx@ams@matrix@
  // Similar to \lx@gen@plain@matrix@, but takes DigestedBody for ams environments
  DefConstructor!("\\lx@ams@matrix@ RequiredKeyVals:lx@GEN DigestedBody",
    "?#needXMDual(\
       <ltx:XMDual>\
         ?#delimitermeaning(<ltx:XMApp><ltx:XMTok meaning='#delimitermeaning'/>)()\
         ?#datameaning(<ltx:XMApp><ltx:XMTok meaning='#datameaning'/>)()\
         <ltx:XMRef _xmkey='#xmkey'/>\
         ?#delimitermeaning(</ltx:XMApp>)()\
         ?#datameaning(</ltx:XMApp>)()\
         <ltx:XMWrap>#left<ltx:XMArg _xmkey='#xmkey'>#2</ltx:XMArg>#right</ltx:XMWrap>\
       </ltx:XMDual>\
     )(\
       #2\
     )",
    properties => sub[args] {
      let mut props = stored_map!();
      if let Some(d) = &args[0] {
        if let DigestedData::KeyVals(ref kv) = d.data() {
          // Store left/right as Digested directly from digested keyvals.
          // Perl's absorb() handles Tokens natively; Rust constructor templates
          // only absorb Digested. Using get_value_digested avoids the revert->re-digest
          // round-trip that loses the original \lx@left CS (alias resolves to \left on revert).
          for prop_key in &["left", "right"] {
            if let Some(digested) = kv.get_value_digested(prop_key) {
              props.insert(prop_key, Stored::Digested(digested.clone()));
            }
          }
          for (k, v) in kv.get_pairs() {
            if k != "left" && k != "right" {
              props.insert(k, Stored::String(arena::pin(v.to_string())));
            }
          }
        }
      }
      Ok(props)
    },
    after_digest => sub[whatsit] {
      let has_datameaning = whatsit.get_property("datameaning")
        .is_some_and(|v| !v.to_string().is_empty());
      let has_delimmeaning = whatsit.get_property("delimitermeaning")
        .is_some_and(|v| !v.to_string().is_empty());
      if has_datameaning || has_delimmeaning {
        whatsit.set_property("needXMDual", "1");
        whatsit.set_property("xmkey", get_xmarg_id()?);
      }
      Ok(Vec::new())
    },
    reversion => sub[_whatsit, args] {
      // Perl: reversion => sub { my ($whatsit, $kv, $body) = @_;
      //   my $name  = $kv->getValue('name');
      //   my $align = $kv->getValue('alignment');
      //   ($name ? (T_CS('\begin'), T_BEGIN, Revert($name), T_END) : ()),
      //   (IsEmpty($align) ? ()
      //     : ($kv->getValue('alignment-required')
      //       ? (T_BEGIN, Revert($align), T_END) : (T_OTHER('['), Revert($align), T_OTHER(']')))),
      //   Revert($body),
      //   ($name ? (T_CS('\end'), T_BEGIN, Revert($name), T_END) : ())); }
      let mut name = String::new();
      let mut align = String::new();
      let mut alignment_required = false;
      if let Some(d) = &args[0] {
        if let DigestedData::KeyVals(ref kv) = d.data() {
          name = kv.get_value("name").map(|v| v.to_string()).unwrap_or_default();
          align = kv.get_value("alignment").map(|v| v.to_string()).unwrap_or_default();
          alignment_required = kv.has_key("alignment-required");
        }
      }
      let body_rev = match &args[1] { Some(inner) => inner.revert()?, None => Tokens!() };
      let mut tks = Vec::new();
      if !name.is_empty() {
        tks.push(T_CS!("\\begin"));
        tks.push(T_BEGIN!());
        tks.extend(Explode!(&name));
        tks.push(T_END!());
      }
      // Perl: emit alignment spec if present
      if !align.is_empty() {
        if alignment_required {
          tks.push(T_BEGIN!());
          tks.extend(Explode!(&align));
          tks.push(T_END!());
        } else {
          tks.push(T_OTHER!("["));
          tks.extend(Explode!(&align));
          tks.push(T_OTHER!("]"));
        }
      }
      tks.extend(body_rev.unlist());
      if !name.is_empty() {
        tks.push(T_CS!("\\end"));
        tks.push(T_BEGIN!());
        tks.extend(Explode!(&name));
        tks.push(T_END!());
      }
      Ok(Tokens::new(tks))
    }
  );

  //----------------------------------------------------------------------
  // Cases: Generalized
  // keys are
  //  name  : the name of the command (for reversion)
  //  meaning: the (presumed) meaning of the construct
  //  style : \textstyle or \displaystyle
  //  conditionmode : mode of 2nd column, text or math
  //  left  : TeX code for left of cases
  //  right  : TeX code for right

  // Perl Base_XMath.pool.ltxml L695-699:
  //   DefConstructorI('\lx@cases@condition', undef,
  //     "<ltx:XMText>#body</ltx:XMText>",
  //     alias => '', beforeDigest => sub { $_[0]->beginMode('restricted_horizontal'); },
  //     captureBody => 1);
  //   DefConstructorI('\lx@cases@end@condition', undef, "", alias => '',
  //     beforeDigest => sub { $_[0]->endMode('restricted_horizontal'); });
  // Mirrors \lx@begin@inmath@text / \lx@end@inmath@text pattern at
  // tex_math.rs:501-510 — begin/end restricted-horizontal spanning the
  // captured body, emitting <ltx:XMText> wrapper around it.
  DefConstructor!("\\lx@cases@condition",
    "<ltx:XMText>#body</ltx:XMText>",
    alias => "",
    before_digest => sub { begin_mode("restricted_horizontal")?; },
    capture_body => true
  );
  DefConstructor!("\\lx@cases@end@condition", "",
    alias => "",
    before_digest => sub { end_mode("restricted_horizontal")?; });

  // Perl: Base_XMath.pool.ltxml line 701
  DefPrimitive!("\\lx@gen@cases@bindings RequiredKeyVals:lx@GEN", sub[(kv)] {
    use latexml_core::alignment::cell::Cell;
    use latexml_core::alignment::template::TemplateConfig;
    use crate::engine::tex_tables::alignment_bindings;

    bgroup();
    let style_tok = kv.get_value("style")
      .map(|a| {
        let s = a.to_string();
        if s.starts_with('\\') { T_CS!(&s) } else { T_CS!("\\textstyle") }
      })
      .unwrap_or_else(|| T_CS!("\\textstyle"));
    let condmode = kv.get_value("conditionmode").map(ToString::to_string).unwrap_or_default();
    let _condtext = condmode == "text";

    // Column 1: value (style + \hfil after)
    let col1 = Cell {
      before: Some(Tokens::new(vec![style_tok])),
      after: Some(Tokens::new(vec![T_CS!("\\hfil")])),
      empty: true,
      ..Cell::default()
    };
    // Column 2: condition (style + optional text mode, trim right + \hfil after)
    // TODO: add \lx@cases@condition / \lx@cases@end@condition for condtext=true
    let mut before2 = vec![style_tok];
    let mut after2 = vec![T_CS!("\\lx@column@trimright"), T_CS!("\\hfil")];
    if _condtext {
      // Perl: before => Tokens($style, T_CS('\lx@cases@condition'))
      // Perl: after  => Tokens(T_CS('\lx@column@trimright'), T_CS('\lx@cases@end@condition'), T_CS('\hfil'))
      before2.push(T_CS!("\\lx@cases@condition"));
      after2 = vec![T_CS!("\\lx@column@trimright"), T_CS!("\\lx@cases@end@condition"), T_CS!("\\hfil")];
    }

    let col2 = Cell {
      before: Some(Tokens::new(before2)),
      after: Some(Tokens::new(after2)),
      empty: true,
      ..Cell::default()
    };

    let template = Template::new(TemplateConfig {
      columns: Some(vec![col1, col2]),
      ..TemplateConfig::default()
    });

    let properties = SymHashMap::default();
    alignment_bindings(template, String::from("math"), properties, HashMap::default());
    state::let_i(&T_CS!("\\\\"), &T_CS!("\\lx@alignment@newline"), None);
    state::let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@math@intercol"), None);
    def_macro(T_CS!("\\lx@alignment@row@before"), None, Tokens!(), None)?;
    def_macro(T_CS!("\\lx@alignment@row@after"), None, Tokens!(), None)?;
  });

  DefMacro!(
    "\\lx@gen@plain@cases{}{}",
    "\\lx@gen@cases@bindings{#1}\
      \\lx@gen@plain@cases@{#1}{\\lx@begin@alignment#2\\lx@end@alignment}
      \\lx@end@gen@cases"
  );
  DefPrimitive!("\\lx@end@gen@cases", {
    egroup()?;
  });

  // Perl: Base_XMath.pool.ltxml line 730
  // The logical structure for cases extracts the columns of the alignment
  // to give alternating value,condition (empty conditions become "otherwise")
  DefConstructor!("\\lx@gen@plain@cases@ RequiredKeyVals:lx@GEN {}",
    "<ltx:XMWrap>#left#2#right</ltx:XMWrap>",
    alias => "\\cases",
    reversion => "\\cases{#2}",
    properties => sub[args] {
      let mut props = stored_map!();
      if let Some(d) = &args[0] {
        if let DigestedData::KeyVals(ref kv) = d.data() {
          for prop_key in &["left", "right"] {
            if let Some(digested) = kv.get_value_digested(prop_key) {
              props.insert(prop_key, Stored::Digested(digested.clone()));
            }
          }
          for (k, v) in kv.get_pairs() {
            if k != "left" && k != "right" {
              props.insert(k, Stored::String(arena::pin(v.to_string())));
            }
          }
        }
      }
      Ok(props)
    },
    after_construct => sub[document, _whatsit] {
      // Perl afterConstruct: wrap in XMDual with meaning='cases'
      // Get the XMWrap we just created (last child of current element)
      if let Some(current) = document.get_element() {
        if let Some(mut point) = current.get_last_element_child() {
          let cells = document.findnodes("ltx:XMArray/ltx:XMRow/ltx:XMCell", Some(&point));
          // Strip "align" from empty/whitespace-only cells
          // (Perl doesn't set align on empty condition cells in \cases)
          for mut cell in cells.iter().cloned() {
            if cell.get_child_elements().is_empty() {
              // If no elements and only whitespace content, treat as empty
              let content = cell.get_content();
              if content.trim().is_empty() {
                // Remove whitespace text nodes
                for mut child in cell.get_child_nodes() {
                  child.unlink();
                }
                cell.remove_attribute("align").ok();
              }
            }
          }
          if !cells.is_empty() {
            // Collect XMRef ids for non-empty cells, "otherwise" text for empty cells
            let mut ref_ids: Vec<Option<String>> = Vec::new();
            for cell in &cells {
              if !cell.get_child_elements().is_empty() {
                // Generate id on the cell's content and create XMRef to it
                for mut child in cell.get_child_elements() {
                  document.generate_id(&mut child, "")?;
                  let id = child.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
                    .unwrap_or_default();
                  ref_ids.push(Some(id));
                }
              } else {
                ref_ids.push(None); // Will become <XMText>otherwise</XMText>
              }
            }
            // Build XMDual structure around the XMWrap
            if let Some(mut parent) = point.get_parent() {
              let mut xm_dual =
                document.open_element_at(&mut parent, "ltx:XMDual", None, None)?;
              point.add_prev_sibling(&mut xm_dual).ok();
              // Content: <XMApp><XMTok meaning="cases"/> XMRefs/XMTexts </XMApp>
              let mut xm_app =
                document.open_element_at(&mut xm_dual, "ltx:XMApp", None, None)?;
              let mut tok_attrs = HashMap::default();
              tok_attrs.insert("meaning".to_string(), "cases".to_string());
              let mut xm_tok =
                document.open_element_at(&mut xm_app, "ltx:XMTok", Some(tok_attrs), None)?;
              document.close_element_at(&mut xm_tok)?;
              for ref_id_opt in &ref_ids {
                if let Some(id) = ref_id_opt {
                  let mut ref_attrs = HashMap::default();
                  ref_attrs.insert("idref".to_string(), id.clone());
                  let mut xm_ref =
                    document.open_element_at(&mut xm_app, "ltx:XMRef", Some(ref_attrs), None)?;
                  document.close_element_at(&mut xm_ref)?;
                } else {
                  let mut xm_text =
                    document.open_element_at(&mut xm_app, "ltx:XMText", None, None)?;
                  let _ = xm_text.set_content("otherwise");
                  document.close_element_at(&mut xm_text)?;
                }
              }
              document.close_element_at(&mut xm_app)?;
              // Move original XMWrap into XMDual (presentation branch)
              point.unlink_node();
              xm_dual.add_child(&mut point)?;
              document.close_element_at(&mut xm_dual)?;
            }
          }
        }
      }
    }
  );

  // Perl: amsmath.sty.ltxml line 694 (defined in Perl Package but logically belongs here)
  // AMS variant of cases — takes DigestedBody
  DefConstructor!("\\lx@ams@cases@ RequiredKeyVals:lx@GEN DigestedBody",
    "<ltx:XMWrap>#left#2#right</ltx:XMWrap>",
    alias => "\\begin{cases}",
    reversion => "\\begin{cases}#2\\end{cases}",
    properties => sub[args] {
      let mut props = stored_map!();
      if let Some(d) = &args[0] {
        if let DigestedData::KeyVals(ref kv) = d.data() {
          for prop_key in &["left", "right"] {
            if let Some(digested) = kv.get_value_digested(prop_key) {
              props.insert(prop_key, Stored::Digested(digested.clone()));
            }
          }
          for (k, v) in kv.get_pairs() {
            if k != "left" && k != "right" {
              props.insert(k, Stored::String(arena::pin(v.to_string())));
            }
          }
        }
      }
      Ok(props)
    },
    after_construct => sub[document, _whatsit] {
      // Same as \lx@gen@plain@cases@ — wrap in XMDual with meaning='cases'
      if let Some(current) = document.get_element() {
        if let Some(mut point) = current.get_last_element_child() {
          let cells = document.findnodes("ltx:XMArray/ltx:XMRow/ltx:XMCell", Some(&point));
          if !cells.is_empty() {
            let mut ref_ids: Vec<Option<String>> = Vec::new();
            for cell in &cells {
              if !cell.get_child_elements().is_empty() {
                for mut child in cell.get_child_elements() {
                  document.generate_id(&mut child, "")?;
                  let id = child.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
                    .unwrap_or_default();
                  ref_ids.push(Some(id));
                }
              } else {
                ref_ids.push(None);
              }
            }
            if let Some(mut parent) = point.get_parent() {
              let mut xm_dual =
                document.open_element_at(&mut parent, "ltx:XMDual", None, None)?;
              point.add_prev_sibling(&mut xm_dual).ok();
              let mut xm_app =
                document.open_element_at(&mut xm_dual, "ltx:XMApp", None, None)?;
              let mut tok_attrs = HashMap::default();
              tok_attrs.insert("meaning".to_string(), "cases".to_string());
              let mut xm_tok =
                document.open_element_at(&mut xm_app, "ltx:XMTok", Some(tok_attrs), None)?;
              document.close_element_at(&mut xm_tok)?;
              for ref_id_opt in &ref_ids {
                if let Some(id) = ref_id_opt {
                  let mut ref_attrs = HashMap::default();
                  ref_attrs.insert("idref".to_string(), id.clone());
                  let mut xm_ref =
                    document.open_element_at(&mut xm_app, "ltx:XMRef", Some(ref_attrs), None)?;
                  document.close_element_at(&mut xm_ref)?;
                } else {
                  let mut xm_text =
                    document.open_element_at(&mut xm_app, "ltx:XMText", None, None)?;
                  let _ = xm_text.set_content("otherwise");
                  document.close_element_at(&mut xm_text)?;
                }
              }
              document.close_element_at(&mut xm_app)?;
              point.unlink_node();
              xm_dual.add_child(&mut point)?;
              document.close_element_at(&mut xm_dual)?;
            }
          }
        }
      }
    }
  );
});

/// Helper: get first child element node
fn first_child_element(node: &Node) -> Option<Node> { node.get_child_elements().into_iter().next() }

/// Perl: openMathFork (Base_XMath.pool.ltxml L780-786)
/// Creates a MathFork structure with two branches: main (semantic) and presentation.
/// Returns (mainfork_math_node, branch_node).
pub fn open_math_fork(document: &mut Document, equation: &mut Node) -> Result<(Node, Node)> {
  let mut fork = document.open_element_at(equation, "ltx:MathFork", None, None)?;
  let mut mainfork = document.open_element_at(&mut fork, "ltx:Math", None, None)?;
  let _xmath = document.open_element_at(&mut mainfork, "ltx:XMath", None, None)?;
  let branch = document.open_element_at(&mut fork, "ltx:MathBranch", None, None)?;
  Ok((mainfork, branch))
}

/// Perl: closeMathFork (Base_XMath.pool.ltxml L789-803)
/// Closes all elements of an ltx:MathFork.
pub fn close_math_fork(
  document: &mut Document,
  equation: &mut Node,
  mainfork: &mut Node,
  branch: &mut Node,
) -> Result<()> {
  // Synthesize tex= attribute on mainfork Math from MathBranch cell tex attributes.
  // Perl does this via MathWhatsit box accumulation + add_body_TeX afterClose,
  // but Rust doesn't have MathWhatsit. Instead, we compose the tex attribute here.
  if !mainfork.has_attribute("tex") {
    let mut tex_parts: Vec<String> = Vec::new();
    let tds = document.findnodes("descendant::ltx:td", Some(branch));
    for td in &tds {
      // Collect ALL content from the td: text nodes, Math[@tex], text[@class=ltx_markedasmath]
      // Perl's MathWhatsit captures reversion of all cell content in order.
      let mut cell_parts: Vec<String> = Vec::new();
      for child in td.get_child_nodes() {
        match child.get_type() {
          Some(NodeType::TextNode) => {
            let content = child.get_content();
            let trimmed = content.trim();
            if !trimmed.is_empty() {
              cell_parts.push(trimmed.to_string());
            }
          },
          Some(NodeType::ElementNode) => {
            let name = child.get_name();
            if name == "Math" || name == "math" {
              if let Some(tex) = child.get_attribute("tex") {
                cell_parts.push(tex);
              }
            } else if name == "text" {
              let class = child.get_attribute("class").unwrap_or_default();
              if class.contains("ltx_markedasmath") {
                let content = child.get_content();
                let content = content.trim();
                if !content.is_empty() {
                  cell_parts.push(format!("\\mbox{{{content}}}"));
                }
              } else {
                // Plain text element — include content
                let content = child.get_content();
                let content = content.trim();
                if !content.is_empty() {
                  cell_parts.push(content.to_string());
                }
              }
            }
            // Skip other element types (p wrappers etc) — descend into them
            else if name == "p" || name == "para" {
              let inner_maths = document.findnodes("ltx:Math[@tex]", Some(&child));
              for math in inner_maths {
                if let Some(tex) = math.get_attribute("tex") {
                  cell_parts.push(tex);
                }
              }
            }
          },
          _ => {},
        }
      }
      if !cell_parts.is_empty() {
        tex_parts.push(cell_parts.join(""));
      }
    }
    if !tex_parts.is_empty() {
      // Normalize: Perl's MathWhatsit extracts \displaystyle to the front.
      // Each cell's tex starts with "\displaystyle"; strip and prepend once.
      let has_displaystyle = tex_parts.iter().any(|t| t.starts_with("\\displaystyle"));
      let stripped: Vec<&str> = tex_parts
        .iter()
        .map(|t| t.strip_prefix("\\displaystyle").unwrap_or(t).trim_start())
        .collect();
      // Join cell parts with CS-aware spacing: if previous part ends with a letter
      // and next starts with a letter, insert a space. This preserves cell boundaries
      // and handles CS termination (\word followed by letter needs space).
      let mut body = String::new();
      for (i, part) in stripped.iter().enumerate() {
        if i > 0 && !body.is_empty() && !part.is_empty() {
          let prev_ends_letter = body.ends_with(|c: char| c.is_ascii_alphabetic());
          let next_starts_letter = part.starts_with(|c: char| c.is_ascii_alphabetic());
          if prev_ends_letter && next_starts_letter {
            body.push(' ');
          }
        }
        body.push_str(part);
      }
      let combined_tex = if has_displaystyle {
        // Add space after \displaystyle only if body starts with a letter
        // (TeX CS needs space termination before letters, not before operators)
        if body.starts_with(|c: char| c.is_ascii_alphabetic()) {
          format!("\\displaystyle {body}")
        } else {
          format!("\\displaystyle{body}")
        }
      } else {
        body
      };
      document.set_attribute(mainfork, "tex", &combined_tex)?;
    }
  }

  document.close_element_at(branch)?;
  // Close XMath (first child of mainfork)
  if let Some(mut xmath) = first_child_element(mainfork) {
    document.close_element_at(&mut xmath)?;
  }
  document.close_element_at(mainfork)?;
  // Fix broken XMRef idrefs: append_clone's id mapping can get out of sync
  // with the ids assigned by close_element_at (which triggers afterClose
  // callbacks that may re-generate ids via generate_id).
  if let Some(xmath) = first_child_element(mainfork) {
    fixup_xmref_idrefs(document, &xmath);
  }
  // Close the MathFork — find it defensively via xpath
  let mfs = document.findnodes("ltx:MathFork", Some(equation));
  if let Some(mut last_mf) = mfs.into_iter().last() {
    document.close_element_at(&mut last_mf)?;
  }
  // Check if branch came up empty (only 1 child = just the Math)
  let fork = branch.get_parent().unwrap();
  let branches: Vec<Node> = fork.get_child_nodes();
  if branches.len() == 1 {
    // Whoops, came up empty! Remove the fork (recursively unrecord any
    // xml:ids in the dropped subtree — the lone surviving child may be
    // a Math whose descendants were populated via append_clone).
    document.safe_unlink(fork);
  }
  Ok(())
}

/// Perl: addColumnToMathFork (Base_XMath.pool.ltxml L839-899)
/// Distributes content from an ltx:_Capture_ cell into both the presentation branch
/// (as ltx:td) AND the semantic main branch (cloned into XMath).
pub fn add_column_to_math_fork(
  document: &mut Document,
  mainfork: &mut Node,
  inbranch: &mut Node,
  cell: &mut Node,
) -> Result<()> {
  let mut td = document.open_element_at(inbranch, "ltx:td", None, None)?;
  // Copy align and colspan attributes
  if let Some(align) = cell.get_attribute("align") {
    document.set_attribute(&mut td, "align", &align)?;
  }
  if let Some(colspan) = cell.get_attribute("colspan") {
    document.set_attribute(&mut td, "colspan", &colspan)?;
  }
  // Remove the _Capture_ from the document
  cell.unlink_node();
  // Process each child of _Capture_
  let children: Vec<Node> = cell.get_child_nodes();
  let math_qname = arena::pin_static("ltx:Math");
  let text_qname = arena::pin_static("ltx:text");
  let p_qname = arena::pin_static("ltx:p");
  for node in children {
    let qname = document::get_node_qname(&node);
    if qname == math_qname {
      // Clone XMath children to the main branch
      if let Some(xmath) = first_child_element(&node) {
        let xmath_children: Vec<Node> = xmath.get_child_elements();
        if !xmath_children.is_empty() {
          state::assign_value("ID_SUFFIX", Stored::String(arena::pin_static(".mf")), None);
          if let Some(mut mainfork_xmath) = first_child_element(mainfork) {
            document.append_clone(&mut mainfork_xmath, xmath_children)?;
          }
          state::assign_value("ID_SUFFIX", Stored::None, None);
        }
      }
    } else if qname == text_qname || qname == p_qname {
      let text_content = node.get_content();
      if !text_content.is_empty() {
        if let Some(mut mainfork_xmath) = first_child_element(mainfork) {
          state::assign_value("ID_SUFFIX", Stored::String(arena::pin_static(".mf")), None);
          let mut txt = document.open_element_at(&mut mainfork_xmath, "ltx:XMText", None, None)?;
          document.append_clone(&mut txt, vec![node.clone()])?;
          document.close_element_at(&mut txt)?;
          state::assign_value("ID_SUFFIX", Stored::None, None);
        }
      }
    } else if node.get_type() == Some(libxml::tree::NodeType::TextNode) {
      let string = node.get_content();
      if !string.trim().is_empty() {
        if let Some(mut mainfork_xmath) = first_child_element(mainfork) {
          let mut txt = document.open_element_at(&mut mainfork_xmath, "ltx:XMText", None, None)?;
          let _ = txt.set_content(&string);
          document.close_element_at(&mut txt)?;
        }
      }
    } else if node.get_type() == Some(libxml::tree::NodeType::CommentNode) {
      // Skip comments
    } else {
      if let Some(mut mainfork_xmath) = first_child_element(mainfork) {
        state::assign_value("ID_SUFFIX", Stored::String(arena::pin_static(".mf")), None);
        let mut txt = document.open_element_at(&mut mainfork_xmath, "ltx:XMText", None, None)?;
        document.append_clone(&mut txt, vec![node.clone()])?;
        document.close_element_at(&mut txt)?;
        state::assign_value("ID_SUFFIX", Stored::None, None);
      }
    }
    // Move the original node to the td (presentation side)
    document.unrecord_node_ids(&node);
    document.append_tree(&mut td, vec![node])?;
  }
  document.close_element_at(&mut td)?;
  Ok(())
}

/// Collect all xml:id values from a subtree by DOM walking.
fn collect_xml_ids(node: &Node, ids: &mut Vec<String>) {
  // Try all possible attribute forms for xml:id
  let id = node
    .get_attribute("xml:id")
    .or_else(|| node.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace"))
    .or_else(|| {
      // Fallback: scan attributes for anything named "id" or "xml:id"
      node
        .get_attributes()
        .into_iter()
        .find(|(k, _)| k == "id" || k == "xml:id")
        .map(|(_, v)| v)
    });
  if let Some(id) = id {
    ids.push(id);
  }
  for child in node.get_child_nodes() {
    if child.get_type() == Some(libxml::tree::NodeType::ElementNode) {
      collect_xml_ids(&child, ids);
    }
  }
}

/// Collect all XMRef nodes from a subtree by DOM walking.
fn collect_xmrefs(node: &Node, refs: &mut Vec<Node>) {
  if node.get_name() == "XMRef" && node.has_attribute("idref") {
    refs.push(node.clone());
  }
  for child in node.get_child_nodes() {
    if child.get_type() == Some(libxml::tree::NodeType::ElementNode) {
      collect_xmrefs(&child, refs);
    }
  }
}

/// Fix broken XMRef idrefs in a subtree after append_clone.
/// append_clone's id_map and record_id_with_node can produce inconsistent
/// xml:id/idref pairs. This function scans the subtree for all xml:ids,
/// then updates XMRef idrefs to point to actual existing ids.
fn fixup_xmref_idrefs(_document: &mut Document, root: &Node) {
  use std::collections::HashMap;
  // Collect all xml:ids in the subtree, keyed by their "base" (without suffix variants)
  // Collect xml:ids from the subtree by walking the DOM directly
  // (XPath namespace handling may miss xml:id attributes)
  let mut all_ids: Vec<String> = Vec::new();
  collect_xml_ids(root, &mut all_ids);
  if all_ids.is_empty() {
    return;
  }
  // Build a reverse lookup: for each id, map its base forms to the actual id
  // E.g., "Ch0.E16.m2.1" could be the actual id for what was originally "Ch0.E16.m1.1"
  // We use a simpler approach: map by the TRAILING number(s) after the last dot
  let mut id_by_suffix: HashMap<String, String> = HashMap::new();
  for id in &all_ids {
    // Extract suffix like ".1", ".2", ".3" etc.
    if let Some(dot_pos) = id.rfind('.') {
      let suffix = &id[dot_pos..];
      id_by_suffix.insert(suffix.to_string(), id.clone());
    }
    id_by_suffix.insert(id.clone(), id.clone()); // exact match
  }
  // Find all XMRef nodes and fix their idrefs
  let mut xmrefs: Vec<Node> = Vec::new();
  collect_xmrefs(root, &mut xmrefs);
  let fixed_count = xmrefs.len();
  for mut xmref in xmrefs {
    if let Some(idref) = xmref.get_attribute("idref") {
      // Check if idref points to an existing id in the subtree
      if all_ids.contains(&idref) {
        continue;
      }
      // Try to find the matching id by suffix
      if let Some(dot_pos) = idref.rfind('.') {
        let suffix = &idref[dot_pos..];
        // Try suffix match: e.g. ".1" in idref matches ".1" in actual id
        if let Some(actual_id) = id_by_suffix.get(suffix) {
          xmref.set_attribute("idref", actual_id).ok();
        }
      }
    }
  }
  let _ = fixed_count;
}

/// Perl: equationgroupJoinCols (Base_XMath.pool.ltxml L970-980)
/// Groups every $ncols columns into a MathFork structure within an equation.
pub fn equationgroup_join_cols(
  document: &mut Document,
  ncols: usize,
  equation: &mut Node,
) -> Result<()> {
  let mut col = 0usize;
  let mut mainfork: Option<Node> = None;
  let mut branch: Option<Node> = None;
  let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(equation));
  for mut cell in cells {
    let qname_sym = document::get_node_qname(&cell);
    if !arena::with(qname_sym, |s| s.ends_with("_Capture_")) {
      continue;
    }
    if col.is_multiple_of(ncols) {
      if let (Some(ref mut mf), Some(ref mut br)) = (&mut mainfork, &mut branch) {
        close_math_fork(document, equation, mf, br)?;
      }
      let (mf, br) = open_math_fork(document, equation)?;
      mainfork = Some(mf);
      branch = Some(br);
    }
    if let (Some(ref mut mf), Some(ref mut br)) = (&mut mainfork, &mut branch) {
      add_column_to_math_fork(document, mf, br, &mut cell)?;
    }
    col += 1;
  }
  if let (Some(ref mut mf), Some(ref mut br)) = (&mut mainfork, &mut branch) {
    close_math_fork(document, equation, mf, br)?;
  }
  Ok(())
}

/// Perl: equationgroupJoinRows (Base_XMath.pool.ltxml L926-963)
/// Combines multiple row equations into a single semantic equation with a MathFork structure.
pub fn equationgroup_join_rows(
  document: &mut Document,
  equationgroup: &mut Node,
  mut equations: Vec<Node>,
) -> Result<()> {
  if equations.is_empty() {
    return Ok(());
  }
  // Create new equation at the position of the first input equation
  let mut equation = document.open_element_at(equationgroup, "ltx:equation", None, None)?;
  // Move it before the first input equation
  equations[0].add_prev_sibling(&mut equation).ok();
  // Consolidate labels, id, refnum, tags from the input equations
  let mut labels: Option<String> = None;
  let mut id: Option<String> = None;
  let mut tags: Option<Node> = None;
  for eq in equations.iter() {
    if let Some(l) = eq.get_attribute("labels") {
      labels = Some(match labels {
        Some(existing) => s!("{existing} {l}"),
        None => l,
      });
    }
    if let Some(eq_id) = eq
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .or_else(|| eq.get_attribute("xml:id"))
    {
      id = Some(eq_id);
    }
    let found_tags = document.findnodes("ltx:tags", Some(eq));
    if let Some(t) = found_tags.into_iter().last() {
      tags = Some(t);
    }
  }
  if let Some(ref id_str) = id {
    document.unrecord_id(id_str);
  }
  if let Some(ref l) = labels {
    document.set_attribute(&mut equation, "labels", l)?;
  }
  if let Some(ref id_str) = id {
    document.set_attribute(&mut equation, "xml:id", id_str)?;
  }
  if let Some(mut t) = tags {
    t.unlink_node();
    equation.add_child(&mut t).ok();
  }
  // Pre-advance the ID counter to skip over cell Math IDs.
  // For multi-row eqnarrays where different rows have different equation IDs,
  // only count cells from the equation that provides the target ID.
  // This prevents inflation from rows that belong to different equation contexts.
  let has_different_ids = equations.len() > 1 && {
    let ids: Vec<_> = equations
      .iter()
      .filter_map(|eq| {
        eq.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
          .or_else(|| eq.get_attribute("xml:id"))
      })
      .collect();
    ids.len() > 1 || (ids.len() == 1 && equations.len() > 1)
  };
  let mut cell_count = 0;
  for eq in equations.iter() {
    if has_different_ids {
      // Multi-row with different IDs: only count cells with Math from the target equation
      let eq_id = eq
        .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
        .or_else(|| eq.get_attribute("xml:id"));
      if eq_id != id {
        continue;
      }
      // Count only cells that contain Math elements (empty cells don't consume IDs)
      let captures = document.findnodes("ltx:_Capture_", Some(eq));
      for cap in &captures {
        if !document.findnodes("ltx:Math", Some(cap)).is_empty() {
          cell_count += 1;
        }
      }
    } else {
      // Single-equation: count all _Capture_ cells (including empty ones)
      let captures = document.findnodes("ltx:_Capture_", Some(eq));
      cell_count += captures.len();
    }
  }
  if cell_count > 0 {
    let ctrkey = s!("_ID_counter_m_");
    let current = equation.get_attribute(&ctrkey).unwrap_or_else(|| s!("0"));
    let new_val = current.parse::<u32>().unwrap_or(0) + cell_count as u32;
    equation.set_attribute(&ctrkey, &new_val.to_string())?;
  }
  // Create MathFork
  let (mut mainfork, mut branch_node) = open_math_fork(document, &mut equation)?;
  for eq in equations {
    let mut eq = eq;
    // D3b: unrecord eq's own xml:id before detaching so idstore doesn't
    // retain a dangling entry pointing to a soon-to-be-dropped node.
    // (Descendant ids remain recorded; append_clone inside
    // add_column_to_math_fork relies on modify_id suffixing for clones
    // of cell children.)
    if let Some(eq_id) = eq
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .or_else(|| eq.get_attribute("xml:id"))
    {
      document.unrecord_id(&eq_id);
    }
    eq.unlink_node();
    let mut tr = document.open_element_at(&mut branch_node, "ltx:tr", None, None)?;
    // Note: Perl also checks for lefteqn class on first _Capture_ to add ltx_eqn_lefteqn
    // to <tr>, but in practice the class is on a child td (via \multicolumn), not on
    // _Capture_ itself, so the condition never fires in Perl output.
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&eq));
    for mut cell in cells {
      add_column_to_math_fork(document, &mut mainfork, &mut tr, &mut cell)?;
    }
    document.close_element_at(&mut tr)?;
  }
  close_math_fork(document, &mut equation, &mut mainfork, &mut branch_node)?;
  document.close_element_at(&mut equation)?;
  Ok(())
}

/// Perl: addMeaningRec — recursively add meaning to XMTok elements with UNKNOWN role
pub fn add_meaning_rec(document: &mut Document, node: Node, meaning: &str) -> Result<()> {
  if node.get_type() != Some(NodeType::ElementNode) {
    return Ok(());
  }
  let qname = document::get_node_qname(&node);
  if qname == arena::pin_static("ltx:XMArg") {
    // DON'T cross through into arguments
  } else if qname == arena::pin_static("ltx:XMTok") {
    let role = node.get_attribute("role").unwrap_or_default();
    if (role.is_empty() || role == "UNKNOWN") && !node.has_attribute("meaning") {
      document.set_attribute(&mut node.clone(), "meaning", meaning)?;
    }
  } else {
    for child in node.get_child_nodes() {
      add_meaning_rec(document, child, meaning)?;
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use libxml::parser::Parser as XmlParser;

  // See wisdom_libxml_node_doc_lifetime.md — keep the Document alive.
  fn parse(xml: &str) -> libxml::tree::Document {
    XmlParser::default().parse_string(xml).expect("parse")
  }

  #[test]
  fn first_child_element_returns_first_elem() {
    let doc = parse(r#"<parent>text<a/><b/></parent>"#);
    let root = doc.get_root_element().unwrap();
    let first = first_child_element(&root).expect("has a child");
    assert_eq!(first.get_name(), "a");
  }

  #[test]
  fn first_child_element_none_for_leaf() {
    let doc = parse(r#"<leaf/>"#);
    let root = doc.get_root_element().unwrap();
    assert!(first_child_element(&root).is_none());
  }

  #[test]
  fn first_child_element_skips_text_nodes() {
    // get_child_elements already filters text — so a pure-text leading child
    // is not returned.
    let doc = parse(r#"<p>   <i/></p>"#);
    let root = doc.get_root_element().unwrap();
    let first = first_child_element(&root).expect("has element child");
    assert_eq!(first.get_name(), "i");
  }

  #[test]
  fn collect_xml_ids_walks_recursively() {
    let doc = parse(
      r#"<root xml:id="r">
           <child xml:id="c1"/>
           <inner>
             <deep xml:id="d1"/>
           </inner>
           <child xml:id="c2"/>
         </root>"#,
    );
    let root = doc.get_root_element().unwrap();
    let mut ids = Vec::new();
    collect_xml_ids(&root, &mut ids);
    // Order is DOM order: root first, then depth-first children.
    assert_eq!(ids, vec!["r", "c1", "d1", "c2"]);
  }

  #[test]
  fn collect_xml_ids_empty_for_no_ids() {
    let doc = parse(r#"<root><a/><b><c/></b></root>"#);
    let root = doc.get_root_element().unwrap();
    let mut ids = Vec::new();
    collect_xml_ids(&root, &mut ids);
    assert!(ids.is_empty());
  }

  #[test]
  fn collect_xmrefs_finds_by_element_name_and_idref() {
    let doc = parse(
      r#"<root>
           <XMRef idref="a1"/>
           <inner>
             <XMRef idref="b1"/>
             <XMRef/>
             <other idref="c1"/>
           </inner>
         </root>"#,
    );
    let root = doc.get_root_element().unwrap();
    let mut refs = Vec::new();
    collect_xmrefs(&root, &mut refs);
    // Must have idref attribute AND be named XMRef.
    assert_eq!(refs.len(), 2);
    let idrefs: Vec<String> = refs
      .iter()
      .map(|r| r.get_attribute("idref").unwrap())
      .collect();
    assert_eq!(idrefs, vec!["a1", "b1"]);
  }

  #[test]
  fn collect_xmrefs_empty_for_no_matches() {
    let doc = parse(r#"<root><child/></root>"#);
    let root = doc.get_root_element().unwrap();
    let mut refs = Vec::new();
    collect_xmrefs(&root, &mut refs);
    assert!(refs.is_empty());
  }
}
