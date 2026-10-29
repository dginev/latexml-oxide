//! The one reading of a formula's mark-ness from its digested XMath (frontmatter refactor stage 1,
//! `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`; OXIDIZED_DESIGN_DIVERGENCES #837, #911): whether a formula in a name
//! or an affiliation is a mark, what its scripts print, and the base a script sits on. It reads structure and printed
//! text, never TeX spelling: a font switch is a token's font, spacing an `XMHint`, `\{` and `\lbrace` one "{".
//!
//! Its readers see two vocabularies, both taken:
//! - **as digested** (the observer writer's view of today's frontmatter, `relocate::shown_marks`,
//!   `author_block::name_text` — all before `finish_document`'s rewrite and math parse): `XMApp[@role]` of
//!   `FLOATSUPERSCRIPT`/`FLOATSUBSCRIPT` (a script with nothing before it) or `POSTSUPERSCRIPT`/`POSTSUBSCRIPT` (a script
//!   after a base), its `XMArg[@rule]` the script, beside flat `XMTok`/`XMText`/`XMHint` siblings (the constructors of
//!   `tex_math.rs` `\lx@floating@superscript`, `\lx@post@superscript`, chosen at digestion by `script_handler`);
//! - **as parsed** (`ab_audit fmshadow`, the final XML): a lone floating script is `ltx:sup` (no formula: the rewrite
//!   of `tex_math.rs`'s floating-script rule; Perl `TeX_Math.pool.ltxml:530`); a script the parser attached is
//!   `XMApp(XMTok[@role='SUPERSCRIPTOP'][@scriptpos], base, script)`, `scriptpos` `preN` for a script before its base
//!   (³He, `^{a})`) and `postN` after it; a surviving `FLOATSUPERSCRIPT` holds its script unwrapped; `XMDual` (its
//!   presentation, the last child) and `XMWrap` around lists; any other application LaTeXML lays out as a row read in
//!   print order ([`printed_children`]), so a script the parser nests under a word's invisible times (`$Ann^{1}$` as
//!   `⁢(A, n, n¹)`, `$\mathrm{Ann~Able}^{1}$`) or a hyphenated word's minus reads as the digested shape's flat items.
//!
//! In both vocabularies a two-dimensional layout is one printed item (`row_layout`, by the operator's role and
//! meaning as Perl `Post/MathML.pm` picks its presenter): a fraction, a root, an accent, an enclosure, a big operator,
//! limits. A script under a fraction's bar or a root's sign is not a script the formula ends on (`\frac{a}{b^{1}}`).
//!
//! A formula **is a mark** when it opens with scripts nothing visible precedes (floating, or after a box that prints
//! nothing, `$\mbox{ }^1$`: cond-mat0206265), at least one a superscript, and the first item printed after them is no
//! letter or digit. That criterion is ours, not the parser's: the parser attaches a prescript (`scriptpos="pre"`) to
//! whatever follows, a `)` as readily as a word. Spacing is no ink,
//! punctuation and symbols are no base (`^{a})`, cond-mat9808038; `^{1}{}^{,}`; `^{1,2}\dagger`, 2501.04244; `^1,2`,
//! 1912.02075's `A.P.~Orlov$^1,2$`). Its marks are its superscripts before the first letter or digit. Scripts after
//! printed ink are on a **base** (`97403^{1}`, `\text{Leon Bello}^{1,3}`): the base, with its subscripts, is the item's
//! text, the superscripts no mark. Scripts right before a letter or digit (an isotope, `{}^{3}He`, `{\rm ^{3}He}`) are
//! a prescript on a printed word, no mark: the fail-safe, the formula's text kept as printed. What a reader has only as
//! tokens (the label keys of `\lx@frontmatter@sup`, revtex's group rule, `notes::leads_with_mark`) is stages 3–5's.

use super::*;

/// A node of a formula as [`read_formula`] walks it: the engine's libxml node, or `ab_audit fmshadow`'s roxmltree one
/// (a newtype there).
pub trait FmMathNode: Sized + Clone {
  /// The element's local name when it is a LaTeXML element (`XMApp`, `XMTok`, …); `None` for any other node.
  fn ltx_name(&self) -> Option<String>;
  /// The element's attribute `name`.
  fn attribute(&self, name: &str) -> Option<String>;
  /// The node's children, elements and text, in document order.
  fn children(&self) -> Vec<Self>;
  /// A text node's text; `None` for an element.
  fn text(&self) -> Option<String>;
}

/// The engine's document node (the writer's view, `shown_marks`, `is_name_mark`).
impl FmMathNode for Node {
  fn ltx_name(&self) -> Option<String> {
    (self.get_type() == Some(NodeType::ElementNode))
      .then(|| document::with_node_qname(self, |q| q.strip_prefix("ltx:").map(str::to_string)))
      .flatten()
  }

  fn attribute(&self, name: &str) -> Option<String> { self.get_attribute(name) }

  fn children(&self) -> Vec<Self> { self.get_child_nodes() }

  fn text(&self) -> Option<String> {
    (self.get_type() == Some(NodeType::TextNode)).then(|| self.get_content())
  }
}

/// What a formula is to the frontmatter ([`read_formula`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaMarks {
  /// A mark: the printed text of each of its superscripts, in order (`$^{1,2}$` "1,2"; `${}^{a,\;b}$` "a,b").
  Marks(Vec<String>),
  /// Scripts on a printed base, nothing printed after them: the base's printed text with its subscripts (`$97403^{1}$`
  /// "97403", `$\text{Leon Bello}^{1,3}$` "Leon Bello", `$G_2'$` "G2") and the superscripts' texts.
  OnBase {
    base:    String,
    scripts: Vec<String>,
  },
  /// Any other formula: its text is its own.
  Other,
}

/// One printed item of a formula, in print order.
enum Item {
  /// ink: its printed text, and whether it is a text box (`XMText`, `\text{…}`)
  Ink { text: String, text_box: bool },
  /// a script, in print order (a parsed `preN` before its base): a superscript or not, its printed text
  Script {
    superscript: bool,
    text:        String,
  },
}

/// The element children of `node`.
fn elements<N: FmMathNode>(node: &N) -> Vec<N> {
  node
    .children()
    .into_iter()
    .filter(|n| n.ltx_name().is_some())
    .collect()
}

/// What `node` prints: its text, without spacing hints, an `XMDual` by its presentation (its last child), an `XMRef`
/// nothing (the presentation holds what it refers to), an `XMApp` in print order ([`printed_children`]).
fn printed<N: FmMathNode>(node: &N) -> String {
  if let Some(text) = node.text() {
    return text;
  }
  match node.ltx_name().as_deref() {
    None | Some("XMHint" | "XMRef") => String::new(),
    Some("XMDual") => elements(node).last().map(printed).unwrap_or_default(),
    // (an application in print order, its silent operator none: `⁢(S, m, i, t, h)` "Smith")
    Some("XMApp") => printed_children(node).iter().map(printed).collect(),
    Some(_) => node.children().iter().map(printed).collect(),
  }
}

/// Whether an operator prints nothing: no text, or only the parser's invisible operators (U+2061 function
/// application, U+2062 times, U+2063 separator, U+2064 plus).
fn prints_nothing<N: FmMathNode>(op: &N) -> bool {
  printed(op)
    .chars()
    .all(|c| c.is_whitespace() || ('\u{2061}'..='\u{2064}').contains(&c))
}

/// The children of `node` in the order they print. An `XMApp`'s operator and arguments are set as LaTeXML's own
/// presentation sets them (Perl `Post/MathML.pm`): a script before or after its base by its `scriptpos`
/// (`pmml_script`, L876), the operator of an infix role between its arguments, or before a lone one (`pmml_infix`,
/// L626; the roles of `Apply:ADDOP:?` and its siblings, L1540, L1600-1622, L1699-1700, L1739), a `POSTFIX` operator
/// after its argument (L1632), any other before its arguments (`Apply:?:?`, L1523). That is the print order of a row
/// layout; a two-dimensional one's parts (a fraction's, a root's, an accent's, a limit's: `row_layout`) stay in
/// document order, the order the formula's text has always been read in (`\sqrt[3]{x}` "3x"). An operator that prints nothing (no text, or only the
/// invisible U+2061–U+2064) is left out, and so is the text between an application's children (the XML's
/// indentation). Any other node's children are as they are, its text nodes too.
pub fn printed_children<N: FmMathNode>(node: &N) -> Vec<N> {
  // (as digested, and a floating script the parser left, an application's role names its script, no operator)
  if node.ltx_name().as_deref() != Some("XMApp")
    || node
      .attribute("role")
      .as_deref()
      .and_then(digested_script)
      .is_some()
  {
    return node.children();
  }
  let mut parts = elements(node).into_iter();
  let Some(op) = parts.next() else {
    return Vec::new();
  };
  let args: Vec<N> = parts.collect();
  let role = op.attribute("role").unwrap_or_default();
  let silent = prints_nothing(&op);
  let mut out = Vec::new();
  match role.as_str() {
    "SUPERSCRIPTOP" | "SUBSCRIPTOP" => {
      let pre = op
        .attribute("scriptpos")
        .is_some_and(|p| p.starts_with("pre"));
      if !silent {
        out.push(op);
      }
      out.extend(if pre {
        args.into_iter().rev().collect()
      } else {
        args
      });
    },
    "ADDOP" | "MULOP" | "BINOP" | "COMPOSEOP" | "MODIFIEROP" | "MIDDLE" | "RELOP" | "METARELOP"
    | "ARROW"
      if args.len() > 1 =>
    {
      for (i, arg) in args.into_iter().enumerate() {
        if i > 0 && !silent {
          out.push(op.clone());
        }
        out.push(arg);
      }
    },
    "POSTFIX" => {
      out.extend(args);
      if !silent {
        out.push(op);
      }
    },
    _ => {
      if !silent {
        out.push(op);
      }
      out.extend(args);
    },
  }
  out
}

/// The script role of an `XMApp` as digested (floating, `FLOAT*`, or after a base, `POST*`): whether a superscript.
fn digested_script(role: &str) -> Option<bool> {
  match role {
    "FLOATSUPERSCRIPT" | "POSTSUPERSCRIPT" => Some(true),
    "FLOATSUBSCRIPT" | "POSTSUBSCRIPT" => Some(false),
    _ => None,
  }
}

/// Whether `node` holds a script: a digested script application, or a parsed script operator.
fn holds_a_script<N: FmMathNode>(node: &N) -> bool {
  elements(node).iter().any(|c| {
    let role = c.attribute("role");
    match c.ltx_name().as_deref() {
      Some("XMTok") => matches!(role.as_deref(), Some("SUPERSCRIPTOP" | "SUBSCRIPTOP")),
      Some("XMApp") if role.as_deref().and_then(digested_script).is_some() => true,
      _ => holds_a_script(c),
    }
  })
}

/// The role an operator presents by (Perl `Post/MathML.pm` `getOperatorRole`, L173): its own `role`, or, for an
/// operator a script or an accent embellishes (`\sum_{i}` as `XMApp(SUBSCRIPTOP, ∑, i)`), the role of what it
/// embellishes.
fn operator_role<N: FmMathNode>(op: &N) -> Option<String> {
  if let Some(role) = op.attribute("role") {
    return Some(role);
  }
  if op.ltx_name().as_deref() != Some("XMApp") {
    return None;
  }
  let parts = elements(op);
  let (Some(inner), Some(base)) = (parts.first(), parts.get(1)) else {
    return None;
  };
  matches!(
    inner.attribute("role").as_deref(),
    Some(
      "SUPERSCRIPTOP" | "SUBSCRIPTOP" | "OVERACCENT" | "UNDERACCENT" | "MODIFIER" | "MODIFIEROP"
    )
  )
  .then(|| operator_role(base))
  .flatten()
}

/// Whether LaTeXML lays the application `node` out as a row (Perl `Post/MathML.pm`, which picks a presenter by the
/// operator's role and meaning, `lookupPresenter` L205): the infix roles (`pmml_infix`, L1540, L1600-1622, L1699-1700,
/// L1739), `POSTFIX` (L1632), any other operator (`Apply:?:?`, L1523), and a script after or
/// before its base (`pmml_script`'s `msup`/`mmultiscripts`, L876). Not a row, and so one printed item ([`flatten`]):
/// a fraction (`Apply:FRACOP:?` → `mfrac`, L1609; `\frac`, `\tfrac`, `\binom`'s), a root by its meaning
/// (`square-root`/`nth-root` → `msqrt`/`mroot`, L1636-1645), a continued fraction (L1954's display `mfrac`), an accent
/// over or under
/// its base (`Apply:OVERACCENT:?`/`UNDERACCENT:?` → `mover`/`munder`, L1489-1505; `\hat`, `\overline`, `\overset`),
/// an enclosure (`Apply:ENCLOSE:?` → `menclose`, L1507), a big operator with its body, a
/// script set over or under its base (`scriptpos` `mid`, `pmml_script_mid_layout`'s `munder`/`mover`: limits,
/// `\stackrel`), and anything carrying `enclose` (any node, MathML.pm L338-342: `\boxed`, amsmath.sty.ltxml:893-895).
/// Two of these Perl lays out as a row, and are read as one item here on the safe side, with no output change on any
/// probe (review of 90a r4): a big operator (`pmml_summation`, L1796-1798, an `mrow` of the operator and its body) and
/// an inline continued fraction (`cfrac-inline`, `pmml_row`, L1954). The operator is read as written, not through
/// `realize` as Perl reads it (L419): an `XMRef` operator would count as a row (none in the probes or the census).
fn row_layout<N: FmMathNode>(node: &N) -> bool {
  if node.attribute("enclose").is_some() {
    return false;
  }
  let Some(op) = elements(node).into_iter().next() else {
    return true;
  };
  // (a root's presenter is keyed by its meaning, which Perl looks up before the role's: `Apply:?:square-root`)
  if matches!(
    op.attribute("meaning").as_deref(),
    Some("square-root" | "nth-root" | "continued-fraction")
  ) {
    return false;
  }
  match operator_role(&op).as_deref() {
    Some("FRACOP" | "OVERACCENT" | "UNDERACCENT" | "ENCLOSE" | "BIGOP" | "INTOP" | "SUMOP") => {
      false
    },
    Some("SUPERSCRIPTOP" | "SUBSCRIPTOP") => !op
      .attribute("scriptpos")
      .is_some_and(|p| p.starts_with("mid")),
    _ => true,
  }
}

/// `node`'s printed items, in print order, into `out`.
fn flatten<N: FmMathNode>(node: &N, out: &mut Vec<Item>) {
  let Some(name) = node.ltx_name() else {
    if let Some(text) = node.text().filter(|t| !t.trim().is_empty()) {
      out.push(Item::Ink { text, text_box: false });
    }
    return;
  };
  // (an enclosure prints its content inside one frame, MathML.pm L338-342: `\boxed{x^1}` as digested is an
  // `XMArg[@enclose]`, amsmath.sty.ltxml:893-895, as parsed an `XMApp[@enclose]`)
  if node.attribute("enclose").is_some() {
    out.push(Item::Ink {
      text:     printed(node),
      text_box: false,
    });
    return;
  }
  match name.as_str() {
    "XMHint" | "XMRef" => {},
    "XMWrap" | "XMArg" => node.children().iter().for_each(|c| flatten(c, out)),
    "XMDual" => {
      if let Some(presentation) = elements(node).last() {
        flatten(presentation, out);
      }
    },
    "XMApp" => {
      if let Some(superscript) = node.attribute("role").as_deref().and_then(digested_script) {
        let text = printed(node);
        // (a limit set over or under the item before it, `scriptpos` `mid`, prints with it: ink, as parsed)
        out.push(
          if node
            .attribute("scriptpos")
            .is_some_and(|p| p.starts_with("mid"))
          {
            Item::Ink { text, text_box: false }
          } else {
            Item::Script { superscript, text }
          },
        );
        return;
      }
      // (a two-dimensional layout is one printed item: a script under a fraction's bar or a root's sign is no script
      // the formula ends on, review 90a r3 F1)
      if !row_layout(node) {
        out.push(Item::Ink {
          text:     printed(node),
          text_box: false,
        });
        return;
      }
      // (as parsed: the script operator, its base, its script)
      let parts = elements(node);
      if let [op, base, script] = parts.as_slice()
        && op.ltx_name().as_deref() == Some("XMTok")
        && let Some(superscript) = match op.attribute("role").as_deref() {
          Some("SUPERSCRIPTOP") => Some(true),
          Some("SUBSCRIPTOP") => Some(false),
          _ => None,
        }
      {
        let item = Item::Script {
          superscript,
          text: printed(script),
        };
        // (a prescript prints before its base: `^{a})`, ³He)
        if op
          .attribute("scriptpos")
          .is_some_and(|p| p.starts_with("pre"))
        {
          out.push(item);
          flatten(base, out);
        } else {
          flatten(base, out);
          out.push(item);
        }
        return;
      }
      // (any other row holding a script: its operator and arguments as they print — a script the parser nested under
      // the invisible times of a word, `$Ann^{1}$` as `⁢(A, n, n¹)`, `$\mathrm{Ann~Able}^{1}$`, `$Smith-Jones^1$` as
      // `−(⁢(S, …), ⁢(J, …, s¹))`, is read as the digested shape's flat items are; one holding none prints one item)
      if holds_a_script(node) {
        printed_children(node).iter().for_each(|c| flatten(c, out));
      } else {
        out.push(Item::Ink {
          text:     printed(node),
          text_box: false,
        });
      }
    },
    other => out.push(Item::Ink {
      text:     printed(node),
      text_box: other == "XMText",
    }),
  }
}

/// The printed text an item holds.
fn item_text(item: &Item) -> &str {
  match item {
    Item::Script { text, .. } | Item::Ink { text, .. } => text,
  }
}

fn has_word(text: &str) -> bool { text.chars().any(char::is_alphanumeric) }

/// A formula `math` (an `ltx:Math`, or its `ltx:XMath`) as the frontmatter reads it ([`FormulaMarks`]; the module
/// docs).
pub fn read_formula<N: FmMathNode>(math: &N) -> FormulaMarks {
  let xmath = match math.ltx_name().as_deref() {
    Some("XMath") => Some(math.children()),
    Some("Math") => elements(math)
      .into_iter()
      .find(|n| n.ltx_name().as_deref() == Some("XMath"))
      .map(|x| x.children()),
    _ => None,
  };
  let Some(children) = xmath else {
    return FormulaMarks::Other;
  };
  let mut items = Vec::new();
  for child in &children {
    flatten(child, &mut items);
  }
  let Some(first) = items.iter().position(|i| matches!(i, Item::Script { .. })) else {
    return FormulaMarks::Other;
  };
  let (lead, rest) = items.split_at(first);
  let base: String = lead.iter().map(item_text).collect();
  // (the superscripts' texts, and the subscripts', to the first letter or digit printed after them)
  let mut scripts = Vec::new();
  let mut subscripts = String::new();
  let mut word_after = false;
  for item in rest {
    match item {
      Item::Ink { text, .. } if has_word(text) => {
        word_after = true;
        break;
      },
      Item::Script { superscript: true, text, .. } => scripts.push(text.clone()),
      Item::Script { text, .. } => subscripts.push_str(text),
      _ => {},
    }
  }
  if scripts.is_empty() {
    return FormulaMarks::Other;
  }
  if base.trim().is_empty() {
    // (floating scripts opening the formula: a mark, unless the item printed right after them is a letter or digit —
    // their base, a prescript on a printed word: an isotope; punctuation or a symbol is none, `^{a})`, `^1,2`)
    let base_after = rest
      .iter()
      .find(|i| matches!(i, Item::Ink { .. }))
      .is_some_and(|i| has_word(item_text(i)));
    // (a script after a box that prints nothing, `$\mbox{ }^1$` — cond-mat0206265 — floats as printed: its digested
    // role is `POSTSUPERSCRIPT`, its base blank)
    return if base_after {
      FormulaMarks::Other
    } else {
      FormulaMarks::Marks(scripts)
    };
  }
  if word_after {
    return FormulaMarks::Other;
  }
  // (a base is a text box, or a word or a number: `97403`, `Ul. Ainy $299^A$`'s 299 — 2203.01892, 0707.4047)
  let text_box = matches!(lead, [Item::Ink { text_box: true, .. }]);
  let word = base
    .chars()
    .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c.is_whitespace());
  if text_box || word {
    // (a subscript is the base's: `$G_2'$`, 2208.14409's keyword "G₂′")
    FormulaMarks::OnBase {
      base: format!("{}{subscripts}", base.trim()),
      scripts,
    }
  } else {
    FormulaMarks::Other
  }
}

/// A superscript of one or two lowercase letters glued on both sides to letters (`James M<sup>c</sup>Kernan`,
/// math0610203; `I. M. M$^{\rm c}$Hardy`, 2003.01448): part of the word, no mark (the lexer's reading, `events::lexer`).
/// `before`/`after`: the text printed just before and after it, when a text node.
pub fn glued_in_a_word(letters: &str, before: Option<&str>, after: Option<&str>) -> bool {
  (1..=2).contains(&letters.chars().count())
    && letters.chars().all(char::is_lowercase)
    && before
      .and_then(|t| t.chars().next_back())
      .is_some_and(char::is_alphabetic)
    && after
      .and_then(|t| t.chars().next())
      .is_some_and(char::is_alphabetic)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A test node: an element `(name, attributes, children)` or a text.
  #[derive(Clone)]
  enum T {
    E(&'static str, Vec<(&'static str, &'static str)>, Vec<T>),
    X(&'static str),
  }

  impl FmMathNode for T {
    fn ltx_name(&self) -> Option<String> {
      match self {
        T::E(name, ..) => Some((*name).to_string()),
        T::X(_) => None,
      }
    }

    fn attribute(&self, name: &str) -> Option<String> {
      match self {
        T::E(_, attrs, _) => attrs
          .iter()
          .find(|(k, _)| *k == name)
          .map(|(_, v)| (*v).to_string()),
        T::X(_) => None,
      }
    }

    fn children(&self) -> Vec<Self> {
      match self {
        T::E(_, _, kids) => kids.clone(),
        T::X(_) => Vec::new(),
      }
    }

    fn text(&self) -> Option<String> {
      match self {
        T::X(t) => Some((*t).to_string()),
        T::E(..) => None,
      }
    }
  }

  fn tok(t: &'static str) -> T { T::E("XMTok", vec![], vec![T::X(t)]) }
  fn hint() -> T { T::E("XMHint", vec![("width", "2.8pt")], vec![]) }
  fn script(role: &'static str, kids: Vec<T>) -> T {
    T::E("XMApp", vec![("role", role)], vec![T::E(
      "XMArg",
      vec![("rule", "Superscript")],
      kids,
    )])
  }
  fn math(kids: Vec<T>) -> T { T::E("Math", vec![], vec![T::E("XMath", vec![], kids)]) }
  fn op(pos: &'static str) -> T {
    T::E(
      "XMTok",
      vec![("role", "SUPERSCRIPTOP"), ("scriptpos", pos)],
      vec![],
    )
  }
  fn marks(texts: &[&str]) -> FormulaMarks {
    FormulaMarks::Marks(texts.iter().map(|s| (*s).to_string()).collect())
  }

  /// The shapes of `agent_reports/2026-10-29_rf1_xmath_shapes.md`, as digested and as parsed: floating scripts with
  /// no letter or digit printed right after them, the formula's end, spacing, punctuation or a symbol.
  #[test]
  fn floating_scripts_before_no_letter_or_digit_are_a_mark() {
    let float = |kids| script("FLOATSUPERSCRIPT", kids);
    // as digested: `$^{1}\hspace{1mm}$`, `$\,^{3}$`, `${}^{a,\;b}$`, `$^{a})$`, `$^{1}{}^{,}$`, `$\rm ^a$`
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("1")]), hint()])),
      marks(&["1"])
    );
    assert_eq!(
      read_formula(&math(vec![hint(), float(vec![tok("3")])])),
      marks(&["3"])
    );
    assert_eq!(
      read_formula(&math(vec![float(vec![
        tok("a"),
        tok(","),
        hint(),
        tok("b")
      ])])),
      marks(&["a,b"])
    );
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("a")]), tok(")")])),
      marks(&["a"])
    );
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("1")]), float(vec![tok(",")])])),
      marks(&["1", ","])
    );
    // `$\mbox{ }^1$` (cond-mat0206265): a script after a blank box floats
    let blank = T::E("XMText", vec![], vec![T::X(" ")]);
    assert_eq!(
      read_formula(&math(vec![
        blank,
        script("POSTSUPERSCRIPT", vec![tok("1")])
      ])),
      marks(&["1"])
    );
    // `^1,2` (1912.02075's `A.P.~Orlov$^1,2$`): the comma is the script's base, no word — a mark, its marks before the 2
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("1")]), tok(","), tok("2")])),
      marks(&["1"])
    );
    // as parsed: a surviving float under a dual's wrap; a prescript on ")"
    let dual = T::E("XMDual", vec![], vec![
      T::E("XMRef", vec![], vec![]),
      T::E("XMWrap", vec![], vec![
        T::E("XMApp", vec![("role", "FLOATSUPERSCRIPT")], vec![tok("1")]),
        hint(),
      ]),
    ]);
    assert_eq!(read_formula(&math(vec![dual])), marks(&["1"]));
    let pre = T::E("XMApp", vec![], vec![op("pre1"), tok(")"), tok("a")]);
    assert_eq!(read_formula(&math(vec![pre])), marks(&["a"]));
  }

  #[test]
  fn a_script_before_or_on_printed_ink_is_no_mark() {
    let float = |kids| script("FLOATSUPERSCRIPT", kids);
    let post = |kids| script("POSTSUPERSCRIPT", kids);
    // an isotope: `${\rm ^{3}He}$` as digested, and as parsed
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("3")]), tok("H"), tok("e")])),
      FormulaMarks::Other
    );
    let pre = T::E("XMApp", vec![], vec![op("pre2"), tok("He"), tok("3")]);
    assert_eq!(read_formula(&math(vec![pre])), FormulaMarks::Other);
    // `^{3}\alpha`: a letter right after the script, its base
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("3")]), tok("\u{3b1}")])),
      FormulaMarks::Other
    );
    // a base: `$97403^{1}$`, `$\text{Leon Bello}^{1,3}$`, both as digested and parsed
    let digits: Vec<T> = ["9", "7", "4", "0", "3"].into_iter().map(tok).collect();
    let mut kids = digits;
    kids.push(post(vec![tok("1")]));
    let on = |base: &str, s: &str| FormulaMarks::OnBase {
      base:    base.to_string(),
      scripts: vec![s.to_string()],
    };
    assert_eq!(read_formula(&math(kids)), on("97403", "1"));
    // a script the parser nests under the invisible times of a word, as the digested shape's flat items read
    // (`$Ann^{1}$`: `⁢(A, n, n¹)`; `$\mathrm{Ann~Able}^{1}$`: `⁢(Ann, Able¹)`; `$Smith-Jones^1$`:
    // `−(⁢(Smith), ⁢(J, o, n, e, s¹))`; review 90a r2 F1)
    let times = || {
      T::E(
        "XMTok",
        vec![("meaning", "times"), ("role", "MULOP")],
        vec![T::X("\u{2062}")],
      )
    };
    let app = |kids| T::E("XMApp", vec![], kids);
    let on_base = |base, s| app(vec![op("post1"), base, tok(s)]);
    let ann = |n| app(vec![times(), tok("A"), tok("n"), on_base(tok("n"), n)]);
    assert_eq!(read_formula(&math(vec![ann("1")])), on("Ann", "1"));
    let ann_digested = vec![tok("A"), tok("n"), tok("n"), post(vec![tok("1")])];
    assert_eq!(read_formula(&math(ann_digested)), on("Ann", "1"));
    let able = app(vec![times(), tok("Ann"), on_base(tok("Able"), "1")]);
    assert_eq!(read_formula(&math(vec![able])), on("AnnAble", "1"));
    let able_digested = vec![
      tok("A"),
      tok("n"),
      tok("n"),
      hint(),
      tok("A"),
      tok("b"),
      tok("l"),
      tok("e"),
      post(vec![tok("1")]),
    ];
    assert_eq!(read_formula(&math(able_digested)), on("AnnAble", "1"));
    let minus = T::E(
      "XMTok",
      vec![("meaning", "minus"), ("role", "ADDOP")],
      vec![T::X("-")],
    );
    let jones = app(vec![
      times(),
      tok("J"),
      tok("o"),
      tok("n"),
      tok("e"),
      on_base(tok("s"), "1"),
    ]);
    let smith = app(vec![times(), tok("Smith")]);
    assert_eq!(
      read_formula(&math(vec![app(vec![minus, smith, jones])])),
      on("Smith-Jones", "1")
    );
    // a prescript on an accented letter, `$^{1}\mathrm{\acute{e}}$`: an application holding no script prints one item,
    // a letter (review 90a r2's probes AFs20, ANs20), as digested and as parsed
    let acute = || {
      T::E(
        "XMTok",
        vec![("name", "acute"), ("role", "OVERACCENT")],
        vec![T::X("\u{b4}")],
      )
    };
    let accented = app(vec![acute(), T::E("XMArg", vec![], vec![tok("e")])]);
    assert_eq!(
      read_formula(&math(vec![float(vec![tok("1")]), accented])),
      FormulaMarks::Other
    );
    let pre_accented = app(vec![op("pre1"), app(vec![acute(), tok("e")]), tok("1")]);
    assert_eq!(read_formula(&math(vec![pre_accented])), FormulaMarks::Other);
    // a two-dimensional layout is one printed item, as digested and as parsed (review 90a r3 F1; Perl `Post/MathML.pm`
    // `mfrac`, `msqrt`, `mroot`, `menclose`, `mover`): a script under a fraction's bar, a root's sign, a box or an
    // accent is no script the formula ends on — `\frac{a}{b^{1}}`, `\sqrt{x^{1}}`, `\sqrt[3]{x^{1}}`, `\boxed{x^{1}}`,
    // `\hat{x^{1}}`, `\stackrel{a}{b^{1}}` (a script set over its base, `scriptpos` `mid`)
    let arg = |kids| T::E("XMArg", vec![], kids);
    let with =
      |name: &'static str, attrs: Vec<(&'static str, &'static str)>, text: &'static str| {
        T::E(
          name,
          attrs,
          if text.is_empty() {
            vec![]
          } else {
            vec![T::X(text)]
          },
        )
      };
    let frac = || with("XMTok", vec![("meaning", "divide"), ("role", "FRACOP")], "");
    let sqrt = || with("XMTok", vec![("meaning", "square-root")], "");
    let root = || with("XMTok", vec![("meaning", "nth-root")], "");
    let hat = || with("XMTok", vec![("name", "hat"), ("role", "OVERACCENT")], "^");
    let mid = T::E(
      "XMTok",
      vec![("role", "SUPERSCRIPTOP"), ("scriptpos", "mid1")],
      vec![],
    );
    let x1 = || vec![tok("x"), post(vec![tok("1")])];
    let boxed = |kids| T::E("XMApp", vec![("enclose", "box")], kids);
    let layouts = [
      // (as digested, as parsed)
      (
        app(vec![
          frac(),
          arg(vec![tok("a")]),
          arg(vec![tok("b"), post(vec![tok("1")])]),
        ]),
        app(vec![frac(), tok("a"), on_base(tok("b"), "1")]),
      ),
      (
        app(vec![sqrt(), arg(x1())]),
        app(vec![sqrt(), on_base(tok("x"), "1")]),
      ),
      (
        app(vec![root(), arg(vec![tok("3")]), arg(x1())]),
        app(vec![root(), tok("3"), on_base(tok("x"), "1")]),
      ),
      (
        T::E("XMArg", vec![("enclose", "box")], x1()),
        boxed(vec![op("post1"), tok("x"), tok("1")]),
      ),
      (
        app(vec![hat(), arg(x1())]),
        app(vec![hat(), on_base(tok("x"), "1")]),
      ),
      (
        T::E("XMApp", vec![("role", "RELOP")], vec![
          mid.clone(),
          arg(vec![tok("b"), post(vec![tok("1")])]),
          arg(vec![tok("a")]),
        ]),
        T::E("XMApp", vec![("role", "RELOP")], vec![
          mid,
          on_base(tok("b"), "1"),
          tok("a"),
        ]),
      ),
    ];
    for (digested, parsed) in layouts {
      assert_eq!(read_formula(&math(vec![digested])), FormulaMarks::Other);
      assert_eq!(read_formula(&math(vec![parsed])), FormulaMarks::Other);
    }
    // a script on the last item of a parsed list, `$a,b^{1}$`: its base "a,b" is no word, the formula's text (as digested)
    let list = T::E("XMDual", vec![], vec![
      T::E("XMRef", vec![], vec![]),
      T::E("XMWrap", vec![], vec![
        tok("a"),
        tok(","),
        on_base(tok("b"), "1"),
      ]),
    ]);
    assert_eq!(read_formula(&math(vec![list])), FormulaMarks::Other);
    let text = T::E("XMText", vec![], vec![T::X("Leon Bello")]);
    assert_eq!(
      read_formula(&math(vec![
        text.clone(),
        post(vec![tok("1"), tok(","), tok("3")])
      ])),
      on("Leon Bello", "1,3")
    );
    let parsed = T::E("XMApp", vec![], vec![op("post1"), text, tok("1,3")]);
    assert_eq!(read_formula(&math(vec![parsed])), on("Leon Bello", "1,3"));
    // a subscript is the base's (`$G_2'$`, 2208.14409)
    assert_eq!(
      read_formula(&math(vec![
        tok("G"),
        script("POSTSUBSCRIPT", vec![tok("2")]),
        post(vec![tok("\u{2032}")])
      ])),
      on("G2", "\u{2032}")
    );
    // a base no word: `$a+b^{2}$`; no superscript: `$_{1}$`; no script: `$x$`
    assert_eq!(
      read_formula(&math(vec![
        tok("a"),
        tok("+"),
        tok("b"),
        post(vec![tok("2")])
      ])),
      FormulaMarks::Other
    );
    assert_eq!(
      read_formula(&math(vec![script("FLOATSUBSCRIPT", vec![tok("1")])])),
      FormulaMarks::Other
    );
    assert_eq!(read_formula(&math(vec![tok("x")])), FormulaMarks::Other);
  }

  #[test]
  fn a_lowercase_script_glued_to_letters_is_the_words() {
    assert!(glued_in_a_word("c", Some("James M"), Some("Kernan")));
    assert!(!glued_in_a_word("c", Some("James M "), Some("Kernan")));
    assert!(!glued_in_a_word("1", Some("M"), Some("K")));
    assert!(!glued_in_a_word("abc", Some("M"), Some("K")));
  }
}
