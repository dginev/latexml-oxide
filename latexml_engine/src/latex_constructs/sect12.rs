//! `latex_constructs` section 12: C.12-C.13 Line/Page Breaking, Boxes
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

/// The type TeX scans for an assignment to a register of `register` type
/// (`\setlength#1#2{#1 #2\relax}`, latex.ltx:10253): its own, a `\chardef`'s
/// being a number. A register of no numeric type falls back to a dimension,
/// Perl's `{Dimension}`.
fn length_register_type(register: Option<RegisterType>) -> RegisterType {
  match register {
    Some(
      kind @ (RegisterType::Number
      | RegisterType::Dimension
      | RegisterType::Glue
      | RegisterType::MuDimension
      | RegisterType::MuGlue),
    ) => kind,
    Some(RegisterType::CharDef) => RegisterType::Number,
    _ => RegisterType::Dimension,
  }
}

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.12-C.13 Line/Page Breaking, Boxes
  // ======================================================================

  //======================================================================
  // C.12.1 Line Breaking
  //======================================================================
  DefPrimitive!("\\linebreak[]");
  DefPrimitive!("\\nolinebreak[]");
  DefPrimitive!("\\-"); // We don't do hyphenation.
  // \hyphenation in TeX.pool
  DefPrimitive!("\\sloppy");
  DefPrimitive!("\\fussy");
  // sloppypar can be used as an environment, or by itself.
  DefMacro!("\\sloppypar", "\\par\\sloppy");
  DefMacro!("\\endsloppypar", "\\par");
  DefMacro!("\\nobreakdashes", "-");
  def_macro_identity("\\showhyphens{}")?;
  //======================================================================
  // C.12.2 Page Breaking
  //======================================================================
  DefMacro!("\\pagebreak[Default:4]", sub[(arg_opt)] {
      let arg : u32 = if let Some(arg_t) = arg_opt {
        arg_t.to_string().parse::<u32>().unwrap_or(0)
      } else { 0 };
      if arg <= 2 {
        Ok(Tokens!()) }
      else {
        Ok(Tokens::new(vec![T_CS!("\\vadjust"), T_BEGIN!(), T_CS!("\\clearpage"), T_END!()]))
      }
  });
  DefPrimitive!("\\nopagebreak[]");
  DefPrimitive!("\\columnbreak"); // latex? or multicol?
  DefPrimitive!("\\enlargethispage OptionalMatch:* {}");

  DefMacro!("\\clearpage", "\\lx@newpage");
  DefMacro!("\\cleardoublepage", "\\lx@newpage");
  DefPrimitive!("\\samepage");

  //======================================================================
  // C.13.1 Length
  //======================================================================
  // \fill
  DefMacro!("\\stretch{}", "0pt plus #1fill\\relax");

  // \@check@length helper (used by \newlength below) lives in
  // `latex_constructs_rust_only.rs` (Rust-only addition).

  DefPrimitive!("\\newlength DefToken", sub[(cs)] {
    DefRegister!(cs, None, Glue::new(0), allocate => "\\skip");
    Ok(vec![])
  });

  // Perl parity: `return unless $defn && ($defn ne 'missing');` — silently
  // skip when the target variable has no register definition (e.g. undefined
  // length register). Matches calc_sty.rs's \setlength/\addtolength fallback.
  //
  // The value is read the way latex.ltx:10253-10254 reads it — `\setlength#1#2{#1
  // #2\relax}`, `\addtolength#1#2{\advance#1 #2\relax}`: by the register's own
  // type (a skip keeps its `plus`/`minus`, where Perl's `{Dimension}` read the
  // natural width only), and what follows the value in the braces stays in the
  // input, read after the assignment (`\setlength{\parindent}{2pt\foo}G` with
  // `\def\foo{x}` typesets "xG"; Perl drops it, KNOWN_PERL_ERRORS #275).
  // OXIDIZED_DESIGN #317; guard `braced_quantity_tail`.
  DefPrimitive!("\\setlength {Variable}{}", sub[(variable, length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      if let Some(defn) = rtoken.to_register() {
        let (value, tail) = read_braced_value(length, length_register_type(defn.register_type()))?;
        defn.set_value(value, None, params);
        unread_vec(tail);
      }
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\addtolength {Variable}{}", sub[(variable, length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      if let Some(defn) = rtoken.to_register() {
        let (value, tail) = read_braced_value(length, length_register_type(defn.register_type()))?;
        // TODO: can we avoid cloning the params?
        let oldlength = defn.value_of(params.clone()).unwrap_or_default();
        defn.set_value(oldlength.add(value), None, params);
        unread_vec(tail);
      }
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
  // \settototalheight sets its register to \ht+\dp of the box. Perl
  // calc.sty.ltxml L73-77 models it as a DefPrimitive that directly
  // sums getHeight+getDepth; we follow the same trampoline shape as
  // the sibling \setto* macros and use \advance to add the depth.
  DefMacro!(
    "\\settototalheight{}{}",
    "\\setbox\\@tempboxa\\hbox{{#2}}\
     #1\\ht\\@tempboxa\
     \\advance#1\\dp\\@tempboxa\
     \\setbox\\@tempboxa\\box\\voidb@x"
  );
  DefMacro!(r"\@settopoint{}", r"\divide#1\p@\multiply#1\p@");

  DefRegister!("\\fill", Glue!("0pt plus 1fill"));

  //======================================================================
  // C.13.2 Space
  //======================================================================

  // The length is a skip: latex.ltx:9425 `\@hspace#1{\setlength\sp@ce@skip{#1}…}`,
  // `\sp@ce@skip` a `\newskip` (:9424). Perl's `{Dimension}` stopped before a
  // `plus`/`minus`, which then passed for text after the value (OXIDIZED_DESIGN
  // #317): `\hspace{0pt plus 1fil}`, `\hspace{\stretch{1}}`. The space is the
  // natural width, as `\hskip`'s.
  DefPrimitive!("\\hspace OptionalMatch:* {SetlengthGlue}", sub[(star,skip)] {
    let skip: Glue = skip;
    let length = Dimension::new(skip.value_of());
    // Perl `latex_constructs.pool.ltxml:4686-4691` always emits a Box once
    // `DimensionToSpaces` returns a defined value — and `''` is defined,
    // so a literal `\hspace{0pt}` still yields an `isSpace` whatsit. In
    // math mode that Box becomes an `<ltx:XMHint>` atom that anchors a
    // following script (`\hspace{0mm}^c`), preventing the false
    // `unexpected:double-superscript` reported on 1603.08690 and similar
    // papers. Do NOT gate on `!s.is_empty()`.
    let s = dimension_to_spaces(length);
    let length_tokens = skip.revert()?;
    let tokens = Invocation!(T_CS!("\\hskip"), vec![length_tokens]);
    // Glue (latex.ltx:9425 `\hskip`): `\unskip` removes it. `\hspace*` puts a zero-width rule and a penalty
    // before its glue and `\z@skip` after it (latex.ltx:9427-9428), so `\unskip` takes the `\z@skip`.
    Tbox::new(pin(&s), None, None, tokens,
      stored_map!("width" => length, "isSpace" => true, "isSkip" => star.is_none()))
  });

  // Perl: DefMacro('\vspace OptionalMatch:* {}', '\vskip #2\relax');
  //
  // Restored Perl-faithful expansion 2026-04-28. The earlier no-op
  // DefPrimitive stub (kept as `WISDOM #44`) was retained to dodge a
  // moderncv cascade, but the no-op breaks `\bigskip\hrule` paragraph
  // separation (latex.ltx defines `\bigskip` as `\vspace\bigskipamount`,
  // which our no-op silently swallowed — no `\vskip` reached, so
  // `<ltx:para>` stayed open and `<rule>` landed inside it).
  // ntheorem_test, plus the simpler `bigskip_test`, both confirm the
  // Perl-faithful expansion produces the expected sibling layout.
  DefMacro!("\\vspace OptionalMatch:* {}", "\\vskip #2\\relax");
  def_primitive_noop("\\addvspace {}")?;
  def_primitive_noop("\\addpenalty {}")?;
  def_primitive_noop("\\@endparenv")?;

  //======================================================================
  // C.13.3 Boxes
  //======================================================================
  // Can't really get these?
  DefMacro!("\\height", "0pt");
  DefMacro!("\\totalheight", "0pt");
  DefMacro!("\\depth", "0pt");
  DefMacro!("\\width", "0pt");

  // Perl latex_constructs.pool.ltxml L4709-4714:
  //   beforeDigest => sub { Let(T_MATH, T_CS('\lx@dollar@default')); }
  // Rebinds `$` to the default text-mode toggle (so `\mbox{$x$}` opens
  // inline math). Match Perl literally rather than via reenter_text_mode.
  // BOX CONTENT IS A LIVE HBOX BODY (batch 54n; OXIDIZED_DESIGN #188). latex.ltx
  // `\mbox{#1}` = `\leavevmode\hbox{#1}` and `\@imakebox` = `\hb@xt@w{…#3…}`:
  // the pre-scanned argument is re-inserted and read as an `\hbox{` body in
  // the SAME list, so a group closed inside it from another macro closes the
  // box — ulem's `\hss` (`\let\hskip\UL@hskip`, `\afterassignment\UL@reskip`
  // → `\UL@stop` = `\egroup\egroup` … `\UL@start`) inside `\makebox[.5in][r]
  // {\hss}` (examdesign.cls:1210, the truefalse answer key) ends the makebox
  // at ulem's first `\egroup` and the makebox's own `}` later closes the box
  // ulem reopened. Digesting the `{}` argument in an isolated mouth under a
  // mode frame met that `\egroup` with the frame instead (Perl
  // latex_constructs.pool:4709-4724 shares it; examdesign examplea/b/c). So
  // the content parameter is `HBoxArgContents`: `read_box_arg_contents` reads
  // ONE macro argument — the `{` of a braced one, or a single token wrapped
  // as its own group (`\mbox\qquad` = `\hbox{\qquad}`, latex.ltx:16082; the
  // `\hbox`-style forward scan of `HBoxContents` swallowed an enclosing
  // group's `}` there, ltnews issue 40) — and the one-frame `readBoxContents`
  // loop digests from the live gullet until the frame closes. `bounded`
  // stays to scope the `$` rebind. Guards:
  // `perfect_kernel_batch54::box_constructor_content_is_a_live_hbox_body`,
  // `mbox_argument_is_bounded::unbraced_box_argument_is_one_token`.
  DefConstructor!("\\mbox HBoxArgContents", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    // `mode` stays, as on `\\hbox` (tex_box.rs): the constructor's own font is
    // the TEXT font, so a box in math carries no `font="italic"` (golden
    // 81_babel numprints).
    mode => "restricted_horizontal",
    // latex.ltx:16082 `\leavevmode\hbox{#1}`: the box starts the paragraph, so
    // the space after it is read in horizontal mode and kept (`\mbox{A} b` at a
    // paragraph's or an item's start; fancyvrb's `\Verb` ends in `\mbox`,
    // matapli-doc `\item \Verb+FiraSans+ et`; the biblatex manual's ltxdockit
    // `\sty` is an `\mbox`). Perl's `\mbox` has no
    // enterHorizontal (latex_constructs.pool.ltxml:4649-4654) and drops the
    // space too. Repro `tools/perfect_kernel/repros/boxes-groups/mbox_leavevmode_space.tex`.
    enter_horizontal => true,
    bounded => true,
    sizer => "#1",
    before_digest => {
      Let!(T_MATH!(), "\\lx@dollar@default");
    }
  );

  // Perl #2829: %makebox_alignment = (l=>'left', r=>'right', c=>'center',
  // s=>'stretched'); a width with no explicit alignment defaults to 'c'
  // (center). We map `s` to schema-valid `justified` instead of Perl's
  // `stretched` (not in the align enum) — OXIDIZED_DESIGN #239, makebox_alignment
  // in mod.rs.
  // Perl latex_constructs.pool.ltxml L4717: `robust => 1` so \makebox
  // survives \write/\edef contexts (e.g. captions, moving arguments).
  DefMacro!("\\makebox", "\\@ifnextchar(\\pic@makebox\\@makebox",
    robust => true);
  // Perl latex_constructs.pool.ltxml:4658-4667: `\@makebox` has NO
  // beforeDigest — the outer T_MATH binding persists — and enterHorizontal => 1
  // (latex.ltx:16077-16078 `\leavevmode`), which the mode alone does not give:
  // the space after a box starting a paragraph was dropped (batch 54n's mode
  // change; arXiv 2605.21322's `\makebox[0pt][c]` figure panels).
  DefConstructor!("\\@makebox[TempboxaDimension][] HBoxArgContents",
    "<ltx:text width='#width' align='#align' _noautoclose='1'>#3</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true, bounded => true,
    alias => "\\makebox", sizer => "#3",
    properties   => sub[args] {
      let mut props = stored_map!();
      let mut has_width = false;
      if let Some(ref dim_d) = args[0]
        && let DigestedData::RegisterValue(v) = dim_d.data() {
          let dim: Dimension = v.into();
          props.insert("width", Stored::from(dim));
          has_width = true;
        }
      let mut align_str = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
      // Perl #2829: `$align = 'c' if !$align && $width;`
      if align_str.is_empty() && has_width {
        align_str = "c".to_string();
      }
      let align = makebox_alignment(&align_str);
      if !align.is_empty() {
        props.insert("align", Stored::from(align));
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

  // Perl latex_constructs.pool.ltxml L4744-4745: both \fbox and
  // \framebox are defined with `robust => 1` so they survive
  // \write/\edef moving-argument contexts.
  DefMacro!("\\fbox{}", "\\@framebox{#1}", robust => true);
  DefMacro!("\\framebox", "\\@ifnextchar(\\pic@framebox\\@framebox",
    robust => true);
  // Perl: DefConstructor('\@framebox[Dimension][]{}', ...)
  // Perl uses restricted_horizontal mode, saves IN_MATH, unwraps single children
  // When in math mode, produces <ltx:XMArg enclose='box'> instead of <ltx:text framed='rectangle'>
  DefConstructor!("\\@framebox[TempboxaDimension][] HBoxArgContents",
    "?#mathframe(<ltx:XMArg enclose='box'>#inner</ltx:XMArg>)\
     (<ltx:text ?#width(width='#width') ?#align(align='#align') ?#cssstyle(cssstyle='#cssstyle') framed='rectangle' framecolor='#framecolor' _noautoclose='1'>#3</ltx:text>)",
    alias => "\\framebox",
    sizer => "#3",
    // `\fbox` (latex.ltx:16182-16183) and `\framebox`'s `\@iframebox`
    // (16196-16197) open with `\leavevmode` (Perl's `\@framebox` has no
    // enterHorizontal, pool:4689-4699): `\fbox{A} b` keeps its space (titlesec
    // manual's `\fbox{2.9}` version marks).
    enter_horizontal => true,
    before_digest => {
      // Perl: $wasmath = LookupValue('IN_MATH') — uses boolean value, not key existence.
      // IN_MATH is initialized to false at startup, so is_some() would always be true.
      let wasmath = lookup_bool_sym(pin!("IN_MATH"));
      begin_mode("restricted_horizontal")?;
      assign_value("FRAME_IN_MATH", wasmath, None); },
    properties => sub[args] {
      // Perl #2829: framedProperties(margin => '\fboxsep', rule =>
      // '\fboxrule') supplies framecolor, cssstyle padding (border-width
      // only when \fboxrule differs from the 0.4pt default) and the four
      // pad* Dimension properties for size computation. This REPLACES the
      // old hand-rolled block (including the faithful mirror of Perl's
      // `$sep ne '3.0pt'` object-stringification bug — upstream fixed it
      // via this refactor). Width/align keep their args, with align
      // defaulting to 'c' (center) when a width is given.
      let mut props = framed_properties(FramedOptions {
        margin: Some("\\fboxsep".to_string()),
        rule: Some("\\fboxrule".to_string()),
        ..FramedOptions::default()
      });
      // A typed width, as `\@makebox`'s: the size code reads only a Dimension
      // (an attribute string was ignored, so `\framebox[1em]` measured its
      // content plus padding, 12.08pt; latex.ltx:16196-16229 puts the frame
      // inside the width; witness: the bxcalc manual, batch 56kf). The template
      // renders it as before.
      let mut has_width = false;
      if let Some(ref dim_d) = args[0]
        && let DigestedData::RegisterValue(v) = dim_d.data()
      {
        let dim: Dimension = v.into();
        props.insert("width", Stored::from(dim));
        has_width = true;
      }
      let mut align_str = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
      if align_str.is_empty() && has_width {
        align_str = "c".to_string();
      }
      let align = makebox_alignment(&align_str);
      if !align.is_empty() {
        props.insert("align", Stored::from(align));
      }
      Ok(props)
    },
    after_digest => sub[whatsit] {
      let wasmath = lookup_bool("FRAME_IN_MATH");
      let arg = whatsit.get_arg(3).cloned();
      end_mode("restricted_horizontal")?;
      if wasmath
        && let Some(ref a) = arg {
          // Perl: $arg->isMath checks mode property =~ /math$/
          // For \fbox{$...$}, the body is a List in restricted_horizontal mode
          // containing a math whatsit. Check if any child has isMath.
          let is_math = a.get_property_bool("isMath")
            || a.unlist().iter().any(|child| child.get_property_bool("isMath"));
          if is_math {
            whatsit.set_property("mathframe", true);
            // Extract inner body for the XMArg template
            // For \fbox{$...$}, get the math body from the inner whatsit
            match a.get_body() { Ok(Some(body)) => {
              whatsit.set_property("inner", body);
            } _ => {
              // Fallback: use the entire arg
              whatsit.set_property("inner", a.clone());
            }}
          }
        }
    },
    after_construct => sub[document, whatsit] {
      // Perl afterConstruct: if the <ltx:text> has a single non-text child
      // that can have 'framed', unwrap the text and copy attributes to the child.
      // #2829: NOT when an explicit width was given — \framebox shouldn't
      // lose its width (the child can't carry it).
      if whatsit.get_property("width").is_some() {
        return Ok(());
      }
      let current = document.get_node().clone();
      if let Some(node) = current.get_last_child() {
        if document::get_node_qname(&node) != pin!("ltx:text") {
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
  // `\linewidth\hsize` appended to Perl's register trio (L4746) — same
  // intentional divergence as the {minipage} binding: real LaTeX \@iiiparbox
  // runs \@parboxrestore, so nested raw-loaded boxes read the reduced
  // \linewidth. See the minipage after_digest_begin note. Its interline
  // values too (latex.ltx:16284-16286), restored inside a table cell where
  // `\@array` zeroed them (60l, `\lx@begin@alignment`).
  // The `\ifx.#2.` dispatch closes BEFORE the box body is digested
  // (`\@firstoftwo`/`\@secondoftwo`): real `\@iiiparbox` wraps the body in
  // no conditional, so a body that leaves an `\if` open (jourcl.cls:145
  // `\RecommendedPerson` — three `\if`, two `\fi`; pdflatex only warns
  // "incomplete \ifx") must not meet the wrapper's own `\else` — Perl's
  // form (:4748) raised `Extra \else` (SHARED). Guard:
  // `perfect_kernel_batch56::parbox_body_dangling_conditional_is_not_the_wrappers`.
  DefMacro!(
    "\\parbox[] [] [] {SetlengthDimension}{}",
    r"\lx@hidden@bgroup\hsize=#4\textwidth\hsize\columnwidth\hsize\linewidth\hsize\parindent\z@\parskip\z@skip\lx@restore@interline\ifx.#2.\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{\lx@parbox[#1]{#4}{#5}}{\lx@parbox[#1][#2][#3]{#4}{#5}}\lx@hidden@egroup"
  );
  DefConstructor!("\\lx@parbox[][TempboxaDimension] OptionalUndigested {Dimension} VBoxContents",
    sub[document, args, props] {
      let body = args[4].as_ref().unwrap();
      let mut attr = string_map!("class" => "ltx_parbox");
      if let Some(w) = props.get("width") { attr.insert("width".to_string(), w.to_attribute()); }
      if let Some(v) = props.get("vattach") { attr.insert("vattach".to_string(), v.to_string()); }
      // Between paragraphs the parbox starts one (`enter_horizontal` below).
      insert_block_in_paragraph(document, body, attr, props)?;
    },
    alias => "\\parbox",
    properties => sub[args] {
      let attachment = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      // Typed sizes, as Perl's (pool.ltxml:4758-4761): a width rendered to its
      // 0.1pt attribute and parsed back measured `\parbox{5em}` 50.0pt, not
      // 50.00008pt; the constructor renders the attribute.
      let mut props = stored_map!("vattach" => translate_attachment(&attachment));
      if let Some(width) = args[3].as_ref().and_then(|w| w.get_dimension()) {
        props.insert("width", Stored::Dimension(width));
      }
      // Perl: totalheight => $_[2] — the optional [height] argument.
      if let Some(th) = args[1].as_ref().and_then(|a| a.get_dimension()) {
        props.insert("totalheight", Stored::Dimension(th));
      }
      Ok(props)
    },
    // Perl: sizer => '#5' + Box::computeSizeStore (Box.pm L267-287): size the
    // BODY through font computeBoxesSize with the whatsit's own sizing
    // properties riding in the options — `vattach` the stack split,
    // `totalheight` the final divide; the body list breaks its lines at its
    // own width (Font.pm:683, never an option's); the REQUESTED width wins
    // while computed height/depth are adopted. (The previous hand-rolled estimate here — unwrapped-width /
    // width, ceil, × baselineskip — predated the #2798 computeBoxesSize port
    // and over-counted: an fvextra breaklines one-liner measured 2
    // baselineskips, inflating every prompt-box budget ~2× into a bottom
    // whitespace river, witness 2605.00468.)
    sizer => sub[whatsit] {
      let w_req: Option<Dimension> = match whatsit.get_property("width").as_deref() {
        Some(Stored::Dimension(d)) => Some(*d),
        _ => None,
      };
      let mut opts: SymHashMap<Stored> = SymHashMap::default();
      if let Some(v) = whatsit.get_property("vattach")
        && let Stored::String(s) = &*v
      {
        opts.insert("vattach", Stored::String(*s));
      }
      if let Some(th) = whatsit.get_property("totalheight")
        && let Stored::Dimension(d) = &*th
      {
        opts.insert("totalheight", Stored::Dimension(*d));
      }
      if let Some(body) = whatsit.get_arg(5) {
        let (bw, h, d) = body.compute_size(opts)?;
        Ok((w_req.unwrap_or(bw), h, d))
      } else {
        Ok((w_req.unwrap_or_default(), Dimension::default(), Dimension::default()))
      }
    },
    // Perl latex_constructs.pool.ltxml:4763 `mode => 'inline_internal_vertical'`
    // — the INLINE flavour, as `{minipage}` below: a plain `internal_vertical`
    // made `begin_mode` end the enclosing paragraph (stomach.rs `begin_mode_opt`
    // leaves horizontal mode for any non-inline vertical mode), so a `\parbox`
    // in running text closed its `<p>` and `insert_block` chose `logical-block`
    // for a multi-paragraph body in `<para>` context — schema-invalid (the KOMA
    // letter demos; 54 documents, 381 jing lines, sweep 86). Perl keeps the
    // `<p>` and emits `inline-logical-block`.
    mode => "inline_internal_vertical",
    // latex.ltx:16249-16250 `\@iiiparbox` begins with `\leavevmode`: a parbox met between
    // paragraphs starts one, and the text after it continues that paragraph, as `\mbox` and
    // `\makebox` do (56kb). Perl's `\lx@parbox` has no enterHorizontal, so the box became a
    // block of its own and the following text a new paragraph (KNOWN_PERL_ERRORS #309,
    // DIVERGENCES #338; found by the K13 binding-conformance audit). The whatsit's
    // `in_paragraph` records where that `\leavevmode` left TeX; `insert_block_in_paragraph` opens
    // the paragraph from it — not inside a restricted box (arXiv 2605.20645's
    // `\rotatebox{90}{\parbox…}`) nor among a float's panels (2605.27134); a line of nothing but
    // boxes that are floats (`\captionof`) gives way to them once the document is built (57g). Repros
    // `tools/perfect_kernel/repros/boxes-groups/parbox_starts_the_paragraph.tex`,
    // `parbox_in_a_restricted_box_starts_no_paragraph.tex`.
    enter_horizontal => true,
    before_digest => {
      // Perl `\@parboxrestore` does `\let\\\@normalcr` (latex_dump L2310): a parbox
      // restores `\\` to the STABLE newline alias, not to the `\lx@newline` CS.
      // `\@normalcr` holds the original newline constructor directly, so it is
      // immune to `\shortstack`'s `\let\lx@newline\@shortstack@cr` rebinding. Using
      // `\lx@newline` here meant that inside `\shortstack{…\parbox{…}{… \\ …}…}`
      // the parbox's `\\` resolved to the shortstack row-break `\@shortstack@cr`,
      // which then tried to close the surrounding alignment from inside a nested
      // `itemize` → "Attempt to close a group that switched to mode … due to
      // \begin{itemize}" (witness 1904.00943). In the non-shortstack case
      // `\@normalcr` equals `\lx@newline`'s original meaning, so behavior is
      // unchanged.
      Let!("\\\\", "\\@normalcr");
    }
  );
  // INTENTIONAL DIVERGENCE from Perl (empty stub, latex_constructs.pool
  // L4767): the kernel's \@parboxrestore = \@arrayparboxrestore (see
  // latex_base.rs for the ported body and rationale — the load-bearing
  // effect is `\linewidth\hsize` for nested raw-loaded boxes) followed by
  // restoring `\\`. We restore `\\` to \@normalcr exactly as the parbox
  // constructor's before_digest does above (stable against \shortstack's
  // \lx@newline rebinding; witness 1904.00943).
  TeX!(r"\def\@parboxrestore{\@arrayparboxrestore\let\\\@normalcr}");

  DefConditional!("\\if@minipage");
  def_macro_noop("\\@setminipage")?;
  // Perl: latex_constructs.pool.ltxml lines 4769-4796 (DefEnvironment at 4771)
  DefEnvironment!("{minipage}[] OptionalUndigested [] {SetlengthDimension}",
    sub[document, args, props] {
      let attachment = args
        .first()
        .and_then(|a| a.as_ref())
        .map(|a| a.to_string())
        .unwrap_or_default();
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
        // Between paragraphs the minipage starts one (`enter_horizontal` below).
        insert_block_in_paragraph(document, body, attr, props)?;
      }
      Ok(())
    },
    // Perl #2798: minipage is an inline block — internal_vertical, no leaveHorizontal.
    mode => "inline_internal_vertical",
    // latex.ltx:16305-16306 `\@iiiminipage` begins with `\leavevmode`, as `\@iiiparbox` does
    // (57d): a minipage met between paragraphs starts one, and the text after it continues that
    // paragraph. Perl's `{minipage}` (latex_constructs.pool.ltxml:4771) has no enterHorizontal
    // (KNOWN_PERL_ERRORS #309, DIVERGENCES #338). Repro
    // `tools/perfect_kernel/repros/boxes-groups/minipage_starts_the_paragraph.tex`.
    enter_horizontal => true,
    before_digest => {
      digest(Tokens!(T_CS!("\\@minipagetrue")))?;
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
        assign_register("\\hsize", rv.clone(), None, Vec::new())?;
        assign_register("\\textwidth", rv.clone(), None, Vec::new())?;
        assign_register("\\columnwidth", rv.clone(), None, Vec::new())?;
        // INTENTIONAL DIVERGENCE from Perl (which assigns only the trio
        // above): real LaTeX \@iiiminipage follows `\hsize#3 \textwidth\hsize
        // \columnwidth\hsize` with \@parboxrestore, whose `\linewidth\hsize`
        // is what raw-loaded packages read back. tcolorbox wraps box content
        // in \minipage (tcb@lrbox), and a NESTED tcolorbox takes
        // `width=\linewidth` — with \linewidth stale at the page width, the
        // inner box drew itself full-outer-width, overflowing the parent
        // frame (arXiv 2605.02240; pdflatex INNER linewidth=282.40pt vs
        // stale 345.0pt). See OXIDIZED_DESIGN.
        assign_register("\\linewidth", rv, None, Vec::new())?;
        whatsit.set_property("width", Stored::Dimension(dim));
      }
      // `\\@parboxrestore`'s interline values (latex.ltx:16284-16286): lines set as the document's, though an
      // enclosing `\\@array` zeroed `\\baselineskip` (`\\lx@begin@alignment`).
      digest(Tokens!(T_CS!("\\lx@restore@interline")))?;
      whatsit.set_property("vattach", Stored::from(vattach.to_string()));
      Let!("\\\\", "\\lx@newline");
    },
    after_digest_body => sub[whatsit] {
      // Perl: afterDigestBody copies vattach from whatsit to body
      if let Some(vattach) = whatsit.get_property("vattach").map(|v| v.into_owned())
        && let Some(Stored::Digested(body)) = whatsit.properties.get("body").cloned() {
          let mut body = body;
          body.set_property("vattach", vattach);
        }
    }
  );

  // The box takes its size from Dimensions, as Perl stores them (latex_constructs
  // .pool.ltxml:4797-4799). The attribute strings were stored instead, so the
  // box measured 0×0: bfhsciposter.cls:171 `\hbox_gset:Nn … {\rule{0pt}{1cm}}`
  // then `\box_gresize_to_ht_plus_dp` divided by 0 and raised 11 l3 `\???`
  // errors (bfh-ci DEMO-BFHSciPoster). The raise follows latex.ltx:16360-16368
  // `\@rule`: height h + raise, depth -raise, each floored at 0 by hpack
  // (tex.web §653; Perl ignores the raise; DIVERGENCES #298). Guard:
  // `perfect_kernel_batch56::rule_box_has_its_size`.
  DefConstructor!("\\rule[SetlengthDimension]{SetlengthDimension}{SetlengthDimension}",
    "<ltx:rule ?#offset(yoffset='#offset') width='#rwidth' height='#rheight'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let dim = |i: usize| args[i].as_ref().and_then(|a| a.get_dimension()).unwrap_or_default();
      let (raise, width, height) = (dim(0), dim(1), dim(2));
      let depth = Dimension::new((-raise.value_of()).max(0));
      Ok(stored_map!(
        "offset" => args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "rwidth" => args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "rheight" => args[2].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "width" => width,
        "height" => Dimension::new((height.value_of() + raise.value_of()).max(0)),
        "depth" => depth
      ))
    }
  );
  // Perl latex_constructs.pool.ltxml:4800-4802: `\raisebox` has NO
  // beforeDigest — the outer T_MATH binding persists — and enterHorizontal => 1
  // (latex.ltx:16373-16374 `\leavevmode`).
  DefConstructor!("\\raisebox{SetlengthDimension}[TempboxaDimension][TempboxaDimension] HBoxArgContents",
    "<ltx:text yoffset='#1' _noautoclose='1'>#4</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true, bounded => true,
    // Perl: sizer => raisedSizer($_[0]->getArg(4), $_[0]->getArg(1)). latex.ltx
    // `\@irsbox`/`\@iirsbox` (16378-16393) then set `\ht` to [height] and `\dp`
    // to [depth] when given, which Perl's sizer omits; they are read with the
    // box set, `\height` etc. measuring it (`TempboxaDimension`; siamart's
    // `\raisebox{0pt}[\height][0pt]`, arXiv 2605.00332, 2605.01276, 2605.00859).
    // The raise is read so too in latex.ltx (`\raisebox{-.5\height}{icon}`,
    // 2605.18894, 2605.03941), but not yet here: its `yoffset` renders as
    // `position:relative` (LaTeXML-common.xsl), which reserves no space, so a
    // true -½-height raise overflows into the next table row. RED repro
    // `raisebox_raise_measures_the_box.tex`.
    sizer => sub[whatsit] {
      let y = whatsit.get_arg(1).and_then(|a| a.get_dimension()).map_or(0, |d| d.value_of());
      let (w, h, d) = crate::tex_kern::raised_sizer(whatsit.get_arg(4), y)?;
      let height = whatsit.get_arg(2).and_then(|a| a.get_dimension()).unwrap_or(h);
      let depth = whatsit.get_arg(3).and_then(|a| a.get_dimension()).unwrap_or(d);
      Ok((w, height, depth))
    }
  );

  // Perl: latex_constructs.pool.ltxml L4857 — \@finalstrut emits a
  // zero-dimension strut taking depth from box #1. Used by tabular-cell
  // end-of-row spacing (\@arstrutbox + \@finalstrut\@arstrutbox idiom).
  DefMacro!(
    "\\@finalstrut{}",
    r"\unskip\ifhmode\nobreak\fi\vrule\@width\z@\@height\z@\@depth\dp#1"
  );

  Ok(())
}
