// math_common — Common math definitions, always loaded.
// Corresponds to Perl Engine/math_common.pool.ltxml.
//
// Contains: Greek letters, symbols, operators, relations, arrows,
// delimiters, accents, log-like functions, phantoms, roots, \not handling.
//
// In Perl, this is loaded by plain_constructs.pool.ltxml which is loaded
// by latex_constructs.pool.ltxml. These definitions are always available.
use crate::prelude::*;

/// DEP-17d helper for the dominant 3-arg `DefMath!("\\cs", None, "char"…)`
/// shape (195+ entries here). `paramlist = None` matches the macro's
/// `(text, paramlist, present)` arm exactly — distinct from
/// `def_math_sym`'s 2-arg form which goes through `parse_prototype` and
/// produces `Some(empty Parameters)`. Both yield the same `nargs = 0`
/// downstream, but keeping the None form preserves struct-level parity
/// for any consumer that distinguishes.
fn def_math_atom(cs: &str, present: &str, role: Option<&str>, meaning: Option<&str>) -> Result<()> {
  let cs_tok = T_CS!(cs);
  let mut opts = MathPrimitiveOptions::default();
  if let Some(r) = role { opts.role = Some(r.to_string()); }
  if let Some(m) = meaning { opts.meaning = Some(m.to_string()); }
  def_math(cs_tok, None, present.to_string(), opts)?;
  Ok(())
}

// Match negations of many operators
// our %NOTS
static MATH_CHAR_NEGATIONS: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
  map!("=" => "\u{2260}", "<" => "\u{226E}", ">" => "\u{226F}",
  "\u{2208}" => "\u{2209}",                              //\in=>\notin
  "\u{2264}" => "\u{2270}", "\u{2265}" => "\u{2271}",    // Less eq, greater eq.
  "\u{227A}" => "\u{2280}", "\u{227B}" => "\u{2281}",    // prec, succ
  "\u{2AAF}" => "\u{22E0}", "\u{2AB0}" => "\u{22E1}",    // preceq, succeq
  "\u{2282}" => "\u{2284}", "\u{2283}" => "\u{2285}",    // subset, supset
  "\u{2286}" => "\u{2288}", "\u{2287}" => "\u{2289}",    // subseteq, supseteq
  "\u{2291}" => "\u{22E2}", "\u{2290}" => "\u{22E3}",    // sqsubseteq, sqsupseteq
  "\u{2261}" => "\u{2262}",                              // equiv
  "\u{224D}" => "\u{226D}", "\u{2248}" => "\u{2249}",    // asymp, approx
  "\u{22B2}" => "\u{22EA}", "\u{22B3}" => "\u{22EB}",    // lhd, rhd
  "\u{22B4}" => "\u{22EC}", "\u{22B5}" => "\u{22ED}",    // unlhd, unrhd
  "\u{2203}" => "\u{2204}"                              // Exists
  )
});

/// Delimiter char properties for augmenting delimiter elements.
/// Mirrors Perl's %DELIMITER_MAP (TeX_Math.pool.ltxml L732-823).
struct DelimCharProps {
  left_role:      &'static str,
  right_role:     &'static str,
  name:           Option<&'static str>,
  remove_meaning: bool,
  replace_char:   Option<char>,
}

/// Char-keyed delimiter map for augmentDelimiterProperties lookup.
static DELIM_CHAR_MAP: Lazy<HashMap<char, DelimCharProps>> = Lazy::new(|| {
  raw_map!(
    '(' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    ')' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '[' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    ']' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '{' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: None, remove_meaning: false, replace_char: None },
    '\u{230A}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("lfloor"), remove_meaning: false, replace_char: None },
    '\u{230B}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rfloor"), remove_meaning: false, replace_char: None },
    '\u{2308}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("lceil"), remove_meaning: false, replace_char: None },
    '\u{2309}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rceil"), remove_meaning: false, replace_char: None },
    '\u{27E8}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("langle"), remove_meaning: false, replace_char: None },
    '\u{27E9}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rangle"), remove_meaning: false, replace_char: None },
    '<' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("langle"), remove_meaning: true, replace_char: Some('\u{27E8}') },
    '>' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("rangle"), remove_meaning: true, replace_char: Some('\u{27E9}') },
    '/' => DelimCharProps { left_role: "MULOP", right_role: "MULOP",
      name: None, remove_meaning: false, replace_char: None },
    '\u{005C}' => DelimCharProps { left_role: "MULOP", right_role: "MULOP",
      name: Some("backslash"), remove_meaning: false, replace_char: None },
    '|' => DelimCharProps { left_role: "VERTBAR", right_role: "VERTBAR",
      name: None, remove_meaning: false, replace_char: None },
    '\u{2225}' => DelimCharProps { left_role: "VERTBAR", right_role: "VERTBAR",
      name: None, remove_meaning: false, replace_char: None },
    '\u{2191}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("uparrow"), remove_meaning: false, replace_char: None },
    '\u{21D1}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("Uparrow"), remove_meaning: false, replace_char: None },
    '\u{2193}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("downarrow"), remove_meaning: false, replace_char: None },
    '\u{21D3}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("Downarrow"), remove_meaning: false, replace_char: None },
    '\u{2195}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("updownarrow"), remove_meaning: false, replace_char: None },
    '\u{21D5}' => DelimCharProps { left_role: "OPEN", right_role: "CLOSE",
      name: Some("Updownarrow"), remove_meaning: false, replace_char: None }
  )
});

/// Set role and augment delimiter properties on the last child element.
/// Mirrors Perl's augmentDelimiterProperties (TeX_Math.pool.ltxml).
/// Perl: augmentDelimiterProperties($doc, $whatsit, $role, $stretchy)
/// Look up delimiter character in DELIM_CHAR_MAP and set name/meaning/role.
/// When role is empty, don't change the role (Perl: $role=undef).
fn augment_delimiter_properties(document: &mut Document, role: &str) -> Result<()> {
  let current = document.get_node().clone();
  let delim_opt = current
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .last();
  if let Some(mut delim) = delim_opt {
    let char_content = delim.get_content();
    let first_char = char_content.chars().next();
    // Look up delimiter properties by char content
    if let Some(entry) = first_char.and_then(|c| DELIM_CHAR_MAP.get(&c)) {
      // Role: only change if explicitly requested (non-empty role)
      if !role.is_empty() {
        let new_role = if role == "OPEN" {
          entry.left_role
        } else {
          entry.right_role
        };
        let current_role = delim.get_attribute("role");
        match current_role.as_deref() {
          None | Some("OPEN") | Some("MIDDLE") | Some("CLOSE") | Some("VERTBAR") => {
            document.set_attribute(&mut delim, "role", new_role)?;
          },
          _ => {},
        }
      }
      // Set name if entry has one
      if let Some(name) = entry.name {
        document.set_attribute(&mut delim, "name", name)?;
      }
      // Handle meaning
      if entry.remove_meaning {
        let _ = delim.remove_attribute("meaning");
      }
      // Handle char replacement (e.g. < → ⟨)
      if let Some(replacement) = entry.replace_char {
        if let Some(mut first_child) = delim.get_first_child() {
          let _ = first_child.set_content(&replacement.to_string());
        }
      }
    } else if !role.is_empty() {
      // No map entry — just set role if explicitly requested
      let current_role = delim.get_attribute("role");
      match current_role.as_deref() {
        None | Some("OPEN") | Some("MIDDLE") | Some("CLOSE") | Some("VERTBAR") => {
          document.set_attribute(&mut delim, "role", role)?;
        },
        _ => {},
      }
    }
  }
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  //----------------------------------------------------------------------
  //  Actually from LaTeX; Table 3.3, Greek, p.41
  //----------------------------------------------------------------------
  def_math_atom("\\alpha", "\u{03B1}", None, None)?;
  def_math_atom("\\beta", "\u{03B2}", None, None)?;
  def_math_atom("\\gamma", "\u{03B3}", None, None)?;
  def_math_atom("\\delta", "\u{03B4}", None, None)?;
  def_math_atom("\\epsilon", "\u{03F5}", None, None)?;
  def_math_atom("\\varepsilon", "\u{03B5}", None, None)?;
  def_math_atom("\\zeta", "\u{03B6}", None, None)?;
  def_math_atom("\\eta", "\u{03B7}", None, None)?;
  def_math_atom("\\theta", "\u{03B8}", None, None)?;
  def_math_atom("\\vartheta", "\u{03D1}", None, None)?;
  def_math_atom("\\iota", "\u{03B9}", None, None)?;
  def_math_atom("\\kappa", "\u{03BA}", None, None)?;
  def_math_atom("\\lambda", "\u{03BB}", None, None)?;
  def_math_atom("\\mu", "\u{03BC}", None, None)?;
  def_math_atom("\\nu", "\u{03BD}", None, None)?;
  def_math_atom("\\xi", "\u{03BE}", None, None)?;
  def_math_atom("\\pi", "\u{03C0}", None, None)?;
  def_math_atom("\\varpi", "\u{03D6}", None, None)?;
  def_math_atom("\\rho", "\u{03C1}", None, None)?;
  def_math_atom("\\varrho", "\u{03F1}", None, None)?;
  def_math_atom("\\sigma", "\u{03C3}", None, None)?;
  def_math_atom("\\varsigma", "\u{03C2}", None, None)?;
  def_math_atom("\\tau", "\u{03C4}", None, None)?;
  def_math_atom("\\upsilon", "\u{03C5}", None, None)?;
  def_math_atom("\\phi", "\u{03D5}", None, None)?;
  def_math_atom("\\varphi", "\u{03C6}", None, None)?;
  def_math_atom("\\chi", "\u{03C7}", None, None)?;
  def_math_atom("\\psi", "\u{03C8}", None, None)?;
  def_math_atom("\\omega", "\u{03C9}", None, None)?;
  def_math_atom("\\Gamma", "\u{0393}", None, None)?;
  def_math_atom("\\Delta", "\u{0394}", None, None)?;
  def_math_atom("\\Theta", "\u{0398}", None, None)?;
  def_math_atom("\\Lambda", "\u{039B}", None, None)?;
  def_math_atom("\\Xi", "\u{039E}", None, None)?;
  def_math_atom("\\Pi", "\u{03A0}", None, None)?;
  def_math_atom("\\Sigma", "\u{03A3}", None, None)?;
  def_math_atom("\\Upsilon", "\u{03A5}", None, None)?;
  def_math_atom("\\Phi", "\u{03A6}", None, None)?;
  def_math_atom("\\Psi", "\u{03A8}", None, None)?;
  def_math_atom("\\Omega", "\u{03A9}", None, None)?;

  //----------------------------------------------------------------------
  // Non-English Symbols ligatures (\OE/\oe/\AE/\ae/\AA/\aa/\O/\o/\ss)
  // moved to plain_base.rs (Perl plain_base.pool.ltxml L525-533).
  // Extended set (\dh/\DH/\dj/\DJ/\ng/\NG/\th/\TH) is in latex_constructs.rs.


  // Perl TeX_Character.pool.ltxml: DefPrimitive('\accent Number', sub { ... })
  // \accent <number> <optional assignments> <character>; See TeX Book p.287
  // Reads a number (font position), then optional assignments, then a character.
  // Decodes the font position to a glyph, looks up accent data, applies accent.
  DefPrimitive!("\\accent Number", sub[(num)] {
    use crate::tex_character;
    // 1. Decode the accent glyph from font position (BEFORE processing assignments)
    let n = num.value_of() as i32;
    let (glyph_opt, _font) = font_decode(n, None, None);

    // 2. Process optional assignments (Perl lines 117-123)
    //    <assignments>: (<prefix>) simple assignment or macro assignment
    //    <character> : letter, other, \char, \chardef token, \noboundary
    let mut assignments: Vec<Digested> = Vec::new();
    let mut last_token: Option<Token> = None;
    let mut last_defn: Option<Rc<dyn Definition>> = None;
    loop {
      let token_opt = gullet::read_x_non_space()?;
      let token = match token_opt {
        Some(t) => t,
        None => { break; }
      };
      let defn = if token.get_catcode().is_active_or_cs() {
        state::lookup_definition(&token)?.map(|d| d as Rc<dyn Definition>)
      } else {
        None
      };
      // Perl: isPrefix || isFontDef || (isRegister && !isCharDef)
      //       || token matches \def|\edef|\gdef|\xdef
      let is_assignment = if let Some(ref d) = defn {
        if d.is_prefix() || (
          d.is_register() && !matches!(d.register_type(), Some(RegisterType::CharDef))) {
          true
        } else {
          // Check isFontDef: lookupValue("fontinfo_<cs>")
          let cs_str = token.to_string();
          let fontinfo_key = s!("fontinfo_{}", cs_str);
          if state::with_value(&fontinfo_key, |v| v.is_some()) {
            true
          } else {
            // Check \def, \edef, \gdef, \xdef
            matches!(cs_str.as_str(), "\\def" | "\\edef" | "\\gdef" | "\\xdef")
          }
        }
      } else {
        false
      };
      if !is_assignment {
        last_token = Some(token);
        last_defn = defn;
        break;
      }
      // Process the assignment: invoke the token
      let digested = stomach::invoke_token(&token)?;
      assignments.extend(digested);
    }

    // 3. Read the base character token (Perl lines 126-134)
    let letter = if let Some(t) = last_token {
      let cc = t.get_catcode();
      if cc == Catcode::LETTER || cc == Catcode::OTHER
        || last_defn.as_ref().is_some_and(|d|
             matches!(d.register_type(), Some(RegisterType::CharDef))) {
        Tokens!(t)
      } else if t == T_CS!("\\char") {
        Tokens!(t, ExplodeText!(&gullet::read_number()?.to_string()))
      } else if t == T_CS!("\\noboundary") {
        Tokens!() // Treat as empty
      } else {
        gullet::unread_one(t);
        Tokens!()
      }
    } else {
      Tokens!()
    };

    // 4. Enter horizontal mode
    enter_horizontal();

    // 5. Apply accent (Perl lines 137-141)
    let accent_result: Vec<Digested> = if let Some(glyph_char) = glyph_opt {
      let glyph_str = glyph_char.to_string();
      if let Some(entry) = tex_character::unicode_accent(&glyph_str) {
        let mut rev_toks = vec![T_CS!("\\accent")];
        rev_toks.extend(ExplodeText!(&num.to_string()));
        rev_toks.push(T_OTHER!(" "));
        rev_toks.extend(letter.unlist_ref().iter().copied());
        let reversion = Tokens::new(rev_toks);
        let tbox = tex_character::apply_accent(
          letter, entry.combiner, entry.standalone, Some(reversion))?;
        vec![tbox.into()]
      } else {
        // Unknown accent: overlay glyph on letter using \lx@overlay
        let glyph_s = glyph_char.to_string();
        let overlay_toks = Tokens!(
          T_CS!("\\lx@overlay"), T_BEGIN!(),
          letter, T_END!(), T_BEGIN!(),
          ExplodeText!(&glyph_s), T_END!()
        );
        vec![stomach::digest(overlay_toks)?]
      }
    } else {
      // No glyph found: produce empty or just the letter
      let text = if letter.is_empty() {
        String::new()
      } else {
        letter.untex()
      };
      let tbox = Tbox::new(arena::pin(text), None, None, Tokens!(), SymHashMap::default());
      vec![tbox.into()]
    };

    // 6. Return assignments + accent result (Perl line 142)
    let mut result = assignments;
    result.extend(accent_result);
    Ok(result)
  });
  // Note that these two apparently work in Math? BUT the argument is treated as text!!!

  //======================================================================
  // TeX Book, Appendix B. p. 357
  // RIGHTWARDS ARROW??? a bit more explicitly
  def_math_atom("\\to", "\u{2192}", Some("ARROW"), None)?;

  // \sp, \sb, and the literal-tab Box moved to plain_base.rs (Perl
  // plain_base.pool.ltxml L537-543 — Perl-faithful location).

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.7. Miscellaneous Symbols, p.43
  //----------------------------------------------------------------------
  // Some should be differential operators, qualifiers, ...
  def_math_atom("\\aleph", "\u{2135}", None, None)?;
  def_math_atom("\\hbar", "\u{210F}", Some("ID"), Some("Planck-constant-over-2-pi"))?;
  def_math_atom("\\imath", "\u{0131}", None, None)?;
  def_math_atom("\\jmath", "\u{0237}", None, None)?;
  def_math_atom("\\ell", "\u{2113}", None, None)?;
  DefMath!("\\wp", None, "\u{2118}", meaning => "Weierstrass-p");
  DefMath!("\\Re", None, "\u{211C}", role    => "OPFUNCTION", meaning => "real-part");
  DefMath!("\\Im", None, "\u{2111}", role    => "OPFUNCTION", meaning => "imaginary-part");
  def_math_atom("\\mho", "\u{2127}", None, None)?;

  DefMath!("\\prime",    None, "\u{2032}", role => "SUPOP",    locked  => true);
  def_math_atom("\\emptyset", "\u{2205}", Some("ID"), Some("empty-set"))?;
  def_math_atom("\\nabla", "\u{2207}", Some("OPERATOR"), None)?;
  def_math_atom("\\surd", "\u{221A}", Some("OPERATOR"), Some("square-root"))?;
  def_math_atom("\\top", "\u{22A4}", Some("ADDOP"), Some("top"))?;
  def_math_atom("\\bot", "\u{22A5}", Some("ADDOP"), Some("bottom"))?;
  DefMath!("\\|", None, "\u{2225}", role => "VERTBAR", name => "||");
  // should get meaning => "parallel"to' when used as infix, but NOT when for OPEN|CLOSE
  def_math_atom("\\angle", "\u{2220}", None, None)?;

  // NOTE: This is probably the wrong role.
  // Also, should probably carry info about Binding for OpenMath
  def_math_atom("\\forall", "\u{2200}", Some("BIGOP"), Some("for-all"))?;
  def_math_atom("\\exists", "\u{2203}", Some("BIGOP"), Some("exists"))?;
  def_math_atom("\\neg", "\u{00AC}", Some("BIGOP"), Some("not"))?;
  def_math_atom("\\lnot", "\u{00AC}", Some("BIGOP"), Some("not"))?;
  def_math_atom("\\flat", "\u{266D}", None, None)?;
  def_math_atom("\\natural", "\u{266E}", None, None)?;
  def_math_atom("\\sharp", "\u{266F}", None, None)?;
  def_math_atom("\\backslash", "\u{005C}", Some("MULOP"), None)?;
  def_math_atom("\\partial", "\u{2202}", Some("DIFFOP"), Some("partial-differential"))?;

  def_math_atom("\\infty", "\u{221E}", Some("ID"), Some("infinity"))?;
  def_math_atom("\\Box", "\u{25A1}", None, None)?;
  def_math_atom("\\Diamond", "\u{25C7}", None, None)?;
  def_math_atom("\\triangle", "\u{25B3}", None, None)?;
  def_math_atom("\\clubsuit", "\u{2663}", None, None)?;
  def_math_atom("\\diamondsuit", "\u{2662}", None, None)?;
  def_math_atom("\\heartsuit", "\u{2661}", None, None)?;
  def_math_atom("\\spadesuit", "\u{2660}", None, None)?;

  DefMacro!("\\active@math@prime", {
    let mut sup = vec![T_CS!("\\prime")];
    // Collect up all ', convering to \prime
    let prime_token = T_OTHER!("\'");

    while gullet::if_next(prime_token)? {
      gullet::read_token()?;
      sup.push(T_CS!("\\prime"));
    }
    // Combine with any following superscript!
    // However, this is semantically screwed up!
    // We really need to set up separate superscripts, but at same level!
    if gullet::if_next(T_SUPER!())? {
      gullet::read_token()?;
      let arg = gullet::read_arg(ExpansionLevel::Off)?;
      let arg_tks = arg.unlist();
      sup.extend(arg_tks);
    }
    let mut activated = vec![T_SUPER!(), T_BEGIN!()];
    activated.extend(sup);
    activated.push(T_END!());
    activated
  },
  locked => true); // Only in math!
  assign_mathcode('\'', 0x8000u16, None);
  Let!("'", "\\active@math@prime");

  // Mathcode assignments from plain_base.pool.ltxml (Table 17.2 of TeX Book)
  // Punctuation and operators
  assign_mathcode('!', 0x5021u16, None);
  assign_mathcode('(', 0x4028u16, None);
  assign_mathcode(')', 0x5029u16, None);
  assign_mathcode('*', 0x2203u16, None);  // \ast (class 2 BINOP, family 2 OMS, pos 3)
  assign_mathcode('+', 0x202Bu16, None);
  assign_mathcode(',', 0x613Bu16, None);
  assign_mathcode('-', 0x2200u16, None);
  assign_mathcode('.', 0x013Au16, None);
  assign_mathcode('/', 0x013Du16, None);
  assign_mathcode(':', 0x303Au16, None);
  assign_mathcode('?', 0x503Fu16, None);  // class CLOSE (from plain.tex dump)
  assign_mathcode(';', 0x603Bu16, None);
  assign_mathcode('<', 0x313Cu16, None);
  assign_mathcode('=', 0x303Du16, None);
  assign_mathcode('>', 0x313Eu16, None);
  assign_mathcode('[', 0x405Bu16, None);
  assign_mathcode('\\', 0x026Eu16, None);
  assign_mathcode(']', 0x505Du16, None);
  assign_mathcode('{', 0x4266u16, None);
  assign_mathcode('|', 0x026Au16, None);
  assign_mathcode('}', 0x5267u16, None);
  // _ and ' are active (handled above/below)
  assign_mathcode('_', 0x8000u16, None);

  //----------------------------------------------------------------------
  // Table 3.8. Variable-sized Symbols (from math_common.pool.ltxml)
  // Perl: scriptpos => \&doScriptpos  — "mid" in display, "post" in inline
  //       mathstyle => \&doVariablesizeOp — "display" in display, "text" in inline
  // NOTE: \int and \oint have NO scriptpos (only mathstyle)
  //       \smallint has scriptpos but STATIC mathstyle => 'text'
  //----------------------------------------------------------------------
  DefMath!("\\smallint", None, "\u{222B}",
    meaning => "integral", role => "INTOP",
    font => { size => 9.0 },
    dynamic_scriptpos => true, mathstyle => "text");
  DefMath!("\\sum",    None, "\u{2211}", role => "SUMOP", meaning => "sum",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\prod",   None, "\u{220F}", role => "SUMOP", meaning => "product",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\coprod", None, "\u{2210}", role => "SUMOP", meaning => "coproduct",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\int",    None, "\u{222B}", role => "INTOP", meaning => "integral",
    dynamic_mathstyle => true);
  DefMath!("\\oint",   None, "\u{222E}", role => "INTOP", meaning => "contour-integral",
    dynamic_mathstyle => true);
  DefMath!("\\bigcap",    None, "\u{22C2}", role => "SUMOP", meaning => "intersection",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigcup",    None, "\u{22C3}", role => "SUMOP", meaning => "union",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigsqcup",  None, "\u{2A06}", role => "SUMOP", meaning => "square-union",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigvee",    None, "\u{22C1}", role => "SUMOP", meaning => "or",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigwedge",  None, "\u{22C0}", role => "SUMOP", meaning => "and",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigodot",   None, "\u{2A00}", role => "SUMOP",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigotimes", None, "\u{2A02}", role => "SUMOP", meaning => "tensor-product",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigoplus",  None, "\u{2A01}", role => "SUMOP", meaning => "direct-sum",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\biguplus",  None, "\u{2A04}", role => "SUMOP", meaning => "symmetric-difference",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.4. Binary Operation Symbols, p.42
  //----------------------------------------------------------------------
  def_math_atom("\\pm", "\u{00B1}", Some("ADDOP"), Some("plus-or-minus"))?;
  def_math_atom("\\mp", "\u{2213}", Some("ADDOP"), Some("minus-or-plus"))?;
  def_math_atom("\\times", "\u{00D7}", Some("MULOP"), Some("times"))?;
  def_math_atom("\\div", "\u{00F7}", Some("MULOP"), Some("divide"))?;
  def_math_atom("\\ast", "\u{2217}", Some("MULOP"), None)?;
  def_math_atom("\\star", "\u{22C6}", Some("MULOP"), None)?;
  def_math_atom("\\circ", "\u{2218}", Some("MULOP"), Some("compose"))?;
  def_math_atom("\\bullet", "\u{2219}", Some("MULOP"), None)?;
  def_math_atom("\\cdot", "\u{22C5}", Some("MULOP"), None)?;
  ////  , meaning=>"inner-product");  that"s pushing it a bit far...

  // Need to classify set operations more carefully....
  def_math_atom("\\cap", "\u{2229}", Some("ADDOP"), Some("intersection"))?;
  def_math_atom("\\cup", "\u{222A}", Some("ADDOP"), Some("union"))?;
  def_math_atom("\\uplus", "\u{228E}", Some("ADDOP"), None)?;
  def_math_atom("\\sqcap", "\u{2293}", Some("ADDOP"), Some("square-intersection"))?;
  def_math_atom("\\sqcup", "\u{2294}", Some("ADDOP"), Some("square-union"))?;
  def_math_atom("\\vee", "\u{2228}", Some("ADDOP"), Some("or"))?;
  def_math_atom("\\lor", "\u{2228}", Some("ADDOP"), Some("or"))?;
  def_math_atom("\\wedge", "\u{2227}", Some("ADDOP"), Some("and"))?;
  def_math_atom("\\land", "\u{2227}", Some("ADDOP"), Some("and"))?;
  def_math_atom("\\setminus", "\u{2216}", Some("ADDOP"), Some("set-minus"))?;
  def_math_atom("\\wr", "\u{2240}", Some("MULOP"), None)?;

  // Should this block be ADDOP or something else?
  def_math_atom("\\diamond", "\u{22C4}", Some("ADDOP"), None)?;
  def_math_atom("\\bigtriangleup", "\u{25B3}", Some("ADDOP"), None)?;
  def_math_atom("\\bigtriangledown", "\u{25BD}", Some("ADDOP"), None)?;
  def_math_atom("\\triangleleft", "\u{22B2}", Some("ADDOP"), None)?;
  def_math_atom("\\triangleright", "\u{22B3}", Some("ADDOP"), None)?;
  def_math_atom("\\lhd", "\u{22B2}", Some("ADDOP"), Some("subgroup-of"))?;
  def_math_atom("\\rhd", "\u{22B3}", Some("ADDOP"), Some("contains-as-subgroup"))?;
  def_math_atom("\\unlhd", "\u{22B4}", Some("ADDOP"), Some("subgroup-of-or-equals"))?;
  def_math_atom("\\unrhd", "\u{22B5}", Some("ADDOP"), Some("contains-as-subgroup-or-equals"))?;

  def_math_atom("\\oplus", "\u{2295}", Some("ADDOP"), Some("direct-sum"))?;
  def_math_atom("\\ominus", "\u{2296}", Some("ADDOP"), Some("symmetric-difference"))?;
  def_math_atom("\\otimes", "\u{2297}", Some("MULOP"), Some("tensor-product"))?;
  def_math_atom("\\oslash", "\u{2298}", Some("MULOP"), None)?;
  def_math_atom("\\odot", "\u{2299}", Some("MULOP"), Some("direct-product"))?;
  def_math_atom("\\bigcirc", "\u{25CB}", Some("MULOP"), None)?;
  def_math_atom("\\dagger", "\u{2020}", Some("MULOP"), None)?;
  def_math_atom("\\ddagger", "\u{2021}", Some("MULOP"), None)?;
  def_math_atom("\\amalg", "\u{2210}", Some("MULOP"), Some("coproduct"))?;

  //----------------------------------------------------------------------
  // LaTeX; Table 3.5. Relation Symbols, p.43
  //----------------------------------------------------------------------
  def_math_atom("\\leq", "\u{2264}", Some("RELOP"), Some("less-than-or-equals"))?;
  def_math_atom("\\prec", "\u{227A}", Some("RELOP"), Some("precedes"))?;
  def_math_atom("\\preceq", "\u{2AAF}", Some("RELOP"), Some("precedes-or-equals"))?;
  def_math_atom("\\ll", "\u{226A}", Some("RELOP"), Some("much-less-than"))?;
  def_math_atom("\\subset", "\u{2282}", Some("RELOP"), Some("subset-of"))?;
  def_math_atom("\\subseteq", "\u{2286}", Some("RELOP"), Some("subset-of-or-equals"))?;
  def_math_atom("\\sqsubset", "\u{228F}", Some("RELOP"), Some("square-image-of"))?;
  def_math_atom("\\sqsubseteq", "\u{2291}", Some("RELOP"), Some("square-image-of-or-equals"))?;
  def_math_atom("\\in", "\u{2208}", Some("RELOP"), Some("element-of"))?;
  def_math_atom("\\vdash", "\u{22A2}", Some("METARELOP"), Some("proves"))?;

  def_math_atom("\\geq", "\u{2265}", Some("RELOP"), Some("greater-than-or-equals"))?;
  def_math_atom("\\succ", "\u{227B}", Some("RELOP"), Some("succeeds"))?;
  def_math_atom("\\succeq", "\u{2AB0}", Some("RELOP"), Some("succeeds-or-equals"))?;
  def_math_atom("\\gg", "\u{226B}", Some("RELOP"), Some("much-greater-than"))?;
  def_math_atom("\\supset", "\u{2283}", Some("RELOP"), Some("superset-of"))?;
  def_math_atom("\\supseteq", "\u{2287}", Some("RELOP"), Some("superset-of-or-equals"))?;
  def_math_atom("\\sqsupset", "\u{2290}", Some("RELOP"), Some("square-original-of"))?;
  def_math_atom("\\sqsupseteq", "\u{2292}", Some("RELOP"), Some("square-original-of-or-equals"))?;
  def_math_atom("\\ni", "\u{220B}", Some("RELOP"), Some("contains"))?;
  def_math_atom("\\dashv", "\u{22A3}", Some("METARELOP"), Some("does-not-prove"))?;

  // I have the impression think that "identical" is a stronger notion than "equivalence"
  // Note that the unicode here is called "Identical To",
  // and that the notion of "equivalent to" usually involves the tilde operator.
  def_math_atom("\\equiv", "\u{2261}", Some("RELOP"), Some("equivalent-to"))?;
  def_math_atom("\\sim", "\u{223C}", Some("RELOP"), Some("similar-to"))?;
  def_math_atom("\\simeq", "\u{2243}", Some("RELOP"), Some("similar-to-or-equals"))?;
  def_math_atom("\\asymp", "\u{224D}", Some("RELOP"), Some("asymptotically-equals"))?;
  def_math_atom("\\approx", "\u{2248}", Some("RELOP"), Some("approximately-equals"))?;
  def_math_atom("\\cong", "\u{2245}", Some("RELOP"), Some("approximately-equals"))?;
  def_math_atom("\\neq", "\u{2260}", Some("RELOP"), Some("not-equals"))?;
  def_math_atom("\\doteq", "\u{2250}", Some("RELOP"), Some("approaches-limit"))?;
  def_math_atom("\\notin", "\u{2209}", Some("RELOP"), Some("not-element-of"))?;

  def_math_atom("\\models", "\u{22A7}", Some("RELOP"), Some("models"))?;
  def_math_atom("\\perp", "\u{27C2}", Some("RELOP"), Some("perpendicular-to"))?;
  def_math_atom("\\mid", "\u{2223}", Some("VERTBAR"), None)?; // DIVIDES (RELOP?) ?? well, sometimes...
  def_math_atom("\\parallel", "\u{2225}", Some("VERTBAR"), Some("parallel-to"))?;
  def_math_atom("\\bowtie", "\u{22C8}", Some("RELOP"), None)?; // BOWTIE
  def_math_atom("\\Join", "\u{2A1D}", Some("RELOP"), Some("join"))?;
  def_math_atom("\\smile", "\u{2323}", Some("RELOP"), None)?; // SMILE
  def_math_atom("\\frown", "\u{2322}", Some("RELOP"), None)?; // FROWN
  def_math_atom("\\propto", "\u{221D}", Some("RELOP"), Some("proportional-to"))?;

  // TeX defines these as alternate names...
  Let!("\\le", "\\leq");
  Let!("\\ge", "\\geq");
  Let!("\\ne", "\\neq");
  // And it defines some others as alternate names, but they seem to
  // potentially imply slightly different meanings???  Leave them out for now..

  //----------------------------------------------------------------------
  // Not;  (Is fullwidth solidus appropriate for when \not appears in isolation?)
  def_math_atom("\\not", "\u{FF0F}", Some("OPFUNCTION"), Some("not"))?;

  // For a \not operator that is followed by anything, concoct an appropriate not or cancelation.
  DefRewrite!(select =>
    "descendant-or-self::ltx:XMTok[text()='\u{FF0F}' and @meaning='not'][following-sibling::*]",
  select_count => 2,
  replace =>  sub[document, nodes] {
    // TODO: This argument low-level boilerplate is annoying
    // what is a good design pattern to "destructure" a Vec?
    // should it be another datastructure?
    let thing = nodes.pop().unwrap();
    let not_node = nodes.pop().unwrap();
    let text = model::with_node_qname(thing, |thing_str| match thing_str {
      "ltx:XMTok" => { thing.get_content() },
      _ => String::new()
    });
    // eprintln debug removed
    if text.chars().count() != 1 { // Not simple char token.
      // Wrap with a cancel op
      document.open_element("ltx:XMApp",
        Some(map!("_box" => not_node.to_hashable().to_string())), None)?;
      let mut strike = document.insert_math_token("",
        string_map!("role" => "ENCLOSE", "enclose" => "updiagonalstrike",
        "meaning" => "not", "_box" => not_node.to_hashable()), None)?;
      if let Some(id) = not_node.get_attribute_ns("id",XML_NS) {
        not_node.remove_attribute("xml:id")?;
        document.unrecord_id(&id);
        document.set_attribute(&mut strike, "xml:id", &id)?;
      }
      // Use append_tree to avoid DOM corruption from add_child on detached nodes
      let inner_children = vec![thing.clone()];
      let mut current = document.get_node().clone();
      document.append_tree(&mut current, inner_children)?;
      document.close_element("ltx:XMApp")?;
    } else {
      // For simple tokens, we'll modify the relevant content & attributes.
      // The children are DISCARDED — they're dropped here by
      // `unbind_node()` without being re-attached elsewhere. If any
      // child has an xml:id, the idstore would carry a dangling Node
      // reference past the eventual libxml2 free — the exact UAF class
      // that caused the 1605.08055 Finalizing SIGSEGV. Unrecord first.
      for child in thing.get_child_nodes() {
        document.unrecord_node_ids(&child);
      }
      for mut child in thing.get_child_nodes() {
        child.unbind_node();
      }

      if let Some(meaning) = thing.get_attribute("meaning") {
        document.set_attribute(thing, "meaning",  &format!("not-{meaning}"))?; }
      if let Some(name) = thing.get_attribute("name") {
        document.set_attribute(thing, "name", &format!("not-{name}"))?; }
      else if !text.is_empty() {
        document.set_attribute(thing, "name", &format!("not-{text}"))?; }

      let known_c = MATH_CHAR_NEGATIONS.get(&text);
      let new : Cow<'_, str> = match known_c {
        Some(c) => Cow::Borrowed(c),
        None => Cow::Owned(text + "\u{0338}")
      };
      thing.append_text(&new)?;
      // Put the modified node back in using append_tree
      let inner_children = vec![thing.clone()];
      let mut current = document.get_node().clone();
      document.append_tree(&mut current, inner_children)?;
      // Since the <not> element is disappearing, if it had an id that was referenced...!?!?
      if let Some(id) = not_node.get_attribute_ns("id",XML_NS) {
        let idref_xpath = format!("descendant-or-self::ltx:XMRef[@idref='{id}']");
        for n in document.findnodes(&idref_xpath, None) {
          document.remove_node(n);
        }
      }   // ? Hopefully this is safe.
    }
  });

  //----------------------------------------------------------------------
  // \joinrel
  def_math_atom("\\relbar", "-", Some("RELOP"), None)?; // ???
  def_math_atom("\\Relbar", "=", Some("RELOP"), None)?; // ???

  // \joinrel is \mathrel{\mkern-3\mu} — but the effect is to join two
  // "relations" into one. Perl math_common L368-386.
  DefPrimitive!("\\joinrel", {
    gullet::skip_spaces()?;
    let Some(left) = pop_box_list() else {
      // Nothing there? no-op
      return Ok(Vec::new());
    };
    // Read tokens, invoke each, until an invocation returns a non-empty
    // digested list. That list's first item is the "right" operand;
    // anything after becomes trailing content.
    let mut stuff: Vec<Digested> = Vec::new();
    while let Some(tok) = gullet::read_x_token(None, false, None)? {
      stuff = stomach::invoke_token(&tok)?;
      if !stuff.is_empty() {
        break;
      }
    }
    if stuff.is_empty() {
      return Ok(Vec::new());
    }
    let right = stuff.remove(0);
    let mut properties = stored_map!("isMath" => true);
    if let Some(font) = right.get_font()? {
      properties.insert("font", font.into());
    }
    let whatsit = Whatsit {
      definition: lookup_definition(&T_CS!("\\@@joinrel"))?.unwrap(),
      args: vec![Some(left), Some(right)],
      properties,
      locator: Some(gullet::get_locator()),
      ..Whatsit::default()
    };
    stuff.push(Digested::from(whatsit));
    stuff
  });

  // Perl math_common L388-404: absorb left+right; if the last 2 children
  // include any XMTok, replace them with a single merged XMTok whose text
  // is the concatenation and whose role is combined:
  //   same role → that role
  //   any ARROW → ARROW
  //   otherwise → RELOP
  DefConstructor!("\\@@joinrel{}{}", sub[document, args] {
    let left = args[0].as_ref().unwrap();
    let right = args[1].as_ref().unwrap();
    document.absorb(left, None)?;
    document.absorb(right, None)?;
    let parent = document.get_node().clone();
    let kids = parent.get_child_elements();
    if kids.len() >= 2 {
      let xmtok_sym = arena::pin_static("ltx:XMTok");
      let n1 = kids[kids.len() - 2].clone();
      let n2 = kids[kids.len() - 1].clone();
      let qn1 = document::get_node_qname(&n1);
      let qn2 = document::get_node_qname(&n2);
      if qn1 == xmtok_sym || qn2 == xmtok_sym {
        let role1 = n1.get_attribute("role").unwrap_or_default();
        let role2 = n2.get_attribute("role").unwrap_or_default();
        let merged_role = if role1 == role2 {
          role1
        } else if role1 == "ARROW" || role2 == "ARROW" {
          "ARROW".to_string()
        } else {
          "RELOP".to_string()
        };
        let merged_text = format!("{}{}", n1.get_content(), n2.get_content());
        document.safe_unlink(n1);
        document.safe_unlink(n2);
        let mut attrs = HashMap::default();
        if !merged_role.is_empty() {
          attrs.insert("role".to_string(), merged_role);
        }
        let mut tok = document.insert_element("ltx:XMTok", Vec::new(), Some(attrs))?;
        let _ = tok.set_content(&merged_text);
      }
    }
    },
    reversion => "#1\\joinrel #2");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.6. Arrow Symbols, p.43
  //----------------------------------------------------------------------
  // Arrows get treated somewhat like relations (or meta-relations),
  // but it's hard to associate any particular "meaning" to them.

  DefMath!("\\leftarrow",      "\u{2190}", role => "ARROW"); // LEFTWARDS ARROW
  DefMath!("\\Leftarrow",      "\u{21D0}", role => "ARROW"); // LEFTWARDS DOUBLE ARROW
  DefMath!("\\rightarrow",     "\u{2192}", role => "ARROW"); // RIGHTWARDS ARROW
  DefMath!("\\Rightarrow",     "\u{21D2}", role => "ARROW"); // RIGHTWARDS DOUBLE ARROW
  DefMath!("\\leftrightarrow", "\u{2194}", role => "METARELOP"); // LEFT RIGHT ARROW
  DefMath!("\\Leftrightarrow", "\u{21D4}", role => "METARELOP"); // LEFT RIGHT DOUBLE ARROW
  DefMath!("\\iff", "\u{21D4}", role => "METARELOP", meaning => "iff"); // LEFT RIGHT DOUBLE ARROW
  DefMath!("\\mapsto",        "\u{21A6}", role => "ARROW", meaning => "maps-to");
  DefMath!("\\hookleftarrow", "\u{21A9}", role => "ARROW"); // LEFTWARDS ARROW WITH HOOK
  DefMath!("\\leftharpoonup", "\u{21BC}", role => "ARROW"); // LEFTWARDS HARPOON WITH BARB UPWARDS
  DefMath!("\\leftharpoondown", "\u{21BD}", role => "ARROW"); // LEFTWARDS HARPOON WITH BARB DOWNWARDS
  DefMath!("\\rightleftharpoons", "\u{21CC}", role => "METARELOP"); // RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON
  DefMath!("\\longleftarrow",      "\u{27F5}", role => "ARROW"); // LONG LEFTWARDS ARROW
  DefMath!("\\Longleftarrow",      "\u{27F8}", role => "ARROW"); // LONG LEFTWARDS DOUBLE ARROW
  DefMath!("\\longrightarrow",     "\u{27F6}", role => "ARROW"); // LONG RIGHTWARDS ARROW
  DefMath!("\\Longrightarrow",     "\u{27F9}", role => "ARROW"); // LONG RIGHTWARDS DOUBLE ARROW
  DefMath!("\\longleftrightarrow", "\u{27F7}", role => "METARELOP"); // LONG LEFT RIGHT ARROW
  DefMath!("\\Longleftrightarrow", "\u{27FA}", role => "METARELOP"); // LONG LEFT RIGHT DOUBLE ARROW
  DefMath!("\\longmapsto",     "\u{27FC}", role => "ARROW"); // LONG RIGHTWARDS ARROW FROM BAR
  DefMath!("\\hookrightarrow", "\u{21AA}", role => "ARROW"); // RIGHTWARDS ARROW WITH HOOK
  DefMath!("\\rightharpoonup", "\u{21C0}", role => "ARROW"); // RIGHTWARDS HARPOON WITH BARB UPWARDS
  DefMath!("\\rightharpoondown", "\u{21C1}", role => "ARROW"); // RIGHTWARDS HARPOON WITH BARB DOWNWARDS
  DefMath!("\\leadsto",          "\u{219D}", role => "ARROW", meaning => "leads-to");

  DefMath!("\\uparrow",     "\u{2191}", role => "ARROW"); // UPWARDS ARROW
  DefMath!("\\Uparrow",     "\u{21D1}", role => "ARROW"); // UPWARDS DOUBLE ARROW
  DefMath!("\\downarrow",   "\u{2193}", role => "ARROW"); // DOWNWARDS ARROW
  DefMath!("\\Downarrow",   "\u{21D3}", role => "ARROW"); // DOWNWARDS DOUBLE ARROW
  DefMath!("\\updownarrow", "\u{2195}", role => "ARROW"); // UP DOWN ARROW
  DefMath!("\\Updownarrow", "\u{21D5}", role => "ARROW"); // UP DOWN DOUBLE ARROW
  DefMath!("\\nearrow",     "\u{2197}", role => "ARROW"); // NORTH EAST ARROW
  DefMath!("\\searrow",     "\u{2198}", role => "ARROW"); // SOUTH EAST ARROW
  DefMath!("\\swarrow",     "\u{2199}", role => "ARROW"); // SOUTH WEST ARROW
  DefMath!("\\nwarrow",     "\u{2196}", role => "ARROW"); // NORTH WEST ARROW

  // \mapstochar (3237), \lhook(312C), \rhook(312D)
  // These are really wrong; I can't find the right Unicode Glyphs.
  // These are only fragments intended to be assembled into meaningful(?) symbols.
  DefMath!("\\mapstochar", "\u{2E20}"); // TeX 3237
  DefMath!("\\lhook", "\u{2E26}"); // TeX 312C
  DefMath!("\\rhook", "\u{2E27}"); // TeX 312D

  //======================================================================
  // TeX Book, Appendix B. p. 359

  // Ah, since \ldots can appear in text and math....
  DefMacro!("\\ldots", "\\lx@ldots");
  DefConstructor!(
    "\\vdots",
    "?#isMath(<ltx:XMTok name='vdots' font='#font' role='ID'>\u{22EE}</ltx:XMTok>)(\u{22EE})",
    properties => {
      if state::lookup_bool_sym(pin!("IN_MATH")) {
        Ok(stored_map!("font" => lookup_font().unwrap().merge(
          fontmap!(family => "serif", series => "medium", shape => "upright")
            .specialize("\u{22EE}"))))
      } else {
        Ok(SymHashMap::default())
      }
    });
  //                   # But not these!
  // Design note: Perl LaTeXML uses role 'ID' for \cdots, but latexml-oxide intentionally uses
  // 'ELIDEOP' to enable dedicated grammar rules (e.g. `term mulop tight_term elideop`) that
  // produce better-structured math parse trees for elision patterns like a⋅b⋅c⋯.
  def_math_atom("\\cdots", "\u{22EF}", Some("ELIDEOP"), None)?; // MIDLINE HORIZONTAL ELLIPSIS
  def_math_atom("\\ddots", "\u{22F1}", Some("ID"), None)?; // DOWN RIGHT DIAGONAL ELLIPSIS
  def_math_atom("\\colon", ":", Some("METARELOP"), None)?; // Seems like good default role
  //         # Note that amsmath redefines \dots to be `smart'.
  //         # Aha, also can be in text...
  DefConstructor!(
    "\\dots",
    "?#isMath(<ltx:XMTok name='dots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})",
    sizer      => "\u{2026}",
    properties => {
      if state::lookup_bool_sym(pin!("IN_MATH")) {
        Ok(stored_map!("font" => lookup_font().unwrap().merge(
          fontmap!(family => "serif", series => "medium", shape => "upright")
            .specialize("\u{2026}"))))
      } else {
        Ok(SymHashMap::default())
      }
    });

  // And while we're at it...

  // Pretest for XMath to keep from interpreting math that the DOM may not allow!!

  // Same design note as \cdots above: ELIDEOP is an intentional Rust-specific choice.
  DefMathLigature!("\u{22C5}\u{22C5}\u{22C5}", "\u{22EF}", role => "ELIDEOP", name => "cdots");

  DefLigature!(r"[.][.][.]", "\u{2026}", fontTest => sub[arg] {arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" }); // ldots

  DefMathLigature!("...", "\u{2026}", role => "ID", name => "ldots");

  //----------------------------------------------------------------------
  // Math Accents.
  //----------------------------------------------------------------------
  // LaTeX; Table 3.11. Math Mode Accents, p.50.
  // Are these all TeX (or LaTeX)?
  // Note that most of these should NOT be stretchy, by default!
  DefMath!("\\hat Digested", "\u{005E}",
    operator_role => "OVERACCENT", operator_stretchy => false);
  DefMath!("\\check Digested", "\u{02C7}",
    operator_role => "OVERACCENT", operator_stretchy => false); // CARON
  DefMath!("\\breve Digested", "\u{02D8}", operator_role => "OVERACCENT"); // BREVE
  DefMath!("\\acute Digested", "\u{00B4}",  operator_role => "OVERACCENT"); // ACUTE ACCENT
  DefMath!("\\grave Digested", "\u{0060}",  operator_role => "OVERACCENT"); // GRAVE ACCENT
  DefMath!("\\tilde Digested", "\u{007E}",
    operator_role => "OVERACCENT", operator_stretchy => false); // TILDE
  DefMath!("\\bar Digested", "\u{00AF}",
    operator_role => "OVERACCENT", operator_stretchy => false); // MACRON
  DefMath!("\\vec Digested", "\u{2192}",
    operator_role => "OVERACCENT", operator_stretchy => false); // RIGHTWARDS ARROW
  DefMath!("\\dot Digested",      "\u{02D9}", operator_role => "OVERACCENT"); // DOT ABOVE
  DefMath!("\\ddot Digested",     "\u{00A8}",  operator_role => "OVERACCENT"); // DIAERESIS
  DefMath!("\\mathring Digested", "\u{030A}", operator_role => "OVERACCENT"); // COMBINING RING ABOVE
  DefMath!("\\widehat Digested", "\u{005E}", operator_role => "OVERACCENT"); // CIRCUMFLEX ACCENT [plain? also amsfonts]
  DefMath!("\\widetilde Digested", "\u{007E}", operator_role => "OVERACCENT"); // TILDE [plain? also amsfonts]
  // Perl: math_common.pool.ltxml lines 535-536
  // overbrace/underbrace canonical defs are \lx@math@* in tex_math.rs
  Let!("\\overbrace", "\\lx@math@overbrace");
  Let!("\\underbrace", "\\lx@math@underbrace");

  // NOTE that all the above accents REQUIRE math mode
  // EXCEPT underline, overrightarrow and overleftarrow!
  Let!("\\underbar", "\\underline"); // Will anyone notice?

  DefMacro!(
    "\\overrightarrow{}",
    r"\protect\ifmmode\lx@math@overrightarrow{#1}\else$\lx@math@overrightarrow{#1}$\fi"
  );
  DefMacro!(
    "\\overleftarrow{}",
    r"\protect\ifmmode\lx@math@overleftarrow{#1}\else$\lx@math@overleftarrow{#1}$\fi"
  );

  DefMacro!("\\skew{}{}{}", r"{#2{#3\mkern#1mu}\mkern-#1mu}{}"); // ignore the subtle spacing for now?
  //
  //----------------------------------------------------------------------
  // LaTeX; Table 3.10. Delimiters, p.47
  //----------------------------------------------------------------------
  // The meaning of OPEN/CLOSE tends to depend upon the pairing,
  // rather than the individual tokens.
  // This meaning is handled in MathParser (for now)
  DefMacro!("\\{", r"\ifmmode\lx@math@lbrace\else\lx@text@lbrace\fi", protected => true);
  DefMacro!("\\}", r"\ifmmode\lx@math@rbrace\else\lx@text@rbrace\fi", protected => true);
  DefMath!("\\lx@math@lbrace", None, "{", role => "OPEN",  stretchy => false, alias => "\\{");
  DefMath!("\\lx@math@rbrace", None, "}", role => "CLOSE", stretchy => false, alias => "\\}");
  DefPrimitive!("\\lx@text@lbrace", "{", alias => "\\{",
  font => { shape => "upright" }, bounded => true); // Since not DefMath!
  DefPrimitive!("\\lx@text@rbrace", "}", alias => "\\}",
  font => { shape => "upright" }, bounded => true); // Since not DefMath!
  Let!("\\lbrace", "\\{");
  Let!(&T_CS!("\\lbrack"), T_OTHER!("["));
  Let!("\\rbrace", "\\}");
  Let!(&T_CS!("\\rbrack"), T_OTHER!("]"));
  DefMath!("\\lceil",  None, "\u{2308}", role => "OPEN",  stretchy => false); // LEFT CEILING
  DefMath!("\\rceil",  None, "\u{2309}", role => "CLOSE", stretchy => false); // RIGHT CEILING
  DefMath!("\\lfloor", None, "\u{230A}", role => "OPEN",  stretchy => false); // LEFT FLOOR
  DefMath!("\\rfloor", None, "\u{230B}", role => "CLOSE", stretchy => false); // RIGHT FLOOR
  // \lgroup / \rgroup are defined below with Perl #2762 parity comment (match Perl source layout).

  // Note: We should be using 27E8,27E9, which are "mathematical", not 2329,232A

  DefMath!("\\langle", None, "\u{27E8}", role => "OPEN", stretchy => false); // LEFT-POINTING ANGLE BRACKET
  DefMath!("\\rangle", None, "\u{27E9}", role => "CLOSE", stretchy => false); // RIGHT-POINTING ANGLE BRACKET

  // Not sure these should be defined here, or latex, or even latex compat mode.
  // Updated to use proper codepoints: U+27EE/27EF (MATHEMATICAL LEFT/RIGHT FLATTENED PARENTHESIS)
  // Perl commit "Lrgroup (#2762)": removed bold font, using dedicated Unicode codepoints.
  DefMath!("\\lgroup", None, "\u{27EE}", role => "OPEN",  stretchy => false);
  DefMath!("\\rgroup", None, "\u{27EF}", role => "CLOSE", stretchy => false);
  DefMath!("\\bracevert", None, "|", font => { series => "bold" }, role => "VERTBAR");

  // TeX marks some symbols as delimiters which can be used with \left,\right,
  // but many of which have different grammatical roles otherwise, eg. arrows, <, >.
  // Short of setting up TeX's complicated encoding machinery, I need an explicit
  // mapping.  Unfortunately, this doesn't (yet) support people declaring thier own delimiters!

  // With new treatment of Simple Symbols as just Box's with assigned attributes,
  // we're not getting whatsits, and so we're not looking them up the same way!!!
  // TEMPORARILY (?) hack the Delimiter map
  // foreach my $entry (values %DELIMITER_MAP) {
  //   $DELIMITER_MAP{ $$entry{char} } = $entry; }

  // sub lookup_delimiter {
  //   my ($delim) = @_;
  //   return $DELIMITER_MAP{$delim}; }

  // Perl PR#2596: TeXDelimiter reads like {} (for correct math XMTok digestion)
  // but reverts WITHOUT adding braces: \Big( not \Big{(}
  // Perl: augmentDelimiterProperties($doc, $whatsit, undef, 0) — look up delimiter
  // in map and set name/meaning (but don't change role or stretchy).
  // Perl: font => { size => 'big' } where 'big' → 1.2 * DEFSIZE(10) = 12.0 absolute pt.
  // Named sizes map to absolute values, NOT scale factors.
  DefConstructor!("\\big TeXDelimiter",  "#1", bounded => true, font => { size => 12.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "")?; });
  DefConstructor!("\\Big TeXDelimiter",  "#1", bounded => true, font => { size => 16.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "")?; });
  DefConstructor!("\\bigg TeXDelimiter", "#1", bounded => true, font => { size => 21.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "")?; });
  DefConstructor!("\\Bigg TeXDelimiter", "#1", bounded => true, font => { size => 26.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "")?; });

  // sub addDelimiterRole {
  //   my ($document, $role) = @_;
  //   my $current = $document->getNode;
  //   my $delim   = $document->getLastChildElement($current) || $current;
  //   my $delim_role = (($delim && ($delim->nodeType == XML_ELEMENT_NODE) &&
  // $delim->getAttribute('role')) || '<none>');   # if there is some delimiter-like role on the
  // "delimiter", switch it, otherwise, leave it alone!   if ($delim && ($delim_role =~
  // /^(OPEN|MIDDLE|CLOSE|VERTBAR|<none>)$/)) {     ## Maybe we shouldn't switch VERTBAR ?
  //     ## The catch is that occasionally people use a single \Bigl (or whatever)
  //     ## where they should have used a \Big
  //     $document->setAttribute($delim, role => $role); }
  //   return; }

  // Sized delimiters with role assignment (l=OPEN, m=MIDDLE, r=CLOSE)
  // Perl: font => { size => 'big' } where rationalizeFontSize('big') = 1.2 * DEFSIZE(10) = 12.0pt
  // Named sizes are absolute, not relative — must use `size` (not `scale`).
  DefConstructor!("\\bigl TeXDelimiter",  "#1", bounded => true, font => { size => 12.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\bigm TeXDelimiter",  "#1", bounded => true, font => { size => 12.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\bigr TeXDelimiter",  "#1", bounded => true, font => { size => 12.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefConstructor!("\\Bigl TeXDelimiter",  "#1", bounded => true, font => { size => 16.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\Bigm TeXDelimiter",  "#1", bounded => true, font => { size => 16.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\Bigr TeXDelimiter",  "#1", bounded => true, font => { size => 16.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefConstructor!("\\biggl TeXDelimiter", "#1", bounded => true, font => { size => 21.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\biggm TeXDelimiter", "#1", bounded => true, font => { size => 21.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\biggr TeXDelimiter", "#1", bounded => true, font => { size => 21.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefConstructor!("\\Biggl TeXDelimiter", "#1", bounded => true, font => { size => 26.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\Biggm TeXDelimiter", "#1", bounded => true, font => { size => 26.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "MIDDLE")?; });
  DefConstructor!("\\Biggr TeXDelimiter", "#1", bounded => true, font => { size => 26.0 },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  Let!(&T_CS!("\\vert"), T_OTHER!("|"));
  Let!("\\Vert", "\\|");
  //======================================================================
  // TeX Book, Appendix B. p. 360
  //
  // Note that in TeX, all 4 args get digested(!)
  // and the choice is made when absorbing!

  DefMacro!(
    "\\mathpalette{}{}",
    r"\mathchoice{#1\displaystyle{#2}}{#1\textstyle{#2}}{#1\scriptstyle{#2}}{#1\scriptscriptstyle{#2}}"
  );

  // Perl: DefConstructor('\phantom{}', "?#isMath(...)(...)", properties => {isSpace=>1}, afterDigest => ...)
  DefConstructor!(
    "\\phantom{}",
    "?#isMath(<ltx:XMHint width='#width' height='#height' depth='#depth' name='phantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)",
    properties => { stored_map!("isSpace" => true) },
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg_mut(1) {
        let (w, h, d, _, _, _) = arg.get_size(None)?;
        whatsit.set_property("width", Stored::Dimension(w));
        whatsit.set_property("height", Stored::Dimension(h));
        whatsit.set_property("depth", Stored::Dimension(d));
      }
    });

  DefConstructor!(
    "\\hphantom{}",
    "?#isMath(<ltx:XMHint width='#width' name='hphantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)",
    properties => { stored_map!("isSpace" => true) },
    // Perl 09fb2e6f: In text mode, wrap argument in restricted_horizontal
    // to prevent display math from leaking through (e.g. quantikz2).
    before_digest => {
      if !LookupBool!("IN_MATH") {
        begin_mode("restricted_horizontal")?;
        AssignValue!("_hphantom_mode_override" => true);
      } else {
        AssignValue!("_hphantom_mode_override" => false);
      }
    },
    after_digest => sub[whatsit] {
      if LookupBool!("_hphantom_mode_override") {
        end_mode("restricted_horizontal")?;
      }
      if let Some(arg) = whatsit.get_arg_mut(1) {
        let (w, h, d, _, _, _) = arg.get_size(None)?;
        whatsit.set_property("width", Stored::Dimension(w));
        whatsit.set_property("height", Stored::Dimension(h));
        whatsit.set_property("depth", Stored::Dimension(d));
      }
    });

  DefConstructor!(
    "\\vphantom{}",
    "?#isMath(<ltx:XMHint height='#height' depth='#depth' name='vphantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)",
    properties => { stored_map!("isSpace" => true) },
    after_digest => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg_mut(1) {
        let (w, h, d, _, _, _) = arg.get_size(None)?;
        whatsit.set_property("width", Stored::Dimension(w));
        whatsit.set_property("height", Stored::Dimension(h));
        whatsit.set_property("depth", Stored::Dimension(d));
      }
    });

  DefConstructor!("\\mathstrut", "?#isMath(<ltx:XMHint name='mathstrut'/>)()",
    properties => { stored_map!("isSpace" => true) });
  DefConstructor!("\\smash{}", "#1"); // well, what?

  //======================================================================
  // TeX Book, Appendix B. p. 361

  // This is actually LaTeX's definition, but let's just do it this way.
  // Locked to prevent raw plain-TeX/amstex overrides that expand via
  // the \radical primitive (undefined in LaTeXML) — arxiv 1012.3836
  // uses amstex's `\def\sqrt#1{\radical"270370 {#1}}`.
  DefConstructor!(
    "\\sqrt OptionalInScriptStyle Digested",
    "?#1(<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
    <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)\
    (<ltx:XMApp><ltx:XMTok meaning='square-root'/><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)",
    locked => true
  );

  DefParameterType!(ScriptStyleUntil, sub[_inner,until] {
    gullet::read_until(&until[0]) },
  before_digest => {
    bgroup();
    MergeFont!(mathstyle => "script");
  },
  after_digest => {
    egroup()?; },
  reversion => sub[args,_inner,_extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) });

  DefConstructor!("\\root ScriptStyleUntil:\\of {}",
    "<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>",
    reversion => "\\root #1 \\of {#2}");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.9. Log-like Functions, p.44.
  //----------------------------------------------------------------------
  // NOTE: Classifying some as TRIGFUNCTION might clarify 'pi' ambiguities ?
  def_math_atom("\\arccos", "arccos", Some("OPFUNCTION"), Some("inverse-cosine"))?;
  def_math_atom("\\arcsin", "arcsin", Some("OPFUNCTION"), Some("inverse-sine"))?;
  def_math_atom("\\arctan", "arctan", Some("OPFUNCTION"), Some("inverse-tangent"))?;
  def_math_atom("\\arg", "arg", Some("OPFUNCTION"), Some("argument"))?;

  def_math_atom("\\cos", "cos", Some("TRIGFUNCTION"), Some("cosine"))?;
  def_math_atom("\\cosh", "cosh", Some("TRIGFUNCTION"), Some("hyperbolic-cosine"))?;
  def_math_atom("\\cot", "cot", Some("TRIGFUNCTION"), Some("cotangent"))?;
  def_math_atom("\\coth", "coth", Some("TRIGFUNCTION"), Some("hyperbolic-cotangent"))?;

  def_math_atom("\\csc", "csc", Some("TRIGFUNCTION"), Some("cosecant"))?;
  def_math_atom("\\deg", "deg", Some("OPFUNCTION"), Some("degree"))?;
  DefMath!("\\det", None, "det", role => "LIMITOP", meaning => "determinant",
    dynamic_scriptpos => true);
  def_math_atom("\\dim", "dim", Some("LIMITOP"), Some("dimension"))?;

  def_math_atom("\\exp", "exp", Some("OPFUNCTION"), Some("exponential"))?;
  DefMath!("\\gcd", None, "gcd", role => "OPFUNCTION", meaning => "gcd",
    dynamic_scriptpos => true);
  def_math_atom("\\hom", "hom", Some("OPFUNCTION"), None)?;
  DefMath!("\\inf", None, "inf", role => "LIMITOP", meaning => "infimum",
    dynamic_scriptpos => true);

  def_math_atom("\\ker", "ker", Some("OPFUNCTION"), Some("kernel"))?;
  def_math_atom("\\lg", "lg", Some("OPFUNCTION"), None)?;
  DefMath!("\\lim", None, "lim", role => "LIMITOP", meaning => "limit",
    dynamic_scriptpos => true);
  DefMath!("\\liminf", None, "lim inf", role => "LIMITOP", meaning => "limit-infimum",
    dynamic_scriptpos => true);
  DefMath!("\\limsup", None, "lim sup", role => "LIMITOP", meaning => "limit-supremum",
    dynamic_scriptpos => true);
  def_math_atom("\\ln", "ln", Some("OPFUNCTION"), Some("natural-logarithm"))?;
  def_math_atom("\\log", "log", Some("OPFUNCTION"), Some("logarithm"))?;
  DefMath!("\\max", None, "max", role => "OPFUNCTION", meaning => "maximum",
    dynamic_scriptpos => true);
  DefMath!("\\min", None, "min", role => "OPFUNCTION", meaning => "minimum",
    dynamic_scriptpos => true);
  DefMath!("\\Pr",  None, "Pr",  role => "OPFUNCTION",
    dynamic_scriptpos => true);
  DefMath!("\\sec", None, "sec", role => "TRIGFUNCTION", meaning   => "secant");
  DefMath!("\\sin", None, "sin", role => "TRIGFUNCTION", meaning   => "sine");

  def_math_atom("\\sinh", "sinh", Some("TRIGFUNCTION"), Some("hyperbolic-sine"))?;
  DefMath!("\\sup", None, "sup", role => "LIMITOP", meaning => "supremum",
    dynamic_scriptpos => true);
  def_math_atom("\\tan", "tan", Some("TRIGFUNCTION"), Some("tangent"))?;
  def_math_atom("\\tanh", "tanh", Some("TRIGFUNCTION"), Some("hyperbolic-tangent"))?;

  //----------------------------------------------------------------------
  // Modulo

  DefMath!("\\pmod{}", r"\;\;(\mathop{{\rm mod}} #1)", role => "MODIFIER"); //  , meaning=>"modulo");
  DefMath!("\\bmod", "mod", role => "MODIFIEROP", meaning => "modulo");

  //======================================================================
  // \choose & friends — Perl: math_common.pool.ltxml L634-642
  // `protected` matches the TeX-primitive semantics — see the comment
  // on `\atop`/`\over`/`\above` in `tex_math.rs` for the same rationale.
  DefMacro!("\\choose",
    "\\lx@generalized@over{\\choose}{meaning=binomial,thickness=0pt,left=\\lx@left(,right=\\lx@right)}",
    protected => true);
  DefMacro!("\\brace",
    "\\lx@generalized@over{\\brace}{thickness=0pt,left=\\lx@left\\{,right=\\lx@right\\}}",
    protected => true);
  DefMacro!("\\brack",
    "\\lx@generalized@over{\\brack}{thickness=0pt,left=\\lx@left[,right=\\lx@right]}",
    protected => true);
});
