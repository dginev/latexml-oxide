use crate::package::*;
use rtx_core::list::List;
use rtx_core::tbox::Tbox;
use rtx_core::TexMode;

//**********************************************************************
// Primitives
// See The TeXBook, Chapter 24, Summary of Vertical Mode
//  and Chapter 25, Summary of Horizontal Mode.
// Parsing of basic types (pp.268--271) is (mostly) handled in Gullet.pm
//**********************************************************************

LoadDefinitions!(state, {
  //======================================================================
  // Remaining Mode independent primitives in Ch.24, pp.279-280
  // \relax was done as expandable (isn't that right?)
  // }
  // Note, we don't bother making sure begingroup is ended by endgroup.

  // These define the handler for { } (or anything of catcode BEGIN, END)

  // These are actually TeX primitives, but we treat them as a Whatsit so they
  // remain in the constructed tree.
  DefPrimitiveI!(
    "{",
    primitivesub!(stomach, _args, state, {
      stomach.bgroup(state);
      let open = Tbox::new(String::new(), None, None, Tokens!(T_BEGIN!()), HashMap::new(), state);
      let mode = if LookupBool!("IN_MATH") {
        Some(TexMode::Math)
      } else {
        Some(TexMode::Text)
      };
      let body = stomach.digest_next_body(None, state)?;
      let mut boxes = vec![Digested::TBox(Rc::new(open))];
      boxes.extend(body);
      let return_list = List { boxes, mode, font: None };

      return_list.into()
    })
  );

  DefPrimitiveI!(
    "}",
    primitivesub!(stomach, _args, state, {
      let f = LookupFont!();
      stomach.egroup(state)?;
      let return_box = Tbox::new(String::new(), f, None, Tokens!(T_END!()), HashMap::new(), state);
      return_box.into()
    })
  );

  // // These are for those screwy cases where you need to create a group like box,
  // // more than just bgroup, egroup,
  // // BUT you DON'T want extra {, } showing up in any untex-ing.
  // DefConstructor('\@hidden@bgroup', '//body', beforeDigest => sub { $_[0]->bgroup; },
  // captureBody => 1,   reversion => sub { Revert($_[0]->getProperty('body')); });
  // DefConstructor('\@hidden@egroup', '', afterDigest => sub { $_[0]->egroup; },
  //   reversion => '');

  DefPrimitiveI!(
    "\\begingroup",
    primitiveproc!(stomach, _args, state, {
      stomach.begingroup(state);
    })
  );
  DefPrimitiveI!(
    "\\endgroup",
    primitiveproc!(stomach, _args, state, {
      stomach.endgroup(state)?;
    })
  );

  // // Debugging aids; Ignored!
  DefPrimitive!("\\show Token",     None);
  DefPrimitive!("\\showbox Number", None);
  DefPrimitive!("\\showlists",      None);
  DefPrimitive!("\\showthe Token",  None);

  // // DefPrimitive('\shipout ??
  DefPrimitiveI!("\\ignorespaces SkipSpaces", noprimitive!());

  DefPrimitive!("\\lx@ignorehardspaces", sub[stomach, whatsit, state] {
    let mut boxes = Vec::new();
    while let Some(token) = stomach.get_gullet_mut().read_x_token(false, false, state)? {
      boxes = stomach.invoke_token(&token, state)?;
      if boxes.is_empty() {
        break;
      }
      while !boxes.is_empty() {
        let is_space = if let Some(space_val) = boxes[0].get_property("isSpace", state) {
          match space_val {
            Cow::Borrowed(Stored::Bool(space_bool)) => *space_bool,
            Cow::Owned(Stored::Bool(ref space_bool))  => *space_bool, // TODO : is there match syntax for Cow ?
            _ => false
          }
        } else {
          false
        };

        if is_space {
          boxes = boxes[1..].to_vec();
        } else {
          break;
        }
      }

      if !boxes.is_empty() {
        break;
      }
    }
    Ok(boxes)
  });

  // // \afterassignment saves ONE token (globally!) to execute after the next assignment
  // DefPrimitive('\afterassignment Token', sub { AssignValue(afterAssignment => $_[1], 'global');
  // }); 
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the
  // next egroup or } 
  // DefPrimitive('\aftergroup Token', sub { PushValue(afterGroup => $_[1]); });

  // // \uppercase<general text>, \lowercase<general text>
  // sub ucToken {
  //   my ($token) = @_;
  //   my $code = $STATE->lookupUCcode($token->getString);
  //   return ((defined $code) && ($code != 0) ? Token(chr($code), $token->getCatcode) : $token); }

  // sub lcToken {
  //   my ($token) = @_;
  //   my $code = $STATE->lookupLCcode($token->getString);
  //   return ((defined $code) && ($code != 0) ? Token(chr($code), $token->getCatcode) : $token); }

  // DefMacro('\uppercase GeneralText', sub {
  //     my ($gullet, $tokens) = @_;
  //     return map { ucToken($_) } $tokens->unlist; });

  // DefMacro('\lowercase GeneralText', sub {
  //     my ($gullet, $tokens) = @_;
  //     return map { lcToken($_) } $tokens->unlist; });

  // DefPrimitive('\message{}', sub {
  //     my ($stomach, $stuff) = @_;
  //     print STDERR ToString(Expand($stuff)) . "\n" if LookupValue('VERBOSITY') > -1;
  //     return; });

  // DefRegister('\errhelp' => Tokens());
  // DefPrimitive('\errmessage{}', sub {
  //     my ($stomach, $stuff) = @_;
  // print STDERR ToString(Expand($stuff)) . ": " . ToString(Expand(Tokens(T_CS('\the'),
  // T_CS('\errhelp')))) . "\n";     return; });

  // # TeX I/O primitives
  // DefPrimitive('\openin Number SkipMatch:= SkipSpaces TeXFileName', sub {
  //     my ($stomach, $port, $filename) = @_;
  //     # possibly should close $port if it's already been opened?
  //     $port     = ToString($port);
  //     $filename = ToString($filename);
  //     # Rely on FindFile to enforce any access restrictions
  //     if (my $path = FindFile($filename)) {
  //       my $mouth = LaTeXML::Core::Mouth->create($path,
  //         content => LookupValue($path . '_contents'));
  //       AssignValue('input_file:' . $port => $mouth, 'global'); }
  //     return; });

  // DefPrimitive('\closein Number', sub {
  //     my ($stomach, $port, $filename) = @_;
  //     #   close the mouth (if any) and clear the variable
  //     $port = ToString($port);
  //     if (my $mouth = LookupValue('input_file:' . $port)) {
  //       $mouth->finish;
  //       AssignValue('input_file:' . $port => undef, 'global'); }
  //     return; });

  // DefPrimitive('\read Number SkipKeyword:to SkipSpaces Token', sub {
  //     my ($stomach, $port, $token) = @_;
  //     $port = ToString($port);
  //     if (my $mouth = LookupValue('input_file:' . $port)) {
  //       $stomach->bgroup;
  //       AssignValue(PRESERVE_NEWLINES => 2);
  //       my @tokens = ();
  //       my ($t, $level) = (undef, 0);
  //       while ($t = $mouth->readToken) {
  //         my $cc = $t->getCatcode;
  //         push(@tokens, $t);
  //         $level++ if $cc == CC_BEGIN;
  //         $level-- if $cc == CC_END;
  //         last if ((($cc == CC_SPACE) && ($t->getString eq "\n"))
  //           || ($cc == CC_COMMENT)
  //           || ($t->equals(T_CS('\par')))) && !$level; }
  //       $stomach->egroup;
  //       @tokens = (T_CS('\par')) unless @tokens;    # trailing blank line
  //       DefMacroI($token, undef, Tokens(@tokens)); }
  //     return; });

  // DefConditional('\ifeof Number', sub {
  //     my ($gullet, $port) = @_;
  //     $port = ToString($port);
  //     if (my $mouth = LookupValue('input_file:' . $port)) {
  //       return $$mouth{at_eof}; }
  //     else {
  //       return 1; } });

  // # For output files, we'll write the data to a cached internal copy
  // # rather than to the actual file system.
  // DefPrimitive('\openout Number SkipMatch:= SkipSpaces TeXFileName', sub {
  //     my ($stomach, $port, $filename) = @_;
  //     $port     = ToString($port);
  //     $filename = ToString($filename);
  //     AssignValue('output_file:' . $port  => $filename, 'global');
  //     AssignValue($filename . '_contents' => "",        'global');
  //     return; });

  // DefPrimitive('\closeout Number', sub {
  //     my ($stomach, $port) = @_;
  //     $port = ToString($port);
  //     AssignValue('output_file:' . $port => undef, 'global');
  //     return; });

  // DefPrimitive('\write Number {}', sub {
  //     my ($stomach, $port, $tokens) = @_;
  //     $port = ToString($port);
  //     if (my $filename = LookupValue('output_file:' . $port)) {
  //       my $handle   = $filename . '_contents';
  //       my $contents = LookupValue($handle);
  //       AssignValue($handle => $contents . UnTeX($tokens) . "\n", 'global'); }
  //     else {
  //       print STDERR UnTeX(Expand($tokens)) . "\n"; }
  //     return; });

  // # Since we don't paginate, we're effectively always "shipping out",
  // # so all operations are \immediate
  // DefPrimitive('\immediate', undef);

  // #======================================================================
  // # Remaining semi- Vertical Mode primitives in Ch.24, pp.280--281

  // DefPrimitive('\special {}',     undef);
  // DefPrimitive('\penalty Number', undef);
  // DefPrimitive('\kern Dimension', undef);
  // DefMacro('\mkern MuGlue', '\ifmmode\@math@mskip #1\relax\else\@text@mskip #1\relax\fi');
  // DefPrimitiveI('\unpenalty', undef, undef);
  // DefPrimitiveI('\unkern',    undef, undef);
  // ## Worrisome, but...
  // DefPrimitiveI('\unskip', undef, sub {
  //     my ($stomach) = @_;
  //     my $box;
  //     while (($box = $LaTeXML::LIST[-1]) && IsEmpty($box)) {
  //       pop(@LaTeXML::LIST); }
  //     return; });

  // DefPrimitive('\mark{}', undef);
  // # \insert<8bit><filler>{<vertical mode material>}
  // DefPrimitive('\insert Number', undef);    # Just let the insertion get processed(?)
  //                                           # \vadjust<filler>{<vertical mode material>}
  //                                           # Note: \vadjust ignores in vertical mode...
  //     # is it sufficient to just clear the macro to avoid recursion?
  //     # (we don't track horizontal/vertical mode)
  // DefMacroI('\LTX@vadjust@afterpar', undef, '\def\LTX@vadjust@afterpar{}');
  // DefMacroI('\LTX@clear@vadjust@afterpar', undef, '\def\LTX@vadjust@afterpar{\def\LTX@vadjust@afterpar{}}');
  // DefPrimitive('\vadjust {}', sub {
  //     AddToMacro('\LTX@vadjust@afterpar', $_[1]->unlist);
  //     return; });

  // #======================================================================
  // # Remaining Vertical Mode primitives in Ch.24, pp.281--283
  // # \vskip<glue>, \vfil, \vfill, \vss, \vfilneg
  // # <leaders> = \leaders | \cleaders | \xleaders
  // # <box or rule> = <box> | <vertical rule> | <horizontal rule>
  // # <vertical rule> = \vrule<rule specification>
  // # <horizontal rule> = \hrule<rule specification>
  // # <rule specification> = <optional spaces> | <rule dimension><rule specification>
  // # <rule dimension> = width <dimen> | height <dimen> | depth <dimen>

  // # Stuff to ignore for now...
  // foreach my $op ('\vfil', '\vfill', '\vss', '\vfilneg',
  //   '\leaders', '\cleaders', '\xleaders') {
  //   DefPrimitive($op, undef); }

  // # \moveleft<dimen><box>, \moveright<dimen><box>
  // DefConstructor('\moveleft Dimension MoveableBox',
  //   "<ltx:text xoffset='#x' _noautoclose='1'>#2</ltx:text>",
  //   afterDigest => sub {
  //     $_[1]->setProperty(x => $_[1]->getArg(1)->multiply(-1)); });
  // DefConstructor('\moveright Dimension MoveableBox',
  //   "<ltx:text xoffset='#x' _noautoclose='1'>#2</ltx:text>",
  //   afterDigest => sub {
  //     $_[1]->setProperty(x => $_[1]->getArg(1)); });

  // # \unvbox<8bit>, \unvcopy<8bit>
  // DefPrimitive('\unvbox Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     AssignValue($box, undef);
  //     (defined $stuff ? $stuff->unlist : ()); });
  // DefPrimitive('\unvcopy Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     (defined $stuff ? $stuff->unlist : ()); });

  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, props, state] {
    unpack!(args => length);
    let length = length.pt_value(None);
    
    if length > 10.0 {    // Or what!?!?!?!
      if document.is_closeable("ltx:para", state).is_some() {
        document.close_element("ltx:para", state)?;
      } else if document.is_openable("ltx:break", state) {
        document.insert_element("ltx:break", Vec::new(), None, state)?;
      }
    }},
    properties => properties!(map!("isSpace" => true.into(), "isVerticalSpace" => true.into()))
  );
});
