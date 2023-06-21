use crate::package::*;
use once_cell::sync::Lazy;

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
  //======================================================================
  // Alignments
  //
  // & gives an error except within the right context
  // (which should redefine it!)
  DefConstructor!("&", sub[doc,_a] { Error!("unexpected", "&", doc, "Stray alignment \"&\""); });

  //**********************************************************************
  // Plain;  Extracted from Appendix B.
  //**********************************************************************
  //
  //======================================================================
  // TeX Book, Appendix B, p. 344
  //======================================================================
  RawTeX!(r"\outer\def^^L{\par}");
  DefMacro!("\\dospecials", r"\do\ \do\\\do\{\do\}\do\$\do\&\do\#\do\^\do\^^K\do\_\do\^^A\do\%\do\~");

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

  DefConstructor!("\\@ASSERT@MEANING{}{}", "#2",
    reversion      => "#2",
    after_construct => sub[document,whatsit] {
      let node    = document.get_node().clone(); // This should be the wrapper just added.
      let meaning = whatsit.get_arg(1).unwrap().to_string();
      add_meaning_rec(document, node, meaning)?;
    });

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
    let qn = model!().get_node_qname(node_ref);
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
         if state::model.with_node_qname(node_mut, |qname| qname != "ltx:XMTok")
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

  //======================================================================
  // TeX Book, Appendix B, p. 347
  DefPrimitive!("\\wlog{}", sub[(arg)] {
    let mut gullet = gullet_mut!();
    let message = Expand!(arg,gullet);
    eprintln!("{message}");
    Ok(Vec::new())
  }, locked => true);
  // From plain.tex
  DefPrimitive!("\\newcount Token", sub[ (name)] {
    DefRegister!(name, None, Number::new(0), allocate=>"\\count");
  });
  DefPrimitive!("\\newdimen Token", sub[ (name)] {
    DefRegister!(name, None, Dimension::new(0), allocate=>"\\dimen");
  });
  DefPrimitive!("\\newskip Token", sub[ (name)] {
    DefRegister!(name, None, Glue::new(0), allocate=>"\\skip");
  });
  DefPrimitive!("\\newmuskip Token", sub[ (name)] {
    DefRegister!(name, None, MuGlue::new(0), allocate=>"\\muskip");
  });
  DefPrimitive!("\\newtoks Token", sub[ (name)] {
    DefRegister!(name, None, Tokens!(), allocate=>"\\toks");
  });

  AssignValue!("allocated_boxes" => false);
  DefPrimitive!("\\newbox DefToken", sub[ (t)] {
    let n = state!().lookup_int("allocated_boxes");
    AssignValue!("allocated_boxes" => n + 1, Some(Scope::Global));
    let empty_list = List::new(Vec::new());
    AssignValue!(&s!("box{}",n), empty_list);
    DefRegister!(t, None, Number(n));
  });
  DefPrimitive!("\\newhelp Token {}", sub[(token,arg)] {
    state_mut!().assign_value(&token.to_string(), arg, None);
  });
  DefPrimitive!("\\newtoks Token", sub[(name)] {
    DefRegister!(name, None, Tokens!(), allocate=>"\\toks");
  });

  // the next 4 actually work by doing a \chardef instead of \countdef, etc.
  // which means they actually work quite differently
  DefRegister!("\\allocationnumber" => Number::new(0));
  DefPrimitive!("\\alloc@@ {}", sub[ (atype)] {
    let c = s!("allocation @{}", atype);
    let n = LookupRegisterOrDefault!(&c).value_of();
    state_mut!().assign_value(&c, n + 1, Some(Scope::Global));
    state_mut!().assign_register("\\allocationnumber", Number::new(n).into(), Some(Scope::Global), Vec::new())?;
    Ok(Vec::new())
  });
  DefMacro!(
    "\\newread Token",
    r"\alloc@@{read}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newwrite Token",
    r"\alloc@@{write}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newfam Token",
    r"\alloc@@{fam}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newlanguage Token",
    r"\alloc@@{language}\global\chardef#1=\allocationnumber"
  );
  DefMacro!("\\e@alloc{}{}{}{}{}{}",
  r"\global\advance#3\@ne
  \allocationnumber#3\relax
  \global#2#6\allocationnumber");
  DefMacro!("\\alloc@{}{}{}{}", r"\e@alloc#2#3{\count1#1}#4\float@count");
  DefMacro!("\\newread",        r"\e@alloc\read \chardef{\count16}\m@ne\sixt@@n");
  DefMacro!("\\newwrite", r"\e@alloc\write
                    {\ifnum\allocationnumber=18
                      \advance\count17\@ne
                      \allocationnumber\count17 %
                      \fi
                      \global\chardef}%
                    {\count17}%
                    \m@ne
                    {128}");

  // This implementation is quite wrong
  DefPrimitive!("\\newinsert Token", sub[ (t)] {
    DefRegister!(t, None, Number::new(0));
  });
  // \alloc@, \ch@ck

  // TeX plain uses \newdimen, etc. for these.
  // Is there any advantage to that?
  // note: rust complains about the 16_383.99999 having excessive precision, hence simplifying
  DefRegister!("\\maxdimen", Dimension::new(16_384 * 65536));
  // DefRegister!("\\hideskip", Glue!(-1000 * 65536, "1fill"));
  DefRegister!("\\centering", Glue!("0pt plus 1000pt minus 1000pt"));
  DefRegister!("\\p@", Dimension::new(65536));
  DefRegister!("\\z@", Dimension::new(0));
  DefRegister!("\\z@skip", Glue::new(0));

  // First approximation. till I figure out \newbox
  // RawTeX('\newbox\voidb@x');
  //======================================================================
  // TeX Book, Appendix B, p. 348

  DefMacro!("\\newif DefToken", sub[ (cs)] {
    def_conditional(cs, None,None,ConditionalOptions::default(),gullet)
  });

  // See the section Registers & Parameters, above for setting default values.
  //======================================================================
  // TeX Book, Appendix B, p. 349
  // See the section Registers & Parameters, above for setting default values.

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
  DefMacro!("\\magstep{}", sub[ (mag)] {
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
    unimplemented!();
    Ok(())
}
