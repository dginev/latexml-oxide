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

  // Perl: DefMacro('\vspace OptionalMatch:* {}', '\vskip #2\relax');
  // Note: wiring to \vskip causes paragraph breaks in moderncv — keep as stub for now
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
      reenter_text_mode(false); },
    properties   => sub[args] {
      // Perl: (($_[2] ? (align => $makebox_alignment{...}) : ()), ($_[1] ? (width => $_[1]) : ()))
      let mut props = stored_map!();
      if let Some(ref dim_d) = args[0] {
        if let DigestedData::RegisterValue(v) = dim_d.data() {
          let dim: Dimension = v.into();
          props.insert("width", Stored::from(dim));
        }
      }
      if let Some(ref align_d) = args[1] {
        let align_str = align_d.to_string();
        let align = match align_str.as_str() {
          "l" => "left",
          "r" => "right",
          "s" => "justified",
          _ => "",
        };
        if !align.is_empty() {
          props.insert("align", Stored::from(align));
        }
      }
      Ok(props)
    }
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
  // When in math mode, produces <ltx:XMArg enclose='box'> instead of <ltx:text framed='rectangle'>
  DefConstructor!("\\@framebox[Dimension][]{}",
    "?#mathframe(<ltx:XMArg enclose='box'>#inner</ltx:XMArg>)\
     (<ltx:text ?#width(width='#width') ?#align(align='#align') ?#cssstyle(cssstyle='#cssstyle') framed='rectangle' framecolor='#framecolor' _noautoclose='1'>#3</ltx:text>)",
    alias => "\\framebox",
    sizer => "#3",
    before_digest => {
      let wasmath = state::lookup_value("IN_MATH").is_some();
      stomach::begin_mode("restricted_horizontal")?;
      state::assign_value("FRAME_IN_MATH", wasmath, None); },
    properties => sub[args] {
      // Perl: framecolor => LookupValue('font')->getColor
      let framecolor = lookup_font()
        .and_then(|f| f.get_color().cloned())
        .map(|c| c.to_attribute())
        .unwrap_or_else(|| s!("#000000"));
      let mut props = stored_map!("framecolor" => framecolor);
      // Perl: align from arg 2 (optional []) — only set when explicitly given
      // Perl: only emit align for l/r/s; 'c' (center) is default → not emitted
      if let Some(align_val) = args[1].as_ref() {
        let align_str = align_val.to_string();
        let mapped = match align_str.as_str() {
          "l" => Some("left"),
          "r" => Some("right"),
          "s" => Some("justified"),
          _ => None, // 'c' or empty → default center, not emitted
        };
        if let Some(m) = mapped {
          props.insert("align", Stored::String(arena::pin_static(m)));
        }
      }
      if let Some(width_val) = args[0].as_ref() {
        props.insert("width", Stored::String(arena::pin(width_val.to_attribute())));
      }
      // Perl: ($sep ne '3.0pt' ? (cssstyle => 'padding:' . $sep) : ())
      if let Some(sep) = lookup_dimension("\\fboxsep") {
        let sep_str = sep.to_attribute();
        if sep_str != "3.0pt" {
          props.insert("cssstyle", Stored::String(arena::pin(s!("padding:{sep_str}"))));
        }
      }
      Ok(props)
    },
    after_digest => sub[whatsit] {
      let wasmath = state::lookup_value("FRAME_IN_MATH").is_some_and(|v| matches!(v, Stored::Bool(true)));
      let arg = whatsit.get_arg(3).cloned();
      stomach::end_mode("restricted_horizontal")?;
      if wasmath {
        if let Some(ref a) = arg {
          // Perl: $arg->isMath checks mode property =~ /math$/
          // For \fbox{$...$}, the body is a List in restricted_horizontal mode
          // containing a math whatsit. Check if any child has isMath.
          let is_math = a.get_property_bool("isMath")
            || a.unlist().iter().any(|child| child.get_property_bool("isMath"));
          if is_math {
            whatsit.set_property("mathframe", true);
            // Extract inner body for the XMArg template
            // For \fbox{$...$}, get the math body from the inner whatsit
            if let Ok(Some(body)) = a.get_body() {
              whatsit.set_property("inner", body);
            } else {
              // Fallback: use the entire arg
              whatsit.set_property("inner", a.clone());
            }
          }
        }
      }
    },
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
    // Sizer: width from arg #4 (Dimension), height/depth from body (arg #5)
    // Perl: sizer => '#5' — uses font.computeBoxesSize(body, vattach => ..., width => ...)
    // which does proper line breaking and vattach transformation.
    sizer => sub[whatsit] {
      // Width from the "width" property (arg #4 Dimension)
      let w = whatsit.get_property("width")
        .and_then(|s| Dimension::new_f64(Dimension::spec_to_f64(&s.to_string()).ok()?).into())
        .unwrap_or_default();
      // Height/depth from body (arg #5 VBoxContents)
      if let Some(body) = whatsit.get_arg(5) {
        let w_val = w.value_of();
        if w_val > 0 {
          // Approximate paragraph height: measure total unwrapped width,
          // estimate lines, use \baselineskip for line height.
          let (body_w, body_h, body_d) = body.compute_size(SymHashMap::default())?;
          let total_w = body_w.value_of();
          let (mut ht, mut dp) = if total_w > w_val {
            // Paragraph wrapping: estimate number of lines
            let num_lines = ((total_w as f64) / (w_val as f64)).ceil() as i64;
            // Use \baselineskip (typically 12pt = 786432 sp) for line height
            let baseline_skip = state::lookup_dimension("\\baselineskip")
              .unwrap_or(Dimension::new(786432)); // 12pt default
            let line_h = baseline_skip.value_of();
            let total_h = num_lines * line_h;
            // Default: top alignment (first line as height)
            let first_line_h = body_h.value_of().max(line_h * 2 / 3);
            (first_line_h, total_h - first_line_h)
          } else {
            (body_h.value_of(), body_d.value_of())
          };
          // Perl Font.pm L793-800: apply vattach transformation
          let vattach = whatsit.get_property("vattach")
            .map(|v| v.to_string())
            .unwrap_or_default();
          let total = ht + dp;
          if vattach == "middle" {
            let font_size = lookup_font()
              .and_then(|f| f.get_size().map(|s| s as i64))
              .unwrap_or(10);
            let hh = total / 2;
            let c = font_size * UNITY / 4; // math axis ≈ size/4
            ht = hh + c;
            dp = hh - c;
          } else if vattach == "bottom" {
            // Align to baseline of bottom row
            let last_line_d = body_d.value_of().min(total);
            dp = last_line_d;
            ht = total - dp;
          }
          // else: "top"/"baseline" — keep first line as height (default above)
          Ok((w, Dimension::new(ht), Dimension::new(dp)))
        } else {
          let (_, h, d) = body.compute_size(SymHashMap::default())?;
          Ok((w, h, d))
        }
      } else {
        Ok((w, Dimension::default(), Dimension::default()))
      }
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
      let attachment = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let vattach = translate_attachment(&attachment);
      let width = match props.get("width") {
        Some(Stored::Dimension(d)) => d.to_attribute(),
        Some(w) => w.to_string(),
        None => args.get(3).and_then(|a| a.as_ref()).map(|a| a.to_attribute())
          .unwrap_or_default(),
      };
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
    after_digest_begin => sub[whatsit] {
      // Perl: afterDigestBegin sets \hsize, \textwidth, \columnwidth from width arg
      let vattach = whatsit.get_arg(1)
        .map(|a| translate_attachment(a.to_string()))
        .unwrap_or("middle");
      if let Some(width_arg) = whatsit.get_arg(4) {
        let width_val = width_arg.value_of();
        let dim = Dimension::new(width_val);
        let rv: RegisterValue = dim.into();
        state::assign_register("\\hsize", rv.clone(), None, Vec::new())?;
        state::assign_register("\\textwidth", rv.clone(), None, Vec::new())?;
        state::assign_register("\\columnwidth", rv, None, Vec::new())?;
        whatsit.set_property("width", Stored::Dimension(dim));
      }
      whatsit.set_property("vattach", Stored::from(vattach.to_string()));
      Let!("\\\\", "\\lx@newline");
    },
    after_digest_body => sub[whatsit] {
      // Perl: afterDigestBody copies vattach from whatsit to body
      if let Some(vattach) = whatsit.get_property("vattach").map(|v| v.into_owned()) {
        if let Some(Stored::Digested(body)) = whatsit.properties.get("body").cloned() {
          let mut body = body;
          body.set_property("vattach", vattach);
        }
      }
    }
  );

  DefConstructor!("\\rule[Dimension]{Dimension}{Dimension}",
    "<ltx:rule ?#offset(yoffset='#offset') width='#width' height='#height'/>",
    enter_horizontal => true,
    properties => sub[args] {
      Ok(stored_map!(
        "offset" => args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "width" => args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "height" => args[2].as_ref().map(|a| a.to_attribute()).unwrap_or_default()
      ))
    }
  );
  DefConstructor!("\\raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:text yoffset='#1' _noautoclose='1'>#4</ltx:text>",
    mode         => "text", bounded => true,
    before_digest => {
      reenter_text_mode(false); }
    // TODO
    // sizer        => sub { raisedSizer($_[0]->getArg(4), $_[0]->getArg(1)); }
  );
});
