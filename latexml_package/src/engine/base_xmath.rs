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
    add_meaning_rec(document, node, meaning)?;
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
  //   afterDigest => sub {
  //     $_[1]->setFont($_[1]->getArg(2)->getFont);
  //     XMath_copy_keyvals(@_); });

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
  //     (IsEmpty($sup)
  //       ? Revert($base)
  //       : (($bump ? (T_BEGIN, Revert($base), T_END) : Revert($base)), T_SUPER,
  // revertScript($sup))); },   properties => sub {
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
  //     (IsEmpty($sub)
  //       ? Revert($base)
  //       : (($bump ? (T_BEGIN, Revert($base), T_END) : Revert($base)), T_SUB,
  // revertScript($sub))); },   properties => sub {
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
  DefMath!("\\lx@InvisibleTimes", None, "\u{2062}", reversion => "", name => "",
    meaning => "times", role => "MULOP");
  DefMath!("\\lx@InvisibleComma", None, "\u{2063}", reversion => "", name => "", role => "PUNCT");
  DefMath!("\\lx@InvisiblePlus", None, "\u{2064}", reversion => "", name => "", meaning => "plus", role => "ADDOP");
  // Perl: beforeDigest => sub { $_[0]->enterHorizontal; }
  DefConstructor!("\\lx@kludged{}",
    "?#isMath(<ltx:XMWrap rule='kludge'>#1</ltx:XMWrap>)(#1)",
    // TODO: enter_horizontal causes io_test failure — investigate
    // before_digest => { enter_horizontal(); },
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

  DefKeyVal!("XMath", "reversion", "UndigestedDefKey");
  DefKeyVal!("XMath", "content_reversion", "UndigestedDefKey");
  DefKeyVal!("XMath", "presentation_reversion", "UndigestedDefKey");

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

  //======================================================================

  // We OUGHT to be able to do this using \llap,\rlap,\hss...
  DefMacro!(
    "\\lx@tweaked{}{}",
    r"\ifmmode\lx@math@tweaked{#1}{#2}\else\lx@text@tweaked{#1}{#2}\fi"
  );
  // TODO:
  // DefConstructor!("\\lx@math@tweaked RequiredKeyVals {}", "<ltx:XMWrap
  // $XMath_attributes>#2</ltx:XMWrap>",   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my ($kv,      $body)    = $whatsit->getArgs;
  //     XMath_copy_keyvals($stomach, $whatsit);
  //     $whatsit->setFont($body->getFont);
  //     return; },
  // reversion => "#2");

  // DefConstructor('\lx@text@tweaked RequiredKeyVals {}',
  //   "<ltx:text _noautoclose='1' %&GetKeyVals(#1)>#2</ltx:text>",
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my ($kv,      $body)    = $whatsit->getArgs;
  //     $whatsit->setProperties($kv->getPairs); });

  DefConstructor!(T_CS!("\\lx@ldots"), None,
  "?#isMath(<ltx:XMTok name='ldots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})",
  sizer      => "\u{2026}",
  reversion  => "\\ldots",
  properties => {
    if lookup_bool("IN_MATH") {
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
  // DefKeyVal('lx@GEN', 'style', 'UndigestedKey');

  // DefPrimitive('\lx@gen@matrix@bindings RequiredKeyVals:lx@GEN', sub {
  //     my ($stomach, $kv) = @_;
  //     $stomach->bgroup;
  //     my $style = $kv->getValue('style')               || T_CS('\textstyle');
  //     my $align = ToString($kv->getValue('alignment')) || 'c';
  //     # We really should be using ReadAlignmentTemplate (LaTeXML::Core::Alignment)
  //     # but we'd have to convert it to a repeating spec somehow.
  //     my @colspec = (before => Tokens(($align =~ /^(?:c|r)/ ? (T_CS('\hfil')) : ()), $style),
  //       after => Tokens(($align =~ /^(?:c|l)/ ? (T_CS('\hfil')) : ())));
  //     my $ncols      = ToString($kv->getValue('ncolumns'));
  //     my %attributes = ();
  //     foreach my $key (qw(rowsep)) {    # Probably more?
  //       if (my $value = $kv->getValue($key)) {
  //         $attributes{$key} = $value; } }
  //     alignmentBindings(LaTeXML::Core::Alignment::Template->new(
  //         ($ncols ? (columns => [map { { @colspec } } 1 .. $ncols])
  //           : (repeated => [{@colspec}]))),
  //       'math',
  //       (keys %attributes ? (attributes => {%attributes}) : ()));    # });
  //     Let("\\\\", '\lx@alignment@newline');
  // });

  DefPrimitive!("\\lx@end@gen@matrix", {
    egroup()?;
  });

  DefMacro!(
    "\\lx@gen@plain@matrix{}{}",
    "\\lx@gen@matrix@bindings{#1}\
      \\lx@gen@plain@matrix@{#1}{\\lx@begin@alignment#2\\lx@end@alignment}\\lx@end@gen@matrix"
  );

  // # The delimiters on a matrix are presumably just for notation or readability (not an operator);
  // # the array data itself is the matrix.
  // DefConstructor('\lx@gen@plain@matrix@ RequiredKeyVals:lx@GEN {}',
  //   "?#needXMDual("
  //     . "<ltx:XMDual>"
  //     . "?#delimitermeaning(<ltx:XMApp><ltx:XMTok meaning='#delimitermeaning'/>)()"
  //     . "?#datameaning(<ltx:XMApp><ltx:XMTok meaning='#datameaning'/>)()"
  //     . "<ltx:XMRef _xmkey='#xmkey'/>"
  //     . "?#delimitermeaning(</ltx:XMApp>)()"
  //     . "?#datameaning(</ltx:XMApp>)()"
  //     . "<ltx:XMWrap>#left<ltx:XMArg _xmkey='#xmkey'>#2</ltx:XMArg>#right</ltx:XMWrap>"
  //     . "</ltx:XMDual>"
  //     . ")("
  //     . "#2"
  //     . ")",
  //   properties => sub { %{ $_[1]->getKeyVals }; },
  //   reversion  => sub {
  //     my ($whatsit, $kv, $body) = @_;
  //     my $name      = ToString($kv->getValue('name'));
  //     my $alignment = $whatsit->getProperty('alignment');
  // ##    (T_CS('\\' . $name), T_BEGIN, Revert($body), T_END); },
  // ##    (T_CS('\\' . $name), T_BEGIN, Revert($alignment), T_END); },
  //     (T_CS('\\' . $name), T_BEGIN, $alignment->revert, T_END); },

  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $kv = $whatsit->getArg(1);
  //     if ($kv->getValue('datameaning') || $kv->getValue('delimitermeaning')) {
  //       $whatsit->setProperties(
  //         needXMDual => 1,
  //         xmkey      => LaTeXML::Package::getXMArgID()); }
  //     $whatsit->setProperties(alignment => LookupValue('Alignment'));
  //     return; });

  //----------------------------------------------------------------------
  // Cases: Generalized
  // keys are
  //  name  : the name of the command (for reversion)
  //  meaning: the (presumed) meaning of the construct
  //  style : \textstyle or \displaystyle
  //  conditionmode : mode of 2nd column, text or math
  //  left  : TeX code for left of cases
  //  right  : TeX code for right

  // DefConstructorI('\lx@cases@condition', undef,
  //   "<ltx:XMText>#body</ltx:XMText>",
  //   alias => '', beforeDigest => sub { $_[0]->beginMode('text'); }, captureBody => 1);
  // DefConstructorI('\lx@cases@end@condition', undef, "", alias => '',
  //   beforeDigest => sub { $_[0]->endMode('text'); });

  // DefPrimitive('\lx@gen@cases@bindings RequiredKeyVals:lx@GEN', sub {
  //     my ($stomach, $kv) = @_;
  //     $stomach->bgroup;
  //     my $style = $kv->getValue('style') || T_CS('\textstyle');
  //     $style = T_CS($style) unless ref $style;
  //     my @mode = (ToString($kv->getValue('conditionmode')) eq 'text'
  //       ? (T_MATH) : ());
  //     my $condtext = ToString($kv->getValue('conditionmode')) eq 'text';
  //     alignmentBindings(LaTeXML::Core::Alignment::Template->new(
  //         columns => [
  //           { before => Tokens($style), after => Tokens(T_CS('\hfil')) },
  //           { before => Tokens($style,
  //               ($condtext ? (T_CS('\lx@cases@condition')) : ())),
  //             after => Tokens(T_CS('\lx@column@trimright'),
  //               ($condtext ? (T_CS('\lx@cases@end@condition')) : ()),
  //               T_CS('\hfil')) }]),
  //       'math');
  //     Let("\\\\", '\lx@alignment@newline');
  //     DefMacro('\lx@alignment@row@before', '');    # Don't inherit counter stepping from
  // containing environments     DefMacro('\lx@alignment@row@after',  '');
  // });

  DefMacro!(
    "\\lx@gen@plain@cases{}{}",
    "\\lx@gen@cases@bindings{#1}\
      \\lx@gen@plain@cases@{#1}{\\lx@begin@alignment#2\\lx@end@alignment}
      \\lx@end@gen@cases"
  );
  DefPrimitive!("\\lx@end@gen@cases", {
    egroup()?;
  });

  // The logical structure for cases extracts the columns of the alignment
  // to give alternating value,condition (an empty condition is replaced by "otherwise" !?!?!)
  // DefConstructor('\lx@gen@plain@cases@ RequiredKeyVals:lx@GEN {}',
  //   '<ltx:XMWrap>#left#2#right</ltx:XMWrap>',
  //   properties     => sub { %{ $_[1]->getKeyVals }; },
  //   afterConstruct => sub {
  //     my ($document) = @_;
  //     if (my $point = $document->getElement->lastChild) {
  //       # Get the sequence of alternating (case, condition).
  //       # Expecting ltx:XMArray/ltx:XMRow/ltx:XMCell [should have /ltx:XMArg, but could be
  // empty!!!]       my @cells = $document->findnodes('ltx:XMArray/ltx:XMRow/ltx:XMCell', $point);
  //       my @stuff = map { ($_->hasChildNodes ? createXMRefs($document, element_nodes($_))
  //           : ['ltx:XMText', {}, 'otherwise']) } @cells;
  //       $document->replaceTree(['ltx:XMDual', {},
  //           ['ltx:XMApp', {}, ['ltx:XMTok', { meaning => 'cases' }], @stuff],
  //           $point],
  //         $point); } },
  //   reversion => sub {
  //     my ($whatsit, $kv, $body) = @_;
  //     my $name = $kv->getValue('name');
  //     (T_CS('\cases'), T_BEGIN, Revert($body), T_END); });

  // TODO: Continue MathFork and equationgroup
});

pub fn add_meaning_rec(_document: &mut Document, _node: Node, _meaning: String) -> Result<()> {
  // if ($node->nodeType == XML_ELEMENT_NODE) {
  //   my $qname = $document->getModel->getNodeQName($node);
  //   if    ($qname eq 'ltx:XMArg') { }              # DONT cross through into arguments!
  //   elsif ($qname eq 'ltx:XMTok') {
  //     if ((($node->getAttribute('role') || 'UNKNOWN') eq 'UNKNOWN')
  //       && !$node->getAttribute('meaning')) {
  //       $document->setAttribute($node, meaning => $meaning); } }
  //   else {
  //     foreach my $c ($node->childNodes) {
  // addMeaningRec($document, $c, $meaning); } } }
  todo!()
}
