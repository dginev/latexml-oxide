use crate::package::*;
use std::collections::hash_map::Entry;
use crate::common::xml::XML_NS;

LoadDefinitions!({
  // ======================================================================
  //  Support for constructing mathematical expressions

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

  // # Build an ltx:XMApp, application of function/operator to arguments
  // # first piece of (TeX) argument is expected to be the operator
  // # Usually used on content side, but at least the arguments should be properly encapsulated:
  // # They should build individual subtrees; use ltx::XMArg, ltx:XMWrap ... if needed
  // DefConstructor('\lx@apply OptionalKeyVals:XMath {}{}',
  //   "<ltx:XMApp $XMath_attributes>#2#3</ltx:XMApp>",
  //   reversion   => '#2#3',
  //   afterDigest => sub { XMath_copy_keyvals(@_); });

  // # Build an ltx:XMTok, a mathematical symbol, with given attributes
  // # the argument should create text to be the content of the token.
  // DefConstructor('\lx@symbol OptionalKeyVals:XMath {}',
  //   "<ltx:XMTok $XMath_attributes>#2</ltx:XMTok>",
  //   reversion   => '#2',
  //   afterDigest => sub { XMath_copy_keyvals(@_); });

  // # Wrap the contents in an ltx:XMWrap, to stand as a single subtree & providing attributes
  // # The ltx:XMWrap may be collapsed, later, by parsing
  // DefConstructor('\lx@wrap OptionalKeyVals:XMath {}',
  //   "<ltx:XMWrap $XMath_attributes>#2</ltx:XMWrap>",
  //   reversion   => '#2',
  //   afterDigest => sub { XMath_copy_keyvals(@_); });

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

  // # These two accept key operator_meaning, operator_omcd to give a meaning to the sub/superscript
  // # NOTE (BUG): We SHOULD nest paired sub/superscripts, but avoid conflicting double scripts
  // # To do that we need to sniff at the base, whether it already contains scripts.
  // # However, IsScript isn't quite sufficient if the scripts are hidden within Whatsits, duals,
  // etc. # Currently, LaTeXML manages to deal with the double scripts anyway;
  // # The reversion ALWAYS wraps the base (which will render non-optimally in images but avoid
  // Errors) DefConstructor('\lx@superscript OptionalKeyVals:XMath {} InScriptStyle',
  //   "<ltx:XMApp $XMath_attributes>"
  //     . "<ltx:XMTok role='SUPERSCRIPTOP' meaning='#operator_meaning' omcd='#operator_omcd'
  // scriptpos='#scriptpos'/>"     . "<ltx:XMArg>#2</ltx:XMArg>"
  //     . "<ltx:XMArg rule='Superscript'>#3</ltx:XMArg>"
  //     . "</ltx:XMApp>",
  //   afterDigest => sub { XMath_copy_keyvals(@_); },
  //   reversion   => sub {
  //     my ($whatsit, $kv, $base, $sup) = @_;
  //     my $bump = $whatsit->getProperty('bump');
  //     $bump = 1;    # For now: ALWAYS {} wrap base in the reversion!
  //     ($sup && $sup->unlist
  //       ? (($bump ? (T_BEGIN, Revert($base), T_END) : Revert($base)), T_SUPER,
  // revertScript($sup))       : Revert($base)); },
  //   properties => sub {
  //     my ($stomach, $kv, $base, $script) = @_;
  //     my $basetype = IsScript($base);
  //     my $bump     = ($basetype && ($$basetype[1] eq 'SUPERSCRIPT') ? 1 : 0);
  //     (scriptpos => "post" . ($_[0]->getScriptLevel + $bump),
  //       bump => $bump); },
  //   sizer => sub { scriptSizer($_[0]->getArg(3), $_[0]->getArg(2), undef, 'SUPERSCRIPT', 'post');
  // });

  // DefConstructor('\lx@subscript OptionalKeyVals:XMath {} InScriptStyle',
  //   "<ltx:XMApp $XMath_attributes>"
  //     . "<ltx:XMTok role='SUBSCRIPTOP' meaning='#operator_meaning' omcd='#operator_omcd'
  // scriptpos='#scriptpos'/>"     . "<ltx:XMArg>#2</ltx:XMArg>"
  //     . "<ltx:XMArg rule='Subscript'>#3</ltx:XMArg>"
  //     . "</ltx:XMApp>",
  //   afterDigest => sub { XMath_copy_keyvals(@_); },
  //   reversion   => sub {
  //     my ($whatsit, $kv, $base, $sub) = @_;
  //     my $bump = $whatsit->getProperty('bump');
  //     $bump = 1;    # For now: ALWAYS {} wrap base in the reversion!
  //     ($sub && $sub->unlist
  // ###      ? (T_BEGIN, Revert($base), T_END, T_SUB, revertScript($sub))
  //       ? (($bump ? (T_BEGIN, Revert($base), T_END) : Revert($base)), T_SUB, revertScript($sub))
  //       : Revert($base)); },
  //   properties => sub {
  //     my ($stomach, $kv, $base, $script) = @_;
  //     my $basetype = IsScript($base);
  //     my $bump     = ($basetype && ($$basetype[1] eq 'SUBSCRIPT') ? 1 : 0);
  //     (scriptpos => "post" . ($_[0]->getScriptLevel + $bump),
  //       bump => $bump); },
  //   sizer => sub { scriptSizer($_[0]->getArg(3), $_[0]->getArg(2), undef, 'SUBSCRIPT', 'post');
  // });

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
  DefMacro!("\\lx@power{}{}", "\\lx@superscript[operator_meaning=power]{#1}{#2}");
  // Superscript meaning functional (or applicative) power; iterated function/operator application
  DefMacro!("\\lx@functionalpower{}{}",
    "\\lx@superscript[operator_meaning=functional-power]{#1}{#2}");

  // These to be used in presentation side
  DefMath!("\\lx@ApplyFunction", None, "\u{2061}", reversion => "", name => "", role =>"APPLYOP");
  DefMath!("\\lx@InvisibleTimes", None, "\u{2062}", reversion => "", name => "",
    meaning => "times", role => "MULOP");
  DefMath!("\\lx@InvisibleComma", None, "\u{2063}", reversion => "", name => "", role => "PUNCT");
  DefMath!("\\lx@InvisiblePlus", None, "\u{2064}", reversion => "", name => "", meaning => "plus", role => "ADDOP");
  DefConstructor!("\\lx@kludged{}",
    "?#isMath(<ltx:XMWrap rule='kludge'>#1</ltx:XMWrap>)(#1)",
    reversion => "#1");
  // TODO:
  // DefConstructor!("\\lx@padded[MuDimension]{MuDimension}{}",
  //   "#3",
  //   afterConstruct => sub {
  //     my ($document, $whatsit) = @_;
  //     my $node = $document->getLastChildElement($document->getNode);
  //     if ($document->getNodeQName($node) eq 'ltx:XMDual') {
  //       my (@ch) = $node->childNodes;
  //       $node = $ch[1]; }
  //     if (my $lpadding = $whatsit->getArg(1)) {
  //       $document->setAttribute($node, lpadding => $lpadding); }
  //     if (my $rpadding = $whatsit->getArg(2)) {
  //       $document->setAttribute($node, rpadding => $rpadding); } },
  //   reversion => '#3');

  // #======================================================================
  // # Building XMDuals for Mathematical Parallel markup
  // # Used when the content and presentation forms have different structure.

  DefKeyVal!("XMath", "reversion",              "UndigestedDefKey");
  DefKeyVal!("XMath", "content_reversion",      "UndigestedDefKey");
  DefKeyVal!("XMath", "presentation_reversion", "UndigestedDefKey");

  DefConstructor!("\\lx@dual OptionalKeyVals:XMath {}{}",
  "<ltx:XMDual role='#role' name='#name' meaning='#meaning' omcd='#omcd' width='#width' height='#height' xoffset='#xoffset' yoffset='#yoffset' lpadding='#lpadding' rpadding='#rpadding'>#2<ltx:XMWrap>#3</ltx:XMWrap></ltx:XMDual>",
  before_digest => {
    push_value("PENDING_DUAL_XMARGS", Stored::HashStored(HashMap::default()))
  },
  after_digest => sub[whatsit] {
    // let kv     = whatsit.get_arg(1);
    if let Some(Stored::HashStored(xmargs)) = pop_value("PENDING_DUAL_XMARGS")? { // Really SHOULD be a hash
      whatsit.set_properties(xmargs);  // Hopefully no name class with XM<digits>
    }
    // TODO:
    // whatsit.set_properties($kv->getPairs) if $kv;

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
            todo!()
            // Tokens!(T_CS!("\\lx@dual"))
            //               : ($r eq 'dual'
            //                 ? Tokens(T_CS('\lx@dual'), I_keyvals($kvs),
            //                   T_BEGIN, ($cr || Revert($c)), T_END,
            //                   T_BEGIN, ($pr || Revert($p)), T_END)
         },
          _other => {
            todo!()
            //                 : (($LaTeXML::DUAL_BRANCH || '') eq 'presentation'    # Context dependent reversion
            //                   ? $pr || Revert($p)
            //                   : $cr || Revert($c))))); }
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
          pending.insert(xmid, arg.into());
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
      document.set_attribute(&mut r, "idref", ids.get(&r_xmkey).unwrap())?;
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

  // #----------------------------------------------------------------------
  // # This group should be renamed to \lx@somethings and deprecated
  // # NOTE: work through this systematically!
  DefMacro!("\\FCN{}", r"\lx@wrap[role=FUNCTION]{#1}");
  DefMacro!("\\ROLE{}{}", r"\lx@wrap[role={#1}]{#2}");
  DefMacro!("\\@SYMBOL{}", r"\lx@wrap[role=ID]{#1}");
  DefMacro!("\\@CSYMBOL{}", r"\lx@symbol[meaning={#1}]{}");
  DefMacro!("\\@APPLY{}", r"\lx@apply[]{#1}{}"); // Sorta broken?
  DefMacro!("\\@MAYBEAPPLY{}{}", r"\ifx.#2.#1\else\lx@apply{#1}{#2}\fi");
  DefMacro!("\\@WRAP{}", r"\lx@wrap[]{#1}");
  DefMacro!("\\@TOKEN{}", r"\lx@symbol[name={#1}]{}");
  DefMacro!(
    "\\@SUPERSCRIPT{}{}",
    r"\ifx.#2.#1\else\lx@superscript[]{#1}{#2}\fi"
  );
  DefMacro!(
    "\\@SUBSCRIPT{}{}",
    r"\ifx.#2.#1\else\lx@subscript[]{#1}{#2}\fi"
  );
  Let!("\\@PADDED", "\\lx@padded");
  Let!("\\DUAL", "\\lx@dual");
  Let!("\\@XMArg", "\\lx@xmarg");
  Let!("\\@XMRef", "\\lx@xmref");
  Let!("\\@APPLYFUNCTION", "\\lx@ApplyFunction");
  Let!("\\@INVISIBLETIMES", "\\lx@InvisibleTimes");
  Let!("\\@INVISIBLECOMMA", "\\lx@InvisibleComma");
  Let!("\\@INVISIBLEPLUS", "\\lx@InvisiblePlus");

  // End of stuff to be deprecated.
  //----------------------------------------------------------------------

});