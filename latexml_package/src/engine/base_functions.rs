use crate::prelude::*;
use latexml_core::common::arena::SymHashMap;
use latexml_core::common::xml::content_nodes;
use std::char::{REPLACEMENT_CHARACTER, decode_utf16};

pub fn reenter_text_mode(vertical_mode: bool) {
  let mode_key = if vertical_mode {
    "VTEXT_MODE_BINDINGS"
  } else {
    "HTEXT_MODE_BINDINGS"
  };
  let text_key = "TEXT_MODE_BINDINGS";
  let mode_bindings = checkout_value(mode_key);
  let text_bindings = checkout_value(text_key);
  let mut bindings: VecDeque<&Stored> = match mode_bindings {
    Some(Stored::VecDequeStored(ref vdq)) => vdq.iter().collect::<VecDeque<&Stored>>(),
    _ => VecDeque::new(),
  };
  if let Some(Stored::VecDequeStored(ref vdq)) = text_bindings {
    bindings.extend(vdq.iter().collect::<Vec<_>>());
  }
  for binding in bindings {
    if let Stored::Tokens(tks) = binding {
      let vec = tks.unlist_ref();
      state::let_i(&vec[0], &vec[1], None);
    }
  }
  if let Some(value) = mode_bindings {
    checkin_value(mode_key, value);
  }
  if let Some(value) = text_bindings {
    checkin_value(text_key, value);
  }
}

// Similarly, for metadata appearing within peculiar environments, fonts, etc
// You'll typically want this within a group or bounded=>1.
pub fn neutralize_font() {
  assign_value("font", Font::text_default(), Some(Scope::Local));
  assign_value("mathfont", Font::math_default(), Some(Scope::Local));
}

pub fn today() -> Result<String> {
  let month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  let month = month_names[state::lookup_register("\\month", vec![])?
    .unwrap()
    .value_of() as usize
    - 1];
  let day = state::lookup_register("\\day", vec![])?.unwrap().value_of();
  let year = state::lookup_register("\\year", vec![])?
    .unwrap()
    .value_of();
  Ok(s!("{} {}, {}", month, day, year))
}

pub fn parse_def_parameters(cs: &Token, params_in: Tokens) -> Result<Option<Parameters>> {
  let mut tokens: VecDeque<Token> = VecDeque::from(params_in.pack_parameters()?.unlist());
  // Now, recognize parameters and delimiters.
  let mut params = Vec::new();
  let mut n = 0;
  while let Some(mut t) = tokens.pop_front() {
    let cc = t.get_catcode();
    if cc == Catcode::PARAM || cc == Catcode::ARG {
      if cc == Catcode::PARAM {
        if tokens.is_empty() {
          // Special case: lone # NOT following a numbered parameter
          // Note that we require a { to appear next, but do NOT read it!
          params.push(Parameter::new(
            Cow::Borrowed("RequireBrace"),
            Cow::Borrowed("RequireBrace"),
            None,
          )?);
          break;
        } else {
          n += 1;
          if let Some(t_next) = tokens.pop_front() {
            t = t_next;
          } else {
            unreachable!("tokens.is_empty() was false, so pop_front must return Some");
          }
        }
      } else {
        // CC_ARG case, keep looking at this token
        n += 1;
      }
      if n > 0 {
        let t_num = t.with_str(|ts| ts.parse::<i8>()).unwrap_or(-1);
        if t_num != n {
          fatal!(
            ParamSpec,
            Expected,
            s!(
              "Parameters for {:?} not in order. Got {:?}, expected {:?}. in {:?}",
              cs,
              t,
              n,
              params
            )
          );
        }
      }
      // Check for delimiting text following the parameter #n
      let mut delim = Vec::new();
      let mut pc = Catcode::MARKER; // throwaway initial val
      while !tokens.is_empty() {
        let inner_cc = tokens.front().unwrap().get_catcode();
        if inner_cc == Catcode::PARAM || inner_cc == Catcode::ARG {
          break;
        }
        let d = tokens.pop_front().unwrap();
        if !(pc == Catcode::SPACE && inner_cc == Catcode::SPACE) {
          // BUT collapse whitespace!
          delim.push(d);
        }
        pc = inner_cc;
      }
      // Found text that marks the end of the parameter
      if !delim.is_empty() {
        let extra = Tokens::new(delim);
        params.push(
          Parameter {
            name: arena::pin_static("Until"),
            spec: arena::pin(format!("Until:{extra}")),
            extra: vec![extra],
            ..Parameter::default()
          }
          .init()?,
        );
      } else if tokens.len() == 1 && tokens.front().unwrap().get_catcode() == Catcode::PARAM {
        // Special case: trailing sole # => delimited by next opening brace.
        tokens.pop_front();
        params.push(Parameter::new("UntilBrace", "UntilBrace", None)?);
      } else {
        // Nothing? Just a plain parameter.
        params.push(Parameter::new("Plain", "{}", None)?);
      }
    } else {
      // Initial delimiting text is required.
      let mut lit: Vec<Token> = vec![t];
      while !tokens.is_empty() {
        let lit_cc = tokens.front().unwrap().get_catcode();
        if lit_cc == Catcode::PARAM || lit_cc == Catcode::ARG {
          break;
        }
        lit.push(tokens.pop_front().unwrap());
      }
      let expected = Tokens::new(lit);
      params.push(
        Parameter {
          name: arena::pin_static("Match"),
          spec: arena::pin(s!("Match:{expected}")),
          extra: vec![expected],
          novalue: true,
          ..Parameter::default()
        }
        .init()?,
      );
    }
  }
  // return (@params ? LaTeXML::Core::Parameters->new(@params) : undef);
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(params)))
  }
}

pub fn do_def(globally: bool, cs: Token, params: Tokens, body: Tokens) -> Result<()> {
  let paramlist = parse_def_parameters(&cs, params)?;
  let scope = if globally { Some(Scope::Global) } else { None };
  state::install_definition(
    Expandable::new(
      cs,
      paramlist,
      Some(ExpansionBody::Tokens(body)),
      Some(ExpandableOptions {
        nopack_parameters: true,
        ..ExpandableOptions::default()
      }),
    )?,
    scope,
  );
  after_assignment();
  Ok(())
}

// Kinda rough: We don't really keep track of modes as carefully as TeX does.
// We'll assume that a box is horizontal if there's anything at all,
// but it's not a vbox (!?!?)
pub fn classify_box(boxnum: Number) -> Result<&'static str> {
  with_value(&s!("box{}", boxnum.value_of()), |val_opt| {
    Ok(match val_opt {
      Some(Stored::Digested(ref d)) => match d.data() {
        DigestedData::Whatsit(ref w)
          if w.borrow().definition == lookup_definition(&T_CS!("\\vbox"))?.unwrap() =>
        {
          "vbox"
        },
        _ => "hbox",
      },
      _ => "",
    })
  })
}

const MATH_CLASS_ROLE: [&str; 8] = ["", "BIGOP", "BINOP", "RELOP", "OPEN", "CLOSE", "PUNCT", ""];

/// Properties for a decoded math character, mirroring Perl's decodeMathChar return
#[derive(Debug, Clone, Default)]
pub struct MathCharProps {
  pub role: Option<String>,
  pub glyph: Option<char>,
  pub meaning: Option<String>,
  pub name: Option<String>,
  pub stretchy: Option<String>,
  pub need_scriptpos: bool,
  pub need_mathstyle: bool,
  pub scriptpos: Option<String>,
  pub mathstyle: Option<String>,
}

impl MathCharProps {
  /// Convert need_scriptpos/need_mathstyle flags to actual values based on display mode
  pub fn resolve_style_props(&mut self) {
    let in_display = state::lookup_string("IN_MATH_DISPLAY") == "true"
      || lookup_font()
        .map(|f| f.get_mathstyle().map(|s| s.as_ref()) == Some("display"))
        .unwrap_or(false);
    if self.need_scriptpos {
      self.scriptpos = Some(if in_display { "mid" } else { "post" }.to_string());
    }
    if self.need_mathstyle {
      self.mathstyle = Some(if in_display { "display" } else { "text" }.to_string());
    }
  }

  /// Insert all properties into a HashMap for Tbox construction
  pub fn into_props_map(self) -> HashMap<&'static str, Stored> {
    let mut props = HashMap::default();
    if let Some(role) = self.role {
      props.insert("role", Stored::String(arena::pin(role)));
    }
    if let Some(meaning) = self.meaning {
      props.insert("meaning", Stored::String(arena::pin(meaning)));
    }
    if let Some(name) = self.name {
      props.insert("name", Stored::String(arena::pin(name)));
    }
    if let Some(stretchy) = self.stretchy {
      props.insert("stretchy", Stored::String(arena::pin(stretchy)));
    }
    if let Some(scriptpos) = self.scriptpos {
      props.insert("scriptpos", Stored::String(arena::pin(scriptpos)));
    }
    if let Some(mathstyle) = self.mathstyle {
      props.insert("mathstyle", Stored::String(arena::pin(mathstyle)));
    }
    props
  }
}

/// Lookup Unicode math properties for a character, mirroring Perl's %math_props in Unicode.pm
pub fn unicode_math_properties(c: char) -> Option<MathCharProps> {
  // The struct fields: role, meaning, name, stretchy, need_scriptpos, need_mathstyle
  // (glyph is set separately)
  let (role, meaning, name, stretchy, need_sp, need_ms) = match c {
    // Digits
    '0'..='9' => ("NUMBER", Some(c.to_string()), None, None, false, false),
    // ASCII operators and punctuation
    '=' => ("RELOP", Some("equals".into()), None, None, false, false),
    '+' => ("ADDOP", Some("plus".into()), None, None, false, false),
    '-' => ("ADDOP", Some("minus".into()), None, None, false, false),
    '*' => ("MULOP", Some("times".into()), None, None, false, false),
    '/' => ("MULOP", Some("divide".into()), None, None, false, false),
    '!' => ("POSTFIX", Some("factorial".into()), None, None, false, false),
    ',' => ("PUNCT", None, None, None, false, false),
    '.' => ("PERIOD", None, None, None, false, false),
    ';' => ("PUNCT", None, None, None, false, false),
    ':' => ("METARELOP", None, Some("colon".into()), None, false, false),
    '|' => ("VERTBAR", None, None, Some("false".into()), false, false),
    '<' => ("RELOP", Some("less-than".into()), None, None, false, false),
    '>' => ("RELOP", Some("greater-than".into()), None, None, false, false),
    '(' => ("OPEN", None, None, Some("false".into()), false, false),
    ')' => ("CLOSE", None, None, Some("false".into()), false, false),
    '[' => ("OPEN", None, None, Some("false".into()), false, false),
    ']' => ("CLOSE", None, None, Some("false".into()), false, false),
    '{' => ("OPEN", None, None, Some("false".into()), false, false),
    '}' => ("CLOSE", None, None, Some("false".into()), false, false),
    '&' => ("ADDOP", Some("and".into()), None, None, false, false),
    '%' => ("POSTFIX", Some("percent".into()), None, None, false, false),
    '$' => ("OPERATOR", Some("currency-dollar".into()), None, None, false, false),
    '?' => ("UNKNOWN", None, None, None, false, false),
    // Backslash
    '\\' => ("ADDOP", Some("set-minus".into()), None, None, false, false),
    // Latin-1 supplement
    '\u{00AC}' => ("BIGOP", Some("not".into()), None, None, false, false),       // ¬ \neg, \lnot
    '\u{00B1}' => ("ADDOP", Some("plus-or-minus".into()), None, None, false, false), // ± \pm
    '\u{00D7}' => ("MULOP", Some("times".into()), None, None, false, false),     // × \times
    '\u{00F7}' => ("MULOP", Some("divide".into()), None, None, false, false),    // ÷ \div
    // General symbols
    '\u{2020}' => ("MULOP", None, None, None, false, false),  // † \dagger
    '\u{2021}' => ("MULOP", None, None, None, false, false),  // ‡ \ddagger
    '\u{2032}' => ("SUPOP", None, None, None, false, false),  // ′ \prime
    '\u{2061}' => ("APPLYOP", None, Some("".into()), None, false, false), // ⁡ function application
    '\u{2062}' => ("MULOP", Some("times".into()), Some("".into()), None, false, false), // ⁢ invisible times
    '\u{2063}' => ("PUNCT", None, Some("".into()), None, false, false),  // ⁣ invisible separator
    '\u{2064}' => ("ADDOP", Some("plus".into()), Some("".into()), None, false, false), // ⁤ invisible plus
    '\u{210F}' => ("ID", Some("Planck-constant-over-2-pi".into()), None, None, false, false), // ℏ \hbar
    '\u{2111}' => ("OPFUNCTION", Some("imaginary-part".into()), None, None, false, false), // ℑ \Im
    '\u{2118}' => ("OPFUNCTION", Some("Weierstrass-p".into()), None, None, false, false), // ℘ \wp
    '\u{211C}' => ("OPFUNCTION", Some("real-part".into()), None, None, false, false),     // ℜ \Re
    // Arrows
    '\u{2190}' => ("ARROW", None, None, None, false, false), // ← \leftarrow
    '\u{2191}' => ("ARROW", None, Some("uparrow".into()), None, false, false), // ↑ \uparrow
    '\u{2192}' => ("ARROW", None, None, None, false, false), // → \rightarrow
    '\u{2193}' => ("ARROW", None, Some("downarrow".into()), None, false, false), // ↓ \downarrow
    '\u{2194}' => ("METARELOP", None, None, None, false, false), // ↔ \leftrightarrow
    '\u{2195}' => ("ARROW", None, Some("updownarrow".into()), None, false, false), // ↕ \updownarrow
    '\u{2196}' => ("ARROW", None, None, None, false, false), // ↖ \nwarrow
    '\u{2197}' => ("ARROW", None, None, None, false, false), // ↗ \nearrow
    '\u{2198}' => ("ARROW", None, None, None, false, false), // ↘ \searrow
    '\u{2199}' => ("ARROW", None, None, None, false, false), // ↙ \swarrow
    '\u{219D}' => ("ARROW", Some("leads-to".into()), None, None, false, false), // ⇝ \leadsto
    '\u{21A6}' => ("ARROW", Some("maps-to".into()), None, None, false, false),  // ↦ \mapsto
    '\u{21A9}' => ("ARROW", None, None, None, false, false), // ↩ \hookleftarrow
    '\u{21AA}' => ("ARROW", None, None, None, false, false), // ↪ \hookrightarrow
    '\u{21BC}' => ("ARROW", None, None, None, false, false), // ↼ \leftharpoonup
    '\u{21BD}' => ("ARROW", None, None, None, false, false), // ⇀ \leftharpoondown
    '\u{21C0}' => ("ARROW", None, None, None, false, false), // ⇁ \rightharpoonup
    '\u{21C1}' => ("ARROW", None, None, None, false, false), // ⇂ \rightharpoondown
    '\u{21CC}' => ("METARELOP", None, None, None, false, false), // ⇌ \rightleftharpoons
    '\u{21D0}' => ("ARROW", None, None, None, false, false), // ⇐ \Leftarrow
    '\u{21D1}' => ("ARROW", None, Some("Uparrow".into()), None, false, false), // ⇑ \Uparrow
    '\u{21D2}' => ("ARROW", None, None, None, false, false), // ⇒ \Rightarrow
    '\u{21D3}' => ("ARROW", None, Some("Downarrow".into()), None, false, false), // ⇓ \Downarrow
    '\u{21D4}' => ("METARELOP", Some("iff".into()), None, None, false, false), // ⇔ \Leftrightarrow
    '\u{21D5}' => ("ARROW", None, Some("Updownarrow".into()), None, false, false), // ⇕ \Updownarrow
    // Quantifiers and set theory
    '\u{2200}' => ("BIGOP", Some("for-all".into()), None, None, false, false),    // ∀ \forall
    '\u{2202}' => ("DIFFOP", Some("partial-differential".into()), None, None, false, false), // ∂ \partial
    '\u{2203}' => ("BIGOP", Some("exists".into()), None, None, false, false),     // ∃ \exists
    '\u{2205}' => ("ID", Some("empty-set".into()), None, None, false, false),     // ∅ \emptyset
    '\u{2207}' => ("OPERATOR", None, None, None, false, false),                   // ∇ \nabla
    '\u{2208}' => ("RELOP", Some("element-of".into()), None, None, false, false), // ∈ \in
    '\u{2209}' => ("RELOP", Some("not-element-of".into()), None, None, false, false), // ∉ \notin
    '\u{220B}' => ("RELOP", Some("contains".into()), None, None, false, false),   // ∋ \ni
    // Big operators
    '\u{220F}' => ("SUMOP", Some("product".into()), None, None, true, true),      // ∏ \prod
    '\u{2210}' => ("SUMOP", Some("coproduct".into()), None, None, true, true),    // ∐ \coprod
    '\u{2211}' => ("SUMOP", Some("sum".into()), None, None, true, true),          // ∑ \sum
    // Arithmetic operators
    '\u{2213}' => ("ADDOP", Some("minus-or-plus".into()), None, None, false, false), // ∓ \mp
    '\u{2216}' => ("ADDOP", Some("set-minus".into()), None, None, false, false),     // ∖ \setminus
    '\u{2217}' => ("MULOP", Some("times".into()), None, None, false, false),         // ∗ \ast
    '\u{2218}' => ("MULOP", Some("compose".into()), None, None, false, false),       // ∘ \circ
    '\u{2219}' => ("MULOP", None, None, None, false, false),                         // ∙ \bullet
    '\u{221A}' => ("OPERATOR", Some("square-root".into()), None, None, false, false), // √ \surd
    '\u{221D}' => ("RELOP", Some("proportional-to".into()), None, None, false, false), // ∝ \propto
    '\u{221E}' => ("ID", Some("infinity".into()), None, None, false, false),         // ∞ \infty
    '\u{2223}' => ("VERTBAR", None, None, None, false, false),                       // ∣ \mid
    '\u{2225}' => ("VERTBAR", Some("parallel-to".into()), Some("||".into()), None, false, false), // ∥ \parallel
    // Logical operators
    '\u{2227}' => ("ADDOP", Some("and".into()), None, None, false, false),       // ∧ \land, \wedge
    '\u{2228}' => ("ADDOP", Some("or".into()), None, None, false, false),        // ∨ \lor, \vee
    '\u{2229}' => ("ADDOP", Some("intersection".into()), None, None, false, false), // ∩ \cap
    '\u{222A}' => ("ADDOP", Some("union".into()), None, None, false, false),     // ∪ \cup
    // Integrals
    '\u{222B}' => ("INTOP", Some("integral".into()), None, None, false, true),   // ∫ \int
    '\u{222E}' => ("INTOP", Some("contour-integral".into()), None, None, false, true), // ∮ \oint
    // Relations
    '\u{223C}' => ("RELOP", Some("similar-to".into()), None, None, false, false), // ∼ \sim
    '\u{2240}' => ("MULOP", None, None, None, false, false),                      // ≀ \wr
    '\u{2243}' => ("RELOP", Some("similar-to-or-equals".into()), None, None, false, false), // ≃ \simeq
    '\u{2245}' => ("RELOP", Some("approximately-equals".into()), None, None, false, false), // ≅ \cong
    '\u{2248}' => ("RELOP", Some("approximately-equals".into()), None, None, false, false), // ≈ \approx
    '\u{224D}' => ("RELOP", Some("asymptotically-equals".into()), None, None, false, false), // ≍ \asymp
    '\u{2250}' => ("RELOP", Some("approaches-limit".into()), None, None, false, false), // ≐ \doteq
    '\u{2260}' => ("RELOP", Some("not-equals".into()), None, None, false, false), // ≠ \neq
    '\u{2261}' => ("RELOP", Some("equivalent-to".into()), None, None, false, false), // ≡ \equiv
    '\u{2264}' => ("RELOP", Some("less-than-or-equals".into()), None, None, false, false), // ≤ \leq
    '\u{2265}' => ("RELOP", Some("greater-than-or-equals".into()), None, None, false, false), // ≥ \geq
    '\u{226A}' => ("RELOP", Some("much-less-than".into()), None, None, false, false), // ≪ \ll
    '\u{226B}' => ("RELOP", Some("much-greater-than".into()), None, None, false, false), // ≫ \gg
    '\u{227A}' => ("RELOP", Some("precedes".into()), None, None, false, false), // ≺ \prec
    '\u{227B}' => ("RELOP", Some("succeeds".into()), None, None, false, false), // ≻ \succ
    // Subset/superset
    '\u{2282}' => ("RELOP", Some("subset-of".into()), None, None, false, false), // ⊂ \subset
    '\u{2283}' => ("RELOP", Some("superset-of".into()), None, None, false, false), // ⊃ \supset
    '\u{2286}' => ("RELOP", Some("subset-of-or-equals".into()), None, None, false, false), // ⊆ \subseteq
    '\u{2287}' => ("RELOP", Some("superset-of-or-equals".into()), None, None, false, false), // ⊇ \supseteq
    '\u{228E}' => ("ADDOP", None, None, None, false, false), // ⊎ \uplus
    '\u{228F}' => ("RELOP", Some("square-image-of".into()), None, None, false, false), // ⊏ \sqsubset
    '\u{2290}' => ("RELOP", Some("square-original-of".into()), None, None, false, false), // ⊐ \sqsupset
    '\u{2291}' => ("RELOP", Some("square-image-of-or-equals".into()), None, None, false, false), // ⊑ \sqsubseteq
    '\u{2292}' => ("RELOP", Some("square-original-of-or-equals".into()), None, None, false, false), // ⊒ \sqsupseteq
    '\u{2293}' => ("ADDOP", Some("square-intersection".into()), None, None, false, false), // ⊓ \sqcap
    '\u{2294}' => ("ADDOP", Some("square-union".into()), None, None, false, false), // ⊔ \sqcup
    // Circled operators
    '\u{2295}' => ("ADDOP", Some("direct-sum".into()), None, None, false, false), // ⊕ \oplus
    '\u{2296}' => ("ADDOP", Some("symmetric-difference".into()), None, None, false, false), // ⊖ \ominus
    '\u{2297}' => ("MULOP", Some("tensor-product".into()), None, None, false, false), // ⊗ \otimes
    '\u{2298}' => ("MULOP", None, None, None, false, false), // ⊘ \oslash
    '\u{2299}' => ("MULOP", Some("direct-product".into()), None, None, false, false), // ⊙ \odot
    // Turnstiles
    '\u{22A2}' => ("METARELOP", Some("proves".into()), None, None, false, false), // ⊢ \vdash
    '\u{22A3}' => ("METARELOP", Some("does-not-prove".into()), None, None, false, false), // ⊣ \dashv
    '\u{22A4}' => ("ADDOP", Some("top".into()), None, None, false, false), // ⊤ \top
    '\u{22A5}' => ("ADDOP", Some("bottom".into()), None, None, false, false), // ⊥ \bot
    '\u{22A7}' => ("RELOP", Some("models".into()), None, None, false, false), // ⊧ \models
    '\u{22B2}' => ("ADDOP", Some("subgroup-of".into()), None, None, false, false), // ⊲ \lhd
    '\u{22B3}' => ("ADDOP", Some("contains-as-subgroup".into()), None, None, false, false), // ⊳ \rhd
    '\u{22B4}' => ("ADDOP", Some("subgroup-of-or-equals".into()), None, None, false, false), // ⊴ \unlhd
    '\u{22B5}' => ("ADDOP", Some("contains-as-subgroup-or-equals".into()), None, None, false, false), // ⊵ \unrhd
    // Big operators (N-ary)
    '\u{22C0}' => ("SUMOP", Some("and".into()), None, None, true, true),         // ⋀ \bigwedge
    '\u{22C1}' => ("SUMOP", Some("or".into()), None, None, true, true),          // ⋁ \bigvee
    '\u{22C2}' => ("SUMOP", Some("intersection".into()), None, None, true, true), // ⋂ \bigcap
    '\u{22C3}' => ("SUMOP", Some("union".into()), None, None, true, true),       // ⋃ \bigcup
    '\u{22C4}' => ("ADDOP", None, None, None, false, false), // ⋄ \diamond
    '\u{22C5}' => ("MULOP", None, None, None, false, false), // ⋅ \cdot
    '\u{22C6}' => ("MULOP", None, None, None, false, false), // ⋆ \star
    '\u{22C8}' => ("RELOP", None, None, None, false, false), // ⋈ \bowtie
    '\u{22EF}' => ("ID", None, None, None, false, false),    // ⋯ \cdots
    '\u{22F1}' => ("ID", None, None, None, false, false),    // ⋱ \ddots
    // Delimiters
    '\u{2308}' => ("OPEN", None, Some("lceil".into()), Some("false".into()), false, false),  // ⌈ \lceil
    '\u{2309}' => ("CLOSE", None, Some("rceil".into()), Some("false".into()), false, false), // ⌉ \rceil
    '\u{230A}' => ("OPEN", None, Some("lfloor".into()), Some("false".into()), false, false), // ⌊ \lfloor
    '\u{230B}' => ("CLOSE", None, Some("rfloor".into()), Some("false".into()), false, false), // ⌋ \rfloor
    '\u{2322}' => ("RELOP", None, None, None, false, false), // ⌢ \frown
    '\u{2323}' => ("RELOP", None, None, None, false, false), // ⌣ \smile
    // Triangles
    '\u{25B3}' => ("ADDOP", None, None, None, false, false), // △ \bigtriangleup
    '\u{25B7}' => ("ADDOP", None, None, None, false, false), // ▷ \triangleright
    '\u{25B9}' => ("ADDOP", None, None, None, false, false), // ▹ \triangleright
    '\u{25BD}' => ("ADDOP", None, None, None, false, false), // ▽ \bigtriangledown
    '\u{25C1}' => ("ADDOP", None, None, None, false, false), // ◁ \triangleleft
    '\u{25C3}' => ("ADDOP", None, None, None, false, false), // ◃ \triangleleft
    '\u{25CB}' => ("MULOP", None, None, None, false, false), // ○ \bigcirc
    '\u{27C2}' => ("RELOP", Some("perpendicular-to".into()), None, None, false, false), // ⟂ \perp
    // Angle brackets
    '\u{27E8}' => ("OPEN", None, Some("langle".into()), Some("false".into()), false, false), // ⟨ \langle
    '\u{27E9}' => ("CLOSE", None, Some("rangle".into()), Some("false".into()), false, false), // ⟩ \rangle
    '\u{27EE}' => ("OPEN", None, Some("lgroup".into()), Some("false".into()), false, false), // ⟮ \lgroup
    '\u{27EF}' => ("CLOSE", None, Some("rgroup".into()), Some("false".into()), false, false), // ⟯ \rgroup
    // Long arrows
    '\u{27F5}' => ("ARROW", None, None, None, false, false), // ⟵ \longleftarrow
    '\u{27F6}' => ("ARROW", None, None, None, false, false), // ⟶ \longrightarrow
    '\u{27F7}' => ("METARELOP", None, None, None, false, false), // ⟷ \longleftrightarrow
    '\u{27F8}' => ("ARROW", None, None, None, false, false), // ⟸ \Longleftarrow
    '\u{27F9}' => ("ARROW", None, None, None, false, false), // ⟹ \Longrightarrow
    '\u{27FA}' => ("METARELOP", None, None, None, false, false), // ⟺ \Longleftrightarrow
    '\u{27FC}' => ("ARROW", None, None, None, false, false), // ⟼ \longmapsto
    // N-ary circled operators
    '\u{2A00}' => ("SUMOP", None, None, None, true, true),   // ⨀ \bigodot
    '\u{2A01}' => ("SUMOP", Some("direct-sum".into()), None, None, true, true), // ⨁ \bigoplus
    '\u{2A02}' => ("SUMOP", Some("tensor-product".into()), None, None, true, true), // ⨂ \bigotimes
    '\u{2A04}' => ("SUMOP", Some("symmetric-difference".into()), None, None, true, true), // ⨄ \biguplus
    '\u{2A06}' => ("SUMOP", Some("square-union".into()), None, None, true, true), // ⨆ \bigsqcup
    '\u{2A1D}' => ("RELOP", Some("join".into()), None, None, false, false), // ⨝ \Join
    '\u{2AAF}' => ("RELOP", Some("precedes-or-equals".into()), None, None, false, false), // ⪯ \preceq
    '\u{2AB0}' => ("RELOP", Some("succeeds-or-equals".into()), None, None, false, false), // ⪰ \succeq
    '\u{FF0F}' => ("OPFUNCTION", Some("not".into()), None, None, false, false), // ／ \not
    _ => return None,
  };
  Some(MathCharProps {
    role: Some(role.to_string()),
    glyph: None,
    meaning,
    name,
    stretchy,
    need_scriptpos: need_sp,
    need_mathstyle: need_ms,
    scriptpos: None,
    mathstyle: None,
  })
}

// Is this "fontinfo" stuff sufficient to maintain a math font "family" ??
// What we're really after is a connection to a font encoding mapping.
pub fn decode_math_char(mut n: u16) -> Result<MathCharProps> {
  let class: u16 = n / (16 * 256);
  n %= 16 * 256;
  let fam: u16 = n / 256;
  n %= 256;
  let font = lookup_value(&s!("textfont_{fam}")).unwrap_or_else(|| {
    lookup_value(&s!("scriptfont_{fam}"))
      .unwrap_or_else(|| lookup_value(&s!("scriptscriptfont_{fam}")).unwrap_or(Stored::Bool(false)))
  });
  let c = n as u8 as char;
  let class_role = MATH_CLASS_ROLE[class as usize];

  // Decode the glyph from the font encoding
  let glyph = with_font_info(&T_CS!(font.to_string()), |fontinfo| {
    let cinfo = if let Some(Stored::Font(ref info)) = fontinfo? {
      if let Some(ref data) = info.encoding {
        font::decode(n as u8, Some(data.to_string()), false)
      } else {
        Some(c)
      }
    } else {
      None
    };
    Ok::<Option<char>, latexml_core::Error>(cinfo)
  })?;

  // Look up unicode math properties for the decoded glyph
  let glyph_char = glyph.unwrap_or(c);
  let mut props = unicode_math_properties(glyph_char).unwrap_or_default();
  props.glyph = glyph;

  // Apply class-based role if no role from unicode properties, or class gives a specific role
  if !class_role.is_empty() {
    // Class-based role takes precedence when it's specific (not empty)
    // But unicode_math_properties may have a more specific role (e.g. SUMOP vs BIGOP)
    if props.role.is_none() {
      props.role = Some(class_role.to_string());
    } else if let Some(ref unicode_role) = props.role {
      // If class says BIGOP but unicode says SUMOP, keep SUMOP (more specific)
      // If class says something else, and unicode has a role, keep unicode's role
      if unicode_role == class_role || class_role == "BIGOP" {
        // keep the unicode role
      } else {
        props.role = Some(class_role.to_string());
      }
    }
  } else if props.role.is_none() {
    // No class role and no unicode role — try math_token_attributes
    with_value(&s!("math_token_attributes_{}", c), |charinfo| {
      if let Some(Stored::HashString(ref info)) = charinfo {
        let inner_role = &info["role"];
        if !inner_role.is_empty() {
          props.role = Some(inner_role.to_string());
        }
      }
    });
  }

  // Resolve need_scriptpos/need_mathstyle to actual values
  props.resolve_style_props();

  Ok(props)
}

/// Stomach-level hook for decoding math characters.
/// Called from stomach::invoke_token_simple when IN_MATH and mathcode is set.
/// Perl: decodeMathChar($mathcode, $meaning) in Stomach::invokeToken_simple
pub fn decode_math_char_for_stomach(
  mathcode: u16,
  meaning: Token,
) -> Result<Digested> {
  let props = decode_math_char(mathcode)?;
  let mut properties = SymHashMap::default();
  properties.insert("mode", Stored::String(*arena::MATH_SYM));
  if let Some(ref role) = props.role {
    properties.insert("role", Stored::String(arena::pin(role)));
  }
  if let Some(ref m) = props.meaning {
    properties.insert("meaning", Stored::String(arena::pin(m)));
  }
  if let Some(ref name) = props.name {
    properties.insert("name", Stored::String(arena::pin(name)));
  }
  if let Some(ref stretchy) = props.stretchy {
    properties.insert("stretchy", Stored::String(arena::pin(stretchy)));
  }
  let glyph_sym = if let Some(glyph) = props.glyph {
    arena::pin(&glyph.to_string())
  } else {
    meaning.get_sym()
  };
  let font = lookup_font().map(|f| {
    Rc::new(arena::with(glyph_sym, |s| f.specialize(s)))
  });
  Ok(Digested::from(Tbox::new(
    glyph_sym,
    font,
    None,
    Tokens!(meaning),
    properties,
  )))
}

/// Stomach-level counterpart to `read_box_contents`.
///
/// Perl: readBoxContents calls $stomach->beginMode($mode), then reads/digests tokens
/// in a loop until T_END, then calls $stomach->endMode($mode), returning a List.
/// This mirrors that approach: beginMode, digest loop, endMode.
pub fn predigest_box_contents_with_mode(
  _tokens: ArgWrap,
  mode: &str,
) -> Result<Option<Digested>> {
  // Perl: $stomach->beginMode($mode);
  stomach::begin_mode(mode)?;
  // Perl: local @LaTeXML::LIST = (); read tokens via readXToken until T_END
  let mut box_list: Vec<Digested> = Vec::new();
  while let Some(t) = match gullet::get_pending_comment() {
    Some(comment) => Some(comment),
    None => gullet::read_x_token(Some(true), false, None)?,
  } {
    if t.get_catcode() == Catcode::END {
      break;
    }
    let invoked = stomach::invoke_token(&t)?;
    box_list.extend(invoked);
  }
  // Perl: $stomach->endMode($mode);
  stomach::end_mode(mode)?;
  let mut list = List::new(box_list);
  list.mode = Some(match mode {
    "restricted_horizontal" => TexMode::Text,
    "internal_vertical" => TexMode::Text,
    _ => TexMode::Text,
  });
  Ok(Some(list.into()))
}

/// Legacy predigest — kept for cases that don't need full body reading.
pub fn predigest_box_contents(_tokens: ArgWrap) -> Result<Option<Digested>> {
  let mut contents = stomach::invoke_token(&T_BEGIN!())?;
  if contents.is_empty() {
    Ok(None)
  } else {
    Ok(Some(contents.remove(0)))
  }
}

/// Perl: revertSpec($whatsit, $keyword)
/// If whatsit has property $keyword, return Explode($keyword) ++ Revert($value)
pub fn revert_spec(whatsit: &Whatsit, keyword: &str) -> Vec<Token> {
  if let Some(value) = whatsit.get_property(keyword) {
    // Explode the keyword string into T_OTHER tokens
    let mut tokens: Vec<Token> = keyword.chars()
      .map(|c| { let s = c.to_string(); T_OTHER!(s) }).collect();
    // Revert the stored value to tokens
    let val_str = value.to_attribute();
    tokens.extend(val_str.chars()
      .map(|c| { let s = c.to_string(); T_OTHER!(s) }));
    tokens
  } else {
    Vec::new()
  }
}

pub fn p_revert<T>(arg: T) -> Result<Tokens>
where T: Sized + Object {
  set_dual_branch("presentation");
  let result = arg.revert();
  expire_dual_branch();
  result
}

pub fn c_revert<T>(arg: T) -> Result<Tokens>
where T: Sized + Object {
  set_dual_branch("content");
  let result = arg.revert();
  expire_dual_branch();
  result
}

/// This attempts to be a generalize vbox construction;
///
/// The idea is to receeive block-like material, possibly wrapped in appropriate
/// container which gets attributes.
///
/// The contents are constructed in an ltx:_CaptureBlock_ element,
/// designed to accept all reasonable block material from several levels,
/// and then determine which container element is most apprpriate for both the conent & context
/// from block, logical-block or sectional-block, or the inline- variants.
/// Perl: isVAttached — checks if node or any single-child descendant has 'vattach'
fn is_v_attached(node: &Node) -> bool {
  let mut current = node.clone();
  loop {
    if current.get_attribute("vattach").is_some() {
      return true;
    }
    let children: Vec<_> = current
      .get_child_nodes()
      .into_iter()
      .filter(|n| matches!(n.get_type(), Some(NodeType::ElementNode)))
      .collect();
    if children.len() != 1 {
      return false;
    }
    current = children[0].clone();
  }
}

pub fn insert_block(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
) -> Result<Vec<Node>> {
  // Create something like:
  // "<ltx:inline-block vattach='$vattach' height='#height'>#2</ltx:inline-block>"
  let context_opt = document.get_element(); // Where we originally start inserting.
  if context_opt.is_none() {
    // edge case: if we start the doc with a block, the context is empty
    document.absorb(contents, None)?;
    return Ok(Vec::new());
  }
  let mut context = context_opt.unwrap();
  let mut context_tag = document::get_node_qname(&context);
  // svg is slightly tricky
  let (is_svg, is_xmath, is_xmtext) = arena::with(context_tag, |tag| {
    (
      tag.starts_with("svg:"),
      tag.starts_with("ltx:XM"),
      tag == "ltx:XMText",
    )
  });
  let mut ignorable_attr = is_svg || block_attr.is_empty(); // if we do not REQUIRE the attributes
  if is_xmath && !is_xmtext {
    // but math always needs this
    context = document.open_element("ltx:XMText", None, None)?;
    context_tag = document::get_node_qname(&context);
  }
  let is_inline = is_svg || document::can_contain(&context, "#PCDATA");
  let mut container_attr = block_attr.clone();
  container_attr.insert("_vertical_mode_".to_string(), "true".to_string());
  let mut container = document.open_element("ltx:_CaptureBlock_", Some(container_attr), None)?;
  document.absorb(contents, None)?;

  let mut nodes = content_nodes(&container);
  let node_tags = nodes
    .iter()
    .map(document::get_node_qname)
    .collect::<Vec<_>>();
  let nnodes = nodes.len();
  document.close_to_node(&container, true)?;
  document.close_node(&container)?;
  document.close_to_node(&context, true)?;

  // Perl: Hack: apparently TeX doesn't shift (vattach) a single node in a vbox/vtop/...
  let mut block_attr = block_attr;
  let mut ignorable_attr = ignorable_attr;
  if nnodes == 1 && block_attr.contains_key("vattach") && is_v_attached(&nodes[0]) {
    container.remove_attribute("vattach")?;
    block_attr.remove("vattach");
    ignorable_attr = is_svg || block_attr.is_empty();
  }

  if nnodes < 1 {
    // Insertion came up empty?
    document.remove_node(container); // then remove the new block entirely
    return Ok(nodes);
  } else if ignorable_attr
    && node_tags
      .iter()
      .all(|tag| document::can_contain_qsym(context_tag, *tag))
  {
    // No attributes, contents allowed in context?
    document.unwrap_nodes(container)?; // No container needed, at all.
    return Ok(nodes);
  } else if nnodes == 1 {
    if document::can_contain_qsym(context_tag, node_tags[0])
      && (ignorable_attr
        || block_attr
          .keys()
          .all(|key| document::sym_can_have_attribute(node_tags[0], arena::pin(key))))
    {
      // IF: Single node, allowed in context & accepts attributes
      // THEN: Add attributes and unwrap the single node
      for (k, v) in block_attr.iter() {
        document.set_attribute(&mut nodes[0], k, v)?;
      }
      document.unwrap_nodes(container)?;
      return Ok(nodes);
    } else if let Some(newcontainer) = document::sym_can_contain_somehow(context_tag, node_tags[0])
    {
      if ignorable_attr
        || block_attr.keys().all(|key| {
          newcontainer
            .map(|nc| document::sym_can_have_attribute(nc, arena::pin(key)))
            .unwrap_or(false)
        })
      {
        if let Some(nc) = newcontainer {
          // rename the capture to that container
          document.rename_node_qsym(container, nc, true)?;
          return Ok(nodes);
        }
      }
    }
  }
  // This jagged conditional is a "code smell", due to the difficulty of refactoring
  // the in-conditional-assignments from Perl.

  // Otherwise, rename the capture
  // MAY need foreignObject wrapper
  if is_svg
    && node_tags
      .iter()
      .any(|tag| arena::with(*tag, |tag_str| tag_str.starts_with("ltx:")))
  {
    context = document
      .wrap_nodes("svg:foreignObject", vec![container.clone()])?
      .expect("foreign object wrap should always succeed in SVG");
    context_tag = document::get_node_qname(&context);
  }
  let candidates = if is_inline {
    [
      "ltx:inline-block",
      "ltx:inline-logical-block",
      "ltx:inline-sectional-block",
    ]
    .map(arena::pin_static)
    .to_vec()
  } else {
    [
      "ltx:block",
      "ltx:logical-block",
      "ltx:sectional-block",
      "ltx:figure",
    ]
    .map(arena::pin_static)
    .to_vec()
  };
  let filtered_candidates = candidates
    .into_iter()
    .filter(|candidate| {
      node_tags
        .iter()
        .all(|tag| document::sym_can_contain_somehow(*candidate, *tag).is_some())
    })
    .collect::<Vec<_>>();
  // and are allowed in the context
  let allowed_candidates = filtered_candidates
    .iter()
    .filter(|candidate| document::can_contain_qsym(context_tag, **candidate))
    .copied()
    .collect::<Vec<_>>();
  if let Some(final_tag) = allowed_candidates
    .first()
    .map_or(filtered_candidates.first(), Some)
  {
    // Rename the capture to the correct container
    // TODO: There is an arena code smell here. The `Model` interface needs to become lock-free
    // where Symbol tickets and &str are equally intuitive to use without runtime panics from
    // arena mutability exceptions.
    document.rename_node(container, &arena::to_string(*final_tag), true)?;
  } else {
    // we didn't know what to do?
    let message = arena::with(context_tag, |ctxt_str| {
      s!(
        "Did not find a block-like candidate in {} (with attributes ({})",
        ctxt_str,
        block_attr
          .iter()
          .map(|(k, v)| s!("{k}={v}"))
          .collect::<Vec<_>>()
          .join(";")
      )
    });
    Warn!("malformed", "_CaptureBlock_", message);
    document.rename_node(container, "ltx:block", true)?;
  }
  Ok(nodes)
}

pub fn cleanup_math(document: &mut Document, mathnode: Node) -> Result<()> {
  // Cleanup ltx:Math elements; particularly if they aren't "really" math.
  // But record the oddity with class=ltx_markedasmath

  // If the Math ONLY contains XMath/XMText, it apparently isn't math at all!?!
  if document
    .findnodes("ltx:XMath/ltx:*[local-name() != 'XMText']", Some(&mathnode))
    .is_empty()
  {
    // So unwrap down to the contents of the XMText's.
    let xmtexts = mathnode.get_child_nodes().into_iter().flat_map(|child| {
      child
        .get_child_nodes()
        .into_iter()
        .flat_map(|grandhcild| grandhcild.get_child_nodes())
    });
    let mut texts = vec![];
    for mut text in xmtexts {
      text = if text.get_type() == Some(NodeType::ElementNode) {
        // Make sure we've got an element
        text
      } else {
        document.wrap_nodes("ltx:text", vec![text])?.unwrap()
      };
      // Now record that it originally was marked as math
      document.add_class(&mut text, "ltx_markedasmath")?;
      texts.push(text)
    }
    document.replace_node(mathnode.clone(), texts)?; // and replace the whole Math with the pieces
  } else {
    // Cleanup any remaining XMTexts
    cleanup_xmtext_outer(document, &mathnode)?;
  }
  Ok(())
}

// Here's for an inverse case: when an XMText isn't "really" just text
// if it only contains an Math  ORR, a tabular with only Math in the cells?
// First case: pull it back into the math, but in an XMWrap to isolate it for parsing.
// Should we just pull any mixed text math up or only a single Math?
// For the tabular case, convert it to an XMArray.

// Note that normally, we'd do afterClose on ltx:XMText,
// but since the ltx:XMText closes before the outer ltx:Math,
// we would keep cleanup_Math from recognizing the trivial case of
// a single ltx:tabular in an equation (perverse, but people do that).
// So, we put this one on ltx:Math also, and scan for any contained XMText to fixup.

fn cleanup_xmtext_outer(document: &mut Document, math_node: &Node) -> Result<()> {
  for text_node in document.findnodes("descendant::ltx:XMText", Some(math_node)) {
    cleanup_xmtext(document, text_node)?;
  }
  Ok(())
}

fn cleanup_xmtext(document: &mut Document, mut text_node: Node) -> Result<()> {
  // We're really only interested in reducing nested math, right?
  // But actually also collapsing ltx:XMText/ltx:text
  // Apply "outer" simplifications: remove ltx:text or ltx:p wrappings.

  // A single "simple" element, with a single child
  let mut children;
  loop {
    children = text_node.get_child_nodes();
    if (children.len() != 1)
      || document
        .findnodes(
          "ltx:text | ltx:inline-block[count(*)=1] | ltx:p",
          Some(&text_node),
        )
        .is_empty()
    {
      break;
    }
    let child = children.pop().unwrap();
    document.copy_node_font(&child, &mut text_node)?;
    for (key, value) in child.get_attributes() {
      // Copy the child's attributes (should Merge!!)
      if key != "xml:id" {
        text_node.set_attribute(&key, &value)?;
      }
    }
    document.unwrap_nodes(child)?;
  }

  // Now apply a simplifying rule for nested Math
  // If the XMText contains a single Math, pull it's content up in
  if children.len() == 1 && !document.findnodes("ltx:Math", Some(&text_node)).is_empty() {
    // Replace XMText by XMWrap/*  (this should preserve the parse?)
    document.rename_node(text_node, "ltx:XMWrap", false)?; // text_node =
    let first_child = children.pop().unwrap();
    let first_granchildren = first_child.get_child_nodes();
    document.replace_node(
      first_child,
      first_granchildren
        .into_iter()
        .flat_map(|grandchild| grandchild.get_child_nodes())
        .collect(),
    )?;
  // # # RISKY!!!! If SOME nodes are math...
  // # # pull the whole sequence up, unwrap the math and putting the rest back in XMText.
  // # # Even with the XMWrap, this seems to wreak havoc on parsing and structure?
  // # if(document.findnodes('ltx:Math',$text_node)){
  // #   # Replace XMText by XMWrap/*  (this should preserve the parse?)
  // #   $text_node=document.renameNode($text_node,'ltx:XMWrap');
  // #   foreach my $child (@children){
  // #     if($model->getNodeQName($child) eq 'ltx:Math'){
  // #       document.replaceNode($child,map($_->childNodes,$child->childNodes)); }
  // #     else {
  // #       document.wrapNodes('ltx:XMText',$child); }}}
  // If a single tabular that ONLY(?) contains Math, turn into an XMArray
  // Well, a tabular REALLY shouldn't be in math;
  // How much math should determine the switch?
  // [will alignment attributes be lost?]
  } else if children.len() == 1
    && model::with_node_qname(children.first().as_ref().unwrap(), |qname| {
      qname == "ltx:tabular"
    })
  //// Should we ALWAYS do this, or just for some minimal amount of math???
  ////        && !document.findnodes('ltx:tabular/ltx:tr/ltx:td/text()'
  ////                                 .' | ltx:tabular/ltx:tbody/ltx:tr/ltx:td/text()'
  ////                                 .' | ltx:tabular/ltx:tr/ltx:td[not(ltx:Math)]'
  ////                                 .' | ltx:tabular/ltx:tbody/ltx:tr/ltx:td[not(ltx:Math)]',
  ////                                 $text_node)
  {
    // Stub: tabular→XMArray conversion in math mode is complex and deferred.
    // Perl code unwraps tbody, renames nodes to XMArray/XMRow/XMCell.
    // // First step is remove any ltx:tbody from the tabular!
    // foreach my $tb (document.findnodes('ltx:tabular/ltx:tbody', $text_node)) {
    //   document.unwrapNodes($tb); }
    // // Now, we can start replacing tabular=>XMArray, tr=>XMRow, td=>XMCell
    // my $table = document.renameNode($children[0], 'ltx:XMArray');
    // foreach my $row ($table->childNodes) {
    //   $row = document.renameNode($row, 'ltx:XMRow');
    //   foreach my $cell ($row->childNodes) {
    //     $cell = document.renameNode($cell, 'ltx:XMCell');
    //     foreach my $m ($cell->childNodes) {
    //       if ($model->getNodeQName($m) eq 'ltx:Math') {    // Math cell, unwrap
    // the Math/XMath layer         document.replaceNode($m,
    // map { $_->childNodes } $m->childNodes); }       else
    // {                                           // Otherwise, wrap whatever it
    // is in an XMText         document.wrapNodes('ltx:
    // XMText', $m); } } } }
    // And now we don't need the XMText any more.
    // foreach my $attr ($text_node->attributes) {    // Copy the child's
    // attributes (should Merge!!)
    //   $table->setAttribute($attr->nodeName => $attr->getValue); }
    // my $newtable = document.unwrapNodes($text_node);
    // if (my $id = $text_node->getAttribute('xml:id')) {
    //   document.unRecordID($id);
    //   document.recordID($id, $newtable); } }
  }
  Ok(())
}

//======================================================================
// A random collection of utility functions.
// [maybe need to do some reorganization?]
// Since this is used for textual tokens, typically to split author lists,
// we don't split within braces or math
pub fn split_tokens(tokens: Tokens, delims: Vec<Token>) -> Vec<Tokens> {
  let mut items = Vec::new();
  let mut toks = Vec::new();
  if !tokens.is_empty() {
    let tokens = tokens.unlist();
    let mut tokens_iter = tokens.into_iter();
    while let Some(t) = tokens_iter.next() {
      if delims.iter().any(|d| d == &t) {
        items.push(Tokens::new(std::mem::take(&mut toks)));
      } else if t == T_BEGIN!() {
        toks.push(t);
        let mut level = 1;
        for t in tokens_iter.by_ref() {
          match t.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => level -= 1,
            _ => {},
          }
          toks.push(t);
          if level < 1 {
            // done if balanced.
            break;
          }
        }
      } else if t == T_MATH!() {
        toks.push(t);
        for t in tokens_iter.by_ref() {
          let is_math = t.get_catcode() == Catcode::MATH;
          toks.push(t);
          if is_math {
            break;
          }
        }
      } else {
        toks.push(t);
      }
    }
    // last author is in toks, add to items
    items.push(Tokens::new(toks));
  }
  items
}

pub fn and_split(cs: Token, tokens: Tokens) -> Vec<Token> {
  split_tokens(tokens, vec![T_CS!("\\and")])
    .into_iter()
    .flat_map(|t| {
      let mut with_cs = vec![cs, T_BEGIN!()];
      with_cs.extend(t.unlist());
      with_cs.push(T_END!());
      with_cs
    })
    .collect()
}

/// Converts tokens to a string in the fashion of \message and others
///
/// doubles #, converts to string; optionally adds spaces after control sequences
/// in the spirit of the B Book, "show_token_list" routine, in 292.
/// [This could be a $tokens->unpackParameters, but for the curious space treatment]
pub fn writable_tokens(tokens: &Tokens) -> String {
  let mut wv = Vec::new();
  for t in tokens.unlist_ref().iter() {
    match t.code {
      Catcode::CS => {
        wv.push(*t);
        // Perl: add space after CS unless it's a single non-alpha char CS (like \{, \\, \#)
        // i.e. skip space only for "\X" where X is exactly one non-[a-zA-Z] character
        let is_single_nonalpha_cs = arena::with(t.text, |s| {
          s.starts_with('\\') && {
            let rest = &s[1..];
            rest.chars().count() == 1 && !rest.chars().next().unwrap_or(' ').is_ascii_alphabetic()
          }
        });
        if !is_single_nonalpha_cs {
          wv.push(T_SPACE!());
        }
      },
      Catcode::SPACE => {
        wv.push(T_SPACE!());
      },
      Catcode::PARAM => {
        wv.push(*t);
        wv.push(*t);
      },
      Catcode::ARG => {
        // B Book, 294. Reduce to param+integer
        wv.push(T_PARAM!());
        wv.push(t.as_other());
      },
      _ => {
        wv.push(*t);
      },
    }
  }
  Tokens::new(wv).untex()
}

/// Support for Key / Value arguments.
// The very basic form is
//   RequiredKeyVals: $keyset
//   OptionalKeyVals: $keyset
// to parse Key-Value pairs from a given keyset (see the 'keyval' package
// documentation for more information). These types of KeyVal
// parameters will return a LaTeXML::Core::KeyVals object, which can then be
// used to access the values of the individual items.
// The difference between the two forms is that RequiredKeyVals expects a set of
// key-value pairs wrapped in T_BEGIN T_END, where as OptionalKeyVals optionally
// expects a set of KeyValue pairs wrapped in T_OTHER('[') T_OTHER(']')
//
// Several extension of the keyval package exist, the most common one we support
// is the xkeyval package. This introduces further variations on the keyval
// arguments parsing, in particular it allows to read keys from more than one
// keyset at once. These can be specified by giving comma-seperated values in
// the keyset argument. By default, a key will only be set in the **first**
// keyset it occurs in. By using
//   RequiredKeyVals+: $keysets
//   OptionalKeyVals+: $keysets
// the key will be set in all keysets instead.
//
// All keys to be parsed with these arguments should be declared using
// DefKeyVal in LaTeXML::Package. By default, an error is thrown if an unknown
// key is encountered. To surpress this behaviour, and instead store all
// undefined keys, use
//   RequiredKeyVals*: $keysets
//   OptionalKeyVals*: $keysets
// instead. The '*' and '+' modifiers can be combined by using:
//   RequiredKeyVals*+: $keysets
//   OptionalKeyVals*+: $keysets
//
// Furthermore, the xkeyval package supports giving prefixes to keys,
//   RequiredKeyVals[*][+]: $prefix|$keysets
//   OptionalKeyVals[*][+]: $prefix|$keysets
//
// Finally, it is possible to specify specific keys to skip when digesting the
// object. This can be achieved using comma-seperated key values in
//   RequiredKeyVals[*][+]: $prefix|$keysets|$skip
//   OptionalKeyVals[*][+]: $prefix|$keysets|$skip

// function to handle all the
#[derive(Default)]
pub struct KVSpec {
  pub star:    bool,
  pub plus:    bool,
  pub prefix:  Option<String>,
  pub keysets: Vec<String>,
  pub skip:    Vec<String>,
}
pub fn keyvals_aux(until: Option<Token>, spec: KVSpec) -> Result<KeyVals> {
  let KVSpec {
    mut star,
    plus,
    mut prefix,
    mut keysets,
    skip,
  } = spec;
  // support both "keysets" and "prefix|keysets"
  if keysets.is_empty() {
    if let Some(pfx) = prefix.take() {
      keysets = vec![pfx];
    }

    // to emulate old behaviour, throw no errors
    // when we have a single keyset and no prefix (or no keyset at all)
    if keysets.is_empty() {
      star = true;
    }
  }

  // create a new set of Key-Value arguments
  let mut keyvals = KeyVals::new(KeyvalsConfig {
    prefix,
    keysets,
    set_all: plus,
    set_internals: true,
    skip,
    skip_missing: if star {
      keyvals::SkipMissing::All
    } else {
      keyvals::SkipMissing::None
    },
    hook_missing: None,
  });
  // and read it from the gullet
  if let Some(until_token) = until {
    keyvals.read_from(until_token, false)?;
  }
  // we still want to make use of the hash
  Ok(keyvals)
}

pub fn uppercase_token(token: Token) -> Token { either_case_token(token, true) }
pub fn lowercase_token(token: Token) -> Token { either_case_token(token, false) }

fn either_case_token(token: Token, is_upper: bool) -> Token {
  let (chars_count, thischar) = token.with_str(|s| (s.chars().count(), s.chars().next()));
  // DG: new idea, short-circuit if more than 1 char, since our lccode/uccode tables are single
  // char-based (for now?)
  if chars_count != 1 {
    return token;
  }
  let mut result = String::new();
  let cased = if is_upper {
    lookup_uccode(thischar.unwrap())
  } else {
    lookup_lccode(thischar.unwrap())
  };
  if let Some(code) = cased {
    if code != 0 {
      result.push_str(
        &decode_utf16([code])
          .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
          .collect::<String>(),
      )
    } else {
      result.push(thischar.unwrap());
    }
  } else {
    result.push(thischar.unwrap());
  }
  if token.with_str(|initial_str| initial_str != result) {
    Token::new(result, token.get_catcode())
  } else {
    token
  }
}

/// a candidate for use by \hskip, \hspace, etc... ?
pub fn dimension_to_spaces<T: NumericOps>(dimen: T) -> Cow<'static, str> {
  let fs = lookup_font().unwrap().get_size(); // 1 em
  let pt = dimen.pt_value(None);
  let ems = pt / fs.unwrap();
  if ems < 0.01 {
    Cow::Borrowed("")
  } else if ems < 0.17 {
    Cow::Borrowed("\u{2006}")
  }
  // 6em
  else if ems < 0.30 {
    Cow::Borrowed("\u{2005}")
  }
  // 4em
  else if ems < 0.40 {
    Cow::Borrowed("\u{2004}")
  }
  // 3em (same as nbsp?)
  else {
    let n = (ems + 0.3 / 0.333).trunc() as usize; // 10pts per space...?
    Cow::Owned("\u{00A0}".repeat(n))
  }
}

pub fn aligning_environment(
  align: &str,
  class: &str,
  document: &mut Document,
  props: &SymHashMap<Stored>,
) -> Result<()> {
  if let Some(Stored::Digested(body)) = props.get("body") {
    // Add class attribute to new nodes.
    for mut node in insert_block(document, body, HashMap::default())?.into_iter() {
      set_align_or_class(document, &mut node, align, class)?;
    }
  }
  Ok(())
}

pub fn set_align_or_class(
  document: &mut Document,
  node: &mut Node,
  align: &str,
  class: &str,
) -> Result<()> {
  let qname = model::get_node_qname(node);
  if qname == arena::pin_static("ltx:tag") {
  }
  // HACK
  else if !align.is_empty() && model::can_have_attribute(qname, arena::pin_static("align")) {
    node.set_attribute("align", align)?;
  } else if !class.is_empty() && model::can_have_attribute(qname, arena::pin_static("class")) {
    document.add_class(node, class)?;
  }
  Ok(())
}

pub fn make_generic_message(cmd: &str, args: Vec<Tokens>, kind: &str) -> Result<()> {
  bgroup();
  state::let_i(&T_CS!("\\protect"), &T_CS!("\\string"), None);
  state::let_i(
    &T_CS!("\\MessageBreak"),
    &T_CS!("\\ltx@hard@MessageBreak"),
    None,
  ); // tricky, we need Expand() to execute it
  let mut message = String::new();
  for arg in args.into_iter() {
    let mut arg_toks = arg.unlist();
    arg_toks.push(T_CS!("\\MessageBreak"));
    let arg_str = Expand!(arg_toks).to_string();
    message.push_str(&arg_str);
  }

  egroup()?;
  //   return ('latex', $cmd, $stomach, $message);
  match kind {
    "error" => {
      Error!("latex", cmd, message);
    },
    "warn" => {
      Warn!("latex", cmd, message);
    },
    "info" => {
      Info!("latex", cmd, message);
    },
    _other => panic!("Only call make_generic_message with error|warn|info message kinds."),
  };
  Ok(())
}

/// Convert a vertical positioning, optional argument.
///
///  t = "top", b = "bottom"; default is "middle".
/// Note that the default for vattach attribute is "baseline".
/// Utility, not really TeX, but used by LaTeX, AmSTeX.
pub fn translate_attachment<T: ToString>(pos: T) -> &'static str {
  //implementor note:
  //  T: AsRef<str> would be more efficient than allocating a string every time
  //  but we first need `Stored` and `Digested` to be capable of that.
  match pos.to_string().as_str() {
    "t" => "top",
    "b" => "bottom",
    _ => "middle",
  } // undef meaning 'baseline'
}

pub fn in_svg(document: &Document) -> bool {
  if let Some(context) = document.get_element() {
    document::with_node_qname(&context, |qname| qname.starts_with("svg:"))
  } else {
    false
  }
}

pub fn adjust_box_color(tbox: &Digested) -> Result<()> {
  let color_opt = lookup_font().and_then(|f| f.get_color().map(|c| c.clone().into_owned()));
  if let Some(color) = color_opt {
    if color != "black" {
      adjust_box_color_rec(&color, HashMap::default(), tbox);
    }
  }
  Ok(())
}

fn adjust_box_color_rec(_color: &str, _props: HashMap<String, String>, _tbox: &Digested) {
  // Perl: adjustBoxColor recursively propagates color through box tree.
  // Currently a stub — color propagation is not yet critical for test passage.
}

// Hmm... I wonder, should getString itself be dealing with escapechar?
pub fn escapechar() -> String {
  let code: i64 = match state::lookup_register("\\escapechar", Vec::new()).unwrap() {
    Some(RegisterValue::Number(v)) => v.value_of(),
    _ => -1,
  };
  if (0..=255).contains(&code) {
    let char_code = (code as u8) as char;
    char_code.to_string()
  } else {
    String::new()
  }
}
