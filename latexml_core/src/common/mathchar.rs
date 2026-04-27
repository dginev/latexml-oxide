use std::collections::HashMap;
use std::rc::Rc;

use crate::common::arena::{self, SymHashMap};
use crate::common::error::Result;
use crate::common::store::Stored;
use crate::digested::Digested;
use crate::pin;
use crate::state;
use crate::tbox::Tbox;
use crate::token::Token;

const MATH_CLASS_ROLE: [&str; 8] = ["", "BIGOP", "BINOP", "RELOP", "OPEN", "CLOSE", "PUNCT", ""];

/// Properties for a decoded math character, mirroring Perl's decodeMathChar return
#[derive(Debug, Clone, Default)]
pub struct MathCharProps {
  pub role:           Option<String>,
  pub glyph:          Option<char>,
  pub meaning:        Option<String>,
  pub name:           Option<String>,
  pub stretchy:       Option<String>,
  pub need_scriptpos: bool,
  pub need_mathstyle: bool,
  pub scriptpos:      Option<String>,
  pub mathstyle:      Option<String>,
  pub reversion:      Option<crate::tokens::Tokens>,
  pub font:           Option<crate::common::font::Font>,
}

impl MathCharProps {
  /// Convert need_scriptpos/need_mathstyle flags to actual values based on display mode
  pub fn resolve_style_props(&mut self) {
    let in_display = state::lookup_bool("IN_MATH_DISPLAY")
      || state::lookup_font()
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
    '!' => (
      "POSTFIX",
      Some("factorial".into()),
      None,
      None,
      false,
      false,
    ),
    ',' => ("PUNCT", None, None, None, false, false),
    '.' => ("PERIOD", None, None, None, false, false),
    ';' => ("PUNCT", None, None, None, false, false),
    ':' => ("METARELOP", None, Some("colon".into()), None, false, false),
    '|' => ("VERTBAR", None, None, Some("false".into()), false, false),
    '<' => ("RELOP", Some("less-than".into()), None, None, false, false),
    '>' => (
      "RELOP",
      Some("greater-than".into()),
      None,
      None,
      false,
      false,
    ),
    '(' => ("OPEN", None, None, Some("false".into()), false, false),
    ')' => ("CLOSE", None, None, Some("false".into()), false, false),
    '[' => ("OPEN", None, None, Some("false".into()), false, false),
    ']' => ("CLOSE", None, None, Some("false".into()), false, false),
    '{' => ("OPEN", None, None, Some("false".into()), false, false),
    '}' => ("CLOSE", None, None, Some("false".into()), false, false),
    '&' => ("ADDOP", Some("and".into()), None, None, false, false),
    '%' => ("POSTFIX", Some("percent".into()), None, None, false, false),
    '$' => (
      "OPERATOR",
      Some("currency-dollar".into()),
      None,
      None,
      false,
      false,
    ),
    '?' => ("UNKNOWN", None, None, None, false, false),
    // Backslash
    '\\' => ("ADDOP", Some("set-minus".into()), None, None, false, false),
    // Latin-1 supplement
    '\u{00AC}' => ("BIGOP", Some("not".into()), None, None, false, false), // ¬ \neg, \lnot
    '\u{00B1}' => (
      "ADDOP",
      Some("plus-or-minus".into()),
      None,
      None,
      false,
      false,
    ), // ± \pm
    '\u{00D7}' => ("MULOP", Some("times".into()), None, None, false, false), // × \times
    '\u{00F7}' => ("MULOP", Some("divide".into()), None, None, false, false), // ÷ \div
    // General symbols
    '\u{2020}' => ("MULOP", None, None, None, false, false), // † \dagger
    '\u{2021}' => ("MULOP", None, None, None, false, false), // ‡ \ddagger
    '\u{2032}' => ("SUPOP", None, None, None, false, false), // ′ \prime
    '\u{2061}' => ("APPLYOP", None, Some("".into()), None, false, false), // ⁡ function application
    '\u{2062}' => (
      "MULOP",
      Some("times".into()),
      Some("".into()),
      None,
      false,
      false,
    ), // ⁢ invisible times
    '\u{2063}' => ("PUNCT", None, Some("".into()), None, false, false), // ⁣ invisible separator
    '\u{2064}' => (
      "ADDOP",
      Some("plus".into()),
      Some("".into()),
      None,
      false,
      false,
    ), // ⁤ invisible plus
    '\u{210F}' => (
      "ID",
      Some("Planck-constant-over-2-pi".into()),
      None,
      None,
      false,
      false,
    ), // ℏ \hbar
    '\u{2111}' => (
      "OPFUNCTION",
      Some("imaginary-part".into()),
      None,
      None,
      false,
      false,
    ), // ℑ \Im
    '\u{2118}' => (
      "OPFUNCTION",
      Some("Weierstrass-p".into()),
      None,
      None,
      false,
      false,
    ), // ℘ \wp
    '\u{211C}' => (
      "OPFUNCTION",
      Some("real-part".into()),
      None,
      None,
      false,
      false,
    ), // ℜ \Re
    // Arrows
    '\u{2190}' => ("ARROW", None, None, None, false, false), // ← \leftarrow
    '\u{2191}' => ("ARROW", None, Some("uparrow".into()), None, false, false), // ↑ \uparrow
    '\u{2192}' => ("ARROW", None, None, None, false, false), // → \rightarrow
    '\u{2193}' => ("ARROW", None, Some("downarrow".into()), None, false, false), // ↓ \downarrow
    '\u{2194}' => ("METARELOP", None, None, None, false, false), // ↔ \leftrightarrow
    '\u{2195}' => (
      "ARROW",
      None,
      Some("updownarrow".into()),
      None,
      false,
      false,
    ), // ↕ \updownarrow
    '\u{2196}' => ("ARROW", None, None, None, false, false), // ↖ \nwarrow
    '\u{2197}' => ("ARROW", None, None, None, false, false), // ↗ \nearrow
    '\u{2198}' => ("ARROW", None, None, None, false, false), // ↘ \searrow
    '\u{2199}' => ("ARROW", None, None, None, false, false), // ↙ \swarrow
    '\u{219D}' => ("ARROW", Some("leads-to".into()), None, None, false, false), // ⇝ \leadsto
    '\u{21A6}' => ("ARROW", Some("maps-to".into()), None, None, false, false), // ↦ \mapsto
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
    '\u{21D5}' => (
      "ARROW",
      None,
      Some("Updownarrow".into()),
      None,
      false,
      false,
    ), // ⇕ \Updownarrow
    // Quantifiers and set theory
    '\u{2200}' => ("BIGOP", Some("for-all".into()), None, None, false, false), // ∀ \forall
    '\u{2202}' => (
      "DIFFOP",
      Some("partial-differential".into()),
      None,
      None,
      false,
      false,
    ), // ∂ \partial
    '\u{2203}' => ("BIGOP", Some("exists".into()), None, None, false, false),  // ∃ \exists
    '\u{2205}' => ("ID", Some("empty-set".into()), None, None, false, false),  // ∅ \emptyset
    '\u{2207}' => ("OPERATOR", None, None, None, false, false),                // ∇ \nabla
    '\u{2208}' => ("RELOP", Some("element-of".into()), None, None, false, false), // ∈ \in
    '\u{2209}' => (
      "RELOP",
      Some("not-element-of".into()),
      None,
      None,
      false,
      false,
    ), // ∉ \notin
    '\u{220B}' => ("RELOP", Some("contains".into()), None, None, false, false), // ∋ \ni
    // Big operators
    '\u{220F}' => ("SUMOP", Some("product".into()), None, None, true, true), // ∏ \prod
    '\u{2210}' => ("SUMOP", Some("coproduct".into()), None, None, true, true), // ∐ \coprod
    '\u{2211}' => ("SUMOP", Some("sum".into()), None, None, true, true),     // ∑ \sum
    // Arithmetic operators
    '\u{2213}' => (
      "ADDOP",
      Some("minus-or-plus".into()),
      None,
      None,
      false,
      false,
    ), // ∓ \mp
    '\u{2216}' => ("ADDOP", Some("set-minus".into()), None, None, false, false), // ∖ \setminus
    '\u{2217}' => ("MULOP", Some("times".into()), None, None, false, false),     // ∗ \ast
    '\u{2218}' => ("MULOP", Some("compose".into()), None, None, false, false),   // ∘ \circ
    '\u{2219}' => ("MULOP", None, None, None, false, false),                     // ∙ \bullet
    '\u{221A}' => (
      "OPERATOR",
      Some("square-root".into()),
      None,
      None,
      false,
      false,
    ), // √ \surd
    '\u{221D}' => (
      "RELOP",
      Some("proportional-to".into()),
      None,
      None,
      false,
      false,
    ), // ∝ \propto
    '\u{221E}' => ("ID", Some("infinity".into()), None, None, false, false),     // ∞ \infty
    '\u{2223}' => ("VERTBAR", None, None, None, false, false),                   // ∣ \mid
    '\u{2225}' => (
      "VERTBAR",
      Some("parallel-to".into()),
      Some("||".into()),
      None,
      false,
      false,
    ), // ∥ \parallel
    // Logical operators
    '\u{2227}' => ("ADDOP", Some("and".into()), None, None, false, false), // ∧ \land, \wedge
    '\u{2228}' => ("ADDOP", Some("or".into()), None, None, false, false),  // ∨ \lor, \vee
    '\u{2229}' => (
      "ADDOP",
      Some("intersection".into()),
      None,
      None,
      false,
      false,
    ), // ∩ \cap
    '\u{222A}' => ("ADDOP", Some("union".into()), None, None, false, false), // ∪ \cup
    // Integrals
    '\u{222B}' => ("INTOP", Some("integral".into()), None, None, false, true), // ∫ \int
    '\u{222E}' => (
      "INTOP",
      Some("contour-integral".into()),
      None,
      None,
      false,
      true,
    ), // ∮ \oint
    // Relations
    '\u{223C}' => ("RELOP", Some("similar-to".into()), None, None, false, false), // ∼ \sim
    '\u{2240}' => ("MULOP", None, None, None, false, false),                      // ≀ \wr
    '\u{2243}' => (
      "RELOP",
      Some("similar-to-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ≃ \simeq
    '\u{2245}' => (
      "RELOP",
      Some("approximately-equals".into()),
      None,
      None,
      false,
      false,
    ), // ≅ \cong
    '\u{2248}' => (
      "RELOP",
      Some("approximately-equals".into()),
      None,
      None,
      false,
      false,
    ), // ≈ \approx
    '\u{224D}' => (
      "RELOP",
      Some("asymptotically-equals".into()),
      None,
      None,
      false,
      false,
    ), // ≍ \asymp
    '\u{2250}' => (
      "RELOP",
      Some("approaches-limit".into()),
      None,
      None,
      false,
      false,
    ), // ≐ \doteq
    '\u{2260}' => ("RELOP", Some("not-equals".into()), None, None, false, false), // ≠ \neq
    '\u{2261}' => (
      "RELOP",
      Some("equivalent-to".into()),
      None,
      None,
      false,
      false,
    ), // ≡ \equiv
    '\u{2264}' => (
      "RELOP",
      Some("less-than-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ≤ \leq
    '\u{2265}' => (
      "RELOP",
      Some("greater-than-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ≥ \geq
    '\u{226A}' => (
      "RELOP",
      Some("much-less-than".into()),
      None,
      None,
      false,
      false,
    ), // ≪ \ll
    '\u{226B}' => (
      "RELOP",
      Some("much-greater-than".into()),
      None,
      None,
      false,
      false,
    ), // ≫ \gg
    '\u{227A}' => ("RELOP", Some("precedes".into()), None, None, false, false),   // ≺ \prec
    '\u{227B}' => ("RELOP", Some("succeeds".into()), None, None, false, false),   // ≻ \succ
    // Subset/superset
    '\u{2282}' => ("RELOP", Some("subset-of".into()), None, None, false, false), // ⊂ \subset
    '\u{2283}' => (
      "RELOP",
      Some("superset-of".into()),
      None,
      None,
      false,
      false,
    ), // ⊃ \supset
    '\u{2286}' => (
      "RELOP",
      Some("subset-of-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⊆ \subseteq
    '\u{2287}' => (
      "RELOP",
      Some("superset-of-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⊇ \supseteq
    '\u{228E}' => ("ADDOP", None, None, None, false, false),                     // ⊎ \uplus
    '\u{228F}' => (
      "RELOP",
      Some("square-image-of".into()),
      None,
      None,
      false,
      false,
    ), // ⊏ \sqsubset
    '\u{2290}' => (
      "RELOP",
      Some("square-original-of".into()),
      None,
      None,
      false,
      false,
    ), // ⊐ \sqsupset
    '\u{2291}' => (
      "RELOP",
      Some("square-image-of-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⊑ \sqsubseteq
    '\u{2292}' => (
      "RELOP",
      Some("square-original-of-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⊒ \sqsupseteq
    '\u{2293}' => (
      "ADDOP",
      Some("square-intersection".into()),
      None,
      None,
      false,
      false,
    ), // ⊓ \sqcap
    '\u{2294}' => (
      "ADDOP",
      Some("square-union".into()),
      None,
      None,
      false,
      false,
    ), // ⊔ \sqcup
    // Circled operators
    '\u{2295}' => ("ADDOP", Some("direct-sum".into()), None, None, false, false), // ⊕ \oplus
    '\u{2296}' => (
      "ADDOP",
      Some("symmetric-difference".into()),
      None,
      None,
      false,
      false,
    ), // ⊖ \ominus
    '\u{2297}' => (
      "MULOP",
      Some("tensor-product".into()),
      None,
      None,
      false,
      false,
    ), // ⊗ \otimes
    '\u{2298}' => ("MULOP", None, None, None, false, false),                      // ⊘ \oslash
    '\u{2299}' => (
      "MULOP",
      Some("direct-product".into()),
      None,
      None,
      false,
      false,
    ), // ⊙ \odot
    // Turnstiles
    '\u{22A2}' => ("METARELOP", Some("proves".into()), None, None, false, false), // ⊢ \vdash
    '\u{22A3}' => (
      "METARELOP",
      Some("does-not-prove".into()),
      None,
      None,
      false,
      false,
    ), // ⊣ \dashv
    '\u{22A4}' => ("ADDOP", Some("top".into()), None, None, false, false),        // ⊤ \top
    '\u{22A5}' => ("ADDOP", Some("bottom".into()), None, None, false, false),     // ⊥ \bot
    '\u{22A7}' => ("RELOP", Some("models".into()), None, None, false, false),     // ⊧ \models
    '\u{22B2}' => (
      "ADDOP",
      Some("subgroup-of".into()),
      None,
      None,
      false,
      false,
    ), // ⊲ \lhd
    '\u{22B3}' => (
      "ADDOP",
      Some("contains-as-subgroup".into()),
      None,
      None,
      false,
      false,
    ), // ⊳ \rhd
    '\u{22B4}' => (
      "ADDOP",
      Some("subgroup-of-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⊴ \unlhd
    '\u{22B5}' => (
      "ADDOP",
      Some("contains-as-subgroup-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⊵ \unrhd
    // Big operators (N-ary)
    '\u{22C0}' => ("SUMOP", Some("and".into()), None, None, true, true), // ⋀ \bigwedge
    '\u{22C1}' => ("SUMOP", Some("or".into()), None, None, true, true),  // ⋁ \bigvee
    '\u{22C2}' => ("SUMOP", Some("intersection".into()), None, None, true, true), // ⋂ \bigcap
    '\u{22C3}' => ("SUMOP", Some("union".into()), None, None, true, true), // ⋃ \bigcup
    '\u{22C4}' => ("ADDOP", None, None, None, false, false),             // ⋄ \diamond
    '\u{22C5}' => ("MULOP", None, None, None, false, false),             // ⋅ \cdot
    '\u{22C6}' => ("MULOP", None, None, None, false, false),             // ⋆ \star
    '\u{22C8}' => ("RELOP", None, None, None, false, false),             // ⋈ \bowtie
    '\u{22EF}' => ("ID", None, None, None, false, false),                // ⋯ \cdots
    '\u{22F1}' => ("ID", None, None, None, false, false),                // ⋱ \ddots
    // Delimiters
    '\u{2308}' => (
      "OPEN",
      None,
      Some("lceil".into()),
      Some("false".into()),
      false,
      false,
    ), // ⌈ \lceil
    '\u{2309}' => (
      "CLOSE",
      None,
      Some("rceil".into()),
      Some("false".into()),
      false,
      false,
    ), // ⌉ \rceil
    '\u{230A}' => (
      "OPEN",
      None,
      Some("lfloor".into()),
      Some("false".into()),
      false,
      false,
    ), // ⌊ \lfloor
    '\u{230B}' => (
      "CLOSE",
      None,
      Some("rfloor".into()),
      Some("false".into()),
      false,
      false,
    ), // ⌋ \rfloor
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
    '\u{27C2}' => (
      "RELOP",
      Some("perpendicular-to".into()),
      None,
      None,
      false,
      false,
    ), // ⟂ \perp
    // Angle brackets
    '\u{27E8}' => (
      "OPEN",
      None,
      Some("langle".into()),
      Some("false".into()),
      false,
      false,
    ), // ⟨ \langle
    '\u{27E9}' => (
      "CLOSE",
      None,
      Some("rangle".into()),
      Some("false".into()),
      false,
      false,
    ), // ⟩ \rangle
    '\u{27EE}' => (
      "OPEN",
      None,
      Some("lgroup".into()),
      Some("false".into()),
      false,
      false,
    ), // ⟮ \lgroup
    '\u{27EF}' => (
      "CLOSE",
      None,
      Some("rgroup".into()),
      Some("false".into()),
      false,
      false,
    ), // ⟯ \rgroup
    // Long arrows
    '\u{27F5}' => ("ARROW", None, None, None, false, false), // ⟵ \longleftarrow
    '\u{27F6}' => ("ARROW", None, None, None, false, false), // ⟶ \longrightarrow
    '\u{27F7}' => ("METARELOP", None, None, None, false, false), // ⟷ \longleftrightarrow
    '\u{27F8}' => ("ARROW", None, None, None, false, false), // ⟸ \Longleftarrow
    '\u{27F9}' => ("ARROW", None, None, None, false, false), // ⟹ \Longrightarrow
    '\u{27FA}' => ("METARELOP", None, None, None, false, false), // ⟺ \Longleftrightarrow
    '\u{27FC}' => ("ARROW", None, None, None, false, false), // ⟼ \longmapsto
    // N-ary circled operators
    '\u{2A00}' => ("SUMOP", None, None, None, true, true), // ⨀ \bigodot
    '\u{2A01}' => ("SUMOP", Some("direct-sum".into()), None, None, true, true), // ⨁ \bigoplus
    '\u{2A02}' => (
      "SUMOP",
      Some("tensor-product".into()),
      None,
      None,
      true,
      true,
    ), // ⨂ \bigotimes
    '\u{2A04}' => (
      "SUMOP",
      Some("symmetric-difference".into()),
      None,
      None,
      true,
      true,
    ), // ⨄ \biguplus
    '\u{2A06}' => ("SUMOP", Some("square-union".into()), None, None, true, true), // ⨆ \bigsqcup
    '\u{2A1D}' => ("RELOP", Some("join".into()), None, None, false, false), // ⨝ \Join
    '\u{2AAF}' => (
      "RELOP",
      Some("precedes-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⪯ \preceq
    '\u{2AB0}' => (
      "RELOP",
      Some("succeeds-or-equals".into()),
      None,
      None,
      false,
      false,
    ), // ⪰ \succeq
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
    reversion: None,
    font: None,
  })
}

// Is this "fontinfo" stuff sufficient to maintain a math font "family" ??
// What we're really after is a connection to a font encoding mapping.
pub fn decode_math_char(
  mut n: u16,
  reversion: Option<crate::tokens::Tokens>,
) -> Result<MathCharProps> {
  let class: u16 = n / (16 * 256);
  n %= 16 * 256;
  let mut fam: u16 = n / 256;

  let curfam_val: i32 = match state::lookup_register("\\fam", Vec::new()) {
    Ok(Some(crate::definition::register::RegisterValue::Number(curfam))) => curfam.0 as i32,
    _ => -1,
  };

  if class == 7 && (0..=15).contains(&curfam_val) {
    fam = curfam_val as u16;
  }
  n %= 256;

  let curfont = state::lookup_font().unwrap();
  // `with_value` borrows the Stored — for `Stored::Font(f)` we need
  // the `Rc<Font>` out, so clone the Rc (cheap refcount bump) rather
  // than the entire Stored enum.
  let initfont = state::with_value("initial_math_font", |v| match v {
    Some(Stored::Font(f)) => Rc::clone(f),
    _ => Rc::clone(&curfont),
  });

  let mut use_current_font = false;
  let mut maybe_rev = curfam_val >= 0 && fam != 1;
  let mut fontdef_tok: Option<Token> = None;

  if class == 7 && curfam_val < 0 && curfont != initfont {
    use_current_font = true;
    maybe_rev = true;
    fontdef_tok = Some(crate::T_CS!("\\font"));
  }

  let mut downsize = 0;
  if fontdef_tok.is_none() {
    // Token is Copy (SymStr + Catcode = 8 bytes), so the closure extracts
    // the Token by-value without cloning the enclosing Stored.
    let extract_token = |key: &str| -> Option<Token> {
      state::with_value(key, |v| match v {
        Some(Stored::Token(t)) => Some(*t),
        _ => None,
      })
    };
    let style = curfont
      .get_mathstyle()
      .map(|s| s.to_string())
      .unwrap_or_default();
    let style_str = if style == "script" || style == "scriptscript" || style == "text" {
      style.as_str()
    } else {
      "text"
    };
    if style_str == "text" {
      fontdef_tok = extract_token(&crate::s!("textfont_{fam}"));
    } else if style_str == "script" {
      fontdef_tok = extract_token(&crate::s!("scriptfont_{fam}"));
      if fontdef_tok.is_none() {
        fontdef_tok = extract_token(&crate::s!("textfont_{fam}"));
        if fontdef_tok.is_some() {
          downsize = 1;
        }
      }
    } else if style_str == "scriptscript" {
      fontdef_tok = extract_token(&crate::s!("scriptscriptfont_{fam}"));
      if fontdef_tok.is_none() {
        fontdef_tok = extract_token(&crate::s!("scriptfont_{fam}"));
        if fontdef_tok.is_some() {
          downsize = 1;
        } else {
          fontdef_tok = extract_token(&crate::s!("textfont_{fam}"));
          if fontdef_tok.is_some() {
            downsize = 2;
          }
        }
      }
    }
  }

  let c = n as u8 as char;
  // Guard against invalid class values from corrupted mathchar codes
  // (e.g., during expl3 loading when \__int_eval_end: errors corrupt state)
  if (class as usize) >= MATH_CLASS_ROLE.len() {
    return Ok(MathCharProps::default());
  }
  let class_role = MATH_CLASS_ROLE[class as usize];

  let mut f = (*curfont).clone();
  if let Some(ftok) = &fontdef_tok {
    if use_current_font {
      // f is already curfont
    } else {
      // Merge textfont info (family, encoding, etc.) but preserve the current
      // font's size. The textfont defines design-time properties, while the
      // current size comes from context (e.g. \big's font => { size => 12 }).
      let preserved_size = f.size;
      state::with_font_info(ftok, |fontinfo| {
        if let Some(Stored::Font(ref info)) = fontinfo.unwrap_or(None) {
          f = f.merge_ref(info);
        } else {
          // Perl: fallback to \lx@default@font if not found
          let d_tok_opt = state::with_value("\\lx@default@font", |v| match v {
            Some(Stored::Token(t)) => Some(*t),
            _ => None,
          });
          if let Some(d_tok) = d_tok_opt {
            state::with_font_info(&d_tok, |d_info| {
              if let Some(Stored::Font(ref d_f)) = d_info.unwrap_or(None) {
                f = f.merge_ref(d_f);
              }
            });
          }
        }
      });
      f.size = preserved_size;
    }
  }

  if downsize > 0 {
    f.scripted = Some(true);
  }
  if downsize > 1 {
    f.scripted = Some(true);
  }

  let d = f.relative_to(&curfont);

  let glyph = if use_current_font {
    if let Some(ref data) = curfont.encoding {
      crate::common::font::decode(n as u8, Some(data.to_string()), false)
    } else {
      Some(c)
    }
  } else if let Some(ftok) = &fontdef_tok {
    // Extract the encoding BEFORE calling font::decode. font::decode may
    // call preload_font_map which mutates state, and with_font_info holds
    // a State borrow while its closure runs — the reentrant mutation
    // panics with "RefCell already borrowed" (sandbox paper 0711.4787).
    let mut encoding_opt: Option<String> = state::with_font_info(ftok, |fontinfo| {
      if let Some(Stored::Font(ref info)) = fontinfo? {
        Ok::<Option<String>, crate::common::error::Error>(
          info.encoding.as_ref().map(|s| s.to_string()),
        )
      } else {
        Ok(None)
      }
    })?;
    // Fallback: when `fontinfo_<token>` is missing (typical post-dump-load:
    // `Stored::Font` isn't currently Font-serialized in the dump_writer,
    // so the rich props don't round-trip — only the `font_shared_key_<cs>`
    // pointer survives), derive the encoding from the font NAME via
    // `font::decode_fontname`. This recovers `cmmi10 → OML`, so plain.tex's
    // `.` mathcode 0x013A (class 0, fam 1, char 0x3A) decodes to glyph
    // `.` instead of the raw ASCII `:` that the no-encoding path emits.
    // Without this, `12345.67890` math input split as
    // <NUMBER>12345</NUMBER><METARELOP>:</METARELOP><NUMBER>67890</NUMBER>
    // — see `00_tokenize::ligatures_test` / `mathtokens_test` 2026-04-27.
    if encoding_opt.is_none() {
      let shared_key = state::with_value(
        &crate::s!("font_shared_key_{}", ftok.with_str(ToString::to_string)),
        |v| match v {
          Some(Stored::String(s)) => crate::common::arena::with(*s, |str| Some(str.to_string())),
          _ => None,
        },
      );
      if let Some(sk) = shared_key {
        // shared_key is "fontinfo_<name>"; strip the "fontinfo_" prefix to get the font name
        if let Some(name) = sk.strip_prefix("fontinfo_") {
          if let Some(props) = crate::common::font::decode_fontname(name, None, None) {
            encoding_opt = props.encoding.as_ref().map(|s| s.to_string());
          }
        }
      }
    }
    if let Some(data) = encoding_opt {
      crate::common::font::decode(n as u8, Some(data), false)
    } else {
      Some(c)
    }
  } else {
    Some(c)
  };

  let glyph_char = glyph.unwrap_or(c);
  let charinfo = unicode_math_properties(glyph_char);
  let mut props = charinfo.clone().unwrap_or_default();
  props.glyph = glyph;

  let mut role = charinfo.as_ref().and_then(|info| info.role.clone());
  if role.is_none() && !class_role.is_empty() {
    role = Some(class_role.to_string());
  }
  if role.is_some() && props.role.is_none() {
    props.role = role;
  }

  props.resolve_style_props();

  let mut final_reversion = reversion;
  if let Some(rev) = final_reversion.clone() {
    let mut wrap = maybe_rev && !d.is_empty();
    if state::with_value("LaTeX.pool_loaded", |v| v.is_some()) {
      wrap = false;
    }
    if wrap {
      if let Some(ftok) = fontdef_tok {
        let mut new_rev = vec![crate::T_BEGIN!(), ftok];
        new_rev.extend(rev.unlist());
        new_rev.push(crate::T_END!());
        final_reversion = Some(crate::tokens::Tokens::new(new_rev));
      }
    }
    props.reversion = final_reversion;
  }

  props.font = Some(f);

  Ok(props)
}

/// Stomach-level hook for decoding math characters.
/// Called from stomach::invoke_token_simple when IN_MATH and mathcode is set.
/// Perl: decodeMathChar($mathcode, $meaning) in Stomach::invokeToken_simple
pub fn decode_math_char_for_stomach(mathcode: u16, meaning: Token) -> Result<Option<Digested>> {
  let props = decode_math_char(mathcode, Some(crate::Tokens!(meaning)))?;

  let glyph = match props.glyph {
    Some(g) => g,
    None => return Ok(None),
  };

  let mut properties = SymHashMap::default();
  properties.insert("mode", Stored::String(pin!("math")));
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
  if let Some(ref scriptpos) = props.scriptpos {
    properties.insert("scriptpos", Stored::String(arena::pin(scriptpos)));
  }
  if let Some(ref mathstyle) = props.mathstyle {
    properties.insert("mathstyle", Stored::String(arena::pin(mathstyle)));
  }

  let glyph_sym = arena::pin_char(glyph);

  let font = props
    .font
    .map(|f| Rc::new(arena::with(glyph_sym, |s| f.specialize(s))));
  Ok(Some(Digested::from(Tbox::new(
    glyph_sym,
    font,
    None,
    props.reversion.unwrap_or(crate::Tokens!(meaning)),
    properties,
  ))))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn math_class_role_table_has_expected_values() {
    // The Perl %mathclass mapping: 0,7 (ord, variable — no role);
    // 1=BIGOP, 2=BINOP, 3=RELOP, 4=OPEN, 5=CLOSE, 6=PUNCT.
    assert_eq!(MATH_CLASS_ROLE[0], "");
    assert_eq!(MATH_CLASS_ROLE[1], "BIGOP");
    assert_eq!(MATH_CLASS_ROLE[2], "BINOP");
    assert_eq!(MATH_CLASS_ROLE[3], "RELOP");
    assert_eq!(MATH_CLASS_ROLE[4], "OPEN");
    assert_eq!(MATH_CLASS_ROLE[5], "CLOSE");
    assert_eq!(MATH_CLASS_ROLE[6], "PUNCT");
    assert_eq!(MATH_CLASS_ROLE[7], "");
  }

  #[test]
  fn unicode_math_properties_digits() {
    let p = unicode_math_properties('5').expect("digits should resolve");
    assert_eq!(p.role.as_deref(), Some("NUMBER"));
    assert_eq!(p.meaning.as_deref(), Some("5"));
  }

  #[test]
  fn unicode_math_properties_basic_relops() {
    let eq = unicode_math_properties('=').unwrap();
    assert_eq!(eq.role.as_deref(), Some("RELOP"));
    assert_eq!(eq.meaning.as_deref(), Some("equals"));
    let lt = unicode_math_properties('<').unwrap();
    assert_eq!(lt.role.as_deref(), Some("RELOP"));
    assert_eq!(lt.meaning.as_deref(), Some("less-than"));
  }

  #[test]
  fn unicode_math_properties_basic_addops() {
    let plus = unicode_math_properties('+').unwrap();
    assert_eq!(plus.role.as_deref(), Some("ADDOP"));
    assert_eq!(plus.meaning.as_deref(), Some("plus"));
    let minus = unicode_math_properties('-').unwrap();
    assert_eq!(minus.role.as_deref(), Some("ADDOP"));
    assert_eq!(minus.meaning.as_deref(), Some("minus"));
  }

  #[test]
  fn unicode_math_properties_openclose() {
    // Paired delimiters get OPEN/CLOSE with stretchy="false".
    for (c, role) in [
      ('(', "OPEN"),
      (')', "CLOSE"),
      ('[', "OPEN"),
      (']', "CLOSE"),
      ('{', "OPEN"),
      ('}', "CLOSE"),
    ] {
      let p = unicode_math_properties(c).unwrap();
      assert_eq!(p.role.as_deref(), Some(role), "{c}");
      assert_eq!(p.stretchy.as_deref(), Some("false"), "{c} stretchy");
    }
  }

  #[test]
  fn unicode_math_properties_punct() {
    let comma = unicode_math_properties(',').unwrap();
    assert_eq!(comma.role.as_deref(), Some("PUNCT"));
    let semi = unicode_math_properties(';').unwrap();
    assert_eq!(semi.role.as_deref(), Some("PUNCT"));
  }

  #[test]
  fn into_props_map_only_includes_set_fields() {
    // A mostly-empty MathCharProps should produce an empty map —
    // into_props_map skips None fields.
    let p = MathCharProps {
      role:           Some("RELOP".into()),
      meaning:        Some("equals".into()),
      name:           None,
      stretchy:       None,
      need_scriptpos: false,
      need_mathstyle: false,
      scriptpos:      None,
      mathstyle:      None,
      reversion:      None,
      font:           None,
      glyph:          None,
    };
    let m = p.into_props_map();
    assert_eq!(m.len(), 2, "only role and meaning should populate");
    assert!(m.contains_key("role"));
    assert!(m.contains_key("meaning"));
    assert!(!m.contains_key("name"));
    assert!(!m.contains_key("stretchy"));
  }

  #[test]
  fn into_props_map_empty_when_all_none() {
    let p = MathCharProps::default();
    let m = p.into_props_map();
    assert_eq!(m.len(), 0);
  }
}
