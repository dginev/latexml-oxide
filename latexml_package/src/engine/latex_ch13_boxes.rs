use crate::prelude::*;

//**********************************************************************
// C.13 Lengths, Spaces and Boxes
//**********************************************************************

//////////
//  Complete to here
//  [except for NOTE'd entries, of course]
//////////

// TODO:
// sub raisedSizer {
//   my ($box, $y) = @_;
//   my ($w, $h, $d) = $box->getSize;
//   my $z = Dimension(0);
//   $h = $h->add($y)->larger($z);
//   $d = $d->subtract($y)->larger($z);
//   return ($w, $h, $d); }

LoadDefinitions!({
  //======================================================================
  // C.13.1 Length
  //======================================================================
  // \fill
  DefMacro!("\\stretch{}", "0pt plus #1fill\\relax");

  DefPrimitive!("\\@check@length DefToken", sub[(cs)] {
    match lookup_definition(&cs)? {
      None => {
        let message = s!("'{}' is not a length; defining it now", cs.stringify());
        Warn!("undefined", cs, message);
        DefRegister!(cs, None, Dimension::new(0));
      },
      Some(defn) => if !defn.is_register() {
        let message = s!("'{}' length was expected, got {:?} instead of register.",
          cs.to_string(), defn.register_type());
        Error!("misdefined", cs, message);
      }
    };
  });

  DefPrimitive!("\\newlength DefToken", sub[(cs)] {
    DefRegister!(cs, None, Glue::new(0), allocate => "\\skip");
    Ok(vec![])
  });

  DefPrimitive!("\\setlength {Variable}{Dimension}", sub[(variable,length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      let defn = rtoken.to_register()
        .expect("if a Variable parameter provides a token, it must have a Register definition.");
      defn.set_value(length.into(), None, params);
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\addtolength {Variable}{Dimension}", sub[(variable,length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      let defn = rtoken.to_register()
        .expect("if a Variable parameter provides a token, it must have a Register definition.");
      // TODO: can we avoid cloning the params?
      let oldlength = defn.value_of(params.clone()).unwrap_or_default();
      defn.set_value(oldlength.add(length), None, params);
    }
    Ok(Vec::new())
  });

  DefMacro!(
    "\\@settodim{}{}{}",
    "\\setbox\\@tempboxa\\hbox{{#3}}#2#1\\@tempboxa\\setbox\\@tempboxa\\box\\voidb@x"
  );
  DefMacro!("\\settoheight", "\\@settodim\\ht");
  DefMacro!("\\settodepth", "\\@settodim\\dp");
  DefMacro!("\\settowidth", "\\@settodim\\wd");
  DefMacro!(r"\@settopoint{}", r"\divide#1\p@\multiply#1\p@");

  DefRegister!("\\fill", Glue!("0pt plus 1fill"));

  //======================================================================
  // C.13.2 Space
  //======================================================================

  DefPrimitive!("\\hspace OptionalMatch:* {Dimension}", sub[(_star,length)] {
    let s = dimension_to_spaces(length);
    if !s.is_empty() {
      let length_tokens = length.revert()?;

      let tokens = Invocation!(T_CS!("\\hskip"), vec![length_tokens]);
      Tbox::new(arena::pin(&s), None, None, tokens,
        stored_map!("width" => length, "isSpace" => true));
    }
  });

  DefPrimitive!("\\vspace OptionalMatch:* {}", None);
  DefPrimitive!("\\addvspace {}", None);
  DefPrimitive!("\\addpenalty {}", None);
  DefPrimitive!("\\@endparenv", None);

  //======================================================================
  // C.13.3 Boxes
  //======================================================================
  // Can't really get these?
  DefMacro!("\\height", "0pt");
  DefMacro!("\\totalheight", "0pt");
  DefMacro!("\\depth", "0pt");
  DefMacro!("\\width", "0pt");

  DefConstructor!("\\mbox {}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "text",
    bounded => true,
    sizer => "#1",
    before_digest => {
      reenter_text_mode(false); }
  );

  // our %makebox_alignment = (l => 'left', r => 'right', s => 'justified');
  DefMacro!("\\makebox", "\\@ifnextchar(\\pic@makebox\\@makebox");
  // Perl: enterHorizontal => 1 (now automatic via mode => "text")
  DefConstructor!("\\@makebox[Dimension][]{}",
    "<ltx:text ?#width(width='#width') ?#align(align='#align') _noautoclose='1'>#3</ltx:text>",
    mode         => "text", bounded => true, alias => "\\makebox", sizer => "#3",
    before_digest => {
      reenter_text_mode(false); }
    // properties   => sub[args] {
    //   let arg1 = &args[0];
    //   let arg2 = &args[1];
    //   (($_[2] ? (align => $makebox_alignment{ ToString($_[2]) }) : ()),
    //     ($_[1] ? (width => $_[1]) : ())) }
  );

  DefRegister!("\\fboxrule", Dimension!(".4pt"));
  DefRegister!("\\fboxsep", Dimension!("3pt"));

  // Peculiar special case!
  //  These are nominally text mode macros. However, there is a somewhat common idiom:
  //     $ ... \framebox{$operator$} ... $
  // in which case the operator gets boxed and really should be treated as a math object.
  // (and ultimately converted to mml:menclose)
  // So, we need to switch to text mode, as usual, but FIRST note whether we started in math mode!
  // Afterwards, if we were in math mode, and the content is math, we'll convert the whole thing
  // to a framed math object.
  // Second special issue:
  //   Although framebox doesn't allow flowed content inside, it is also somewhat common
  // to put a vbox or some other block construct inside.
  // Seemingly, the ultimate html gets somewhat tangled (browser bugs?)
  // At any rate, since we're wrapping with an ltx:text, we'll try to unwrap it,
  // if the contents are a single child that can handle the framing.

  DefMacro!("\\fbox{}", "\\@framebox{#1}");
  DefMacro!("\\framebox", "\\@ifnextchar(\\pic@framebox\\@framebox");
  // Perl: DefConstructor('\@framebox[Dimension][]{}', ...)
  // Perl uses restricted_horizontal mode, saves IN_MATH, unwraps single children
  DefConstructor!("\\@framebox[Dimension][]{}",
    "<ltx:text ?#width(width='#width') framed='rectangle' _noautoclose='1'>#3</ltx:text>",
    alias => "\\framebox",
    mode => "text", bounded => true,
    before_digest => {
      reenter_text_mode(false); },
    after_construct => sub[document, _whatsit] {
      // Perl afterConstruct: if the <ltx:text> has a single non-text child
      // that can have 'framed', unwrap the text and copy attributes to the child.
      let current = document.get_node().clone();
      if let Some(node) = current.get_last_child() {
        if document::get_node_qname(&node) != arena::pin_static("ltx:text") {
          return Ok(());
        }
        // Filter to non-whitespace children
        let children: Vec<Node> = node.get_child_nodes().into_iter().filter(|n| {
          if n.get_type() == Some(NodeType::ElementNode) {
            true
          } else {
            // text node — keep only if non-whitespace
            n.get_content().chars().any(|c| !c.is_whitespace())
          }
        }).collect();
        if children.len() == 1
          && children[0].get_type() == Some(NodeType::ElementNode)
          && document::can_node_have_attribute(&children[0], "framed")
          && !children[0].has_attribute("framed")
        {
          // Copy attributes from ltx:text to child, then unwrap
          for attr in ["width", "align", "framed"] {
            if let Some(v) = node.get_attribute(attr) {
              document.set_attribute(&mut children[0].clone(), attr, &v)?;
            }
          }
          document.unwrap_nodes(node)?;
        }
      }
    }
  );

  AssignValue!("SAVEBOX", 100);
  TeX!(
    r#"""\def\newsavebox#1{\@ifdefinable{#1}{\newbox#1}}
  \DeclareRobustCommand\savebox[1]{%
    \@ifnextchar(%)
      {\@savepicbox#1}{\@ifnextchar[{\@savebox#1}{\sbox#1}}}%
  \DeclareRobustCommand\sbox[2]{\setbox#1\hbox{%
    \color@setgroup#2\color@endgroup}}
  \def\@savebox#1[#2]{%
    \@ifnextchar [{\@isavebox#1[#2]}{\@isavebox#1[#2][c]}}
  \long\def\@isavebox#1[#2][#3]#4{%
    \sbox#1{\@imakebox[#2][#3]{#4}}}
  \def\@savepicbox#1(#2,#3){%
    \@ifnextchar[%]
      {\@isavepicbox#1(#2,#3)}{\@isavepicbox#1(#2,#3)[]}}
  \long\def\@isavepicbox#1(#2,#3)[#4]#5{%
    \sbox#1{\@imakepicbox(#2,#3)[#4]{#5}}}
  \def\lrbox#1{%
    \edef\reserved@a{%
      \endgroup
      \setbox#1\hbox{%
        \begingroup\aftergroup}%
          \def\noexpand\@currenvir{\@currenvir}%
          \def\noexpand\@currenvline{\on@line}}%
    \reserved@a
      \@endpefalse
      \color@setgroup
        \ignorespaces}
  \def\endlrbox{\unskip\color@endgroup}
  \DeclareRobustCommand\usebox[1]{\leavevmode\copy #1\relax}
  """#
  );

  // DefMacro!(T_CS!("\\begin{lrbox}"), '{Token}', "\@begin@lrbox #1");
  // DefPrimitive!("\\end{lrbox}", primtiveproc!( args, {stomach.egroup()?; }));
  // DefPrimitive!("\\@begin@lrbox Token", sub {
  //     my ($stomach, $token) = @_;
  //     $stomach->bgroup;
  //     my $box = List($stomach->digestNextBody());
  //     AssignValue('box' . ToString($token), $box); });

  // DefPrimitive!("\\usebox {Register}", sub {
  //     my ($defn) = @{ $_[1] };
  //     return Box() unless $defn && ($defn ne 'missing');
  //     my $value = $defn->valueOf()->valueOf;
  //     LookupValue('box' . $value) || Box(); });

  // A soft sorta \par that only closes an ltx:p, but not ltx:para
  DefConstructor!("\\lx@parboxnewline[]", sub[document, _args, _props] {
    document.maybe_close_element("ltx:p")?;
  });

  // Perl: latex_constructs.pool.ltxml lines 4795-4818
  Let!("\\lx@parboxnewline", "\\lx@newline");
  // NOTE: There are 2 extra arguments (See LaTeX Companion, p.866)
  // for height and inner-pos. We're ignoring inner-pos, for now, though.
  DefMacro!("\\parbox[] [] [] {Dimension}{}",
    r"\lx@hidden@bgroup\hsize=#4\textwidth\hsize\columnwidth\hsize\ifx.#2.\lx@parbox[#1]{#4}{#5}\else\lx@parbox[#1][#2][#3]{#4}{#5}\fi\lx@hidden@egroup");
  DefConstructor!("\\lx@parbox[][Dimension] OptionalUndigested {Dimension} VBoxContents",
    sub[document, args, props] {
      let body = args[4].as_ref().unwrap();
      let mut attr = string_map!("class" => "ltx_parbox");
      if let Some(w) = props.get("width") { attr.insert("width".to_string(), w.to_string()); }
      if let Some(v) = props.get("vattach") { attr.insert("vattach".to_string(), v.to_string()); }
      insert_block(document, body, attr)?;
    },
    alias => "\\parbox",
    properties => sub[args] {
      let attachment = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let width = args[3].as_ref().map(|w| w.to_attribute()).unwrap_or_default();
      Ok(stored_map!("width" => width, "vattach" => translate_attachment(&attachment)))
    },
    mode => "internal_vertical",
    before_digest => {
      Let!("\\\\", "\\lx@newline");
    }
  );
  DefMacro!("\\@parboxrestore", "");

  DefConditional!("\\if@minipage");
  DefMacro!("\\@setminipage", "");
  // Perl: latex_constructs.pool.ltxml lines 4822-4846
  DefEnvironment!("{minipage}[] OptionalUndigested [] {Dimension}",
    sub[document, args, props] {
      let attachment = args.get(0).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let vattach = translate_attachment(&attachment);
      let width = props.get("width").map(|w| w.to_string())
        .or_else(|| args.get(3).and_then(|a| a.as_ref()).map(|a| a.to_attribute()))
        .unwrap_or_default();
      let mut attr = string_map!("class" => "ltx_minipage");
      if !width.is_empty() { attr.insert("width".to_string(), width); }
      attr.insert("vattach".to_string(), vattach.to_string());
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
      }
      Ok(())
    },
    mode => "internal_vertical",
    before_digest => {
      stomach::digest(Tokens!(T_CS!("\\@minipagetrue")))?;
    },
    after_digest_begin => {
      // TODO: set hsize/textwidth/columnwidth from width arg via whatsit access
      Let!("\\\\", "\\lx@newline");
    }
  );

  // DefConstructor("\\rule[Dimension]{Dimension}{Dimension}",
  //   "<ltx:rule ?#offset(yoffset='#offset') width='#width' height='#height'/>",
  //   properties => sub { (offset => $_[1], width => $_[2], height => $_[3]) });
  // Perl: enterHorizontal => 1 (now automatic via mode => "text")
  DefConstructor!("\\raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:text yoffset='#1' _noautoclose='1'>#4</ltx:text>",
    mode         => "text", bounded => true,
    before_digest => {
      reenter_text_mode(false); }
    // TODO
    // sizer        => sub { raisedSizer($_[0]->getArg(4), $_[0]->getArg(1)); }
  );
});
