//! The `\lxDeclare` / `\lxDefMath` declaration helpers — one Rust function
//! per Perl sub in `latexml.sty.ltxml`, so the binding stays diffable against
//! the reference source sub-by-sub:
//!
//! | Rust                         | Perl (latexml.sty.ltxml)                  |
//! |------------------------------|-------------------------------------------|
//! | [`next_declaration_id`]      | `next_declaration_id` (L544-550)          |
//! | [`split_declare_tag`]        | `splitDeclareTag` (L437-449)              |
//! | [`normalize_declare_keys`]   | `normalizeDeclareKeys` (L417-434)         |
//! | [`get_declaration_scope`]    | `getDeclarationScope` (L552-559)          |
//! | [`create_declaration_rewrite`] | `createDeclarationRewrite` (L561-580)   |
//! | [`emit_declare_element`]     | the shared `<ltx:declare>` constructor    |
//! |                              | body (`\lxDeclare` L474-485 /             |
//! |                              | `\@lxDefMathDeclare` L387-396)            |
//! | [`record_declaration_lines`] | Rust-only: the LATEXML_DECLARATIONS       |
//! |                              | fast-path registry consumed by            |
//! |                              | `apply_lx_declarations`                   |
//!
//! The pattern COMPILER lives with the rewrite engine
//! (`latexml_core::rewrite::declare`), matching Perl's `domToXPath` in
//! `Core/Rewrite.pm`.

use latexml_core::rewrite::declare::{DeclarePattern, DeclarePatternType, compile_declare_pattern};

use crate::prelude::*;

/// Perl `next_declaration_id()`: `StepCounter('@XMDECL')` then expand
/// `\the@XMDECL@ID`. The `@XMDECL` counter is subordinate to `section`, so
/// ids reset per-section: `S1.XMD1`, `S1.XMD2`, …, `S2.XMD1`, ….
pub(super) fn next_declaration_id() -> Result<String> {
  step_counter("@XMDECL", false)?;
  Ok(
    do_expand(T_CS!("\\the@XMDECL@ID"))
      .ok()
      .map(|t| t.to_string().trim().to_string())
      .unwrap_or_default(),
  )
}

/// Perl `splitDeclareTag`: the boxes before the first `:` box become the
/// TERM (typically math), the rest the description; no `:` → neither.
pub(super) fn split_declare_tag(
  stuff: &Digested,
) -> (Option<Vec<Digested>>, Option<Vec<Digested>>) {
  let boxes = stuff.unlist();
  match boxes
    .iter()
    .position(|b| b.get_string().map(|s| s.trim() == ":").unwrap_or(false))
  {
    Some(pos) => (Some(boxes[..pos].to_vec()), Some(boxes[pos + 1..].to_vec())),
    None => (None, None),
  }
}

/// Perl `normalizeDeclareKeys`: synthesize term/short/description for the
/// `<ltx:declare>` element out of the DIGESTED tag/description values — a
/// description like `$x$: a real variable` contains a real math box that
/// must survive to the term tag (the term Math is then itself subject to
/// the declaration rewrites). Stores the results as `*_boxes` whatsit
/// properties for the constructor half ([`emit_declare_element`]).
pub(super) fn normalize_declare_keys(
  whatsit: &mut Whatsit,
  tag_digested: Option<&Digested>,
  description_digested: Option<&Digested>,
) {
  let stuff = description_digested.or(tag_digested);
  let (term, mut desc) = stuff.map(split_declare_tag).unwrap_or((None, None));
  let short: Option<Vec<Digested>> = if description_digested.is_some() {
    tag_digested.map(|d| d.unlist()).or_else(|| desc.clone())
  } else {
    None
  };
  if desc.is_none() {
    desc = description_digested.or(tag_digested).map(|d| d.unlist());
  }
  if let Some(t) = term {
    whatsit.set_property("term_boxes", Stored::VecDigested(t));
  }
  if let Some(s) = short {
    whatsit.set_property("short_boxes", Stored::VecDigested(s));
  }
  if let Some(d) = desc {
    whatsit.set_property("desc_boxes", Stored::VecDigested(d));
  }
}

/// Register a declaration in the LATEXML_DECLARATIONS state string — the
/// math parser's font-aware fast-path registry (`apply_lx_declarations`).
/// Rust-only companion to the rewrite path.
///
/// Line format: `body_text \t role \t name \t meaning \t decl_id \t
/// match_font \t scope_prefix`. The trailing match_font makes
/// apply_lx_declarations font-aware (a plain italic `$x$` must not annotate
/// a bold `\mathbf{x}`), mirroring the font-aware rewrite path
/// (declare_node_matches). Empty when the pattern carried no distinguishing
/// font. The 7th field gates UNTAGGED `scope=section` declarations (no
/// decl_id to carry the section prefix) so the fast path doesn't apply them
/// document-globally (PR_READINESS cluster C).
pub(super) fn record_declaration_lines(
  body_text: &str,
  role: &str,
  name_val: &str,
  meaning: &str,
  decl_id: &str,
  match_font: Option<&str>,
  scope_opt: &str,
) -> Result<()> {
  let key = "LATEXML_DECLARATIONS";
  let mut decls: Vec<String> = match lookup_value(key) {
    Some(Stored::String(s)) => {
      let s_str = with(s, |r| r.to_string());
      if s_str.is_empty() {
        Vec::new()
      } else {
        s_str.split('\n').map(String::from).collect()
      }
    },
    _ => Vec::new(),
  };
  let match_font_field = match_font.unwrap_or("");
  let scope_prefix = if scope_opt == "section" {
    if !decl_id.is_empty() {
      decl_id.split('.').next().unwrap_or("").to_string()
    } else {
      // afterDigest — where \thesection@ID is still correct.
      do_expand(T_CS!("\\thesection@ID"))
        .ok()
        .map(|t| t.to_string().trim().to_string())
        .unwrap_or_default()
    }
  } else {
    String::new()
  };
  decls.push(format!(
    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
    body_text, role, name_val, meaning, decl_id, match_font_field, scope_prefix
  ));
  // Mathcode decoding for single-char bodies
  if body_text.chars().count() == 1 {
    let ch = body_text.chars().next().unwrap();
    if let Some(mathcode) = lookup_mathcode(&ch.to_string())
      && mathcode > 0
    {
      let decoded_pos = (mathcode % 256) as u8;
      let decoded_fam = (mathcode / 256) % 16;
      let font_key = format!("textfont_{decoded_fam}");
      if let Some(Stored::Token(ref ftok)) = lookup_value(&font_key) {
        // Extract encoding before calling font::decode — decode may
        // trigger preload_font_map → assign_value, and with_font_info
        // holds a State borrow while its closure runs (see
        // mathchar.rs fix for 0711.4787 RefCell panic pattern).
        let mut encoding_opt: Option<String> = with_font_info(ftok, |fontinfo| {
          if let Some(Stored::Font(info)) = fontinfo.unwrap_or(None) {
            info.encoding.as_ref().map(|s| s.to_string())
          } else {
            None
          }
        });
        // Fallback (mirror mathchar.rs L862-887): when `fontinfo_<cs>`
        // didn't round-trip through the dump as a `Stored::Font`, but
        // its `font_shared_key_<cs>` pointer DID, derive encoding from
        // the font name via `decode_fontname`. Without this, dump-mode
        // \lxDeclare doesn't add the alternate codepoint pattern (e.g.
        // `*` → `∗`) and overrides on \ast etc. silently fail.
        if encoding_opt.is_none() {
          let shared_key = with_value(
            &format!("font_shared_key_{}", ftok.with_str(ToString::to_string)),
            |v| match v {
              Some(Stored::String(s)) => with(*s, |str| Some(str.to_string())),
              _ => None,
            },
          );
          if let Some(sk) = shared_key
            && let Some(name) = sk.strip_prefix("fontinfo_")
          {
            let props = font::decode_fontname(name, None, None);
            if let Some(props) = props {
              encoding_opt = props.encoding.as_ref().map(|s| s.to_string());
            }
          }
        }
        if let Some(encoding) = encoding_opt {
          let decoded = font::decode(decoded_pos, Some(encoding), false);
          if let Some(dc) = decoded {
            let ds = dc.to_string();
            if ds != body_text {
              // Same 6-field shape (empty decl_id, trailing match_font)
              // so apply_lx_declarations parses it uniformly.
              decls.push(format!(
                "{}\t{}\t{}\t{}\t\t{}",
                ds,
                role,
                name_val,
                meaning,
                match_font.unwrap_or("")
              ));
            }
          }
        }
      }
    }
  }
  assign_value(
    key,
    Stored::String(pin(decls.join("\n"))),
    Some(Scope::Global),
  );
  Ok(())
}

/// The shared `<ltx:declare>` element constructor — Perl `\lxDeclare`
/// L474-485 / `\@lxDefMathDeclare` L387-396: `<tags><tag role="term">…</tag>
/// <tag role="short">…</tag></tags>` then `<text>description</text>`, all
/// from DIGESTED boxes (a `$x$: …` description term renders as real Math
/// and is itself subject to the declaration rewrites).
pub(super) fn emit_declare_element(
  document: &mut Document,
  decl_id: &str,
  term_boxes: Option<Vec<Digested>>,
  short_boxes: Option<Vec<Digested>>,
  desc_boxes: Option<Vec<Digested>>,
) -> Result<()> {
  // Perl: floatToElement('ltx:declare') positions at a container that accepts <declare>
  let saved = document.float_to_element("ltx:declare", false)?;
  let mut attrs_map = HashMap::default();
  attrs_map.insert("xml:id".to_string(), decl_id.to_string());
  let _decl_node = document.open_element("ltx:declare", Some(attrs_map), None)?;
  if term_boxes.is_some() || short_boxes.is_some() {
    document.open_element("ltx:tags", None, None)?;
    if let Some(term) = term_boxes {
      let mut tag_attrs = HashMap::default();
      tag_attrs.insert("role".to_string(), "term".to_string());
      document.open_element("ltx:tag", Some(tag_attrs), None)?;
      for b in &term {
        document.absorb(b, None)?;
      }
      document.close_element("ltx:tag")?;
    }
    if let Some(short) = short_boxes {
      let mut tag_attrs = HashMap::default();
      tag_attrs.insert("role".to_string(), "short".to_string());
      document.open_element("ltx:tag", Some(tag_attrs), None)?;
      for b in &short {
        document.absorb(b, None)?;
      }
      document.close_element("ltx:tag")?;
    }
    document.close_element("ltx:tags")?;
  }
  if let Some(desc) = desc_boxes {
    let _text_node = document.open_element("ltx:text", None, None)?;
    for b in &desc {
      document.absorb(b, None)?;
    }
    document.close_element("ltx:text")?;
  }
  document.close_element("ltx:declare")?;
  if let Some(ref save) = saved {
    document.set_node(save);
  }
  Ok(())
}

/// Perl `getDeclarationScope`: resolve `scope=section` to the current
/// section's `id:` scope. Uses the decl_id prefix (e.g. `S1` from
/// `S1.XMD1`) since it was computed in afterDigest where `\thesection@ID`
/// is correct — in afterConstruct it may be stale; falls back to walking
/// the document node's ancestor section.
pub(super) fn get_declaration_scope(
  document: &Document,
  scope_opt: &str,
  decl_id: &str,
) -> Option<Scope> {
  if scope_opt != "section" {
    return None;
  }
  let section_id = if !decl_id.is_empty() {
    decl_id.split('.').next().unwrap_or("").to_string()
  } else {
    // Fallback: use the node's ancestor section id
    let mut node = document.get_node().clone();
    let mut sid = String::new();
    loop {
      if node.get_name() == "section" {
        if let Some(id) = node
          .get_property("xml:id")
          .or_else(|| node.get_property("id"))
        {
          sid = id;
        }
        break;
      }
      match node.get_parent() {
        Some(p) => node = p,
        None => break,
      }
    }
    sid
  };
  if !section_id.is_empty() {
    Some(Scope::Named(pin(format!("id:{section_id}"))))
  } else {
    None
  }
}

/// Perl `createDeclarationRewrite`: build the rewrite rule from whatever
/// attributes exist — role/name/meaning AND decl_id alike (so a tag-only
/// declaration still marks its matches) — compile the pattern, and UNSHIFT
/// the rule in front of the others (later declarations preempt earlier ones
/// via `_matched`). `replace` and `attributes` are mutually exclusive: a
/// `replace=` declaration digests its replacement at rewrite time
/// (Core/Rewrite.pm `compile_replacement`) instead of marking attributes.
#[allow(clippy::too_many_arguments)]
pub(super) fn create_declaration_rewrite(
  scope: Option<Scope>,
  role: String,
  name_val: String,
  meaning: String,
  decl_id: String,
  body_text: &str,
  nowrap: bool,
  replace_tokens: Option<Tokens>,
) {
  use latexml_core::rewrite::{Rewrite, RewriteOptions};
  use rustc_hash::FxHashMap;
  let mut attrs = FxHashMap::default();
  if !role.is_empty() {
    attrs.insert("role".to_string(), role);
  }
  if !name_val.is_empty() {
    attrs.insert("name".to_string(), name_val);
  }
  if !meaning.is_empty() {
    attrs.insert("meaning".to_string(), meaning);
  }
  if !decl_id.is_empty() {
    attrs.insert("decl_id".to_string(), decl_id);
  }
  // Perl createDeclarationRewrite: ($nowrap ? (_nowrap => $nowrap) : ()) —
  // read by set_attributes_wild; underscore-prefixed so it is never
  // serialized onto the document.
  if nowrap {
    attrs.insert("_nowrap".to_string(), "1".to_string());
  }
  // Compile pattern: determine XPath, type, filters, wildcard paths.
  // Font-awareness is applied Rust-side (declare_node_matches for the
  // rewrite path, apply_lx_declarations for the post-rewrite fast path) —
  // NOT baked into the XPath, since the serialized `@font` attribute isn't
  // finalized until after math parsing (see compile_declare_pattern).
  let pat = if body_text.contains('_') || body_text.contains('\\') || body_text.contains('\'') {
    compile_declare_pattern(body_text)
  } else {
    // Simple single-token pattern: match XMTok by text; the Simple
    // filter in declare_node_matches rejects non-matching fonts.
    DeclarePattern {
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
    }
  };
  if pat.xpath.is_empty() {
    // Unrecognized pattern: make the compile failure VISIBLE — a silent
    // skip is indistinguishable from a legitimate no-match, the exact
    // precondition of the historical "wildcard XMDuals vanished" mode
    // (PR_READINESS cluster C).
    Warn!(
      "unexpected",
      "lxDeclare",
      "\\lxDeclare pattern not recognized by the rewrite compiler; declaration will not match",
      format!("pattern: '{body_text}'")
    );
    return;
  }
  // Pattern types determine select_count (see DeclarePattern::select_count):
  // Subscript/prime patterns match base XMTok + POSTSUBSCRIPT/POSTSUPERSCRIPT
  // sibling (select_count=2, pre-parsed DOM). Accents match the single XMApp.
  let select_count = pat.select_count();
  let rewrite = if let Some(replace_tks) = replace_tokens {
    use std::rc::Rc;

    use latexml_core::rewrite::RewriteReplaceClosure;
    let closure: RewriteReplaceClosure = Rc::new(move |document, _nodes| {
      // Perl Core/Rewrite.pm::compile_replacement (Tokens branch, as
      // fixed by upstream b17cc621): digest the pattern in
      // restricted_horizontal mode with a neutral font; for a math
      // rule, unwrap the outer List (Rust's digest() always wraps its
      // result in a List — the same shape Perl's changed autosimplify
      // now produces), take its single body, then getBody and absorb.
      begin_mode("restricted_horizontal")?;
      neutralize_font();
      let mut rbox = digest(replace_tks.clone())?;
      end_mode("restricted_horizontal")?;
      let unwrapped = if let DigestedData::List(l) = rbox.data() {
        let items = l.borrow().unlist();
        if items.len() == 1 {
          Some(items[0].clone())
        } else {
          None
        }
      } else {
        None
      };
      if let Some(u) = unwrapped {
        rbox = u;
      }
      if let Some(body) = rbox.get_body()? {
        rbox = body;
      }
      document.absorb(&rbox, None)?;
      Ok(())
    });
    Rewrite::new("math", RewriteOptions {
      xpath: Some(pat.xpath.clone()),
      replace: Some(closure),
      wildcard_paths: pat.wildcard_paths.clone(),
      select_count,
      scope,
      // Replace rules need the SAME declare-side filtering as attribute
      // rules — without it a `$x_\WildCard$` replace pattern deletes
      // the matched x plus an ARBITRARY next sibling even with no
      // subscript present (PR_READINESS cluster C).
      declare_filter: Some(pat),
      ..RewriteOptions::default()
    })
  } else {
    Rewrite::new("math", RewriteOptions {
      xpath: Some(pat.xpath.clone()),
      attributes_map: Some(attrs),
      wildcard_paths: pat.wildcard_paths.clone(),
      select_count,
      scope,
      declare_filter: Some(pat),
      ..RewriteOptions::default()
    })
  };
  // Perl: "Put this rule IN FRONT of other rules!" — UnshiftValue.
  unshift_value("DOCUMENT_REWRITE_RULES", vec![rewrite]);
}
