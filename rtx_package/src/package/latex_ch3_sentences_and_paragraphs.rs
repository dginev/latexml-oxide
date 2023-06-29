//**********************************************************************
// C.3. Sentences and Paragraphs
//**********************************************************************
use crate::package::*;

LoadDefinitions!({
  //======================================================================
  // C.3.1 Making Sentences
  //======================================================================
  // quotes;  should these be handled in DOM/construction?
  // dashes:  We'll need some sort of Ligature analog, or something like
  // Omega's OTP, to combine sequences of "-" into endash, emdash,
  // Perhaps it also applies more semantically?
  // Such as interpreting certain sequences as section headings,
  // or math constructs.

  // Spacing; in TeX.pool.ltxml

  // Special Characters; in TeX.pool.ltxml

  // Logos
  // \TeX is in TeX.pool.ltxml
  DefMacro!("\\LaTeX", "LaTeX");
  DefMacro!("\\LaTeXe", "LaTeX2e");
  DefConstructor!("\\LaTeX",
  r###"<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'>L<ltx:text
  fontsize='80%' yoffset='0.4ex'>A</ltx:text>T<ltx:text
  yoffset='-0.4ex'>E</ltx:text>X</ltx:text>"###,
  sizer => { Ok((Dimension!("2.6em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefConstructor!("\\LaTeXe",
  "<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'>L<ltx:text
  fontsize='80%' yoffset='0.4ex'>A</ltx:text>T<ltx:text
  yoffset='-0.4ex'>E</ltx:text>X\u{2002}2<ltx:text yoffset='-0.4em'>\u{03B5}</ltx:text></ltx:text>",
  sizer => { Ok((Dimension!("3.7em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefMacro!("\\fmtname", "LaTeX2e");
  DefMacro!("\\fmtversion", "2018/12/01");

  DefMacro!("\\today", { ExplodeText!(Today!()) });

  // Use fonts (w/ special flag) to propogate emphasis as a font change,
  // but preserve it's "emph"-ness.
  DefConstructor!("\\emph{}", "<ltx:emph _force_font='1'>#1",
    mode => "text",
    bounded        => true,
    font=> { emph => true },
    alias => "\\emph",
    before_digest => {
      if Expand!(T_CS!("\\f@shape")).to_string() == "it" {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("n")));
      } else {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("i"),T_LETTER!("t")));
      }
    },
    after_construct => sub[doc,_args] {
      doc.maybe_close_element("ltx:emph")?; }
  );

  //======================================================================
  // C.3.2 Making Paragraphs
  //======================================================================
  // \noindent, \indent, \par in TeX.pool.ltxml

  Let!("\\@@par", "\\par");
  DefMacro!("\\@par", r"\let\par\@@par\par");
  DefMacro!("\\@restorepar", r"\def\par{\@par}");

  // Style parameters
  // \parindent, \baselineskip, \parskip alreadin in TeX.pool.ltxml

  DefPrimitive!("\\linespread{}", None);

  // ?
  DefMacro!("\\@noligs", "");
  DefConditional!("\\if@endpe");
  DefMacro!("\\@doendpe", "");
  DefMacro!("\\@bsphack", "\\relax"); // what else?
  DefMacro!("\\@esphack", "\\relax");
  DefMacro!("\\@Esphack", "\\relax");

  //======================================================================
  // C.3.3 Footnotes
  //======================================================================

  NewCounter!("footnote");
  DefMacro!("\\thefootnote", "\\arabic{footnote}");
  NewCounter!("mpfootnote");
  DefMacro!("\\thempfn", "\\thefootnote");
  DefMacro!("\\thempfootnote", "\\arabic{mpfootnote}");
  DefMacro!("\\footnotetyperefname", "footnote");

  DefMacro!("\\ext@footnote", None);
  DefConstructor!("\\lx@note[]{}[]{}",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags#4</ltx:note>",
  mode         => "text", bounded => true,
  before_digest => {
    reenter_text_mode(true);
    neutralize_font(); },
  properties   => sub [args] {
    let arg1 = &args[0];
    let arg2 = &args[1];
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = arg2.as_ref().map(ToString::to_string).unwrap_or_default();
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("list".to_string(), digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    props.insert("role".to_string(), note_type.into());
    Ok(props)
  },
  reversion => "");

  DefConstructor!("\\lx@notemark[]{}[]",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags</ltx:note>",
  mode       => "text",
  properties => sub[args] {
    let arg1 = &args[0];
    let arg2 = &args[1];
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = arg2.as_ref().map(ToString::to_string).unwrap_or_default();
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("role".to_string(), s!("{note_type}mark").into());
    props.insert("list".to_string(), digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    Ok(props)
  },
  reversion => "");

  DefConstructor!("\\lx@notetext[]{}[]{}",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id'>#4</ltx:note>",
  mode       => "text",
  properties => sub [args] {
    let arg1 = &args[0];
    let arg2 = &args[1];
    let arg3 = &args[2];
    let note_type = arg2.as_ref().map(ToString::to_string).unwrap_or_default();
    let arg3_ready = if let Some(v) = arg3 { Cow::Borrowed(v) } else {
      Cow::Owned(
        stomach::digest(T_CS!(s!("\\the{note_type}")))?
      )
    };
    let mut props = make_note_tags(&note_type, arg1, Some(arg3_ready))?;
    props.insert("role".to_string(), s!("{note_type}text").into());
    Ok(props)
  },
  reversion => "");

  DefMacro!("\\footnote",      "\\lx@note{footnote}",     locked => true);
  DefMacro!("\\footnotemark",  "\\lx@notemark{footnote}", locked => true);
  DefMacro!("\\footnotetext",  "\\lx@notetext{footnote}", locked => true);
  DefMacro!("\\@footnotetext", "\\lx@notetext{footnote}", locked => true);
  // we don't implement the internals directly, so lock them to the latexml variant
  Let!("\\@thefnmark", "\\lx@notemark{footnote}");

  Tag!("ltx:note", after_close => sub[doc, node] { relocate_footnote(doc, node)?; });

  // Style parameters
  DefRegister!("\\footnotesep" => Dimension::new(0));
  DefPrimitive!("\\footnoterule", None);
});
