//! The `\lxDeclare` pattern compiler and its paired structural matcher.
//!
//! Perl keeps this machinery in `Core/Rewrite.pm` (`domToXPath` digests the
//! declaration pattern and compiles an XPath with baked-in predicates); the
//! Rust port instead recognizes the pattern SOURCE string (one arm per
//! structural family), emits a deliberately BROAD XPath, and verifies each
//! match Rust-side in [`declare_node_matches`] — sidestepping the nested
//! XPath-predicate problems and the font-at-rewrite-time trap (see
//! `base_text_predicate`). The compiler ([`compile_declare_pattern`]) and the
//! matcher are a PAIRED construction: every [`DeclarePatternType`] variant has
//! one arm in each, and both matches are exhaustive so adding a family breaks
//! both at compile time (the same drift-protection principle as the
//! fingerprint/estimate pair in `digested.rs`).

use libxml::tree::Node;

use crate::document::Document;

/// Structural family of a compiled `\lxDeclare` pattern. One variant per
/// compiler arm; consumed exhaustively by [`declare_node_matches`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarePatternType {
  /// Bare token / symbol command (`x`, `\pi`) — the XPath is exact.
  Simple,
  /// Wildcard subscript `x_\WildCard` / `x_{\WildCard,...}` (arity in sub_text).
  Subscript,
  /// Literal subscript `x_1`, `x_{2n-1}`.
  LiteralSubscript,
  /// Prime `x'` / `x^{\prime}`.
  Prime,
  /// Accent `\hat{\WildCard}` / `\hat{x}`.
  Accent,
  /// Function application `f\WildCard[(\WildCard,...)]` (arity in sub_text).
  FuncApply,
  /// Leading wildcard with literal content `\WildCard[a]b`.
  LeadWild,
  /// Command application `\cs{\WildCard}...` matching the use-site XMDual.
  CmdDual,
  /// Unrecognized pattern — compiles to an empty XPath, never matches.
  Unknown,
}

/// Metadata for a compiled \lxDeclare pattern.
/// Contains the XPath, pattern type for Rust-side filtering, and wildcard info.
#[derive(Debug, Clone)]
pub struct DeclarePattern {
  pub xpath:          String,
  pub pattern_type:   DeclarePatternType,
  /// Base token text for subscript/prime/accent base matching (e.g. "x")
  pub base_text:      Option<String>,
  /// For literal subscripts: the subscript content text (e.g. "1")
  pub sub_text:       Option<String>,
  /// For accent patterns: the accent name (e.g. "hat")
  pub accent_name:    Option<String>,
  #[allow(dead_code)]
  pub has_wildcard:   bool,
  pub wildcard_paths: Option<Vec<Vec<usize>>>,
  /// Font CLASS the matched base must carry (e.g. "caligraphic"), checked
  /// Rust-side — never baked into the XPath (see base_text_predicate).
  pub font_class:     Option<&'static str>,
}

impl DeclarePattern {
  /// Number of sibling nodes the match spans — Perl's `$nnodes` from
  /// `domToXPath` (Rewrite.pm). Subscript/prime patterns match the base
  /// XMTok plus its POSTSUBSCRIPT/POSTSUPERSCRIPT sibling; accents match
  /// the single XMApp; function applications span base + `(` + n args +
  /// (n-1) commas + `)`.
  pub fn select_count(&self) -> Option<usize> {
    match self.pattern_type {
      DeclarePatternType::LiteralSubscript
      | DeclarePatternType::Prime
      | DeclarePatternType::Subscript => Some(2),
      DeclarePatternType::Accent => Some(1),
      DeclarePatternType::FuncApply => self
        .sub_text
        .as_deref()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|n| 2 * n + 2),
      // wildcard content tokens + literal suffix tokens
      DeclarePatternType::LeadWild => match (&self.base_text, &self.sub_text) {
        (Some(content), Some(suffix)) => Some(content.chars().count() + suffix.chars().count()),
        _ => None,
      },
      _ => None,
    }
  }
}

/// Generate an XPath text predicate for a base token specification, plus a
/// font-CLASS requirement checked RUST-SIDE (declare_node_matches).
///
/// NEVER bake `@font` into the XPath: the serialized attribute does not
/// exist at rewrite time (only the interned `_font` id does), so a
/// `@font='caligraphic'` predicate silently matches NOTHING — the historical
/// wildcard-vanish failure mode (declare.tex golden was 51 decl_id vs Perl's
/// 84 until 2026-07-03). Likewise digestion stamps command tokens with
/// `@name` (e.g. varepsilon), not only `@meaning` — accept either.
fn base_text_predicate(base: &str) -> (String, Option<&'static str>) {
  if base.starts_with('\\') {
    let cmd = base.trim_start_matches('\\');
    if let Some(inner) = cmd
      .strip_prefix("mathcal{")
      .and_then(|s| s.strip_suffix('}'))
    {
      (format!("text()='{inner}'"), Some("caligraphic"))
    } else {
      (format!("(@meaning='{cmd}' or @name='{cmd}')"), None)
    }
  } else {
    (format!("text()='{}'", base.replace('\'', "&apos;")), None)
  }
}

/// Compile a \lxDeclare body_text into pattern metadata.
/// Handles both wildcard and non-wildcard patterns.
///
/// Perl: compile_match1 digests tokens to DOM, then domToXPath.
/// Rust: pattern-match on body_text string and generate broad XPath
/// with Rust-side filtering criteria (avoids XPath nested predicate bug).
///
/// Font-awareness is deliberately NOT baked into these XPaths: the serialized
/// `@font` attribute is only finalized after math parsing, so a rewrite-time
/// `@font='…'` predicate matches nothing (and would silently break the
/// wildcard/subscript/prime rewrites). Font discrimination happens Rust-side
/// instead — `declare_node_matches` (rewrite path, via the resolved `_font`
/// id) and `apply_lx_declarations` (post-rewrite fast path, via match_font).
pub fn compile_declare_pattern(body_text: &str) -> DeclarePattern {
  // === Subscript patterns ===
  // IMPORTANT: Rewrites run BEFORE math parsing. The pre-parsed DOM has:
  //   <XMTok>x</XMTok> <XMApp role="POSTSUBSCRIPT"><XMTok>n</XMTok></XMApp>
  // NOT the post-parsed: <XMApp><XMTok role="SUBSCRIPTOP"/><XMTok>x</XMTok><XMTok>n</XMTok></XMApp>
  // Match the BASE XMTok, with select_count=2 to include the POSTSUBSCRIPT sibling.
  // Rust-side filtering verifies the sibling structure.

  // Wildcard: x_\WildCard, \varepsilon_\WildCard, \mathcal{T}_\WildCard
  if let Some(base) = body_text.strip_suffix("_\\WildCard") {
    let base = base.trim().to_string();
    let (base_pred, font_class) = base_text_predicate(&base);
    return DeclarePattern {
      // Match the base XMTok; Rust-side filter checks POSTSUBSCRIPT sibling
      xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type: DeclarePatternType::Subscript,
      base_text: Some(base),
      sub_text: None,
      accent_name: None,
      has_wildcard: true,
      // Wildcard = child 1 of sibling 2 (the content of POSTSUBSCRIPT XMApp)
      wildcard_paths: Some(vec![vec![2, 1]]),
      font_class,
    };
  }
  // Braced wildcard subscripts: x_{\WildCard}, x_{\WildCard,\WildCard}
  if body_text.contains("_{\\WildCard")
    && let Some(idx) = body_text.find("_{")
  {
    let base = body_text[..idx].trim().to_string();
    let (base_pred, font_class) = base_text_predicate(&base);
    let brace_content = &body_text[idx + 2..body_text.len().saturating_sub(1)];
    let nwilds = brace_content.matches("\\WildCard").count();
    // Perl semantics diverge by arity (Rewrite.pm domToXPath):
    //  - ONE wildcard: the XMArg-single-wildcard branch matches the WHOLE
    //    subscript argument regardless of content (that is the fixture's
    //    "accidental" q_{a+b} match) — wildcard = child 1 of sibling 2.
    //  - TWO+: the wildcards and literal commas compile as a positional
    //    child sequence [*, ',', *, ...] — wildcard i = content child 2i-1
    //    (commas at the even positions), and declare_node_matches must
    //    verify the comma-list shape (`sub_text` carries the arity).
    let (wpaths, sub_text) = if nwilds <= 1 {
      (vec![vec![2, 1]], None)
    } else {
      (
        (1..=nwilds).map(|i| vec![2, 1, 2 * i - 1]).collect(),
        Some(nwilds.to_string()),
      )
    };
    return DeclarePattern {
      xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type: DeclarePatternType::Subscript,
      base_text: Some(base),
      sub_text,
      accent_name: None,
      has_wildcard: true,
      wildcard_paths: Some(wpaths),
      font_class,
    };
  }
  // Literal subscript: x_1, x_{1}, x_{2n-1}
  // Pre-parsed: XMTok[x] + XMApp[POSTSUBSCRIPT, XMTok[1]]
  if let Some((base, sub)) = parse_subscript_literal(body_text) {
    let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
    return DeclarePattern {
      xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type:   DeclarePatternType::LiteralSubscript,
      base_text:      Some(base),
      sub_text:       Some(sub),
      accent_name:    None,
      has_wildcard:   false,
      wildcard_paths: None,
      font_class:     None,
    };
  }

  // === Accent patterns ===
  // Wildcard accent: \hat{\WildCard}, \widehat{\WildCard}
  for accent in &[
    "hat", "widehat", "tilde", "bar", "vec", "dot", "ddot", "check", "breve",
  ] {
    let pattern = format!("\\{accent}{{\\WildCard}}");
    if body_text == pattern {
      return DeclarePattern {
        // Broad: match any XMApp. Rust filters by accent name in first child.
        xpath:          "descendant-or-self::*[local-name()='XMApp']".to_string(),
        pattern_type:   DeclarePatternType::Accent,
        base_text:      None,
        sub_text:       None,
        accent_name:    Some(accent.to_string()),
        has_wildcard:   true,
        // Wildcard = child 2 (base content) of the accent XMApp
        wildcard_paths: Some(vec![vec![1, 2]]),
        font_class:     None,
      };
    }
  }
  // Literal accent: \hat{x}, \widehat{x}
  for accent in &[
    "hat", "widehat", "tilde", "bar", "vec", "dot", "ddot", "check", "breve",
  ] {
    if let Some(rest) = body_text.strip_prefix(&format!("\\{accent}{{"))
      && let Some(inner) = rest.strip_suffix('}')
      && !inner.contains("WildCard")
    {
      return DeclarePattern {
        xpath:          "descendant-or-self::*[local-name()='XMApp']".to_string(),
        pattern_type:   DeclarePatternType::Accent,
        base_text:      Some(inner.to_string()),
        sub_text:       None,
        accent_name:    Some(accent.to_string()),
        has_wildcard:   false,
        wildcard_paths: None,
        font_class:     None,
      };
    }
  }

  // === Prime pattern ===
  // x^{\prime} → after parsing: XMApp[SUPERSCRIPTOP, XMTok(x), XMTok(prime)]
  // Match the XMApp with SUPERSCRIPTOP and base text.
  if let Some(base) = body_text.strip_suffix("^{\\prime}") {
    let base = base.trim().to_string();
    if !base.is_empty() && !base.contains('\\') {
      let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
      return DeclarePattern {
        // Pre-parsed: XMTok[x] + XMApp[POSTSUPERSCRIPT, XMTok[prime]]
        xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type:   DeclarePatternType::Prime,
        base_text:      Some(base),
        sub_text:       None,
        accent_name:    None,
        has_wildcard:   false,
        wildcard_paths: None,
        font_class:     None,
      };
    }
  }
  // Also handle raw prime: x'
  if body_text.ends_with('\'') && body_text.len() > 1 {
    let base = body_text[..body_text.len() - 1].trim().to_string();
    if !base.is_empty() && !base.contains('\\') {
      let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
      return DeclarePattern {
        // Pre-parsed: XMTok[x] + XMApp[POSTSUPERSCRIPT, XMTok[prime]]
        xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type:   DeclarePatternType::Prime,
        base_text:      Some(base),
        sub_text:       None,
        accent_name:    None,
        has_wildcard:   false,
        wildcard_paths: None,
        font_class:     None,
      };
    }
  }

  // === Function application: base\WildCard[(\WildCard)] / [(\WildCard,\WildCard)] ===
  // Perl digests `\WildCard[content]` into <_WildCard_>content</_WildCard_>;
  // domToXPath (Rewrite.pm L443-450) compiles the CONTENT as literal following
  // siblings of the base — `(`, one single-node arg per \WildCard (comma-
  // separated), `)` at exact positions — and counts EVERY content node as a
  // wildcard position. So `f\WildCard[(\WildCard)]` matches the pre-parse
  // token run `f ( a )` (single-token args only: `f(a+b)` does NOT match,
  // its `)` sits past the position predicate), marking the base as the
  // non-wildcard attribute carrier (nowrap) or wrapping the span in an
  // XMDual whose content applies the decl-op to XMRefs of `(`/arg/`)`.
  if let Some(idx) = body_text.find("\\WildCard[(") {
    let base = body_text[..idx].trim().to_string();
    let content = &body_text[idx + "\\WildCard[(".len()..];
    if let Some(args) = content.strip_suffix(")]")
      && !base.is_empty()
    {
      let parts: Vec<&str> = args.split(',').collect();
      if parts.iter().all(|p| p.trim() == "\\WildCard") {
        let nargs = parts.len();
        let (base_pred, font_class) = base_text_predicate(&base);
        // Sibling positions 2..=2n+2 (the whole parenthesized content) are
        // wildcards, matching Perl's `$n = scalar(@children)` counting.
        let span = 2 * nargs + 2;
        let wpaths = (2..=span).map(|i| vec![i]).collect();
        return DeclarePattern {
          xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
          pattern_type: DeclarePatternType::FuncApply,
          base_text: Some(base),
          sub_text: Some(nargs.to_string()),
          accent_name: None,
          has_wildcard: true,
          wildcard_paths: Some(wpaths),
          font_class,
        };
      }
    }
  }

  // === Leading wildcard with literal content: \WildCard[a]b, \WildCard[ab]c ===
  // Perl digests `\WildCard[content]suffix` to [_WildCard_[tokens...], tokens...];
  // domToXPath compiles the wildcard CONTENT as the leading match span (every
  // content token a wildcard position — including the matched node itself,
  // sibling 1) followed by the literal suffix tokens at exact positions. With
  // nowrap, the attributes land on the first NON-wildcard node = the suffix.
  if let Some(rest) = body_text.strip_prefix("\\WildCard[")
    && let Some(close) = rest.find(']')
  {
    let content = &rest[..close];
    let suffix = &rest[close + 1..];
    if !content.is_empty()
      && !suffix.is_empty()
      && !content.contains('\\')
      && !suffix.contains('\\')
    {
      let k = content.chars().count();
      let first = content.chars().next().unwrap();
      let wpaths = (1..=k).map(|i| vec![i]).collect();
      return DeclarePattern {
        xpath:          format!(
          "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
          first.to_string().replace('\'', "&apos;")
        ),
        pattern_type:   DeclarePatternType::LeadWild,
        base_text:      Some(content.to_string()),
        sub_text:       Some(suffix.to_string()),
        accent_name:    None,
        has_wildcard:   true,
        wildcard_paths: Some(wpaths),
        font_class:     None,
      };
    }
  }

  // === Command application with wildcard args: \weird{\WildCard}{\WildCard} ===
  // A DefMath-defined command (e.g. via \lxDefMath) digests each USE into an
  // XMDual whose content arm is XMApp(XMTok[@name=cs], XMRef per arg). Perl
  // digests the pattern itself (with _WildCard_ args) and domToXPath matches
  // that dual; setAttributes_wild's single-XMDual branch then sets the
  // attributes (decl_id) directly on the dual node — mirrored here by the
  // "cmddual" Rust-side filter with no wildcard paths (nmatched=1).
  if body_text.starts_with('\\')
    && let Some(cmd_end) = body_text.find("{\\WildCard}")
  {
    let cmd = &body_text[1..cmd_end];
    let rest = &body_text[cmd_end..];
    if !cmd.is_empty() && cmd.chars().all(|c| c.is_ascii_alphabetic()) {
      let nargs = rest.matches("{\\WildCard}").count();
      if nargs >= 1 && rest == "{\\WildCard}".repeat(nargs) {
        return DeclarePattern {
          xpath:          "descendant-or-self::*[local-name()='XMDual']".to_string(),
          pattern_type:   DeclarePatternType::CmdDual,
          base_text:      Some(cmd.to_string()),
          sub_text:       Some(nargs.to_string()),
          accent_name:    None,
          has_wildcard:   true,
          wildcard_paths: None,
          font_class:     None,
        };
      }
    }
  }

  // === Bare math symbol command, e.g. "\pi", "\alpha", "\cpi" ===
  // Perl digests $\pi$ and matches the resulting XMTok via domToXPath. In our
  // pre-parse DOM the symbol carries a `name` attribute equal to the control
  // sequence (DefMath sets `name => <cs>`), so keying the match on @name is the
  // string-pattern equivalent and lets \lxDeclare target symbol commands.
  if let Some(cmd) = body_text.strip_prefix('\\')
    && !cmd.is_empty()
    && cmd.chars().all(|c| c.is_ascii_alphabetic())
  {
    return DeclarePattern {
      xpath:          format!(
        "descendant-or-self::*[local-name()='XMTok' and @name='{}']",
        cmd
      ),
      pattern_type:   DeclarePatternType::Simple,
      base_text:      None,
      sub_text:       None,
      accent_name:    None,
      has_wildcard:   false,
      wildcard_paths: None,
      font_class:     None,
    };
  }

  // === Fallback: simple token pattern ===
  // For single characters/words without special structure, match as XMTok by text.
  // This handles DefMathRewrite match strings like 'a', 'f', 'x', etc.
  if !body_text.is_empty() && !body_text.contains('\\') {
    return DeclarePattern {
      xpath:          format!(
        "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
        body_text.replace('\'', "&apos;")
      ),
      pattern_type:   DeclarePatternType::Simple,
      base_text:      None,
      sub_text:       None,
      accent_name:    None,
      has_wildcard:   false,
      wildcard_paths: None,
      font_class:     None,
    };
  }

  // Truly unrecognized pattern (e.g. complex TeX commands without matching rules)
  DeclarePattern {
    xpath:          String::new(),
    pattern_type:   DeclarePatternType::Unknown,
    base_text:      None,
    sub_text:       None,
    accent_name:    None,
    has_wildcard:   false,
    wildcard_paths: None,
    font_class:     None,
  }
}

/// Parse a literal (non-wildcard) subscript pattern like "x_1" or "x_{2n-1}".
/// Returns (base, subscript_content) if recognized.
fn parse_subscript_literal(body_text: &str) -> Option<(String, String)> {
  if body_text.contains("WildCard") {
    return None;
  }
  // Check for _ subscript
  let idx = body_text.find('_')?;
  let base = body_text[..idx].trim().to_string();
  if base.is_empty() {
    return None;
  }
  let sub = body_text[idx + 1..].trim();
  // Strip braces: {1} → 1, {2n-1} → 2n-1
  let sub = sub
    .strip_prefix('{')
    .and_then(|s| s.strip_suffix('}'))
    .unwrap_or(sub);
  Some((base, sub.to_string()))
}

/// Rust-side filtering for \lxDeclare pattern matching.
/// XPath matches are broad (to avoid nested predicate bugs); this function
/// verifies the matched node's children match the specific pattern.
///
/// Pattern types:
/// - "subscript": node is XMApp[@role='POSTSUBSCRIPT'], check base text + optional sub text
/// - "prime": node is XMApp[@role='POSTSUPERSCRIPT'], check base text
/// - "accent": node is XMApp, check accent name in first child, optional base text
/// - "simple": no extra filtering needed (XPath is specific enough)
pub fn declare_node_matches(document: &Document, node: &Node, pat: &DeclarePattern) -> bool {
  let base_text = pat.base_text.as_deref();
  let sub_text = pat.sub_text.as_deref();
  let accent_name = pat.accent_name.as_deref();
  let font_class = pat.font_class;
  // Font-CLASS requirement (e.g. caligraphic for a \mathcal pattern): the
  // XPath deliberately carries no @font predicate (the attribute is only an
  // interned `_font` id at rewrite time) — discriminate here on the RESOLVED
  // font instead. Class containment, not exact string (WISDOM: the exact
  // serialized font string is unreliable at rewrite time).
  if let Some(class) = font_class {
    let font = document.get_node_font(node);
    if !font.font_attribute_string().contains(class) {
      return false;
    }
  }
  let children = node.get_child_nodes();
  match pat.pattern_type {
    DeclarePatternType::LiteralSubscript => {
      // Matched node is the BASE XMTok. Check that next sibling is POSTSUBSCRIPT
      // with specific subscript content.
      let next_sib = node.get_next_sibling();
      let next_role = next_sib.as_ref().and_then(|s| s.get_property("role"));
      if next_role.as_deref() != Some("POSTSUBSCRIPT") {
        return false;
      }
      // Check subscript content text
      if let Some(sub) = sub_text {
        let sub_content = next_sib
          .as_ref()
          .map(|s| s.get_content())
          .unwrap_or_default();
        if sub_content.trim() != sub {
          return false;
        }
      }
      true
    },
    DeclarePatternType::Subscript => {
      // Wildcard subscript: matched node is BASE XMTok.
      // Check that next sibling is POSTSUBSCRIPT.
      let next_sib = node.get_next_sibling();
      let next_role = next_sib.as_ref().and_then(|s| s.get_property("role"));
      if next_role.as_deref() != Some("POSTSUBSCRIPT") {
        return false;
      }
      // Multi-wildcard `x_{\WildCard,\WildCard}`: sub_text carries the arity
      // and the subscript content must be EXACTLY the comma list
      // [any, ',', any, ...] — Perl compiles the literal commas as positional
      // child predicates, so `q_{a}` / `q_{a+b}` do NOT match a 2-ary pattern
      // (they fall to the 1-ary declaration, whose wildcard takes the whole
      // argument). Content children live under the POSTSUBSCRIPT's first
      // element child (the XMArg/XMWrap argument holder).
      if let Some(n) = sub_text.and_then(|s| s.parse::<usize>().ok())
        && n >= 2
      {
        let content: Vec<Node> = next_sib
          .as_ref()
          .and_then(|s| {
            s.get_child_nodes()
              .into_iter()
              .find(|c| c.get_type() == Some(libxml::tree::NodeType::ElementNode))
          })
          .map(|holder| {
            holder
              .get_child_nodes()
              .into_iter()
              .filter(|c| c.get_type() == Some(libxml::tree::NodeType::ElementNode))
              .collect()
          })
          .unwrap_or_default();
        if content.len() != 2 * n - 1 {
          return false;
        }
        for (i, c) in content.iter().enumerate() {
          // odd 0-based positions must be the literal comma separators
          if i % 2 == 1 && (c.get_name() != "XMTok" || c.get_content().trim() != ",") {
            return false;
          }
        }
      }
      true
    },
    DeclarePatternType::FuncApply => {
      // Matched node is the base XMTok of `base\WildCard[(\WildCard...)]`.
      // Require the EXACT following element siblings `(`, arg, [`,`, arg...],
      // `)` — single-node args in the pre-parse DOM, mirroring Perl
      // domToXPath_seq's position()=N predicates (so `f(a+b)` does not match
      // an 1-ary pattern: its `)` sits past the expected position).
      let Some(nargs) = sub_text.and_then(|s| s.parse::<usize>().ok()) else {
        return false;
      };
      let mut expected: Vec<Option<&str>> = vec![Some("(")];
      for i in 0..nargs {
        if i > 0 {
          expected.push(Some(","));
        }
        expected.push(None); // any single element node (the wildcard arg)
      }
      expected.push(Some(")"));
      let mut cur = node.clone();
      for want in expected {
        let mut next = cur.get_next_sibling();
        while let Some(ref s) = next {
          if s.get_type() == Some(libxml::tree::NodeType::ElementNode) {
            break;
          }
          next = s.get_next_sibling();
        }
        let Some(sib) = next else {
          return false;
        };
        if let Some(text) = want
          && (sib.get_name() != "XMTok" || sib.get_content().trim() != text)
        {
          return false;
        }
        cur = sib;
      }
      true
    },
    DeclarePatternType::CmdDual => {
      // `\cs{\WildCard}...`: matched node is an XMDual whose content arm is
      // XMApp(XMTok[@name=cs or @meaning=cs], one XMRef per wildcard arg).
      let (Some(cmd), Some(nargs)) = (base_text, sub_text.and_then(|s| s.parse::<usize>().ok()))
      else {
        return false;
      };
      let elem_children: Vec<Node> = children
        .iter()
        .filter(|c| c.get_type() == Some(libxml::tree::NodeType::ElementNode))
        .cloned()
        .collect();
      let Some(content) = elem_children.first() else {
        return false;
      };
      if content.get_name() != "XMApp" {
        return false;
      }
      let app_children: Vec<Node> = content
        .get_child_nodes()
        .into_iter()
        .filter(|c| c.get_type() == Some(libxml::tree::NodeType::ElementNode))
        .collect();
      if app_children.len() != nargs + 1 {
        return false;
      }
      let op = &app_children[0];
      op.get_name() == "XMTok"
        && (op.get_property("name").as_deref() == Some(cmd)
          || op.get_property("meaning").as_deref() == Some(cmd))
    },
    DeclarePatternType::LeadWild => {
      // `\WildCard[content]suffix`: matched node is the FIRST content token;
      // the remaining content chars and then the literal suffix chars must
      // follow as adjacent single-token element siblings (Perl domToXPath_seq
      // position()=N predicates).
      let (Some(content), Some(suffix)) = (base_text, sub_text) else {
        return false;
      };
      let expected: Vec<char> = content.chars().skip(1).chain(suffix.chars()).collect();
      let mut cur = node.clone();
      for want in expected {
        let mut next = cur.get_next_sibling();
        while let Some(ref s) = next {
          if s.get_type() == Some(libxml::tree::NodeType::ElementNode) {
            break;
          }
          next = s.get_next_sibling();
        }
        let Some(sib) = next else {
          return false;
        };
        if sib.get_name() != "XMTok" || sib.get_content().trim() != want.to_string() {
          return false;
        }
        cur = sib;
      }
      true
    },
    DeclarePatternType::Prime => {
      // Matched node is BASE XMTok. Check that next sibling is POSTSUPERSCRIPT
      // with prime content.
      let next_sib = node.get_next_sibling();
      let next_role = next_sib.as_ref().and_then(|s| s.get_property("role"));
      if next_role.as_deref() != Some("POSTSUPERSCRIPT") {
        return false;
      }
      // Check prime content
      let sup_content = next_sib
        .as_ref()
        .map(|s| s.get_content())
        .unwrap_or_default();
      sup_content.contains('′')
    },
    DeclarePatternType::Accent => {
      // XMApp with children: [accent_op, base_content]
      if children.len() < 2 {
        return false;
      }
      // Check accent name on first child
      if let Some(accent) = accent_name {
        let first_name = children[0]
          .get_property("name")
          .or_else(|| children[0].get_property("meaning"));
        if first_name.as_deref() != Some(accent) {
          return false;
        }
        // Accent ops should have OVERACCENT or UNDERACCENT role
        let role = children[0].get_property("role");
        let is_accent = role
          .as_deref()
          .map(|r| r.contains("ACCENT"))
          .unwrap_or(false);
        if !is_accent {
          return false;
        }
      }
      // Check base content text if specified
      if let Some(base) = base_text
        && !declare_base_matches(&children[1], base)
      {
        return false;
      }
      true
    },
    DeclarePatternType::Simple => {
      // Font check: plain declarations (e.g. $x$) should NOT match tokens with
      // non-default fonts (bold, caligraphic, typewriter).
      // Perl: font_match_xpaths generates XPath predicates from _font attribute.
      let font = document.get_node_font(node);
      if let Some(series) = font.get_series()
        && series.as_ref() == "bold"
      {
        return false;
      }
      if let Some(family) = font.get_family() {
        let fam = family.as_ref();
        if fam == "caligraphic" || fam == "typewriter" {
          return false;
        }
      }
      true
    },
    // Unknown compiles to an empty XPath and is never registered.
    DeclarePatternType::Unknown => true,
  }
}

/// Check if a node matches a base text specification.
/// Handles both plain text (e.g. "x") and command names (e.g. "\varepsilon").
fn declare_base_matches(node: &Node, base_spec: &str) -> bool {
  if base_spec.starts_with('\\') {
    // Command base: match by meaning or name attribute
    let cmd = base_spec.trim_start_matches('\\');
    // Handle \mathcal{X} → check font=caligraphic + text=X
    if let Some(inner) = cmd
      .strip_prefix("mathcal{")
      .and_then(|s| s.strip_suffix('}'))
    {
      let font = node.get_property("font").unwrap_or_default();
      let text = node.get_content();
      return font == "caligraphic" && text.trim() == inner;
    }
    // General command: check meaning attribute
    let meaning = node.get_property("meaning").unwrap_or_default();
    meaning == cmd
  } else {
    // Plain text base: match node text content
    let text = node.get_content();
    text.trim() == base_spec
  }
}
