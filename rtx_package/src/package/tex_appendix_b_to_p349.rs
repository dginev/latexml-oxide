use crate::package::*;
use lazy_static::lazy_static;
lazy_static! {
  static ref NAMED_SPACE_CHARS: HashMap<&'static str, &'static str> = static_map!("negthinspace" => "", "thinspace" => "\u{2009}",
      "medspace" => "\u{2005}", "thickspace" => "\u{2004}", "space" => " ");
  static ref DECIMAL_SEP: HashMap<&'static str, &'static str> =
    static_map!("en" => ".", "de" => ",", "fr" => ",", "nl" => ",", "pt" => ",", "es" => ",");
  static ref THOUSANDS_SEP: HashMap<&'static str, &'static str> =
    static_map!("en" => ",", "de" => ".", "fr" => ".", "nl" => ".", "pt" => ".", "es" => ".");
}

LoadDefinitions!(state, {
  //**********************************************************************
  // Plain;  Extracted from Appendix B.
  //**********************************************************************
  //
  //======================================================================
  // TeX Book, Appendix B, p. 344
  //======================================================================
  // \dospecials ??
  //
  // Normally, the content branch contains the pure structure and meaning of a construct,
  // and the presentation is generated from lower level TeX macros that only concern
  // themselves with how to display the object.
  // Nevertheless, it is sometimes useful to know where the tokens in the presentation branch
  // came from;  particularly what their presumed "meaning" is.
  // For example, when search-indexing pmml, or providing links to definitions from the pmml.
  //
  // The following constructor (see how it's used in DefMath), adds meaning attributes
  // whereever it seems sensible on the presentation branch, after it has been generated.

  // DefConstructor('\@ASSERT@MEANING{}{}', '#2',
  //   reversion      => '#2',
  //   afterConstruct => sub {
  //     my ($document, $whatsit) = @_;
  //     let node    = $document->getNode;              # This should be the wrapper just added.
  //     my $meaning = ToString($whatsit->getArg(1));
  //     addMeaningRec($document, $node, $meaning);
  //     $node; });

  //======================================================================
  // Properties for plain characters.
  // These are allowed in plain text, but need to act a bit special in math.
  DefMath!('=', None, '=', role => "RELOP",   meaning  => "equals");
  DefMath!('+', None, '+', role => "ADDOP",   meaning  => "plus");
  DefMath!('-', None, '-', role => "ADDOP",   meaning  => "minus");
  DefMath!('*', None, '*', role => "MULOP",   meaning  => "times");
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

  DefMathLigature!(matcher => sub[document, node, state] {
  let lang = document.get_node_language(node);
  let lang = lang.split('-').next().unwrap(); // strip off region code, if any.
  let dec     = DECIMAL_SEP.get(lang).unwrap_or(&".");
  let thou    = THOUSANDS_SEP.get(lang).unwrap_or(&",");
  let decrole = if dec == &"." { "PERIOD" } else { "" };
  // let mut chars : Vec<char> = Vec::new();
  let (mut n, mut combined, mut number, w, mut font) = (0, String::new(), String::new(), 0, None);
  //     NOTE: We're scanning chars from END!
  let mut node_ref = node;
  let mut current;
  loop {
    let qn = state.model.get_node_qname(node_ref);
    if qn == "ltx:XMTok" || qn == "ltx:XMWrap" {
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
    } else if qn == "ltx:XMHint" {
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
  // DefRewrite(select => ['descendant-or-self::ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . '[ following-sibling::*[1][self::ltx:XMApp]'
  //       . ' [child::*[1][self::ltx:XMTok[@meaning="divide"]]]'
  //       . ' [child::*[2]['
  //       . 'self::ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . 'or self::ltx:XMArg[count(child::*)=1]/ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . ']]'
  //       . ' [child::*[3]['
  //       . 'self::ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . 'or self::ltx:XMArg[count(child::*)=1]/ltx:XMTok[@role="NUMBER" and translate(@meaning,"0123456789","")=""]'
  //       . ']]'
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

  // Recognize !!
  DefMathLigature!("!!", "!!", role => "POSTFIX", meaning => "double-factorial");
  // Recognize :=
  DefMathLigature!(":=", ":=", role => "RELOP", meaning => "assign");

  //======================================================================
  // Combine letters, when the fonts are right. (sorta related to mathcode)
  // well, maybe a letter followed by letters & digits?
  DefMathLigature!(matcher => sub [document,node_opt,state] {
    //  let mut chars :Vec<char> = Vec::new();
     let font  = document.get_node_font(node_opt);
     let mut this_node;
     let mut node_mut = node_opt;
     if font.is_sticky() {
       let mut n      = 0;
       let mut text = String::new();
       loop {
         if state.model.get_node_qname(node_mut) != "ltx:XMTok"
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
  // TeX Book, Appendix B, p. 345

  RawTeX!(
    r###"
    \chardef\active=13
    \chardef\@ne=1
    \chardef\tw@=2
    \chardef\thr@@=3
    \chardef\sixt@@n=16
    \chardef\@cclv=255
    \mathchardef\@cclvi=256
    \mathchardef\@m=1000
    \mathchardef\@M=10000
    \mathchardef\@MM=20000
    \countdef\m@ne=21\relax
    \m@ne=-1
  "###
  );

  //======================================================================
  // TeX Book, Appendix B, p. 346
  RawTeX!(
    r###"
  \countdef\count@=255
  \toksdef\toks@=0
  \skipdef\skip@=0
  \dimendef\dimen@=0
  \dimendef\dimen@i=1
  \dimendef\dimen@ii=2
  \count10=22
  \count11=9
  \count12=9
  \count13=9
  \count14=9
  \count15=9
  \count16=-1
  \count17=-1
  \count18=3
  \count19=0
  \count20=255
  \countdef\insc@unt=20
  \countdef\allocationnumber=21
  \countdef\m@ne=22 \m@ne=-1
  "###
  );
  // Various \count's are set; should we?

  // #======================================================================
  // # TeX Book, Appendix B, p. 347
  // # \wlog ??
  // # From plain.tex
  DefPrimitive!("\\newcount Token", sub[stomach, (name), state] {
    DefRegister!(name, None, Number::new(0));
  });
  DefPrimitive!("\\newdimen Token", sub[stomach, (name), state] {
    DefRegister!(name, None, Dimension::new(0));
  });
  DefPrimitive!("\\newskip Token", sub[stomach, (name), state] {
    DefRegister!(name, None, Glue::new(0));
  });
  DefPrimitive!("\\newmuskip Token", sub[stomach, (name), state] {
    DefRegister!(name, None, MuGlue::new(0));
  });
  AssignValue!("allocated_boxes" => false);
  DefPrimitive!("\\newbox Token", sub[stomach, (t), state] {
    let n = state.lookup_int("allocated_boxes");
    AssignValue!("allocated_boxes" => n + 1, Some(Scope::Global));
    let empty_list = List::new(Vec::new(), state);
    AssignValue!(&s!("box{}",n), empty_list);
    DefRegister!(t, None, Number(n));
  });
  // DefPrimitive('\newhelp Token {}', sub { AssignValue(ToString($_[1]) => $_[2]); });
  // DefPrimitive('\newtoks Token', sub { DefRegisterI($_[1], undef, Tokens()); });
  // # the next 4 actually work by doing a \chardef instead of \countdef, etc.
  // # which means they actually work quite differently
  DefRegister!("\\allocationnumber" => Number::new(0));
  DefMacro!("\\alloc@@ {}", sub[gullet, (atype_tokens), state] {
    let atype = atype_tokens.to_string();
    let c = s!("allocation @{}", atype);
    let n = LookupRegisterOrDefault!(c).value_of();
    AssignValue!(&c                  => n + 1,     Some(Scope::Global));
    AssignValue!("\\allocationnumber" => Number!(n), Some(Scope::Global));
  });
  DefMacro!("\\newread Token", r"\alloc@@{read}\global\chardef#1=\allocationnumber");
  DefMacro!("\\newwrite Token", r"\alloc@@{write}\global\chardef#1=\allocationnumber");
  DefMacro!("\\newfam Token", r"\alloc@@{fam}\global\chardef#1=\allocationnumber");
  DefMacro!("\\newlanguage Token", r"\alloc@@{language}\global\chardef#1=\allocationnumber");

  // # This implementation is quite wrong
  DefPrimitive!("\\newinsert Token", sub[stomach, args, state] {
    unpack_to_token!(args => t);
    DefRegister!(t, None, Number::new(0));
  });
  // # \alloc@, \ch@ck

  // TeX plain uses \newdimen, etc. for these.
  // Is there any advantage to that?
  // note: rust complains about the 16_383.99999 having excessive precision, hence simplifying
  DefRegister!("\\maxdimen", Dimension::new(16_384 * 65536));
  // DefRegister!("\\hideskip", Glue!(-1000 * 65536, "1fill"));
  DefRegister!("\\centering", Glue!("0pt plus 1000pt minus 1000pt"));
  DefRegister!("\\p@", Dimension::new(65536));
  DefRegister!("\\z@", Dimension::new(0));
  DefRegister!("\\z@skip", Glue::new(0));

  // # First approximation. till I figure out \newbox
  // RawTeX('\newbox\voidb@x');
  // #======================================================================
  // # TeX Book, Appendix B, p. 348

  DefMacro!("\\newif DefToken", sub[gullet, args, state] {
    unpack_to_token!(args => cs);
    def_conditional(cs, None,None,ConditionalOptions::default(),gullet,state);
  });

  // # See the section Registers & Parameters, above for setting default values.
  // #======================================================================
  // # TeX Book, Appendix B, p. 349
  // # See the section Registers & Parameters, above for setting default values.

  // These are originally defined with \newskip, etc
  DefRegister!("\\smallskipamount", Glue!("3pt plus1pt minus1pt"));
  DefRegister!("\\medskipamount", Glue!("6pt plus2pt minus2pt"));
  DefRegister!("\\bigskipamount", Glue!("12pt plus4pt minus4pt"));
  DefRegister!("\\normalbaselineskip", Glue!("12pt"));
  DefRegister!("\\normallineskip", Glue!("1pt"));
  DefRegister!("\\normallineskiplimit", Dimension!("0pt"));
  DefRegister!("\\jot", Dimension!("3pt"));
  DefRegister!("\\lx@default@jot", LookupRegister!("\\jot"));
  DefRegister!("\\interdisplaylinepenalty", Number(100));
  DefRegister!("\\interfootnotelinepenalty", Number(100));

  DefMacro!("\\magstephalf", "1095");
  DefMacro!("\\magstep{}", sub[gullet, (mag), state] {
    Explode!(match mag.to_string().as_str() {
      "0" => "1000",
      "1" => "1200",
      "2" => "1440",
      "3" => "1728",
      "4" => "2074",
      "5" => "2488",
      _ => ""
    })
  });

  // #======================================================================
  // # TeX Book, Appendix B, p. 350

  // # Font stuff ...
  RawTeX!(
    r###"
  \font\tenrm=cmr10
  \font\sevenrm=cmr7
  \font\fiverm=cmr5
  \font\teni=cmmi10
  \font\seveni=cmmi7
  \font\fivei=cmmi7
  \font\tensy=cmsy10
  \font\sevensy=cmsy7
  \font\fivesy=cmsy5
  \font\tenex=cmex10
  \font\tenbf=cmbx10
  \font\sevenbf=cmbx7
  \font\fivebf=cmbx5
  \font\tensl=cmsl10
  \font\tentt=cmtt10
  \font\tenit=cmti10
  \newfam\itfam
  \newfam\slfam
  \newfam\bffam
  \newfam\ttfam
  \textfont0=\tenrm\scriptfont0=\sevenrm\scriptscriptfont0=\fiverm
  \textfont1=\teni\scriptfont1=\seveni\scriptscriptfont1=\fivei
  \textfont2=\tensy\scriptfont2=\sevensy\scriptscriptfont2=\fivesy
  \textfont3=\tenex
"###
  );

  // # Note: \newfam in math should be font switching(?)

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)
  Tag!("ltx:text", auto_open => true, auto_close => true);

  // Note that these, unlike \rmfamily, should set the other attributes to the defaults!
  DefPrimitive!("\\rm", None,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefPrimitive!("\\sf", None,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefPrimitive!("\\bf", None,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefPrimitive!("\\it", None,
    font => {shape => "italic", family => "serif", series => "medium" });
  DefPrimitive!("\\tt", None,
    font => {family => "typewriter", series => "medium", shape => "upright" });
  // No effect in math for the following 2 ?
  DefPrimitive!("\\sl", None,
    font => {shape => "slanted", family => "serif", series => "medium" });
  DefPrimitive!("\\sc", None,
    font => {shape => "smallcaps", family => "serif", series => "medium" });

  // Ideally, we should set these sizes from class files
  AssignValue!("NOMINAL_FONT_SIZE", 10);
  DefPrimitive!("\\tiny",         None, font => {size => 5 });
  DefPrimitive!("\\scriptsize",   None, font => {size => 7 });
  DefPrimitive!("\\footnotesize", None, font => {size => 8 });
  DefPrimitive!("\\small",        None, font => {size => 9 });
  DefPrimitive!("\\normalsize",   None, font => {size => 10 });
  DefPrimitive!("\\large",        None, font => {size => 12 });
  DefPrimitive!("\\Large",        None, font => {size => 14.4 });
  DefPrimitive!("\\LARGE",        None, font => {size => 17.28 });
  DefPrimitive!("\\huge",         None, font => {size => 20.74 });
  DefPrimitive!("\\Huge",         None, font => {size => 29.8 });

  DefPrimitive!("\\mit", None, require_math => true, font => {family => "italic"});

  DefPrimitive!("\\frenchspacing", None);
  DefPrimitive!("\\nonfrenchspacing", None);
  // DefMacro!("\\normalbaselines", undef,
  //   '\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit');
  DefMacro!(T_CS!("\\space"), None, T_SPACE!());
  DefMacro!(T_CS!("\\lq"), None, T_OTHER!("`"));
  DefMacro!(T_CS!("\\rq"), None, T_OTHER!("'"));
  Let!("\\empty", "\\@empty");
  //DefMacro!("\\null", "\hbox{}");
  Let!("\\bgroup", T_BEGIN!());
  Let!("\\egroup", T_END!());
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");

  DefPrimitive!("\\endline", None);

  // Use \r for the newline from TeX!!!
  DefMacro!(T_CS!("\\\r"), None, T_CS!("\\ ")); // \<cr> == \<space> Interesting (see latex.ltx)
  Let!(&T_ACTIVE!("\r"), T_CS!("\\par")); // (or is this just LaTeX?)

  Let!("\\\t", "\\\r"); // \<tab> == \<space>, also

  //======================================================================
  // TeX Book, Appendix B, p. 352

  DefPrimitive!("\\obeyspaces", {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(" "), T_CS!("\\space"));
  });
  // Curiously enough, " " (a space) is ALREADY defined to be the same as "\space"
  // EVEN before it is made active. (see p.380)
  Let!(&T_ACTIVE!(" "), T_CS!("\\space"));

  DefPrimitive!("\\obeylines", {
    AssignCatcode!('\r', Catcode::ACTIVE);
    Let!(&T_ACTIVE!("\r"), T_CS!("\\@break")); // More appropriate than \par, I think?
  });

  DefConstructor!("\\@break", "<ltx:break/>");

  RawTeX!(
    r###"
  \def\loop#1\repeat{\def\body{#1}\iterate}
  \def\iterate{\body \let\next=\iterate \else\let\next=\relax\fi \next}
  \let\repeat=\fi
  "###
  );

  DefMacro!("\\enskip", "\\ifmmode\\@math@enskip\\else\\@text@enskip\\fi");
  // DefConstructor('\@math@enskip', undef,
  //   "<ltx:XMHint name='enskip' width='#width'/>",
  //   alias => '\enskip',
  //   properties => { isSpace => 1, width => sub { Dimension('0.5em'); } });
  // DefPrimitiveI('\@text@enskip', undef, "\x{2002}", alias => '\enskip');

  DefMacro!("\\enspace", "\\ifmmode\\@math@enspace\\else\\@text@enspace\\fi");
  // DefConstructor('\@math@enspace', undef,
  //   "<ltx:XMHint name='enskip' width='#width'/>",
  //   alias => '\enspace',
  //   properties => { isSpace => 1, width => sub { Dimension('0.5em'); } });
  // DefPrimitiveI('\@text@enspace', undef, "\x{2002}", alias => '\enspace');

  DefMacro!("\\quad", "\\ifmmode\\@math@quad\\else\\@text@quad\\fi");
  // DefConstructor('\@math@quad', undef,
  //   "<ltx:XMHint name='quad' width='#width'/>",
  //   alias => '\quad',
  //   properties => { isSpace => 1, width => sub { Dimension('1em'); } });
  // DefPrimitiveI('\@text@quad', undef, "\x{2003}", alias => '\quad');

  // # Conceivably should be treated as punctuation! (but maybe even \quad should !?!)
  DefMacro!("\\qquad", "\\ifmmode\\@math@qquad\\else\\@text@qquad\\fi");
  // DefConstructor('\@math@qquad', undef,
  //   "<ltx:XMHint name='qquad' width='#width'/>",
  //   alias => '\qquad',
  //   properties => { isSpace => 1, width => sub { Dimension('2em'); } });
  // DefPrimitiveI('\@text@qquad', undef, "\x{2003}\x{2003}", alias => '\qquad');

  DefMacro!("\\thinspace", "\\ifmmode\\@math@thinspace\\else\\@text@thinspace\\fi");
  // DefConstructor('\@math@thinspace', undef,
  //   "<ltx:XMHint name='thinspace' width='#width'/>",
  //   alias => '\thinspace',
  //   properties => { isSpace => 1, width => sub { Dimension('0.16667em'); } });
  // DefPrimitiveI('\@text@thinspace', undef, "\x{2009}", alias => '\thinspace');

  DefMacro!("\\negthinspace", "\\ifmmode\\@math@negthinspace\\else\\@text@negthinspace\\fi");
  // DefConstructor('\@math@negthinspace', undef,
  //   "<ltx:XMHint name='negthinspace' width='#width'/>",
  //   alias => '\negthinspace',
  //   properties => { isSpace => 1, width => sub { Dimension('-0.16667em'); } });
  // DefPrimitiveI('\@text@negthinspace', undef, "", alias => '\negthinspace');

  // DefConstructor('\hglue Glue', "?#isMath(<ltx:XMHint name='hglue' width='#width'/>)(\x{2003})",
  //   properties => sub { (isSpace => 1, width => $_[1]); });
  DefPrimitive!("\\vglue Glue", None);
  DefPrimitive!("\\topglue", None);
  DefPrimitive!("\\nointerlineskip", None);
  DefPrimitive!("\\offinterlineskip", None);

  DefMacro!("\\smallskip", "\\vskip\\smallskipamount");
  DefMacro!("\\medskip", "\\vskip\\medskipamount");
  DefMacro!("\\bigskip", "\\vskip\\bigskipamount");

  //======================================================================
  // TeX Book, Appendix B, p. 353

  DefPrimitive!("\\break", None);
  DefPrimitive!("\\nobreak", None);
  DefPrimitive!("\\allowbreak", None);
  DefMacro!("\\nobreakspace", "\\ifmmode\\math@nobreakspace\\else\\text@nobreakspace\\fi");

  DefPrimitive!("\\text@nobreakspace", sub[stomach, (), state] {
    Tbox::new(String::from("\u{00A0}"), None, None, Tokens!(T_CS!("~")), map!("isSpace" => Stored::Bool(true)), state)
  });

  // DefConstructor!("\\math@nobreakspace", "<ltx:XMHint name='nobreakspace' width='#width'/>",
  //   properties => { isSpace => 1, width => sub { Dimension('0.333em'); } },
  //   alias => '~');
  DefMacro!("~", "\\nobreakspace{}");

  DefMacro!("\\slash", "/");
  DefPrimitive!("\\filbreak", None);
  DefMacro!("\\goodbreak", "\\par");
  DefMacro!("\\eject", "\\par\\LTX@newpage");
  Let!("\\newpage", "\\eject");

  DefConstructor!("\\LTX@newpage", "^<ltx:pagination role='newpage'/>",
  before_digest=>sub[stomach,state] {
    state.after_assignment(stomach.get_gullet_mut());
    Ok(Vec::new())
  });
  DefMacro!("\\supereject", "\\par\\LTX@newpage");
  DefPrimitive!("\\removelastskip", None);
  DefMacro!("\\smallbreak", "\\par");
  DefMacro!("\\medbreak", "\\par");
  DefMacro!("\\bigbreak", "\\par");
  DefMacro!("\\line", "\\hbox to \\hsize");
  // DefConstructor('\leftline{}', sub {
  //     alignLine($_[0], $_[1], 'left'); },
  //   bounded => 1);
  // DefConstructor('\rightline{}', sub {
  //     alignLine($_[0], $_[1], 'right'); },
  //   bounded => 1);
  // DefConstructor('\centerline{}', sub {
  //     alignLine($_[0], $_[1], 'center'); },
  //   bounded => 1);

  // sub alignLine {
  //   my ($document, $line, $alignment) = @_;
  //   if ($document->isOpenable('ltx:p')) {
  //     $document->insertElement('ltx:p', $line, class => 'ltx_align_' . $alignment); }
  //   elsif ($document->isOpenable('ltx:text')) {
  //     $document->insertElement('ltx:text', $line, class => 'ltx_align_' . $alignment);
  //     $document->insertElement('ltx:break'); }
  //   else {
  //     $document->absorb($line); }
  //   return; }

  // # These should be 0 width, but perhaps also shifted?
  DefMacro!("\\llap{}", "\\hbox to 0pt{#1}");
  DefMacro!("\\rlap{}", "\\hbox to 0pt{#1}");
  DefMacro!("\\m@th", "\\mathsurround=0pt ");

  // # \strutbox
  DefMacro!("\\strut", "");
  RawTeX!("\\newbox\\strutbox");

  // #======================================================================
  // # TeX Book, Appendix B. p. 354

  // # TODO: Not yet done!!
  // # tabbing stuff!!!

  DefMacro!("\\settabs", "");

  // #======================================================================
  // # TeX Book, Appendix B. p. 355

  DefPrimitive!("\\hang", None);

  // # TODO: \item, \itemitem not done!
  // # This could probably be adopted from LaTeX, if the <itemize> could auto-open
  // # and close!
  DefConstructor!("\\item{}", "#1");
  DefConstructor!("\\itemitem{}", "#1");

  // DefMacro('\textindent{}', '#1');

  // # Conceivably this should enclose the next para in a block?
  // # Or add attribute to it? Or...
  // DefPrimitiveI('\narrower', undef, undef);
});
