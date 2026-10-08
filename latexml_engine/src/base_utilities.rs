//! Base Utilities — Perl: Base_Utility.pool.ltxml
//!
//! Core TeX Implementation for LaTeXML.
//! Also contains shared Rust helper functions (Perl: LaTeXML::Package.pm utilities).

use std::char::{REPLACEMENT_CHARACTER, decode_utf16};

use latexml_core::{
  common::{
    arena::SymHashMap,
    cleaners::clean_label,
    xml::{content_nodes, element_nodes},
  },
  document::tag::{RawFrontmatter, TagAttrs, TagContent, TagData},
};
use libxml::tree::NodeType;
use rustc_hash::FxHashSet as HashSet;
const FRONTMATTER_ELEMENTS: &[&str] = &[
  "ltx:title",
  "ltx:toctitle",
  "ltx:subtitle",
  "ltx:creator",
  "ltx:date",
  "ltx:abstract",
  "ltx:keywords",
  "ltx:classification",
  "ltx:acknowledgements",
];

/// Frontmatter tags that are "replaceable" — only one per document; a later entry
/// replaces the earlier ones instead of stacking. Ported from a later upstream
/// LaTeXML (`Base_Utility.pool.ltxml` `%ReplaceableFrontmatterTags` +
/// `\@add@frontmatter@now`), which the vendored copy predates — the vendored
/// `\lx@add@frontmatter@{now,until}` push unconditionally, so a title/abstract emitted
/// twice keeps BOTH, producing duplicated frontmatter. Surpass over the vendored Perl
/// (a forward-port of the upstream fix). OXIDIZED_DESIGN #154. Witnesses: arXiv
/// 2002.09766 (appendix `\twocolumn[\icmltitle{…}]` re-adds `ltx:title`), 2511.21969
/// (nested `{abstract}` env). Creators/notes are deliberately NOT here — multi-author
/// frontmatter must accumulate.
const REPLACEABLE_FRONTMATTER_TAGS: &[&str] = &[
  "ltx:title",
  "ltx:toctitle",
  "ltx:subtitle",
  "ltx:date",
  "ltx:abstract",
  "ltx:keywords",
];
use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // LaTeX has a very particular, but useful, notion of "Undefined",
  //    so let's get that squared away at the outset; it's useful for TeX, too!
  //
  // Naturally, it uses \csname to check, which ends up DEFINING the possibly undefined macro as
  // \relax
  // Perl Base_Utility.pool.ltxml L23-31
  DefMacro!("\\lx@ifundefined{}{}{}", sub[(name, if_token, else_token)] {
    let cs = T_CS!(s!("\\{}", Expand!(name).to_string()));
    // Autoload triggers (declared via `def_autoload` at tex.rs:238-247)
    // install a closure under the trigger CS so the package auto-loads on
    // first invocation. Perl scopes the equivalent `DefAutoload` entries
    // to `OmniBus.cls.ltxml`, so for non-OmniBus papers Perl sees these
    // CSes as truly undefined. Mirror Perl by treating an unfired
    // autoload trigger as "undefined" in `\@ifundefined`. We must NOT
    // overwrite the trigger CS with `\relax` in this case (the kernel
    // `\csname X\endcsname \ifx \relax` idiom DOES overwrite, but doing
    // so here would destroy the autoload — subsequent use of the trigger
    // CS would no-op instead of loading its package). Driver:
    // arXiv:2507.23241v1 (smfart.cls) — line 373's
    // `\@ifundefined{numberwithin}` branches wrong when our preloaded
    // `\numberwithin` autoload makes the CS "look" defined, then the
    // `\@gobbletwo` branch eats `\ifx \relax` and orphans `\else` / `\fi`.
    // An autoload TRIGGER counts as "undefined" only while UNFIRED — i.e. its
    // target package has not yet loaded. `def_autoload` stores the package name
    // (`.sty`) as a String; once `<pkg>.sty_loaded`/`_raw_loaded` is set the
    // trigger CS holds the package's real definition and must read as DEFINED
    // (fixes `\@ifundefined{align}` after `\usepackage{amsmath}`). `.pool`
    // triggers keep the legacy Bool form and stay "undefined until used".
    let is_autoload = cs.with_cs_name(|cs_name| {
      let flagged = match lookup_value(&s!("{cs_name}:autoload")) {
        Some(Stored::String(pkg_sym)) => {
          let pkg = with(pkg_sym, |s| s.to_string());
          !lookup_bool(&s!("{pkg}.sty_loaded"))
            && !lookup_bool(&s!("{pkg}.sty_raw_loaded"))
        },
        Some(Stored::Bool(b)) => b,
        _ => false,
      };
      // The flag is only meaningful while the CS still HOLDS the trigger
      // definition. The kernel dump redefines some trigger CSes itself
      // (`\ProvidesExplPackage` et al. are real latex.ltx macros) — then
      // the CS is genuinely defined regardless of the package flag. Compare
      // definition identity against the snapshot def_autoload stored.
      flagged
        && match lookup_value(&s!("{cs_name}:autoload_trigger")) {
          Some(Stored::Expandable(trigger)) => with_meaning(&cs, |m| {
            matches!(m, Some(Stored::Expandable(current)) if Rc::ptr_eq(current, &trigger))
          }),
          None => true, // legacy trigger without snapshot: old behavior
          _ => false,
        }
    });
    if IsDefined!(&cs) && !is_autoload {
      Ok(else_token)
    } else {
      // No `\relax` pollution of the probed name. latex.ltx:1729-1737 defines
      // `\@ifundefined` with `\ifcsname` precisely so a probe leaves the name
      // undefined (the `\csname…\endcsname\relax` form at :1738 is only the
      // pre-`\ifcsname` fallback, and Perl Base_Utility.pool:23-31 still
      // pollutes). polyglossia's gloss-*.ldf:591 `\@ifundefined
      // {initiate@active@char}{\input{babelsh.def}}{}` then hands babelsh.def:1
      // `\ifx\initiate@active@char\@undefined\else\bbl@afterfi\endinput\fi` a
      // `\relax`-valued name, so the shorthand file early-exits with
      // `\bbl@afterfi` undefined (hang, sample; 19 gloss files load it this
      // way). Guard: `perfect_kernel_batch54::ifundefined_does_not_define_the_name`.
      Ok(if_token)
    }
  }, locked=>true);
  // \@ifundefined is a LaTeX-kernel macro, but our amsppt / amssymb
  // bindings invoke it from Plain-TeX/AMSTeX context too — and those
  // contexts don't load latex_constructs (which has the same Let).
  // Surfacing it here makes it available regardless of which constructs
  // pool is loaded. Surpasses Perl's Base_Utility.pool.ltxml gap.
  Let!("\\@ifundefined", "\\lx@ifundefined");

  // Dash and space primitives used by ligatures and other mechanisms.
  // Perl Base_Utility.pool.ltxml L44-45
  DefPrimitive!("\\lx@endash", {
    Tbox::new(
      pin!("\u{2013}"),
      None,
      None,
      Tokens!(T_CS!("\\lx@endash")),
      SymHashMap::default(),
    )
  });
  // Perl Base_Utility.pool.ltxml L46-47
  DefPrimitive!("\\lx@emdash", {
    Tbox::new(
      pin!("\u{2014}"),
      None,
      None,
      Tokens!(T_CS!("\\lx@emdash")),
      SymHashMap::default(),
    )
  });
  // Perl Base_Utility.pool.ltxml L50-52: stand-in for T_ACTIVE('~').
  DefPrimitive!("\\lx@NBSP", {
    Tbox::new(
      pin!("\u{00A0}"),
      None,
      None,
      Tokens!(T_ACTIVE!('~')),
      // `~` is a penalty and glue (latex.ltx:9411-9419): `\unskip` removes the glue.
      stored_map!("isSpace" => true, "isSkip" => true, "width" => Dimension::from_str("0.333em")?),
    )
  }, locked => true);
  // Perl Base_Utility.pool.ltxml L53-55
  DefPrimitive!("\\lx@nobreakspace", {
    Tbox::new(
      pin!("\u{00A0}"),
      None,
      None,
      Tokens!(T_CS!("\\lx@nobreakspace")),
      // `~` is a penalty and glue (latex.ltx:9411-9419): `\unskip` removes the glue.
      stored_map!("isSpace" => true, "isSkip" => true, "width" => Dimension::from_str("0.333em")?),
    )
  });

  // Perl Base_Utility.pool.ltxml L57-65
  DefPrimitive!("\\lx@ignorehardspaces", {
    let mut boxes = Vec::new();
    while let Some(token) = read_x_token(None, false, None)? {
      boxes = invoke_token(&token)?;
      if boxes.is_empty() {
        break;
      }
      while !boxes.is_empty() {
        if match boxes[0].get_property("isSpace") {
          Some(Cow::Borrowed(Stored::Bool(space_bool))) => *space_bool,
          Some(Cow::Owned(Stored::Bool(ref space_bool))) => *space_bool,
          _ => false,
        } {
          boxes = boxes[1..].to_vec();
        } else {
          break;
        }
      }
      if !boxes.is_empty() {
        break;
      }
    }
    Ok(boxes)
  });

  // Perl Base_Utility.pool.ltxml L42 (PR #2767)
  DefMacro!("\\lx@strip@braces{}", sub[(arg)] {
    Ok(arg.strip_braces())
  });

  // Perl Base_Utility.pool.ltxml L85-87 (renamed from \@ADDCLASS in PR #2767)
  DefConstructor!("\\lx@add@cssclass Semiverbatim", sub[document,args] {
      document.add_class(&mut document.get_element().unwrap(),
        &args[0].as_ref().unwrap().to_string())?;
    }, sizer => 0);

  // Perl Base_Utility.pool.ltxml L101-103 (PR #2767)
  DefConstructor!("\\lx@set@attribute Semiverbatim {}", sub[document,args] {
      let key = args[0].as_ref().map(ToString::to_string).unwrap_or_default();
      let value = args[1].as_ref().map(ToString::to_string).unwrap_or_default();
      if let Some(mut element) = document.get_element() {
        document.set_attribute(&mut element, &key, &value)?;
      }
    }, sizer => 0);

  // Perl Base_Utility.pool.ltxml (PR #2767): split #3 on the delimiter
  // tokens in #2, invoking #1 on each piece.
  // DefMacro('\lx@splitting{}{}{}', ...)
  DefMacro!("\\lx@splitting{}{}{}", sub[(op, delimiters, tokens)] {
    // Note: Perl tests `$delimiters ?` — a Tokens object is always truthy,
    // so the split always applies (an empty delimiter list yields one piece).
    let delims: Vec<SplitDelim> = delimiters.unlist().into_iter().map(SplitDelim::from).collect();
    let mut result: Vec<Token> = Vec::new();
    for piece in split_tokens(tokens, delims) {
      result.extend(op.unlist_ref().iter().copied());
      result.push(T_BEGIN!());
      result.extend(piece.unlist());
      result.push(T_END!());
    }
    Ok(Tokens::new(result))
  });

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // General support for Front Matter. (PR #2767 rework)
  // Not (yet) used by TeX (finish plain?)
  // But provides support for LaTeX (and other formats?) for handling frontmatter.
  //
  // The idea is to accumulate any frontmatter material (title, author,...)
  // rather than directly drop it into the digested stream.
  // When we begin constructing the document, all accumulated material is output.
  // See LaTeX.ltxml for usage.
  // Note: could be circumstances where you'd want modular frontmatter?
  // (ie. frontmatter for each sectional unit)

  // Perl: DebuggableFeature('frontmatter'); enable with `--debug frontmatter`.
  debuggable_feature("frontmatter");

  // Perl Base_Utility.pool.ltxml L219-222 (PR #2767): moved here from
  // latex_constructs (was \@personname). Perl's beforeDigest rebinds
  // `\thanks` → `\lx@add@thanks` so an author's `\thanks{...}` becomes a
  // role=thanks contact via \lx@annotate@frontmatter@now (which applies the
  // `\lx@contact@thanks@name` default, "Thanks: "). The earlier Rust port
  // mis-bound it to the now-removed `\person@thanks` constructor, which built
  // a bare <contact role=thanks> with no name; faithful binding restored.
  DefConstructor!("\\lx@personname{}", "<ltx:personname>#1</ltx:personname>",
    before_digest => { Let!("\\thanks", "\\lx@add@thanks"); },
    bounded => true,
    mode => "text",
    enter_horizontal => true
  );
  DefMacro!("\\lx@ignore@tabular[]{}", "");
  DefMacro!("\\lx@ignore@endtabular", "");

  // Sanitize person names for (obvious) punctuation abuse at start+end
  // (moved here from latex_constructs per PR #2767; the existing Rust
  // punctuation-strip port is carried over).
  Tag!("ltx:personname", after_close => sub[document, node] {
    // Normalize a whole-name bold BEFORE the punctuation strip, so the unwrapped
    // text is punctuation-cleaned like a plain name (html_feedback#61).
    unwrap_whole_name_bold(document, node)?;
    if let Some(mut first) = node.get_first_child() {
      if first.get_type() == Some(NodeType::TextNode) {
        let first_text = first.get_content();
        let mut first_text_iter = first_text.chars().peekable();
        while let Some(peeked) = first_text_iter.peek() {
          if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
            first_text_iter.next();
          } else {
            break;
          }
        }
        let new_text = first_text_iter.collect::<String>();
        if first_text != new_text {
          first.set_content(&new_text)?;
        }
      }
      if let Some(mut last) = node.get_last_child()
        && last.get_type() == Some(NodeType::TextNode) {
          let last_text = last.get_content();
          let mut last_text_iter  = last_text.chars().rev().peekable();
          while let Some(peeked) = last_text_iter.peek() {
            if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
              last_text_iter.next();
            } else {
              break;
            }
          }
          let mut new_text = last_text_iter.rev().collect::<String>();
          // Beyond Perl (KNOWN_PERL_ERRORS #523): the period of an initial or of a suffix written with one is the
          // name's (`Sridhar K.`, hep-ph9306209; `John Smith Jr.`)
          if last_text[new_text.len()..].starts_with('.') && ends_with_abbreviation(&new_text) {
            new_text.push('.');
          }
          if last_text != new_text {
            last.set_content(&new_text)?;
          }
        }
    }
  });

  //======================================================================
  // Perl Base_Utility.pool.ltxml L161
  AssignValue!(
    "frontmatter",
    Stored::HashTagData(HashMap::default()),
    Some(Scope::Global)
  );

  // Perl Base_Utility.pool.ltxml L163
  DefConditional!("\\if@in@preamble", { lookup_bool_sym(pin!("inPreamble")) });

  DefKeyVal!("Frontmatter", "role", "Semiverbatim");
  DefKeyVal!("Frontmatter", "class", "Semiverbatim");
  DefKeyVal!("Frontmatter", "graphic", "Semiverbatim");
  DefKeyVal!("Frontmatter", "annotations", "");
  DefKeyVal!("Frontmatter", "label", "");
  DefKeyVal!("Frontmatter", "labelref", "Semiverbatim");
  DefKeyVal!("Frontmatter", "labelseq", "");
  DefKeyVal!("Frontmatter", "annotate", "");
  // Rust-only: an entry of a replaceable tag that adds to the ones before instead of replacing them (acmart's
  // `\received` history, acmart.cls:1859-1872: several revised dates; 2307.05988), honoured by
  // `\lx@add@frontmatter@now` and `@until` ([`takes_accumulate`])
  DefKeyVal!("Frontmatter", "accumulate", "");
  DefKeyVal!("Frontmatter", "name", "");

  // \lx@clear@frontmatter{tag}[kv]
  // Remove all pending frontmatter element matching $tag, and role (if given) in keyvals
  DefPrimitive!("\\lx@clear@frontmatter {} OptionalKeyVals:Frontmatter", sub[(tag, kv)] {
    let role = kv.as_ref()
      .and_then(|kv| kv.get_value("role"))
      .map(|v| v.to_string())
      .filter(|r| !r.is_empty());
    match role {
      Some(ref role) => dequeue_front_matter(&tag.to_string(), &[("role", role)]),
      None => dequeue_front_matter(&tag.to_string(), &[]),
    }
  });

  // Remove all creators with given role (default author)
  DefMacro!(
    "\\lx@clear@creators []",
    "\\lx@clear@frontmatter{ltx:creator}[#1]"
  );

  // The various \lx@add@<frontmatter> commands
  //  (1) queue the command (appending @now) in the frontmatter_raw state variable
  //    to defer digestion (& possible replacement)
  //  (2) when the @now form is digested (see digest_front_matter)
  //    will add an entry to the frontmatter hash state variable.
  //    That hash is keyed by the tag, with values contains a list of
  //      [tag, {attr}, @content]
  //    to create an element <tag> with the given attributes and content.
  //    Each content item is either a Box (List,Whatsit)
  //    or recursively an array [tag,{attr},@content].

  // See clean_trailing_break for cleanup of misused \\.
  Tag!("ltx:personname", after_close => sub[document, node] {
    clean_trailing_break(document, node)?;
  });
  Tag!("ltx:contact", after_close => sub[document, node] {
    clean_trailing_break(document, node)?;
  });

  // Add a new frontmatter item that will be enclosed in <$tag %attr>...</$tag>
  // The content is the result of digesting $tokens.
  // \lx@add@frontmatter[keys]{tag}[attributes]{content}
  // keys can have
  //   replace (to replace the current entry, if any)
  //   ifnew   (only add if no previous entry)
  DefPrimitive!("\\lx@add@frontmatter OptionalKeyVals {} OptionalKeyVals {}",
    sub[(keys_opt,tag_tks,attrs_opt,tokens)] {
    // Beyond-Perl hardening (OXIDIZED_DESIGN #51; general frontmatter principle,
    // user-directed 2026-07-07): a frontmatter entry with an empty tag or empty
    // content is VOID — early-quit, emit nothing. Perl's `\lx@add@frontmatter`
    // queues unconditionally (L354-358), so a binding fed an empty argument
    // yields a stray empty element: e.g. icml's `\printAffiliationsAndNotice{}`
    // (empty braces = the ICML-sanctioned "no notice") -> an empty
    // `<ltx:note role="affiliationnotice">` rendering as a bare "affiliationnotice:"
    // marker (witness arXiv:2606.00309). The affiliation list itself survives —
    // it is fed separately via `\icmlaffiliation`.
    if tag_tks.to_string().trim().is_empty() || tokens.to_string().trim().is_empty() {
      return Ok(Vec::new());
    }
    queue_add_frontmatter_now(keys_opt.as_ref(), &tag_tks, attrs_opt.as_ref(), Some(&tokens))?;
  });

  // Beyond-Perl best-practice API (OXIDIZED_DESIGN #51): open an *empty*
  // frontmatter container element as a deliberate anchor for later annotations
  // — the intention-revealing counterpart to `\lx@add@frontmatter`, which is now
  // a no-op on empty content. A few bindings legitimately need an empty
  // container: moderncv's cv `<ltx:creator>` exists only to receive the
  // \firstname/\familyname/\email/... contacts that annotate the most-recent
  // creator. (Perl abuses `\lx@add@frontmatter{ltx:creator}[role=cv]{}` for
  // this; we name the intent instead of smuggling it through empty content.)
  // \lx@add@frontmatter@container[keys]{tag}[attributes]
  DefPrimitive!("\\lx@add@frontmatter@container OptionalKeyVals {} OptionalKeyVals",
    sub[(keys_opt,tag_tks,attrs_opt)] {
    queue_add_frontmatter_now(keys_opt.as_ref(), &tag_tks, attrs_opt.as_ref(), None)?;
  });

  DefPrimitive!("\\lx@add@frontmatter@now OptionalKeyVals {} OptionalKeyVals:Frontmatter {}",
    sub[(_obsoletekeys, tag_tks, kv, content)] {
    let tag = tag_tks.to_string();
    // %options = digested keyvals hash; role read from the UNdigested keyvals
    // (Careful! Multiple values — Perl getValue returns the last one.)
    let mut options = TagAttrs::default();
    let mut role = String::new();
    if let Some(kv) = kv {
      role = kv.get_value("role").map(|v| v.to_string()).unwrap_or_default();
      if let DigestedData::KeyVals(dkv) = kv.be_digested()?.data() {
        for key in dkv.get_keyvals().keys() {
          if let Some(v) = dkv.get_value_digested(key) {
            options.insert(key.clone(), v.to_string());
          }
        }
      }
    }
    let accumulate = takes_accumulate(&mut options);
    // extract (possibly multiple!) labels
    let mut labels = clean_frontmatter_labels(
      options.get("annotations").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = lookup_mapping_int(&s!("num_{tag}"), &role) + 1;
      assign_mapping(&s!("num_{tag}"), &role, Some(Stored::Int(n)));
      options.insert("role".to_string(), role.clone());
      options.insert("_num".to_string(), n.to_string());
      // record sequence position as potential attachment label
      labels.push(clean_label(&n.to_string(), Some(&role)).into_owned());
    }
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    options.insert("_annotations".to_string(), labels.join(","));
    let entry = TagData {
      tag: tag.clone(),
      attr: options,
      content: vec![TagContent::PlaceKeeper], // (in case embedded)
    };
    DebugFeature!("frontmatter", "FRONT Add {}\n   for: {}",
      show_frontmatter(&entry), content);
    // Replaceable tags (title/toctitle/subtitle/date) keep only one entry — a later
    // one replaces earlier ones. Ported from upstream `%ReplaceableFrontmatterTags`
    // (the vendored copy pushes unconditionally → duplicate <title> when a document
    // re-adds it, e.g. arXiv 2002.09766's appendix `\icmltitle`). OXIDIZED_DESIGN #154.
    // An `accumulate` entry adds to the ones before (acmart's `\received` history, 2307.05988).
    if REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate {
      frontmatter_clear_same_name(&tag, entry.attr.get("name").map(String::as_str));
    }
    let index = frontmatter_push(&tag, entry);
    // REPLACE only 'place_keeper'!!
    let digested = digest_frontmatter_item(&tag, content)?;
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
  }, bounded => true);

  // This is a variant of \lx@add@frontmatter which digests immediately
  // until a terminator token;
  // It is useful for frontmatter environments, like {abstract}
  // (expanding until \end{abstract} generally gets tangled by contents which expect
  // digestion and side effects).
  DefPrimitive!("\\lx@add@frontmatter@until {} OptionalKeyVals:Frontmatter DefToken",
    sub[(tag_tks, kv, end)] {
    let tag = tag_tks.to_string();
    let mut options = TagAttrs::default();
    let mut role = String::new();
    if let Some(kv) = kv {
      role = kv.get_value("role").map(|v| v.to_string()).unwrap_or_default();
      if let DigestedData::KeyVals(dkv) = kv.be_digested()?.data() {
        for key in dkv.get_keyvals().keys() {
          if let Some(v) = dkv.get_value_digested(key) {
            options.insert(key.clone(), v.to_string());
          }
        }
      }
    }
    let accumulate = takes_accumulate(&mut options);
    // extract (possibly multiple!) labels
    let mut labels = clean_frontmatter_labels(
      options.get("annotations").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = lookup_mapping_int(&s!("num_{tag}"), &role) + 1;
      assign_mapping(&s!("num_{tag}"), &role, Some(Stored::Int(n)));
      options.insert("role".to_string(), role.clone());
      options.insert("_num".to_string(), n.to_string());
      // record sequence position as potential attachment label
      labels.push(clean_label(&n.to_string(), Some(&role)).into_owned());
    }
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    options.insert("_annotations".to_string(), labels.join(","));
    let entry = TagData {
      tag: tag.clone(),
      attr: options,
      content: vec![TagContent::PlaceKeeper], // (in case embedded)
    };
    // Replaceable tags (abstract/keywords) keep only one entry — but ONLY dedup when no
    // same-tag `@until` is already in progress (open PlaceKeeper), so a nested/malformed
    // `\begin{abstract}\begin{abstract}…` isn't corrupted by clearing a parent's still-
    // open entry. OXIDIZED_DESIGN #154. Witness 2511.21969 (nested abstract env).
    if REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str())
      && !accumulate
      && !frontmatter_has_open_placekeeper(&tag)
    {
      frontmatter_clear_same_name(&tag, entry.attr.get("name").map(String::as_str));
    }
    let index = frontmatter_push(&tag, entry);
    // A terminal reached INSIDE a nested group (`\section`'s
    // `\@startsection@hook` inside an unbalanced `\abstract{…`) ends the
    // body from there: `digest_next_body` records the terminal and depth on
    // this frame, `until_terminal_inside_group` (stomach.rs) acts.
    let body = digest_next_body(Some(end))?;
    let digested = Digested::from(List::new(body));
    DebugFeature!("frontmatter", "FRONT Add (until) {} for: {}", tag, digested);
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
  }, bounded => true);

  // Some frontmatter elements are "structured" in the sense of having a main bit of data
  // and several optional extra bits.  For example, LaTeX classes typically have markup
  // to define "creators" (authors, editors,etc) and a variety of markup strategies to
  // annotate them with "contacts" (affiliation, email, etc)
  // In the easy case, that markup is embedded within \author. Otherwise, it appears
  // separately and will be "attached" to the most recent creator,
  //
  // The \lx@annotate@frontmatter command is used to annotate some frontmatter elements,
  // with additional data. Several keywords support different attachment methods:
  //   label : $label; find the $parenttag with annotations containing $label.
  //   labelseq=$prefix : n-th $tag+$role attaches to $parenttag using label = prefix+n
  //   annotate=(all | new | <number> )
  //     all : attaches to all preceding $parenttag
  //     new : like all, but only those not yet having this type of annotation
  //     <number> : attaches to the <number>-th previous $parenttag
  //   <default> : attach to preceding $parenttag.

  // \lx@annotate@frontmatter{parenttag}{tag}[options]{content}
  // adds a tag element, containing content, to an appropriate parenttag,
  // according to the attachment criterion in options keyvals.
  DefPrimitive!("\\lx@annotate@frontmatter {} {} OptionalKeyVals:Frontmatter {}",
    sub[(parenttag, tag, kv, content)] {
    // Perl: queueFrontMatter($stomach, ToString($tag), $kv,
    //   Invocation(T_CS('\lx@annotate@frontmatter@now'), $parenttag, $tag, $kv, $content))
    let mut inv_tokens: Vec<Token> = vec![T_CS!("\\lx@annotate@frontmatter@now")];
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(parenttag.unlist_ref().iter().copied());
    inv_tokens.push(T_END!());
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(tag.unlist_ref().iter().copied());
    inv_tokens.push(T_END!());
    if let Some(ref kv) = kv {
      inv_tokens.push(T_OTHER!("["));
      inv_tokens.extend(kv.revert()?.unlist());
      inv_tokens.push(T_OTHER!("]"));
    }
    inv_tokens.push(T_BEGIN!());
    inv_tokens.extend(content.unlist());
    inv_tokens.push(T_END!());
    queue_front_matter(&tag.to_string(), kv.as_ref(), Tokens::new(inv_tokens));
  });

  DefPrimitive!("\\lx@annotate@frontmatter@now {}{} OptionalKeyVals:Frontmatter {}",
    sub[(parenttag_tks, tag_tks, kv, content)] {
    let parenttag = parenttag_tks.to_string();
    let tag = tag_tks.to_string();
    let preformatted = tag == "preformatted"; // Obsolete API? $content is constructor!
    let mut options = TagAttrs::default();
    if let Some(kv) = kv
      && let DigestedData::KeyVals(dkv) = kv.be_digested()?.data() {
        for key in dkv.get_keyvals().keys() {
          if let Some(v) = dkv.get_value_digested(key) {
            options.insert(key.clone(), v.to_string());
          }
        }
      }
    // Perl: $role = $options{role} = ToString($options{role}) — digested value here
    let role = options.get("role").cloned().unwrap_or_default();
    options.insert("role".to_string(), role.clone());
    let mut labels = clean_frontmatter_labels(
      options.get("label").map(String::as_str).unwrap_or(""), "");
    if !role.is_empty() {
      let n = lookup_mapping_int(&s!("num_{tag}"), &role) + 1;
      assign_mapping(&s!("num_{tag}"), &role, Some(Stored::Int(n)));
      if let Some(labelseq) = options.get("labelseq")
        && !labelseq.is_empty() {
          labels.push(clean_label(&n.to_string(), Some(labelseq)).into_owned());
        }
    }
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    options.insert("_label".to_string(), labels.join(","));

    // Snapshot the parent entries (those not role=pending), inherit labels
    // from an enclosing pending entry, and push a tentative stub entry —
    // in case digestion changes labels!
    let (parent_indices, stub_idx) = with_value_mut("frontmatter", |val_opt| {
      if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
        let list = frnt.entry(parenttag.clone()).or_insert_with(Vec::new);
        let parent_indices: Vec<usize> = list.iter().enumerate()
          .filter(|(_, e)| e.attr.get("role").map(String::as_str).unwrap_or("") != "pending")
          .map(|(i, _)| i)
          .collect();
        // IF this item encountered WITHIN another frontmatter
        // we inherit that frontmatter's labels (even override!)
        if let Some(last) = list.last()
          && last.attr.get("role").map(String::as_str) == Some("pending")
            && matches!(last.content.first(), Some(TagContent::PlaceKeeper))
          {
            let inherited = last.attr.get("_annotations").cloned().unwrap_or_default();
            options.insert("_label".to_string(), inherited);
          }
        let mut stub_attr = TagAttrs::default();
        stub_attr.insert("role".to_string(), "pending".to_string());
        stub_attr.insert("_annotations".to_string(),
          options.get("_label").cloned().unwrap_or_default());
        let stub = TagData {
          tag: parenttag.clone(),
          attr: stub_attr,
          content: vec![TagContent::PlaceKeeper],
        };
        DebugFeature!("frontmatter", "FRONT Add stub {}\n  for annotation {} [{}]",
          show_frontmatter(&stub), tag, options.get("_label").map(String::as_str).unwrap_or(""));
        list.push(stub);
        (parent_indices, list.len() - 1)
      } else {
        (Vec::new(), 0)
      }
    });
    let nparents = parent_indices.len();
    let xcontent = digest_frontmatter_item(&tag, content)?;
    // Reset if changed (eg. by \lx@set@frontmatter@label during digestion)!
    let stub_label = with_value("frontmatter", |v| {
      if let Some(Stored::HashTagData(frnt)) = v {
        frnt.get(&parenttag)
          .and_then(|l| l.get(stub_idx))
          .and_then(|e| e.attr.get("_annotations"))
          .cloned()
      } else {
        None
      }
    }).unwrap_or_default();
    options.insert("_label".to_string(), stub_label.clone());
    let datum = if preformatted {
      TagContent::Box(xcontent)
    } else {
      TagContent::Entry(TagData {
        tag: tag.clone(),
        attr: options.clone(),
        content: vec![TagContent::Box(xcontent)],
      })
    };
    if !preformatted && (!stub_label.is_empty() || nparents == 0) {
      // deferred until we can compare labels
      DebugFeature!("frontmatter", "... deferring to label={stub_label}");
      frontmatter_set_first_content(&parenttag, stub_idx, datum);
    } else {
      let annotate = options.get("annotate").cloned().unwrap_or_default();
      let (mut nprev, newonly): (i64, bool) = if annotate.is_empty() {
        (1, false)
      } else if annotate.chars().all(|c| c.is_ascii_digit()) {
        (annotate.parse().unwrap_or(1), false)
      } else if annotate == "all" {
        (nparents as i64, false)
      } else if annotate == "new" {
        (nparents as i64, true)
      } else {
        Info!("unexpected", &tag, s!("Frontmatter annotate '{annotate}' unrecognized"));
        (1, false)
      };
      DebugFeature!("frontmatter", "...adding to {nprev} previous{}",
        if newonly { " new" } else { "" });
      with_value_mut("frontmatter", |val_opt| {
        if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
          && let Some(list) = frnt.get_mut(&parenttag) {
            // Remove unneeded (stub) entry.
            if list.get(stub_idx)
              .map(|e| e.attr.get("role").map(String::as_str) == Some("pending"))
              .unwrap_or(false)
            {
              list.remove(stub_idx);
            }
            let has_role_key = s!("_has{role}");
            let mut indices = parent_indices.clone();
            while let Some(pi) = indices.pop() {
              if nprev <= 0 {
                break;
              }
              nprev -= 1;
              if let Some(parent) = list.get_mut(pi) {
                parent.content.push(datum.clone());
                if !role.is_empty() {
                  parent.attr.insert(has_role_key.clone(), "1".to_string());
                  if newonly
                    && let Some(&next_pi) = indices.last()
                      && list.get(next_pi)
                        .map(|p| p.attr.contains_key(&has_role_key))
                        .unwrap_or(false)
                      {
                        break;
                      }
                }
              }
            }
          }
      });
    }
  });

  // These next two are primitives executing during digestion (BEFORE XML);
  // they add to or replace labels to be used when building the frontmatter plan.

  // \lx@request@frontmatter@annotation adds additional (comma separated) labels to the
  // currently being digesting frontmatter;
  // Typically would \let\inst to this, and used within creator's content
  // to identify contacts to include.
  // Can provide the prefix to distinguish different sets of labels
  DefPrimitive!("\\lx@request@frontmatter@annotation[]{}", sub[(prefix, label)] {
    let prefix = prefix.as_ref().map(ToString::to_string).unwrap_or_default();
    let label = clean_frontmatter_labels(
      &label.to_string(),
      if prefix.is_empty() { "LABEL" } else { &prefix }).join(",");
    with_pending_entry_attr(move |attr| {
      let labels = attr.get("_annotations").cloned().unwrap_or_default();
      let newval = if labels.is_empty() { label.clone() } else { s!("{labels},{label}") };
      DebugFeature!("frontmatter", "FRONT add annotation label {label}");
      attr.insert("_annotations".to_string(), newval);
    });
  });

  // \lx@set@frontmatter@label Internal to digesting annotation contents.
  // it sets (replaces) the _annotation labels on the currently being digesting
  // frontmatter so that the annotation inherits it as label (!!)
  // Typically would \let\label to this so that a \ref within a parent frontmatter
  // will get an annotation with corresponding \label will be attached.
  DefPrimitive!("\\lx@set@frontmatter@label [] Semiverbatim", sub[(prefix, label)] {
    // Optional prefix (default "LABEL"). The superscript heuristic passes the
    // role ("affiliation") so the label exactly matches the corresponding
    // \lx@request@frontmatter@annotation[affiliation] on the author; otherwise
    // relocate_annotations falls back to prefix-stripped matching and conflates
    // it with the creator's own "author:N" sequence label (double-attach).
    let prefix = prefix.as_ref().map(ToString::to_string)
      .filter(|p| !p.is_empty()).unwrap_or_else(|| "LABEL".to_string());
    let label = clean_frontmatter_labels(&label.to_string(), &prefix).into_iter().next();
    with_pending_entry_attr(move |attr| {
      match label {
        Some(label) => {
          DebugFeature!("frontmatter", "FRONT set label {label}");
          attr.insert("_annotations".to_string(), label);
        },
        None => {
          attr.remove("_annotations");
        },
      }
    });
  });

  //======================================================================
  // Some shorthands?

  DefMacro!(
    "\\lx@add@title[]{}",
    "\\lx@clear@frontmatter{ltx:title}\\lx@add@frontmatter{ltx:title}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@toctitle[]{}",
    "\\lx@clear@frontmatter{ltx:toctitle}\\lx@add@frontmatter{ltx:toctitle}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@subtitle[]{}",
    "\\lx@clear@frontmatter{ltx:subtitle}\\lx@add@frontmatter{ltx:subtitle}[#1]{#2}"
  );

  // careful: the "name", #2, can contain much more than just the name!
  DefMacro!(
    "\\lx@add@creator [] {}",
    "\\lx@add@frontmatter{ltx:creator}[role=author,#1]{\\lx@personname{\\lx@author@withinst{#2}}}"
  );
  DefMacro!(
    "\\lx@add@author[]{}",
    "\\lx@add@frontmatter{ltx:creator}[role=author,#1]{\\lx@personname{\\lx@author@withinst{#2}}}"
  );
  // Author content is digested at `\author` time, before a class's title
  // box can define `\inst` (bfhsciposter.cls:445,476 sets it to a
  // superscript inside its title box; beamerbasetitle.sty:262
  // `\providecommand\inst[1]{}`), so `\inst` is provided for the author
  // content ONLY — the shape of Perl's `\lx@author@withsup`
  // (Base_Utility.pool.ltxml:729) — and with its frontmatter MEANING: an
  // affiliation-link request (llncs.cls.ltxml:50
  // `\lx@request@frontmatter@annotation[affiliation]`), never a typeset
  // superscript — except a footnote-SYMBOL mark (`\inst{*}`, `\inst{\dagger}`:
  // an equal-contribution note, not an affiliation number), which keeps its
  // glyph exactly as `rewrite_symbol_superscripts` keeps `$^{*}$` (OXIDIZED
  // #52), so the poster's `*` stays visible as pdflatex renders it.
  // `\providecommand`, so a class binding's own `\inst` wins; scoped, so a
  // class's later `\newcommand\inst` (ptptex.cls:616) is not blocked.
  // SURPASS, KNOWN_PERL_ERRORS #201 (Perl: `undefined:\inst` on the poster
  // witness, whose class defines `\inst` only inside its title box).
  // `\def`, not `\providecommand`: inside author content `\inst{n}` ALWAYS
  // means "link to affiliation n" (llncs' own `\inst` is this same request),
  // and K11 may have rerouted a raw class's document-level `\inst` (the
  // affiliation LIST setter, ptptex/jpsj2) before any author is read. A
  // save/restore pair, not a group: author content may carry a `\\` line
  // break (keyval2e-guide `\author{Name \Email{…}\\Preston, …}`), and an
  // extra `\bgroup…\egroup` around it adds one more mode-check error per
  // author.
  DefMacro!(
    "\\lx@author@withinst{}",
    "\\let\\lx@saved@inst\\inst\\def\\inst##1{\\lx@inst@mark{##1}}\\lx@author@markup@begin\\expandafter\\lx@author@markup@run\\expandafter{\\lx@author@markup@unprovide}{#1}\\let\\inst\\lx@saved@inst"
  );
  // A raw class's own author markup — `\fnms`/`\snm`/`\inits`/`\degs`/`\roles`, its `\orcid` — is often defined only
  // inside the class's `\author` (IOS-Book-Article.cls:1146-1184, econsocart.cls:2095-2160), which the locked kernel
  // `\author` replaces, so the author content met it undefined (IOS: 2407.04130, 2609.06231, 13776, 15113, 28673;
  // econsocart: 2609.06865; OXIDIZED_DESIGN_DIVERGENCES #444). Only when the lock refused an `\author`
  // (`\author:redefined`), each one still undefined takes, for the author content, OmniBus's meaning
  // (OmniBus.cls.ltxml:97-102: `\prefix`, `\suffix`, `\fnms`, `\snm`, `\inits`; `\orcid`, omnibus_cls.rs) or the
  // classes' own identity one (`\degs`, `\roles`: IOS-Book-Article.cls:1153-1174; `\particle`: econsocart.cls:2104) and
  // is undefined again after it; a class's or binding's own definition stands, and elsewhere an undefined one still
  // errors as in pdflatex. The restore list is captured before the content runs, so a nested author keeps it.
  DefMacro!(
    "\\lx@author@markup@provide{}{}",
    "\\ifx#1\\@undefined\\let#1#2\\expandafter\\def\\expandafter\\lx@author@markup@unprovide\\expandafter{\\lx@author@markup@unprovide\\let#1\\@undefined}\\fi"
  );
  DefMacro!("\\lx@author@markup@begin", sub[()] {
    Ok(if lookup_bool("\\author:redefined") {
      TokenizeInternal!(
        r"\def\lx@author@markup@unprovide{}\lx@author@markup@provide\prefix\@firstofone\lx@author@markup@provide\suffix\@firstofone\lx@author@markup@provide\particle\@firstofone\lx@author@markup@provide\fnms\@firstofone\lx@author@markup@provide\snm\@firstofone\lx@author@markup@provide\inits\@firstofone\lx@author@markup@provide\degs\@firstofone\lx@author@markup@provide\roles\@firstofone\lx@author@markup@provide\orcid\lx@author@orcid"
      )
    } else {
      TokenizeInternal!(r"\def\lx@author@markup@unprovide{}")
    })
  });
  DefMacro!("\\lx@author@markup@run{}{}", "#2#1");
  DefMacro!("\\lx@author@orcid[]{}", "\\lx@add@orcid{#2}");
  DefMacro!("\\lx@inst@mark{}", sub[(label)] {
    let call = if is_footnote_symbol_operand(label.unlist_ref()) {
      Tokens::new(keepsup(label.unlist()))
    } else {
      Invocation!(
        T_CS!("\\lx@request@frontmatter@annotation"),
        vec![Some(Tokens!(T_LETTER!("a"), T_LETTER!("f"), T_LETTER!("f"), T_LETTER!("i"), T_LETTER!("l"), T_LETTER!("i"), T_LETTER!("a"), T_LETTER!("t"), T_LETTER!("i"), T_LETTER!("o"), T_LETTER!("n"))), Some(label)]
      )
    };
    Ok(call)
  });
  DefMacro!(
    "\\lx@add@editor[]{}",
    "\\lx@add@frontmatter{ltx:creator}[role=editor,#1]{\\lx@personname{#2}}"
  );
  DefMacro!(
    "\\lx@add@translator[]{}",
    "\\lx@add@frontmatter{ltx:creator}[role=translator,#1]{\\lx@personname{#2}}"
  );

  DefMacro!(
    "\\lx@add@date[]{}",
    "\\lx@clear@frontmatter{ltx:date}[role=created,#1]\\lx@add@frontmatter{ltx:date}[role=created,#1]{#2}"
  ); // no duplicates w/same role
  DefMacro!("\\lx@copyright@holder", "");
  DefMacro!("\\lx@copyright@date", "");
  DefMacro!("\\lx@add@copyright{}", "\\lx@add@date[role=copyright]{#1}");
  // Next two are for when copyright holder & year are given by 2 separate macros
  DefMacro!(
    "\\lx@add@copyrightholder{}",
    "\\gdef\\lx@copyright@holder{#1}\\lx@add@copyright{\\lx@copyright@holder\\ \\lx@copyright@date}"
  );
  // A year with no holder is the year alone: Perl's `\ifx.\lx@copyright@holder.` (Base_Utility.pool
  // .ltxml:606-608; aipproc's `\copyrightyear`) compares `.` with the macro and never holds, so it read ", 2020" (KPE #418).
  DefMacro!(
    "\\lx@add@copyrightyear{}",
    "\\gdef\\lx@copyright@date{#1}\\lx@add@copyright{\\ifx\\lx@copyright@holder\\@empty\\else\\lx@copyright@holder, \\fi\\lx@copyright@date}"
  );

  // Where the abstract SITS in the source: an empty internal marker constructed at
  // the current point when the abstract is queued (both the `{abstract}` env and a
  // class's `\abstract{…}` store route through the two macros below). The
  // abstract-only fallback reads it to tell title-page LAYOUT (what precedes the
  // abstract) from BODY (what follows it) — the flush point (first `\section`, or
  // `\end{document}`) cannot: a sectionless manual would otherwise see its whole
  // body as cover. `_Capture_`-suffixed names are admitted anywhere by the model
  // (model.rs) so no paragraph is auto-opened; every marker is removed by
  // `insert_frontmatter`/`remove_frontmatter_marks` before output. OXIDIZED_DESIGN #246.
  DefConstructor!("\\lx@frontmatter@mark{}", sub[document, _args] {
    // Only meaningful BEFORE the frontmatter is placed: a later abstract (lips'
    // `\DocInput`-ed `\begin{abstract}` after `\maketitle`) is handled by
    // `insert_late_frontmatter`, and a marker built then would outlive every
    // removal point (s107: 8 docs leaked `<_Frontmatter_Capture_>`).
    // …and only once there is an open element to sit in: a preamble `\abstract{…}`
    // (svjour's setter, fixture structure/svabstract) is constructed BEFORE the
    // whatsit that opens <ltx:document>, and inserting anything there auto-creates
    // the root early (a duplicated RelaxNGSchema PI). Construction runs after
    // digestion, so the digest-time `inPreamble` flag is useless here — ask the
    // tree via the current element (which, unlike the root element, also holds
    // inside a streaming fragment; 114_streaming_structure faketitlepage).
    if !lookup_bool("frontmatter_done") && document.get_element().is_some() {
      document.insert_element("ltx:_Frontmatter_Capture_", Vec::new(), None)?;
    }
  });
  DefMacro!(
    "\\lx@add@abstract[]{}",
    "\\lx@frontmatter@mark{ltx:abstract}\\lx@clear@frontmatter{ltx:abstract}\\lx@add@frontmatter{ltx:abstract}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@keywords[]{}",
    "\\lx@clear@frontmatter{ltx:keywords}\\lx@add@frontmatter{ltx:keywords}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@classification[]{}",
    "\\lx@add@frontmatter{ltx:classification}[#1]{#2}"
  );
  // To handle the above as environments
  DefMacro!(
    "\\lx@begin@abstract[]",
    "\\lx@frontmatter@mark{ltx:abstract}\\lx@clear@frontmatter{ltx:abstract}\\lx@add@frontmatter@until{ltx:abstract}[#1]{\\lx@end@abstract}"
  );

  // Like \let \relax, but \relax not def yet!
  // The until-body terminals: no-ops when the body's loop sees them; inside a
  // nested group they end the body from there (stomach.rs).
  DefPrimitive!("\\lx@end@abstract", {
    until_terminal_inside_group(&T_CS!("\\lx@end@abstract"))?;
  });

  DefMacro!(
    "\\lx@begin@keywords[]",
    "\\lx@clear@frontmatter{ltx:keywords}\\lx@add@frontmatter@until{ltx:keywords}[#1]{\\lx@end@keywords}"
  );

  DefPrimitive!("\\lx@end@keywords", {
    until_terminal_inside_group(&T_CS!("\\lx@end@keywords"))?;
  });

  // Add random notes about the document itself
  DefMacro!(
    "\\lx@add@pubnote[]{}",
    "\\lx@add@frontmatter{ltx:pubnote}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@pubnote@thanks[]{}",
    "\\lx@add@frontmatter{ltx:pubnote}[role=thanks,#1]{#2}"
  );

  // Other kinds of notes?

  // The following add various forms of contact information to a creator
  DefMacro!(
    "\\lx@add@contact []{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[#1]{#2}"
  );

  DefMacro!(
    "\\lx@add@affiliation[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=affiliation,#1]{\\lx@affiliation@withinst{#2}}"
  );
  // The affiliation-side twin of `\lx@author@withinst`: inside affiliation
  // content `\inst{n}` SETS the affiliation's label (beamerbasetitle.sty:148
  // `\institute{\inst{1}Univ A \and \inst{2}Univ B}` marks each institute
  // with the number the authors' `\inst{1}` request) — the meaning
  // `\lx@affiliation@withsup` gives a `\textsuperscript` mark, never a
  // typeset `<sup>`; `\def`, as in `\lx@author@withinst` (a class's
  // document-level `\inst` is the affiliation LIST, not a label).
  DefMacro!(
    "\\lx@affiliation@withinst{}",
    "\\let\\lx@saved@inst\\inst\\def\\inst##1{\\lx@sup@setlabel@affiliation{##1}}#1\\let\\inst\\lx@saved@inst"
  );
  DefMacro!(
    "\\lx@add@altaffiliation[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=altaffiliation,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@address[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=address,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@altaddress[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=altaddress,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@currentaddress[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=currentaddress,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@email [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=email,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@url [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=url,#1]{#2}"
  );
  // The ORCID iD logo — the green "iD" badge — as ONE self-contained SVG asset,
  // defined in the kernel so every orcid rendering path reuses it (orcidlink.sty's
  // \orcidlogo aliases this; \lx@add@orcid embeds it). Adapted from
  // https://orcid.org/assets/vectors/orcid.logo.icon.svg; the viewBox exactly
  // frames the 72×72 disk and the `.ltx_orcidlogo` CSS rule sizes it to the text
  // (1em), so it renders as an inline glyph — not the oversized 1.7em badge that
  // overflowed the line (html_feedback #6895, #6016, #5789, #2176, #5615).
  DefConstructor!("\\lx@orcidlogo", sub[document, _args, _props] {
    document.open_element("svg:svg", Some(string_map!(
      "class" => "ltx_orcidlogo", "width" => "1em", "height" => "1em",
      "viewBox" => "0 0 72 72", "version" => "1.1")), None)?;
    document.open_element("svg:path", Some(string_map!(
      "fill" => "#A6CE39",
      "d" => "M72,36 C72,55.884375 55.884375,72 36,72 C16.115625,72 0,55.884375 0,36 C0,16.115625 16.115625,0 36,0 C55.884375,0 72,16.115625 72,36 Z")), None)?;
    document.close_element("svg:path")?;
    document.open_element("svg:g", Some(string_map!(
      "fill" => "#FFFFFF", "transform" => "translate(18.868966, 12.910345)")), None)?;
    document.open_element("svg:polygon", Some(string_map!(
      "points" => "5.03734929 39.1250878 0.695429861 39.1250878 0.695429861 9.14431787 5.03734929 9.14431787 5.03734929 22.6930505 5.03734929 39.1250878")), None)?;
    document.close_element("svg:polygon")?;
    document.open_element("svg:path", Some(string_map!(
      "d" => "M11.409257,9.14431787 L23.1380784,9.14431787 C34.303014,9.14431787 39.2088191,17.0664074 39.2088191,24.1486995 C39.2088191,31.846843 33.1470485,39.1530811 23.1944669,39.1530811 L11.409257,39.1530811 L11.409257,9.14431787 Z M15.7511765,35.2620194 L22.6587756,35.2620194 C32.49858,35.2620194 34.7541226,27.8438084 34.7541226,24.1486995 C34.7541226,18.1301509 30.8915059,13.0353795 22.4332213,13.0353795 L15.7511765,13.0353795 L15.7511765,35.2620194 Z")), None)?;
    document.close_element("svg:path")?;
    document.open_element("svg:path", Some(string_map!(
      "d" => "M5.71401206,2.90182329 C5.71401206,4.441452 4.44526937,5.72914146 2.86638958,5.72914146 C1.28750978,5.72914146 0.0187670918,4.441452 0.0187670918,2.90182329 C0.0187670918,1.33420133 1.28750978,0.0745051096 2.86638958,0.0745051096 C4.44526937,0.0745051096 5.71401206,1.36219458 5.71401206,2.90182329 Z")), None)?;
    document.close_element("svg:path")?;
    document.close_element("svg:g")?;
    document.close_element("svg:svg")?;
  });
  // A link to https://orcid.org/<#1> wrapping content #2 (class ltx_orcid). Shared
  // by \lx@add@orcid and orcidlink.sty's \orcidlink family. Perl orcidlink #2681.
  DefConstructor!(
    "\\lx@orcidlink{}{}",
    "<ltx:ref title='ORCID #1' class='ltx_orcid' href='https://orcid.org/#1'>#2</ltx:ref>"
  );
  // The single canonical author-ORCID frontmatter macro: a `ltx:contact[role=orcid]`
  // (consistent with \lx@add@email / \lx@add@affiliation &c.) whose value is the iD
  // logo linked to the author's orcid.org page. The id is preserved in the href for
  // the metadata. Every class binding routes its \orcid here (html_feedback #6571).
  DefMacro!(
    "\\lx@add@orcid [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=orcid,#1]{\\lx@orcidlink{#2}{\\lx@orcidlogo}}"
  );
  // Beyond-Perl (OXIDIZED_DESIGN #52): the arXiv "\thanks abuse" idiom smuggles
  // affiliations into an author \thanks{...}, linking them to authors by a
  // leading superscript mark ($^{n}$ / \textsuperscript{n}). \thanks is
  // semantically an acknowledgement, so we re-route ONLY when the content BEGINS
  // with such a mark (the abuse signature — a genuine acknowledgement never
  // starts with a bare superscript); every other \thanks stays a role=thanks
  // note. When detected we feed each $^{n}$-delimited segment through
  // \lx@affiliation@withsup, which sets the affiliation:N label that the authors'
  // own marks already request (relocate_annotations then links
  // author<->affiliation). Witness arXiv:2606.00313.
  // The argument is an ordinary one, as LaTeX's `\thanks` reads it and as a title's `\thanks`
  // (`\lx@add@pubnote@thanks`) does: Perl's Semiverbatim (Base_Utility.pool.ltxml:661) froze `$`, `^`, `_` to
  // catcode other, so an author's `\thanks{... 10 m$^{2}$ lab.}` printed its math as text (KNOWN_PERL_ERRORS #515;
  // repro sectioning-frontmatter/author_thanks_and_footnote_keep_their_math).
  DefMacro!("\\lx@add@thanks [] {}", sub[(attr, content)] {
    if starts_with_affiliation_mark(&content) {
      let mut calls: Vec<Token> = Vec::new();
      for seg in split_wrapped_affiliation_marks(content) {
        if seg.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
          continue;
        }
        let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(seg)]);
        calls.extend(
          Invocation!(T_CS!("\\lx@add@affiliation"), vec![None, Some(withsup)]).unlist());
      }
      Ok(Tokens::new(calls))
    } else {
      // SURPASS over Perl (OXIDIZED_DESIGN #156): render an author-attached `\thanks`
      // as a MARKED note (a superscript mark on the author name + margin/footnote
      // content) instead of a `role=thanks` CONTACT that reads inline like an
      // affiliation next to the name. Route to `<ltx:note role="thanks">` so the
      // existing `ltx:note` footnote template (mark + `ltx_note_outer`/`ltx_note_content`)
      // renders it, and attach semantic CSS hooks so a theme can style each kind of
      // thanks content: `ltx_note_frontmatter` (the ar5iv frontmatter-note hook) plus a
      // best-effort content-kind class `ltx_thanks_<kind>` from `classify_thanks`. Perl
      // keeps the inline contact (SHARED readability gap). Witnesses arXiv 2512.24601
      // (correspondence), 1510.02728 (funding).
      let kind = classify_thanks(content.clone().untex_string().as_ref());
      let mut opts = mouth::tokenize_internal(TeXString::assembled(s!(
        "role=thanks,class=ltx_note_frontmatter ltx_thanks_{kind}"
      )))
      .unlist();
      if let Some(a) = &attr {
        opts.push(T_OTHER!(","));
        opts.extend(a.unlist_ref().iter().copied());
      }
      Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"),
        vec![Some(mouth::tokenize_internal("ltx:creator")),
             Some(mouth::tokenize_internal("ltx:note")),
             Some(Tokens::new(opts)), Some(note_body(content))]))
    }
  });
  // An ordinary argument, as `\lx@add@thanks` (an author's `\footnote`; KNOWN_PERL_ERRORS #515).
  DefMacro!("\\lx@add@note [] {}", sub[(attr, content)] {
    let mut opts = mouth::tokenize_internal("role=note").unlist();
    if let Some(a) = &attr {
      opts.push(T_OTHER!(","));
      opts.extend(a.unlist_ref().iter().copied());
    }
    Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"),
      vec![Some(mouth::tokenize_internal("ltx:creator")),
           Some(mouth::tokenize_internal("ltx:contact")),
           Some(Tokens::new(opts)), Some(note_body(content))]))
  });

  // This corresponds to standard LaTeX,
  // The command replaces any previous authors/creators;
  // It defining several creators separated by \and;
  // and can use \\ to separate affiliation from each author.
  // BUT ALSO, this markup is commonly abused by putting authors & affiliations in
  // seemingly random orders, but adding superscript markers to connect them.
  // Assumption: superscript near END is used for authors; near FRONT for affiliation.
  // NOTE: This is a mess! really should use role, so could apply to editors also
  // AND, matching \\ this way fails to catch \\[1em], so really should Let it

  // 56fl (OXIDIZED_DESIGN #253) — the raw-class `\author` surplus. `\author` is
  // locked to the kernel shape `[short]{name}` (Perl latex_constructs.pool.ltxml
  // :1076 too), so a raw class's own redefinition is refused and recorded as
  // `\author:redefined` (state.rs `install_definition`). Such a class declared
  // MORE arguments — a trailing `[keyval]` (cas-common.sty:895 `O{} m O{}`:
  // `[type=editor, orcid=…]`) or a second mandatory `{affiliation}`
  // (cnbwp.cls:273 `\def\author{\@ifnextchar[…}` → `\CNB@authorLong#1#2`) —
  // which the kernel shape left in the stream to typeset as a `<para>` before
  // the frontmatter (schema-invalid, faux content; Perl leaks identically).
  // Running the class body instead would LOSE the authors: cas-common/cnbwp/
  // aomart store names in class-private accumulators laid out only by their
  // own (locked) `\maketitle` (verified: zero <creator>). So the locked
  // `\author` keeps the semantics and, ONLY when a class redefined it, (a)
  // absorbs the trailing argument into the frontmatter API — `orcid=` →
  // `ltx:contact role=orcid`, a trailing group → `role=affiliation`, other
  // keys (type, auid, bioid, prefix, suffix, role, style) are presentational
  // and dropped — and (b) APPENDS creators (`\lx@add@author`) instead of the
  // LaTeX-faithful dequeue-replace, since such a class calls `\author` once
  // per author (cnbwp: 3 creators, each with its affiliation). A document that
  // never redefined `\author` is byte-identical: `\@ifnextchar` is never even
  // reached. Twins carrying the tail: kernel `\author` (sect05.rs),
  // inst_support, sv_support, llncs. Residual risk, documented: with a same-
  // shape class redefinition, a brace group immediately after `\author{…}` on
  // the same paragraph is read as the class's affiliation argument (`\par`
  // stops `\@ifnextchar`).
  DefMacro!("\\lx@add@authors@adaptive{}", sub[(stuff)] {
    add_authors_calls(stuff, !lookup_bool("\\author:redefined"))
  });
  DefMacro!("\\lx@author@trailing", sub[()] {
    if lookup_bool("\\author:redefined") {
      Ok(TokenizeInternal!(
        r"\@ifnextchar[{\lx@author@trailing@opt}{\@ifnextchar\bgroup{\lx@author@trailing@mand}{}}"
      ))
    } else {
      Ok(Tokens!())
    }
  });
  DefMacro!(
    "\\lx@author@trailing@opt[]",
    r"\lx@add@author@keyvals{#1}\lx@author@trailing"
  );
  // An author keyval block (`type=editor, auid=000, orcid=0000-…`, the
  // cas-common `stm/author` key family, .sty:686) → frontmatter contacts:
  // `orcid=` is identity (`ltx:contact role=orcid`); `type`, `auid`, `bioid`,
  // `alt`, `style`, `prefix`, `suffix`, `role` are layout/production keys
  // and are dropped. Shared by the raw-class tail above and the cas-dc/cas-sc
  // binding's `\author[marks]{name}[keyvals]` (cas_dc_cls.rs).
  DefMacro!("\\lx@add@author@keyvals{}", sub[(keyvals)] {
    let mut out: Vec<Token> = Vec::new();
    let keyvals = keyvals.to_string();
    for item in keyvals.split(',') {
      if let Some((key, value)) = item.split_once('=')
        && key.trim() == "orcid"
      {
        let value = value.trim().trim_matches(|c| c == '{' || c == '}').trim();
        if !value.is_empty() {
          let value = mouth::tokenize(TeXString::assembled(value.to_string()));
          out.extend(Invocation!(T_CS!("\\lx@add@orcid"), vec![None, Some(value)]).unlist());
        }
      }
    }
    Ok(Tokens::new(out))
  });
  DefMacro!(
    "\\lx@author@trailing@mand{}",
    r"\lx@add@affiliation{#1}\lx@author@trailing"
  );

  /// The `\lx@add@authors` machinery (author-line parsing: `\and`/`and`
  /// splitting, `$^{1,}$` affiliation marks, tabular/minipage lines, ijcai
  /// `\affiliations`) as a function, so both the replacing kernel `\author`
  /// and the appending raw-class mode share it.
  fn add_authors_calls(stuff: Tokens, replace: bool) -> Result<Tokens> {
    add_authors_calls_sectioned(stuff, replace, true, None, false)
  }
  /// [`add_authors_calls`]; `sectioned` false for the names an IJCAI split already took out, which
  /// go round no more (Perl's `\lx@add@authors` has no marker branch, ijcai.sty.ltxml:40), and for a per-author
  /// class's `\author`, which Perl adds whole (its names re-entering as `\lx@ijcai@names` would replace the authors
  /// before it). `keyvals` are handed to every `\lx@add@author` the block makes (a per-author class's
  /// `\author[labels]{…}`: revtex `annotations=`); `cautious` for a per-author class binding, whose names lines split
  /// only where they read as names ([`split_author_line_cautious`]).
  fn add_authors_calls_sectioned(
    stuff: Tokens,
    replace: bool,
    sectioned: bool,
    keyvals: Option<Tokens>,
    cautious: bool,
  ) -> Result<Tokens> {
    // Beyond-Perl (surpasses Perl; KNOWN_PERL_ERRORS #100): IJCAI-style author
    // blocks — ijcai97.sty and its derivatives (e.g. the ttm.sty in
    // arXiv:2401.03955) — pack names, `\affiliations` and a comma-separated
    // `\emails` list into ONE `\author{}`. The default splitter below does not
    // recognise those section markers, so the comma-joined email list is shredded
    // into phantom author creators and the affiliation is dropped (Perl 0.8.8 does
    // the same, and both raise `\affiliations`/`\emails` as undefined). When the
    // body carries either marker, delegate to the shared sectioned-author machinery
    // (`\lx@ijcai@authorsplit`, also used by `ijcai_sty`): it consumes the markers
    // as `Until:` delimiters (so they no longer error) and splits names /
    // affiliations / emails, attaching the n-th email to the n-th author. This runs
    // before any dequeue/normalization because the delegate re-enters
    // `\lx@add@authors` on the (marker-free) name list.
    // Witness html_feedback#1361 + #1362. A marker counts at the block's top level only, and only as a separator:
    // undefined there or the ijcai binding's no-op. A document's own `\emails`/`\affiliations` with text is author
    // text, which LaTeX expands — counted, the split handed it back unsplit, without end (`\emails` before any
    // `\affiliations`, 2407.10582; `\affiliations` inside `\thanks`, 2505.05474; `PushbackLimit`).
    if sectioned && has_ijcai_section_marker(&stuff)? {
      let mut out = vec![T_CS!("\\lx@ijcai@authorsplit")];
      out.extend(stuff.unlist());
      out.push(T_CS!("\\affiliations"));
      out.push(T_CS!("\\done"));
      return Ok(Tokens::new(out));
    }
    let mut calls: Vec<Token> = Vec::new();
    // A per-author class binding's `\author` (aastex, amsart, revtex, an IEEE block, a labelled authblk `\author`),
    // which Perl reads as one author, splits its names only where they read as names (63e); a raw class's own
    // `\author` (`\lx@add@authors@adaptive`, 2309.03769's dan2e.sty) splits as the article `\author` does.
    let split_names = |line: Tokens| {
      if cautious {
        split_author_line_cautious(line)
      } else {
        split_author_line(line)
      }
    };
    // LaTeX-faithful `\author` REPLACES (the last call wins); a raw class that
    // redefined `\author` to call it once per author appends instead (56fl).
    if replace {
      dequeue_front_matter("ltx:creator", &[("role", "author")]);
      // LaTeX's `\def\@author{#2}` replaces the authors a `\maketitle` that left
      // `\author` live already typeset (KOMA ≥ 3.12, scrbook.cls:3263-3290; a class's
      // title page from `\AfterEndPreamble`, udesoftec.cls:1230-1241) for the NEXT
      // `\maketitle`: they are superseded there (`\lx@maketitle@supersede`), not here —
      // without one, the authors a fallback placed stay (Perl the same).
      assign_value(
        "lx_authors_superseded",
        Stored::Bool(true),
        Some(Scope::Global),
      );
      // A store handed on since that flush went to the authors replaced here: the next
      // harvest hands it again (`harvest_stores`, `lx_stores_late`).
      assign_value(
        "lx_stores_late",
        Stored::String(pin("")),
        Some(Scope::Global),
      );
      // The replaced authors take their handed tail with them (a binding's own
      // `\lx@add@authors` never runs `\lx@author@flush`), and the creators this
      // `\author` makes are counted from here (`\lx@author@handed`).
      assign_value("lx_author_handed", Stored::Bool(false), Some(Scope::Global));
      assign_value(
        "lx_author_creators_before",
        Stored::Int(queued_creator_count() as i64),
        Some(Scope::Global),
      );
    }
    // Consume any `\\[len]` / `\\*[len]` row-break optionals up front so the line
    // splits below see a bare `\\` (KNOWN_PERL_ERRORS #75, witness 2605.23553). This
    // also runs ahead of the tabular/minipage fallback: that fallback is a
    // last-resort escape hatch for author markup we cannot structure, and its
    // `<tabular>`-in-`<personname>` output is a presentational artifact we do not
    // want in frontmatter anyway, so there is nothing to preserve by skipping it.
    let stuff = strip_linebreak_options(stuff);
    let stuff = unbrace_acm_author_columns(stuff);
    let stuff = unwrap_alignment_environment(stuff);
    // Beyond-Perl (OXIDIZED_DESIGN #52), two composable normalizations applied
    // BEFORE branch selection so both branches benefit, and so a symbol mark can
    // no longer spuriously trigger the affiliation-marker branch:
    //   1. horizontal-space separators (`\hspace{len}` / `\hfill`) → `\quad`, the
    //      "regular" poor-man's author separator LaTeXML's `\and`/`\quad` splitter
    //      otherwise misses (witness arXiv:2506.06941, six authors bunched);
    //   2. footnote-SYMBOL superscripts (`$^{*}$`, `\textsuperscript{\dagger}` —
    //      equal-contribution / corresponding notes, NEVER affiliation numbers) →
    //      a visible `\lx@frontmatter@keepsup` sup, so they render instead of
    //      being consumed into an unmatched `affiliation:*` label and dropped.
    // For the witness, (2) removes its ONLY `^`, dropping the whole block into the
    // clean no-marker branch where (1)'s `\quad`s split all six authors.
    let stuff = normalize_hspace_separators(stuff);
    let stuff = rewrite_symbol_superscripts(stuff);
    let stuff = rewrite_ordinal_superscripts(stuff);
    // If too much formatting, fall back to unstructured author content
    let stuff_string = stuff.to_string();
    // Perl Base_Utility.pool.ltxml:693 tests `{tabular}`, `{minipage}` and `\halign`; a `\\` inside any alignment ends
    // its row, not an author line, so the whole family keeps the author content whole: a `tabular*`/`tabularx` author
    // block was split at its row ends, leaving the alignment open across the pieces (2609.34061, 34965, 39374, 39909;
    // KNOWN_PERL_ERRORS #502).
    if [
      "{tabular}",
      "{tabular*}",
      "{tabularx}",
      "{tabulary}",
      "{longtable}",
      "{array}",
      "{minipage}",
      "\\halign",
    ]
    .iter()
    .any(|form| stuff_string.contains(form))
    {
      calls
        .extend(Invocation!(T_CS!("\\lx@add@author"), vec![keyvals.clone(), Some(stuff)]).unlist());
    } else if position_of(&stuff, &authorsup_markers()).is_some()
      && (!cautious || answers_its_marks(&stuff))
    {
      // (a per-author class binding keeps its marks visible on the names, as Perl does, and takes the unmarked path
      // below — its affiliation commands answer no mark (an IEEE block's `\IEEEauthorblockA`, amsart's `\address`) —
      // unless the block holds its marked affiliations itself (2401.14196's `\author[*]{… \\ $^1$DeepSeek-AI …}`))
      let mut entries: Vec<(AuthorLineKind, Tokens)> = Vec::new();
      let mut prefix_marked_names = false;
      // Split on the `\and` family FIRST so an `\and` is a HARD author boundary:
      // a marker-less line — an author whose only superscript is macro-delivered
      // (`\handPointerZ`), or a continuation affiliation — never merges into an
      // author from a PREVIOUS `\and` group. Within a group, `\quad`/`\\` still
      // separate the name line from its affiliation lines, and a marker-less line
      // still continues the group's own last entry. Groups carry no `\and`, so
      // reusing `author_affil_splits` for the intra-group split is equivalent to
      // splitting on `\quad`/`\qquad`/`\\`. html_feedback#1021 F2 residual:
      // `Alice\mk$^*$ \and Bob\mk` was one merged creator; now two.
      // OXIDIZED_DESIGN #52(g).
      // The block's lines, in the order the loop below reads them, for a line to look at the lines after it.
      let groups: Vec<Tokens> = split_tokens(stuff, author_and_splits())
        .into_iter()
        .map(unwrap_alignment_environment)
        .filter(|group| !group.is_empty())
        .collect();
      let block_lines: Vec<Tokens> = groups
        .iter()
        .flat_map(|group| split_tokens_delimited(group.clone(), author_affil_splits()))
        .map(|(_, line)| line)
        .collect();
      let mut position = 0;
      for group in groups {
        // Marker-less lines may only continue an entry created WITHIN this
        // `\and` group — never one from a previous group.
        let group_start = entries.len();
        let mut names_line = true;
        for (delimiter, line) in split_tokens_delimited(group, author_affil_splits()) {
          position += 1;
          let beside = beside_prefix_marked_names(prefix_marked_names, &mut names_line, &delimiter);
          if line.is_empty() {
            continue;
          }
          match position_of(&line, &authorsup_markers()) {
            None => {
              // A marker-less line that is purely a list of email addresses is a
              // SHARED email line (\texttt{a@x, b@y}) covering all the authors, not
              // a continuation of the previous affiliation — give it its own email
              // contact so it is shown once instead of being welded into an
              // affiliation's text (KNOWN_PERL_ERRORS #75, witness 2605.23553).
              if line_is_email_list(&line) {
                entries.push((AuthorLineKind::Email, line));
              } else if entries.len() > group_start {
                // continues an entry from THIS `\and` group; Append, with the delimiter that split the lines
                // (`\\` → break, `\quad`/`\qquad` → space) put back where it stood: Perl (Base_Utility.pool.ltxml:701-703 `Tokens($entries[-1][1],
                // $line)`) welds the two lines' words ("Department of PhysicsUniversity of Somewhere";
                // KNOWN_PERL_ERRORS #446, witness jacow-collaboration). Repro
                // sectioning-frontmatter/author_continuation_line_keeps_its_break.
                let last = entries.last_mut().unwrap();
                let mut appended = last.1.clone().unlist();
                appended.extend(delimiter);
                appended.extend(line.unlist());
                last.1 = Tokens::new(appended);
              } else {
                // First line of this `\and` group and it has no marker → a NEW author (never merged
                // back into the previous group), whole as Perl keeps it ("safest to assume author?",
                // Base_Utility.pool.ltxml:705-706) — unless it reads as several names, then each its own
                // (`Aghil Alaee\footnote{…} \,\,and Hari K. Kunduri\footnote{…}\\ … $^a$ Department …`,
                // 1407.0988), as the merged-author check would otherwise report.
                if name_count(&visible_name_text(line.unlist_ref())) >= 2 {
                  for author in split_author_line(line) {
                    entries.push((AuthorLineKind::Author, author));
                  }
                } else {
                  entries.push((AuthorLineKind::Author, line));
                }
              }
            },
            Some(_) => {
              // "\textsuperscript{n}Affil" (the marker LEADS the line) → an
              // affiliation; "Name\textsuperscript{n}" (a name precedes the
              // marker) → an author line, split into the individual creators it
              // names (see split_author_line). The old `p < 8` token-count proxy
              // misread short author names like "Min Xu" (html_feedback#6614) —
              // key on name-before-marker, which is length-independent.
              // The block's first line is its names even when the marks lead them (`\textsuperscript{a}Ann Able,
              // \textsuperscript{b}Bob Baker\\ \textsuperscript{a}Univ A`; repro
              // sectioning-frontmatter/author_prefix_marks_first_line_is_names): an author block opens with authors.
              // So is the first line of a later `\and` group once the block's first line set that convention, when
              // its mark is one no author before it requests (`\textsuperscript{1}Ann Able \and
              // \textsuperscript{2}Bob Baker\\ ...`); a mark an author requests leads its affiliation, the `\and`
              // separating affiliations (`... \and \textsuperscript{2}Univ B`), and once the affiliations have begun,
              // the authors before them, every later group is one (`... \and $^{3}$Univ C`, which no author cites).
              let first_line = !entries
                .iter()
                .any(|(kind, _)| *kind == AuthorLineKind::Author);
              let leads = marker_leads(&line);
              if first_line && leads {
                prefix_marked_names = true;
              }
              let names_a_new_mark = || {
                let requested: Vec<String> = entries
                  .iter()
                  .filter(|(kind, _)| *kind == AuthorLineKind::Author)
                  .flat_map(|(_, author)| author_mark_operands(author.unlist_ref()))
                  .flat_map(|operand| {
                    clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation")
                  })
                  .collect();
                mark_operands(line.unlist_ref())
                  .first()
                  .is_some_and(|operand| {
                    clean_frontmatter_labels(
                      &Tokens::new(operand.clone()).to_string(),
                      "affiliation",
                    )
                    .iter()
                    .any(|label| !requested.contains(label))
                  })
              };
              let affiliations_begun = entries
                .iter()
                .any(|(kind, _)| *kind == AuthorLineKind::Affiliation);
              // A line marked as the first that reads as names is a name by where it stands: as the first line of an
              // `\and` group when no author before it requests its mark (Perl-era rule) or an affiliation below
              // answers it ([`mark_answered_below`]: `… $^{1}$Univ A \and $^{2}$Bob Baker\\ $^{2}$Univ B`); before the
              // affiliations, beside the first on its printed line when its mark is new or answered below
              // (`$^{1}$Ann Able \quad $^{1}$Bob Baker \quad $^{2}$Cat Cole\\ $^{1}$Univ A\\ $^{2}$Univ B`), and on a
              // line of its own when its mark is new and answered below (`$^{1}$Ann Able\\ $^{2}$Bob Baker\\
              // $^{1}$Univ A\\ $^{2}$Univ B`). On a line of its own, an affiliation whose mark a name above requests stays
              // one however it reads (`$^{1}$Carnegie Mellon\\ $^{1}$School of Computer Science`); beside the first or at
              // an `\and` group's head it does unless an affiliation below answers it (`$^{1}$Ann Able \quad
              // $^{1}$Google DeepMind`); one nothing below answers stays one (`$^{3}$Google DeepMind`).
              let names = || reads_as_names(&line);
              let answered = || mark_answered_below(&line, &block_lines[position..]);
              let group_first = entries.len() == group_start;
              let marked_name = || {
                (group_first
                  && ((!affiliations_begun && names_a_new_mark()) || (names() && answered())))
                  || (!affiliations_begun
                    && names()
                    && if beside {
                      names_a_new_mark() || answered()
                    } else {
                      names_a_new_mark() && answered()
                    })
              };
              if first_line || !leads || (prefix_marked_names && marked_name()) {
                for author in split_names(line) {
                  entries.push((AuthorLineKind::Author, author));
                }
              } else {
                // A marker-led affiliation line may carry MULTIPLE
                // `\textsuperscript{n}Affil` institutions on one space-separated line
                // (html_feedback#6242, arXiv:2510.02340) — `\textsuperscript{1}Univ A
                // \textsuperscript{2}Univ B`. Split at each whitespace-preceded mark
                // (reusing the `\thanks`-abuse splitter, which never breaks a
                // superscript glued INSIDE an institution name) so each numbered
                // institution becomes its own affiliation and attaches to its authors
                // by number, instead of merging into one.
                for seg in split_wrapped_affiliation_marks(line) {
                  if seg.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
                    continue;
                  }
                  entries.push((AuthorLineKind::Affiliation, seg));
                }
              }
            },
          }
        }
      }
      let author_count = entries
        .iter()
        .filter(|(k, _)| *k == AuthorLineKind::Author)
        .count();
      for (kind, line) in entries {
        match kind {
          AuthorLineKind::Author => {
            let withsup = Invocation!(T_CS!("\\lx@author@withsup"), vec![Some(line)]);
            calls.extend(
              Invocation!(T_CS!("\\lx@add@author"), vec![
                keyvals.clone(),
                Some(withsup)
              ])
              .unlist(),
            );
          },
          AuthorLineKind::Affiliation => {
            let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(line)]);
            calls.extend(
              Invocation!(T_CS!("\\lx@add@affiliation"), vec![None, Some(withsup)]).unlist(),
            );
          },
          AuthorLineKind::Email => {
            // A shared email line otherwise attaches to whatever creator is
            // "current" (the LAST author), bunching every address under one
            // person. Instead, resolve the individual addresses (`a@x, b@y, c@z`
            // distributed, or `{a,b,c}@dom` grouped -> expanded) and, when there
            // are no more than one per author, hand each to `\lx@add@email` with
            // labelseq=author: address i gets label author:i, which relocates to
            // the creator's own author:N sequence label. So N addresses distribute
            // one-per-author, and a single shared address lands on author:1 (the
            // lead), never a random trailing author. Preserve a whole-line
            // \texttt/\url wrapper by re-wrapping each address. If there are MORE
            // addresses than authors (cannot map cleanly), keep the original line
            // as one contact (the prior behavior). OXIDIZED_DESIGN #52(j).
            let addresses = email_addresses(&line).map_or(0, |a| a.len());
            let placement = if author_count >= 1 && addresses <= author_count {
              AddressPlacement::Sequence
            } else {
              AddressPlacement::OneContact
            };
            calls.extend(email_line_calls(line, placement)?);
          },
        }
      }
    } else {
      // No superscript markers. Split into author GROUPS on the \and family /
      // \quad only — NOT comma (author_group_splits). Within a group, the first
      // \\-delimited line is the author-name list (comma / " and "-split via
      // split_author_line); each remaining \\-line is an affiliation, attached to
      // the group's LAST author (the "affiliation follows the name(s) before \\"
      // convention). Splitting groups on comma shredded multi-part addresses
      // ("…Laboratory, Laurel, MD 20723") into fake authors and mislabeled the
      // trailing \email line as an affiliation. OXIDIZED_DESIGN #52 (surpass-Perl,
      // Perl's @authorsplits shares the comma); witness arXiv:2606.00315.
      for block in split_tokens(stuff, author_group_splits()) {
        let block = unwrap_alignment_environment(block);
        if block.is_empty() {
          continue;
        }
        // Drop empty `\\`-pieces up front. A group that BEGINS with `\\` — because
        // the previous author line ended with a trailing `\quad\\` (or `\and\\`) that
        // leaked the line-break into this group — would otherwise give an EMPTY
        // names_line, demoting the group's real first author to an affiliation. The
        // first NON-empty piece is the name list; the rest are affiliations. Witness
        // arXiv 2507.06670 (acl): `…Zhiyuan Zhu\quad \\ \textbf{Ruiqi Li}\quad…`
        // rendered "Ruiqi Li" as an empty `<personname/>` + a bold "Ruiqi Li"
        // affiliation; now "Ruiqi Li" is an author. (A `\\`-only block still yields a
        // single empty author below, so its affiliations are not dropped.)
        let lines: Vec<Tokens> = split_tokens(block, author_line_breaks())
          .into_iter()
          .filter(|l| !l.is_empty())
          .collect();
        for (names_line, affils) in name_groups(lines) {
          let mut names = split_names(names_line);
          if names.is_empty() {
            names.push(Tokens::default());
          }
          let last = names.len() - 1;
          for (i, name) in names.into_iter().enumerate() {
            let mut body: Vec<Token> = name.unlist();
            if i == last {
              for line in &affils {
                let (line, annotations) = if cautious {
                  take_author_annotations(line)
                } else {
                  (line.clone(), Vec::new())
                };
                if !line.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
                  // A bare email line (\texttt{user@host}) is an email, not an
                  // affiliation (OXIDIZED_DESIGN #52; witness arXiv:2606.00315).
                  let cs = if line_is_email(&line) {
                    T_CS!("\\lx@add@email")
                  } else {
                    T_CS!("\\lx@add@affiliation")
                  };
                  body.extend(Invocation!(cs, vec![None, Some(line)]).unlist());
                }
                body.extend(annotations);
              }
            }
            calls.extend(
              Invocation!(T_CS!("\\lx@add@author"), vec![
                keyvals.clone(),
                Some(Tokens::new(body))
              ])
              .unlist(),
            );
          }
        }
      }
    }
    Ok(Tokens::new(calls))
  }
  DefMacro!("\\lx@add@authors{}", sub[(stuff)] { add_authors_calls(stuff, true) });
  // A class's own author command, called once per author or author group and each adding to the ones before
  // (informs4.cls:1198 `\AUTHOR{Ann Able$^{a}$, Bob Baker$^{b}$}`): the author-line parsing of `\lx@add@authors`
  // without its replacing, so the names' superscript marks request the affiliations labelled with them.
  // A per-author class's `\author` (amsart, aastex, revtex) parses its body as one more author block too: a name list
  // there names several people (`\author{Klemens Fellner and Bao Quoc Tang}`, 1708.01427; aastex's
  // `Name\altaffilmark{1}, Name\altaffilmark{2}`, 1010.1318), and a `\\` line under a name is its affiliation
  // (`\author{Masao Ishikawa\\ \small Faculty of Education, …}`, math0606082). The optional keyvals go to each author
  // (revtex's `annotations=`).
  DefMacro!("\\lx@add@authors@append[]{}", sub[(keyvals, stuff)] {
    add_authors_calls_sectioned(stuff, false, false, keyvals, true)
  });
  // A class's own author-list command (informs4.cls:1198 `\AUTHOR{Ann Able$^{a}$, Bob Baker$^{b}$}`, documented as a
  // name list): appended like `\lx@add@authors@append`, but split at every separator as the article `\author` is.
  DefMacro!("\\lx@add@authors@list{}", sub[(stuff)] {
    add_authors_calls_sectioned(stuff, false, true, None, false)
  });
  // A name list where a binding has no lines or affiliations to read: each name `split_author_line` finds (comma,
  // " and ", " \& ") is an author of its own, the keyvals given to each (`\author{R. Braun\inst{1} and W. B.
  // Burton\inst{2}}`, astro-ph9810433; `\IEEEauthorblockN{Clemens Paul Zengler, Niels Troldborg, Mac Gaunaa.}`,
  // 2410.19527).
  DefMacro!("\\lx@add@author@split[]{}", sub[(keyvals, names)] {
    let mut calls: Vec<Token> = Vec::new();
    for name in split_author_line(names) {
      calls.extend(Invocation!(T_CS!("\\lx@add@author"), vec![keyvals.clone(), Some(name)]).unlist());
    }
    Ok(Tokens::new(calls))
  });
  // That class's affiliation command, under its authors (informs4.cls:1199 `\AFF{$^a$Univ A}`): when the queued authors
  // carry marks, its marked lines are affiliations labelled by their marks, which go to the authors requesting them
  // (`affiliation_calls`; 2609.17368, 22690, 28084); otherwise it is one affiliation, of the last author or as the
  // optional attributes place it (apa7's unnumbered list, `annotate=all`).
  DefMacro!("\\lx@add@affiliation@marked[]{}", sub[(attr, stuff)] {
    // (before any author, the marks are labels for the authors to come: an affiliation without its label had no one
    // to go to)
    if (queued_creators_have_marks() || queued_creator_count() == 0)
      && position_of(&stuff, &authorsup_markers()).is_some()
    {
      Ok(Tokens::new(affiliation_calls(None, None, stuff, true, true)?))
    } else {
      Ok(Invocation!(T_CS!("\\lx@add@affiliation"), vec![attr, Some(stuff)]))
    }
  });
  DefMacro!("\\lx@ijcai@names{}", sub[(stuff)] { add_authors_calls_sectioned(stuff, true, false, None, false) });

  // Shared "sectioned author block" machinery for the IJCAI author idiom
  // (ijcai97.sty and its derivatives): one `\author{}` holding names, then
  // `\affiliations`, then a comma-separated `\emails` list. Used both by the
  // `ijcai_sty` binding's `\author` override and by the `\lx@add@authors`
  // marker-branch above (so raw-loaded derivatives like ttm.sty work too).
  // `\lx@ijcai@authorsplit` reads the names up to `\affiliations` and runs them
  // through `\lx@ijcai@names` (`\lx@add@authors` without the marker branch), then splits the remainder into
  // affiliations (up to `\emails`) and the comma-separated emails, attaching the
  // n-th email to the n-th author. Ported from Perl ijcai.sty.ltxml (PR #2767).
  DefMacro!(
    "\\lx@ijcai@authorsplit Until:\\affiliations Until:\\done",
    "\\lx@ijcai@names{#1}\\ifx.#2.\\else\\lx@ijcai@affilsplit#2\\emails\\affiliations\\done\\fi"
  );
  DefMacro!(
    "\\lx@ijcai@affilsplit  Until:\\emails Until:\\affiliations Until:\\done",
    "\\ifx.#1.\\else\\expandafter\\lx@ijcai@affiliations\\expandafter{\\lx@strip@braces{#1}}\\fi\\ifx.#2.\\else\\expandafter\\lx@ijcai@emails\\expandafter{\\lx@strip@braces{#2}}\\fi"
  );
  DefMacro!(
    "\\lx@ijcai@affiliations{}",
    "\\lx@add@affiliations[labelseq=author]{#1}"
  );
  DefMacro!(
    "\\lx@ijcai@emails{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=email]\\lx@splitting{\\lx@ijcai@email}{,}{#1}"
  );
  DefMacro!("\\lx@ijcai@email{}", "\\lx@add@email[labelseq=author]{#1}");

  // Superscript markers in author/affiliation blocks carry the affiliation
  // number. Both sides use the "affiliation" prefix so the author's requested
  // annotation ("affiliation:N") exactly matches the affiliation's label
  // ("affiliation:N") in relocate_annotations -- avoiding the prefix-stripped
  // fallback that would otherwise conflate them with a creator's own "author:N"
  // sequence label and double-attach.
  // OXIDIZED_DESIGN #129 (html_feedback#1021, arXiv:2403.11905): these two are
  // `\let` onto `^`/`\textsuperscript` inside an author/affiliation line, so they
  // must consume their operand the way a real math superscript does — grabbing a
  // FULL nucleus (`\text{...}` kept with its group, any `$...$` nested inside it
  // captured whole+undigested). The old `[affiliation]{}` read grabbed only the
  // leading `\text`, orphaning its `{...}`; inside inline math that stray group
  // left a brace-group frame on top and the closing `$` fired
  // `\lx@end@inline@math` against it ("Attempt to end mode math"), arbitrarily
  // deep for `$^\text{$...$}$` markers. Both engines erred (SHARED-FAILURE);
  // reading the whole operand keeps the surrounding math balanced.
  DefPrimitive!(T_CS!("\\lx@sup@request@affiliation"), None, {
    let operand = read_frontmatter_sup_operand()?;
    let label = clean_frontmatter_labels(&operand.to_string(), "affiliation").join(",");
    with_pending_entry_attr(move |attr| {
      let labels = attr.get("_annotations").cloned().unwrap_or_default();
      let newval = if labels.is_empty() {
        label.clone()
      } else {
        s!("{labels},{label}")
      };
      DebugFeature!("frontmatter", "FRONT add annotation label {label}");
      attr.insert("_annotations".to_string(), newval);
    });
    Ok(Vec::new())
  });
  DefPrimitive!(T_CS!("\\lx@sup@setlabel@affiliation"), None, {
    let operand = read_frontmatter_sup_operand()?;
    // A line's first mark is its label; a later superscript in it is its text, shown (`$^{2}$Laboratory for
    // $^{3}$He`: the isotope's, 63i review; repro affiliation_line_inner_superscript_is_its_text).
    let mut labelled = false;
    with_pending_entry_attr(|attr| labelled = attr.contains_key("_bymark"));
    if labelled {
      return Ok(vec![digest(Invocation!(
        T_CS!("\\lx@frontmatter@keepsup"),
        vec![Some(operand)]
      ))?]);
    }
    let label = clean_frontmatter_labels(&operand.to_string(), "affiliation")
      .into_iter()
      .next();
    // the marks it is (`\mathrm{a}`: a), as the authors' are read (relocate_annotations)
    let keys = script_marks(&operand.to_string()).join(",");
    with_pending_entry_attr(move |attr| match label {
      Some(label) => {
        DebugFeature!("frontmatter", "FRONT set label {label}");
        attr.insert("_annotations".to_string(), label);
        // labelled by a mark, which the authors showing it answer (relocate_annotations)
        if !keys.is_empty() {
          attr.insert("_bymark".to_string(), keys);
        }
      },
      None => {
        attr.remove("_annotations");
      },
    });
    Ok(Vec::new())
  });
  // `\lx@let@superscript\cs` — `\let` the superscript catcode's definition key
  // (the `^` a digested SUPER token runs) to `\cs`, locally. Perl
  // (Base_Utility.pool.ltxml:729-737) writes `\let^\cs`, but `\let` names its
  // target with tex.web §1215 `get_r_token`, which rejects a character that is
  // not active ("Missing control sequence inserted"); the frontmatter idiom
  // below needs the SUPER key itself, which no TeX name reaches.
  DefPrimitive!("\\lx@let@superscript Token", sub[(cs)] {
    Let!(T_SUPER!(), cs);
  });
  // The plain meanings of the superscript and `\textsuperscript`, which an author or affiliation line rebinds to its
  // mark readers: saved where it rebinds them (the first time, so a line inside another keeps the outer save) and
  // restored for a note's content (`\lx@frontmatter@plainsups`), whose superscripts are the author's text, not marks
  // — a `\thanks{... 10 m$^{2}$ lab.}` in a marked author line lost its note to an orphaned `affiliation:2` label
  // (KNOWN_PERL_ERRORS #515, where Perl's Semiverbatim `\thanks` hid them; repro
  // sectioning-frontmatter/author_thanks_and_footnote_keep_their_math).
  DefPrimitive!(T_CS!("\\lx@frontmatter@savesups"), None, {
    // Saved once per group chain, keyed on the superscript (`\textsuperscript` is undefined under plain TeX).
    if lookup_meaning(&T_CS!("\\lx@frontmatter@plainsup")).is_none() {
      // The definition a superscript digests by (a `\let` from `^` copies only the character).
      if let Some(plain) = lookup_digestable_definition(&T_SUPER!()) {
        assign_meaning(&T_CS!("\\lx@frontmatter@plainsup"), plain, None);
      }
      Let!(
        T_CS!("\\lx@frontmatter@plaintextsuperscript"),
        T_CS!("\\textsuperscript")
      );
    }
    Ok(Vec::new())
  });
  // A note's text is typeset as a footnote's, under `\@footnotetext`'s `\@parboxrestore` (latex.ltx:17663, 16288),
  // which also gives `\\` back its plain meaning: in a `p{}` cell of an author tabular it ended the row in the middle
  // of `\thanks{Corresponding author.\\ Code: none.}` (2609.32661).
  DefPrimitive!(T_CS!("\\lx@frontmatter@plainsups"), None, {
    Let!(T_CS!("\\\\"), T_CS!("\\@normalcr"));
    if let Some(plain) = lookup_meaning(&T_CS!("\\lx@frontmatter@plainsup")) {
      assign_meaning(&T_SUPER!(), plain, None);
    }
    if is_defined_token(&T_CS!("\\lx@frontmatter@plaintextsuperscript")) {
      Let!(
        T_CS!("\\textsuperscript"),
        T_CS!("\\lx@frontmatter@plaintextsuperscript")
      );
    }
    Ok(Vec::new())
  });
  DefMacro!(
    "\\lx@author@withsup{}",
    "\\bgroup\\lx@frontmatter@savesups\\lx@let@superscript\\lx@sup@request@affiliation\\let\\textsuperscript\\lx@sup@request@affiliation#1\\egroup"
  );
  DefMacro!(
    "\\lx@affiliation@withsup{}",
    "\\bgroup\\lx@frontmatter@savesups\\lx@let@superscript\\lx@sup@setlabel@affiliation\\let\\textsuperscript\\lx@sup@setlabel@affiliation#1\\egroup"
  );
  // A VISIBLE author superscript mark that must survive the `\lx@author@withsup`
  // hijack (which points `^`, via `\lx@let@superscript`, and `\textsuperscript`
  // at the affiliation-linker).
  // `rewrite_symbol_superscripts` rewrites footnote-SYMBOL marks (`$^{*}$`,
  // `\textsuperscript{\dagger}` — equal-contribution / corresponding-author
  // notes, never affiliation numbers) onto this sentinel BEFORE author parsing,
  // so they neither trigger the affiliation-marker branch nor get consumed into
  // an unmatched `affiliation:*` label and dropped. Renders identically to
  // `\textsuperscript` but under a name the hijack does not touch.
  // OXIDIZED_DESIGN #52; witness arXiv:2506.06941 (Mirzadeh's `$^{*}$`).
  DefConstructor!("\\lx@frontmatter@keepsup{}", "<ltx:sup>#1</ltx:sup>", mode => "text");
  // A footnote-symbol author mark (`$^{*}$`, `\inst{\dagger}`), shown by `\lx@frontmatter@keepsup`, also requests
  // the affiliation of its symbol, as a lettered mark does: a list labelling an entry with it (informs4's
  // `\AFF{$^*$Corresponding author}`, 2609.38842) gives that entry to its authors; a request nothing answers is not
  // used. Only on a creator: in an affiliation the label is its own (`\lx@affiliation@withsup`).
  DefPrimitive!("\\lx@frontmatter@symbolmark{}", sub[(sym)] {
    let label = clean_frontmatter_labels(&sym.to_string(), "affiliation").join(",");
    with_pending_entry_attr(move |attr| {
      if attr.get("role").map(String::as_str) != Some("pending") && !label.is_empty() {
        let labels = attr.get("_annotations").cloned().unwrap_or_default();
        let newval = if labels.is_empty() { label.clone() } else { s!("{labels},{label}") };
        attr.insert("_annotations".to_string(), newval);
      }
    });
  });

  DefMacro!("\\lx@add@affiliations[]{}", sub[(attr, stuff)] {
    dequeue_front_matter("ltx:contact", &[("role", "affiliation")]);
    Ok(Tokens::new(affiliation_calls(attr, None, stuff, true, true)?))
  });

  DefMacro!("\\lx@date@received@name", "Received~");
  DefMacro!("\\lx@date@revised@name", "Revised~");
  DefMacro!("\\lx@date@accepted@name", "Accepted~");
  DefMacro!("\\lx@date@draft@name", "Drafted~");
  DefMacro!("\\lx@date@posted@name", "Posted~");
  DefMacro!("\\lx@date@copyright@name", "\u{A9} ");

  DefMacro!("\\lx@pubnote@type@name", "Publication type:~");
  DefMacro!("\\lx@pubnote@note@name", "Note:~");
  DefMacro!("\\lx@pubnote@pubid@name", "PubID:~");
  DefMacro!("\\lx@pubnote@doi@name", "DOI:~");
  DefMacro!("\\lx@pubnote@isbn@name", "ISBN:~");
  DefMacro!("\\lx@pubnote@arxiv@name", "arXiv:~");
  DefMacro!("\\lx@pubnote@preprint@name", "Preprint:~");
  DefMacro!("\\lx@pubnote@journal@name", "Journal:~");
  DefMacro!("\\lx@pubnote@conference@name", "Conference:~");
  DefMacro!("\\lx@pubnote@issue@name", "Issue:~");
  DefMacro!("\\lx@pubnote@volume@name", "Volume:~");
  DefMacro!("\\lx@pubnote@dedication@name", "Dedication:~");
  DefMacro!("\\lx@pubnote@thanks@name", "Thanks:~");

  DefMacro!("\\lx@abstract@name", "Abstract");
  DefMacro!("\\lx@keywords@name", "Keywords:~");
  DefMacro!("\\lx@classification@name", "Classification:~");

  DefMacro!("\\lx@contact@affiliation@name", "Affiliation:~");
  DefMacro!(
    "\\lx@contact@altaffiliation@name",
    "Alternate Affiliation:~"
  );
  DefMacro!("\\lx@contact@address@name", "Address:~");
  DefMacro!("\\lx@contact@email@name", "Email:~");
  DefMacro!("\\lx@contact@url@name", "URL:~");
  // Empty: an ORCID contact renders the iD glyph (\lx@orcidlogo), which already
  // identifies itself — so it carries no textual "OrcID:" label (get_frontmatter_name
  // maps an empty per-role default to no label at all).
  DefMacro!("\\lx@contact@orcid@name", "");
  // A row of an author block appended to `\@author` (`\lx@author@tail`): printed as the class typeset it,
  // with no label.
  DefMacro!("\\lx@contact@authorblock@name", "");
  DefMacro!("\\lx@contact@note@name", "Note:~");
  DefMacro!("\\lx@contact@thanks@name", "Thanks:~");
  DefMacro!("\\lx@contact@correspondent@name", "Corresponding author:~");

  //======================================================================
  // This is called by afterOpen (by default on <ltx:document>) to
  // output any frontmatter that was accumulated.

  // Add a annotation target based on the name for fuzzy matching of annotations.
  Tag!("ltx:creator", after_close => sub[document, creator] {
    if let Some(person) = document.findnode("ltx:personname", Some(&*creator)) {
      let label = clean_label(&person.get_content(), Some("fuzzy")).into_owned();
      let labels = creator.get_attribute("_annotations").unwrap_or_default();
      let value = if labels.is_empty() { label } else { s!("{labels},{label}") };
      document.set_attribute(creator, "_annotations", &value)?;
    }
  });

  // Add FrontMatter at document begin, unless deferred to a better position.
  Tag!("ltx:document", after_open_late => sub[document,_root] {
    if !lookup_bool("frontmatter_deferred") {
      insert_frontmatter(document)?;
    }
  });

  // A `\maketitle` that typesets authors an `\author` set after the last flush replaces
  // the ones that flush digested (`add_authors_calls`; DIVERGENCES #406). First in
  // `\lx@maketitle@body`: the class's stores (`\@address`, `\@email`, which `\@maketitle`
  // reads again) are handed to the new authors by the `\lx@store@defaults` after it.
  DefPrimitive!("\\lx@maketitle@supersede", sub[_args] {
    if lookup_bool("lx_authors_superseded") && queued_author_count() > 0 {
      assign_value("lx_authors_superseded", Stored::Bool(false), Some(Scope::Global));
      if supersede_digested_authors() {
        assign_value("lx_stores_harvested", Stored::Bool(false), Some(Scope::Global));
      }
    }
    Ok(())
  });

  // Request Frontmatter to appear HERE (if not already done),
  // deferring it from document begin.
  // This should be where ALL the digestion of frontmatter happens
  DefConstructor!("\\lx@frontmatterhere", sub[doc,_args] {
    // The `\maketitle` flush: the one placement pass (OXIDIZED_DESIGN #247). A
    // content-free lead (a `\clearpage`/`\frontmatter` pagebreak, an empty box
    // paragraph) ends up behind the frontmatter; a built <titlepage> ahead of it
    // anchors it (a superset of #242, which placed at the current point when a
    // pagebreak sat between the titlepage and `\maketitle` — no corpus doc has that
    // shape; the anchored tree is the valid one); preamble residue ahead of it moves
    // behind it, and visible body content ahead of it stays in order after the
    // head-placed frontmatter (#262; Perl's current-point placement stranded the
    // title — amsldoc/tkz-doc/tuda-ci/tzplot and the logo-cover controls).
    place_frontmatter(doc, false, FrontmatterAnchor::LeadingFrontmatter)?;
  },
  after_digest => {
    digest_front_matter()?;
    assign_value("frontmatter_deferred", true, Some(Scope::Global));
  });

  // Same, but put it at the beginning of document, but after any ltx:resources
  DefConstructor!("\\lx@frontmatter@fallback", sub[document,_args] {
    // The fallback flushes queued frontmatter when there is no \maketitle (triggered
    // by the first \section via \@startsection@hook, or at document end). Perl (post
    // PR#2767 Frontmatter API) always inserts it at the document TOP (after
    // ltx:resource), and so do we — in BOTH branches. The ABSTRACT-ONLY case (exactly
    // the one `insert_frontmatter` defers, see its "defer until abstract's document
    // location" branch) first recovers a hand-formatted title: a manual \begin{center}
    // "title" block preceding \begin{abstract} with no \maketitle (arXiv 1609.07638)
    // was deposited as ordinary body BEFORE the abstract was queued, so a blind
    // top-flush would float the abstract ABOVE the title. `maybe_promote_leading_title`
    // turns that block into a real <ltx:title> when it is unambiguous, the rest of the
    // hand-typeset cover is wrapped as <ltx:titlepage> (`place_frontmatter (the titlepage wrap)`),
    // and the flush lands right after them (`insert_frontmatter_at`); with
    // neither, the flush goes to the top. An earlier
    // rescue flushed the abstract at the CURRENT position instead — which, when the
    // body had emitted a cover block/TOC/pagebreak by then, left `<abstract>` trailing
    // body content, schema-invalid and RUST-ONLY (#246). Title/author/date frontmatter
    // (e.g. a preamble \title with no \maketitle — tests/digestion/rebox) was registered
    // before any body and keeps the top-of-document insertion as before.
    let abstract_only = with_value("frontmatter", |v| match v {
      Some(Stored::HashTagData(frnt)) => frnt.len() == 1 && frnt.contains_key("ltx:abstract"),
      _ => false,
    });
    if abstract_only {
      // No \title/\author/\maketitle but a leading hand-formatted display block
      // may BE the title (arXiv 1609.07638) — promote it to <ltx:title> first.
      // Whatever else was hand-typeset before the first \section is title-page LAYOUT
      // and is wrapped as <ltx:titlepage> in its exact order; the abstract then follows
      // the titlepage (or the bare promoted title), and with neither it goes to the TOP
      // like Perl (Base_Utility.pool.ltxml:927-945).
      // Flushing at the CURRENT point (the first \section or \end{document}) put the
      // <abstract> after every cover block, TOC and pagebreak the body had emitted by
      // then — schema-invalid, and later than the source order, RUST-ONLY
      // (tikz-mirror-lens, tipfr-doc, pgf-interference, axodraw2-man, derivative,
      // russ_doc, schulmathematik, jourcl, isosigns-docs, bootstrapicons-docs — 16
      // s106 docs; OXIDIZED_DESIGN #246).
      // Only on the FIRST flush: once frontmatter has been placed (`\maketitle`), a
      // later abstract-only queue is a LATE abstract — what precedes its marker is
      // body, not cover, and `insert_late_frontmatter` places it beside the
      // existing frontmatter. (`frontmatter_done` is reset by that path itself.)
      // First flush only: promote an unambiguous hand-made title, then let the pass
      // wrap the rest of the cover (before the abstract's marker) as <titlepage>. A
      // LATE abstract (after `\maketitle`) is filed by `insert_late_frontmatter`.
      let first_flush = !lookup_bool("frontmatter_done");
      if first_flush {
        maybe_promote_leading_title(document)?;
      }
      place_frontmatter(document, first_flush, FrontmatterAnchor::LeadingFrontmatter)?;
    } else {
      place_frontmatter(document, false, FrontmatterAnchor::Top)?;
      maybe_dedup_leading_title_ink(document)?;
    }
  },
  after_digest => {
    // A raw class's stores the document set, read where the frontmatter is, before its queue is
    // digested (frontmatter_stores.rs): the authors' marks are still there to match.
    crate::frontmatter_stores::harvest_stores(false)?;
    digest_front_matter()?;
    assign_value("frontmatter_deferred", true, Some(Scope::Global));
  });

  // Maintain a list of classes that apply to the document root.
  // This might involve global style options, like leqno.
  Tag!("ltx:document", after_open_late => sub[document, root] {
    let classes = with_mapping_keys("DOCUMENT_CLASSES", |keys| join(&keys," "));
    if !classes.is_empty()  {
      document.add_class(root, &classes)?;
    }
  });

  //======================================================================
  // Tags & Titles
  // The reference numbers, titles, captions etc, for various objects have
  // different styling conventions, and the styling various depending on context.
  // We'll use ltx:tags as a container for the various forms of ltx:tag with different @role's.
  // The role=refnum form is simply formatted by \the<counter> and used by \ref;
  // An ltx:tag w/o @role are for the numbers, often formatted differently, which
  // appear alongside the object; Such a tag also may be embedded within the title or caption.
  // Cross-references automatically generated by LaTeXML benefit from a bit more context:
  // these are the role=typerefnum forms.
  // Additional forms are needed for bibliographies, hyperref's autoref, etc.
  // An additional complication is that while the "type" determines the formatting
  // of the various forms, some types (eg. theorems) share the same counter.
  // LaTeX defines this handling on an adhoc basis; defines \fnum@table, \fnum@figure for some types
  // but \labelenumi, etc for others.

  // This section synthesizes a more uniform support for reference numbers,
  // references to reference numbers, title formatting etc.
  // It allows you to customize each of the forms for each type encountered.
  // The design reflects LaTeX needs, more than TeX, but support starts here!

  // This collects up the various declared ltx:tag's into an ltx:tags
  DefMacro!("\\lx@make@tags {}", sub[(ttype)] {
    // Pull the (role -> formatter) pairs out of HashStored via with_value
    // so we don't clone the whole hashmap envelope; the per-role tokens
    // are Copy and the per-role String arm just dereferences a SymStr.
    let role_formatters: Vec<(String, Option<Token>)> = with_value(
      "type_tag_formatter",
      |v| match v {
        Some(Stored::HashStored(formatters)) => {
          let keys_sym: Vec<_> = formatters.keys().copied().collect();
          let mut sorted_keys: Vec<String> = with_many(&keys_sym, |keys| {
            keys.into_iter().map(str::to_owned).collect()
          });
          sorted_keys.sort();
          sorted_keys
            .into_iter()
            .map(|role| {
              let ft = match formatters.get(&role) {
                Some(Stored::Token(t)) => Some(*t),
                Some(Stored::String(sym)) => {
                  Some(Token!(sym *sym, Catcode::CS))
                },
                _ => None,
              };
              (role, ft)
            })
            .collect()
        }
        _ => Vec::new(),
      },
    );
    let mut tags = Vec::new();
    for (role, formatter_opt) in role_formatters {
      if let Some(formatter_token) = formatter_opt {
        // A tag only a reference prints (`type_tag_deferred`) holds its diagnostics (`\lx@tag@intags@held`).
        let constructor = if with_mapping("type_tag_deferred", &role, |held| held.is_some()) {
          "\\lx@tag@intags@held"
        } else {
          "\\lx@tag@intags"
        };
        tags.push((role.is_empty(), Invocation!(T_CS!(constructor),
          vec![
            Tokens!(T_OTHER!(role.as_str())),
            build_invocation(formatter_token, vec![Some(ttype.clone())])?
          ])
        ));
      }
    }

    // A tag a reference prints (refnum, typerefnum, autoref, the cleveref forms) is built where its target is (an
    // eqnarray row's in its math group, where a document may rebind `~`: vdm.sty:135-138, 202-205
    // `\everymath{\let~\hook}`, a `\vbox{\ialign…}` that re-stepped the equation, an endless recursion on 1601.02132)
    // and printed in text. Its `~` is the kernel's no-break space, LaTeX's `\nobreakspace`, which babel's `~` also is
    // except in a language that makes it a shorthand; such a shorthand, or a document's own text `~`, no longer reaches
    // these tags (OXIDIZED_DESIGN_DIVERGENCES #98 addendum). The role-less tag is the target's own label, printed where
    // it is built (`\item[Espa~na]`), and keeps the document's `~`. Guard
    // `perfect_kernel_batch63::autoref_tag_tilde_is_the_kernel_space`.
    let mut lx_tags = vec![T_CS!("\\lx@tags"), T_BEGIN!()];
    for (printed_here, invoked_tag) in tags {
      if printed_here {
        lx_tags.extend(invoked_tag.unlist());
      } else {
        lx_tags.extend([T_CS!("\\begingroup"), T_CS!("\\let"), T_ACTIVE!('~'), T_CS!("\\lx@tag@texttilde")]);
        lx_tags.extend(invoked_tag.unlist());
        lx_tags.push(T_CS!("\\endgroup"));
      }
    }
    lx_tags.push(T_END!());
    Ok(Tokens::new(lx_tags))
  });
  DefMacro!("\\lx@tag@texttilde", "\\lx@NBSP", protected => true);

  // Remove the last closed node, if it's empty.
  let remove_empty_element: Vec<ConstructionClosure> = construct!(document, _whatsit, {
    if let Some(node) = document.get_node().get_last_child() {
      // This should be the wrapper just added.
      if node.get_first_child().is_none() {
        document.remove_node(node);
      }
    }
  });

  // \lx@tag[open][close]{stuff}
  let remove_empty_element_1 = remove_empty_element.clone();
  DefConstructor!("\\lx@tag[][][]{}", "<ltx:tag open='#1' close='#2'>#4</ltx:tag>",
    mode => "restricted_horizontal",
    after_construct => remove_empty_element_1
  );

  // \lx@tag@intags{role}{stuff}
  // The role is an identifier (`refnum`, `autoref`, …), read undigested — Perl digests `[]` (Base_Utility.pool.ltxml:
  // 1001) after `neutralizeFont`, whose OT1 leaves ASCII alone; the neutral font here keeps the document's encoding
  // (`neutralize_font`), and under babel greek's LGR (`\selectlanguage{greek}`, greek.ldf:151-153) a digested `refnum`
  // read back `ρεφνυμ`. Guard `perfect_kernel_batch60::identifiers_are_not_font_decoded`.
  let remove_empty_element_2 = remove_empty_element.clone();
  DefConstructor!("\\lx@tag@intags OptionalUndigested {}", "<ltx:tag role='#1' _deferred='#deferred'>#2</ltx:tag>",
    mode => "restricted_horizontal",
    before_digest => sub { neutralize_font(); },
    after_construct => remove_empty_element_2
  );
  // A tag typeset only where a reference prints it — hyperref's `autoref` name (`type_tag_deferred`): TeX expands
  // `\<type>autorefname` only in `\autoref` (hyperref.sty:8202-8278), so the name's diagnostics belong there. Built
  // here, at the target (ruling 2026-10-02 (A)), its diagnostics are held and deferred (ruling 7e,
  // logger.rs `DeferredDiagnostics`): the tag carries their id (`_deferred`), the labelled node ties it to its labels
  // as it closes (sect11.rs `tie_held_tags`), and they are replayed once the document is built if a reference shows
  // the tag's role (`replay_shown_deferred`). Witness biblatex-gost-examples (hyperref's Russian `\cyr…` names under TU).
  // Repro singletons/autoref_name_evaluated_at_every_target.
  DefPrimitive!("\\lx@tag@intags@held[]{}", sub[(role, stuff)] {
    let role = role.unwrap_or_default();
    let hold = util::logger::DiagnosticsHold::begin();
    let tag = digest(Invocation!(T_CS!("\\lx@tag@intags"), vec![role, stuff]))?;
    if hold.is_empty() {
      hold.commit();
    } else if let Some(whatsit) = held_tag_whatsit(&tag)
      && let DigestedData::Whatsit(w) = whatsit.data()
    {
      if let Some(deferred) = hold.defer() {
        let id = util::logger::keep_deferred(deferred);
        w.borrow_mut().set_property("deferred", Stored::String(pin(id.to_string())));
      }
    } else {
      // A tag with no whatsit to carry the id raises its diagnostics here: never lost.
      hold.commit();
    }
    Ok(vec![tag])
  });
  DefConstructor!("\\lx@tags{}","<ltx:tags>#1</ltx:tags>",
    after_construct => remove_empty_element
  );

  //----------------------------------------------------------------------
  // "refnum" is the lowest level reference number for an object is typically \the<counter>
  // but be sure to use the right counter!  This is how \ref will show the number.
  // You'll typically customize this by defining \the<counter> (and \p@<counter) as in LaTeX.
  DefMacro!("\\lx@counterfor{}", sub[(ctr_type)] {
    with_mapping("counter_for_type", &ctr_type.to_string(), |ctr_opt|
    if let Some(ctr) = ctr_opt {
      Tokens!(T_OTHER!(ctr.to_string()))
    } else {
      ctr_type
    })
  });
  DefMacro!(
    "\\lx@the@@{}",
    "\\expandafter\\lx@@the@@\\expandafter{\\lx@counterfor{#1}}"
  );
  DefMacro!("\\lx@@the@@{}", "\\csname the#1\\endcsname");

  DefMacro!(
    "\\lx@therefnum@@{}",
    "\\expandafter\\lx@@therefnum@@\\expandafter{\\lx@counterfor{#1}}"
  );
  // `\expandafter` before the first `\endcsname`: ctex's `\labelformat`
  // makes `\p@section` ARGUMENT-TAKING (`\p@section#1->\CTEX@thesection`,
  // ctex-heading-article.def:747) and patches every kernel `\p@#1\the#1`
  // site to that shape (:770-771) — a site it cannot reach here let
  // `\p@section` swallow the following `\csname` ("Extra \endcsname" on
  // every heading: caspervector 23, sduthesis 36, tabular2 28, inkpaper-en;
  // Perl Base_Utility.pool.ltxml:1028 identical, KPE #149). Guard:
  // `perfect_kernel_batch54::ctex_argument_taking_p_macro_keeps_the_refnum`.
  DefMacro!("\\lx@@therefnum@@{}", "{\\normalfont\\lx@@p@the@@{#1}}");
  // `\p@<ctr>\the<ctr>` in latex.ltx:14976's exact shape, resolved through
  // `counter_for_type` like `\lx@the@@`. Shared by the refnum and typerefnum
  // formatters so an argument-taking `\p@<ctr>` (`\labelformat`, latex.ltx:14978)
  // always receives the single `\the<ctr>` token: the typerefnum's former
  // `\csname p@#1\endcsname\lx@the@@{#1}` handed it `\lx@the@@` and left
  // `{sentence}` behind ("You can't use } after \the" on every contract.sty
  // sentence; Perl Base_Utility.pool.ltxml:1080-1084 identical, KPE #160).
  // Guard: `perfect_kernel_batch54::labelformat_is_a_kernel_macro`.
  DefMacro!(
    "\\lx@p@the@@{}",
    "\\expandafter\\lx@@p@the@@\\expandafter{\\lx@counterfor{#1}}"
  );
  DefMacro!(
    "\\lx@@p@the@@{}",
    "\\csname p@#1\\expandafter\\endcsname\\csname the#1\\endcsname"
  );

  AssignMapping!("type_tag_formatter", "refnum" => "\\lx@therefnum@@");

  //----------------------------------------------------------------------
  // \lx@fnum@@{type}  Gets the formatted form of the refnum, as part of the object, (no @role).
  // Customize by defining \fnum@<type> or \<type>name and \fnum@font@<type>
  // Default uses \fnum@font@<type> \<type>name prefix + space (if any) and \the<counter>.
  // When using the "name", uses \<type>name in preference to fallback \lx@name@<type>
  DefMacro!(
    r"\lx@refnum@compose{}{}",
    r"\expandafter\lx@refnum@compose@\expandafter{#2}{#1}"
  );
  DefMacro!(r"\lx@refnum@compose@{}{}", r"\if.#1.#2\else#2\space#1\fi");

  // The `{}` after `\endcsname` is OXIDIZED_DESIGN #85, a deliberate divergence
  // from Perl's otherwise byte-identical `Base_Utility.pool.ltxml` L1041-1043.
  // LaTeX's `\@makecaption` is `\sbox\@tempboxa{#1: #2}`, so the widely-copied
  // "Fig. 1: -> Fig. 1." hack —
  //   \renewcommand*{\fnum@figure}[1]{\figurename~\thefigure.}
  // — works under pdflatex by having the hook eat that `:` TOKEN. LaTeXML has
  // no `:` to eat: the separator is a tag ATTRIBUTE (`\lx@tag[][: ]`,
  // `latex_constructs.rs::format@title@figure`). So the argument scan ran past
  // the hook and swallowed the caption group's closing brace — the `<figure>`
  // never closed and every following section, bibliography included, was
  // absorbed into it. The empty group gives an arg-taking hook something
  // harmless to consume and is inert for the 0-arg hooks that are the normal
  // case. The `\lx@@fnum@@` branch below — what fires when no `\fnum@<type>`
  // exists at all, i.e. for nearly every caption — is untouched.
  // The `fnum@font@<type>` value WRAPS the number as a braced argument
  // (real enumitem: `\enit@format{<label>}`, enumitem.sty:451/1478) — a bare
  // prefix let an argument-taking font command (`\textbf`, `\meta`,
  // tcbdocumentation's `\docAuxKey`; `\setlist[description]{font=…}`) grab
  // the following `\@ifundefined` instead (non-decimal-units manual, 40
  // LR-mode/boxing errors; Perl Base_Utility.pool:1041 shares the bare
  // prefix). The group is transparent for the declaration-style values.
  DefMacro!(
    r"\lx@fnum@@{}",
    r"{\normalfont\@ifundefined{fnum@font@#1}{}{\csname fnum@font@#1\endcsname}{\@ifundefined{fnum@#1}{\lx@@fnum@@{#1}}{\csname fnum@#1\endcsname{}}}}"
  );

  // Really seems like <type>name should take precedence over \lx@name@<type>,
  // since users might define it.
  // BUT amsthm defines \thmname{}!
  //
  // `\<type>name` is a NAME NOUN only when it is an expandable macro whose
  // arguments are all optional (`\figurename`, `\chaptername`, babel captions). Perl
  // Base_Utility.pool.ltxml:1048 takes any defined `\<type>name`, so a package
  // that pairs counter `af` with the drawing command `\afname{…}`
  // (argumentation.sty:403, `\NewDocumentCommand{\afname}{…}{… \node …}`)
  // executes `\node` inside `\refstepcounter{af}` — SHARED Perl failure
  // (KPE #194); pdflatex never expands `\afname` there. Guard:
  // `perfect_kernel_batch56::counter_name_command_is_not_a_name_noun`.
  DefConditional!("\\iflx@namenoun Token", sub[(t)] {
    match lookup_definition(&t) {
      Ok(Some(def)) => is_name_noun(&def, 0),
      _ => false,
    }
  });
  // LaTeXML's own English reference names (`\footnotetyperefname` "footnote", `\itemtyperefname` "item", a binding's
  // `\<type>typerefname`) are Latin text: set in babel's `\latinencoding` (babel.sty:3915-3933 in TL2025 — TU under
  // fontspec, T1 when loaded, else OT1; what babel's `\textlatin` selects), else OT1, whatever the tag's encoding. In
  // a Greek region (`\encodingdefault` LGR) they read back `φοοτνοτε`, `ιτεμ`; a class's or babel's `\<type>name`
  // (greek.ldf's `\figurename` Σχήμα, in LGR-only symbols), a theorem's or float's name and the numerals (babel
  // greek's `\@alph`) stay in the tag's encoding, as LaTeX sets the label. Guard
  // `perfect_kernel_batch60::identifiers_are_not_font_decoded`.
  DefMacro!(
    "\\lx@latin@name{}",
    r"{\ifdefined\latinencoding\fontencoding{\latinencoding}\else\fontencoding{OT1}\fi\selectfont#1}"
  );
  DefMacro!(
    "\\lx@@fnum@@ {}",
    r"\@ifundefined{lx@name@#1}{\@ifundefined{#1name}{\lx@the@@{#1}}{\expandafter\iflx@namenoun\csname #1name\endcsname\lx@refnum@compose{\csname #1name\endcsname}{\lx@the@@{#1}}\else\lx@the@@{#1}\fi}}{\lx@refnum@compose{\csname lx@name@#1\endcsname}{\lx@the@@{#1}}}"
  );

  AssignMapping!("type_tag_formatter", "" => "\\lx@fnum@@"); // Default!

  //----------------------------------------------------------------------
  // \\lx@fnum@toc@{type} is similar, but formats the number for use within \\toctitle
  // Customize by defining \\fnum@toc@<type> or \\fnum@tocfont@<type>
  // Default uses just \\the<counter>, else composes using \\lx@@fnum@@{type}
  // Same `{}` as `\lx@fnum@@` above, same reason — OXIDIZED_DESIGN #85.
  // (`\lx@typerefnum@@` below has the identical *shape* but is deliberately NOT
  // changed: `typerefnum@<type>` is a LaTeXML-internal hook with no LaTeX
  // kernel behind it, so no author writes an arg-taking version to eat a
  // separator token that LaTeXML never emits. The divergence is justified by
  // pdflatex compatibility, and that argument does not reach this one.)
  DefMacro!(
    r"\lx@fnum@toc@@{}",
    r"{\normalfont\@ifundefined{fnum@tocfont@#1}{}{\csname fnum@tocfont@#1\endcsname}{\@ifundefined{fnum@toc@#1}{\lx@the@@{#1}}{\csname fnum@toc@#1\endcsname{}}}}"
  );

  //----------------------------------------------------------------------
  // "typerefnum" form is used by automatic cross-references, typically "type number" or similar.
  // Customize by defining \typerefnum@<type> or \typerefnum@font@<type>
  // Default uses either \<type>typerefname or \<type>name (if any, followed by space, then
  // \\the<counter>
  DefMacro!(
    "\\lx@typerefnum@@{}",
    r"{\normalfont\@ifundefined{typerefnum@font@#1}{}{\csname typerefnum@font@#1\endcsname}{\@ifundefined{typerefnum@#1}{\lx@@typerefnum@@{#1}}{\csname typerefnum@#1\endcsname}}}"
  );

  DefMacro!(
    "\\lx@@typerefnum@@{}",
    r"\@ifundefined{#1typerefname}{\@ifundefined{lx@name@#1}{\@ifundefined{#1name}{}{\expandafter\iflx@namenoun\csname #1name\endcsname\lx@refnum@compose{\csname #1name\endcsname}{\lx@p@the@@{#1}}\fi}}{\lx@refnum@compose{\csname lx@name@#1\endcsname}{\lx@p@the@@{#1}}}}{\lx@refnum@compose{\csname #1typerefname\endcsname}{\lx@p@the@@{#1}}}"
  );

  AssignMapping!("type_tag_formatter", "typerefnum" => "\\lx@typerefnum@@");

  //----------------------------------------------------------------------
  // The following macros provide similar customization for titles & toctitles
  // in particular for supporting localization for different languages.
  // Redefine these if you want to assemble the name (eg. \chaptername), refnum and titles
  // differently
  //----------------------------------------------------------------------
  // \lx@format@title@@{type}{title}
  // Format a title (or caption) appropriately for type.
  // Customize by defining \format@title@type{title}
  // Default composes \lx@fnum@@{type} space title.
  DefMacro!(
    "\\lx@format@title@@{}{}",
    r"\lx@@format@title@@{#1}{{\lx@format@title@font@@{#1}#2}}"
  );
  DefMacro!(
    "\\lx@@format@title@@{}{}",
    r"{\@ifundefined{format@title@#1}{\lx@@compose@title{\lx@fnum@@{#1}}{#2}}{\csname format@title@#1\endcsname{#2}}}"
  );

  // \\lx@format@toctitle@@{type}{toctitle}
  // Similar for toctitle, typically briefer
  // Customize by defining \\format@toctitle@type{title}
  // Default composes \\lx@fnum@toc@@{type} space title.
  // The toc copy is digested with notes, `\label`, `\index` neutralized
  // (`\lx@toc@copy@neutralize`, defined for LaTeX in latex_constructs_rust_only.rs; a no-op
  // without it), as LaTeX writes it to the .toc untypeset (DIVERGENCES #399; witness 2605.15775,
  // whose heading note took two numbers).
  DefMacro!("\\lx@toc@copy@neutralize", "");
  DefMacro!(
    "\\lx@format@toctitle@@{}{}",
    r"\lx@@format@toctitle@@{#1}{{\lx@toc@copy@neutralize\lx@format@toctitle@font@@{#1}#2}}"
  );

  DefMacro!(
    "\\lx@@format@toctitle@@{}{}",
    r"{\@ifundefined{format@toctitle@#1}{\lx@@compose@title{\lx@fnum@toc@@{#1}}{#2}}{\csname format@toctitle@#1\endcsname{#2}}}"
  );

  DefMacro!("\\lx@@compose@title{}{}", r"\lx@tag[][ ]{#1}#2");

  DefMacro!(
    r"\lx@format@title@font@@{}",
    r"\@ifundefined{format@title@font@#1}{}{\csname format@title@font@#1\endcsname}"
  );
  DefMacro!(
    r"\lx@format@toctitle@font@@{}",
    r"\@ifundefined{format@toctitle@font@#1}{}{\csname format@toctitle@font@#1\endcsname}"
  );

  // NOTE that a 3rd form seems desirable: an concise form that cannot rely on context for the type.
  // This would be useful for the titles in links; thus can be plain (unicode) text.

  //======================================================================
  // Normally definitions disappear; the macros are expanded or have their expected effect.
  // But in a few cases (eg tabular column definitions, or LaTeX \Declarexxxx)
  // they will need declarations in the (La)TeX preamble to allow (La)TeX to process snippets
  // (eg. math) in order to create images.
  // Returning a call to this utility from Primitives will add a preamble Processing Instruction

  // TODO
  // sub AddToPreamble {
  //   my ($cs, @args) = @_;
  //   return Digest(Invocation(T_CS('\lx@add@Preamble@PI'), Invocation((ref $cs ? $cs : T_CS($cs)),
  // @args))); }

  // Perl: DefConstructor('\lx@add@Preamble@PI Undigested', "<?latexml preamble='#1'?>");
  // PI syntax not supported in constructor templates, so use procedural body.
  DefConstructor!("\\lx@add@Preamble@PI Undigested",
    sub[document, args, _props] {
      if let Some(Some(preamble_arg)) = args.first() {
        let preamble_text = preamble_arg.untex()?;
        if !preamble_text.is_empty() {
          let mut attrs = HashMap::default();
          attrs.insert(String::from("preamble"), preamble_text);
          document.insert_pi("latexml", Some(attrs))?;
        }
      }
    }
  );
});

// is_definable — defined in latexml_core::binding::def::dialect, re-exported via prelude.
// Check if a token is "definable" — undefined or equivalent to `\relax`.
// Port of Perl `isDefinable($token)` (Base_Utility.pool.ltxml L33-40).

// split_tokens is defined below (moved from base_functions.rs) — includes meaning-based matching.
// Perl: SplitTokens($tokens, @delims) — Base_Utility.pool.ltxml L106-132.
// Splits a token list by delimiter tokens, respecting brace nesting and math mode.

/// The `\lx@add@affiliation` calls for an affiliation list: one per line or `\and`-separated
/// entry, each with its superscript marker as its label when the list has markers (Perl
/// Base_Utility.pool.ltxml:742-753, the body of `\lx@add@affiliations` after its dequeue). `attr`
/// is passed to each entry, `marked_attr` instead when the list has markers and `marks_are_labels`.
/// `authors_queued` says every author is queued, so an unmarked line may be placed by the marks: an
/// address run by its authors' positions, another line with the affiliation before it. An `\author`
/// call's tail (`author_tail_calls`) may come before later authors, and then its unmarked lines are
/// its own creators' rows, as each was before.
pub fn affiliation_calls(
  attr: Option<Tokens>,
  marked_attr: Option<Tokens>,
  stuff: Tokens,
  marks_are_labels: bool,
  authors_queued: bool,
) -> Result<Vec<Token>> {
  let mut calls: Vec<Token> = Vec::new();
  // Consume `\\[len]` row-break optionals before splitting (KNOWN_PERL_ERRORS #75).
  let stuff = rewrite_ordinal_superscripts(strip_linebreak_options(stuff));
  let with_sup = marks_are_labels && position_of(&stuff, &authorsup_markers()).is_some();
  // In a marked list, the affiliations it names, each a mark-led line (or a piece of one) with the unmarked lines
  // that follow it.
  let mut marked_entries: Vec<Vec<Token>> = Vec::new();
  // The runs of address lines, placed once the whole list is read (`place_address_runs`); whether a run is being
  // read, and whether the line just read was an entry or its continuation.
  let mut address_runs: Vec<AddressRun> = Vec::new();
  let mut in_run = false;
  let mut follows_entry = false;
  for (delimiter, line) in split_tokens_delimited(stuff, affil_splits()) {
    // Skip empty segments (e.g. a trailing \\ or a line that was wholly
    // consumed by an \email/\url) so they don't become blank affiliations.
    // Mirrors \lx@add@authors.
    if line.is_empty() {
      continue;
    }
    if with_sup {
      let across_and = delimiter.iter().any(|t| {
        author_and_splits()
          .iter()
          .any(|d| matches!(d, SplitDelim::Token(a) if a == t))
      });
      if authors_queued
        && position_of(&line, &authorsup_markers()).is_none()
        && line_is_email_list(&line)
      {
        // (`\and` separates entries, and runs: `Univ A \\ a@x \and b@y` is two)
        if !in_run || across_and {
          let under = (follows_entry && !across_and).then(|| {
            let entry = marked_entries.len() - 1;
            (entry, marked_entries[entry].len())
          });
          address_runs.push(AddressRun { lines: Vec::new(), under });
          in_run = true;
        }
        if let Some(run) = address_runs.last_mut() {
          run.lines.push((delimiter, line));
        }
        follows_entry = false;
        continue;
      }
      in_run = false;
      // An unmarked line continues the affiliation before it, with its `\\` put back, as an unmarked line continues
      // an entry in `\lx@add@authors`: `$^1$Department\\University` is one affiliation; on its own, an unlabelled
      // line was placed by position, not by mark (2609.19448; repro
      // sectioning-frontmatter/marked_affiliation_continuation_lines_stay_with_it). Not across `\and`, which
      // separates entries there too (DIVERGENCES #52(g)), nor a footnote-symbol legend (`* Equal contribution`,
      // `$\dagger$ Corresponding author`), which belongs to the whole block (llncs `\institute`, 2609.06094).
      if authors_queued
        && position_of(&line, &authorsup_markers()).is_none()
        && !across_and
        && !starts_with_footnote_symbol(&line)
        && let Some(last) = marked_entries.last_mut()
      {
        last.extend(delimiter);
        last.extend(line.unlist());
        follows_entry = true;
        continue;
      }
      // The superscript markers ARE the affiliation labels here, so drop any
      // caller-supplied labelseq: applying both double-labels each affiliation
      // and duplicates it onto every \inst{n} author. Mirrors \lx@add@authors,
      // which likewise passes no attr in its with-superscript branch.
      // A line that starts with a mark and names several marked institutions (`$^1$Univ A; $^2$Univ B`)
      // is split at each whitespace-preceded mark, as `\lx@add@authors` splits its marker-led affiliation
      // lines: one label per affiliation, else the last mark labelled them all (2609.22690, 21347). A
      // line with text before its first mark (`Univ X \\ Present address: $^2$Univ Y`'s second line)
      // stays whole, as there.
      let segs = if marker_leads(&line) {
        split_wrapped_affiliation_marks(line)
      } else {
        vec![line]
      };
      for seg in segs {
        if seg.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
          continue;
        }
        marked_entries.push(seg.unlist());
        follows_entry = true;
      }
    } else {
      calls.extend(
        Invocation!(T_CS!("\\lx@add@affiliation"), vec![
          attr.clone(),
          Some(line)
        ])
        .unlist(),
      );
    }
  }
  let email_calls = place_address_runs(address_runs, &mut marked_entries)?;
  for entry in marked_entries {
    let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(Tokens::new(
      entry
    ))]);
    calls.extend(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        marked_attr.clone(),
        Some(withsup)
      ])
      .unlist(),
    );
  }
  calls.extend(email_calls);
  Ok(calls)
}

/// A run of consecutive address lines in a marked affiliation list, with the entry it follows (when the line before
/// it was that entry, not across `\and`) and the length that entry had when the run was read: where a continuation
/// goes.
struct AddressRun {
  lines: Vec<(Vec<Token>, Tokens)>,
  under: Option<(usize, usize)>,
}

/// The `\lx@add@email` calls for the address runs of a marked affiliation list (OXIDIZED_DESIGN #52(j)); a run that
/// stays with its affiliation is spliced into that entry. An address claims an author only where the list says
/// whose it is:
/// - when the list puts addresses under its affiliations — it has one, or a run sits between two (2609.19448
///   `$^1$Department\\University\\\texttt{david.abel@ed.ac.uk}\\[2ex]$^2$…`) — one address under an affiliation
///   exactly one author requests is that author's; any other run under an affiliation continues it, as the PDF prints
///   it, claiming no author (an affiliation several authors share, or several addresses under one author's, which
///   may be everyone's);
/// - else the run is the block's: one address for each author goes to the authors in order (llncs
///   `\institute{$^1$A \\ $^2$B \\ \email{a@x} \\ \email{b@y}}`), any other is one contact a line on the shared
///   creator below the authors (#159) — by order, 2609.06094's four addresses for six authors had given Jindong
///   Gu's to Runjia Li, and by the mark of the affiliation they follow, three of them to the wrong author.
fn place_address_runs(runs: Vec<AddressRun>, entries: &mut [Vec<Token>]) -> Result<Vec<Token>> {
  let authors = queued_author_marks();
  // (a legend line, `* Equal contribution`, is an entry with no mark; nor does a footnote-symbol mark after a run,
  // `$^\dagger$ Corresponding author`, show the run sits under an affiliation — as an iopart `\address`'s one mark it
  // still labels one)
  let marks: Vec<Option<Vec<Token>>> = entries.iter().map(|entry| entry_mark(entry)).collect();
  let affiliation_follows = |entry: usize| {
    marks[entry + 1..]
      .iter()
      .flatten()
      .any(|mark| !is_symbol_mark(mark))
  };
  let under_affiliations = marks.iter().flatten().count() == 1
    || runs.iter().any(|run| {
      run
        .under
        .is_some_and(|(entry, _)| affiliation_follows(entry))
    });
  let mut placed: Vec<Vec<Token>> = Vec::new();
  // In reverse, so a continuation spliced into an entry leaves the places of the runs before it there.
  for run in runs.into_iter().rev() {
    let counts: Vec<usize> = run
      .lines
      .iter()
      .map(|(_, line)| email_addresses(line).map_or(0, |a| a.len()))
      .collect();
    let total: usize = counts.iter().sum();
    let mut calls = Vec::new();
    match run.under {
      Some((entry, at)) if under_affiliations => {
        let requesting = entry_mark(&entries[entry])
          .map_or_else(Vec::new, |mark| requesting_authors(&authors, &mark));
        if let ([author], 1) = (requesting.as_slice(), total) {
          for (_, line) in run.lines {
            calls.extend(email_line_calls(
              line,
              AddressPlacement::Authors(vec![*author]),
            )?);
          }
        } else {
          let continuation: Vec<Token> = run
            .lines
            .into_iter()
            .flat_map(|(delimiter, line)| delimiter.into_iter().chain(line.unlist()))
            .collect();
          entries[entry].splice(at..at, continuation);
        }
      },
      _ if total == authors.len() => {
        let mut positions = 1..;
        for ((_, line), count) in run.lines.into_iter().zip(counts) {
          let authors = positions.by_ref().take(count).collect();
          calls.extend(email_line_calls(line, AddressPlacement::Authors(authors))?);
        }
      },
      _ => {
        for (_, line) in run.lines {
          calls.extend(email_line_calls(line, AddressPlacement::Shared)?);
        }
      },
    }
    placed.push(calls);
  }
  Ok(placed.into_iter().rev().flatten().collect())
}

/// The queue positions of the authors whose marks request the affiliation label `mark` sets.
fn requesting_authors(authors: &[Vec<Vec<Token>>], mark: &[Token]) -> Vec<usize> {
  let Some(label) = mark_label(mark) else {
    return Vec::new();
  };
  authors
    .iter()
    .enumerate()
    .filter(|(_, marks)| {
      marks.iter().any(|operand| {
        clean_frontmatter_labels(&Tokens::new(operand.clone()).to_string(), "affiliation")
          .contains(&label)
      })
    })
    .map(|(i, _)| i + 1)
    .collect()
}

/// Where the addresses of a line in an author block go.
enum AddressPlacement {
  /// The whole line is one contact, attached as an unlabelled email is.
  OneContact,
  /// The n-th address to the n-th author, counted over the emails so far (`labelseq=author`).
  Sequence,
  /// Each address to the author at that queue position, by the `author:N` label the creator digests with.
  Authors(Vec<usize>),
  /// The whole line is one contact of the author block, on the shared creator below the authors (#159): its label
  /// names no creator.
  Shared,
}

/// The `\lx@add@email` calls for a line of addresses in an author block (`a@x, b@y`, `{a,b}@dom`), placed as
/// `placement` says: one call per address, a whole-line wrapper repeated on each, or the line itself when it holds one
/// address (`\href{mailto:a@x}{a@x}` stays a link); else the line as one contact. OXIDIZED_DESIGN #52(j).
fn email_line_calls(line: Tokens, placement: AddressPlacement) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  // A wrapper is kept on each address (`\texttt`, `\small`, `\url`), but an email command's (`\email`, `\mailto`,
  // `\emailaddr`) is the contact `\lx@add@email` stands for — nested, it made a second (empty) contact and advanced
  // the `labelseq` count twice, so the second address found no author (llncs `\institute{… \\ \email{a@x, b@y}}`).
  let is_email_command = |cmd: &Token| cmd.with_str(|name| name.to_lowercase().contains("mail"));
  let unwrapped = |line: Tokens| match whole_line_cs_wrapper(&line) {
    Some((cmd, inner)) if is_email_command(&cmd) => inner,
    _ => line,
  };
  let addrs = email_addresses(&line).unwrap_or_default();
  let options: Vec<Option<Tokens>> = match placement {
    AddressPlacement::Sequence if !addrs.is_empty() => addrs
      .iter()
      .map(|_| Some(mouth::tokenize_internal("labelseq=author")))
      .collect(),
    AddressPlacement::Authors(authors) if !addrs.is_empty() && authors.len() == addrs.len() => {
      authors
        .into_iter()
        .map(|n| {
          Some(mouth::tokenize_internal(TeXString::assembled(s!(
            "label=author:{n}"
          ))))
        })
        .collect()
    },
    placement => {
      // One contact; an email command's wrapper is the contact itself.
      let opts = matches!(placement, AddressPlacement::Shared)
        .then(|| mouth::tokenize_internal("label=addresses:block"));
      calls
        .extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(unwrapped(line))]).unlist());
      return Ok(calls);
    },
  };
  if let [opts] = options.as_slice() {
    calls.extend(
      Invocation!(T_CS!("\\lx@add@email"), vec![
        opts.clone(),
        Some(unwrapped(line))
      ])
      .unlist(),
    );
    return Ok(calls);
  }
  let wrapper = whole_line_cs_wrapper(&line)
    .map(|(cmd, _)| cmd)
    .filter(|cmd| !is_email_command(cmd));
  for (addr, opts) in addrs.into_iter().zip(options) {
    let mut toks = mouth::tokenize(TeXString::assembled(addr)).unlist();
    if let Some(cmd) = wrapper {
      let mut wrapped = vec![cmd, T_BEGIN!()];
      wrapped.append(&mut toks);
      wrapped.push(T_END!());
      toks = wrapped;
    }
    calls
      .extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(Tokens::new(toks))]).unlist());
  }
  Ok(calls)
}

/// The marks that request an affiliation label in an author (`\lx@author@withsup`, llncs `\inst`) and set it in an
/// affiliation (`\lx@affiliation@withsup`, `\lx@affiliation@withinst`).
fn affiliation_mark_tokens() -> [Token; 4] {
  [
    T_SUPER!(),
    T_CS!("\\textsuperscript"),
    T_CS!("\\inst"),
    T_CS!("\\lx@frontmatter@symbolmark"),
  ]
}

/// The operand of each mark in `tokens`, in order, read as `read_frontmatter_sup_operand` reads it.
fn mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  let marks = affiliation_mark_tokens();
  let mut operands = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if marks.contains(&tokens[i])
      && let Some((operand, next)) = sup_operand_at(tokens, i + 1)
    {
      operands.push(operand);
      i = next;
    } else {
      i += 1;
    }
  }
  operands
}

/// The marks an author's tokens request: those outside its notes, a `\thanks{...}` or `\footnote{...}` whose own
/// superscripts are its text or label it (`note_body`) — `Ann Able\thanks{$^{a}$ Supported by ...}` requests nothing.
fn author_mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  mark_operands(&outside_notes(tokens))
}

/// An author's tokens without its `\thanks{...}` and `\footnote{...}` (and their optional label).
fn outside_notes(tokens: &[Token]) -> Vec<Token> {
  let mut outside = Vec::with_capacity(tokens.len());
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\thanks") || tokens[i] == T_CS!("\\footnote") {
      let mut j = i + 1;
      if tokens.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[j..].iter().position(|t| *t == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if let Some((_, next)) = sup_operand_at(tokens, j) {
        i = next;
        continue;
      }
    }
    outside.push(tokens[i]);
    i += 1;
  }
  outside
}

/// The notes and requests an affiliation line under a name carries in a per-author class's `\author` (`Steward
/// Observatory\altaffilmark{2}`, `Univ A\thanks{Grant}`), taken off the line with their arguments to annotate the
/// author, as Perl, adding that `\author` whole, gives them to it: digested inside the affiliation, an `\altaffilmark`
/// request made a creator of its own and a `\thanks` a document-level note. The article `\author` nests the line as
/// Perl does (Base_Utility.pool.ltxml:723-725), so a `\thanks` there stays a document note (pubnote, as Perl's
/// `digestFrontmatterItem` binds it in a contact, :332-333; html_feedback#6888).
fn take_author_annotations(line: &Tokens) -> (Tokens, Vec<Token>) {
  let notes = [
    T_CS!("\\thanks"),
    T_CS!("\\thanksref"),
    T_CS!("\\footnote"),
    T_CS!("\\altaffilmark"),
  ];
  let v = line.unlist_ref();
  let mut kept = Vec::with_capacity(v.len());
  let mut taken = Vec::new();
  let mut i = 0;
  while i < v.len() {
    if notes.contains(&v[i]) {
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = v[j..].iter().position(|t| *t == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if let Some((_, next)) = sup_operand_at(v, j) {
        taken.extend_from_slice(&v[i..next]);
        i = next;
        continue;
      }
    }
    kept.push(v[i]);
    i += 1;
  }
  (Tokens::new(kept), taken)
}

/// The operand of an affiliation entry's first mark, which `\lx@sup@setlabel@affiliation` labels it by (a later
/// superscript in the entry is its text, 63i).
fn entry_mark(entry: &[Token]) -> Option<Vec<Token>> { mark_operands(entry).into_iter().next() }

/// The label `\lx@sup@setlabel@affiliation` sets for a mark's operand: the first of its labels.
fn mark_label(operand: &[Token]) -> Option<String> {
  clean_frontmatter_labels(&Tokens::new(operand.to_vec()).to_string(), "affiliation")
    .into_iter()
    .next()
}

/// The mark operands of each queued author, in queue order: the n-th digests with the label `author:n`
/// (`num_ltx:creator{author}`), by which an address goes to it — as the count starts from no author, which holds
/// unless authors an earlier flush digested are still kept (an appending `\author` after `\maketitle`).
fn queued_author_marks() -> Vec<Vec<Vec<Token>>> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .map(|entry| author_mark_operands(entry.2.unlist_ref()))
      .collect(),
    _ => Vec::new(),
  })
}

/// The operand of a mark at `tokens[i..]`, as `read_frontmatter_sup_operand` reads it from the input — a group's
/// content, a control sequence with the group after it, else one token — and the index after it.
fn sup_operand_at(tokens: &[Token], mut i: usize) -> Option<(Vec<Token>, usize)> {
  while tokens.get(i) == Some(&T_SPACE!()) {
    i += 1;
  }
  let group_end = |start: usize| {
    let mut depth = 0usize;
    for (j, t) in tokens.iter().enumerate().skip(start) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth = depth.saturating_sub(1);
          if depth == 0 {
            return Some(j);
          }
        },
        _ => {},
      }
    }
    None
  };
  let first = *tokens.get(i)?;
  if first.get_catcode() == Catcode::BEGIN {
    let end = group_end(i)?;
    return Some((tokens[i + 1..end].to_vec(), end + 1));
  }
  if first.get_catcode() == Catcode::CS
    && tokens
      .get(i + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let end = group_end(i + 1)?;
    return Some((tokens[i..=end].to_vec(), end + 1));
  }
  Some((vec![first], i + 1))
}

/// `\unitlength` in sp, exact (65536, 1pt, when it is undefined): the picture
/// unit every `{picture}` and pict2e coordinate is multiplied by (Perl
/// `picScale`, latex_constructs.pool.ltxml:4896-4921). A skip that calc's
/// `\setlength` put there is its natural width (the register coerces it).
pub fn unitlength_sp() -> Result<i64> {
  Ok(match lookup_register("\\unitlength", Vec::new())? {
    Some(value) => Dimension::from(value).value_of(),
    None => 65536,
  })
}

/// Has this diagnostic already been reported? Marks `key` reported either way.
///
/// Port of Perl's report-once diagnostic idiom, e.g. `\selectfont`
/// (latex_constructs.pool.ltxml L5210-5211):
///
/// ```perl
/// elsif (!LookupValue("reported_unrecognized_font_family_$family")) {
///   AssignValue("reported_unrecognized_font_family_$family", 1, 'global');
///   Info('unexpected', $family, $_[0], "Unrecognized font family '$family'."); }
/// ```
///
/// So a caller reads `if !already_reported(&key) { Info!(…) }`, mirroring the
/// Perl `elsif`. An unrecognized font family/series/shape — or a missing font
/// encoding — is announced once per document, not once per occurrence: the
/// font switch that provokes it typically sits inside a macro that a document
/// expands in every table cell (witness 2503.04421, whose `\XSolidBrush`
/// column produced 28 identical `Info:unexpected:ding` lines).
///
/// Global scope, as in Perl: the suppression has to outlive the group the
/// diagnostic fired in, or a grouped `{\dingfamily …}` reports again each time.
pub fn already_reported(key: &str) -> bool {
  if lookup_bool(key) {
    return true;
  }
  assign_value(key, true, Some(Scope::Global));
  false
}

/// Join token groups with a conjunction token between them.
///
/// Port of Perl `JoinTokens($conjunction, @things)` (Base_Utility.pool.ltxml L142-148).
#[allow(dead_code)]
pub fn join_tokens(conjunction: &Tokens, things: Vec<Tokens>) -> Tokens {
  if things.is_empty() {
    return Tokens::new(vec![]);
  }
  let mut result: Vec<Token> = Vec::new();
  let mut first = true;
  for thing in things {
    if !first {
      result.extend_from_slice(conjunction.unlist_ref());
    }
    result.extend(thing.unlist());
    first = false;
  }
  Tokens::new(result)
}

//======================================================================
// Front Matter machinery (Perl Base_Utility.pool.ltxml, PR #2767)
//======================================================================

/// Perl: showFrontmatter($entry) — debug formatter for a frontmatter entry.
fn show_frontmatter(entry: &TagData) -> String {
  let mut attrs: Vec<String> = entry.attr.iter().map(|(k, v)| s!("{k}={v}")).collect();
  attrs.sort();
  let content: String = entry
    .content
    .iter()
    .map(|c| match c {
      TagContent::PlaceKeeper => "place_keeper".to_string(),
      TagContent::Box(d) => d.to_string(),
      TagContent::Entry(e) => show_frontmatter(e),
    })
    .collect();
  s!("{}  [{}] {}", entry.tag, attrs.join(","), content)
}

/// Perl: LookupMapping returning a number (0 when absent).
fn lookup_mapping_int(map: &str, key: &str) -> i64 {
  match lookup_mapping(map, key) {
    Some(Stored::Int(n)) => n,
    Some(Stored::Number(n)) => n.0,
    _ => 0,
  }
}

/// The `\lx@tag@intags` whatsit a held tag digested to (`\lx@tag@intags@held`): the digested box itself, or the
/// first such whatsit of its list.
fn held_tag_whatsit(tag: &Digested) -> Option<Digested> {
  match tag.data() {
    DigestedData::Whatsit(w) if w.borrow().get_definition().get_cs_name() == "\\lx@tag@intags" => {
      Some(tag.clone())
    },
    DigestedData::List(l) => l.borrow().boxes.iter().find_map(held_tag_whatsit),
    _ => None,
  }
}

/// Walk a `Digested` and concatenate its text content (for attribute use,
/// matching Perl's `setAttribute(..., DigestText(...))` semantics). Tbox
/// children contribute their text; nested Lists recurse; `\hskip`-style
/// Whatsits (which are side-effect-only constructors with no text content)
/// fall back to `dimension_to_spaces(width)` instead of reverting to the
/// macro name. All other Whatsits use their normal `get_string` path.
/// (Rust-only helper, previously in latex_constructs; moved here since
/// digest_front_matter needs it for the creator `before` separators.)
pub fn digested_to_text(d: &Digested) -> Result<String> {
  let mut out = String::new();
  match d.data() {
    DigestedData::TBox(b) => out.push_str(&b.borrow().get_string()?),
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        out.push_str(&digested_to_text(child)?);
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      match w.get_property("width").as_deref() {
        Some(Stored::Dimension(width)) => {
          // The whatsit's OWN font — see tex_glue::dimension_to_spaces.
          let font = w.get_font()?;
          out.push_str(&super::tex_glue::dimension_to_spaces(
            *width,
            font.as_deref(),
          ));
        },
        _ => {
          out.push_str(&w.get_string()?);
        },
      }
    },
    _ => out.push_str(&d.to_string()),
  }
  Ok(out)
}

/// Whether digested material typesets something a reader sees: a character that is not a space, or
/// an image (a whatsit carrying a `graphic`), anywhere in it — through lists, whatsit arguments and
/// bodies, and alignment cells. Paragraph and spacing commands, rules and empty boxes are not content.
/// A whatsit's own string is its reversion (Whatsit.pm `toString`), so a digested `\noindent\par`
/// stringifies as source; this walks the boxes instead. Used to keep a speculative typesetting (a
/// class's `\maketitle` replay, a one-shot page overlay) only when it shows something.
pub fn typesets_content(d: &Digested) -> bool {
  match d.data() {
    DigestedData::TBox(b) => b
      .borrow()
      .get_string()
      .is_ok_and(|s| s.chars().any(|c| !c.is_whitespace())),
    DigestedData::List(l) => l.borrow().boxes.iter().any(typesets_content),
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      w.get_property("graphic").is_some()
        // A tabular or `\halign` keeps its rows in its `alignment` property (tex_tables.rs): a class's `\maketitle`
        // that is only a table typesets it (simplecv's header, brandeis-problemset's title table; dropped since 59p).
        || matches!(w.get_property("alignment").as_deref(), Some(Stored::Digested(a)) if typesets_content(a))
        || w.get_args().iter().flatten().any(typesets_content)
        || w
          .get_body()
          .ok()
          .flatten()
          .is_some_and(|body| typesets_content(&body))
    },
    DigestedData::Alignment(a) => a.borrow().rows().iter().any(|row| {
      row
        .get_columns()
        .iter()
        .any(|cell| cell.boxes.as_ref().is_some_and(typesets_content))
    }),
    _ => false,
  }
}

/// frontmatter_raw contains the undigested commands to create frontmatter,
/// along with the tag & attributes that would be created.
/// Digestion is deferred until \maketitle, or something similar,
/// to avoid extra side-effects, particularly when entries
/// (eg. \title, \author) get redefined & replaced.
/// Use dequeue_front_matter or \lx@clear@frontmatter if there should be only 1 entry of $tag
/// Perl: queueFrontMatter($stomach, $tag, $attr, $command).
pub fn queue_front_matter(tag: &str, attr: Option<&KeyVals>, command: Tokens) {
  // Convert KeyVals to a hash, but be concerned about multiple values!?!?
  // (Perl: ToString($attr->getValue($_)) — the last value for multi-valued keys)
  let mut attr_hash = TagAttrs::default();
  if let Some(kv) = attr {
    for key in kv.get_keyvals().keys() {
      if let Some(value) = kv.get_value(key) {
        attr_hash.insert(key.clone(), value.to_string());
      }
    }
  }
  DebugFeature!(
    "frontmatter",
    "FRONT Queuing {tag} [{:?}] {command}",
    attr_hash
  );
  let missing = with_value("frontmatter_raw", |v| {
    !matches!(v, Some(Stored::FrontmatterRaw(_)))
  });
  if missing {
    assign_value(
      "frontmatter_raw",
      Stored::FrontmatterRaw(Vec::new()),
      Some(Scope::Global),
    );
  }
  with_value_mut("frontmatter_raw", |val_opt| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
      queue.push((tag.to_string(), attr_hash, command));
    }
  });
}

/// Build and queue the deferred `\lx@add@frontmatter@now` invocation for a
/// frontmatter entry `<tag attrs>content</tag>`. Shared by `\lx@add@frontmatter`
/// (content present) and `\lx@add@frontmatter@container` (content = None — a
/// deliberate empty container element that anchors later annotations).
/// Perl: queueFrontMatter($stomach, $tag, $attr,
///   Invocation(T_CS('\lx@add@frontmatter@now'), $keys, $tag, $attr, $tokens)).
fn queue_add_frontmatter_now(
  keys_opt: Option<&KeyVals>,
  tag_tks: &Tokens,
  attrs_opt: Option<&KeyVals>,
  content: Option<&Tokens>,
) -> Result<()> {
  let mut inv_tokens: Vec<Token> = vec![T_CS!("\\lx@add@frontmatter@now")];
  if let Some(keys) = keys_opt {
    inv_tokens.push(T_OTHER!("["));
    inv_tokens.extend(keys.revert()?.unlist());
    inv_tokens.push(T_OTHER!("]"));
  }
  inv_tokens.push(T_BEGIN!());
  inv_tokens.extend(tag_tks.unlist_ref().iter().copied());
  inv_tokens.push(T_END!());
  if let Some(attrs) = attrs_opt {
    inv_tokens.push(T_OTHER!("["));
    inv_tokens.extend(attrs.revert()?.unlist());
    inv_tokens.push(T_OTHER!("]"));
  }
  inv_tokens.push(T_BEGIN!());
  if let Some(content) = content {
    inv_tokens.extend(content.unlist_ref().iter().copied());
  }
  inv_tokens.push(T_END!());
  queue_front_matter(&tag_tks.to_string(), attrs_opt, Tokens::new(inv_tokens));
  Ok(())
}

/// The number of creators queued so far (`frontmatter_raw`).
pub fn queued_creator_count() -> usize {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| entry.0 == "ltx:creator")
      .count(),
    _ => 0,
  })
}

/// Does a queued creator carry superscript marks (`\author{A\textsuperscript{1}}`) for affiliations
/// to be matched against?
pub fn queued_creators_have_marks() -> bool {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.iter().any(|entry| {
      entry.0 == "ltx:creator" && position_of(&entry.2, &authorsup_markers()).is_some()
    }),
    _ => false,
  })
}

/// Is a frontmatter item with this tag queued (`frontmatter_raw`) or already digested into the
/// frontmatter (`frontmatter`, which `digest_front_matter` fills at `\maketitle`; a pending
/// annotation stub is not an item)?
pub fn has_front_matter(tag: &str) -> bool {
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.iter().any(|entry| entry.0 == tag),
    _ => false,
  });
  queued
    || with_value("frontmatter", |v| match v {
      Some(Stored::HashTagData(frnt)) => frnt.get(tag).is_some_and(|list| {
        list
          .iter()
          .any(|e| e.attr.get("role").map(String::as_str) != Some("pending"))
      }),
      _ => false,
    })
}

/// This removes previously stored (but deferred) frontmatter that is being overridden.
/// It matches the tag and any stored attributes in `attr`.
/// Perl: dequeueFrontMatter($tag, %attr).
pub fn dequeue_front_matter(tag: &str, attr: &[(&str, &str)]) {
  with_value_mut("frontmatter_raw", |val_opt| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
      queue.retain(|entry| {
        let keep = entry.0 != tag
          || attr
            .iter()
            .any(|(k, v)| entry.1.get(*k).map(String::as_str).unwrap_or("") != *v);
        if !keep {
          DebugFeature!("frontmatter", "FRONT DEQueuing {} [{:?}]", entry.0, entry.1);
        }
        keep
      });
    }
  });
}

/// Digest the content for a frontmatter item, disabling or masking certain commands.
/// Note that we shouldn't be digesting any frontmatter until within document, so inPreamble unnec.
/// See clean_trailing_break for cleanup of misused \\.
/// Perl: digestFrontmatterItem($stomach, $tag, $item).
fn digest_frontmatter_item(tag: &str, item: Tokens) -> Result<Digested> {
  bgroup();
  let_i(
    &T_CS!("\\label"),
    &T_CS!("\\lx@set@frontmatter@label"),
    None,
  );
  let_i(
    &T_CS!("\\footnote"),
    &(if tag == "ltx:creator" {
      T_CS!("\\lx@add@note")
    } else {
      T_CS!("\\lx@add@pubnote")
    }),
    None,
  );
  let_i(
    &T_CS!("\\thanks"),
    &(if tag == "ltx:creator" {
      T_CS!("\\lx@add@thanks")
    } else {
      T_CS!("\\lx@add@pubnote@thanks")
    }),
    None,
  );
  let digested = digest_text(item);
  egroup()?;
  digested
}

/// Perl: cleanTrailingBreak($document, $node) — remove trailing whitespace
/// text nodes and ltx:break elements.
fn clean_trailing_break(document: &mut Document, node: &mut Node) -> Result<()> {
  while let Some(last) = node.get_last_child() {
    let is_ws_text = last.get_type() == Some(NodeType::TextNode)
      && last.get_content().chars().all(char::is_whitespace);
    let is_break = with(document::get_node_qname(&last), |qname| {
      qname == "ltx:break"
    });
    if !(is_ws_text || is_break) {
      break;
    }
    document.remove_node(last);
  }
  Ok(())
}

/// Unwrap a `font="bold"` that wraps an ENTIRE personname.
///
/// SURPASS-Perl (html_feedback#61, OXIDIZED_DESIGN_DIVERGENCES #122). Some classes
/// (neurips_2023) bold the whole author block with a block-level `\bf` in their
/// `\@maketitle`, which LaTeXML does not emulate — it captures semantic creators.
/// A paper that then `\textbf`s only *some* name lines (relying on the class `\bf`
/// for the rest) renders incoherently: bold on the `\textbf` lines, plain on the
/// others. Bolding a whole author name is presentational author-block styling, not
/// semantic, so a personname whose sole meaningful content is one `<ltx:text
/// font="bold">` is normalized to plain. Mixed-content names (bold on part of the
/// name, or bold+other styles) are left untouched — only the whole-name pure-bold
/// wrapper is stripped. Both Perl and Rust emit the wrapper (SHARED-FAILURE).
fn unwrap_whole_name_bold(document: &mut Document, node: &Node) -> Result<()> {
  let mut sole_bold: Option<Node> = None;
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) => {
        // Any non-whitespace text directly under the name means it is not a
        // single whole-name wrapper — leave it alone.
        if !child.get_content().chars().all(char::is_whitespace) {
          return Ok(());
        }
      },
      Some(NodeType::ElementNode) => {
        // A reference marker (`\footnotemark`/`\thanks` → `<ltx:note>`) or a trailing
        // `<ltx:break>` from a misused `\\` is not name content — skip it so it does
        // not block the unwrap of an otherwise whole-name bold. Witness: "Zhou Zhao"
        // in arXiv 2507.06670, `\textbf{Zhou Zhao} \footnotemark[2]`.
        if with(document::get_node_qname(&child), |qname| {
          qname == "ltx:note" || qname == "ltx:break"
        }) {
          continue;
        }
        // A `<ltx:text>` whose decoded font would serialize as EXACTLY `font="bold"` —
        // i.e. bold is its only departure from the default text font. Reuse the
        // serializer's own `font_attribute_string()` rather than hand-matching the
        // family/series/shape defaults, so this cannot drift from what `font=` emits.
        // Bold+italic / bold-sans yield "bold italic"/"… bold" ≠ "bold" and are left
        // untouched (extra intent).
        let is_pure_bold = with(document::get_node_qname(&child), |qname| {
          qname == "ltx:text"
        }) && child.get_attribute("_font").is_some_and(|fid| {
          document
            .decode_font(&fid)
            .is_some_and(|f| f.font_attribute_string() == "bold")
        });
        if is_pure_bold && sole_bold.is_none() {
          sole_bold = Some(child);
        } else {
          // A second element, or a non-pure-bold element → not a sole whole-name bold.
          return Ok(());
        }
      },
      _ => {},
    }
  }
  if let Some(bold) = sole_bold {
    document.unwrap_nodes(bold)?;
  }
  Ok(())
}

/// Read a full superscript operand as RAW (undigested) tokens for a frontmatter
/// author/affiliation marker (`^X` / `\textsuperscript{X}`).
///
/// A real math superscript digests its nucleus (`TeX_Math` `scriptHandler`
/// "invoke tokens until you get a box"), so `^\text{x}` binds `\text` to its
/// `{x}` and reads as one box — i.e. `^\text{x}` == `^{\text{x}}`. The hijacked
/// marker instead captures the operand as tokens for a matching label; the bare
/// `{}` read used to grab only the leading control sequence (`\text`), orphaning
/// its `{...}` argument. In inline math that stray `{...}` left a brace-group
/// frame on top, so the marker's closing `$` fired `\lx@end@inline@math` against
/// it — "Attempt to end mode math" — arbitrarily deep for nested
/// `$^\text{$...$}$` markers (html_feedback#1021, arXiv:2403.11905; both engines
/// erred, SHARED-FAILURE). Grabbing the control sequence together with its
/// following group keeps `\text{...}` — and any `$...$` nested inside it — whole
/// and undigested, so the surrounding math stays balanced. Numeric/char markers
/// (`^1`) and already-braced markers (`^{...}`, `\textsuperscript{...}`) read
/// exactly as before. See OXIDIZED_DESIGN #129.
fn read_frontmatter_sup_operand() -> Result<Tokens> {
  skip_spaces()?;
  // `^{...}` / `\textsuperscript{...}`: the whole braced group is the operand.
  if if_next(T_BEGIN!())? {
    return read_arg(ExpansionLevel::Off);
  }
  let Some(first) = read_token()? else {
    return Ok(Tokens::new(Vec::new()));
  };
  // `^\text{...}` (bare control-sequence operand): keep a following `{...}` group
  // so the CS is not severed from its argument.
  if first.get_catcode() == Catcode::CS && if_next(T_BEGIN!())? {
    let group = read_arg(ExpansionLevel::Off)?;
    let inner = group.unlist_ref();
    let mut operand = Vec::with_capacity(inner.len() + 3);
    operand.push(first);
    operand.push(T_BEGIN!());
    operand.extend(inner.iter().copied());
    operand.push(T_END!());
    return Ok(Tokens::new(operand));
  }
  Ok(Tokens::new(vec![first]))
}

/// Perl: cleanFrontmatterLabels($labels, $prefix).
fn clean_frontmatter_labels(labels: &str, prefix: &str) -> Vec<String> {
  let labels = labels.replace("\\rm", "");
  let mut cleaned = Vec::new();
  let mut pieces: Vec<&str> = labels.split(',').collect();
  // Perl split drops trailing empty fields
  while pieces.last().is_some_and(|p| p.is_empty()) {
    pieces.pop();
  }
  for label in pieces {
    let label = label.trim();
    // INTENTIONAL DIVERGENCE (OXIDIZED_DESIGN #34, KNOWN_PERL_ERRORS #31,
    // plan decisions log #5): Perl prefixes empty fields too, so a doubled
    // comma or empty keyval yields a contentless "prefix:" label that can
    // spuriously match another in relocate_annotations. Drop them.
    if label.is_empty() {
      continue;
    }
    let mut label = if let Some(inner) = label
      .strip_prefix("\\ref{")
      .and_then(|rest| rest.strip_suffix('}'))
      .filter(|inner| !inner.contains('}'))
    {
      let inner = inner.trim();
      if inner.is_empty() {
        continue; // \ref{} ⇒ contentless "LABEL:" — drop (same divergence)
      }
      s!("LABEL:{inner}")
    } else if !prefix.is_empty() {
      s!("{prefix}:{label}")
    } else {
      label.to_string()
    };
    label = SPACES_RE.replace_all(&label, "_").into_owned();
    label.retain(|c| !matches!(c, '{' | '}' | '(' | ')'));
    cleaned.push(label);
  }
  cleaned
}

/// Look for \lx@<tag>@<role>@name or \lx@<tag>@name
/// Perl: getFrontmatterName($name, $tag, $role).
fn get_frontmatter_name(name: Option<&String>, tag: &str, role: &str) -> Result<Option<String>> {
  let stag = tag.strip_prefix("ltx:").unwrap_or(tag);
  if let Some(name) = name
    && !name.is_empty()
  {
    return Ok(Some(name.clone()));
  }
  if !role.is_empty() {
    let cs = T_CS!(s!("\\lx@{stag}@{role}@name"));
    if lookup_definition(&cs)?.is_some() {
      let n = digest_text(Tokens!(cs))?.to_string();
      // A defined-but-empty per-role default (e.g. `\lx@contact@orcid@name`,
      // emptied because the iD glyph self-identifies) means NO label — return
      // None rather than an empty `ltx:contact_name`, and do not fall back to
      // the generic default. General, not role-special.
      return Ok(if n.is_empty() { None } else { Some(n) });
    }
  }
  let cs = T_CS!(s!("\\lx@{stag}@name"));
  if lookup_definition(&cs)?.is_some() {
    let n = digest_text(Tokens!(cs))?.to_string();
    if !n.is_empty() {
      return Ok(Some(n));
    }
  }
  Ok(None)
}

/// Push an entry to frontmatter{tag}, returning its index
/// (Perl holds a direct entry ref instead).
fn frontmatter_push(tag: &str, entry: TagData) -> usize {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
      let list = frnt.entry(tag.to_string()).or_insert_with(Vec::new);
      list.push(entry);
      list.len() - 1
    } else {
      0
    }
  })
}

/// The author creators queued and not yet digested.
fn queued_author_count() -> usize {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .count(),
    _ => 0,
  })
}

/// Drop the author creators an earlier flush digested, with the pending annotation
/// stubs of that batch, and restart the counts the next batch is numbered and labelled
/// by (`num_ltx:creator{author}`; every `num_ltx:contact` role, which `labelseq=author`
/// numbers its affiliations and emails by). Whether there was any to drop.
fn supersede_digested_authors() -> bool {
  let superseded = with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get_mut("ltx:creator")
    {
      let before = list.len();
      list.retain(|e| {
        !matches!(
          e.attr.get("role").map(String::as_str),
          Some("author" | "pending")
        )
      });
      list.len() < before
    } else {
      false
    }
  });
  if !superseded {
    return false;
  }
  assign_mapping("num_ltx:creator", "author", Some(Stored::Int(0)));
  let contact_roles = with_mapping_keys("num_ltx:contact", |keys| {
    keys.into_iter().map(to_string).collect::<Vec<_>>()
  });
  for role in contact_roles {
    assign_mapping("num_ltx:contact", &role, Some(Stored::Int(0)));
  }
  true
}

/// Drop the `frontmatter{tag}` entries that carry the same `name` as a new one (Perl:
/// `$$frontmatter{$tag} = []`), so a later `REPLACEABLE_FRONTMATTER_TAGS` entry
/// replaces a re-emission of itself (arXiv 2002.09766's second `\icmltitle`). An
/// entry under another name is another element and stays: a bilingual document's
/// "Abstract" and "摘要" abstracts (beamertheme-mirage-doc lost its English one).
/// OXIDIZED_DESIGN #154.
/// Whether a frontmatter entry's options ask it to `accumulate` (any value but `false`), the key taken out of them so it
/// is no attribute.
fn takes_accumulate(options: &mut TagAttrs) -> bool {
  options
    .remove("accumulate")
    .is_some_and(|value| value.trim() != "false")
}

fn frontmatter_clear_same_name(tag: &str, name: Option<&str>) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get_mut(tag)
    {
      list.retain(|e| e.attr.get("name").map(String::as_str) != name);
    }
  });
}

/// True if `frontmatter{tag}` holds an entry still awaiting its content (a lone
/// `PlaceKeeper`). Used by the `\lx@add@frontmatter@until` dedup to detect
/// re-entrancy: a same-tag `@until` nested inside another's `digest_next_body` (e.g.
/// a malformed `\begin{abstract}\begin{abstract}…`) would otherwise clear the parent's
/// still-open entry and later have the parent overwrite the child's content. When a
/// parent is in progress we skip the clear (leaving both entries, as before the fix)
/// rather than corrupt state. OXIDIZED_DESIGN #154.
fn frontmatter_has_open_placekeeper(tag: &str) -> bool {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get(tag)
    {
      list
        .iter()
        .any(|e| matches!(e.content.as_slice(), [TagContent::PlaceKeeper]))
    } else {
      false
    }
  })
}

/// Replace the first content item (the 'place_keeper') of the entry at
/// (tag, index). Perl: `$$entry[2] = ...` on the held entry ref.
fn frontmatter_set_first_content(tag: &str, index: usize, content: TagContent) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(entry) = frnt.get_mut(tag).and_then(|l| l.get_mut(index))
    {
      if let Some(first) = entry.content.first_mut() {
        *first = content;
      } else {
        entry.content.push(content);
      }
    }
  });
}

/// Find the frontmatter entry currently being digested and apply `f` to its attrs.
/// HOPEFULLY, there's only one pending entry ?????????
/// Perl: fetchPendingEntry().
fn with_pending_entry_attr(f: impl FnOnce(&mut TagAttrs)) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
      let mut tags: Vec<String> = frnt.keys().cloned().collect();
      tags.sort();
      for tag in tags {
        if let Some(last) = frnt.get_mut(&tag).and_then(|entries| entries.last_mut())
          && matches!(last.content.first(), Some(TagContent::PlaceKeeper))
        {
          f(&mut last.attr);
          return;
        }
      }
    }
  });
}

/// Digest FrontMatter (if not already?)
/// Perl: digestFrontMatter().
pub fn digest_front_matter() -> Result<()> {
  bgroup();
  // INTENTIONAL DIVERGENCE from Perl PR #2767 (KNOWN_PERL_ERRORS #30,
  // OXIDIZED_DESIGN): clear the queue BEFORE digesting. Perl digests
  // from the live queue and wipes it after the loop — but when a queued
  // entry's own content re-triggers digestFrontMatter (witness aa.cls
  // 0907.0384: `\abstract{...}{}` dispatches the 5-arg \abstract@new,
  // whose greedy params swallow the document's `\maketitle` into arg #5,
  // so the queued abstract CONTAINS \maketitle → \lx@frontmatterhere →
  // afterDigest → re-entry), Perl re-digests the same queue unboundedly:
  // PR-head Perl dies `Fatal:perl:deep_recursion ... invokeToken`, zero
  // output (verified 2026-06-04). Pre-clearing makes the nested
  // invocation see an empty queue and terminate; the same paper then
  // converts with zero errors (real LaTeX also compiles it). Everything
  // else — deferred timing, digestion order, late re-let/\def fidelity —
  // is exactly the PR's. Newly-queued entries during this digest are
  // processed by the next invocation (or the end-of-document fallback).
  let commands: Vec<RawFrontmatter> = match remove_value("frontmatter_raw") {
    Some(Stored::FrontmatterRaw(commands)) => commands,
    _ => Vec::new(),
  };
  // The authors digested here are the current ones: no later `\maketitle` supersedes
  // them unless another `\author` replaces them first (`\lx@maketitle@supersede`).
  if commands.iter().any(|entry| {
    entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
  }) {
    assign_value(
      "lx_authors_superseded",
      Stored::Bool(false),
      Some(Scope::Global),
    );
  }
  // The labels the authors' marks request, for the notes digested here (`note_body`) — this call's authors only: a
  // note queued for a later call (an appending `\author` after `\maketitle`) sees that call's.
  let requested: Vec<String> = commands
    .iter()
    .filter(|entry| {
      entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
    })
    .flat_map(|entry| author_mark_operands(entry.2.unlist_ref()))
    .flat_map(|operand| clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation"))
    .collect();
  assign_value(
    "lx_frontmatter_requested_marks",
    Stored::String(pin(requested.join("\u{1}"))),
    Some(Scope::Local),
  );
  if !commands.is_empty() {
    let_i(
      &T_CS!("\\lx@add@frontmatter"),
      &T_CS!("\\lx@add@frontmatter@now"),
      None,
    );
    let_i(
      &T_CS!("\\lx@annotate@frontmatter"),
      &T_CS!("\\lx@annotate@frontmatter@now"),
      None,
    );
    for (tag, attr, command) in commands {
      DebugFeature!(
        "frontmatter",
        "FRONT Digesting {tag} [{:?}] {command}",
        attr
      );
      // Perl parity (review 2026-06-04, replacing the master-era
      // fatal-swallow from witness arXiv:1903.01633): a Fatal raised
      // while digesting deferred frontmatter propagates and aborts
      // the conversion with a proper `Fatal:` log line, exactly like
      // Perl's un-eval'd `$stomach->digest($command)`. The 1903.01633
      // bug was the *silent* swallow (`let _ = digest(...)` left
      // `report.fatal=true` with no log line); propagation fixes the
      // silence without diverging from Perl. Non-fatal Error!s inside
      // the digest log-and-continue in both engines as before.
      digest(command)?;
    }
  }
  // Add punctuation to all ltx:creators, now that we know how many of each role.
  let mut updates: Vec<(usize, bool)> = Vec::new(); // (index, use_conjunction)
  with_value("frontmatter", |v| {
    if let Some(Stored::HashTagData(frnt)) = v
      && let Some(list) = frnt.get("ltx:creator")
    {
      for (i, item) in list.iter().enumerate() {
        let role = item.attr.get("role").map(String::as_str).unwrap_or("");
        let num: i64 = item
          .attr
          .get("_num")
          .and_then(|n| n.parse().ok())
          .unwrap_or(0);
        if !role.is_empty() && num > 1 {
          let n = lookup_mapping_int("num_ltx:creator", role);
          updates.push((i, num >= n));
        }
      }
    }
  });
  for (index, use_conjunction) in updates {
    let separator = DigestText!(Tokens!(if use_conjunction {
      T_CS!("\\lx@author@conj")
    } else {
      T_CS!("\\lx@author@sep")
    }))?;
    // `\lx@author@conj` may digest to \hskip Whatsits (\qquad) — extract
    // text-or-spaces (see \lx@author@prefix note in the previous port).
    let text = digested_to_text(&separator)?;
    with_value_mut("frontmatter", |val_opt| {
      if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
        && let Some(entry) = frnt.get_mut("ltx:creator").and_then(|l| l.get_mut(index))
      {
        entry.attr.insert("before".to_string(), text.clone());
      }
    });
  }
  egroup()?;
  Ok(())
}

/// First `<ltx:p>` under `root` reached only through plain paragraph wrappers — an
/// `ltx:logical-block` (a `center`/`flushleft` environment) or `ltx:para`. A paragraph
/// inside a further box (tcolorbox, `\parbox`, minipage, tabular) is layout we do not
/// read as a title candidate (see `maybe_promote_leading_title`).
fn first_plain_paragraph(root: &Node) -> Option<Node> {
  let q = document::get_node_qname(root);
  if q == pin_static("ltx:p") {
    return Some(root.clone());
  }
  if q != pin_static("ltx:logical-block") && q != pin_static("ltx:para") {
    return None;
  }
  root
    .get_child_nodes()
    .iter()
    .filter(|c| c.get_type() == Some(NodeType::ElementNode))
    .find_map(first_plain_paragraph)
}

/// Every `<ltx:p>` under `root` reached only through plain paragraph wrappers, in
/// document order (the uniqueness test of `maybe_promote_leading_title`).
fn collect_plain_paragraphs(root: &Node, out: &mut Vec<Node>) {
  let q = document::get_node_qname(root);
  if q == pin_static("ltx:p") {
    out.push(root.clone());
    return;
  }
  if q != pin_static("ltx:logical-block") && q != pin_static("ltx:para") {
    return;
  }
  for c in root.get_child_nodes() {
    if c.get_type() == Some(NodeType::ElementNode) {
      collect_plain_paragraphs(&c, out);
    }
  }
}

/// True if `root` or any descendant is set in a font larger than the body size —
/// the display-title signal. During construction the human-readable `fontsize`
/// attribute does not exist yet (it is derived from `_font` in a later finalize
/// pass); read the interned `_font` id and decode it here instead. `nominal` is
/// the document body size (NOMINAL_FONT_SIZE, mirroring font::defsize), so
/// `size > nominal * 1.1` is the construction-time analogue of `fontsize` > 110%.
fn descendant_has_display_font(document: &Document, root: &Node, nominal: f64) -> bool {
  if let Some(fontid) = root.get_attribute("_font")
    && document
      .decode_font(&fontid)
      .and_then(Font::get_size)
      .is_some_and(|size| size > nominal * 1.1)
  {
    return true;
  }
  root.get_child_nodes().iter().any(|c| {
    c.get_type() == Some(NodeType::ElementNode) && descendant_has_display_font(document, c, nominal)
  })
}

/// True if `node` has at least one element child.
fn has_element_child(node: &Node) -> bool {
  node
    .get_child_nodes()
    .iter()
    .any(|c| c.get_type() == Some(NodeType::ElementNode))
}

/// Where a frontmatter flush lands relative to what the document already holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterAnchor {
  /// Right after the resources, before everything else — Perl's fallback placement.
  Top,
  /// Right after the document's leading run of first-group elements (a directly built
  /// `<titlepage>`, a promoted `<title>`), so the frontmatter group stays contiguous.
  LeadingFrontmatter,
}

/// Attribute marking a root child built before `\begin{document}` (see
/// [`mark_preamble_residue`]). Internal (`_`-prefixed): stripped from the output.
const PREAMBLE_RESIDUE: &str = "_preamble";

/// Mark every element child of the document root as PREAMBLE residue. Called by the
/// `\begin{document}` constructor when the root already exists — the preamble opened
/// it, so each child was typeset where LaTeX forbids typesetting (`\@nodocument`).
/// [`place_frontmatter`] moves such nodes below the frontmatter and never wraps them
/// as a cover (OXIDIZED_DESIGN #262).
pub fn mark_preamble_residue(root: &Node) {
  for mut child in root.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode)
      && document::get_node_qname(&child) != pin_static("ltx:resource")
    {
      let _ = child.set_attribute(PREAMBLE_RESIDUE, "true");
    }
  }
}

/// End the paragraph the flush interrupts: close the open elements between the
/// insertion point and the document root when every one of them auto-closes (a `p`
/// in a `para`) and none is a frontmatter-group element; otherwise leave the point
/// alone. An open `{titlepage}` is not a paragraph — `\begin{titlepage}\maketitle
/// \end{titlepage}` relies on the `\unwind@titlepage` hook finding it open (closing
/// it here left an empty `<titlepage/>` behind).
fn close_auto_closable_to_root(document: &mut Document, root: &Node) -> Result<()> {
  let mut n = document.get_node().clone();
  if n.get_type() != Some(NodeType::ElementNode) {
    match n.get_parent() {
      Some(p) => n = p,
      None => return Ok(()),
    }
  }
  let mut cur = n;
  while &cur != root {
    if cur.get_type() != Some(NodeType::ElementNode)
      || !document::can_auto_close(&cur)
      || is_frontmatter_group_element(&cur)
    {
      return Ok(());
    }
    match cur.get_parent() {
      Some(p) => cur = p,
      None => return Ok(()),
    }
  }
  document.close_to_node(root, true)
}

/// Was `node` built before `\begin{document}`? ([`mark_preamble_residue`])
fn is_preamble_residue(node: &Node) -> bool { node.has_attribute(PREAMBLE_RESIDUE) }

/// The ONE frontmatter placement pass (OXIDIZED_DESIGN #247). The schema wants every
/// frontmatter-group element before all `document.body.class` content
/// (`LaTeXML-structure.rnc:34`), and `Para.class` — hence the body group — includes
/// `pagination`, `rule` and `para` (`LaTeXML-para.rnc:18-20`). Perl only ever places at
/// the current point (`\maketitle`) or at the top (fallback) and leaves whatever that
/// yields. This pass looks at the LEADING REGION — the root children between the
/// resources and the bound (the abstract's own `ltx:_Frontmatter_Capture_` position
/// marker when it has one, else everything built so far) — and classifies each node:
///
/// * a first-group element (`is_frontmatter_group_element`) stays and becomes the
///   anchor candidate;
/// * a content-free node (`node_is_content_free`: a pagebreak, an empty paragraph, an
///   ink-free empty box) or PREAMBLE residue (`is_preamble_residue`: built before
///   `\begin{document}`, where LaTeX forbids typesetting) is left in place and, if it
///   preceded the anchor, moved after the placed frontmatter;
/// * hand-typeset title-page LAYOUT — `para` / demotable `logical-block` / `pagination`
///   with visible ink, `wrap_layout` only (the abstract-only fallback: a cover typeset
///   before a `\begin{abstract}` with no `\maketitle`) — is wrapped, in its exact order,
///   into `<ltx:titlepage>` (`titlepage_model = (FrontMatter.class |
///   SectionalFrontMatter.class | Block.class)*` is the schema's container for exactly
///   that; `para`→`block`, `logical-block` demoted bottom-up as in #244);
/// * anything else — a `<TOC>`, a section, a float, a root `<rule>`, a VISIBLE body
///   paragraph on the `\maketitle` path — ends the region: it stays where it is and the
///   frontmatter is placed ahead of it, so the document still opens with its front
///   group (Perl places at the current point and strands the title; user-approved
///   surpass, OXIDIZED_DESIGN #262).
///
/// The frontmatter is inserted after the wrapped titlepage, else after the leading
/// first-group run (`FrontmatterAnchor::LeadingFrontmatter`) or at the top, via the
/// same `_Capture_` insert→fill→unwrap machinery Perl's fallback uses. Run at every
/// placement point — `\lx@frontmatterhere`, both `\lx@frontmatter@fallback` branches,
/// the `{titlepage}` environment's `after_construct` — so a leading pagebreak before a
/// built titlepage is settled when that titlepage is built, not by a later pass. The
/// bound matters: without the marker a sectionless manual's whole body would read as
/// cover (jourcl's cover LETTER, derivative's after-abstract cover). Superseded by this
/// pass: #242's content-free gate, #245's `\end{document}` relocation, #246's anchor
/// helpers. Kept beside it: the conservative title promotion (`maybe_promote_leading_title`,
/// user-ruled), `maybe_dedup_leading_title_ink` (#6924), `insert_late_frontmatter`, the
/// marker sweep in `insert_frontmatter` and the core `finalize_rec`.
///
/// Witnesses: amsldoc/tkz-doc/tzplot (#242, 14 docs), gitinfo2/gitlog/FrontespizioScudo/
/// toptesi-example-* (#245, 6), toptesi-example-magistrale/tipfr-doc/isosigns-docs/
/// bootstrapicons-docs/derivative/jourcl (#246, 16), the faketitlepage /
/// promote_center_title / svabstract fixtures, `114_streaming_structure` (eager ≡ streaming).
pub fn place_frontmatter(
  document: &mut Document,
  wrap_layout: bool,
  anchor: FrontmatterAnchor,
) -> Result<()> {
  // Drain the raw frontmatter queue first, as `insert_frontmatter` does, so
  // `frontmatter_pending` and the flush agree: the `{titlepage}` environment's
  // `after_construct` is the one placement point without its own digest hook, and
  // under `--streaming` (digestion interleaved with construction) a `\title` given
  // after `\end{titlepage}` may still sit undigested there.
  digest_front_matter()?;
  let Some(root) = document.get_document().get_root_element() else {
    return insert_frontmatter(document);
  };
  let kids: Vec<Node> = root
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  let mut i = 0;
  while i < kids.len() && document::get_node_qname(&kids[i]) == pin_static("ltx:resource") {
    i += 1;
  }
  let bound = kids[i..]
    .iter()
    .position(|n| document::get_node_qname(n) == pin_static("ltx:_Frontmatter_Capture_"))
    .map_or(kids.len(), |k| i + k);
  // The leading prefix: first-group elements and content-free nodes, in any order.
  let mut last_first: Option<Node> = None;
  let mut movers: Vec<Node> = Vec::new();
  while i < bound {
    if is_frontmatter_group_element(&kids[i]) {
      last_first = Some(kids[i].clone());
    } else if node_is_content_free(&kids[i]) || is_preamble_residue(&kids[i]) {
      movers.push(kids[i].clone());
    } else {
      break;
    }
    i += 1;
  }
  // Hand-typeset cover before the abstract → <titlepage>, in order.
  let mut titlepage: Option<Node> = None;
  if wrap_layout && i < bound {
    let end = i
      + kids[i..bound]
        .iter()
        .take_while(|k| is_titlepage_layout(k))
        .count();
    if kids[i..end].iter().any(|k| !node_is_content_free(k)) {
      // The flush ends the paragraph it interrupts before the cover moves: text right
      // before an abstract is still the open paragraph, and wrapping it open left the
      // insertion point inside the renamed `block` (`Error:malformed:ltx:document` at
      // `\end{document}`; serbian-def-cyr/proba).
      close_auto_closable_to_root(document, &root)?;
      titlepage = Some(wrap_as_titlepage(document, &root, &kids[i..end])?);
    }
  }
  // Nothing queued (or the abstract-only deferral): `insert_frontmatter` keeps its
  // side effects; a leading pagebreak before a built titlepage still settles here.
  if !frontmatter_pending() {
    insert_frontmatter(document)?;
    if let Some(tail) = last_first {
      move_content_free_after(&movers, tail)?;
    }
    return Ok(());
  }
  // Visible body content before the flush (prose or a class-drawn cover before
  // `\maketitle`, a section before a late `\title`) does not hold the frontmatter
  // back: the schema's document model puts the frontmatter group first
  // (`LaTeXML-structure.rnc:34`), so it goes to the head and that content follows
  // it in its own order. Perl places at the current point (`\maketitle`,
  // Base_Utility.pool.ltxml:918), stranding `<title>` after the body — schema-invalid
  // (user-approved surpass 2026-09-23, OXIDIZED_DESIGN #262). The flush ends the
  // paragraph it interrupts, as every flush point does in LaTeX (`\maketitle`:
  // article.cls:203 opens with `\par`, report/book with `\begin{titlepage}`;
  // `\section`; `\end{document}`) and as the current-point insertion did by
  // auto-closing: text after `\maketitle` starts a new paragraph instead of
  // continuing one that now stands after the frontmatter.
  close_auto_closable_to_root(document, &root)?;
  let after = titlepage.or(match anchor {
    FrontmatterAnchor::Top => None,
    FrontmatterAnchor::LeadingFrontmatter => last_first.clone(),
  });
  let placed = insert_frontmatter_at(document, after.as_ref())?;
  // Content-free nodes that preceded the placed run now follow it, in their order.
  let tail = match (placed.last().cloned(), last_first) {
    (Some(p), Some(f)) => Some(if node_precedes(&p, &f) { f } else { p }),
    (p, f) => p.or(f),
  };
  if let Some(tail) = tail {
    move_content_free_after(&movers, tail)?;
  }
  Ok(())
}

/// A `{titlepage}` built AFTER body content began cannot be the schema's leading
/// `ltx:titlepage` (`document_model` admits it only in the front group): a leaked
/// preamble argument, an undefined-command marker or genuine prose before it has
/// already opened the body, so the element stands stranded — `element "titlepage"
/// not allowed here` — in both engines (Perl's `{titlepage}` is the same
/// `<ltx:titlepage>#body` constructor, latex_constructs.pool.ltxml:1167, with only an
/// Info: "Frontmatter will not be well-structured"; verified on the minimal repro:
/// identical output, one jing error). Its content is hand-typeset layout, so it is
/// demoted in place to an `ltx:para` carrying `class="ltx_titlepage"` — the same
/// text in the same order, no wrapper the schema forbids (OXIDIZED_DESIGN #257;
/// chemexec_de/en, stanli, l2picfaq, pst-calendar-doc, classicthesis in sweep s110).
/// A titlepage in its proper leading position, or one whose children `ltx:para`
/// cannot hold (a `\maketitle` unwound inside it), is left alone.
pub fn demote_stranded_titlepage(document: &mut Document) -> Result<()> {
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(());
  };
  let kids: Vec<Node> = root
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  let Some(pos) = kids
    .iter()
    .rposition(|n| document::get_node_qname(n) == pin_static("ltx:titlepage"))
  else {
    return Ok(());
  };
  let stranded = kids[..pos].iter().any(|k| {
    document::get_node_qname(k) != pin_static("ltx:resource")
      && !is_frontmatter_group_element(k)
      && !node_is_content_free(k)
  });
  if !stranded {
    return Ok(());
  }
  let titlepage = kids[pos].clone();
  // The same child normalization `wrap_as_titlepage` applies: a `center`'s
  // `logical-block` and a `para` become `block`s (#244's demotion), which
  // `ltx:para` — like `ltx:titlepage` — admits. Only when every such child
  // SURVIVES the demotion (`subtree_is_block_demotable`: no float/figure
  // inside — stanli's cover holds `\begin{figure}`s in its `center`); else the
  // titlepage is left as Perl leaves it, nothing touched.
  let children: Vec<Node> = titlepage
    .get_child_nodes()
    .into_iter()
    .filter(|c| c.get_type() == Some(NodeType::ElementNode))
    .collect();
  // Decide BEFORE mutating: every layout child must survive the demotion, and
  // every child — as it will be after it (`logical-block`/`para` → `block`) —
  // must be something `ltx:para` holds. Otherwise nothing is touched.
  let layout = |q: SymStr| q == pin_static("ltx:logical-block") || q == pin_static("ltx:para");
  let demotable = children
    .iter()
    .all(|c| !layout(document::get_node_qname(c)) || subtree_is_block_demotable(c));
  let holdable = children.iter().all(|c| {
    let q = document::get_node_qname(c);
    layout(q) || document::can_contain_qsym(pin_static("ltx:para"), q)
  });
  if !(demotable && holdable) {
    return Ok(());
  }
  normalize_titlepage_layout_children(document, &titlepage)?;
  let mut para = document.rename_node(titlepage, "ltx:para", true)?;
  document.add_class(&mut para, "ltx_titlepage")?;
  Ok(())
}

/// Would `insert_frontmatter` place anything right now? Mirrors its three early
/// returns (already done; nothing queued; the abstract-only first-call deferral).
fn frontmatter_pending() -> bool {
  if lookup_bool("frontmatter_done") {
    return false;
  }
  let keys: Vec<String> = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.keys().cloned().collect(),
    _ => Vec::new(),
  });
  !(keys.is_empty()
    || (keys.len() == 1 && keys[0] == "ltx:abstract" && !lookup_bool("frontmatter_deferred")))
}

/// A root child that is title-page layout when it stands before the abstract: a
/// pagebreak, a paragraph, or a `logical-block` whose content survives #244's demotion
/// to `<block>`.
fn is_titlepage_layout(node: &Node) -> bool {
  let q = document::get_node_qname(node);
  q == pin_static("ltx:pagination")
    || q == pin_static("ltx:para")
    || (q == pin_static("ltx:logical-block") && subtree_is_block_demotable(node))
}

/// Move `run` (consecutive root children) into a new `<ltx:titlepage>` in place, in
/// order; `para` → `block` as is (its content is already Block.model), `logical-block`
/// demoted bottom-up then renamed. Returns the titlepage (its own handle stays live —
/// only children are renamed).
fn wrap_as_titlepage(document: &mut Document, root: &Node, run: &[Node]) -> Result<Node> {
  // Bare tag + root namespace, as `maybe_promote_leading_title` does for <title>.
  let mut titlepage = document.insert_element_before(&run[0], "titlepage", None)?;
  if let Some(rns) = root.get_namespace() {
    let _ = titlepage.set_namespace(&rns);
  }
  for mut n in run.iter().cloned() {
    n.unlink();
    titlepage.add_child(&mut n)?;
  }
  normalize_titlepage_layout_children(document, &titlepage)?;
  Ok(titlepage)
}

/// The layout children of a title page as the schema's `titlepage_model`
/// (`Block.class*`) — and `ltx:para` — hold them: a `center`'s `logical-block`
/// and a `para` become `block`s (#244's demotion, content first, rename after).
/// Shared by `wrap_as_titlepage` and `demote_stranded_titlepage`.
fn normalize_titlepage_layout_children(document: &mut Document, titlepage: &Node) -> Result<()> {
  for child in titlepage.get_child_nodes() {
    if child.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let q = document::get_node_qname(&child);
    if q == pin_static("ltx:logical-block") {
      demote_para_class_content(document, &child)?;
      document.rename_node(child, "ltx:block", true)?;
    } else if q == pin_static("ltx:para") {
      document.rename_node(child, "ltx:block", true)?;
    }
  }
  Ok(())
}

/// Flush the queued frontmatter right after `after` (a root child), or — `None` — at
/// the top, after `/ltx:document/ltx:resource[last()]` exactly as Perl's fallback
/// (`Base_Utility.pool.ltxml:927-945`; Perl's `findnode` is document-relative, the Rust
/// cached XPath context is root-relative, hence the absolute path). The `_Capture_`
/// wrapper is filled by `insert_frontmatter` and unwrapped; the placed elements are
/// returned (their handles survive the unwrap — `unwrap_nodes` splices children up).
fn insert_frontmatter_at(document: &mut Document, after: Option<&Node>) -> Result<Vec<Node>> {
  let savenode = document.get_node().clone();
  let point = match after {
    Some(a) => a.get_next_sibling(),
    None => document
      .findnode("/ltx:document/ltx:resource[last()]", None)
      .and_then(|r| r.get_next_sibling()),
  };
  let wrapper = match point {
    Some(point) => document.insert_element_before(&point, "ltx:_Capture_", None)?,
    None => {
      let parent = match after {
        Some(a) => a.get_parent(),
        None => document.get_document().get_root_element(),
      };
      match parent {
        Some(mut parent) => document.open_element_at(&mut parent, "ltx:_Capture_", None, None)?,
        None => return Ok(Vec::new()),
      }
    },
  };
  document.set_node(&wrapper);
  insert_frontmatter(document)?;
  let placed: Vec<Node> = wrapper
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  document.unwrap_nodes(wrapper)?;
  document.set_node(&savenode);
  Ok(placed)
}

/// Re-attach, in order, every node of `movers` that precedes `tail` to right after it.
/// Elements only, so `add_next_sibling` never meets libxml2's text-merge-and-free.
fn move_content_free_after(movers: &[Node], mut tail: Node) -> Result<()> {
  for mut m in movers.iter().cloned() {
    if node_precedes(&m, &tail) {
      m.unlink();
      tail.add_next_sibling(&mut m)?;
      tail = m;
    }
  }
  Ok(())
}

/// True when `a` comes before `b` among the same parent's children.
fn node_precedes(a: &Node, b: &Node) -> bool {
  let mut cur = a.get_next_sibling();
  while let Some(n) = cur {
    if n == *b {
      return true;
    }
    cur = n.get_next_sibling();
  }
  false
}

/// A `<ltx:pagination>` (self-closing pagebreak) or a truly empty `<ltx:para>`/
/// `<ltx:p>`/`<ltx:break>` wrapper — a node with no visible content. "Empty" means no
/// text AND no *visible* element descendant: a `<graphics>`/`<rule>`/`<svg>`/
/// `<tabular>`/`<text>`/image inside an otherwise text-empty para (a logo- or
/// rule-only cover top) renders above the title in the PDF, so it is NOT content-free
/// and must block the hoist — only nested empty `<para>`/`<p>`/`<break>` wrappers are
/// transparent. See `place_frontmatter`.
fn node_is_content_free(node: &Node) -> bool {
  let is_pagination = with(document::get_node_qname(node), |q| q == "ltx:pagination");
  if is_pagination {
    return true;
  }
  let is_wrapper = with(document::get_node_qname(node), |q| {
    matches!(q, "ltx:para" | "ltx:p" | "ltx:break")
  });
  is_wrapper
    && text_outside_error_markers(node).trim().is_empty()
    && subtree_elements_all_invisible(node)
}

/// The text of `node`'s subtree with every `ltx:ERROR` subtree left out. An
/// undefined-command marker (`<ERROR class="undefined">\foo</ERROR>`) carries the
/// control-sequence NAME as its text — diagnostic ink, not document content (pdflatex
/// had a definition and typeset nothing for it). A leading paragraph holding only such
/// markers is therefore content-free for the frontmatter hoist (OXIDIZED_DESIGN #258;
/// pst-calendar-doc's `\DeclareDocumentMetadata`, forest-doc's `\@escapeifif`,
/// pmhanguljamo's `\fontid`, 8 docs): the marker stays in the output, relocated below
/// the frontmatter like any other content-free mover. A marker whose ARGUMENTS were
/// typeset as text (`\foo{Real words}` → `<ERROR>\foo</ERROR>Real words`) is not
/// content-free — that text is visible and stays where it is.
fn text_outside_error_markers(node: &Node) -> String {
  fn walk(node: &Node, out: &mut String) {
    for c in node.get_child_nodes() {
      match c.get_type() {
        Some(NodeType::TextNode) => out.push_str(&c.get_content()),
        Some(NodeType::ElementNode) if !is_undefined_command_marker(&c) => {
          walk(&c, out);
        },
        _ => {},
      }
    }
  }
  let mut out = String::new();
  walk(node, &mut out);
  out
}

/// `<ltx:ERROR class="undefined">\foo</ltx:ERROR>` — the marker `make_error` leaves for
/// an undefined control sequence; its text is the CS NAME. Only this class is
/// diagnostic ink: `\lx@ERROR{cls}{text}` markers and constructor failures carry
/// arbitrary text and stay visible content.
fn is_undefined_command_marker(node: &Node) -> bool {
  document::get_node_qname(node) == pin_static("ltx:ERROR")
    && node.get_attribute("class").as_deref() == Some("undefined")
}

/// Every element descendant of `node` is itself a structural wrapper
/// (`ltx:para`/`ltx:p`/`ltx:break`, or an ink-free `ltx:text`) — i.e. the subtree
/// renders nothing visible. Any other element (`ltx:graphics`, `ltx:rule`, `ltx:svg`,
/// `ltx:tabular`, an image, …) is visible content even with no text. An `ltx:text` is
/// transparent only when it can draw nothing on its own: the empty `<ltx:text>` an
/// `\hbox{}`/`\null`/`\mbox{}` leaves in a paragraph (`\cleardoublepage`'s blank
/// page; memoir `\frontmatter`) is folded away by `auto_collapse_children`/`finalize`
/// later anyway, but a `\fbox{}`/`\colorbox{..}{}` — `framed`/`backgroundcolor`, or a
/// `class`/`cssstyle` a stylesheet may paint — has ink and stays visible. Text nodes
/// are ignored here: the caller has already required the subtree's text to be
/// whitespace-only, and LaTeXML wraps loose document text in `<para><p>`, so a bare
/// non-whitespace text child does not occur at this level.
fn subtree_elements_all_invisible(node: &Node) -> bool {
  node.get_child_nodes().iter().all(|c| {
    c.get_type() != Some(NodeType::ElementNode)
      // An undefined-command marker is diagnostic ink (`text_outside_error_markers`).
      || is_undefined_command_marker(c)
      || ((with(document::get_node_qname(c), |q| {
        matches!(q, "ltx:para" | "ltx:p" | "ltx:break")
      }) || (document::get_node_qname(c) == pin_static("ltx:text") && text_is_ink_free(c)))
        && subtree_elements_all_invisible(c))
  })
}

/// An `ltx:text` with none of the attributes that can paint without content.
fn text_is_ink_free(text: &Node) -> bool {
  ![
    "framed",
    "framecolor",
    "backgroundcolor",
    "class",
    "cssstyle",
  ]
  .iter()
  .any(|attr| text.has_attribute(attr))
}

/// Is `node` one of the document model's FIRST-group elements (`LaTeXML-structure.rnc:34`),
/// read from the schema: admitted by `ltx:document` but not by body-only content
/// (`ltx:sectional-block`, model `document.body.class*`), and — since `ltx:document` also
/// admits BackMatter.class (bibliography/appendix/index/glossary) that the body group
/// takes — either `titlepage` or something `ltx:bibliography` admits (its model LEADS
/// with exactly `FrontMatter.class*, SectionalFrontMatter.class*`). A Meta.class element
/// (allowed in both groups) is NOT one. Used by `place_frontmatter` (#247).
fn is_frontmatter_group_element(node: &Node) -> bool {
  let q = document::get_node_qname(node);
  document::can_contain_qsym(pin_static("ltx:document"), q)
    && !document::can_contain_qsym(pin_static("ltx:sectional-block"), q)
    && (q == pin_static("ltx:titlepage")
      || document::can_contain_qsym(pin_static("ltx:bibliography"), q))
}

/// Beyond-Perl heuristic: recover a document title from a hand-formatted leading
/// block when the author never used `\title`/`\maketitle`.
///
/// Many papers set their title as a plain `\begin{center}{\Large ...}\end{center}`
/// block instead of the frontmatter machinery, then declare only an abstract (e.g.
/// arXiv 1609.07638). With no `\title` there is no `ltx:title` frontmatter to
/// flush, so the document renders titleless. This promotes that first block — gated
/// conservatively so it does not fire on epigraphs, dedications, centered figures or
/// "draft" notices — into a real `<ltx:title>` at the document top.
///
/// Called only from the ABSTRACT-ONLY `\lx@frontmatter@fallback` branch: reaching it
/// already means no `\title`/`\author`/`\maketitle` were used yet the paper carries
/// frontmatter (an abstract) — the strongest signal a leading display block is the
/// title. It fires only when no `<ltx:title>` exists yet, the first non-resource body
/// element holds a leading centered `<ltx:p>` set in a larger-than-body font, and
/// that paragraph has non-whitespace text. On a match the paragraph's inline children
/// are MOVED into a fresh `<ltx:title>` after the resources, pruning empty wrappers.
fn maybe_promote_leading_title(document: &mut Document) -> Result<Option<Node>> {
  // Never override an existing (real) title.
  if document.findnode("/ltx:document/ltx:title", None).is_some() {
    return Ok(None);
  }
  // The first non-resource body element (the candidate title block). Fetch it
  // with an ABSOLUTE query: nodes returned from a RELATIVE-context findnode are
  // detached for child traversal (a rust-libxml shared-node artifact —
  // get_content works but get_child_nodes is empty), so every step below walks
  // the live DOM by hand from this anchor.
  let Some(first_body) = document.findnode("/ltx:document/*[not(self::ltx:resource)][1]", None)
  else {
    return Ok(None);
  };
  // Never DESCEND INTO SECTIONAL content: on the second fallback pass (fired
  // at the next \section) the first body element can be the COMPLETED first
  // section, and an unconstrained DFS would steal a centered display-font
  // paragraph from inside it (e.g. a "Dedicated to ..." block) as the paper
  // title. A hand-formatted title block is a plain para/quote at document
  // level — reject sectional anchors outright (PR_READINESS must-fix 3).
  let anchor_q = document::get_node_qname(&first_body);
  let is_sectional = with(anchor_q, |q| {
    matches!(
      q,
      "ltx:section"
        | "ltx:subsection"
        | "ltx:subsubsection"
        | "ltx:paragraph"
        | "ltx:subparagraph"
        | "ltx:chapter"
        | "ltx:part"
        | "ltx:appendix"
        | "ltx:bibliography"
        | "ltx:index"
        | "ltx:glossary"
    )
  });
  if is_sectional {
    return Ok(None);
  }
  // First centered <ltx:p> in document order within the block; it must be set in
  // a larger-than-body font (the display-title signal) and hold real text.
  // Conservative rule (user ruling 2026-09-20, OXIDIZED_DESIGN #246): promote ONLY the
  // unambiguous hand-made-title shape — a plain `center`/paragraph block at document
  // level whose FIRST paragraph is centered and set in a display font that NO other
  // paragraph of the block shares. Corpus counter-examples that a looser rule mis-read
  // as titles: tikz-mirror-lens (author-first cover, `\Large{#1}` leaking onto the
  // title paragraph → two display paragraphs → ambiguous), isosigns/bootstrapicons
  // (a `VERSION 2.1` badge inside a tcolorbox → nested box), tipfr-doc (its real
  // `tipfr.sty` title also sits in a tcolorbox — declined rather than coin-flipped).
  // Whatever is not promoted stays hand-typeset layout and is wrapped as
  // `<ltx:titlepage>` by `place_frontmatter (the titlepage wrap)`.
  let is_plain_block = with(anchor_q, |q| matches!(q, "ltx:logical-block" | "ltx:para"));
  if !is_plain_block {
    return Ok(None);
  }
  let Some(title_p) = first_plain_paragraph(&first_body) else {
    return Ok(None);
  };
  if title_p.get_attribute("align").as_deref() != Some("center") {
    return Ok(None);
  }
  let nominal = {
    // #542: NOMINAL_FONT_SIZE is a float (11pt = 10.95), not an int — mirror
    // `common::font::defsize`, which reads it via lookup_float, not lookup_int.
    let v = lookup_float("NOMINAL_FONT_SIZE").map_or(0.0, |f| f.0);
    if v > 0.0 { v } else { 10.0 }
  };
  if !title_p.get_content().chars().any(char::is_alphabetic)
    || !descendant_has_display_font(document, &title_p, nominal)
  {
    return Ok(None);
  }
  // Uniqueness: any OTHER plain paragraph of the block in a display font makes the
  // title ambiguous (author-first covers, multi-line banners).
  let mut paragraphs = Vec::new();
  collect_plain_paragraphs(&first_body, &mut paragraphs);
  if paragraphs
    .iter()
    .any(|p| *p != title_p && descendant_has_display_font(document, p, nominal))
  {
    return Ok(None);
  }
  // Create <ltx:title> at the document top (after the last resource), mirroring
  // the frontmatter fallback's placement.
  let mut point = document.findnode("/ltx:document/ltx:resource[last()]", None);
  if let Some(p) = point.take() {
    point = p.get_next_sibling();
  }
  // No resources (or resource is the last child): insert before the first
  // body element instead of silently declining (review corner-case).
  let point = point.unwrap_or_else(|| first_body.clone());
  // Create with a BARE tag + bind the root (default LaTeXML) namespace, so it
  // serializes as <title> like a real frontmatter title — a prefixed "ltx:title"
  // qname would emit a stray <ltx:title>. Mirrors open_element_internal's
  // default-namespace path.
  let mut title = document.insert_element_before(&point, "title", None)?;
  if let Some(rns) = document
    .get_document()
    .get_root_element()
    .and_then(|r| r.get_namespace())
  {
    let _ = title.set_namespace(&rns);
  }
  // MOVE the paragraph's inline children into the title (a true move preserves
  // xml:ids, so any footnotes/marks carry over rather than being cloned+orphaned).
  for mut child in title_p.get_child_nodes() {
    child.unbind();
    title.add_child(&mut child)?;
  }
  // Drop the now-empty paragraph, and any wrapper ancestor it leaves empty
  // (para → logical-block), so we don't strand empty blocks above the abstract.
  let mut victim = title_p;
  loop {
    let parent = victim.get_parent();
    document.remove_node(victim);
    match parent {
      Some(p) if !has_element_child(&p) => {
        let is_wrapper = with(document::get_node_qname(&p), |name| {
          name == "ltx:para" || name == "ltx:logical-block"
        });
        if is_wrapper {
          victim = p;
          continue;
        }
      },
      _ => {},
    }
    break;
  }
  // The promoted <title>: the node a deferred frontmatter flush lands AFTER when no
  // titlepage layout follows it. A surviving rest of the hand-formatted block (an
  // author paragraph) is wrapped as <ltx:titlepage> by the caller. `first_body` may
  // have been freed above — never return it.
  Ok(Some(title))
}

/// Whitespace-collapsed, lowercased text — so a `<break>` (`\\`) and any spacing
/// differences between the structured title and its hand-typeset copy don't
/// defeat the comparison in [`maybe_dedup_leading_title_ink`].
fn normalize_frontmatter_text(s: &str) -> String {
  s.split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase()
}

/// Companion to [`maybe_promote_leading_title`] for the mirror case
/// (arXiv/html_feedback#6924, witness arXiv 2608.10928): a structured
/// `<ltx:title>` DOES exist (from `\title`), but the author ALSO hand-typeset the
/// title as a leading centered display-font block and never called `\maketitle`,
/// so that block reproduces the structured title and the title renders twice.
///
/// Prioritize the structured metadata (LaTeXML's unified Frontmatter API stays
/// authoritative): remove the redundant leading title *ink*, keeping the semantic
/// `<ltx:title>` and any author/abstract ink (which has no structured counterpart,
/// so is the only copy). Fires only on a FULL normalized-text match against the
/// structured title, and only for a leading, non-sectional, display-font centered
/// paragraph — so an unrelated centered display block is never removed. Reuses the
/// same detection helpers as the promote path.
fn maybe_dedup_leading_title_ink(document: &mut Document) -> Result<()> {
  // Needs a structured document title to prioritize over.
  let Some(title) = document.findnode("/ltx:document/ltx:title", None) else {
    return Ok(());
  };
  let title_text = normalize_frontmatter_text(&title.get_content());
  if title_text.is_empty() {
    return Ok(());
  }
  let nominal = {
    let v = lookup_float("NOMINAL_FONT_SIZE").map_or(0.0, |f| f.0);
    if v > 0.0 { v } else { 10.0 }
  };
  // Leading centered paragraphs — anywhere in the pre-first-section frontmatter
  // region, NOT inside a section (a hand-formatted title block sits at document
  // level, often after a `\vspace`/`\rule` so it isn't the first body element).
  // Absolute query so the returned nodes support child traversal (shared-node
  // caveat in `maybe_promote_leading_title`).
  let candidates = document.findnodes(
    "/ltx:document//ltx:p[@align='center']\
     [not(preceding::ltx:section) and not(ancestor::ltx:section)]",
    None,
  );
  // Remove the FIRST leading centered display-font block that EXACTLY reproduces
  // the structured title. Only one — a paper may repeat the title text later
  // (e.g. a running head); we drop just the redundant hand-typeset title.
  let Some(victim) = candidates.into_iter().find(|p| {
    descendant_has_display_font(document, p, nominal)
      && normalize_frontmatter_text(&p.get_content()) == title_text
  }) else {
    return Ok(());
  };
  // Log what we drop (never remove content silently; #6924).
  Info!(
    "frontmatter",
    "title_ink_dedup",
    s!(
      "dropped a hand-typeset title block duplicating the structured <ltx:title> (\"{title_text}\")"
    )
  );
  // Remove the redundant title ink, pruning any wrapper it leaves empty
  // (para → logical-block) — mirroring the promote path's cleanup. Sibling `<p>`s
  // (e.g. the author block) keep their wrapper non-empty and survive.
  let mut victim = victim;
  loop {
    let parent = victim.get_parent();
    document.remove_node(victim);
    match parent {
      Some(p) if !has_element_child(&p) => {
        let is_wrapper = with(document::get_node_qname(&p), |name| {
          name == "ltx:para" || name == "ltx:logical-block"
        });
        if is_wrapper {
          victim = p;
          continue;
        }
      },
      _ => {},
    }
    break;
  }
  Ok(())
}

/// Insert FrontMatter into document, if not already added
/// Perl: insertFrontMatter($document).
/// Drop every `ltx:_Frontmatter_Capture_` position marker (`\lx@frontmatter@mark`);
/// called once the frontmatter is placed and again at `\end{document}` as a safety net.
pub fn remove_frontmatter_marks(document: &mut Document) {
  for mark in document.findnodes("//ltx:_Frontmatter_Capture_", None) {
    document.remove_node(mark);
  }
}

pub fn insert_frontmatter(document: &mut Document) -> Result<()> {
  if lookup_bool("frontmatter_done") {
    return Ok(());
  }
  // A raw class's title-page stores in a document without `\maketitle` (frontmatter_stores.rs).
  crate::frontmatter_stores::harvest_stores(false)?;
  digest_front_matter()?; // If needed
  let frontmatter_elements_set: HashSet<String> = FRONTMATTER_ELEMENTS
    .iter()
    .map(ToString::to_string)
    .collect();

  // Collect the frontmatter hash keys via with_value — we only need the
  // key set here; the full HashTagData clone previously happened just
  // to call .keys().cloned().collect(). The hash itself is consumed a
  // few lines below via remove_value, so no iteration on the borrow
  // survives past this closure.
  let set_keys: Vec<String> = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.keys().cloned().collect(),
    _ => Vec::new(),
  });
  if set_keys.is_empty() {
    return Ok(());
  }

  // If doc ONLY has abstract as frontmatter, defer until abstract's document location
  if set_keys.len() == 1 && set_keys[0] == "ltx:abstract" && !lookup_bool("frontmatter_deferred") {
    assign_value("frontmatter_deferred", true, Some(Scope::Global));
    return Ok(());
  }

  // OK, we're placing FrontMatter here, now.
  assign_value("frontmatter_done", true, Some(Scope::Global));
  remove_frontmatter_marks(document);

  // Remove frontmatter and replace with empty
  let mut frontmatter = match remove_value("frontmatter") {
    Some(Stored::HashTagData(frnt)) => frnt,
    _ => return Ok(()),
  };
  assign_value(
    "frontmatter",
    Stored::HashTagData(HashMap::default()),
    Some(Scope::Global),
  );

  // Order: first go through frontmatter_elements order, then any custom keys
  let custom_keys: Vec<String> = frontmatter
    .keys()
    .filter(|key| !frontmatter_elements_set.contains(key.as_str()))
    .map(ToString::to_string)
    .collect();
  let mut all_keys: Vec<String> = FRONTMATTER_ELEMENTS
    .iter()
    .map(ToString::to_string)
    .collect();
  all_keys.extend(custom_keys);

  // A `\maketitle` run inside a box capture (ltx-talk.cls:515 frames, unifront,
  // `\parbox{…}{\maketitle}`) no longer reaches here trapped: `place_frontmatter`
  // always inserts through a root-level `_Capture_` (#262), so the frontmatter lands
  // at the head as real elements. (Before 56gf a current-point flush degraded each
  // entry to `ltx:text class="ltx_<name>"` inside the box.) Guard
  // `perfect_kernel_batch54::maketitle_inside_a_box_goes_to_the_head`.
  for key in &all_keys {
    if let Some(list) = frontmatter.remove(key) {
      // Dubious, but assures that frontmatter appears in text mode...
      document.set_box_to_absorb(
        Tbox::new(
          pin!(""),
          lookup_font(),
          None,
          Tokens!(T_SPACE!()),
          SymHashMap::default(),
        )
        .into(),
      );
      for item in list {
        insert_frontmatter_entry(document, &item)?;
      }
      document.expire_box_to_absorb();
    }
  }
  coalesce_empty_creators(document)?;
  distribute_upfront_contacts(document)?;
  relocate_annotations(document)?;
  flag_merged_creators(document)?;
  Ok(())
}

/// Remove author creators whose `<ltx:personname>` is empty, moving any contacts they
/// carry to the preceding real creator (else the first following one).
///
/// A flat comma author list with interspersed `\IEEEmembership{…}`/`\thanks{…}` — e.g.
/// `\author{Alice, \IEEEmembership{…}, and Bob, …\thanks{…}}` (html_feedback#4539,
/// witness 2508.00603) — comma/" and "-splits into pieces where the membership pieces
/// digest to nothing, surfacing as empty `<ltx:personname/>` creators; a trailing
/// `\thanks` then strands its affil/email on a nameless creator. This coalesces those:
/// contactless empties are dropped; a contact-bearing empty's contacts move to the
/// preceding real author. `\footnotemark`-note markers keep a personname non-empty
/// (2507.06670 "Yu Zhang"), so real authors are untouched.
///
/// Moves BOTH `<ltx:contact>` and `<ltx:note>` annotations: an author `\thanks` is a
/// marked `<ltx:note role="thanks">` (OXIDIZED_DESIGN #156), so a trailing `\thanks` on
/// a nameless comma-split creator would otherwise be dropped with the empty creator —
/// witness 1510.02728 (`\author{Sani,~\IEEEmembership{…} Vosoughi,~\IEEEmembership{…}%
/// \thanks{…NSF…}}`), where the note must land on the last real author, as a contact did.
fn coalesce_empty_creators(document: &mut Document) -> Result<()> {
  let creators = document.findnodes("//ltx:creator[@role='author']", None);
  let mut to_remove: Vec<Node> = Vec::new();
  let mut last_real: Option<Node> = None;
  // Annotations (contacts + notes) of leading empties (before any real author), held
  // until the first real one.
  let mut orphan_annotations: Vec<Node> = Vec::new();
  for creator in creators {
    if creator_personname_empty(document, &creator) {
      let annotations: Vec<Node> = creator
        .get_child_nodes()
        .into_iter()
        .filter(|c| {
          c.get_type() == Some(NodeType::ElementNode)
            && with(document::get_node_qname(c), |q| {
              q == "ltx:contact" || q == "ltx:note"
            })
        })
        .collect();
      if let Some(ref mut prev) = last_real {
        if !annotations.is_empty() {
          document.append_clone(prev, annotations)?;
        }
      } else {
        orphan_annotations.extend(annotations);
      }
      to_remove.push(creator);
    } else {
      if !orphan_annotations.is_empty() {
        let mut first = creator.clone();
        document.append_clone(&mut first, std::mem::take(&mut orphan_annotations))?;
      }
      last_real = Some(creator);
    }
  }
  for creator in to_remove {
    document.remove_node(creator);
  }
  Ok(())
}

/// Redistribute a mis-piled contact block back to its authors (surpass-Perl).
///
/// The amsart idiom `\author{A}\author{B}\author{C}` followed by paired
/// `\address{}\email{}` declares every author up front, then all the contacts.
/// LaTeXML's default "attach a contact to the preceding creator" then bunches
/// EVERY address+email under the LAST author (arXiv:2308.06214v1,
/// arXiv/html_feedback#46); Perl 0.8.8 does the same (SHARED limitation).
///
/// Fix ONLY the clean, unambiguous pile — an `N × m` grid: the other `N-1`
/// authors carry no contact, the last author's `K` contacts split evenly
/// (`K = N·m`) into a role-periodic sequence (`role[i] == role[i+m]`), so group
/// `j` is handed to author `j`. Any irregular pile — a heterogeneous role
/// sequence, per-author counts that differ, or contacts already spread across
/// authors (the interleaved idiom, which is correct as-is) — fails the gate and
/// is left EXACTLY as Perl attached it. Mirrors the "distribute-when-clean, else
/// keep prior" rule of the shared-email splitter (OXIDIZED_DESIGN #52(j)).
fn distribute_upfront_contacts(document: &mut Document) -> Result<()> {
  let creators = document.findnodes("//ltx:creator[@role='author']", None);
  let n = creators.len();
  if n < 2 {
    return Ok(());
  }
  let contacts_of = |c: &Node| -> Vec<Node> {
    c.get_child_nodes()
      .into_iter()
      .filter(|x| {
        x.get_type() == Some(NodeType::ElementNode)
          && with(document::get_node_qname(x), |q| q == "ltx:contact")
      })
      .collect()
  };
  // Signature: only the LAST author carries any contacts.
  for creator in &creators[..n - 1] {
    if !contacts_of(creator).is_empty() {
      return Ok(());
    }
  }
  let last = creators[n - 1].clone();
  let contacts = contacts_of(&last);
  let k = contacts.len();
  if k == 0 || k % n != 0 {
    return Ok(());
  }
  let m = k / n;
  // Role sequence must be periodic with period m (every group has the same
  // role pattern), else the pile is not a clean grid — leave it to Perl's rule.
  let roles: Vec<String> = contacts
    .iter()
    .map(|x| x.get_attribute("role").unwrap_or_default())
    .collect();
  if (0..k - m).any(|i| roles[i] != roles[i + m]) {
    return Ok(());
  }
  // Group j (contacts[j*m .. (j+1)*m]) -> author j. The last group stays put;
  // the earlier groups clone onto their author and the originals are removed.
  let mut to_remove: Vec<Node> = Vec::new();
  for (j, creator) in creators.iter().enumerate().take(n - 1) {
    let group: Vec<Node> = contacts[j * m..(j + 1) * m].to_vec();
    let mut target = creator.clone();
    document.append_clone(&mut target, group.clone())?;
    to_remove.extend(group);
  }
  for node in to_remove {
    document.remove_node(node);
  }
  Ok(())
}

/// True if `creator`'s `<ltx:personname>` carries no name — no element child and no
/// non-whitespace text (a bare `<ltx:personname/>`), or no personname at all. A
/// personname with a real name (text) or a `<ltx:text>`/`<ltx:note>` child is NOT empty.
fn creator_personname_empty(document: &mut Document, creator: &Node) -> bool {
  match document.findnode("ltx:personname", Some(creator)) {
    None => true,
    Some(person) => !person.get_child_nodes().iter().any(|c| {
      c.get_type() == Some(NodeType::ElementNode)
        || (c.get_type() == Some(NodeType::TextNode) && !c.get_content().trim().is_empty())
    }),
  }
}

/// Streaming: insert frontmatter that arrived AFTER the in-document insertion
/// point had already been absorbed.
///
/// Eager conversion runs every `insert_frontmatter` with digestion complete
/// (build starts after digestion ends), so the `\maketitle`-time insertion
/// sees the whole document's frontmatter. Interleaved pass 1 runs it
/// mid-digestion — with the REAL builder navigation, so paragraph splitting
/// and ids come out exactly as eager — but anything queued after that moment
/// (an abstract following `\maketitle` in an AMS-style document; sweep
/// witnesses tests/structure/amsarticle.tex, titlepage.tex) misses the
/// consumption. Those late arrivals re-populate the `frontmatter` state hash;
/// this inserts them at their canonical slot — after the last frontmatter
/// element already placed — by re-running `insert_frontmatter` there
/// (element order within it is canonical, not arrival order).
pub fn insert_late_frontmatter(document: &mut Document) -> Result<()> {
  debug_assert!(!document.root_after_open_deferred());
  let has_late = with_value(
    "frontmatter",
    |v| matches!(v, Some(Stored::HashTagData(h)) if !h.is_empty()),
  ) || with_value(
    "frontmatter_raw",
    |v| matches!(v, Some(Stored::FrontmatterRaw(q)) if !q.is_empty()),
  );
  if !has_late || !lookup_bool("frontmatter_done") {
    // Nothing late, or nothing was ever inserted early — in the latter case
    // the root's deferred late hook performs the one-and-only insertion.
    return Ok(());
  }
  // Anchor: the late elements' CANONICAL slot among the frontmatter already
  // placed — `insert_frontmatter` emits in `FRONTMATTER_ELEMENTS` order, so a
  // late abstract belongs between an existing date and existing keywords
  // (sweep witness tests/structure/amsarticle.tex), not after everything.
  // ROOT CHILDREN only: `//ltx:title` would also match every SECTION's
  // title, and the last of those anchors the insertion deep inside the body.
  // If several late kinds straddle an existing kind they clump at the first
  // late kind's slot — the byte-parity gate guards the day that case
  // materializes.
  let existing = document.findnodes(
    &FRONTMATTER_ELEMENTS
      .iter()
      .map(|q| format!("/ltx:document/{q}"))
      .collect::<Vec<_>>()
      .join(" | "),
    None,
  );
  let Some(last) = existing.last() else {
    return Ok(());
  };
  let rank = |qname: &str| {
    FRONTMATTER_ELEMENTS
      .iter()
      .position(|q| *q == qname)
      .unwrap_or(usize::MAX)
  };
  let min_late_rank = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(h)) => h.keys().map(|k| rank(k)).min(),
    _ => None,
  })
  .unwrap_or(usize::MAX);
  let displaced = existing.iter().find(|node| {
    let qname = format!("ltx:{}", node.get_name());
    rank(&qname) > min_late_rank
  });
  let savenode = document.get_node().clone();
  let wrapper = match displaced {
    // Canonically later frontmatter exists: insert right before it.
    Some(next) => document.insert_element_before(next, "ltx:_Capture_", None)?,
    // Everything placed is canonically earlier: insert after the last.
    None => match last.get_next_sibling() {
      Some(next) => document.insert_element_before(&next, "ltx:_Capture_", None)?,
      None => {
        let mut parent = last
          .get_parent()
          .expect("a frontmatter element always has a parent");
        document.open_element_at(&mut parent, "ltx:_Capture_", None, None)?
      },
    },
  };
  document.set_node(&wrapper);
  assign_value("frontmatter_done", false, Some(Scope::Global));
  insert_frontmatter(document)?;
  document.unwrap_nodes(wrapper)?;
  document.set_node(&savenode);
  Ok(())
}

/// Insert one frontmatter entry `[tag, {attr}, @content]`.
/// Perl: insertFrontMatter_rec($document, $item) for the ARRAY case.
fn insert_frontmatter_entry(document: &mut Document, entry: &TagData) -> Result<()> {
  let TagData { tag, attr, content } = entry;
  DebugFeature!("frontmatter", "FRONT Inserting {}", show_frontmatter(entry));
  // token-locators: frontmatter elements (e.g. <ltx:title> from `\title{…}`)
  // are opened here, far from their source, around content that was digested
  // and stored back at `\lx@add@frontmatter` time. open_element would otherwise
  // stamp them with no/last locator (→ the whole-document fallback in clients).
  // Recover the deferred content's span and stamp the element with it.
  #[cfg(feature = "token-locators")]
  if let Some(TagContent::Box(stuff)) = content.iter().find(|c| matches!(c, TagContent::Box(_))) {
    document.set_current_box_locator(latexml_core::definition::constructor::child_span(stuff));
  }
  // Perl: font => $stuff[0]->getFont, _force_font => 'true' when the tag
  // can have a font attribute and there is content.
  let mut attributes: HashMap<String, String> = attr.clone();
  let mut font: Option<Font> = None;
  if !content.is_empty()
    && document::can_have_attribute(tag, "font")
    && let Some(TagContent::Box(first)) = content.first()
    && let Ok(Some(f)) = first.get_font()
  {
    font = Some((*f).clone());
    attributes.insert("_force_font".to_string(), "true".to_string());
  }
  let from_store = attributes.contains_key("_store");
  let opened = document.open_element(tag, Some(attributes), font.as_ref())?;
  for item in content {
    insert_frontmatter_rec(document, item)?;
  }
  // Scoped close (OXIDIZED_DESIGN #202): a deferred body that ran to the end
  // of the input carries the `\end{document}` whatsit (an unbalanced
  // `\abstract{…`, #207); absorbing it here closes the document — and this
  // element with it — so there is nothing left of ours to close.
  document.close_element_if_open(tag)?;
  // A raw class's store whose value typesets nothing (a default that only warns,
  // umich-thesis.cls:71) carries no datum: its element is not kept (frontmatter_stores.rs).
  if from_store && opened.get_content().trim().is_empty() && opened.get_child_elements().is_empty()
  {
    document.remove_node(opened);
  }
  // At this time, the frontmatter element should really carry the actual literal values intended.
  // (Perl PR #2767 disables the former empty-element pruning here.)
  Ok(())
}

/// Perl: insertFrontMatter_rec($document, $item).
fn insert_frontmatter_rec(document: &mut Document, item: &TagContent) -> Result<()> {
  match item {
    TagContent::Entry(entry) => insert_frontmatter_entry(document, entry)?,
    // Otherwise, assume some sort of Box
    TagContent::Box(digested) => document.absorb(digested, None)?,
    // Perl absorbs the literal 'place_keeper' string for unfilled entries
    TagContent::PlaceKeeper => {
      document.absorb(&Digested::from(String::from("place_keeper")), None)?
    },
  }
  Ok(())
}

/// Find all dummy frontmatter entries (role "pending") containing unattached annotations
/// and attempt to attach to the appropriate frontmatter, based on the identifying labels.
/// Perl: relocateAnnotations($document).
fn relocate_annotations(document: &mut Document) -> Result<()> {
  // Find dummy frontmatter elements containing not-yet-attached annotations
  let pending_nodes = document.findnodes(".//*[@role='pending']", None);
  if pending_nodes.is_empty() {
    return Ok(());
  }
  // Collect the frontmatter that have attachment labels
  let mut labeltable: HashMap<String, Vec<Node>> = HashMap::default();
  // fallback: Same, but without prefix
  let mut unlabeltable: HashMap<String, Vec<Node>> = HashMap::default();
  for target in document.findnodes(".//*[@_annotations]", None) {
    let target_role = target.get_attribute("role").unwrap_or_default();
    if target_role == "pending" {
      continue;
    }
    // Dedup labels PER TARGET: a creator that cites the same annotation label
    // from several of its authors (LLNCS `\author{A\inst{1} B\inst{2,1}}` — both
    // A and B cite institution 1) must receive that affiliation ONCE, not once
    // per citing author. Each duplicate label would otherwise push `target`
    // again, and the pending note gets cloned into it per push. Witness arXiv
    // 2603.23669: two-author creators rendered "Mila … Mila … McGill … McGill".
    let annotations = target.get_attribute("_annotations").unwrap_or_default();
    let mut seen: HashSet<&str> = HashSet::default();
    for label in annotations.split(',') {
      if label.is_empty() || !seen.insert(label) {
        continue;
      }
      labeltable
        .entry(label.to_string())
        .or_default()
        .push(target.clone());
      // Misuse of labelling macros can lead to a prefix mismatch, so fall back to
      // a prefix-stripped ("noprefix") index — e.g. an affiliation labelled
      // `institute:1` can still find an author who cited `affiliation:1`.
      // BUT a creator's OWN role-sequence label (`author:N` / `editor:N` /
      // `translator:N`, auto-assigned in `\lx@add@frontmatter@now`) must NEVER
      // enter that fallback: it would let a SHARED affiliation's `affiliation:1`
      // bind to the first author's `author:1` purely by number, stranding one
      // shared `\institute` on author 1 (Perl does exactly this — arXiv:2402.19043
      // / WDM, 5 authors + one `\institute`, no `\inst`; OXIDIZED_DESIGN #159). A
      // genuine per-author affiliation instead matches EXACTLY via `labeltable`
      // (the `affiliation:1` that `\inst{1}` requests), which is untouched here.
      if let Some(pos) = label.find(':')
        && label[..pos] != target_role
      {
        unlabeltable
          .entry(label[pos + 1..].to_string())
          .or_default()
          .push(target.clone());
      }
    }
  }
  // A label's targets: the exact index, then the prefix-stripped fallbacks for a misused prefix — creators only, the
  // annotations a prefix is misused on: a title's `tnote:t1` must not take an author's orphaned `fn:t1` (63h review,
  // `elsarticle_orphan_fntext_stays_off_the_title`).
  let find_targets = |label: &str| -> Option<Vec<Node>> {
    if let Some(targets) = labeltable.get(label) {
      return Some(targets.clone());
    }
    let noprefix = label
      .find(':')
      .map(|pos| &label[pos + 1..])
      .unwrap_or(label);
    [
      unlabeltable.get(label),
      labeltable.get(noprefix),
      unlabeltable.get(noprefix),
    ]
    .into_iter()
    .flatten()
    .map(|targets| {
      targets
        .iter()
        .filter(|target| target.get_name() == "creator")
        .cloned()
        .collect::<Vec<_>>()
    })
    .find(|targets| !targets.is_empty())
  };
  // An orphan no one cites is kept all the same, as LaTeX prints it. An institute-level contact, or a thanks note
  // (svjour3's `\thankstext` without its `\thanksref`; an EPJ institute's own note, 63g review, repro
  // svjour3_epj_title_and_institute_notes), is shared as a matter of course. A person's own note (elsarticle's
  // `\fntext` without its `\fnref`) lost the link to its person, which is warned about.
  let shared_orphan = |note: &Node| {
    let role = note.get_attribute("role").unwrap_or_default();
    is_shared_contact_role(&role) || role == "thanks"
  };
  // Beyond-Perl (OXIDIZED_DESIGN #159): an orphan's label is one institute with what inherited its label (its
  // `\email`/`\url`), and it goes to the author the evidence names, else is shared:
  // - an institute labelled by a mark (`\affiliation{$^{a}$Univ A}`, an `affiliation:a` label) is the authors' whose
  //   names show that mark (`Ann Able$^{a}$`, a visible `<ltx:sup>` the author line kept), as the reader pairs them
  //   (revtex 2011.01984, 2301.08449; repro revtex_marked_affiliations_link_by_mark); the rest are then not paired by
  //   position;
  // - an institute's `\at` names (svjour3 `fuzzy:` label) that missed the exact name match are the authors with those
  //   surnames, when each names one (`A.M.Bykov \at …` for "Andrei Bykov", 1205.2208; `Olivier Augereau, Koichi Kise,
  //   and Motoi Iwata \at …`, 1811.03214, every one of them), in any order; unless an unlabelled
  //   piece sits among them, svjour3's `\institute{A \and B \at X}` where a name-only piece shares the next institute
  //   (1709.00485, 2003.07295; repro svjour3_name_only_piece_shares_the_next_institute);
  // - numbered institutes (`affiliation:N`, no `\inst`), as many as the authors, are the i-th author's: Perl's numeric
  //   fallback and the grid `distribute_upfront_contacts` pairs (`\author{A \and B}` + `\institute{X \and Y}`);
  // - a sole author's orphans are that author's, as Perl attaches them; its institutes only where no institute is
  //   linked otherwise (an author marked for one institute is not given the one no one marks,
  //   `author_block_orphan_mark_is_kept`).
  // Any other orphan is shared, gathered into ONE trailing name-LESS `<ltx:creator role="author">` below the whole
  // author row, where Perl warns and drops it, or strands one shared `\institute` on author 1 (arXiv:2402.19043, WDM: 5
  // authors + one `\institute`). The ar5iv theme breaks a last-position shared affiliation out to full width. We reuse
  // the first such pending stub in place: it is already a `<creator>` right after the last real author.
  let mut orphan_labels: Vec<(String, bool)> = Vec::new();
  let mut linked_institute = false;
  for pending in &pending_nodes {
    for note in element_nodes(pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      let institute = is_institute_role(&note.get_attribute("role").unwrap_or_default());
      if label.is_empty() {
        continue;
      } else if find_targets(&label).is_some() {
        linked_institute |= institute;
      } else if let Some(entry) = orphan_labels.iter_mut().find(|(known, _)| *known == label) {
        entry.1 |= institute;
      } else {
        orphan_labels.push((label, institute));
      }
    }
  }
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  // the marks each author's name shows (`$^{a,b}$`: two)
  let author_marks: Vec<Vec<String>> = authors
    .iter()
    .map(|author| shown_marks(document, author))
    .collect();
  // Only a label a mark set is answered by the authors showing that mark, as the mark's own keys (`_bymark`, read as the
  // authors' are): a number `labelseq` gave is no mark (aa's one `\institute{{1} Univ A \\ {2} Univ B …}` is one
  // institute, `affiliation:1`, under authors marked 1,2 / 2,3: astro-ph0305539).
  let by_mark: HashMap<String, Vec<String>> = pending_nodes
    .iter()
    .filter_map(|pending| {
      let keys = pending.get_attribute("_bymark")?;
      let label = pending.get_attribute("_annotations")?;
      Some((label, keys.split(',').map(str::to_string).collect()))
    })
    .collect();
  let mut owners: HashMap<String, Vec<Node>> = HashMap::default();
  for (label, institute) in orphan_labels.iter() {
    let Some(keys) = by_mark.get(label) else {
      continue;
    };
    let marked: Vec<Node> = (0..authors.len())
      .filter(|&i| *institute && keys.iter().any(|key| author_marks[i].contains(key)))
      .map(|i| authors[i].clone())
      .collect();
    if !marked.is_empty() {
      owners.insert(label.clone(), marked);
    }
  }
  let linked_institute = linked_institute || !owners.is_empty();
  let unaffiliated = !linked_institute
    && authors.iter().all(|author| {
      element_nodes(author).iter().all(|child| {
        child.get_name() != "contact"
          || !is_institute_role(&child.get_attribute("role").unwrap_or_default())
      })
    });
  let institutes: Vec<&String> = orphan_labels
    .iter()
    .filter(|(label, institute)| *institute && !owners.contains_key(label))
    .map(|(label, _)| label)
    .collect();
  let numbered = |label: &str| {
    label
      .strip_prefix("affiliation:")
      .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
  };
  if !institutes.iter().any(|label| numbered(label)) {
    let surnames: Vec<Option<String>> = authors
      .iter()
      .map(|author| {
        document
          .findnode("ltx:personname", Some(author))
          .and_then(|person| surname_key(&person.get_content()))
      })
      .collect();
    for label in &institutes {
      let Some(names) = label.strip_prefix("fuzzy:") else {
        continue;
      };
      // each name the piece gives before its `\at` (a list: commas, " and ", " \& "; spaces are `_` in a label)
      let mut named: Vec<Node> = Vec::new();
      let each_named_once = names
        .split([',', '&'])
        .flat_map(|part| part.split("_and_"))
        .map(|name| name.trim_matches('_').trim_start_matches("and_"))
        .filter(|name| !name.is_empty())
        .all(|name| {
          let key = surname_key(name);
          let matches: Vec<usize> = (0..authors.len())
            .filter(|&i| key.is_some() && surnames[i] == key)
            .collect();
          if let [i] = matches.as_slice() {
            if !named.contains(&authors[*i]) {
              named.push(authors[*i].clone());
            }
            true
          } else {
            false
          }
        });
      if each_named_once && !named.is_empty() {
        owners.insert((*label).clone(), named);
      }
    }
  } else if unaffiliated
    && authors.len() > 1
    && institutes.len() == authors.len()
    && institutes.iter().all(|label| numbered(label))
  {
    owners.extend(
      institutes
        .iter()
        .map(|label| (*label).clone())
        .zip(authors.iter().map(|author| vec![author.clone()])),
    );
  }
  if let [sole] = authors.as_slice() {
    for (label, institute) in &orphan_labels {
      if !*institute || unaffiliated {
        owners
          .entry(label.clone())
          .or_insert_with(|| vec![sole.clone()]);
      }
    }
  }
  let mut shared_creator: Option<Node> = None;
  for mut pending in pending_nodes {
    let mut promote_this = false;
    // the stub's notes placed elsewhere, which a promoted stub must not keep a second time
    let mut placed: Vec<Node> = Vec::new();
    for note in element_nodes(&pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      if label.is_empty() {
        continue;
      }
      if let Some(targets) = find_targets(&label) {
        placed.push(note.clone());
        for target in targets {
          DebugFeature!("frontmatter", "FRONT Moving annotation for {label}");
          let mut target = target;
          document.append_clone(&mut target, vec![note.clone()])?;
          contact_in_title_as_note(document, &target)?;
        }
      } else {
        if !shared_orphan(&note) {
          let mut known: Vec<&String> = labeltable.keys().collect();
          known.sort();
          Warn!(
            "unexpected",
            "annotation",
            s!("Orphaned frontmatter annotation couldn't find target for label={label}"),
            s!(
              "known labels={}",
              known
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(",")
            )
          );
        }
        // (an institute several authors share is each one's; its `\email`/`\url` names one of them, which, unknown,
        // stays shared: `kono@rice.edu` under one `L. Ren, Q. Zhang, S. Nanot, and J. Kono \at …` piece, 1205.6171)
        let owned = owners.get(&label).filter(|named| {
          named.len() == 1 || is_institute_role(&note.get_attribute("role").unwrap_or_default())
        });
        if let Some(named) = owned {
          placed.push(note.clone());
          DebugFeature!(
            "frontmatter",
            "FRONT Giving orphaned annotation {label} to its authors"
          );
          for owner in named {
            let mut owner = owner.clone();
            document.append_clone(&mut owner, vec![note.clone()])?;
          }
        } else {
          DebugFeature!(
            "frontmatter",
            "FRONT Sharing orphaned annotation {label} below authors"
          );
          match &mut shared_creator {
            // First orphan: promote THIS pending stub in place into the trailing shared creator (its note already
            // lives inside it).
            None => {
              promote_this = true;
            },
            // Later orphans (e.g. the institute's `\email`) join it.
            Some(sc) => {
              let mut sc = sc.clone();
              document.append_clone(&mut sc, vec![note.clone()])?;
            },
          }
        }
      }
    }
    if promote_this {
      // pending → real trailing creator (role=pending would otherwise be dropped
      // with the node; the `_annotations`/`_label` bookkeeping attrs are stripped
      // at serialization). Name-LESS: it carries no `<ltx:personname>`.
      document.set_attribute(&mut pending, "role", "author")?;
      for note in placed {
        document.remove_node(note);
      }
      shared_creator = Some(pending);
    } else {
      document.remove_node(pending);
    }
  }
  Ok(())
}

/// A labelled annotation relocated into an element that holds no contact — a title, whose model has none (aastex's
/// title footnote `\title{..\altaffilmark{1}}` + `\altaffiltext{1}`, 0704.0478, 1001.2402; elsarticle's
/// `\tnoteref`/`\tnotetext`) — is that element's note: the frontmatter thanks note `\lx@add@thanks` makes, as LaTeX
/// prints it (a footnote marked on the title). Perl clones the `ltx:contact` as it is, schema-invalid there.
fn contact_in_title_as_note(document: &mut Document, target: &Node) -> Result<()> {
  let Some(clone) = target.get_last_child() else {
    return Ok(());
  };
  if clone.get_type() != Some(NodeType::ElementNode)
    || document::get_node_qname(&clone) != pin_static("ltx:contact")
    || document::can_contain_qsym(document::get_node_qname(target), pin_static("ltx:contact"))
  {
    return Ok(());
  }
  let kind = classify_thanks(&clone.get_content());
  let mut note = document.rename_node(clone, "ltx:note", false)?;
  document.set_attribute(&mut note, "role", "thanks")?;
  document.add_class(&mut note, &s!("ltx_note_frontmatter ltx_thanks_{kind}"))
}

/// The marks a creator's name shows: each `<ltx:sup>`'s (`\textsuperscript{a}`), and each formula's that is only a
/// superscript (`$^{a}$`, `$^{1,2}$`: a `<ltx:Math>` whose `tex` is `{}^{a}` until the math is rewritten into a
/// `<ltx:sup>` later).
fn shown_marks(document: &mut Document, creator: &Node) -> Vec<String> {
  let mut marks: Vec<String> = document
    .findnodes("ltx:personname//ltx:sup", Some(creator))
    .iter()
    .flat_map(|sup| script_marks(sup.get_content().trim()))
    .collect();
  for math in document.findnodes("ltx:personname//ltx:Math", Some(creator)) {
    let tex = math.get_attribute("tex").unwrap_or_default();
    if tex
      .trim()
      .trim_start_matches("{}")
      .trim_start()
      .starts_with('^')
    {
      marks.extend(superscript_scripts(&tex).concat());
    }
  }
  marks
}

/// The text of the braced group `text` opens (just past its `{`), to its matching `}`, and what follows it.
fn braced_group(text: &str) -> (&str, &str) {
  let mut depth = 0usize;
  for (i, c) in text.char_indices() {
    match c {
      '{' => depth += 1,
      '}' if depth == 0 => return (&text[..i], &text[i + 1..]),
      '}' => depth -= 1,
      _ => {},
    }
  }
  (text, "")
}

/// The letter or number marks of a superscript's script: each item of its list at its top-level commas (`2,\dagger`:
/// the 2), past braces and font switches (`\rm a`, `\mathrm{a}`, `{\rm 2}`: the a, the 2), and before a symbol after
/// it (`2*`, `1\dagger`, `1†`: the 2, the 1). A symbol alone or other markup (`\footnote{…}`) is no mark. Both the marks
/// an affiliation is labelled by and the marks an author shows are read so (63i review).
fn script_marks(script: &str) -> Vec<String> {
  const FONTS: [&str; 10] = [
    "\\mathrm", "\\textrm", "\\mathit", "\\textit", "\\mathbf", "\\textbf", "\\mathsf", "\\rm",
    "\\it", "\\bf",
  ];
  let mut items = Vec::new();
  let (mut depth, mut start) = (0usize, 0usize);
  for (i, c) in script.char_indices() {
    match c {
      '{' => depth += 1,
      '}' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => {
        items.push(&script[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  items.push(&script[start..]);
  items
    .into_iter()
    .filter_map(|item| {
      let mut item: String = item
        .chars()
        .filter(|c| !matches!(c, '{' | '}') && !c.is_whitespace())
        .collect();
      while let Some(rest) = FONTS.iter().find_map(|cs| item.strip_prefix(cs)) {
        item = rest.to_string();
      }
      let lead: String = item.chars().take_while(|c| c.is_alphanumeric()).collect();
      let rest = &item[lead.len()..];
      let symbol_after = rest.is_empty()
        || rest.starts_with(['*', '\\'])
        || rest.starts_with(|c: char| !c.is_ascii() && !c.is_alphanumeric());
      (!lead.is_empty() && symbol_after).then_some(lead)
    })
    .collect()
}

/// The marks of each superscript in a TeX source string, one list per superscript (`Ann Able$^{1,2}$ and Bob$^{1}$`
/// gives [1, 2] and [1]; `^a` gives [a]; `\textsuperscript{b}` gives [b]).
fn superscript_scripts(tex: &str) -> Vec<Vec<String>> {
  const TEXTSUP: &str = "\\textsuperscript";
  let mut scripts = Vec::new();
  let mut rest = tex;
  loop {
    // (a `^` that is no `\^` accent: `Ren\^{e}`)
    let caret = rest
      .char_indices()
      .find(|&(i, c)| c == '^' && !rest[..i].ends_with('\\'))
      .map(|(i, _)| i);
    let (at, skip) = match (caret, rest.find(TEXTSUP)) {
      (Some(caret), Some(text)) if text < caret => (text, TEXTSUP.len()),
      (Some(caret), _) => (caret, 1),
      (None, Some(text)) => (text, TEXTSUP.len()),
      (None, None) => break,
    };
    rest = rest[at + skip..].trim_start();
    if let Some(inner) = rest.strip_prefix('{') {
      let (script, after) = braced_group(inner);
      scripts.push(script_marks(script));
      rest = after;
    } else if let Some(c) = rest.chars().next() {
      scripts.push(script_marks(&c.to_string()));
      rest = &rest[c.len_utf8()..];
    }
  }
  scripts
}

/// Is the affiliation `content` the group's, as revtex's group rule gives it? So it is when an author was queued since
/// a previous affiliation (interleaved `\author`/`\affiliation` groups, whatever their marks say: a mark no author shows
/// is still its group's, as the PDF pairs it), and when the first affiliation carries one mark that every author
/// queued shows (`\author{A$^{1}$ and B$^{1,2}$}\affiliation{$^{1}$…}`, 0808.2763). Authors listed first and their marked
/// affiliations after them are no such group, and an affiliation holding several marked lines (`$^1$… \\ $^2$…`,
/// 1205.4587) is a marked list (63i review).
pub fn queued_group_shares_the_mark(content: &Tokens) -> bool {
  let mut marks: Vec<String> = superscript_scripts(&content.to_string()).concat();
  marks.dedup();
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => {
      let is_affiliation = |entry: &RawFrontmatter| {
        entry.0 == "ltx:contact" && entry.1.get("role").map(String::as_str) == Some("affiliation")
      };
      let since: Vec<&RawFrontmatter> = queue
        .iter()
        .rev()
        .take_while(|entry| !is_affiliation(entry))
        .filter(|entry| entry.0 == "ltx:creator")
        .collect();
      if !since.is_empty() && queue.iter().any(is_affiliation) {
        return true;
      }
      let [mark] = marks.as_slice() else {
        return false;
      };
      let scripts: Vec<Vec<String>> = since
        .iter()
        .flat_map(|entry| superscript_scripts(&entry.2.to_string()))
        .filter(|script| !script.is_empty())
        .collect();
      !scripts.is_empty() && scripts.iter().all(|script| script.contains(mark))
    },
    _ => false,
  })
}

/// The surname a name ends with, to match an institute's `\at` name with its author: the last run of letters,
/// lowercased (`A.M.Bykov` and "Andrei Bykov" give "bykov"). `None` for a name without letters.
fn surname_key(name: &str) -> Option<String> {
  name
    .rsplit(|c: char| !c.is_alphabetic())
    .find(|word| !word.is_empty())
    .map(str::to_lowercase)
}

/// The contact roles that are an institute itself, which its `\email`/`\url` may follow under its label.
fn is_institute_role(role: &str) -> bool {
  matches!(role, "affiliation" | "authorblock" | "address")
}

/// Contact roles that are institute-level information shared by every author (an
/// affiliation, or an address / email / url that inherited its label), as opposed
/// to a single person's own note. When such a contact is orphaned (a shared
/// `\institute` with no per-author `\inst`), it is collected onto the trailing
/// shared creator rather than dropped or stranded on author 1 (OXIDIZED_DESIGN #159).
fn is_shared_contact_role(role: &str) -> bool {
  matches!(
    role,
    "affiliation"
      | "altaffiliation"
      | "authorblock"
      | "address"
      | "altaddress"
      | "currentaddress"
      | "email"
      | "url"
  )
}

//======================================================================
// Shared Rust helper functions (moved from base_functions.rs)
// Perl equivalent: LaTeXML::Package.pm utility exports
//======================================================================

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
    bindings.extend(vdq.iter());
  }
  for binding in bindings {
    if let Stored::Tokens(tks) = binding {
      let vec = tks.unlist_ref();
      let_i(&vec[0], &vec[1], None);
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
/// Reset the text and math fonts to their defaults, **locally**.
///
/// For metadata that must not inherit the font it happens to be written in —
/// a title picked up inside an italic environment, an author in a small-caps
/// box. The assignments are group-local, so use this inside a group (or a
/// `bounded => 1` definition), otherwise the reset outlives the construct that
/// wanted it.
///
/// The NFSS codes follow, as `\reset@font` (`\normalfont`, latex.ltx:14122)
/// sets them where LaTeX resets a note's font (`\@footnotetext`,
/// latex.ltx:17658-17659 `\reset@font\footnotesize`): else a size switch in
/// the note, which ends in `\selectfont`, re-selected the surrounding family,
/// series or shape (`\textit{A\footnote{\footnotesize note}}` came out
/// italic).
///
/// The encoding is `\encodingdefault`, which `\reset@font` selects (`\fontencoding\encodingdefault`,
/// latex.ltx:14113-14122): a document's text encoding — fontenc's last option, fontspec's TU, babel's `\latinencoding`
/// (babel.sty:3936-3938), greek.ldf's `\greekfontencoding` (:151-153) — not the TeX-wide OT1, else a T1 document's
/// footnotes (`\reset@font`, latex.ltx:17659) typeset `<b> x|y` through OT1 as `¡b¿ x—y` (Perl alike,
/// TeX_Box.pool.ltxml:218-221). LaTeXML's tags neutralize too, where a LaTeX label inherits the body's encoding. Plain
/// TeX has no `\encodingdefault`: OT1.
pub fn neutralize_font() { neutralize_font_in(encoding_named("\\encodingdefault")); }

fn neutralize_font_in(encoding: Option<String>) {
  let mut font = Font::text_default();
  if let Some(encoding) = encoding {
    font.encoding = Some(Cow::Owned(encoding));
  }
  sync_nfss_font_codes(&font);
  assign_value("font", font, Some(Scope::Local));
  assign_value("mathfont", Font::math_default(), Some(Scope::Local));
}

/// The encoding the macro `cs` (`\encodingdefault`, `\latinencoding`) names, expanded as `\fontencoding` expands it
/// (a literal name is read without expanding: the common case, on every tag). `None` when undefined (plain TeX) or
/// not an encoding name.
fn encoding_named(cs: &str) -> Option<String> {
  let defn = lookup_definition(&T_CS!(cs)).ok()??;
  let literal = match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => body
      .unlist_ref()
      .iter()
      .all(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
      .then(|| body.to_string()),
    _ => None,
  };
  let encoding = match literal {
    Some(encoding) => encoding,
    None => do_expand(T_CS!(cs)).ok()?.to_string(),
  };
  let encoding = encoding.trim();
  (!encoding.is_empty() && encoding != "ASCII" && !encoding.contains('\\'))
    .then(|| encoding.to_string())
}

/// Today's date as LaTeX's `\today` renders it — `Month D, YYYY`.
///
/// Mirrors Perl `TeX_Job.pool.ltxml` L52-55, reading `\year`/`\month`/`\day`
/// from the **value** table where job startup put them — not from the register
/// meanings, which a class file is free to `\def` out from under us. A missing
/// value falls back to a default instead of aborting the conversion; the body
/// comment records that divergence and its witnesses.
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
  // Mirror Perl TeX_Job.pool.ltxml L52-55:
  //   $MonthNames[LookupValue('\month')->valueOf - 1]
  //     . " " . LookupValue('\day')->valueOf
  //     . ', ' . LookupValue('\year')->valueOf
  // Read from the VALUE table (assigned in tex_job.rs:47-49 at job
  // startup), NOT the register-meaning table. `lookup_register` walks
  // meanings and returns None when a class file (e.g. iopart.cls)
  // `\def`s `\day` for its own purposes, panicking the unwrap on
  // 1208.0134/1702.02270/1705.08909. Intentional Perl divergence:
  // default to 1900-01-01 when a value is missing, where Perl would
  // die on `Can't call method "valueOf" on undef` — keeps the
  // conversion alive when something has clobbered the value table.
  let read = |key: &str, default: i32| -> i32 {
    match lookup_value(key) {
      Some(Stored::Int(n)) => n as i32,
      Some(Stored::Number(n)) => n.value_of() as i32,
      _ => default,
    }
  };
  let m = read("\\month", 1).clamp(1, 12) as usize;
  let month = month_names[m - 1];
  let day = read("\\day", 1);
  let year = read("\\year", 1900);
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
      // Every token of the delimiter is kept, consecutive SPACE tokens
      // included (tex.web §473-476 `scan_toks` reads the parameter text with
      // `get_token`, no collapsing; the tokenizer's `skip_blanks` state has
      // already folded file-sourced runs before they get here). Perl
      // TeX_Macro.pool.ltxml L127 drops the second of two adjacent spaces
      // ("BUT collapse whitespace!"), which only ever bites an
      // expansion-built parameter text: expkv.tex L709-712
      // `\ekv@set@was@blank`'s `#1` delimiter carries TWO real spaces
      // (`{ }` through `\ekv@strip@key` twice), so the collapsed `Until:`
      // never matched, the marker dance derailed (`\ekv@stop`/`\ekv@nil`
      // undefined) and the re-scanned tail ran to `Timeout:TokenLimit`
      // (witness tutodoc-en/fr via clrstrip's `\ekvset{clrstrip}{}`;
      // KNOWN_PERL_ERRORS #119; guard
      // `perfect_kernel_batch53::def_delimiter_keeps_adjacent_spaces`).
      let mut delim = Vec::new();
      while !tokens.is_empty() {
        let inner_cc = tokens.front().unwrap().get_catcode();
        if inner_cc == Catcode::PARAM || inner_cc == Catcode::ARG {
          break;
        }
        delim.push(tokens.pop_front().unwrap());
      }
      // Found text that marks the end of the parameter
      if !delim.is_empty() {
        let extra = Tokens::new(delim);
        params.push(
          Parameter {
            name: pin!("Until"),
            spec: pin(format!("Until:{extra}")),
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
          name: pin!("Match"),
          spec: pin(s!("Match:{expected}")),
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
  install_definition(
    Expandable::new(
      cs,
      paramlist,
      Some(ExpansionBody::Tokens(body)),
      Some(ExpandableOptions {
        nopack_parameters: true,
        tex_declared: true,
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
      Some(Stored::Digested(d)) => match d.data() {
        DigestedData::Whatsit(w)
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

/// Stomach-level counterpart to `read_box_contents`.
///
/// Perl: readBoxContents calls $stomach->beginMode($mode), then reads/digests tokens
/// Predigest box contents by invoking T_BEGIN, which triggers
/// the stomach's bgroup/egroup mechanism to properly handle the box body.
///
/// Perl's `List()` simplification (List.pm line 41-44):
/// When a vertical-mode List has exactly one non-empty item and that item's mode
/// is also vertical, return the item directly instead of wrapping in a List.
/// This enables `is_vbox` property propagation for nested \vbox/\vtop.
pub fn predigest_box_contents(tokens: ArgWrap) -> Result<Option<Digested>> {
  // Default: use the CURRENT mode (legacy behavior, e.g. for `{Body}` arguments
  // that aren't \vbox/\vtop/\hbox-flavored). For VBoxContents / HBoxContents,
  // call predigest_box_contents_in_mode("internal_vertical" / "restricted_horizontal")
  // explicitly so we mirror Perl's `readBoxContents(..., $mode)` (TeX_Box.pool.ltxml L133).
  let mode = lookup_string_from_sym(pin!("MODE"));
  predigest_box_contents_in_mode(tokens, &mode)
}

/// Perl-faithful body-digest for VBoxContents / HBoxContents parameters.
/// Mirrors `readBoxContents($gullet, $everybox, $mode)` exactly:
/// pushes a fresh frame for `$mode`, digests tokens until matching T_END,
/// pops the frame.
///
/// The mode-aware variant matters when `\vtop` is invoked inside a `p{}`
/// column body (alignment is in horizontal mode, but the `\vtop`'s VBox
/// content must be digested in internal_vertical mode so `\@startpbox`'s
/// `\vtop\bgroup ...` doesn't try to close groups in the wrong mode).
/// Witness: 2210.13325 `\begin{tabular}{|p{1cm}|...}` — pre-fix Rust
/// inherited the surrounding horizontal mode and emitted `\vtop`
/// errors; Perl uses 'internal_vertical' here regardless of where the
/// `\vtop` was invoked.
pub fn predigest_box_contents_in_mode(tokens: ArgWrap, mode: &str) -> Result<Option<Digested>> {
  predigest_list_in_mode(tokens, mode, false)
}

/// The material of a `\vadjust`, read live from the input after its `{` as the box loop reads a box's (tex.web §1099:
/// `scan_left_brace`, then `main_control` builds the group's internal vertical list), so catcode changes inside it
/// act on what follows — a short verbatim `|\ifx|` prints `\ifx` — and it is built where it is read (a box register,
/// a macro, a counter as they are there). The mode is `inline_internal_vertical`, which does not end the paragraph
/// around it; a paragraph the material begins ends at the group's end (§1100 `insert_group`: `end_graf`), as the
/// primitive `\lx@normal@par` (KNOWN_PERL_ERRORS #441; repro boxes-groups/vadjust_material_is_read_live).
pub fn predigest_insert_group_contents() -> Result<Option<Digested>> {
  predigest_list_in_mode(ArgWrap::None, "inline_internal_vertical", true)
}

fn predigest_list_in_mode(
  _tokens: ArgWrap,
  mode: &str,
  end_graf: bool,
) -> Result<Option<Digested>> {
  // Perl: readBoxContents calls beginMode($mode) / endMode($mode) around the body reading.
  // This creates a scoped frame where enterHorizontal can change MODE inplace.
  // When endMode is called, leaveHorizontal_internal detects MODE='horizontal' with
  // BOUND_MODE ending in 'vertical', triggers repackHorizontal, then pops the frame.
  //
  // NOTE: read_box_contents already consumed the opening { or \bgroup (§403 `scan_left_brace`, `is_left_brace`).
  // invoke_token(T_BEGIN) pushes a synthetic group frame. The matching } or \egroup
  // in the content will pop this frame, since \egroup is \let to T_END and
  // invoke_token handles it via the standard group-closing mechanism.
  // Perl: $stomach->beginMode($mode) — push a new frame for this box content scope
  // VERTICAL box modes (`\vbox`/`\vtop`/p{}-cell VBoxContents): faithful port of
  // Perl `readBoxContents` (TeX_Box.pool.ltxml L139-160). ONE mode frame as the
  // group; a loop that STOPS at the matching `}` WITHOUT processing it, so
  // `end_mode` runs `leave_horizontal_internal` (repack — capturing the *inner*
  // `\hsize`) BEFORE `pop_stack_frame` restores it. The `invoke_token(&T_BEGIN!())`
  // path used for other modes lets the body's `}` egroup-restore `\hsize` before
  // the post-hoc repack, so a `\vbox{\hsize=W …para…}` would wrap at the OUTER
  // `\hsize` (the Cluster G `\narrow`-hang; rotated p{}-cell mis-sizing —
  // SYNC_STATUS 1610.00974 step-3). hbox/math don't paragraph-wrap → the simpler
  // `invoke_token(T_BEGIN)` path below. (A tabular inside a `\vbox`/`\vtop` is
  // correctly SKIPPED by the repack because `\@@tabular`/`\halign` mark the result
  // box `internal_vertical`.)
  // RESTRICTED HORIZONTAL (`\\hbox`/`\\mbox`/`\\makebox`… contents) takes the
  // same one-frame loop (batch 54n). tex.web §1083 `begin_box` pushes the
  // nest and the save level TOGETHER for `\\hbox\\bgroup` and §1100 `package`
  // pops both; Perl's one frame is that single push. The former two-frame
  // shortcut (`begin_mode` + the `{` primitive's own `bgroup`) made the box
  // depend on an `\\egroup` to close its synthetic group, and ulem's
  // open-here/close-there word boxes (examdesign.cls:186-200 `\\UL@start` =
  // `\\setbox\\UL@box\\hbox\\bgroup…\\bgroup`, `\\UL@stop` = `\\egroup\\egroup`) around a
  // `\\makebox` whose argument digests in isolation met the wrong frame
  // ("\\egroup Attempt to close a group that switched to mode
  // restricted_horizontal"; examdesign examplea/b/c, Perl shares it). Guard:
  // `perfect_kernel_batch54::hbox_reader_is_one_frame`.
  if mode.ends_with("vertical") || mode == "restricted_horizontal" {
    begin_mode(mode)?;
    // The box's OWN frame (tex.web §1068 `handle_right_brace` dispatches on
    // `cur_group`, the group itself): Perl keys the terminal on the save-stack
    // DEPTH (`$level >= getFrameDepth`, TeX_Box.pool.ltxml:172), which a
    // crossing — a `\begingroup` opened inside the box after a frame below
    // it was popped, pgf's node text over our bound `\pgfsys@*` seam — can
    // satisfy with a `\begingroup` frame on top, so `end_mode` then failed
    // ("\hbox Attempt to end mode restricted_horizontal", modernposter; the
    // msc/dsptricks cascades). A `}` that meets another frame is invoked
    // instead: `egroup` reports and drops it (tex.web §1069
    // `extra_right_brace`) and the box goes on to its own `}`.
    let own_frame = current_frame_id();
    new_local_box_list(); // Perl: local @LaTeXML::LIST = ()
    // The box's contents are digested outside any number scan in progress
    // (`gullet::NumberScan::suspend`, as `stomach::digest` does): this loop is
    // a stomach entry of its own.
    let _outside_scan = NumberScan::suspend();
    loop {
      let next = match get_pending_comment() {
        Some(comment) => Some(comment),
        None => read_x_token(Some(true), false, None)?,
      };
      let Some(token) = next else { break };
      // The box's own `}` (its frame on top) ends the loop and is closed by
      // `end_mode`, NOT egroup; any other `}` is processed.
      if is_right_brace(&token) && current_frame_id() == own_frame {
        break;
      }
      check_timeout()?; // runaway guard (mirrors the canonical group-digest loop)
      extend_box_list(invoke_token(&token)?);
    }
    // tex.web §679 `append_to_vlist`: the interline glue between a vertical box's lines follows the
    // `\baselineskip`, `\lineskip` and `\lineskiplimit` in force inside it (`\offinterlineskip` in the box), read
    // before its group ends; Perl's `List` records the `\baselineskip` after it (List.pm:52-53). Its sizing and
    // `\vsplit` (tex_inserts.rs) stack the lines with them (OXIDIZED_DESIGN_DIVERGENCES #421; short-math-guide's
    // `symlist` columns split as its PDF).
    let interline: Vec<(&str, Dimension)> = if mode.ends_with("vertical") {
      [
        ("baseline", "\\baselineskip"),
        ("lineskip", "\\lineskip"),
        ("lineskiplimit", "\\lineskiplimit"),
      ]
      .into_iter()
      .filter_map(|(key, register)| {
        lookup_register_quiet(register).map(|value| (key, Dimension::new(value.value_of())))
      })
      .collect()
    } else {
      Vec::new()
    };
    // §1100 `insert_group`: `end_graf` before the group's list is packaged. The primitive `\lx@normal@par` also when
    // the list ends in vertical mode (where `end_graf` does nothing): it closes what the material opened, so a box
    // set in it (`\vadjust{\box0}`) is no part of the paragraph after.
    if end_graf {
      extend_box_list(invoke_token(&T_CS!("\\lx@normal@par"))?);
    }
    // Perl: $stomach->endMode($mode) — leave_horizontal_internal (repack with the
    // still-in-scope inner `\hsize`) THEN pop_stack_frame (restores `\hsize`).
    end_mode(mode)?;
    let mode_tex = if lookup_bool_sym(pin!("IN_MATH")) {
      TexMode::Math
    } else {
      TexMode::Text
    };
    let mut digested_list = List::new(expire_local_box_list());
    digested_list.mode = Some(mode_tex);
    for (key, value) in interline {
      digested_list
        .properties
        .insert(key, Stored::Dimension(value));
    }
    let mut item: Digested = digested_list.into();
    // Perl: List(@LaTeXML::LIST, mode => $mode)
    item.set_property("mode", Stored::String(pin(mode)));
    // Perl's single-item `List()` simplification is a vertical-mode rule.
    return Ok(Some(if mode.ends_with("vertical") {
      simplify_vertical_list(item)
    } else {
      item
    }));
  }
  if mode.ends_with("vertical") || mode.ends_with("horizontal") {
    begin_mode(mode)?;
  }
  let mut contents = invoke_token(&T_BEGIN!())?;
  if contents.is_empty() {
    // Perl: $stomach->endMode($mode)
    if mode.ends_with("vertical") || mode.ends_with("horizontal") {
      end_mode(mode)?;
    }
    Ok(None)
  } else {
    let mut item = contents.remove(0);
    // Perl's endMode triggers leaveHorizontal_internal → repackHorizontal
    // when enterHorizontal changed MODE to 'horizontal' inplace within this frame.
    // Check the condition BEFORE endMode pops the frame.
    let post_mode = lookup_string_from_sym(pin!("MODE"));
    let bound_mode = lookup_string_from_sym(pin!("BOUND_MODE"));
    if post_mode == "horizontal"
      && bound_mode.ends_with("vertical")
      && has_only_simple_horizontal_content(&item)
    {
      repack_horizontal_in_list(&mut item);
      // Restore MODE like leave_horizontal_internal does
      assign_value_inplace_sym(pin!("MODE"), pin(&bound_mode));
    }
    // Perl: $stomach->endMode($mode) — pop the frame
    if mode.ends_with("vertical") || mode.ends_with("horizontal") {
      end_mode(mode)?;
    }
    // Set the mode property on the resulting item (matching Perl's List(@boxes, mode => $mode))
    if !mode.is_empty() {
      item.set_property("mode", Stored::String(pin(mode)));
    }
    // Apply Perl's List() single-item simplification for vertical modes.
    // In Perl, List(@boxes, mode=>'internal_vertical') returns the single box
    // directly when @boxes has 1 element and the box's mode is also vertical.
    // This is critical for nested \vbox/\vtop: the inner box's `is_vbox` property
    // must be visible to the outer box's constructor.
    Ok(Some(simplify_vertical_list(item)))
  }
}

/// Check if a List contains only simple horizontal content (TBoxes,
/// Comments, or sub-Lists whose mode is horizontal/restricted_horizontal/
/// math). This guards against repack being triggered for cases like
/// `\vtop{\begin{tabular}...}` where the tabular processing leaks
/// MODE='horizontal' but the content (an Alignment/Whatsit) should NOT
/// be paragraph-wrapped.
///
/// Inline brace-groups `{...}` inside running text produce sub-Lists
/// (one per group) whose mode is `restricted_horizontal`. Without
/// accepting those, e.g. `\vbox{\small\bfseries hello {,} world}` would
/// fail the repack gate and be measured as 3 separate vertical lines.
/// Witness: aistats2026.sty's `\def\And{\unskip{,}\enspace}` in the
/// `\@runningauthor` body — every author separator emits a `{,}` group
/// that fragments the vbox into many short rows, making `\ht\autrun`
/// far exceed 10pt and triggering the class's `\PackageError{Document}
/// {Running heading author exceeds size limitations}` (driver paper:
/// arXiv:2602.11863).
fn has_only_simple_horizontal_content(item: &Digested) -> bool {
  if let DigestedData::List(l) = item.data() {
    let list = l.borrow();
    let non_empty: Vec<_> = list
      .boxes
      .iter()
      .filter(|b| !b.get_property_bool("isEmpty"))
      .collect();
    non_empty.iter().all(|b| match b.data() {
      DigestedData::TBox(_) | DigestedData::Comment(_) => true,
      DigestedData::List(sub) => {
        // Inline brace-groups `{...}` in horizontal context digest to
        // a sub-List with no `mode` property (implicit hbox). Accept
        // those plus sub-Lists explicitly tagged as horizontal-flavour.
        // Reject `mode=vertical|internal_vertical` (structural
        // sub-vboxes) and any other tagged mode.
        let sub_mode = sub
          .borrow()
          .properties
          .get("mode")
          .map(|v| v.to_string())
          .unwrap_or_default();
        matches!(
          sub_mode.as_str(),
          "" | "horizontal" | "restricted_horizontal" | "math"
        )
      },
      _ => false,
    })
  } else {
    false
  }
}

/// Replicates Perl's repackHorizontal() within a List's children.
///
/// Perl (Stomach.pm lines 442-456): In readBoxContents, after digesting box content
/// in vertical mode, repackHorizontal groups consecutive horizontal-mode items
/// from @LaTeXML::LIST into a single List(@para, mode => 'horizontal') with
/// width set to \hsize. This enables compute_boxes_size to do paragraph wrapping.
///
/// Without this, \vbox{hop} measures each character individually (width=5.55pt)
/// instead of wrapping as a paragraph at \hsize (width=469.75pt).
fn repack_horizontal_in_list(item: &mut Digested) {
  if let DigestedData::List(l) = item.data() {
    let mut list = l.borrow_mut();
    let children = std::mem::take(&mut list.boxes);
    let mut result: Vec<Digested> = Vec::new();
    let mut para: Vec<Digested> = Vec::new();
    let mut keep = false;

    for child in children {
      let child_mode = child
        .get_property("mode")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "horizontal".to_string());
      // Empty-string mode means "implicit hbox" — produced by inline
      // brace-groups `{...}` in horizontal context. Treat as horizontal
      // so the surrounding running text doesn't get fragmented.
      let effective_mode = if child_mode.is_empty() {
        "horizontal"
      } else {
        child_mode.as_str()
      };
      if effective_mode == "horizontal"
        || effective_mode == "restricted_horizontal"
        || effective_mode == "math"
      {
        // Perl: $keep = 1 if ($mode ne 'horizontal') || !$item->getProperty('isSpace');
        if effective_mode != "horizontal" || !child.get_property_bool("isSpace") {
          keep = true;
        }
        para.push(child);
      } else {
        // Flush accumulated horizontal items as a horizontal List
        if keep {
          let horiz_list = make_horizontal_list(std::mem::take(&mut para));
          result.push(Digested::from(horiz_list));
        } else {
          result.extend(std::mem::take(&mut para));
        }
        keep = false;
        result.push(child);
      }
    }
    // Flush remaining horizontal items
    if keep {
      let horiz_list = make_horizontal_list(para);
      result.push(Digested::from(horiz_list));
    } else {
      result.extend(para);
    }
    list.boxes = result;
  }
}

/// Create a horizontal List with mode='horizontal' and width=\hsize.
/// Perl: push(@LaTeXML::LIST, List(@para, mode => 'horizontal')) if $keep;
/// Perl: $list->setProperty(width => LookupRegister('\hsize')) if $mode eq 'horizontal';
fn make_horizontal_list(para: Vec<Digested>) -> List {
  let mut list = List::new(para);
  list.mode = Some(TexMode::Text);
  list.set_property("mode", Stored::String(pin!("horizontal")));
  if let Some(hsize) = lookup_dimension("\\hsize") {
    list.set_property("width", Stored::Dimension(hsize));
  }
  list
}

/// Perl's List() single-item simplification for vertical modes.
///
/// Perl (List.pm line 41-44):
/// ```perl
/// if ((scalar(@boxes) == 1)
///     && (!$mode || ($mode !~ /vertical$/)
///         || (($boxes[0]->getProperty('mode')||'') =~ /vertical$/))) {
///     return $boxes[0]; }   # Simplify!
/// ```
///
/// When a List in vertical mode contains a single non-empty item whose mode is also
/// vertical, return that item directly. This is critical for nested \vbox/\vtop:
/// the inner \vbox Whatsit has `is_vbox = true` set by after_digest, and the outer
/// \vtop constructor needs to see this property to skip double insertBlock wrapping.
fn simplify_vertical_list(item: Digested) -> Digested {
  // Only simplify if the item is a List
  let is_vertical_list = match item.data() {
    DigestedData::List(l) => {
      let list = l.borrow();
      // Check if the List's mode property indicates vertical
      list
        .properties
        .get("mode")
        .map(|m| m.ends_with_text("vertical"))
        .unwrap_or(false)
    },
    _ => false,
  };
  if !is_vertical_list {
    return item;
  }

  // Extract the List's boxes, filtering out empty marker items (isEmpty property)
  let non_empty: Vec<Digested> = match item.data() {
    DigestedData::List(l) => {
      let list = l.borrow();
      list
        .boxes
        .iter()
        .filter(|b| !b.get_property_bool("isEmpty"))
        .cloned()
        .collect()
    },
    _ => unreachable!(),
  };

  // Perl simplification: single non-empty item whose mode is also vertical
  if non_empty.len() == 1 {
    let single = &non_empty[0];
    let child_is_vertical = match single.data() {
      DigestedData::List(l) => l
        .borrow()
        .properties
        .get("mode")
        .map(|m| m.ends_with_text("vertical"))
        .unwrap_or(false),
      DigestedData::Whatsit(w) => {
        // Check whatsit's mode property (set by DefConstructor mode => "internal_vertical")
        w.borrow()
          .get_property("mode")
          .map(|m| m.ends_with_text("vertical"))
          .unwrap_or(false)
      },
      _ => false,
    };
    if child_is_vertical {
      return non_empty.into_iter().next().unwrap();
    }
  }
  item
}

/// Perl: revertSpec($whatsit, $keyword)
/// If whatsit has property $keyword, return Explode($keyword) ++ Revert($value)
pub fn revert_spec(whatsit: &Whatsit, keyword: &str) -> Vec<Token> {
  match whatsit.get_property(keyword) {
    Some(value) => {
      // Explode the keyword + value strings into T_OTHER tokens. `pin_char`
      // uses a stack-buffer encode_utf8 and skips the per-char
      // `c.to_string()` heap alloc the previous version did.
      let mut tokens: Vec<Token> = keyword.chars().map(|c| CharToken!(c)).collect();
      let val_str = value.to_attribute();
      tokens.extend(val_str.chars().map(|c| CharToken!(c)));
      tokens
    },
    _ => Vec::new(),
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

/// Perl `isVAttached` (TeX_Box.pool.ltxml:433-440): the node, or its only child all the way
/// down, has `vattach`. Every child node counts, text included: a `<p>` holding a v-attached
/// box and the text after it is not v-attached (`\parbox{5cm}{\parbox{2cm}{Inner} after
/// inner.}`, which since 57d builds that `<p>`; counting elements alone dropped the outer box's
/// `vattach`). Guard `perfect_kernel_batch56::a_nested_parbox_keeps_its_vattach`. The function
/// matches Perl's; the DOM it reads may not: a paragraph's trimmed trailing whitespace node is
/// freed here and kept empty by Perl (KNOWN_PERL_ERRORS #311).
fn is_v_attached(node: &Node) -> bool {
  let mut current = node.clone();
  loop {
    if current.get_attribute("vattach").is_some() {
      return true;
    }
    match current.get_child_nodes().as_slice() {
      [only] if only.get_type() == Some(NodeType::ElementNode) => current = only.clone(),
      _ => return false,
    }
  }
}

/// Make `node`'s content Block.class-valid so `node` can safely become a `<ltx:block>`,
/// demoting the Para.class descendants in place. Every direct `<ltx:logical-block>`/
/// `<ltx:sectional-block>` child is renamed to `<ltx:block>` — KEEPING its attributes (a
/// minipage's `width`, alignment, …) — and every direct `<ltx:para>` child is unwrapped
/// (its Block.model children float up, already valid in a block; a `<para>` never
/// directly holds Para.class, so no pre-demotion is needed). Other children (a `<p>`,
/// `<tabular>`, `<itemize>`, a nested `<block>`, …) are valid there and left untouched,
/// so paragraphs inside a still-valid Para.model container (an `<item>`) are never
/// over-flattened.
///
/// BOTTOM-UP: a logical-block child is demoted (its own Para.class content fixed) BEFORE
/// it is renamed to `<block>`, and the caller renames `node` itself only AFTER calling
/// this — so no Para.class node is ever moved into a `<block>` while still Para.class,
/// which would fire the constructor's `malformed`-content check (a spurious `Error:`).
///
/// Memory-safe snapshot iteration: `rename_node` recreates the child (its handle dies —
/// we recurse into the child BEFORE renaming, via the live handle) and `unwrap_nodes`
/// splices the child's kids up; neither moves the SIBLING handles held in the
/// `get_child_nodes` snapshot, and floated/renamed nodes are never revisited through it.
/// True when `node`'s content can be losslessly demoted into a `<ltx:block>`: every
/// element that would end up as a direct child of the block is valid there. Descends
/// only through the transformable Para.class wrappers (`ltx:para`/`ltx:logical-block`/
/// `ltx:sectional-block` — para is unwrapped, the others renamed to `<block>`), whose
/// own content must likewise be demotable; any OTHER element is a leaf kept as-is, so it
/// must itself be block-valid. A container holding a Para.model-only element that
/// `<block>` cannot hold — a `<float>`, `<TOC>`, a sectioning unit, a `<figure>`/`<table>`
/// — is NOT demotable (renaming to block would strand it, e.g. stanli/webquiz), so the
/// caller leaves it at Perl-parity rather than emitting a fresh `malformed` error.
fn subtree_is_block_demotable(node: &Node) -> bool {
  node.get_child_nodes().iter().all(|c| {
    if c.get_type() != Some(NodeType::ElementNode) {
      return true;
    }
    let transformable = with(document::get_node_qname(c), |q| {
      matches!(q, "ltx:para" | "ltx:logical-block" | "ltx:sectional-block")
    });
    if transformable {
      subtree_is_block_demotable(c)
    } else {
      document::can_contain_qsym(pin_static("ltx:block"), document::get_node_qname(c))
    }
  })
}

fn demote_para_class_content(document: &mut Document, node: &Node) -> Result<()> {
  for child in node.get_child_nodes() {
    if child.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let kind = with(document::get_node_qname(&child), |q| match q {
      "ltx:logical-block" | "ltx:sectional-block" => 1u8,
      "ltx:para" => 2,
      _ => 0,
    });
    match kind {
      1 => {
        demote_para_class_content(document, &child)?; // fix content while child still allows Para.class
        document.rename_node(child, "ltx:block", true)?; // THEN rename (content now Block.class-valid)
      },
      2 => document.unwrap_nodes(child)?,
      _ => {},
    }
  }
  Ok(())
}

/// A sectioning unit, read from the schema: admitted by `ltx:sectional-block` (model
/// `document.body.class*`) but not by `ltx:logical-block` (`Para.model`) — part,
/// chapter, section, subsection, subsubsection, paragraph, slide, slidesequence,
/// sidebar, sectional-block; never a figure/table/theorem/para (those are
/// Para.model). `ltx:subparagraph` is not in `document.body` (LaTeXML.model), so a
/// lone top-level `\subparagraph` in a box keeps Perl's shape; one under a floated
/// `\paragraph` rides along inside it.
fn is_sectioning_unit(node: &Node) -> bool {
  let q = document::get_node_qname(node);
  document::can_contain_qsym(pin_static("ltx:sectional-block"), q)
    && !document::can_contain_qsym(pin_static("ltx:logical-block"), q)
}

/// Can `unit` be floated out of `context`? Walk up while each node either admits
/// the unit (then a live `\\section` would open there) or auto-closes (a section,
/// a paragraph, the box's own wrappers). A `<quote>`, a list, a `<figure>` does
/// neither, so a box inside one keeps Perl's shape (#250).
fn sectioning_floatable(context: &Node, unit: SymStr) -> bool {
  let mut n = context.clone();
  loop {
    if document::can_contain_qsym(document::get_node_qname(&n), unit) {
      return true;
    }
    if !document::can_auto_close(&n) {
      return false;
    }
    match n.get_parent() {
      Some(p) if p.get_type() == Some(NodeType::ElementNode) => n = p,
      _ => return false,
    }
  }
}

/// The floats whose children are arranged into panels when they close (the `Tag!` `after_close`
/// hooks of sect09.rs: `arrange_panels`, `collapse_float`).
pub(crate) const PANEL_FLOATS: [&str; 3] = ["ltx:figure", "ltx:table", "ltx:float"];

/// Would text arriving at `tag` open an `ltx:p`? Follows the model's auto-open route for
/// `#PCDATA` (a section opens `ltx:para`, a para `ltx:p`); a route that ends in another text
/// holder (`svg:foreignObject` in a drawing, `ltx:bibblock` in a bibitem) opens none.
fn text_opens_paragraph(tag: SymStr) -> bool {
  let (text, p) = (pin_static("#PCDATA"), pin_static("ltx:p"));
  let mut step = tag;
  // The route is a handful of steps in LaTeXML.model; the bound guards a model cycle.
  for _ in 0..8 {
    match document::sym_can_contain_somehow(step, text) {
      Some(Some(open)) if open == p => return true,
      Some(Some(open)) => step = open,
      _ => return false,
    }
  }
  false
}

/// Figure material: what a figure shows beside its caption — graphics, pictures, tabulars — and the
/// wrappers a box body puts around them (`para`/`p`, a `\scalebox` inline block, a font `text`),
/// with no prose: text nodes are whitespace only.
fn is_figure_material(node: &Node) -> bool {
  match node.get_type() {
    Some(NodeType::TextNode) => node.get_content().trim().is_empty(),
    Some(NodeType::ElementNode) => with(document::get_node_qname(node), |q| match q {
      "ltx:graphics" | "ltx:picture" | "svg:svg" | "ltx:tabular" | "ltx:rule" | "ltx:break" => true,
      "ltx:para" | "ltx:p" | "ltx:block" | "ltx:inline-block" | "ltx:text" => {
        node.get_child_nodes().iter().all(is_figure_material)
      },
      _ => false,
    }),
    _ => true,
  }
}

/// A `para`/`p` left holding nothing but whitespace and such shells once its material moved.
fn is_empty_shell(node: &Node) -> bool {
  with(document::get_node_qname(node), |q| {
    q == "ltx:para" || q == "ltx:p"
  }) && node
    .get_child_nodes()
    .iter()
    .all(|child| match child.get_type() {
      Some(NodeType::TextNode) => child.get_content().trim().is_empty(),
      Some(NodeType::ElementNode) => is_empty_shell(child),
      _ => true,
    })
}

/// The figure material inside `node`, its `para`/`p` wrappers peeled off: the leaves a figure holds.
fn figure_material_leaves(node: &Node, leaves: &mut Vec<Node>) {
  match node.get_type() {
    Some(NodeType::ElementNode)
      if with(document::get_node_qname(node), |q| {
        q == "ltx:para" || q == "ltx:p"
      }) =>
    {
      for child in node.get_child_nodes() {
        figure_material_leaves(&child, leaves);
      }
    },
    Some(NodeType::ElementNode) => leaves.push(node.clone()),
    _ => {},
  }
}

/// A box that is clearly a figure (user ruling 2026-09-27, 57g): its content is one float holding
/// only its caption — the shape `\captionof` makes, wrapping only the caption in the float
/// (`\@captionof@`, caption_sty.rs; Perl the same, KNOWN_PERL_ERRORS #312) and leaving the image
/// beside it — and figure material. The float takes the material in, in source order (what
/// precedes the caption goes before it), so the box becomes the float with its image:
/// `<figure class="ltx_minipage"><graphics/><caption/></figure>` (arXiv 2605.27134's parbox
/// panels, 2605.06890's `\captionof{table}` minipages). A float that already holds content of its
/// own (`\begin{figure}[H]…`) takes nothing: its order with the material is the author's.
/// Returns the float.
fn absorb_figure_material(document: &mut Document, nodes: &[Node]) -> Result<Option<Node>> {
  let is_float = |n: &Node| {
    n.get_type() == Some(NodeType::ElementNode)
      && with(document::get_node_qname(n), |q| PANEL_FLOATS.contains(&q))
  };
  let floats: Vec<usize> = (0..nodes.len()).filter(|&i| is_float(&nodes[i])).collect();
  let [at] = floats.as_slice() else {
    return Ok(None);
  };
  let float = nodes[*at].clone();
  let float_children = float.get_child_elements();
  let caption_only = float_children.iter().all(|c| {
    with(document::get_node_qname(c), |q| {
      q == "ltx:tags" || q == "ltx:toccaption" || q == "ltx:caption"
    })
  });
  let caption = float_children.into_iter().find(|c| {
    with(document::get_node_qname(c), |q| {
      q == "ltx:toccaption" || q == "ltx:caption"
    })
  });
  let (true, Some(mut caption)) = (caption_only, caption) else {
    return Ok(None);
  };
  let others: Vec<&Node> = nodes
    .iter()
    .enumerate()
    .filter(|(i, _)| *i != *at)
    .map(|(_, n)| n)
    .collect();
  if !others
    .iter()
    .any(|n| n.get_type() == Some(NodeType::ElementNode))
    || !others.iter().all(|n| is_figure_material(n))
  {
    return Ok(None);
  }
  let mut float = float;
  for (i, node) in nodes.iter().enumerate() {
    if i == *at || node.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let mut leaves = Vec::new();
    figure_material_leaves(node, &mut leaves);
    for mut leaf in leaves {
      leaf.unlink();
      if i < *at {
        caption.add_prev_sibling(&mut leaf)?;
      } else {
        float.add_child(&mut leaf)?;
      }
    }
    // The emptied `para`/`p` shells, nested ones included (a leaf that was its own node has
    // moved, not emptied).
    if is_empty_shell(node) {
      document.remove_node(node.clone());
    }
  }
  Ok(Some(float))
}

/// Is `context` a float's own content, seen through the captures of the boxes and environments
/// (`{center}`, `{minipage}`) built in it? A float's content is arranged into panels
/// (`arrange_panels`, Perl's `arrange_panels_and_breaks`, latex_constructs.pool.ltxml:3229-3349).
fn in_panel_float(context: &Node) -> bool {
  let mut node = context.clone();
  while document::is_capture_block(&node) {
    match node.get_parent() {
      Some(parent) if parent.get_type() == Some(NodeType::ElementNode) => node = parent,
      _ => return false,
    }
  }
  with(document::get_node_qname(&node), |q| {
    PANEL_FLOATS.contains(&q)
  })
}

pub fn insert_block(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
) -> Result<Vec<Node>> {
  insert_block_as(document, contents, block_attr, false)
}

/// [`insert_block`] for a box that is paragraph material: its definition's `\leavevmode`
/// (`enter_horizontal`) left TeX in a paragraph, the whatsit's `in_paragraph` property
/// (`Constructor::digest_to_whatsit`). latex.ltx's `\parbox` ends in `\@iiiparbox` and `{minipage}`
/// in `\@iiiminipage`, which begin with `\leavevmode` (16249-16250, 16305-16306): between
/// paragraphs the box starts one, and the text after it continues that paragraph (KNOWN_PERL_ERRORS
/// #309, DIVERGENCES #338).
pub fn insert_block_in_paragraph(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
  props: &SymHashMap<Stored>,
) -> Result<Vec<Node>> {
  let in_paragraph = matches!(props.get("in_paragraph"), Some(Stored::Bool(true)));
  let Some(id) = props.get("caption_id").map(|id| id.to_string()) else {
    return insert_block_as(document, contents, block_attr, in_paragraph);
  };
  let captype = props
    .get("caption_type")
    .map(|t| t.to_string())
    .unwrap_or_default();
  // The float the caption is of — the nearest enclosing float element of its type — numbered by it for now (when no
  // caption of its own did), so that what the box's content builds forms under that number as before: its ids, and
  // the `\label`s `float_to_label` gives the nearest numbered ancestor (an `\fbox`, a `\colorbox` or an `lrbox` around
  // the box makes no panel of it, and the float keeps the number).
  let float = caption_float(document, &captype);
  // A float an earlier box's panel took the number from holds a placeholder id meanwhile (`adopt_box_caption`).
  let float_id = float.as_ref().and_then(node_xml_id);
  let placeholder = float_id
    .clone()
    .filter(|held| lookup_bool(&s!("{FLOAT_PLACEHOLDER_ID}{held}")));
  let provisional = match float.clone() {
    Some(mut float) if float_id.is_none() || placeholder.is_some() => {
      if let Some(held) = &placeholder {
        document.unrecord_id(held);
        remove_xml_id(&mut float);
        remove_value(&s!("{FLOAT_PLACEHOLDER_ID}{held}"));
      }
      document.set_attribute(&mut float, "xml:id", &id)?;
      Some((float, placeholder))
    },
    _ => None,
  };
  let open = ancestor_labels(document);
  let mut nodes = insert_block_as(document, contents, block_attr, in_paragraph)?;
  adopt_box_caption(document, &mut nodes, props, &id, provisional, open)?;
  Ok(nodes)
}

/// The state key that marks an id as a float's placeholder (`adopt_box_caption`), by id.
const FLOAT_PLACEHOLDER_ID: &str = "lx@float@placeholder@";

fn node_xml_id(node: &Node) -> Option<String> { node.get_attribute_ns("id", XML_NS) }

/// An `xml:id` is the `id` attribute in the XML namespace, which only the namespaced accessor removes.
pub(crate) fn remove_xml_id(node: &mut Node) { let _ = node.remove_attribute_ns("id", XML_NS); }

/// The nearest enclosing float element of a caption's type: `ltx:figure`, `ltx:table`, or `ltx:float` of class
/// `ltx_float_<type>` (sect09.rs).
fn caption_float(document: &Document, captype: &str) -> Option<Node> {
  let mut node = document.get_element();
  while let Some(n) = node {
    if with(document::get_node_qname(&n), |q| PANEL_FLOATS.contains(&q)) {
      return is_float_of_type(&n, captype).then_some(n);
    }
    node = n.get_parent();
  }
  None
}

fn is_float_of_type(node: &Node, captype: &str) -> bool {
  with(document::get_node_qname(node), |q| match q {
    "ltx:figure" => captype == "figure",
    "ltx:table" => captype == "table",
    "ltx:float" => node.get_attribute("class").is_some_and(|classes| {
      let class = s!("ltx_float_{captype}");
      classes.split_whitespace().any(|c| c == class)
    }),
    _ => false,
  })
}

/// Each element from the insertion point up — the nodes open while a box is built — with its `labels` as they stand.
fn ancestor_labels(document: &Document) -> Vec<(Node, String)> {
  let mut ancestors = Vec::new();
  let mut node = document.get_element();
  while let Some(n) = node {
    if !matches!(n.get_type(), Some(NodeType::ElementNode)) {
      break;
    }
    ancestors.push((n.clone(), n.get_attribute("labels").unwrap_or_default()));
    node = n.get_parent();
  }
  ancestors
}

/// A box that holds its own caption in a float (`end_caption_box`) is that caption's float: the panel it became —
/// itself, or the figure panel it holds, never a node that was open around it — takes the caption's id (from the
/// float that held it for now, `provisional_float`), tags and list, becomes the element of the float it is in when
/// that float is of the caption's type (a captioned minipage in a `table` is a `table`, which `insert_block` never
/// picks; a `\newfloat` one keeps its `ltx_float_<type>` class), and takes the `\label`s its content left on the nodes
/// open around it. With no panel, a float numbered for now keeps the number, and takes its tags and list too.
fn adopt_box_caption(
  document: &mut Document,
  nodes: &mut [Node],
  props: &SymHashMap<Stored>,
  id: &str,
  provisional_float: Option<(Node, Option<String>)>,
  open: Vec<(Node, String)>,
) -> Result<()> {
  let caption_qname = pin_static("ltx:caption");
  let is_open = |n: &Node| open.iter().any(|(o, _)| o == n);
  let in_math = |n: &Node| {
    let mut node = n.get_parent();
    while let Some(p) = node {
      if with(document::get_node_qname(&p), |q| {
        q.starts_with("ltx:XM") || q == "ltx:Math"
      }) {
        return true;
      }
      node = p.get_parent();
    }
    false
  };
  // A panel built here: a float element holding the caption, outside math (whose ids are the math's own).
  let built_panel = |n: &Node| {
    with(document::get_node_qname(n), |q| PANEL_FLOATS.contains(&q))
      && !is_open(n)
      && !in_math(n)
      && n
        .get_child_elements()
        .iter()
        .any(|c| document::get_node_qname(c) == caption_qname)
  };
  let panel = nodes.iter().find(|n| built_panel(n)).cloned().or_else(|| {
    nodes
      .first()
      .and_then(|n| n.get_parent())
      .filter(|p| built_panel(p))
  });
  let Some(mut panel) = panel else {
    if let Some((float, _)) = provisional_float {
      number_caption_float(document, float, props)?;
    }
    return Ok(());
  };
  // The id: the float's, held for it, or the caption's own. The float keeps an id for what is built in it later, a
  // placeholder the next box holds its caption's id in: without one a `\label` there would reach the document.
  if let Some((mut float, placeholder)) = provisional_float {
    if let Some(held) = node_xml_id(&float) {
      document.unrecord_id(&held);
    }
    remove_xml_id(&mut float);
    match placeholder {
      Some(held) => document.set_attribute(&mut float, "xml:id", &held)?,
      None => {
        let prefix = if with(document::get_node_qname(&float), |q| q == "ltx:figure") {
          "fig"
        } else {
          "tab"
        };
        document.generate_id(&mut float, prefix)?;
      },
    }
    if let Some(held) = node_xml_id(&float) {
      assign_value(
        &s!("{FLOAT_PLACEHOLDER_ID}{held}"),
        true,
        Some(Scope::Global),
      );
    }
  }
  if let Some(old) = node_xml_id(&panel) {
    document.unrecord_id(&old);
    remove_xml_id(&mut panel);
  }
  document.set_attribute(&mut panel, "xml:id", id)?;
  // The element of the float it is in, when that float is of the caption's type.
  let captype = props
    .get("caption_type")
    .map(|t| t.to_string())
    .unwrap_or_default();
  if let Some(float) = open
    .iter()
    .map(|(o, _)| o)
    .find(|o| with(document::get_node_qname(o), |q| PANEL_FLOATS.contains(&q)))
    && is_float_of_type(float, &captype)
  {
    let float_qname = document::get_node_qname(float);
    if document::get_node_qname(&panel) != float_qname
      && panel
        .get_parent()
        .is_some_and(|p| with(float_qname, |q| document::can_contain(&p, q)))
    {
      let old = panel.clone();
      panel = document.rename_node_qsym(panel, float_qname, true)?;
      for node in nodes.iter_mut() {
        if *node == old {
          *node = panel.clone();
        }
      }
    }
    if with(float_qname, |q| q == "ltx:float") {
      document.add_class(&mut panel, &s!("ltx_float_{captype}"))?;
    }
  }
  let panel = number_caption_float(document, panel, props)?;
  // The labels its content left on the nodes open around it.
  let mut moved: Vec<String> = Vec::new();
  for (mut ancestor, before) in open {
    let Some(after) = ancestor.get_attribute("labels") else {
      continue;
    };
    let kept: Vec<&str> = before.split_whitespace().collect();
    let added: Vec<&str> = after
      .split_whitespace()
      .filter(|l| !kept.contains(l))
      .collect();
    if added.is_empty() {
      continue;
    }
    moved.extend(added.iter().map(|l| l.to_string()));
    if kept.is_empty() {
      ancestor.remove_attribute("labels")?;
    } else {
      document.set_attribute(&mut ancestor, "labels", &kept.join(" "))?;
    }
  }
  if !moved.is_empty() {
    let mut panel = panel;
    let mut labels: Vec<String> = panel
      .get_attribute("labels")
      .map(|l| l.split_whitespace().map(str::to_string).collect())
      .unwrap_or_default();
    for label in moved {
      if !labels.contains(&label) {
        labels.push(label);
      }
    }
    document.set_attribute(&mut panel, "labels", &labels.join(" "))?;
  }
  Ok(())
}

/// `float` takes a caption's tags (first, where a float's own template places them) and list (`end_caption_box`'s
/// `caption_tags` and `caption_inlist`).
fn number_caption_float(
  document: &mut Document,
  mut float: Node,
  props: &SymHashMap<Stored>,
) -> Result<Node> {
  if let Some(inlist) = props.get("caption_inlist").map(|l| l.to_string())
    && !inlist.trim().is_empty()
  {
    document.set_attribute(&mut float, "inlist", inlist.trim())?;
  }
  if let Some(Stored::Digested(tags)) = props.get("caption_tags") {
    let save = document.get_node().clone();
    document.set_node(&float);
    let before = float.get_last_element_child();
    document.absorb(tags, None)?;
    document.set_node(&save);
    let mut added = float.get_last_element_child();
    if added != before
      && let Some(tags) = added.as_mut()
      && let Some(mut first) = float.get_first_element_child()
      && &first != tags
    {
      tags.unlink();
      first.add_prev_sibling(tags)?;
    }
  }
  Ok(float)
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
fn insert_block_as(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
  in_paragraph: bool,
) -> Result<Vec<Node>> {
  // Perl #2829 (TeX_Box.pool insertBlock L449-456): callers may (sloppily)
  // pass ALL Whatsit properties, but only SOME are intended as attributes.
  // Filter out non-attributes, based on what ltx:figure accepts — a maximal
  // set of the attributes for candidate containers.
  let block_attr: HashMap<String, String> = block_attr
    .into_iter()
    .filter(|(k, _)| document::can_have_attribute("ltx:figure", k))
    .collect();
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
  // An `svg:foreignObject` context is LaTeXML content again (it holds ltx
  // elements), not SVG proper: the block keeps its attributes and its own
  // wrapper there. With the foreignObject model widened to Flow
  // (OXIDIZED_DESIGN #186) the SVG rule "attributes are ignorable" would
  // unwrap a minipage straight into the foreignObject and hoist its
  // `width="31.37em"` over the foreignObject's px width (fixture
  // tikz/various_colors).
  let (is_svg, is_xmath, is_xmtext) = with(context_tag, |tag| {
    (
      tag.starts_with("svg:") && tag != "svg:foreignObject",
      tag.starts_with("ltx:XM"),
      tag == "ltx:XMText",
    )
  });
  // The width stays the Dimension (Perl's insertBlock), in SVG context too: a block a measured foreignObject holds
  // is rewritten to ems of that foreignObject's anchor by its sizer (tex_box.rs), the one basis the browser reads.
  let ignorable_attr = is_svg || block_attr.is_empty(); // if we do not REQUIRE the attributes
  if is_xmath && !is_xmtext {
    // but math always needs this
    context = document.open_element("ltx:XMText", None, None)?;
    context_tag = document::get_node_qname(&context);
  }
  // An inline-only container that holds `inline-block` (Misc.class) but neither
  // `#PCDATA` nor `block` (Block.class): the picture family `ltx:g`/`ltx:picture`/
  // `ltx:clippath`, plus `ltx:creator`/`ltx:equation`/`ltx:inline-item` (the full set
  // this clause flips — verified against LaTeXML.model). `<block>` is schema-invalid in
  // every one, and `<inline-block>` is valid there AND holds full Block.class content,
  // so a `\put`-positioned `\parbox`/`\makebox` digested there — an LR-box, never a
  // block — is renamed to a schema-valid `<inline-block>` IN PLACE: no move, so the
  // `\put` position in the enclosing `<g transform="translate(x,y)">` is preserved
  // (fidelity-safe). Emitting `<block>` here is invalid (e.g. `g_model`,
  // LaTeXML-picture.rng) — a shared Perl bug (`TeX_Box.pool.ltxml:493`), surpassed on
  // the schema-validity axis like #240. A `<para>`/`<inline-block>`/`<float>`/`<note>`
  // holds BOTH block and inline-block, so this clause is false there and the #240
  // block-climb (`context_tag == "ltx:para"`) stays untouched.
  let mut is_inline = is_svg
    || document::can_contain(&context, "#PCDATA")
    || (document::can_contain_qsym(context_tag, pin_static("ltx:inline-block"))
      && !document::can_contain_qsym(context_tag, pin_static("ltx:block")));
  let container_attr = block_attr.clone();
  let mut container = document.open_element("ltx:_CaptureBlock_", Some(container_attr), None)?;
  document.absorb(contents, None)?;

  let mut nodes = content_nodes(&container);
  let mut node_tags = nodes
    .iter()
    .map(document::get_node_qname)
    .collect::<Vec<_>>();
  document.close_to_node(&container, true)?;
  document.close_node(&container)?;
  document.close_to_node(&context, true)?;

  // A sectioning unit inside a block-mode box (`\\section` in a `minipage[t]`, a
  // `framed`/`tcolorbox`) is a REAL section in LaTeX: `\\@startsection` ran, the
  // counter advanced, and the text after the box belongs to the new unit. Perl's
  // `insertBlock` (TeX_Box.pool.ltxml:512) instead wraps the box as
  // `<sectional-block>` and leaves it inside the enclosing section, where the
  // schema admits no `sectional-block` (only `document.body.class` does,
  // LaTeXML-structure.rnc:99) — SHARED, and invalid. User ruling (2026-09-20): the
  // box is presentation, the sectioning is semantics — never bend the schema to a
  // layout container. So when the captured content holds a sectioning unit and the
  // insertion context has an auto-closeable path to an ancestor that admits it, every
  // built unit is MOVED to where a live `\\section` would open (closing the enclosing
  // unit for a same-or-higher level, nesting a deeper one) and becomes the insertion
  // node, so trailing box content and post-box document text nest inside it; the
  // box's own annotation (`class="ltx_minipage"` and the frame attributes
  // `Sectional.attributes` admit — `width`/`vattach` are not, as `sectional-block`
  // already dropped them) rides the FIRST floated unit. A box inside a `<quote>`, a
  // list `<item>` or a `<figure>` has no such path (none auto-closes to a section
  // holder) and keeps today's Perl-parity shape. Witnesses: aguplus, fancyvrb-doc,
  // latex-via-exemplos, phonenumbers-{de,en}, recorder-fingering, tasks-manual,
  // unamth-template/tesis (8 s106 docs). OXIDIZED_DESIGN #250.
  if !is_inline
    && let Some(first_unit) = nodes.iter().find(|n| is_sectioning_unit(n))
    && sectioning_floatable(&context, document::get_node_qname(first_unit))
  {
    document.set_node(&context);
    // The floated units are what the caller gets back: `aligning_environment`
    // ({center}, {flushleft}, {flushright}) stamps its alignment on them.
    let mut floated: Vec<Node> = Vec::new();
    for child in nodes.iter().cloned() {
      if child.get_type() != Some(NodeType::ElementNode) {
        continue;
      }
      let q = document::get_node_qname(&child);
      let mut point = document.find_insertion_point_qsym(q, None)?;
      let mut c = child;
      c.unlink();
      point.add_child(&mut c)?;
      if is_sectioning_unit(&c) {
        floated.push(c.clone());
        document.set_node(&c);
      }
    }
    if let Some(mut unit) = floated.first().cloned() {
      let unit_q = document::get_node_qname(&unit);
      for (k, v) in block_attr.iter() {
        if k == "class" {
          document.add_class(&mut unit, v)?;
        } else if document::sym_can_have_attribute(unit_q, pin(k)) {
          document.set_attribute(&mut unit, k, v)?;
        }
      }
    }
    // Everything of substance has been moved out; the capture is spent.
    document.remove_node(container);
    return Ok(floated);
  }

  // A box that is clearly a figure becomes its float, holding its image (`absorb_figure_material`,
  // 57g); whether it then stands as that float or sits in a paragraph is settled when the
  // paragraph closes (`settle_float_paragraph`, latex_constructs).
  if let Some(float) = absorb_figure_material(document, &nodes)? {
    node_tags = vec![document::get_node_qname(&float)];
    nodes = vec![float];
  }
  // Paragraph material (`insert_block_in_paragraph`): where text would open an `ltx:p`, the box
  // goes into that paragraph, which stays open for the text after it. Decided after the capture,
  // so the sectioning float-out above (#250) still sees the original context. Not in a float,
  // whose content is panels (`in_panel_float`: arXiv 2605.27134's `\parbox` at a figure
  // minipage's top; captioned `\parbox` panels in a figure's `{center}` stay sub-figures); not
  // inside a restricted box (`\rotatebox{90}{\parbox…}`, arXiv 2605.20645), where the flag is
  // unset; not for an empty box, which leaves no material; not for content no inline box holds
  // (a bibliography, an index, a caption), which the hoist below places after the box as before.
  // A box whose whole content is one float goes into the paragraph too; a paragraph of nothing but
  // such boxes becomes those floats when it closes (user ruling 2026-09-27: `<figure
  // class="ltx_minipage">`, Perl's fold, for a lone one; a row of panels for several).
  if in_paragraph
    && !is_inline
    && !nodes.is_empty()
    && text_opens_paragraph(context_tag)
    && !in_panel_float(&context)
    && [
      "ltx:inline-block",
      "ltx:inline-logical-block",
      "ltx:inline-sectional-block",
    ]
    .iter()
    .any(|inline| {
      let inline = pin_static(inline);
      node_tags
        .iter()
        .all(|tag| document::sym_can_contain_somehow(inline, *tag).is_some())
    })
  {
    let mut paragraph = document.open_element("ltx:p", None, None)?;
    container.unlink();
    paragraph.add_child(&mut container)?;
    context = paragraph;
    context_tag = document::get_node_qname(&context);
    is_inline = true;
  }

  // Perl `insertBlock` (TeX_Box.pool.ltxml:406-472): the first candidate, in
  // preference order, that can hold ALL of the box's content becomes the
  // container (:457-458 give the candidate sets; :471 the hard `ltx:block`
  // fallback that then reports every child the model rejects). One general
  // rule extends it (Rust-only): when no candidate holds everything, the
  // first candidate that holds the box's LEADING content is the box, and the
  // content it cannot hold is placed after the box, in the nearest flow
  // ancestor that can hold it — the outcome an autoclosing frame produces
  // for the same content written live (`\printbibliography` in mdframed:
  // `perfect_kernel_gemini::mdframed_block_bibliography_juradiss`; a
  // `\caption` in a rotated `\parbox`: heria-proposal, rubik —
  // `perfect_kernel_batch56::caption_in_inline_parbox_floats_to_figure`).
  // The climb stays inside text containers (an ancestor holding paragraphs
  // or text): a drawing or a math box is a different medium, and content
  // hoisted past it would strand the rest of the box (Gemini J3,
  // biblatex-ext) — there the model's own verdict stands, as in Perl.
  // `LXML_TRACE_INSERT_BLOCK=1` prints each decision.
  let all_candidates = [
    "ltx:inline-block",
    "ltx:inline-logical-block",
    "ltx:inline-sectional-block",
    "ltx:block",
    "ltx:logical-block",
    "ltx:sectional-block",
    "ltx:figure",
  ]
  .map(pin_static);
  let candidate_set: &[SymStr] = if is_inline {
    &all_candidates[0..3]
  } else {
    &all_candidates
  };
  let holds = |c: SymStr, t: SymStr| document::sym_can_contain_somehow(c, t).is_some();
  let holds_all = |c: SymStr| node_tags.iter().all(|t| holds(c, *t));
  let container_tag = candidate_set
    .iter()
    .copied()
    .find(|c| holds_all(*c))
    .or_else(|| {
      node_tags
        .first()
        .and_then(|first| candidate_set.iter().copied().find(|c| holds(*c, *first)))
    });
  let hoisted: Vec<bool> = node_tags
    .iter()
    .map(|t| container_tag.is_none_or(|c| !holds(c, *t)))
    .collect();
  if std::env::var_os("LXML_TRACE_INSERT_BLOCK").is_some() {
    eprintln!(
      "insert_block: context={} inline={} tags={:?} container={:?} hoisted={:?}",
      with(context_tag, |s| s.to_string()),
      is_inline,
      node_tags
        .iter()
        .map(|t| with(*t, |s| s.to_string()))
        .collect::<Vec<_>>(),
      container_tag.map(|c| with(c, |s| s.to_string())),
      hoisted
    );
  }
  if hoisted.iter().any(|h| *h) {
    // The box's content in source order, for a figure panel that takes all of it (below).
    let in_order = nodes.clone();
    let order_tags = node_tags.clone();
    let mut tail = Vec::new();
    let mut kept = Vec::new();
    let mut kept_tags = Vec::new();
    for (n, (node, tag)) in nodes.drain(..).zip(node_tags.iter().copied()).enumerate() {
      if hoisted[n] {
        tail.push(node);
      } else {
        kept.push(node);
        kept_tags.push(tag);
      }
    }
    let tail_tags: Vec<SymStr> = tail.iter().map(document::get_node_qname).collect();
    // A text container: holds paragraphs, or text itself (`ltx:p`, an
    // inline block). A drawing (`ltx:picture`, `svg:g`) holds neither —
    // backmatter trapped in a drawing (xebaposter's References `\headerbox`
    // is a pgf node → `svg:foreignObject`) still floats to the enclosing flow
    // exactly as from a plain minipage, but never at the cost of stranding
    // TEXT: the climb crosses a drawing only when no kept node carries
    // non-whitespace text (a graphic in the box is drawing content and stays
    // where it is; Perl's `adjustBackmatterElement` cannot cross at all —
    // `ltx:bibliography` isn't allowed in `ltx:block`, SHARED; batch 56bq).
    let kept_empty = kept.iter().all(|n| n.get_content().trim().is_empty());
    let flow = |n: &Node| {
      document::can_contain(n, "#PCDATA")
        || with(document::get_node_qname(n), |t| {
          document::can_contain_somehow(t, "ltx:p")
        })
    };
    // LIST: the climb never leaves a list. Perl floats nothing out of a list
    // (`adjustBackmatterElement` autocloses only `canAutoClose` nodes and a
    // drawing's `svg:g` is not one, latex_constructs.pool.ltxml:3936 /
    // Document.pm find_insertion_point): a bibliography arriving ALONE from a
    // tcolorbox inside an `\item` (`kept` empty, so the TEXT guard above did
    // not hold it) climbed through the `ltx:item`/`ltx:itemize` to the
    // section, which closed the list under the remaining items — they then
    // nested inside one another (biblatex-ext.tex:1301/1326 `optionlist` +
    // `bibexample`, 4 schema errors; RUST-ONLY). The block stays in the box
    // and the model reports it, as Perl does (SHARED `ltx:bibliography` in
    // `ltx:block`). The list-free hoists keep their surpass (xebaposter,
    // juradiss, the parbox caption). Guard
    // `bibliography_in_a_list_item_box_keeps_the_list`.
    let list_boundary = |n: &Node| {
      with(document::get_node_qname(n), |t| {
        matches!(
          t,
          "ltx:item"
            | "ltx:inline-item"
            | "ltx:itemize"
            | "ltx:enumerate"
            | "ltx:description"
            | "ltx:inline-itemize"
            | "ltx:inline-enumerate"
            | "ltx:inline-description"
        )
      })
    };
    let mut anchor = container.clone();
    let mut ancestor = context.clone();
    let mut placed = false;
    loop {
      if tail_tags
        .iter()
        .all(|t| with(*t, |s| document::can_contain(&ancestor, s)))
      {
        placed = true;
        break;
      }
      if list_boundary(&ancestor) || (!flow(&ancestor) && !kept_empty) {
        break;
      }
      // Never out of an alignment: the hoist moves the insertion point too, past a cell whose `</td>` (and row, and
      // table) the alignment still owes at its `&`, `\\` and `\end` — a caption in a minipage in a tabular cell left
      // `<td>` open in the figure (1601.03744, 5 malformed errors); a math `array` likewise. Perl's `floatToElement`
      // (Document.pm:1061-1072) floats past such nodes without closing them and restores the insertion point after;
      // a box the climb leaves here (the rotated `\parbox`) is closed already.
      let alignment = |n: &Node| {
        with(document::get_node_qname(n), |t| {
          matches!(
            t,
            "ltx:td"
              | "ltx:tr"
              | "ltx:thead"
              | "ltx:tbody"
              | "ltx:tfoot"
              | "ltx:tabular"
              | "ltx:XMCell"
              | "ltx:XMRow"
              | "ltx:XMArray"
          )
        })
      };
      // Nor out of a float: an appendix section (or a bibliography) in a minipage in a `table` was lifted past it,
      // leaving the table empty and its caption at the document's top level, with no diagnostic; it now stays in the
      // float and errors there, as Perl's does (a sectioning unit in a float errors, OXIDIZED_DESIGN_DIVERGENCES #189
      // ruling). Witness 2609.05763 (a nomenclature glossary in a framed minipage in a table); guard
      // `perfect_kernel_batch61::appendix_in_a_minipage_stays_in_its_float`.
      // The climb now has three per-tag stops (list, alignment, float), and a quote still leaks
      // (repros/boxes-groups/bibliography_in_minipage_in_quote_keeps_the_quote.tex): SYNC_STATUS 62zd.
      let float = |n: &Node| {
        with(document::get_node_qname(n), |t| {
          matches!(t, "ltx:table" | "ltx:figure" | "ltx:float")
        })
      };
      if alignment(&ancestor) || float(&ancestor) {
        break;
      }
      match ancestor.get_parent() {
        Some(p) if matches!(p.get_type(), Some(NodeType::ElementNode)) => {
          anchor = ancestor;
          ancestor = p;
        },
        _ => break,
      }
    }
    if placed {
      // `anchor` is an element and a box's content nodes are already
      // coalesced, so no two adjacent text nodes reach `add_next_sibling`
      // (libxml2 merges those and frees the second — see `replace_node`).
      let mut prev = anchor;
      for mut n in tail {
        n.unlink();
        prev.add_next_sibling(&mut n)?;
        prev = n;
      }
      document.set_node(&ancestor);
      nodes = kept;
      node_tags = kept_tags;
      if nodes.is_empty() {
        document.remove_node(container);
        return Ok(Vec::new());
      }
    } else if is_inline
      && order_tags
        .iter()
        .any(|t| with(*t, |s| s == "ltx:caption" || s == "ltx:toccaption"))
      && order_tags
        .iter()
        .all(|t| holds(pin_static("ltx:figure"), *t))
    {
      // A box in running text holding a caption, all of whose content a figure may hold — a minipage in a table
      // cell captioned as a figure, caption first or last: it becomes a figure panel in the box, its content in
      // source order, as the same minipage between paragraphs
      // becomes one (`figure class="ltx_figure_panel ltx_minipage"`), the box an inline logical block, whose model
      // holds a figure where a cell holds none (captions_in_minipages_in_a_tabular; Perl reports the captions in
      // an `ltx:block`).
      drop((kept, tail));
      if let Some(mut panel) = document.wrap_nodes("ltx:figure", in_order.clone())? {
        document.add_class(&mut panel, "ltx_figure_panel")?;
        nodes = vec![panel];
        node_tags = vec![pin_static("ltx:figure")];
      } else {
        // The model's verdict, as below (a box's own nodes always wrap).
        nodes = in_order;
        node_tags = order_tags;
      }
    } else {
      // No flow ancestor holds it: the content stays in the box and the
      // model reports it (Perl's outcome).
      nodes = kept;
      nodes.extend(tail);
      node_tags = kept_tags;
      node_tags.extend(tail_tags);
    }
  }
  let nnodes = nodes.len();

  // Perl: Hack: apparently TeX doesn't shift (vattach) a single node in a vbox/vtop/...
  #[allow(clippy::redundant_locals)]
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
          .all(|key| document::sym_can_have_attribute(node_tags[0], pin(key))))
    {
      // IF: Single node, allowed in context & accepts attributes
      // THEN: Add attributes and unwrap the single node
      //
      // SURPASS-PERL (OXIDIZED_DESIGN #125): `class` MERGES, it does not
      // overwrite. Perl's insertBlock (`TeX_Box.pool.ltxml` L492) uses
      // `setAttribute(class => …)`, so a wrapper's class (e.g. minipage's
      // `ltx_minipage`) clobbered the single child's own class — a `lstlisting`
      // that is the sole content of a `minipage`-in-`figure` became
      // `class="ltx_minipage"`, losing `ltx_lstlisting` and thus the
      // whitespace-preserving CSS, so its indentation collapsed
      // (html_feedback#6632, arXiv:2605.03143). LaTeXML already distinguishes
      // `addClass` from `setAttribute`; merging the child's class with the
      // wrapper's keeps both (`ltx_lstlisting ltx_minipage`).
      for (k, v) in block_attr.iter() {
        if k == "class" {
          document.add_class(&mut nodes[0], v)?;
        } else {
          document.set_attribute(&mut nodes[0], k, v)?;
        }
      }
      document.unwrap_nodes(container)?;
      return Ok(nodes);
    } else if let Some(newcontainer) = document::sym_can_contain_somehow(context_tag, node_tags[0])
      && (ignorable_attr
        || block_attr.keys().all(|key| {
          newcontainer
            .map(|nc| document::sym_can_have_attribute(nc, pin(key)))
            .unwrap_or(false)
        }))
      && let Some(nc) = newcontainer
      // never an SVG element: with the foreignObject model widened to Flow
      // (OXIDIZED_DESIGN #186) a minipage in a TikZ node resolved here to
      // `svg:foreignObject` and the capture — class, vattach, `width` —
      // was renamed INTO a foreignObject, clobbering the real one's px
      // width (fixture tikz/various_colors); it belongs in the inline-block
      // the candidates below pick.
      && !with(nc, |s| s.starts_with("svg:"))
    {
      // rename the capture to that container
      document.rename_node_qsym(container, nc, true)?;
      return Ok(nodes);
    }
  }
  // This jagged conditional is a "code smell", due to the difficulty of refactoring
  // the in-conditional-assignments from Perl.

  // Otherwise, rename the capture
  // MAY need foreignObject wrapper
  if is_svg
    && node_tags
      .iter()
      .any(|tag| with(*tag, |tag_str| tag_str.starts_with("ltx:")))
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
    .map(pin_static)
    .to_vec()
  } else {
    [
      "ltx:block",
      "ltx:logical-block",
      "ltx:sectional-block",
      "ltx:figure",
    ]
    .map(pin_static)
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
  // TODO: There is an arena code smell here. The `Model` interface needs to become lock-free
  // where Symbol tickets and &str are equally intuitive to use without runtime panics from
  // arena mutability exceptions.
  if let Some(final_tag) = allowed_candidates.first() {
    // The context can hold this block — rename the capture in place.
    document.rename_node(container, &to_string(*final_tag), true)?;
  } else if let Some(&final_tag) = filtered_candidates.first() {
    // Surpass (OXIDIZED_DESIGN #240): the context (e.g. an already-open `<para>`)
    // cannot hold this Para.class block, but an ancestor can. Climb to the
    // nearest such ancestor and move the capture there — ending the paragraph —
    // instead of renaming in place, which yields a schema-invalid
    // `<para>/<logical-block>` (LaTeXML-para.rnc:16,56). Perl renames in place
    // (TeX_Box.pool.ltxml:512-513, in insertBlock :449-519), a SHARED schema bug;
    // we keep the block
    // rendering (a `<div>`, LaTeXML-para-xhtml.xsl:53) — the inline variant would
    // wrongly nest block content (a float/table) in a `<span>`. Witnesses:
    // tikz-network, numerica (framed `{shaded}` / `{minipage}` after inline text).
    // Only reached on the currently-invalid path (`allowed_candidates` empty in a
    // non-inline context); the inline/`<p>` path always has a valid candidate.
    // Restrict the climb to a `<para>` context — the "block box ends the open
    // paragraph" case (tikz-network/numerica). Other block-holding contexts (an
    // SVG `foreignObject` / box that wraps its block in place) keep the pre-#4
    // in-place rename, so a `\parbox` inside a picture stays in its foreignObject.
    // A `<para>` only ever lives inside a Para.model container (item, section
    // body, logical-block, …), so the immediate parent holds a `logical-block`
    // and the climb stops one level up. A `sectional-block` (Para.class content
    // carrying a `\paragraph`/`\subsection`) instead ends any enclosing section
    // and lands at body level — valid (document.body.class holds it), a net
    // improvement over Perl's invalid in-place nesting.
    if with(context_tag, |t| t == "ltx:para") {
      let mut child = context.clone();
      while let Some(parent) = child.get_parent() {
        if !matches!(parent.get_type(), Some(NodeType::ElementNode)) {
          break;
        }
        if document::can_contain_qsym(document::get_node_qname(&parent), final_tag) {
          container.unlink();
          child.add_next_sibling(&mut container)?;
          document.set_node(&parent);
          break;
        }
        child = parent;
      }
    } else if document::can_contain_qsym(context_tag, pin_static("ltx:item"))
      && document::can_contain_indirect(context_tag, final_tag).is_some()
    {
      // A list container (`itemize`/`enumerate`/`description`, model `item*`) holds
      // no block, so a box opened in a list before its first `\item`, or between
      // items, lands where the model has no room for it. LaTeX typesets such a box
      // as list material (framed.sty's `shaded` around an `\item`, colorframed-doc
      // :176; a `minipage` of `\item`s beside a table minipage, tableaux/exemples
      // :71). The only schema-valid home is an item, which is autoOpen
      // (latex_constructs.pool.ltxml:1277): place the capture through
      // `find_insertion_point`, which opens `item` → `para` from the list exactly
      // as it does for text arriving before the first `\item` (Perl's
      // `computeIndirectModel` route). Perl's `insertBlock` renames in place
      // (TeX_Box.pool.ltxml:512), a SHARED schema error; the next `\item` closes the
      // auto-opened item. Only where such a route exists: a `sectional-block` has
      // none (no item holds it) and keeps Perl's in-place rename. OXIDIZED_DESIGN
      // #261. Guard
      // `perfect_kernel_batch56::block_in_a_list_before_an_item_gets_an_auto_item`.
      container.unlink();
      document.set_node(&context);
      let mut point = document.find_insertion_point_qsym(final_tag, None)?;
      point.add_child(&mut container)?;
      document.set_node(&point);
    }
    // If no ancestor can hold it (rare — Para.class always fits the section/body
    // model), the capture stays where it is. Perl renames in place to the invalid
    // Para.class element (TeX_Box.pool.ltxml:512); instead, if the current context
    // holds `<block>` but not `final_tag`, emit a schema-valid `<block>` and recursively
    // demote the Para.model content (surpass #244) — a box-forming construct
    // (`\begin{center}`/minipage/`\parbox`) whose body auto-opened a `<para>` when a
    // genuine block sat mid-content, placed in a Block.model container (titlepage,
    // quote, figure, abstract, inline-block). Demotion is IN PLACE (no reorder; render:
    // ltx_para/block/logical-block all `display:block`, only a redundant `<para>`
    // grouping + its auto-`xml:id` drop). The #240 `ltx:para` climb above already moved
    // the capture to an ancestor that holds `final_tag`, so this branch is false there
    // (parent holds `final_tag`) and that path is untouched. Witnesses: webquiz,
    // tabularcalc, short-math-guide, heria (+ the `caption_in_inline_parbox` sibling).
    let demote_para = !is_inline
      && container.get_parent().is_some_and(|parent| {
        let pq = document::get_node_qname(&parent);
        !document::can_contain_qsym(pq, final_tag)
          && document::can_contain_qsym(pq, pin_static("ltx:block"))
      })
      // Only demote when the result is provably valid: a container holding a
      // Para.model-only element `<block>` can't take (a float, a TOC, a sectioning
      // unit) stays at Perl-parity rather than acquiring a fresh `malformed` error.
      && subtree_is_block_demotable(&container);
    // A box (`ltx:inline-block`: `\resizebox`, `\scalebox`, `\rotatebox`) holds none of the
    // block containers but does hold the inline ones: a `{minipage}` there whose content cannot be
    // demoted (a float — tabularray's tall table, 57cp review —, a theorem, a `\section*`) is the
    // first inline container its parent holds that holds all of it (`inline-logical-block` holds
    // floats, theorems and paragraphs; `inline-sectional-block` sections). Perl renames it in place to
    // its first block candidate, `ltx:logical-block` (TeX_Box.pool.ltxml:502-513), invalid in the box
    // (divergence #385). Guard `perfect_kernel_batch57::minipage_blocks_in_a_scaled_box_are_inline`.
    let inline_home = (!is_inline && !demote_para)
      .then(|| container.get_parent())
      .flatten()
      .and_then(|parent| {
        let pq = document::get_node_qname(&parent);
        if document::can_contain_qsym(pq, final_tag) {
          return None;
        }
        [
          "ltx:inline-block",
          "ltx:inline-logical-block",
          "ltx:inline-sectional-block",
        ]
        .map(pin_static)
        .into_iter()
        .find(|c| {
          document::can_contain_qsym(pq, *c)
            && node_tags
              .iter()
              .all(|t| document::sym_can_contain_somehow(*c, *t).is_some())
        })
      });
    if demote_para {
      // Bottom-up: demote the Para.class content while the container still allows it,
      // THEN rename the container to <block> — so nothing is ever moved into a <block>
      // while still Para.class (which would emit a spurious `malformed` Error).
      demote_para_class_content(document, &container)?;
      document.rename_node(container, "ltx:block", true)?;
    } else if let Some(inline) = inline_home {
      document.rename_node_qsym(container, inline, true)?;
    } else {
      document.rename_node(container, &to_string(final_tag), true)?;
    }
  } else {
    // we didn't know what to do?
    let message = with(context_tag, |ctxt_str| {
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

/// Would `cleanup_math`'s unwrap of a trivial (`XMText`/`XMHint`-only) `<Math>`
/// produce a schema-valid child of a `<MathFork>` main branch?
///
/// The `MathFork` content model is `(Math|text), MathBranch*`
/// (`LaTeXML/lib/LaTeXML/resources/RelaxNG/LaTeXML-block.rnc:109`): its main
/// branch admits exactly one leading `<Math>` or `<text>`. The unwrap is safe
/// there ONLY when it yields that single `<text>` — one `<XMText>` holding one
/// text run, with no `<XMHint>` spacing and no `<rule>`/`inline-block`. Anything
/// else (a `<rule>` cell, a second text run, an `inline-block`) would splice a
/// forbidden node straight under `<MathFork>`, so the caller keeps the `<Math>`
/// wrapper instead.
fn cleanup_math_unwrap_valid_under_mathfork(mathnode: &Node) -> bool {
  // <Math> holds exactly one <XMath>.
  let xmaths = mathnode.get_child_nodes();
  let [xmath] = xmaths.as_slice() else {
    return false;
  };
  // An EMPTY main branch (an entirely-empty aligned column, e.g. a leading `&&`
  // in alignat) unwraps to NOTHING — that splices no forbidden node, and
  // removing the empty <Math> lets the empty <MathFork> be pruned downstream
  // (Perl / pre-56eg behavior). So an empty main branch is SAFE to unwrap.
  // (Without this, batch 56eg kept the wrapper and a spurious
  // <MathFork><Math text="absent">…<td/><td/> survived — mathtools_test S12.E27.)
  let inner = xmath.get_child_nodes();
  if inner
    .iter()
    .all(|n| n.get_type() == Some(NodeType::CommentNode))
  {
    return true;
  }
  // Otherwise: <XMath> holds exactly one child, and it is an <XMText> (no XMHint
  // spacing, no multiple runs).
  let [xmtext] = inner.as_slice() else {
    return false;
  };
  if !document::with_node_qname(xmtext, |qname| qname == "ltx:XMText") {
    return false;
  }
  // The <XMText> holds exactly one non-comment child, text-like enough to
  // become a `<text>` (a text node, or an already-`<text>`/`<Math>` element) —
  // never a `<rule>`/`inline-block`.
  let kids: Vec<Node> = xmtext
    .get_child_nodes()
    .into_iter()
    .filter(|kid| kid.get_type() != Some(NodeType::CommentNode))
    .collect();
  let [kid] = kids.as_slice() else {
    return false;
  };
  match kid.get_type() {
    Some(NodeType::TextNode) => true,
    Some(NodeType::ElementNode) => {
      document::with_node_qname(kid, |qname| qname == "ltx:text" || qname == "ltx:Math")
    },
    _ => false,
  }
}

/// The schema's `Meta.class` (LaTeXML-meta.rnc:17): side content that math text
/// never admits (`XMText_model = (text | Inline.class | Misc.class)*`).
pub(crate) const META_CLASS: [&str; 7] = [
  "ltx:note",
  "ltx:indexmark",
  "ltx:glossarydefinition",
  "ltx:declare",
  "ltx:rdf",
  "ltx:resource",
  "ltx:navigation",
];

/// Float the `Meta.class` content of a Math's text out of the math (user ruling
/// 2026-09-23, OXIDIZED_DESIGN #272). `\footnote`/`\index`/`\gls` inside `\text{}`
/// construct in the `ltx:text` that `\text` opens, which admits a note, so their `^`
/// float stops there. `cleanup_xmtext` then unwraps that `ltx:text`, and the marker
/// is left as a direct `XMText` child: schema-invalid, with the footnote's text read
/// into the formula's `text=`. Perl gives the same tree (ribbonproofs, sidenotesplus,
/// ryethesis). Each outermost Meta element moves, in document order, to just after
/// the Math at the nearest ancestor level that admits it. That is where a bare
/// `\footnote` in math floats: the `p` for inline math, the `equation` for a display.
/// Nothing moves when no level admits it.
fn float_meta_out_of_math(document: &mut Document, mathnode: &Node) -> Result<()> {
  let is_meta = |n: &Node| with(document::get_node_qname(n), |t| META_CLASS.contains(&t));
  let movers: Vec<Node> = document
    .findnodes(
      "descendant::ltx:XMText//*[self::ltx:note or self::ltx:indexmark or \
       self::ltx:glossarydefinition or self::ltx:declare or self::ltx:rdf or \
       self::ltx:resource or self::ltx:navigation]",
      Some(mathnode),
    )
    .into_iter()
    .filter(|m| {
      // Outermost only: a Meta element inside another moves with it.
      let mut up = m.get_parent();
      while let Some(p) = up {
        if p == *mathnode {
          return true;
        }
        if is_meta(&p) {
          return false;
        }
        up = p.get_parent();
      }
      true
    })
    .collect();
  let mut last: Option<(Node, Node)> = None; // (anchor, last node placed after it)
  for mut m in movers {
    let qname = document::get_node_qname(&m);
    let mut anchor = mathnode.clone();
    let placed = loop {
      match anchor.get_parent() {
        Some(p) if p.get_type() == Some(NodeType::ElementNode) => {
          if with(qname, |t| document::can_contain(&p, t)) {
            break true;
          }
          anchor = p;
        },
        _ => break false,
      }
    };
    if !placed {
      continue;
    }
    let mut prev = match &last {
      Some((a, p)) if *a == anchor => p.clone(),
      _ => anchor.clone(),
    };
    let mut emptied = m.get_parent();
    // Elements only, so `add_next_sibling` never meets libxml2's text merge.
    m.unlink();
    prev.add_next_sibling(&mut m)?;
    last = Some((anchor, m));
    // A math-text wrapper the move leaves empty goes too: an empty `XMText` would
    // parse as a term of the formula (`a * [] * b`).
    while let Some(w) = emptied {
      let (is_text, is_xmtext) = with(document::get_node_qname(&w), |t| {
        (t == "ltx:text", t == "ltx:XMText")
      });
      let blank = w.get_child_nodes().iter().all(|c| match c.get_type() {
        Some(NodeType::TextNode) => c.get_content().trim().is_empty(),
        Some(NodeType::CommentNode) => true,
        _ => false,
      });
      // Never the insertion point: the Math's after_close restores it afterwards.
      if !(is_text || is_xmtext)
        || !blank
        || w.get_attribute_ns("id", XML_NS).is_some()
        || w == *document.get_node()
      {
        break;
      }
      emptied = if is_xmtext { None } else { w.get_parent() };
      document.remove_node(w);
    }
  }
  Ok(())
}

pub fn cleanup_math(document: &mut Document, mathnode: Node) -> Result<()> {
  float_meta_out_of_math(document, &mathnode)?;
  // Cleanup ltx:Math elements; particularly if they aren't "really" math.
  // But record the oddity with class=ltx_markedasmath

  // If the Math ONLY contains XMath/XMText and XMHint, it apparently isn't math at all!?!
  // Single token PUNCTs can also be taken out of math.
  let xpath = concat!(
    "ltx:XMath/ltx:*[local-name() != 'XMText' and local-name() != 'XMHint'",
    " and not(",
    "local-name() = 'XMTok' and (@role='PUNCT' or @role='PERIOD')",
    " and not(preceding-sibling::*) and not(following-sibling::*) )]"
  );
  if document.findnodes(xpath, Some(&mathnode)).is_empty() {
    // Surpass-Perl schema-validity guard (frege.tex witness; batch 56eg). Perl's
    // cleanup_Math unwraps unconditionally (TeX_Math.pool.ltxml:219) even when
    // the parent is a <MathFork> main branch, splicing a bare <rule>/<inline-block>
    // or a second child straight under <MathFork> — which the MathFork model
    // (Math|text),MathBranch* forbids (LaTeXML-block.rnc:109), yielding
    // schema-invalid XML in ~14 logic/proof manuals (frege, principia, natded, …).
    // When the unwrap here would NOT yield a single valid <text>, keep the <Math>
    // wrapper (the parser's own valid main-branch form) — the same handling the
    // "real math" else-branch below uses.
    if mathnode
      .get_parent()
      .is_some_and(|parent| document::with_node_qname(&parent, |qname| qname == "ltx:MathFork"))
      && !cleanup_math_unwrap_valid_under_mathfork(&mathnode)
    {
      cleanup_xmtext_outer(document, &mathnode)?;
      return Ok(());
    }
    // So unwrap down to the contents of the XMText's.
    let xmath_children: Vec<_> = mathnode
      .get_child_nodes()
      .into_iter()
      .flat_map(|child| child.get_child_nodes())
      .collect();
    let mut texts: Vec<Node> = vec![];
    for xmnode in xmath_children {
      let is_hint = document::with_node_qname(&xmnode, |qname| qname == "ltx:XMHint");
      if is_hint {
        // Convert XMHint width to spacing characters
        if let Some(width_str) = xmnode.get_attribute("width") {
          // Width may be a full glue spec like "2.22217pt plus 1.11108pt minus 2.22217pt"
          // Extract just the base dimension (before "plus" or "minus")
          let base_dim_str = width_str
            .split_once(" plus")
            .or_else(|| width_str.split_once(" minus"))
            .map_or(width_str.as_str(), |(base, _)| base);
          // Try parsing as Dimension (pt). If that fails, handle mu units
          // by converting mu→pt (1mu = font_size/18).
          let dim_opt = Dimension::from_str(base_dim_str).ok().or_else(|| {
            if base_dim_str.ends_with("mu") {
              let mu_str = base_dim_str.trim_end_matches("mu").trim();
              mu_str.parse::<f64>().ok().map(|mu_val| {
                let fs = lookup_font().and_then(|f| f.get_size()).unwrap_or(10.0);
                Dimension::from_str(&format!("{}pt", mu_val * fs / 18.0)).unwrap_or_default()
              })
            } else {
              None
            }
          });
          if let Some(dim) = dim_opt {
            // No font to thread: this runs over an already-built
            // `<ltx:XMHint>`, which records only `width` — the digest font is
            // not on the element. Keeps the ambient read (and with it the
            // WHEN-dependence described in tex_glue::dimension_to_spaces);
            // reachable only for math hints, which the streaming sweep has
            // never caught diverging.
            let spaces = super::tex_glue::dimension_to_spaces(dim, None);
            if !spaces.is_empty()
              && let Ok(text_node) = Node::new_text(&spaces, &document.document)
            {
              texts.push(text_node);
            }
          }
        }
      } else {
        // is XMText — process its children
        for mut child in xmnode.get_child_nodes() {
          let t = child.get_type();
          if t == Some(NodeType::CommentNode) {
            continue;
          }
          if t != Some(NodeType::ElementNode) {
            // Make sure we've got an element
            child = document.wrap_nodes("ltx:text", vec![child])?.unwrap();
            // Perl makes this `ltx:text` in the Math's place, from an array
            // (`appendTree`), so it records the box of the Math's parent, not
            // of the XMText it is wrapped in here (`replace_node_as_tree`).
            document.purge_node_boxes_rec(&child);
          }
          // Now record that it originally was marked as math
          document.add_class(&mut child, "ltx_markedasmath")?;
          texts.push(child);
        }
      }
    }
    // and replace the whole Math with the pieces — Perl `replaceTree`, whose
    // boxes follow the pieces into an auto-opened parent.
    document.replace_node_as_tree(mathnode.clone(), texts)?;
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
      // Copy the child's attributes (should Merge!!) — through the schema-gated
      // setter: a collapsed `\rotatebox`/`\raisebox` inline-block hands over
      // `angle`/`innerdepth`/`innerwidth`/`xtranslate`…, none of which
      // `XMText_attributes` (LaTeXML-math.rnc:223) admits. Perl's own copy here
      // (TeX_Math.pool.ltxml:260) is raw too, yet its OUTPUT keeps only the
      // model's attributes (`depth height width yoffset rpadding …`; verified on
      // the forced-unparsed witness) — this setter reproduces that output. The
      // raw copy showed only when the math failed to parse (`ltx_math_unparsed`
      // keeps this XMText; a parse rebuilds it): principia's `\pmcexists`, six
      // schema errors on one XMText, Perl clean (batch 56fy). `xml:id` arrives
      // from rust-libxml under its LOCAL name `id`; Perl skips it (`:260`).
      if key != "id" && key != "xml:id" {
        document.set_attribute(&mut text_node, &key, &value)?;
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
    // Perl TeX_Math.pool.ltxml L281-310: unwrap tbody, rename
    // tabular→XMArray / tr→XMRow / td→XMCell, within each cell unwrap any
    // Math (pull XMath contents up) or wrap plain content in XMText,
    // propagate XMText attributes up to the table, then unwrap the XMText.
    // First: remove any ltx:tbody wrapping.
    for tb in document.findnodes("ltx:tabular/ltx:tbody", Some(&text_node)) {
      document.unwrap_nodes(tb)?;
    }
    // Rename tabular → XMArray
    let first_child = children.first().cloned().unwrap();
    let mut table = document.rename_node(first_child, "ltx:XMArray", false)?;
    let rows: Vec<Node> = table
      .get_child_nodes()
      .into_iter()
      .filter(|n| n.get_type() == Some(NodeType::ElementNode))
      .collect();
    for row in rows {
      let row = document.rename_node(row, "ltx:XMRow", false)?;
      let cells: Vec<Node> = row
        .get_child_nodes()
        .into_iter()
        .filter(|n| n.get_type() == Some(NodeType::ElementNode))
        .collect();
      for cell in cells {
        let cell = document.rename_node(cell, "ltx:XMCell", false)?;
        // Perl iterates `$cell->childNodes` — TEXT nodes included — and wraps every
        // non-Math child in XMText. A cell that escaped the `${}##{}$` template
        // (`\multispan{n}\upbracefill`, oubraces.sty:40-41; halloweenmath's arrows,
        // nath's braces) holds its glyph as a bare text node, which an
        // elements-only filter left as raw #PCDATA in the XMCell — schema-invalid
        // (`text not allowed here`) where Perl emits `<XMText>⏟</XMText>`. Text nodes
        // are taken as Perl takes them (a whitespace-only cell becomes
        // `<XMText> </XMText>` there too); only comments are skipped (batch 56ez).
        let cell_kids: Vec<Node> = cell
          .get_child_nodes()
          .into_iter()
          .filter(|n| {
            matches!(
              n.get_type(),
              Some(NodeType::ElementNode | NodeType::TextNode)
            )
          })
          .collect();
        for m in cell_kids {
          if model::with_node_qname(&m, |qn| qn == "ltx:Math") {
            // Perl: replaceNode($m, map { $_->childNodes } $m->childNodes)
            //  — Math wraps an XMath, XMath wraps the actual tokens. Pull
            //  those up, discarding the Math/XMath layers.
            let grandkids: Vec<Node> = m
              .get_child_nodes()
              .into_iter()
              .flat_map(|x| x.get_child_nodes())
              .collect();
            document.replace_node(m, grandkids)?;
          } else {
            document.wrap_nodes("ltx:XMText", vec![m])?;
          }
        }
      }
    }
    // Copy all of XMText's attributes (incl. xml:id) onto the table.
    let id_opt = text_node.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace");
    for (key, value) in text_node.get_attributes() {
      table.set_attribute(&key, &value)?;
    }
    // Unwrap the XMText (its only child is now `table`).
    document.unwrap_nodes(text_node)?;
    if let Some(id) = id_opt {
      document.unrecord_id(&id);
      // Re-record the id on the renamed table (and any nested ids).
      document.record_node_ids(&table)?;
    }
  }
  Ok(())
}

//======================================================================
// A random collection of utility functions.
// [maybe need to do some reorganization?]
// Since this is used for textual tokens, typically to split author lists,
// we don't split within braces or math

/// A `SplitTokens` delimiter: Perl PR #2767 allows each delimiter to be a
/// single `Token`, OR a `Tokens` sequence to match in order.
#[derive(Debug, Clone)]
pub enum SplitDelim {
  /// A single Token delimiter (matched with the meaning-aware Equals below)
  Token(Token),
  /// A token sequence delimiter (matched literally, with the
  /// `T_SPACE` ~ `\ ` "HACK space" equivalence)
  Tokens(Tokens),
}
impl From<Token> for SplitDelim {
  fn from(t: Token) -> Self { SplitDelim::Token(t) }
}
impl From<Tokens> for SplitDelim {
  fn from(t: Tokens) -> Self { SplitDelim::Tokens(t) }
}

/// Does the remaining `stream` contain a `)` that balances the `(` just
/// popped? Used by `split_tokens` to protect delimiters inside *balanced*
/// parens only — an unbalanced `(` must NOT trigger paren-protection (else it
/// would greedily swallow the rest of the author block, including `\\`
/// name/affiliation separators).
fn paren_closes_ahead(stream: &VecDeque<Token>) -> bool {
  let mut level = 1usize;
  for t in stream {
    if *t == T_OTHER!("(") {
      level += 1;
    } else if *t == T_OTHER!(")") {
      level -= 1;
      if level == 0 {
        return true;
      }
    }
  }
  false
}

/// Perl: SplitTokens($tokens, @delims) — Base_Utility.pool.ltxml (PR #2767).
/// Each of `delims` is a Token, OR Tokens to match a sequence.
/// Returns a list of Tokens for the sub-sequences, with leading/trailing
/// spaces trimmed from each piece, and any empty trailing piece dropped.
///
/// Like Perl, delimiters are not matched inside `{…}` braces or `$…$` math.
/// As an INTENTIONAL DIVERGENCE FROM PERL, they are also not matched inside
/// balanced `(…)` parentheses (see the paren branch below + OXIDIZED_DESIGN).
pub fn split_tokens(tokens: Tokens, delims: Vec<SplitDelim>) -> Vec<Tokens> {
  split_tokens_delimited(tokens, delims)
    .into_iter()
    .map(|(_, item)| item)
    .collect()
}

/// [`split_tokens`], each piece paired with the delimiter tokens that preceded it (none for the first), so a caller
/// that joins pieces back can put the delimiter where it stood.
pub fn split_tokens_delimited(
  tokens: Tokens,
  delims: Vec<SplitDelim>,
) -> Vec<(Vec<Token>, Tokens)> {
  let mut items: Vec<(Vec<Token>, Tokens)> = Vec::new();
  let mut before: Vec<Token> = Vec::new();
  let mut toks: Vec<Token> = Vec::new();
  let trim_spaces = |toks: &mut Vec<Token>| {
    while toks.first().is_some_and(|x| *x == T_SPACE!()) {
      toks.remove(0);
    }
    while toks.last().is_some_and(|x| *x == T_SPACE!()) {
      toks.pop();
    }
  };
  if !tokens.is_empty() {
    let mut stream: VecDeque<Token> = VecDeque::from(tokens.unlist());
    while let Some(t) = stream.pop_front() {
      let mut matched = false;
      let mut delimiter: Vec<Token> = Vec::new();
      for delim in &delims {
        match delim {
          SplitDelim::Token(d) => {
            // Perl: Equals($t, $delim); the Rust port additionally matches an alias of
            // the delimiter's own definition, so \AND (let to \and) matches \and. Only
            // its own: a delimiter `\let` to another command (`\and` disabled to `\relax`
            // by a `\maketitle`) would split at every alias of that command — `\protect`,
            // a disabled `\thanks` — in a later `\author` (59e; DIVERGENCES #36).
            if *d == t
              || (t.get_catcode() == Catcode::CS && d.get_catcode() == Catcode::CS && {
                let meaning_t = lookup_definition(&t).ok().flatten();
                let meaning_d = lookup_definition(d).ok().flatten();
                meaning_t.is_some()
                  && meaning_t == meaning_d
                  && meaning_d.is_some_and(|own| *own.get_cs() == *d)
              })
            {
              matched = true;
              delimiter = vec![t];
              break;
            }
          },
          SplitDelim::Tokens(seq) => {
            let mut tomatch: &[Token] = seq.unlist_ref();
            let mut peeked: Vec<Token> = Vec::new(); // tokens consumed beyond `t`
            let mut cur: Option<Token> = Some(t);
            while let (Some(c), Some(m)) = (cur, tomatch.first()) {
              // HACK space! A control space or thin space stands for the space of a space-bearing delimiter
              // (`Aghil Alaee\footnote{…} \,\,and Hari K. Kunduri`, 1407.0988: " and ")
              if c == *m || (*m == T_SPACE!() && (c == T_CS!("\\ ") || c == T_CS!("\\,"))) {
                tomatch = &tomatch[1..];
                if !tomatch.is_empty() {
                  cur = stream.pop_front();
                  if let Some(p) = cur {
                    peeked.push(p);
                  }
                }
              } else {
                break;
              }
            }
            if tomatch.is_empty() {
              matched = true;
              delimiter = std::iter::once(t).chain(peeked).collect();
              break;
            } else {
              // failed to match all: put back the peeked tokens
              for p in peeked.into_iter().rev() {
                stream.push_front(p);
              }
            }
          },
        }
      }
      if matched {
        trim_spaces(&mut toks);
        items.push((
          std::mem::replace(&mut before, delimiter),
          Tokens::new(std::mem::take(&mut toks)),
        ));
      } else if t.defined_as(&T_BEGIN!()) {
        toks.push(t);
        let mut level = 1;
        while let Some(t) = stream.pop_front() {
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
      } else if t.defined_as(&T_MATH!()) {
        // The span closes on a math shift as it opened, by meaning: an active `$` let to it (aastex's tables,
        // aas_support_sty.rs; witness 2609.05675) closes what it opens. Perl closes on any catcode-3 token
        // (Base_Utility.pool.ltxml:160-165), OXIDIZED_DESIGN_DIVERGENCES #450.
        toks.push(t);
        while let Some(t) = stream.pop_front() {
          let is_math = t.defined_as(&T_MATH!());
          toks.push(t);
          if is_math {
            break;
          }
        }
      } else if t == T_CS!("\\begin")
        && let Some(span) = environment_closes_ahead(&stream)
      {
        // INTENTIONAL DIVERGENCE FROM PERL (as for the parentheses below): an unbraced `\begin{name} ... \end{name}`
        // in an author block is one unit — a `{center}` affiliation under the names, a `{tabular}` holding
        // `COSIC, KU Leuven` — not split at its inner `and`/`,`/`\\`, which left the environment's ends in different
        // `\lx@personname`s (Perl's SplitTokens protects only braces and math, Base_Utility.pool.ltxml:152-165;
        // 2609.01563, 17357, 33831). Only when the matching `\end{name}` follows: an unbalanced `\begin` stays a token.
        toks.push(t);
        toks.extend(stream.drain(..span));
      } else if t == T_CS!("\\textsuperscript")
        && stream
          .front()
          .is_some_and(|next| !matches!(next.get_catcode(), Catcode::BEGIN | Catcode::SPACE))
      {
        // A superscript's one-token argument stays with it (`\textsuperscript*`); when that argument is a delimiter
        // (`Bob Baker\textsuperscript, Carl Cole`, the separator typeset raised; 2609.15009) the split is there, and the
        // superscript, with nothing left to raise, goes — it took the piece's closing brace as its argument.
        let next = *stream.front().unwrap();
        let splits_here = delims
          .iter()
          .any(|delim| matches!(delim, SplitDelim::Token(d) if *d == next));
        if !splits_here {
          toks.push(t);
          toks.extend(stream.pop_front());
        }
      } else if t == T_OTHER!("(") && paren_closes_ahead(&stream) {
        // INTENTIONAL DIVERGENCE FROM PERL: also protect delimiters inside
        // BALANCED parentheses (mirroring the brace/math skipping above), so a
        // parenthesized affiliation like "(Scuola Normale Superiore, Pisa)" is
        // NOT split at its internal comma into a spurious second author. Perl's
        // SplitTokens has no paren-awareness and makes exactly this mistake
        // (witness arXiv 0804.0870 — "Pisa)" became a second <personname>).
        // The `paren_closes_ahead` guard means an UNBALANCED `(` (rare/
        // malformed) is treated as an ordinary token, so it never swallows a
        // later `\\` name/affiliation separator. See OXIDIZED_DESIGN
        // "Intentional divergences". NOTE: bare (unparenthesized) commas in an
        // affiliation ("MIT, Cambridge") remain genuinely ambiguous — the same
        // tokens read as either one comma-affiliation or two authors — so we
        // match Perl's recall-oriented over-split there rather than guess.
        toks.push(t);
        let mut level = 1;
        while let Some(t) = stream.pop_front() {
          if t == T_OTHER!("(") {
            level += 1;
          } else if t == T_OTHER!(")") {
            level -= 1;
          }
          toks.push(t);
          if level < 1 {
            break;
          }
        }
      } else {
        toks.push(t);
      }
    }
  }
  trim_spaces(&mut toks);
  if !toks.is_empty() {
    items.push((before, Tokens::new(toks)));
  }
  items
}

/// After a `\begin`: the length of `{name} ... \end{name}` at the front of `stream`, through the matching `\end{name}`
/// (nested environments of the same name counted), or None when it does not close.
fn environment_closes_ahead(stream: &VecDeque<Token>) -> Option<usize> {
  let name_at = |i: usize| -> Option<(String, usize)> {
    if stream.get(i)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let close = (i + 1..stream.len()).find(|&k| stream[k].get_catcode() == Catcode::END)?;
    let name: String = (i + 1..close).map(|k| stream[k].to_string()).collect();
    Some((name, close + 1))
  };
  let (name, mut i) = name_at(0)?;
  let mut depth = 1;
  while i < stream.len() {
    let t = stream[i];
    if (t == T_CS!("\\begin") || t == T_CS!("\\end"))
      && let Some((other, next)) = name_at(i + 1)
    {
      if other == name {
        depth += if t == T_CS!("\\begin") { 1 } else { -1 };
        if depth == 0 {
          return Some(next);
        }
      }
      i = next;
    } else {
      i += 1;
    }
  }
  None
}

/// If `tokens` — ignoring leading/trailing spaces — is exactly a single control
/// sequence applied to one brace group that spans to the end (`\cmd{ … }`),
/// return `(\cmd, inner)`. Used to see through a whole-line font wrapper when
/// splitting an author line.
fn whole_line_cs_wrapper(tokens: &Tokens) -> Option<(Token, Tokens)> {
  let v = tokens.unlist_ref();
  let mut start = 0;
  let mut end = v.len();
  while start < end && v[start] == T_SPACE!() {
    start += 1;
  }
  while end > start && v[end - 1] == T_SPACE!() {
    end -= 1;
  }
  let s = &v[start..end];
  // Need at least `\cmd { }` (the group may be empty, but then there is nothing
  // to split, so the caller's `len > 1` guard makes it a no-op anyway).
  if s.len() < 3 || s[0].get_catcode() != Catcode::CS || s[1].get_catcode() != Catcode::BEGIN {
    return None;
  }
  let mut level = 0;
  for (i, t) in s.iter().enumerate().skip(1) {
    match t.get_catcode() {
      Catcode::BEGIN => level += 1,
      Catcode::END => {
        level -= 1;
        if level == 0 {
          // The group must close exactly on the last token for this to be a
          // whole-line wrapper (no trailing content after `}`).
          return (i == s.len() - 1).then(|| (s[0], Tokens::new(s[2..i].to_vec())));
        }
      },
      _ => {},
    }
  }
  None
}

/// Second-level split of the author heuristic: divide ONE line that has already
/// been classified as an *author* line (its superscript marker sits far enough
/// from the front — `Some(p)` with `p >= 8` in `\lx@add@authors`) into the
/// individual creators it names.
///
/// This is guarded for affiliations by that upstream classification: an
/// *affiliation* line (marker near the front, `p < 8`) or a continuation line
/// (no marker) never reaches here — they take the other arms — so name-level
/// separators are only ever applied to author text, never to institution names.
/// That is why the literal word " and " is a separator here (it splits "Alice
/// and Bob") yet is deliberately absent from the line-level `author_affil_splits`
/// (where it would shred "Language and Intelligence").
///
/// The split proceeds by hierarchy:
///  1. the name separators ("," and " and ", " \& ") at top level; if that already
///     yields more than one name we are done;
///  2. otherwise, if the whole line is a single font wrapper `\cmd{ a, b, c }`
///     (Perl's `SplitTokens` can't see the brace-hidden separators, so the
///     wrapper collapses into ONE creator that then hoards every `$^n$` marker
///     as a duplicate affiliation — arXiv 2605.00347), descend through the
///     wrapper, split its inner name list, and re-apply `\cmd` to each name so
///     every author becomes its own creator with the correct affiliation;
///  3. otherwise, if the line is one group opening with declarations whose content reads as names (`{\bf A, B}`,
///     hep-th9212083), the same with the group and its declarations.
///
/// A qualifier (`Jr.`, `PhD`, `\emph{Member, IEEE}`) or a piece of marks and notes only then rejoins the name before
/// it.
///
/// Public so a class binding whose `\authors{...}` is a comma/"and"-separated
/// name list (e.g. AGU's `agujournal2019`) reuses this canonical splitter
/// instead of re-listing the separators — the same split `\lx@add@authors`
/// applies to a superscript-marked author line.
pub fn split_author_line(line: Tokens) -> Vec<Tokens> {
  rejoin_name_qualifiers(split_author_names(line))
}

/// [`split_author_line`] for a binding whose `\author` Perl reads as one author (aastex, amsart, revtex, an IEEE
/// block, a labelled authblk `\author`): its names line splits only where it reads as two or more names
/// ([`name_count`]: `Klemens Fellner and Bao Quoc Tang`), and otherwise stays one author as written (`John Smith,
/// Max Planck Institute for Mathematics`; a name the shape test cannot read).
fn split_author_line_cautious(line: Tokens) -> Vec<Tokens> {
  if name_count(&visible_name_text(line.unlist_ref())) >= 2 {
    split_author_line(line)
  } else {
    vec![line]
  }
}

/// [`split_author_line`] before a qualifier rejoins its name: each piece with the separator written before it (none
/// for the first). An empty piece — between ", " and " and " in "A, B, and C" — names no one and is dropped, before a
/// wrapper is put back on the pieces: kept, it became an empty creator that took a copy of every affiliation given to
/// all the authors (`annotate=new`) and merged it onto B.
fn split_author_names(line: Tokens) -> Vec<(Vec<Token>, Tokens)> {
  // Name-level separators: comma, the literal word " and " ("Alice and Bob") and " \& " (`Zheng Zheng \& Jordi
  // Miralda-Escud\'e`, astro-ph0201275; 0908.0757).
  let split = |tokens: Tokens| -> Vec<(Vec<Token>, Tokens)> {
    let mut pieces: Vec<(Vec<Token>, Tokens)> = Vec::new();
    // the separators of a dropped empty piece go on to the next (", and")
    let mut carried: Vec<Token> = Vec::new();
    for (delimiter, piece) in split_tokens_delimited(tokens, vec![
      SplitDelim::Token(T_OTHER!(",")),
      literal_and(),
      literal_and_tie(),
      literal_ampersand(),
    ]) {
      carried.extend(delimiter);
      if !piece.is_empty() {
        pieces.push((std::mem::take(&mut carried), piece));
      }
    }
    pieces
  };
  let line = marks_before_glued_commas(line);
  let top = split(line.clone());
  if top.len() > 1 {
    return top;
  }
  if let Some((cmd, inner)) = whole_line_cs_wrapper(&line) {
    // the marks after commas inside the wrapper are its names' too (`\textbf{Todd M. Tripp,\altaffilmark{2} …}`)
    let inner_split = split(marks_before_glued_commas(inner));
    if inner_split.len() > 1 {
      return inner_split
        .into_iter()
        .map(|(delimiter, piece)| {
          let mut v = vec![cmd, T_BEGIN!()];
          v.extend(piece.unlist());
          v.push(T_END!());
          (delimiter, Tokens::new(v))
        })
        .collect();
    }
  }
  // A line that is one group opening with declarations (`{\bf Daniel Boyanovsky, Da- Shin Lee}`, hep-th9212083;
  // `{\bf Jianfei Ma$^{2}$, \enspace Emmanuele Chersoni$^{2}$, …}`, 2608.16650) is the same when what it holds is
  // names: each name in the group and its declarations. Braces alone are kept: they hold one name together
  // (`\author{{Smith, Jr., John}}`).
  if let Some((declarations, inner)) = whole_line_group(&line)
    && declarations.iter().any(|t| *t != T_SPACE!())
    && name_count(&visible_name_text(inner.unlist_ref())) >= 2
  {
    let inner_split = split(marks_before_glued_commas(inner));
    if inner_split.len() > 1 {
      return inner_split
        .into_iter()
        .map(|(delimiter, piece)| {
          let mut v = vec![T_BEGIN!()];
          v.extend(declarations.iter().copied());
          v.extend(piece.unlist());
          v.push(T_END!());
          (delimiter, Tokens::new(v))
        })
        .collect();
    }
  }
  top
}

/// If `tokens` (spaces around it aside) is one brace group, its opening declarations ([`opening_declarations`]:
/// `\bf`, `\small`) and the rest of its content.
fn whole_line_group(tokens: &Tokens) -> Option<(Vec<Token>, Tokens)> {
  let v = tokens.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!())?;
  let end = v.iter().rposition(|t| *t != T_SPACE!())?;
  if end <= start || v[start].get_catcode() != Catcode::BEGIN || skip_group(v, start) != end + 1 {
    return None;
  }
  let inner = Tokens::new(v[start + 1..end].to_vec());
  let declarations = opening_declarations(&inner);
  let rest = Tokens::new(inner.unlist_ref()[declarations.len()..].to_vec());
  Some((declarations, rest))
}

/// In a list of names whose marks follow them, a mark glued to the comma after a name is that name's: `Ann
/// Able,\textsuperscript{a} Bob Baker\textsuperscript{b}` gives Ann `a` (2609.25924; repro
/// sectioning-frontmatter/author_suffix_marks_after_commas_link_affiliations; Perl gives Bob both). The marks move in
/// front of the comma before the line is split, glued commas between marks merging (`Ann$^1$,$^2$,$^3$, Bob` is one
/// name with three marks). Not across a space (`Ann$^1$, $^{*}$Bob` keeps Bob's star), and not when the names open
/// with their marks (`marker_leads`: `\large $^1$Ann Able, $^2$Bob Baker`). A `\thanks` glued among them goes too
/// (`Kerstin Hötte,$^{1}$\thanks{Corresponding author: kerstin.hotte@...}`, 2609.37343).
fn marks_before_glued_commas(line: Tokens) -> Tokens {
  if marker_leads(&line) {
    return line;
  }
  let src = line.unlist();
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut pending_comma: Option<Token> = None;
  let mut depth = 0i32;
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && t == T_OTHER!(",") {
      let end = leading_marks_end(&src[i + 1..]);
      if end > 0 {
        out.extend_from_slice(&src[i + 1..i + 1 + end]);
        pending_comma = Some(t);
        i += 1 + end;
        continue;
      }
      // a glued comma before this one merges into it: commas matter here only as the points the line splits at
      // (the unsplit `whole_line_cs_wrapper` fallback shows one comma for the two)
      pending_comma = None;
      out.push(t);
      i += 1;
      continue;
    }
    if let Some(comma) = pending_comma.take() {
      out.push(comma);
    }
    out.push(t);
    i += 1;
  }
  if let Some(comma) = pending_comma {
    out.push(comma);
  }
  Tokens::new(out)
}

/// The end of the marks a name's tokens open with, with no space before them: `\textsuperscript{a}`, `$^{a}$`, the
/// shown symbol mark `\lx@frontmatter@keepsup{*}\lx@frontmatter@symbolmark{*}`, and a `\thanks{...}` or
/// `\footnote{...}`, whose mark prints with them (0 when none).
fn leading_marks_end(tokens: &[Token]) -> usize {
  let mut i = 0;
  while let Some(&first) = tokens.get(i) {
    let next = if first == T_CS!("\\textsuperscript")
      || first == T_CS!("\\lx@frontmatter@keepsup")
      || first == T_CS!("\\lx@frontmatter@symbolmark")
      // aastex's mark (aas_support: an affiliation request), written after the comma as often as before it
      // (`Todd M. Tripp,\altaffilmark{2,3} Bart P. Wakker,\altaffilmark{4}`, astro-ph0302534)
      || first == T_CS!("\\altaffilmark")
    {
      sup_operand_at(tokens, i + 1).map(|(_, next)| next)
    } else if (first == T_CS!("\\thanks") || first == T_CS!("\\footnote"))
      && tokens
        .get(i + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      sup_operand_at(tokens, i + 1).map(|(_, next)| next)
    } else if first == T_MATH!() && tokens.get(i + 1) == Some(&T_SUPER!()) {
      sup_operand_at(tokens, i + 2)
        .filter(|(_, next)| tokens.get(*next) == Some(&T_MATH!()))
        .map(|(_, next)| next + 1)
    } else {
      None
    };
    match next {
      Some(next) => i = next,
      None => break,
    }
  }
  i
}

pub fn and_split(cs: Token, tokens: Tokens) -> Vec<Token> {
  // Perl: SplitTokens($tokens, T_CS('\and'))
  // Only split on \and. The meaning-based check in split_tokens also matches
  // \AND (which is Let to \and). \And is NOT split here — amsmath overrides
  // its definition with DefMath, so it stays as a text "&" separator inside
  // <personname>, matching Perl's behavior.
  split_tokens(tokens, vec![SplitDelim::Token(T_CS!("\\and"))])
    .into_iter()
    .flat_map(|t| {
      let mut with_cs = vec![cs, T_BEGIN!()];
      with_cs.extend(t.unlist());
      with_cs.push(T_END!());
      with_cs
    })
    .collect()
}

/// Whether an author block is an IJCAI sectioned one: `\affiliations` or `\emails` at its top level (the split's
/// `Until:` delimiters match nothing deeper), each undefined, `\relax` or a no-op there — the ijcai binding's
/// (`ijcai_sty.rs`), or a raw derivative's that defines them only in `\@maketitle` (ttm.sty).
fn has_ijcai_section_marker(stuff: &Tokens) -> Result<bool> {
  let markers = [T_CS!("\\affiliations"), T_CS!("\\emails")];
  let mut depth = 0i32;
  for token in stuff.unlist_ref() {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ if depth == 0 && markers.contains(token) => {
        let separator = token.defined_as(&TOKEN_RELAX)
          || match lookup_definition_stored(token)? {
            None => true,
            Some(Stored::Expandable(e)) => {
              e.paramlist
                .as_ref()
                .is_none_or(|p| p.get_parameters().is_empty())
                && matches!(&e.expansion, Some(ExpansionBody::Tokens(t)) if t.is_empty())
            },
            Some(_) => false,
          };
        if separator {
          return Ok(true);
        }
      },
      _ => {},
    }
  }
  Ok(false)
}

/// Perl: positionOf($tokens, @delims) — Base_Utility.pool.ltxml (PR #2767).
/// Find the position of a Token from `delims` within `tokens`.
/// 1 based, so None == token not present.
pub fn position_of(tokens: &Tokens, delims: &[Token]) -> Option<usize> {
  for (i, t) in tokens.unlist_ref().iter().enumerate() {
    if delims.contains(t) {
      return Some(i + 1);
    }
  }
  None
}

/// " and~", the "and" tied to the name after it (`Luca~Varotto, Angelo~Cenedese, and~Andrea~Cavallaro`, IEEE
/// 2011.10474, 2408.09035; svjour3 1406.6147). Unsplit, the "and" was read as the first word of the last name (63j).
fn literal_and_tie() -> SplitDelim {
  let mut tks = vec![T_SPACE!()];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(T_ACTIVE!('~'));
  SplitDelim::Tokens(Tokens::new(tks))
}
/// " \& " between two names, its spaces kept.
fn literal_ampersand() -> SplitDelim {
  SplitDelim::Tokens(Tokens::new(vec![T_SPACE!(), T_CS!("\\&"), T_SPACE!()]))
}
// Things to split authors (Perl PR #2767, Base_Utility.pool.ltxml)
// This is " and " without the spaces stripped.
fn literal_and() -> SplitDelim {
  let mut tks = vec![T_SPACE!()];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(T_SPACE!());
  SplitDelim::Tokens(Tokens::new(tks))
}
/// An alignment environment (`center`, `flushleft`, `flushright`) that wraps a whole author block, or a whole `\and`
/// group, is layout around the lines the splitter reads, not one unit of them: its content, so its names, `\\`
/// lines and marks split as written (`\author{\begin{center}Ann Able$^{1}$, Bob Baker$^{2}$ \\ $^1$Univ A
/// ...\end{center}}`). One inside a piece — a `{center}` affiliation under the names — stays whole
/// (`split_tokens_delimited`; 2609.01563, 17357). Nested wrappers unwrap in turn.
fn unwrap_alignment_environment(tokens: Tokens) -> Tokens {
  let mut toks = tokens.unlist();
  loop {
    let start = toks.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = toks.len() - toks.iter().rev().take_while(|t| **t == T_SPACE!()).count();
    if start >= end || toks[start] != T_CS!("\\begin") {
      break;
    }
    let stream: VecDeque<Token> = toks[start + 1..end].iter().copied().collect();
    let Some(span) = environment_closes_ahead(&stream) else {
      break;
    };
    let name_end = (start + 2..end)
      .find(|&k| toks[k].get_catcode() == Catcode::END)
      .unwrap_or(start + 1);
    let name: String = toks[start + 2..name_end]
      .iter()
      .map(|t| t.to_string())
      .collect();
    // The environment must end where the tokens do, its `\end{name}` the last thing.
    if start + 1 + span != end || !matches!(name.as_str(), "center" | "flushleft" | "flushright") {
      break;
    }
    let end_cs = (name_end + 1..end)
      .rev()
      .find(|&k| toks[k] == T_CS!("\\end"))
      .unwrap_or(end);
    toks = toks[name_end + 1..end_cs].to_vec();
  }
  Tokens::new(toks)
}

/// The ACM SIG classes' author columns (DIVERGENCES #445), read through their own commands: the brace group that
/// follows `\alignauthor` and is the whole column — sigchi.cls writes each column as one, `\alignauthor{Name\\ \affaddr{…}\\ \email{…}}` — is
/// its content, so the column's `\\` lines split (1906.01122); and an `\affaddr{…}` is its content, so a mark that
/// leads it (`\affaddr{\textsuperscript{1} Univ…}`, 1608.06253) leads the affiliation line, as Perl's OmniBus reads
/// `\affaddr` as `\address` (OmniBus.cls.ltxml:92). Only at depth 0 and only after these two commands, which no
/// TeX Live file defines: a brace group anywhere else (`\author{{Smith, Jr., John}}`) keeps its braces.
fn unbrace_acm_author_columns(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  if !toks
    .iter()
    .any(|t| *t == T_CS!("\\alignauthor") || *t == T_CS!("\\affaddr"))
  {
    return Tokens::new(toks);
  }
  let mut out = Vec::with_capacity(toks.len());
  let mut depth = 0usize;
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    i += 1;
    let column = t == T_CS!("\\alignauthor");
    if depth == 0 && (column || t == T_CS!("\\affaddr")) {
      if column {
        out.push(t);
      }
      let mut k = i;
      while k < toks.len() && toks[k].get_catcode() == Catcode::SPACE {
        k += 1;
      }
      if k < toks.len() && toks[k].get_catcode() == Catcode::BEGIN {
        let mut level = 0usize;
        let mut close = None;
        for (n, u) in toks[k..].iter().enumerate() {
          match u.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => {
              level -= 1;
              if level == 0 {
                close = Some(k + n);
                break;
              }
            },
            _ => {},
          }
        }
        // A column's group is its whole content only when nothing but spaces and `\\` follow it before the next
        // author: `\alignauthor {\large Ann}\\ \affaddr{…}` is a name's font group, kept.
        let whole = |close: usize| {
          let mut n = close + 1;
          while n < toks.len()
            && (toks[n].get_catcode() == Catcode::SPACE || toks[n] == T_CS!("\\\\"))
          {
            n += 1;
          }
          n == toks.len()
            || [
              T_CS!("\\alignauthor"),
              T_CS!("\\and"),
              T_CS!("\\And"),
              T_CS!("\\AND"),
            ]
            .contains(&toks[n])
        };
        if let Some(close) = close
          && (!column || whole(close))
        {
          // The column's own `\affaddr`s are read the same way.
          out.extend(unbrace_acm_author_columns(Tokens::new(toks[k + 1..close].to_vec())).unlist());
          i = close + 1;
          continue;
        }
      }
      if !column {
        out.push(t);
      }
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    out.push(t);
  }
  Tokens::new(out)
}

// GROUP-level separators for the no-marker author heuristic (OXIDIZED_DESIGN
// #52): the \and family plus \quad/\qquad, but NOT the comma and NOT the literal
// " and ". A comma separates NAMES within one author line ("Alice, Bob") and the
// tokens WITHIN a multi-part address ("Laurel, MD 20723") — so splitting author
// *groups* on it shreds addresses into fake authors and turns the following
// \email line into a bogus affiliation (witness arXiv:2606.00315, NeurIPS
// idiom). Perl's @authorsplits (Base_Utility.pool.ltxml L679) includes the comma
// and shares this mis-split; excluding it here is a deliberate surpass-Perl
// divergence. Name-level comma/" and " splitting still happens, but only AFTER a
// line is known to be author names — via split_author_line — never across
// address text. (Compare author_affil_splits(): "NO comma in affiliations!!!".)
fn author_group_splits() -> Vec<SplitDelim> {
  let mut splits = author_and_splits();
  splits.extend([T_CS!("\\quad").into(), T_CS!("\\qquad").into()]);
  splits
}

/// The `\and` family only — the HARD author boundary the superscript-marker
/// branch groups on FIRST, so a marker-less line never merges into an author
/// from a previous `\and` group (html_feedback#1021 F2; OXIDIZED_DESIGN #52(g)).
/// With it the ACM SIG classes' `\alignauthor`, which opens each author's column (sig-alternate.cls, sigchi.cls:
/// `\author{\alignauthor Name\\ \affaddr{…}\\ \email{…} \alignauthor …}`): split here, its raw definition — a
/// tabular juggle for the class's title page — never runs inside a name. Perl's OmniBus makes it empty
/// (OmniBus.cls.ltxml:76) and its `@authorsplits` lacks it; since the arXiv profile runs these shipped classes raw
/// (DIVERGENCES #444) their authors were lost (1605.02827, 2003.09061, 1906.01122).
fn author_and_splits() -> Vec<SplitDelim> {
  vec![
    T_CS!("\\and").into(),
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    T_CS!("\\alignauthor").into(),
  ]
}
// Things to split author & affiliation mix; NO comma in affiliations!!!
fn author_affil_splits() -> Vec<SplitDelim> {
  vec![
    T_CS!("\\and").into(),
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    // Beyond-Perl: the literal word " and " is deliberately NOT a line-level
    // delimiter here (Perl's @authoraffilsplits includes it). This split runs
    // BEFORE author/affiliation classification, so splitting on " and " shreds
    // institution names that contain it — "Princeton Language and Intelligence"
    // → "Princeton Language" + "Intelligence, …" (re-joined without a space).
    // Authors written "Alice and Bob" still split: literal " and " is applied in
    // the author arm (comma_split_author_line), AFTER a line is known to be an
    // author line, so affiliations are never fragmented. Witness 2605.00347.
    T_CS!("\\quad").into(),
    T_CS!("\\qquad").into(),
    T_CS!("\\\\").into(),
    // latex.ltx:9256 `\DeclareRobustCommand\newline{\@normalcr\relax}`: the line break `\\` is (2401.14196's
    // `Ann Able$^{1}$, Bob Baker$^{1,2}$ \newline Cat Cole$^{1}$`)
    T_CS!("\\newline").into(),
  ]
}
/// Whether an author block holds the affiliations its marks request: a line after its first opening with a mark a
/// name before it requests (`Ann Able$^{1}$, Bob Baker$^{2}$\\ $^{1}$Univ A`), which the marked parse labels for the
/// names' marks to find. A line led by a mark no name requests is a name marked before it, or text (`${}^3$He …`), and
/// answers nothing; so does a name marked as the first, beside it on its printed line
/// ([`beside_prefix_marked_names`]: `$^{1}$Ann Able \quad $^{1}$Bob Baker`) or answered by a line below
/// ([`mark_answered_below`]).
fn answers_its_marks(stuff: &Tokens) -> bool {
  let labels = |operands: Vec<Vec<Token>>| -> Vec<String> {
    operands
      .into_iter()
      .flat_map(|operand| {
        clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation")
      })
      .collect()
  };
  let pieces = split_tokens_delimited(stuff.clone(), author_affil_splits());
  let lines: Vec<Tokens> = pieces.iter().map(|(_, line)| line.clone()).collect();
  let mut requested: Vec<String> = Vec::new();
  let mut prefix_marked: Option<bool> = None;
  let mut names_line = true;
  for (index, (delimiter, line)) in pieces.iter().enumerate() {
    let beside =
      beside_prefix_marked_names(prefix_marked == Some(true), &mut names_line, delimiter);
    if line.is_empty() {
      continue;
    }
    let leads = marker_leads(line);
    let Some(prefix_marked) = prefix_marked else {
      prefix_marked = Some(leads);
      requested.extend(labels(author_mark_operands(line.unlist_ref())));
      continue;
    };
    // a line of names beside the first on its printed line (a per-author block of names alone,
    // `\IEEEauthorblockN{$^{1}$Ann Able \quad $^{1}$Bob Baker}`, splits at the `\quad` on the unmarked path), or on a
    // line of its own when its mark is new and an affiliation below answers it, as the marked path reads it
    let new_mark = || {
      labels(
        mark_operands(line.unlist_ref())
          .into_iter()
          .take(1)
          .collect(),
      )
      .iter()
      .any(|label| !requested.contains(label))
    };
    let a_name = prefix_marked
      && reads_as_names(line)
      && (beside || (new_mark() && mark_answered_below(line, &lines[index + 1..])));
    if leads && !a_name {
      let leading = mark_operands(line.unlist_ref())
        .into_iter()
        .take(1)
        .collect();
      if labels(leading)
        .iter()
        .any(|label| requested.contains(label))
      {
        return true;
      }
    } else {
      requested.extend(labels(author_mark_operands(line.unlist_ref())));
    }
  }
  false
}

/// Whether an affiliation below `line` (among `below`, the lines of an author block after it) answers the mark
/// leading `line`: the label a name marked before it requests (`$^{1}$Ann Able\\ $^{1}$Bob Baker\\ $^{1}$Univ A`). The
/// answering part must read as an affiliation ([`reads_as_affiliation`]): a line led by the same mark that is an email
/// or a place (`$^{1}$\texttt{ann@google.com}`, `$^{1}$Beijing, China`) answers nothing, so a name-shaped affiliation
/// above it (`$^{1}$Google DeepMind`) stays one.
fn mark_answered_below(line: &Tokens, below: &[Tokens]) -> bool {
  let leading_label = |line: &Tokens| {
    mark_operands(line.unlist_ref())
      .into_iter()
      .next()
      .and_then(|operand| mark_label(&operand))
  };
  let Some(label) = leading_label(line) else {
    return false;
  };
  below
    .iter()
    .filter(|later| marker_leads(later))
    .any(|later| {
      marked_parts(later)
        .iter()
        .any(|part| leading_label(part).as_ref() == Some(&label) && reads_as_affiliation(part))
    })
}

/// The marked parts of a line (`split_wrapped_affiliation_marks`: `$^{1}$Univ A $^{2}$Univ B` → two), blank ones dropped.
fn marked_parts(line: &Tokens) -> Vec<Tokens> {
  split_wrapped_affiliation_marks(line.clone())
    .into_iter()
    .filter(|part| !part.unlist_ref().iter().all(|t| *t == T_SPACE!()))
    .collect()
}

/// Whether a marked line reads as names: each of its marked parts a name or names ([`name_count`]), so that
/// `$^{1}$Google DeepMind $^{2}$Huawei Technologies` is two parts, not one four-word name.
fn reads_as_names(line: &Tokens) -> bool {
  let parts = marked_parts(line);
  !parts.is_empty()
    && parts
      .iter()
      .all(|part| name_count(&visible_name_text(part.unlist_ref())) > 0)
}

/// Whether a line reads as an affiliation: no name, no address (`@`), and a word that names an institution
/// ([`non_name_word`]: `Univ A`, `Department of Physics`, `MIT`).
fn reads_as_affiliation(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  name_count(&text) == 0 && !text.contains('@') && text.split_whitespace().any(non_name_word)
}

/// Whether the line after `delimiter` stands beside the names of an author block whose names are marked before them
/// (`prefix_marked`: `$^{1}$Ann Able \quad $^{1}$Bob Baker \quad $^{2}$Cat Cole\\ $^{1}$Univ A`): on the printed line
/// of its first name, after `\quad`/`\qquad`, where a line break has not yet ended the names. `names_line` tracks it
/// across a block's lines: a line break ends it, the `\and` family opens the next author's.
fn beside_prefix_marked_names(
  prefix_marked: bool,
  names_line: &mut bool,
  delimiter: &[Token],
) -> bool {
  let written = |delims: &[Token]| delimiter.iter().any(|t| delims.contains(t));
  if written(&[T_CS!("\\\\"), T_CS!("\\newline")]) {
    *names_line = false;
  } else if written(&[T_CS!("\\and"), T_CS!("\\And"), T_CS!("\\AND")]) {
    *names_line = true;
  }
  prefix_marked && *names_line && written(&[T_CS!("\\quad"), T_CS!("\\qquad")])
}

/// The line breaks an author block's lines are split at: `\\` and `\newline` (latex.ltx:9253-9256).
fn author_line_breaks() -> Vec<SplitDelim> { vec![T_CS!("\\\\").into(), T_CS!("\\newline").into()] }
fn affil_splits() -> Vec<SplitDelim> {
  vec![
    // The \and CONTROL-SEQUENCE family is added (vs upstream, which splits
    // affiliations only on \quad/\qquad/\\): it lets the shared affiliation
    // parser handle \and-separated institute/affiliation lists (e.g. LNCS
    // \institute{A \and B}). NOTE: the literal word " and " is deliberately NOT
    // included (unlike author_affil_splits) — institution names routinely
    // contain "and" ("Electrical and Computer Engineering"), so splitting on it
    // wrongly fragments them. Affiliations never use ',' as a separator either.
    T_CS!("\\and").into(),
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    T_CS!("\\quad").into(),
    T_CS!("\\qquad").into(),
    T_CS!("\\\\").into(),
    T_CS!("\\newline").into(),
  ]
}
fn authorsup_markers() -> Vec<Token> { vec![T_SUPER!(), T_CS!("\\textsuperscript")] }

/// In a superscript-labeled author block, decide whether a line reads
/// "Name\textsuperscript{n}" (an author — name TEXT precedes the marker) or
/// "\textsuperscript{n}Affil" (an affiliation — the marker LEADS the line).
/// Returns true iff a letter token precedes the first marker at `marker_pos`
/// (1-based, from `position_of`). Replaces an earlier `position < 8`
/// token-count proxy that misread short author names: "Min Xu" is 7 tokens, so
/// its trailing `\textsuperscript{1}` fell under the threshold and the author
/// was reclassified as an affiliation (html_feedback#6614, arXiv:2606.08234).
/// Keying on "is there a name before the marker" is length-independent.
/// OXIDIZED_DESIGN #52.
fn name_precedes_marker(line: &Tokens, marker_pos: usize) -> bool {
  line.unlist_ref()[..marker_pos.saturating_sub(1)]
    .iter()
    .any(|t| t.code == Catcode::LETTER)
}

/// Drop the optional `*` and `[<len>]` that follow a `\\` line-break token in an
/// author/affiliation block, before the block is split into lines. The
/// author/affiliation splitters key on the bare `\\` control sequence, so a
/// `\\[1em]` (or `\\*[1em]`) would otherwise orphan `[1em]` at the head of the
/// next segment — where it leaks as literal `[1em]` text AND, by pushing a
/// following `\textsuperscript` past the author-vs-affiliation position
/// threshold, flips a comma-bearing affiliation line into phantom author
/// creators. Perl's own `\lx@add@authors` flags this exact gap ("matching `\\`
/// this way fails to catch `\\[1em]`, so really should Let it") but never fixes
/// it, so both engines garble it identically; consuming it here is a beyond-Perl
/// improvement (KNOWN_PERL_ERRORS #75, witness arXiv:2605.23553). The `[...]` is
/// matched only as a balanced OTHER-catcode bracket group immediately following
/// the `\\`, so ordinary bracketed content elsewhere in a name/affiliation is
/// untouched.
fn strip_linebreak_options(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let brk = T_CS!("\\\\");
  let mut i = 0;
  while i < n {
    out.push(src[i]);
    if src[i] == brk {
      let mut j = i + 1;
      let mut consumed = false;
      if j < n && src[j] == T_OTHER!("*") {
        j += 1;
        consumed = true;
      }
      if j < n && src[j] == T_OTHER!("[") {
        let mut k = j + 1;
        let mut depth = 1usize;
        while k < n && depth > 0 {
          if src[k] == T_OTHER!("[") {
            depth += 1;
          } else if src[k] == T_OTHER!("]") {
            depth -= 1;
          }
          k += 1;
        }
        if depth == 0 {
          j = k;
          consumed = true;
        }
      }
      if consumed {
        i = j;
        continue;
      }
    }
    i += 1;
  }
  Tokens::new(out)
}

/// Rewrite horizontal-space macros into `\quad`, so the author/affiliation
/// splitter treats them as separators. `\author{A \hspace{1cm} B \hspace{1cm} C}`
/// (and `\hfill`-separated variants) is a "regular" way to lay out co-authors
/// that LaTeXML's `\and`/`\quad` splitter otherwise misses, collapsing every name
/// into one `<personname>`. `\quad` is already a hard separator in every author /
/// affiliation split set, so rewriting to it needs no new delimiter plumbing.
/// `\hspace`'s optional `*` and mandatory `{len}` argument are consumed together
/// so the length cannot leak as literal text. OXIDIZED_DESIGN #52; witness
/// arXiv:2506.06941 (six authors separated by `\hspace{0.5cm}`).
fn normalize_hspace_separators(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let mut i = 0;
  while i < n {
    let t = src[i];
    if t == T_CS!("\\hspace") {
      let mut j = i + 1;
      // optional `*` (\hspace*)
      if j < n && src[j] == T_OTHER!("*") {
        j += 1;
      }
      // mandatory `{len}` group
      if j < n && src[j] == T_BEGIN!() {
        let mut depth = 1usize;
        j += 1;
        while j < n && depth > 0 {
          if src[j] == T_BEGIN!() {
            depth += 1;
          } else if src[j] == T_END!() {
            depth -= 1;
          }
          j += 1;
        }
      }
      out.push(T_CS!("\\quad"));
      i = j;
      continue;
    }
    if t == T_CS!("\\hfill") || t == T_CS!("\\hfil") {
      out.push(T_CS!("\\quad"));
      i += 1;
      continue;
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// The footnote-SYMBOL characters (`\fnsymbol`-style) and control sequences that,
/// as a superscript on an author, mark an equal-contribution / corresponding /
/// note relation — NEVER an affiliation (which is numbered). Kept deliberately to
/// pure symbols so a numeric (`1`) or lettered (`a`) affiliation mark is never
/// misread as a note. Mirrors the note-vs-affiliation split already documented in
/// `starts_with_affiliation_mark`.
fn is_footnote_symbol_operand(sym: &[Token]) -> bool {
  const SYMBOL_CHARS: &[&str] = &[
    "*", "\u{2217}", "\u{2020}", "\u{2021}", "\u{A7}", "\u{B6}", "\u{22C6}", "\u{2605}",
    "\u{2022}", "\u{25E6}", "\u{2666}",
  ];
  const SYMBOL_CS: &[&str] = &[
    "\\ast",
    "\\star",
    "\\dagger",
    "\\ddagger",
    "\\S",
    "\\P",
    "\\bullet",
    "\\diamond",
    "\\circ",
    "\\sharp",
    "\\|",
    "\\#",
  ];
  let mut saw_symbol = false;
  for t in sym {
    match t.get_catcode() {
      Catcode::SPACE => continue,
      Catcode::CS => {
        if !t.with_str(|s| SYMBOL_CS.contains(&s)) {
          return false;
        }
        saw_symbol = true;
      },
      Catcode::OTHER | Catcode::LETTER => {
        if !t.with_str(|s| SYMBOL_CHARS.contains(&s)) {
          return false;
        }
        saw_symbol = true;
      },
      _ => return false,
    }
  }
  saw_symbol
}

/// Is a mark's operand a footnote symbol (`*`, `\dagger`, `\dag`), an equal-contribution or correspondence legend's
/// mark rather than an affiliation number?
fn is_symbol_mark(operand: &[Token]) -> bool {
  is_footnote_symbol_operand(operand)
    || matches!(
      operand.iter().filter(|t| **t != T_SPACE!()).collect::<Vec<_>>()[..],
      [t] if *t == T_CS!("\\dag") || *t == T_CS!("\\ddag")
    )
}

/// Does `line` open (after spaces, math shifts and braces) with a footnote symbol (`* Equal contribution`,
/// `$\dagger$ Corresponding author`, `\dag Corresponding author`) — a legend for the authors' symbol marks, not an
/// affiliation line? The text-mode `\dag`/`\ddag` count here only: as superscript marks (`$^\dag$`) they still label
/// affiliations (`is_footnote_symbol_operand` leaves them out, so `rewrite_symbol_superscripts` keeps them as marks).
fn starts_with_footnote_symbol(line: &Tokens) -> bool {
  line
    .unlist_ref()
    .iter()
    .find(|t| {
      !matches!(
        t.get_catcode(),
        Catcode::SPACE | Catcode::MATH | Catcode::BEGIN | Catcode::END
      )
    })
    .is_some_and(|t| {
      *t == T_CS!("\\dag")
        || *t == T_CS!("\\ddag")
        || is_footnote_symbol_operand(std::slice::from_ref(t))
    })
}

/// Rewrite footnote-SYMBOL author superscripts — `$^{*}$`, `${}^{\dagger}$`,
/// `\textsuperscript{\ddagger}`, a bare `^{*}` — onto the visible
/// `\lx@frontmatter@keepsup` sentinel, BEFORE author-block branch selection.
///
/// Two effects, both wanted (OXIDIZED_DESIGN #52; witness arXiv:2506.06941, where
/// Iman Mirzadeh's literal `$^{*}$` was silently dropped):
///   * the mark no longer counts as an affiliation superscript, so a block whose
///     ONLY superscript is such a note-mark takes the clean no-marker author
///     branch instead of the affiliation-linking one;
///   * the symbol renders as a real superscript instead of being consumed into an
///     `affiliation:*` label that matches no affiliation and is discarded.
///
/// NUMERIC/lettered affiliation marks (`$^{1}$`, `\textsuperscript{a}`) are left
/// untouched, so affiliation linking is unaffected. Only the specific
/// superscript-mark token shapes are matched; a superscript inside real math
/// (`$x^2$`) is not (its base is not empty), so math content is preserved.
fn rewrite_symbol_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let mut i = 0;
  // Read a superscript operand at `src[k]`: a braced `{...}` group, or a single
  // token. Returns (operand tokens, index past the operand) or None.
  let read_operand = |k: usize| -> Option<(Vec<Token>, usize)> {
    if k >= n {
      return None;
    }
    if src[k] == T_BEGIN!() {
      let mut depth = 1usize;
      let mut m = k + 1;
      let mut inner = Vec::new();
      while m < n && depth > 0 {
        if src[m] == T_BEGIN!() {
          depth += 1;
        } else if src[m] == T_END!() {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        inner.push(src[m]);
        m += 1;
      }
      if depth == 0 {
        Some((inner, m + 1))
      } else {
        None
      }
    } else {
      Some((vec![src[k]], k + 1))
    }
  };
  while i < n {
    let t = src[i];
    // `\textsuperscript{sym}` — always braced in practice.
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((sym, next)) = read_operand(i + 1)
      && is_footnote_symbol_operand(&sym)
    {
      out.extend(keepsup(sym));
      i = next;
      continue;
    }
    // `$ [ {} ] ^ sym $` — a math span whose whole content is a superscript on an
    // empty base. Rewrite the entire span (delimiters included) to a text sup.
    if t == T_MATH!()
      && let Some(close) = (i + 1..n).find(|&m| src[m] == T_MATH!())
    {
      // optional empty base `{}` before the superscript
      let p = if i + 2 < close && src[i + 1] == T_BEGIN!() && src[i + 2] == T_END!() {
        i + 3
      } else {
        i + 1
      };
      if p < close
        && src[p] == T_SUPER!()
        && let Some((sym, next)) = read_operand(p + 1)
        && next == close
        && is_footnote_symbol_operand(&sym)
      {
        out.extend(keepsup(sym));
        i = close + 1;
        continue;
      }
    }
    // Only the math-delimited (`$…$`) and `\textsuperscript{…}` spellings are
    // rewritten — the correct ways to write a text superscript. A bare `^` is NOT
    // matched here: outside math it is invalid LaTeX, and a `^` reached mid-scan is
    // the superscript operator of real inline math on a non-empty base (`$a^{*}$`),
    // which must be left intact.
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// `\lx@frontmatter@keepsup{<sym>}` as a token list: a superscript shown as it is, requesting nothing.
fn visible_sup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = vec![T_CS!("\\lx@frontmatter@keepsup"), T_BEGIN!()];
  v.extend(sym);
  v.push(T_END!());
  v
}

/// A footnote-symbol author mark: shown (`visible_sup`), and requesting the affiliation of its symbol
/// (`\lx@frontmatter@symbolmark`).
fn keepsup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = visible_sup(sym.clone());
  v.extend([T_CS!("\\lx@frontmatter@symbolmark"), T_BEGIN!()]);
  v.extend(sym);
  v.push(T_END!());
  v
}

/// Rewrite an ordinal suffix superscript after a numeral (`5\textsuperscript{th}`, `21$^{st}$`) onto a visible
/// superscript: it is the number's, not an affiliation mark, so a marked list is not split at it nor labelled by it
/// (repro sectioning-frontmatter/affiliation_line_superscript_ordinal_is_not_a_mark; iopart's
/// `\address{$^1$Univ A, 5 \textsuperscript{th} floor}`, 2609.01831).
fn rewrite_ordinal_superscripts(tokens: Tokens) -> Tokens {
  // The suffix as written, a font wrapper around it aside (`$^{\rm th}$`, `\textsuperscript{\mathrm{nd}}`).
  const FONT_WRAPPERS: &[&str] = &[
    "\\rm", "\\mathrm", "\\textrm", "\\text", "\\mbox", "\\textup", "\\mathup", "\\it", "\\mathit",
    "\\textit", "\\bf", "\\mathbf", "\\textbf", "\\sf", "\\mathsf", "\\textsf",
  ];
  let is_ordinal = |operand: &[Token]| {
    let text: String = operand
      .iter()
      .filter(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
      .map(|t| t.with_str(str::to_lowercase))
      .collect();
    operand.iter().all(|t| match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER | Catcode::SPACE | Catcode::BEGIN | Catcode::END => true,
      Catcode::CS => FONT_WRAPPERS.iter().any(|w| *t == T_CS!(*w)),
      _ => false,
    }) && matches!(text.as_str(), "st" | "nd" | "rd" | "th")
  };
  let is_digit = |t: &Token| {
    t.get_catcode() == Catcode::OTHER
      && t.with_str(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
  };
  let after_numeral = |out: &[Token]| {
    out
      .iter()
      .rev()
      .find(|t| **t != T_SPACE!())
      .is_some_and(&is_digit)
  };
  // The suffix is shown in text: a math-only wrapper takes its text form (`$^{\mathrm{th}}$`).
  let text_mode = |operand: Vec<Token>| -> Vec<Token> {
    operand
      .into_iter()
      .map(|t| {
        if t == T_CS!("\\mathrm") {
          T_CS!("\\textrm")
        } else if t == T_CS!("\\mathup") {
          T_CS!("\\textup")
        } else if t == T_CS!("\\mathit") {
          T_CS!("\\textit")
        } else if t == T_CS!("\\mathbf") {
          T_CS!("\\textbf")
        } else if t == T_CS!("\\mathsf") {
          T_CS!("\\textsf")
        } else {
          t
        }
      })
      .collect()
  };
  let src = tokens.unlist();
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    if t == T_CS!("\\textsuperscript")
      && let Some((operand, next)) = sup_operand_at(&src, i + 1)
      && is_ordinal(&operand)
      && after_numeral(&out)
    {
      out.extend(visible_sup(text_mode(operand)));
      i = next;
      continue;
    }
    if t == T_MATH!() {
      // `$^{th}$` after a numeral, or `$5^{th}$` with the numeral inside: the digits stay, the suffix is shown.
      let digits_end = (i + 1..src.len())
        .find(|&k| !is_digit(&src[k]))
        .unwrap_or(src.len());
      if src.get(digits_end) == Some(&T_SUPER!())
        && let Some((operand, next)) = sup_operand_at(&src, digits_end + 1)
        && src.get(next) == Some(&T_MATH!())
        && is_ordinal(&operand)
        && (digits_end > i + 1 || after_numeral(&out))
      {
        out.extend_from_slice(&src[i + 1..digits_end]);
        out.extend(visible_sup(text_mode(operand)));
        i = next + 1;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// Best-effort content-kind classifier for a creator-scope `\thanks`, used ONLY to
/// attach a semantic `ltx_thanks_<kind>` CSS hook (OXIDIZED_DESIGN #156) — it never
/// affects core semantics. Keyword-matched over the flattened, lowercased note text.
/// Order matters: correspondence and equal-contribution are checked before funding
/// (a "supported by …" note is funding; "contributed equally" is contribution).
/// Witnesses: arXiv 2512.24601 (correspondence), 1510.02728 (funding),
/// 2506.06941 "Equal contribution" (contribution). Explicitly best-effort.
fn classify_thanks(text: &str) -> &'static str {
  let t = text.to_lowercase();
  if t.contains("correspond") {
    "correspondence"
  } else if t.contains("equal") && t.contains("contribut") {
    "contribution"
  } else if t.contains("now at") || t.contains("present address") || t.contains("current address") {
    "address"
  } else if t.contains("support")
    || t.contains("grant")
    || t.contains("fund")
    || t.contains("nsf")
    || t.contains("nih")
    || t.contains("onr")
    || t.contains("darpa")
    || t.contains("erc")
  {
    "funding"
  } else {
    "note"
  }
}

/// The body a creator's note (`\lx@add@thanks`, `\lx@add@note`) digests: its leading marks as the author line reads
/// them when an author's mark requests them and the note is one legend — a `\thanks{\textsuperscript{$\dagger$} School
/// ...}` is labelled by its mark and goes to the starred authors (2609.00885, 2609.19600) — and the rest, or all of it
/// when no author requests the mark or more legends follow, with the plain superscripts (`\lx@frontmatter@plainsups`): its `10 m$^{2}$` is text, not a mark
/// (KNOWN_PERL_ERRORS #515), and an unrequested leading mark stays shown, its note with its author.
fn note_body(content: Tokens) -> Tokens {
  let toks = content.unlist();
  let lead = toks.iter().take_while(|t| **t == T_SPACE!()).count();
  let end = lead + leading_marks_end(&toks[lead..]);
  let requested = lookup_string("lx_frontmatter_requested_marks");
  // One legend: no other mark after a space in its text (`$^{\dag}$ These authors ... $^{*}$ This author ...` is several
  // legends and a funding line in one `\thanks`, 2609.39576, which stays whole with its author), where `10 m$^{2}$`
  // is glued to its base.
  // A numeral's spaced suffix (`5 $^{th}$`, shown by `rewrite_ordinal_superscripts`) is no legend.
  let after_numeral = |i: usize| {
    i >= 2
      && toks[i - 2].get_catcode() == Catcode::OTHER
      && toks[i - 2].with_str(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
  };
  let another_legend = (end + 1..toks.len())
    .any(|i| toks[i - 1] == T_SPACE!() && !after_numeral(i) && leading_marks_end(&toks[i..]) > 0);
  let marks_requested = end > lead
    && !another_legend
    && mark_operands(&toks[lead..end]).iter().any(|operand| {
      clean_frontmatter_labels(&Tokens::new(operand.clone()).to_string(), "affiliation")
        .iter()
        .any(|label| requested.split('\u{1}').any(|r| r == label))
    });
  let split = if marks_requested { end } else { 0 };
  let mut body = toks[..split].to_vec();
  body.extend([T_CS!("\\bgroup"), T_CS!("\\lx@frontmatter@plainsups")]);
  body.extend_from_slice(&toks[split..]);
  body.push(T_CS!("\\egroup"));
  Tokens::new(body)
}

/// Does `content` start with a letter or number superscript mark (`$^{1}$Univ A`, `$^a$…`, `\textsuperscript{b}…`):
/// a marked affiliation list, its lines labelled by their marks? A symbol (`$^{*}$Corresponding author`) is a note's
/// mark, and a superscript later in the line (`Laboratory for $^{3}$He`) is the affiliation's own text.
pub fn leads_with_mark(content: &Tokens) -> bool {
  // (past a leading font switch or group: `\affiliation{\it $^1$ Key Laboratory …}`, 0808.2763)
  let declarations = [
    "\\it",
    "\\rm",
    "\\bf",
    "\\sl",
    "\\sf",
    "\\tt",
    "\\em",
    "\\itshape",
    "\\upshape",
    "\\small",
    "\\footnotesize",
    "\\normalsize",
    "\\noindent",
  ];
  let tokens: Vec<&Token> = content
    .unlist_ref()
    .iter()
    .filter(|t| **t != T_SPACE!())
    .skip_while(|t| **t == T_BEGIN!() || declarations.iter().any(|cs| **t == T_CS!(cs)))
    .collect();
  let script = match tokens.as_slice() {
    [t, rest @ ..] if **t == T_CS!("\\textsuperscript") || **t == T_SUPER!() => rest,
    [t, s, rest @ ..] if t.get_catcode() == Catcode::MATH && **s == T_SUPER!() => rest,
    _ => return false,
  };
  script
    .iter()
    .find(|t| ***t != T_BEGIN!() && ***t != T_CS!("\\rm") && ***t != T_CS!("\\mathrm"))
    .is_some_and(|t| {
      t.to_string()
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric)
    })
}

/// Does this token list *begin* with a NUMERIC affiliation superscript mark
/// (`$^{1}…` / `$^1…` / `\textsuperscript{1}…`)? This is the signature of the
/// arXiv "`\thanks` abuse" idiom (affiliations smuggled into an author
/// `\thanks{...}` and linked to authors by a leading mark).
///
/// The mark must be a **digit**: affiliation linking is by number, so a numeric
/// mark is the reliable abuse signal. Crucially, footnote-SYMBOL marks
/// (`$^*$`, `$^\dagger$`, `$^\ddagger$`, `$^\S$`) and lettered marks head
/// *legitimate* acknowledgements — corresponding-author / equal-contribution /
/// present-address notes — which must stay `role=thanks`. Requiring a digit
/// excludes them, so those notes are never re-routed into an affiliation that
/// fails to link and is then discarded (i.e. no silent note loss). Operates on
/// the string form so it is agnostic to the Semiverbatim catcode-freeze on
/// `$`/`^`. (user-directed, 2026-07-07; reviewer-hardened.)
fn starts_with_affiliation_mark(content: &Tokens) -> bool {
  let s = content.to_string();
  let st = s.trim_start();
  let after_opener = match st
    .strip_prefix("$^")
    .or_else(|| st.strip_prefix("\\textsuperscript"))
  {
    Some(rest) => rest,
    None => return false,
  };
  // The superscript content may be braced (`{1}`) or bare (`1`); either way the
  // first content character must be an ASCII digit.
  let content_start = after_opener.strip_prefix('{').unwrap_or(after_opener);
  content_start
    .chars()
    .next()
    .is_some_and(|c| c.is_ascii_digit())
}

/// Split an affiliation blob at each embedded superscript mark (`$^{n}$` /
/// `\textsuperscript{n}`), keeping the mark at the head of the following
/// segment. The abuse idiom packs several affiliations onto ONE line delimited
/// only by their marks (no `\\`), so `affil_splits()` (which splits on `\\`)
/// cannot separate them. Each returned segment then flows through
/// `\lx@affiliation@withsup`, which turns its leading mark into the
/// `affiliation:N` label that the authors' own marks already request
/// (`relocate_annotations` links them).
fn split_before_affiliation_marks(tokens: Tokens) -> Vec<Tokens> {
  // Trim the institution SEPARATOR that clings to the end of a segment when the
  // affiliations are comma-joined (`\textsuperscript{1}Univ A, \textsuperscript{2}
  // Univ B`) rather than space-joined: without this the affiliation contact reads
  // "Univ A," with a stray trailing comma (html_feedback#6588, arXiv:2606.01317).
  // Only trailing whitespace + a SINGLE trailing comma (or semicolon, `$^1$Univ A; $^2$Univ B`,
  // 2609.22690) are removed, so a comma INSIDE an institution name ("Dept X, Univ Y, City") is untouched.
  fn trim_trailing_separator(mut toks: Vec<Token>) -> Tokens {
    while toks.last() == Some(&T_SPACE!()) {
      toks.pop();
    }
    if toks.last() == Some(&T_OTHER!(",")) || toks.last() == Some(&T_OTHER!(";")) {
      toks.pop();
      while toks.last() == Some(&T_SPACE!()) {
        toks.pop();
      }
    }
    Tokens::new(toks)
  }
  let toks = tokens.unlist();
  let mut segments: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  // A boundary only outside every group: a cut inside `{…}` would leave the pieces unbalanced (a mark
  // list inside a font group is split by `split_wrapped_affiliation_marks`, which repeats the group).
  let mut depth = 0usize;
  for (i, t) in toks.iter().enumerate() {
    let dollar_super = *t == T_MATH!() && toks.get(i + 1).is_some_and(|n| *n == T_SUPER!());
    let is_mark_start = dollar_super || *t == T_CS!("\\textsuperscript");
    // Only treat a mark as an affiliation boundary when it is preceded by
    // whitespace (institutions are space-separated). A mark glued to the
    // preceding text — e.g. a superscript INSIDE an institution name,
    // "Center for R$^2$ Studies" — is not a boundary, so the name is not
    // wrongly split (reviewer-flagged). The first mark (current empty) always
    // opens segment 0.
    // Declarations alone before the first mark (`\color{blue} $^1$…`) are no segment: they open the first one.
    let only_declarations = || {
      current.iter().any(|t| t.get_catcode() == Catcode::CS)
        && opening_declarations(&Tokens::new(current.clone())).len() == current.len()
    };
    if is_mark_start && depth == 0 && current.last() == Some(&T_SPACE!()) && !only_declarations() {
      segments.push(trim_trailing_separator(std::mem::take(&mut current)));
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    current.push(*t);
  }
  if !current.is_empty() {
    segments.push(trim_trailing_separator(current));
  }
  segments
}

/// [`split_before_affiliation_marks`] for a marker-led affiliation line that opens with a font command or a group
/// holding the marks (`\textit{$^1$Univ A, $^2$Univ B}`, `{\small $^1$A; $^2$B}`, `\small{$^1$A, $^2$B}. Email: …`):
/// the inner list is split, the wrapper repeated on each piece — a group with the declarations that open it
/// (`\small`) — and what follows the wrapper kept after the last piece; else the marks inside the group were no
/// boundary, or every piece after the first left the font (62zk review; 2609.00995; repro
/// sectioning-frontmatter/affiliation_marks_inside_a_font_group_keep_the_font). A `\thanks{…}` wrapper is not
/// repeated: its mark-led content is the affiliation list (the `\thanks` idiom of `\lx@add@thanks`; 2609.24896).
fn split_wrapped_affiliation_marks(line: Tokens) -> Vec<Tokens> {
  if let Some((cmd, inner, trailing)) = leading_wrapper(&line) {
    // A bare group's opening declarations (`\small`, `\it`, `\color{blue}` with its argument), repeated in each
    // later piece.
    let opening = if cmd.is_none() {
      opening_declarations(&inner)
    } else {
      Vec::new()
    };
    let mut pieces = split_wrapped_affiliation_marks(inner);
    if pieces.len() > 1 {
      let mut out: Vec<Tokens> = pieces
        .drain(..)
        .enumerate()
        .map(|(i, piece)| {
          let mut wrapped = Vec::new();
          if cmd == Some(T_CS!("\\thanks")) {
            wrapped.extend(piece.unlist());
          } else {
            wrapped.extend(cmd);
            wrapped.push(T_BEGIN!());
            if i > 0 {
              wrapped.extend(opening.iter().copied());
            }
            wrapped.extend(piece.unlist());
            wrapped.push(T_END!());
          }
          Tokens::new(wrapped)
        })
        .collect();
      // What follows the wrapper is split at its own marks too (`\textit{$^1$A, $^2$B} $^3$C`): its first piece stays
      // with the last wrapped one, the rest are pieces of their own.
      let mut after = split_before_affiliation_marks(Tokens::new(trailing)).into_iter();
      if let (Some(first), Some(last_piece)) = (after.next(), out.last_mut()) {
        let mut joined = last_piece.clone().unlist();
        joined.extend(first.unlist());
        *last_piece = Tokens::new(joined);
      }
      out.extend(after);
      return out;
    }
  }
  split_before_affiliation_marks(line)
}

/// Does a mark lead `line` — no text before its first mark, looking inside a leading font command or group past the
/// declarations that open it (`{\color{blue} $^1$Univ A, $^2$Univ B}`: the letters of `blue` are no name)?
fn marker_leads(line: &Tokens) -> bool {
  let probe = match leading_wrapper(line) {
    Some((_, inner, _)) => {
      let skip = opening_declarations(&inner).len();
      Tokens::new(inner.unlist_ref()[skip..].to_vec())
    },
    None => {
      let skip = opening_declarations(line).len();
      Tokens::new(line.unlist_ref()[skip..].to_vec())
    },
  };
  match position_of(&probe, &authorsup_markers()) {
    Some(p) => !name_precedes_marker(&probe, p),
    // No mark inside the wrapper (`{}$^1$Univ A`, `\noindent{}$^1$…`): the whole line decides, as before.
    None => position_of(line, &authorsup_markers()).is_some_and(|p| !name_precedes_marker(line, p)),
  }
}

/// The declarations that open a group's content (`\small`, `\bfseries`, `\color{blue}` with its argument), up to its
/// first mark, text, or a command over a braced argument (`\textbf{Ann Able}` is content, not a declaration: read
/// as one, `\large\textbf{Ann Able}$^1$` became an affiliation line and lost its author).
fn opening_declarations(inner: &Tokens) -> Vec<Token> {
  // The declarations that take brace arguments (latex.ltx `\color`, the NFSS font selectors).
  const WITH_ARGUMENTS: &[&str] = &[
    "\\color",
    "\\fontsize",
    "\\fontfamily",
    "\\fontseries",
    "\\fontshape",
    "\\usefont",
  ];
  let v = inner.unlist_ref();
  let mut out = Vec::new();
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    let braced = v
      .get(i + 1)
      .is_some_and(|n| n.get_catcode() == Catcode::BEGIN);
    let with_arguments = WITH_ARGUMENTS.iter().any(|cs| t == T_CS!(*cs));
    if t == T_SPACE!() {
      out.push(t);
      i += 1;
    } else if t.get_catcode() == Catcode::CS
      && t != T_CS!("\\textsuperscript")
      && (with_arguments || !braced)
    {
      out.push(t);
      i += 1;
      // its brace-group arguments
      while i < v.len() && v[i].get_catcode() == Catcode::BEGIN {
        let mut depth = 0usize;
        while i < v.len() {
          let a = v[i];
          out.push(a);
          i += 1;
          match a.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
        }
      }
    } else {
      break;
    }
  }
  out
}

/// A line that opens (spaces aside) with `\cmd{…}` or a bare `{…}`: the command (none for a bare group), the group's
/// inside, and the tokens after it. A leading `\textsuperscript{n}` is a mark, not a wrapper.
fn leading_wrapper(line: &Tokens) -> Option<(Option<Token>, Tokens, Vec<Token>)> {
  let v = line.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!())?;
  let (cmd, open) = match v[start].get_catcode() {
    Catcode::CS
      if v
        .get(start + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN) =>
    {
      if v[start] == T_CS!("\\textsuperscript") {
        return None;
      }
      (Some(v[start]), start + 1)
    },
    Catcode::BEGIN => (None, start),
    _ => return None,
  };
  let mut depth = 0usize;
  for (i, t) in v.iter().enumerate().skip(open) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some((
            cmd,
            Tokens::new(v[open + 1..i].to_vec()),
            v[i + 1..].to_vec(),
          ));
        }
      },
      _ => {},
    }
  }
  None
}

/// Commands whose braced argument is a note or a mark, not part of the name beside it.
const NAME_ANNOTATIONS: &[&str] = &[
  "\\thanks",
  "\\thanksref",
  "\\footnote",
  "\\altaffilmark",
  "\\inst",
  "\\IEEEauthorrefmark",
  "\\textsuperscript",
  "\\email",
  "\\orcidlink",
  "\\orcidID",
  "\\lx@aas@checkorcid",
  "\\IEEEmembership",
  // a footnote-symbol mark, rewritten before the parse (`rewrite_symbol_superscripts`: `$^{*}$`)
  "\\lx@frontmatter@keepsup",
  "\\lx@frontmatter@symbolmark",
];

/// The letter a letter command stands for (latex.ltx / the OT1 and T1 encodings: `\L` Ł, `\o` ø, `\ss` ß), as a
/// name-shape test reads it: its base letter, keeping its case.
fn letter_command(cs: &str) -> Option<char> {
  match cs {
    "\\L" | "\\l" | "\\O" | "\\o" | "\\i" | "\\j" => cs.chars().nth(1),
    "\\AA" | "\\AE" => Some('A'),
    "\\aa" | "\\ae" => Some('a'),
    "\\OE" => Some('O'),
    "\\oe" => Some('o'),
    "\\ss" => Some('s'),
    _ => None,
  }
}

/// The accent commands of latex.ltx / LaTeX's text encodings, whose argument is one letter of the word.
const ACCENT_COMMANDS: &[&str] = &[
  "\\'", "\\`", "\\^", "\\\"", "\\~", "\\=", "\\.", "\\u", "\\v", "\\H", "\\c", "\\d", "\\b",
  "\\t", "\\r", "\\k",
];

/// The visible text of an author line as a name-shape test reads it: its letters, other characters and spaces at
/// any brace depth, without control sequences, math, superscripts, or the argument of a note or mark command
/// ([`NAME_ANNOTATIONS`]). `{\it D.A. Johnston}` reads "D.A. Johnston", `Avi Shporer\altaffilmark{1}` "Avi Shporer",
/// `Plech\'a\v{c}` "Plechac", `Zheng \& Jordi` "Zheng & Jordi".
fn visible_name_text(tokens: &[Token]) -> String {
  let mut text = String::new();
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::CS => {
        // a control space or thin space between initials (`Joshua N.\ Winn`, `J.\,R. Smith`) is a space
        if t == T_CS!("\\ ") || t == T_CS!("\\,") {
          text.push(' ');
        } else if t == T_CS!("\\&") {
          text.push('&');
        } else if let Some(letter) = t.with_str(letter_command) {
          // a letter written as a command (`{\L}ukasz`, `\O stergaard`) is that letter, the spaces ending the
          // command's name aside
          text.push(letter);
          while tokens.get(i + 1) == Some(&T_SPACE!()) {
            i += 1;
          }
        }
        let annotation = t.with_str(|s| NAME_ANNOTATIONS.contains(&s));
        i += 1;
        // an accent's spaces before its letter are not text (`Plech\' a\v{c}`, cond-mat9705101; tex.web §1123
        // `\accent` skips them)
        if t.with_str(|s| ACCENT_COMMANDS.contains(&s)) {
          while tokens.get(i) == Some(&T_SPACE!()) {
            i += 1;
          }
        }
        // an optional `[...]` (`\footnotemark[2]` has nothing more), then a note's or mark's braced argument
        if annotation || t == T_CS!("\\footnotemark") {
          if tokens.get(i) == Some(&T_OTHER!("[")) {
            while i < tokens.len() && tokens[i] != T_OTHER!("]") {
              i += 1;
            }
            i += 1;
          }
          if annotation {
            i = skip_group(tokens, i);
          }
        }
        // a space with its length is no text (`Yu.S.Velikzhanin \vspace{1mm}\\`, hep-ex0105093), a horizontal one a
        // space
        if t == T_CS!("\\vspace") || t == T_CS!("\\hspace") {
          if tokens.get(i) == Some(&T_OTHER!("*")) {
            i += 1;
          }
          i = skip_group(tokens, i);
          if t == T_CS!("\\hspace") {
            text.push(' ');
          }
        }
        continue;
      },
      Catcode::MATH => {
        i += 1;
        while i < tokens.len() && tokens[i].get_catcode() != Catcode::MATH {
          i += 1;
        }
      },
      Catcode::SUPER | Catcode::SUB => {
        i = skip_group(tokens, i + 1);
        continue;
      },
      Catcode::LETTER | Catcode::OTHER => t.with_str(|s| text.push_str(s)),
      Catcode::SPACE => text.push(' '),
      // `~` is a space; another active character (babel's `"`) is no text
      Catcode::ACTIVE if t.with_str(|s| s == "~") => text.push(' '),
      _ => {},
    }
    i += 1;
  }
  text
}

/// The index after the brace group (or single token) at `i`.
fn skip_group(tokens: &[Token], mut i: usize) -> usize {
  if tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let mut depth = 0i32;
    while i < tokens.len() {
      match tokens[i].get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return i + 1;
          }
        },
        _ => {},
      }
      i += 1;
    }
    i
  } else {
    i + 1
  }
}

/// Particles that stand inside a personal name (`Camillo De Lellis`, `Ludwig van Beethoven`) but never open or close it.
const NAME_PARTICLES: &[&str] = &[
  "van", "von", "de", "der", "den", "del", "della", "da", "di", "du", "la", "le", "ten", "ter",
  "bin", "ibn", "al", "y", "dos", "das", "do",
];
/// Words that name an institution, a place or a role, never a person — whole words, lowercased, the punctuation
/// around them (`.,;:()[]`) dropped, in the languages arXiv affiliations are written in (`Dipartimento di Fisica`, `Sezione di Napoli`,
/// `Osservatorio Astronomico`, `École`). Whole words, not word openings: "Strasser", "Schooler", "Campusano" are
/// surnames.
const NON_NAME_WORDS: &[&str] = &[
  "univ",
  "university",
  "universities",
  "universität",
  "universitat",
  "universitaet",
  "universidad",
  "universidade",
  "université",
  "universite",
  "universiteit",
  "università",
  "universita",
  "universitet",
  "uniwersytet",
  "institute",
  "institutes",
  "institut",
  "instituto",
  "istituto",
  "institution",
  "inst",
  "laboratory",
  "laboratories",
  "laboratoire",
  "laboratorio",
  "laboratorium",
  "lab",
  "labs",
  "observatory",
  "observatoire",
  "observatorio",
  "osservatorio",
  "observatorium",
  "sterrewacht",
  "academy",
  "academia",
  "académie",
  "academie",
  "akademie",
  "accademia",
  "acad",
  "department",
  "departments",
  "departement",
  "département",
  "departamento",
  "dipartimento",
  "departament",
  "dept",
  "dep",
  "faculty",
  "faculté",
  "faculte",
  "facultad",
  "faculdade",
  "fakultät",
  "fakultat",
  "fachbereich",
  "division",
  "sezione",
  "section",
  "sektion",
  "collaboration",
  "consortium",
  "hospital",
  "hôpital",
  "hopital",
  "ospedale",
  "klinik",
  "klinikum",
  "clinic",
  "clinique",
  "foundation",
  "fundación",
  "fundacion",
  "fondazione",
  "stiftung",
  "society",
  "société",
  "societe",
  "società",
  "professor",
  "ministry",
  "museum",
  "company",
  "corporation",
  "corp",
  "inc",
  "llc",
  "ltd",
  "gmbh",
  "physics",
  "physik",
  "research",
  "school",
  "scuola",
  "escuela",
  "école",
  "ecole",
  "hochschule",
  "politecnico",
  "polytechnic",
  "polytechnique",
  "politécnica",
  "politecnica",
  "college",
  "collège",
  "colegio",
  "center",
  "centers",
  "centre",
  "centres",
  "centro",
  "zentrum",
  "centrum",
  "ctr",
  "program",
  "programme",
  "agency",
  "consiglio",
  "conseil",
  "avenue",
  "strasse",
  "straße",
  // a two-word place reads as a name (`\author{John Smith, New York}`): its words that no name carries
  "new",
  "tel",
  "team",
  "group",
  "member",
  "fellow",
  "student",
  "email",
  "e-mail",
  "obs",
  "natl",
  "sch",
  "sci",
];
/// Acronyms of institutions and societies, in capitals only: "Meta AI", "ESA" are none of them a name, though Qingyao
/// Ai and Esa Räsänen are.
const NON_NAME_ACRONYMS: &[&str] = &[
  "AI", "IEEE", "ACM", "INFN", "CNRS", "INAF", "CSIC", "NASA", "ESA", "ESO", "CERN", "DESY",
  "RIKEN", "KEK", "IPMU", "MIT", "UCLA", "SISSA",
];

/// The lowercased whole words of `word`: itself, the punctuation around it (`.,;:()[]`) dropped, and the parts of a
/// hyphenated compound (`Max-Planck-Institut`).
fn word_forms(word: &str) -> impl Iterator<Item = String> + '_ {
  let bare = word.trim_matches(|c: char| ".,;:()[]".contains(c));
  std::iter::once(bare)
    .chain(bare.split('-'))
    .map(str::to_lowercase)
}

/// Whether a word names an institution, a place or a role ([`NON_NAME_WORDS`], [`NON_NAME_ACRONYMS`]), never a person.
fn non_name_word(word: &str) -> bool {
  let bare = word.trim_matches(|c: char| ".,;:()[]".contains(c));
  word_forms(word).any(|form| NON_NAME_WORDS.contains(&form.as_str()))
    || bare
      .split('-')
      .any(|part| NON_NAME_ACRONYMS.contains(&part))
}

/// `text` without the marks glued to its end (`Ann Able*`, `Bob Baker1,`): digits and symbols right after a letter.
/// Digits after a space are text — a postal code (`CA 93106`), not a mark.
fn strip_glued_marks(text: &str) -> &str {
  let text = text.trim();
  let stripped = text.trim_end_matches(|c: char| c.is_ascii_digit() || "*∗†‡§¶♯#,".contains(c));
  if stripped.len() < text.len() && stripped.ends_with(char::is_whitespace) {
    text
  } else {
    stripped.trim_end()
  }
}

/// Whether `text` — one part of an author line, as [`visible_name_text`] reads it — is shaped like a personal name:
/// two to five words, each capitalised or an initial (`J.`, `D.A.`), name particles inside but not at either end, no
/// digit, `@`, `:`, `/` or parenthesis and no institution word, once the marks glued to its end are dropped.
fn name_shaped(text: &str) -> bool {
  let text = strip_glued_marks(text);
  if text
    .chars()
    .any(|c| c.is_ascii_digit() || "@:/()".contains(c))
  {
    return false;
  }
  let words: Vec<&str> = text.split_whitespace().flat_map(unglued_initials).collect();
  if !(2..=5).contains(&words.len()) {
    return false;
  }
  let particle = |w: &str| NAME_PARTICLES.contains(&w);
  if particle(words[0]) || particle(words[words.len() - 1]) {
    return false;
  }
  words.iter().all(|word| {
    if non_name_word(word) {
      return false;
    }
    particle(word)
      || (word.chars().next().is_some_and(char::is_uppercase)
        && word
          .chars()
          .all(|c| c.is_alphabetic() || "'’.-".contains(c)))
  })
}

/// A word of initials glued to the surname they precede, as its words: `A.G.Bogdanchikov` → `A.`, `G.`,
/// `Bogdanchikov`, `Yu.M.Shatunov` → `Yu.`, `M.`, `Shatunov` (hep-ex0105093's collaboration list; an initial is a
/// capital and at most one small letter, not a place's abbreviation: `St.Petersburg`, `Mt.Stromlo`); a word without a
/// surname after its initials (`U.S.A.`, `D.A.`, `Ph.D.`) or with none glued stays whole.
fn unglued_initials(word: &str) -> Vec<&str> {
  let mut parts = Vec::new();
  let mut rest = word;
  loop {
    let mut chars = rest.char_indices().peekable();
    let Some((_, capital)) = chars.next() else {
      break;
    };
    chars.next_if(|(_, c)| c.is_lowercase());
    match (chars.next(), chars.next()) {
      (Some((dot, '.')), Some((next, after)))
        if capital.is_uppercase()
          && after.is_uppercase()
          && !["St", "Mt", "Ft"].contains(&&rest[..dot]) =>
      {
        parts.push(&rest[..next]);
        rest = &rest[next..];
      },
      _ => break,
    }
  }
  if parts.is_empty() || !rest.chars().skip(1).any(char::is_lowercase) {
    return vec![word];
  }
  parts.push(rest);
  parts
}

/// Whether a name's last word is one its period belongs to: an initial (a single capital, `K`) or a suffix written with
/// a period (`Jr`, `Esq`, [`NAME_SUFFIXES`]). A two-letter word is a surname as often as an initial (`Wei Li.`, `Andrew
/// Ng.`: sentence punctuation), so its period goes, as Perl's.
fn ends_with_abbreviation(name: &str) -> bool {
  let Some(word) = name.split_whitespace().next_back() else {
    return false;
  };
  let mut chars = word.chars();
  let initial = chars.next().is_some_and(char::is_uppercase) && chars.next().is_none();
  initial || NAME_SUFFIXES.contains(&format!("{word}.").as_str())
}

/// The societies whose membership grade follows an author's name (`Senior Member, IEEE`).
const NAME_GRADE_SOCIETIES: &[&str] = &[
  "IEEE", "ACM", "OSA", "SPIE", "IET", "IEICE", "AAAI", "SIAM", "APS", "IAPR", "IFAC",
];
/// Generational suffixes and academic degrees written after a name, comma-separated (`John Smith, Jr.`, `Jane Doe,
/// MD, PhD`).
const NAME_SUFFIXES: &[&str] = &[
  "Jr", "Jr.", "Sr", "Sr.", "Jun.", "Sen.", "II", "III", "IV", "PhD", "Ph.D.", "Ph.D", "MD",
  "M.D.", "MSc", "M.Sc.", "BSc", "B.Sc.", "MA", "M.A.", "MS", "M.S.", "MBA", "MPH", "MBBS",
  "DPhil", "D.Phil.", "DSc", "D.Sc.", "FRS", "RN", "Esq.",
];

/// Whether an author-list part qualifies the name before it instead of naming a person: a generational suffix or a
/// degree ([`NAME_SUFFIXES`]), or a professional membership grade (`Member, IEEE`, `Senior Member`, or the society
/// alone once the list is split at the grade's comma), which LaTeX prints after the name, comma-separated.
fn is_name_qualifier(text: &str) -> bool {
  let text = strip_glued_marks(text);
  if NAME_SUFFIXES.contains(&text) {
    return true;
  }
  let words: Vec<&str> = text.split([' ', ',']).filter(|w| !w.is_empty()).collect();
  let grade = |w: &&str| {
    [
      "Life",
      "Senior",
      "Student",
      "Graduate",
      "Associate",
      "Member",
      "Fellow",
    ]
    .contains(w)
  };
  !words.is_empty()
    && words
      .iter()
      .all(|w| grade(w) || NAME_GRADE_SOCIETIES.contains(w))
}

/// The parts of an author-line text that may each be one name: split at commas, " and " and " & ", a leading "and"
/// dropped.
fn name_parts(text: &str) -> Vec<String> {
  text
    .split(',')
    .flat_map(|part| {
      part
        .split(" and ")
        .flat_map(|p| p.split(" & "))
        .map(str::to_string)
        .collect::<Vec<_>>()
    })
    .map(|part| {
      let part = part.trim();
      part
        .strip_prefix("and ")
        .or_else(|| part.strip_prefix("& "))
        .unwrap_or(part)
        .trim()
        .to_string()
    })
    .filter(|part| !part.is_empty())
    .collect()
}

/// How many parts of `text` are names when every part is a name or qualifies one ([`is_name_qualifier`]); 0 when a
/// part is neither (an institution, a place).
fn name_count(text: &str) -> usize {
  let parts = name_parts(text);
  if parts.iter().all(|p| name_shaped(p) || is_name_qualifier(p)) {
    parts.iter().filter(|p| name_shaped(p)).count()
  } else {
    0
  }
}

/// Whether `line` continues the author names `names` instead of starting their affiliations: styled as the names are,
/// all of it names (or their qualifiers), and either the names read as an unfinished list — ending in a comma, "and"
/// or "&" (`Joshua N.\ Winn\altaffilmark{1}, Andrew W.\ Howard\altaffilmark{2},\\ Avi Shporer\altaffilmark{1}`,
/// 1010.1318) — or the line opens with "and" (`…, \emph{Member, IEEE},\\ and Warren J. Gross, \emph{Senior Member,
/// IEEE}`, 1611.04834). "and Max--Planck Institut für Mathematik, Bonn" (math0208081) and `John Smith,\\ {\it Bell
/// Labs, Murray Hill}` stay affiliations.
fn names_continue(names: &Tokens, line: &Tokens) -> bool {
  if leading_markup(line) != leading_markup(names) {
    return false;
  }
  let names_text = visible_name_text(names.unlist_ref());
  let names_text = names_text.trim_end();
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim();
  let unfinished =
    names_text.ends_with(',') || names_text.ends_with(" and") || names_text.ends_with('&');
  (unfinished || text.starts_with("and ") || text.starts_with("& ")) && name_count(text) > 0
}

/// Whether `line` is more names under the names `names`, not their affiliation: styled as they are, all of it names
/// (or their qualifiers), and every name of both carrying its mark after it, as authors' marks stand (`Ann
/// Able$^{1}$\\ Bob Baker$^{2}$`; `A\altaffilmark{1}, B\altaffilmark{2}\\ C\altaffilmark{3}`; [`marked_after_name`]).
fn names_marked_alike(names: &Tokens, line: &Tokens) -> bool {
  leading_markup(line) == leading_markup(names)
    && name_count(visible_name_text(line.unlist_ref()).trim()) > 0
    && [names, line].into_iter().all(|tokens| {
      split_author_line(tokens.clone())
        .iter()
        .all(marked_after_name)
    })
}

/// Whether a name carries a mark after it, outside its notes: a superscript ([`affiliation_mark_tokens`]) or a mark
/// command (aastex's `\altaffilmark`, IEEEtran's `\IEEEauthorrefmark`) after its first letter (`Bob Baker$^{2}$`; not
/// `$^{2}$Univ B`).
fn marked_after_name(name: &Tokens) -> bool {
  let outside = outside_notes(name.unlist_ref());
  let Some(first_letter) = outside
    .iter()
    .position(|t| t.get_catcode() == Catcode::LETTER)
  else {
    return false;
  };
  outside[first_letter..].iter().any(|t| {
    affiliation_mark_tokens().contains(t)
      || *t == T_CS!("\\altaffilmark")
      || *t == T_CS!("\\IEEEauthorrefmark")
  })
}

/// The markup an author line opens with, before its first letter: its control sequences and the groups they open
/// (`{\small\em Bartol …}` → `{`, `\small`, `\em`; `T. Stelzer` → none). Lines styled alike share it.
fn leading_markup(line: &Tokens) -> Vec<Token> {
  line
    .unlist_ref()
    .iter()
    .take_while(|t| t.get_catcode() != Catcode::LETTER)
    .filter(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::BEGIN))
    .copied()
    .collect()
}

/// The `\\`-lines of an author group without affiliation marks, as (names, affiliations) pairs. The first line is
/// the names and the lines after it their affiliations (Perl Base_Utility.pool.ltxml:720-725), except that
///  - a line that is only "and" (or `\&`) separates two pairs (cond-mat9705101's `\\ \\ and\\ \\`);
///  - a line continues the names while they read as an unfinished list ([`names_continue`]), and a line of names
///    marked as the names above are opens the next pair ([`names_marked_alike`]);
///  - after an affiliation styled unlike the names, a line of names styled as the names were opens the next pair when
///    an affiliation styled as that one follows it — the alternation (hep-ph9306253's `T. Stelzer` between `{\small
///    \em …}` affiliations; 0811.1526's `Ingo Rehberg and Reinhard Richter` between `{\sl …}` ones). A last line
///    (a city: `Los Angeles, CA 90095, USA`) or an affiliation styled as the names ("University of X" under a plain
///    name) stays an affiliation.
fn name_groups(lines: Vec<Tokens>) -> Vec<(Tokens, Vec<Tokens>)> {
  let mut groups: Vec<(Tokens, Vec<Tokens>)> = Vec::new();
  let mut names_markup: Vec<Token> = Vec::new();
  let mut opens_pair = true;
  let mut pending_and: Option<Tokens> = None;
  for (index, line) in lines.iter().enumerate() {
    let text = visible_name_text(line.unlist_ref());
    let text = text.trim();
    if text == "and" || text == "&" {
      // a second "and" in a row: the first is affiliation text (one before any names separates nothing)
      if let (Some(earlier), Some((_, affiliations))) = (pending_and.take(), groups.last_mut()) {
        affiliations.push(earlier);
      }
      pending_and = Some(line.clone());
      continue;
    }
    // A line of only "and" parts two authors when names follow it (cond-mat9705101's `\\ \\ and\\ \\ {\it P.
    // Plech\' a\v{c}}`); before anything else it is affiliation text, kept with the line after it (`Dept.\ of
    // Physics\\ and\\ Center for Theoretical Physics`; `and\\ SISSA, Trieste`), as Perl keeps both.
    if let Some(and_line) = pending_and.take() {
      if groups.is_empty() || name_count(text) > 0 {
        opens_pair = true;
      } else if let Some((_, affiliations)) = groups.last_mut() {
        affiliations.push(and_line);
        affiliations.push(line.clone());
        continue;
      }
    }
    if opens_pair || groups.is_empty() {
      names_markup = leading_markup(line);
      groups.push((line.clone(), Vec::new()));
      opens_pair = false;
      continue;
    }
    let (names, affiliations) = groups.last_mut().expect("a pair is open");
    let alternates = affiliations.last().is_some_and(|last| {
      let affiliation_markup = leading_markup(last);
      affiliation_markup != names_markup
        && lines
          .get(index + 1)
          .is_some_and(|next| leading_markup(next) == affiliation_markup)
    });
    let and_led = text
      .strip_prefix("and ")
      .or_else(|| text.strip_prefix("& "));
    if affiliations.is_empty() && names_continue(names, line) {
      let mut joined = names.clone().unlist();
      joined.push(T_SPACE!());
      joined.extend(line.clone().unlist());
      *names = Tokens::new(joined);
    } else if affiliations.is_empty() && names_marked_alike(names, line) {
      // the next names, the lines after them their affiliations as after any names
      groups.push((line.clone(), Vec::new()));
    } else if let Some(names_line) = and_led
      .filter(|rest| leading_markup(line) == names_markup && name_count(rest) > 0)
      .and_then(|_| without_leading_and(line))
    {
      // after affiliations, "and" before names opens the next author (`James Brink \\Dept of Math., UC Berkeley,
      // …\\and Zhenghan Wang \\Dept of Math., Indiana University`, math-ph0303018), its "and" dropped
      groups.push((names_line, Vec::new()));
    } else if alternates && leading_markup(line) == names_markup && name_count(text) > 0 {
      groups.push((line.clone(), Vec::new()));
    } else {
      affiliations.push(line.clone());
    }
  }
  if groups.is_empty() {
    // a block of `\\` alone still gives its (empty) author
    groups.push((Tokens::default(), Vec::new()));
  }
  // a closing "and" line is affiliation text
  if let (Some(and_line), Some((_, affiliations))) = (pending_and, groups.last_mut()) {
    affiliations.push(and_line);
  }
  groups
}

/// `line` without the word "and" its text opens with — after its leading markup, which is kept (`{\it and Zhenghan
/// Wang}` → `{\it Zhenghan Wang}`) — and the spaces after it; `None` when its first word is not "and".
fn without_leading_and(line: &Tokens) -> Option<Tokens> {
  let v = line.unlist_ref();
  let first = v.iter().position(|t| t.get_catcode() == Catcode::LETTER)?;
  if v.get(first..first + 3) != Some(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")][..])
    || v
      .get(first + 3)
      .is_some_and(|t| t.get_catcode() == Catcode::LETTER)
  {
    return None;
  }
  let mut rest = first + 3;
  while v.get(rest) == Some(&T_SPACE!()) {
    rest += 1;
  }
  let mut out = v[..first].to_vec();
  out.extend_from_slice(&v[rest..]);
  Some(Tokens::new(out))
}

/// A name-list piece that qualifies the name before it ([`is_name_qualifier`]: `Jr.`, `PhD`, `\emph{Member, IEEE}`)
/// rejoins that name with the separator written between them, instead of becoming an author of its own; so does a
/// piece of marks and notes only ([`only_marks_and_notes`]: `~\IEEEmembership{Senior Member,~IEEE,}\thanks{…}`,
/// 1510.02728), without its separator, which separated nothing visible.
fn rejoin_name_qualifiers(pieces: Vec<(Vec<Token>, Tokens)>) -> Vec<Tokens> {
  let mut out: Vec<Tokens> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let unseen = only_marks_and_notes(piece.unlist_ref());
    match out.last_mut() {
      Some(name) if unseen || is_name_qualifier(&visible_name_text(piece.unlist_ref())) => {
        let mut joined = name.clone().unlist();
        if !unseen {
          // the comma's space went with the piece's trimming
          let comma = delimiter.last() == Some(&T_OTHER!(","));
          joined.extend(delimiter);
          if comma {
            joined.push(T_SPACE!());
          }
        }
        joined.extend(piece.unlist());
        *name = Tokens::new(joined);
      },
      _ => out.push(piece),
    }
  }
  out
}

/// Whether `tokens` hold nothing to read but marks and notes: spaces (`~`, `\ `, `\,`), a note or mark command with
/// its arguments ([`NAME_ANNOTATIONS`], `\footnotemark[…]`), a superscript or subscript, or math holding only those
/// (`$^{*}$`). An author written as a macro (`\A`) is something to read.
fn only_marks_and_notes(tokens: &[Token]) -> bool {
  let mut i = 0;
  let mut any = false;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::SPACE => i += 1,
      Catcode::ACTIVE if t.with_str(|s| s == "~") => i += 1,
      Catcode::CS if t == T_CS!("\\ ") || t == T_CS!("\\,") => i += 1,
      // `${}^{1}$`'s empty group
      Catcode::BEGIN
        if tokens
          .get(i + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::END) =>
      {
        i += 2
      },
      Catcode::CS
        if t == T_CS!("\\footnotemark") || t.with_str(|s| NAME_ANNOTATIONS.contains(&s)) =>
      {
        any = true;
        i += 1;
        if tokens.get(i) == Some(&T_OTHER!("[")) {
          while i < tokens.len() && tokens[i] != T_OTHER!("]") {
            i += 1;
          }
          i += 1;
        }
        if t != T_CS!("\\footnotemark") {
          i = skip_group(tokens, i);
        }
      },
      Catcode::SUPER | Catcode::SUB => {
        any = true;
        i = skip_group(tokens, i + 1);
      },
      Catcode::MATH => {
        let Some(close) = tokens[i + 1..]
          .iter()
          .position(|t| t.get_catcode() == Catcode::MATH)
        else {
          return false;
        };
        if !only_marks_and_notes(&tokens[i + 1..i + 1 + close])
          || tokens[i + 1..i + 1 + close].is_empty()
        {
          return false;
        }
        any = true;
        i += close + 2;
      },
      _ => return false,
    }
  }
  any
}

/// The text a `<ltx:personname>` shows, without its marks and notes (`ltx:sup`, `ltx:note`, math); an element
/// (a `<ltx:break/>`, a cell) parts the words around it.
fn personname_text(node: &Node) -> String {
  let mut text = String::new();
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) => text.push_str(&child.get_content()),
      Some(NodeType::ElementNode) => {
        text.push(' ');
        if !with(document::get_node_qname(&child), |q| {
          q == "ltx:sup" || q == "ltx:note" || q == "ltx:Math" || q == "ltx:contact"
        }) {
          text.push_str(&personname_text(&child));
        }
        text.push(' ');
      },
      _ => {},
    }
  }
  text
}

/// A frontmatter author whose name reads as two or more people ("Ann Able and Bob Baker") lost the boundaries between
/// them: LaTeX typesets several authors where the document has one, and nothing else tells. Each such author is an
/// error, once (the creator is marked, for the frontmatter a streaming conversion places late). Every part must be a
/// name or qualify one (`Jane Doe, PhD`): a name beside its institution ("John Smith, Bell Labs") is not two people.
fn flag_merged_creators(document: &mut Document) -> Result<()> {
  for mut creator in
    document.findnodes("//ltx:creator[@role='author'][not(@_merged_checked)]", None)
  {
    document.set_attribute(&mut creator, "_merged_checked", "1")?;
    // An author block laid out as a tabular is kept whole on purpose (`add_authors_calls`; Perl
    // Base_Utility.pool.ltxml:693): its names are there, in their layout.
    let Some(person) = document.findnode("ltx:personname[not(.//ltx:tabular)]", Some(&creator))
    else {
      continue;
    };
    let text = personname_text(&person);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let names = name_count(&text);
    if names >= 2 {
      Error!(
        "frontmatter",
        "merged_creators",
        s!(
          "One author holds {names} names: \"{text}\"; the separators between them were not recognized"
        )
      );
    }
  }
  Ok(())
}

/// A `\\`-delimited line in a no-marker author block is really an EMAIL, not an
/// affiliation, when its visible text is a single bare address: it contains `@`
/// and NO whitespace. Institution names always contain spaces, so this is
/// conservative by construction (near-zero false positives). Relabels the
/// trailing `\texttt{user@host}` line of the NeurIPS idiom, which would
/// otherwise become a bogus `role="affiliation"` (OXIDIZED_DESIGN #52; witness
/// arXiv:2606.00315). Visible text = letters/other chars only; control sequences
/// (`\texttt`, `\textit`, `\{`, `\}`) and grouping are skipped.
fn line_is_email(line: &Tokens) -> bool {
  let mut visible = String::new();
  for t in line.unlist_ref() {
    if t.code == Catcode::LETTER || t.code == Catcode::OTHER {
      t.with_str(|s| visible.push_str(s));
    } else if t.code == Catcode::SPACE {
      visible.push(' ');
    }
  }
  let v = visible.trim();
  !v.is_empty() && v.contains('@') && !v.chars().any(|c| c.is_whitespace())
}

/// A marker-less line that is *purely* a list of email addresses — a shared
/// `\texttt{a@x, b@y,}` / `\email{...}` line covering all authors. Like
/// [`line_is_email`] but tolerates a comma-separated list (and the trailing
/// comma authors often leave): every non-empty comma item must itself carry '@'.
/// A prose affiliation line ("Dept. of Foo, University of Pisa, Italy") has no
/// '@' and is rejected, so this never misclassifies an address as an email.
fn line_is_email_list(line: &Tokens) -> bool { email_addresses(line).is_some() }

/// Parse an email line into its individual address strings, or `None` if the line
/// is not an email list. Two forms are recognized (both after skipping wrapper
/// commands / braces — only visible letter/other/space chars are read):
/// - **Distributed** — `a@x, b@y, c@z`: every comma item carries its own `@`.
/// - **Grouped brace-expansion** — `{a,b,c}@dom` (flattens to `a,b,c@dom`): only
///   the LAST item carries `@`; the earlier bare local-parts are each expanded
///   against the shared domain → `a@dom, b@dom, c@dom`. This is the compact form
///   co-located authors use for one shared address per person.
///
/// A prose affiliation line ("Dept. of Foo, University of Pisa, Italy") has no `@`
/// and is rejected, so an address is never misclassified.
fn email_addresses(line: &Tokens) -> Option<Vec<String>> {
  let mut visible = String::new();
  // `\href`'s first argument is its link, not printed (`\href{mailto:a@x}{a@x}`).
  let mut skip_group = false;
  let mut depth = 0usize;
  for t in line.unlist_ref() {
    if skip_group {
      match t.code {
        Catcode::SPACE if depth == 0 => {},
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth = depth.saturating_sub(1);
          skip_group = depth > 0;
        },
        _ if depth == 0 => skip_group = false,
        _ => {},
      }
      continue;
    }
    match t.code {
      Catcode::LETTER | Catcode::OTHER => t.with_str(|s| visible.push_str(s)),
      Catcode::SPACE => visible.push(' '),
      Catcode::CS if *t == T_CS!("\\href") => skip_group = true,
      _ => {},
    }
  }
  let v = visible.trim();
  if !v.contains('@') {
    return None;
  }
  let parts: Vec<&str> = v
    .split(',')
    .map(|p| p.trim().trim_start_matches("mailto:"))
    .filter(|p| !p.is_empty())
    .collect();
  if parts.is_empty() {
    return None;
  }
  let has_at = |p: &str| p.contains('@') && !p.chars().any(char::is_whitespace);
  // Distributed: every item is a full address.
  if parts.iter().all(|p| has_at(p)) {
    return Some(parts.iter().map(|s| (*s).to_string()).collect());
  }
  // Grouped: only the last item carries the domain; earlier items are bare
  // local-parts expanded against it (`{a,b,c}@dom` → a@dom, b@dom, c@dom).
  if let Some((last, heads)) = parts.split_last()
    && has_at(last)
    && heads
      .iter()
      .all(|p| !p.contains('@') && !p.chars().any(char::is_whitespace))
    && let Some(at) = last.find('@')
  {
    let domain = &last[at..];
    let mut out: Vec<String> = heads.iter().map(|p| format!("{p}{domain}")).collect();
    out.push((*last).to_string());
    return Some(out);
  }
  None
}

/// Line role within a superscript-labeled author block (see `\lx@add@authors`).
#[derive(Clone, Copy, PartialEq)]
enum AuthorLineKind {
  Author,
  Affiliation,
  Email,
}

/// Converts tokens to a string in the fashion of \message and others
///
/// doubles #, converts to string; optionally adds spaces after control sequences
/// in the spirit of the B Book, "show_token_list" routine, in 292.
/// [This could be a $tokens->unpackParameters, but for the curious space treatment]
pub fn writable_tokens(tokens: &Tokens) -> String {
  // Control sequences are written with the CURRENT `\escapechar`
  // (tex.web §1594 print_esc / eTeX: `\scantokens` writes as `\write`
  // would) — NOT a hardcoded backslash. xint's `\XINT_NewExpr` capture
  // (xintexpr.sty L4713-4740) relies on this: stage 1 sets
  // `\escapechar 126` with active `~` so re-scanned CS names become
  // inert `$noexpand$name` text; our old hardcoded `\` let `\the`/
  // `\romannumeral` EXECUTE inside the capturing edef, corrupting every
  // user-defined xint function body (tkz-grapheur ×4 recursion fatals,
  // the roman-numeral csname monster). Perl shares the hardcode
  // (TeX_Debugging.pool L52) and dies the same way — pdflatex is the
  // oracle. `escapechar()` reads the live register per call: xint flips
  // it between capture stages.
  let esc = escapechar();
  let mut out = String::new();
  for t in tokens.unlist_ref().iter() {
    match t.code {
      Catcode::CS => {
        with(t.text, |s| {
          let name = s.strip_prefix('\\').unwrap_or(s);
          out.push_str(&esc);
          out.push_str(name);
          // tex.web §262 print_cs: multi-letter names get a trailing
          // space; a single-char name gets one iff that char's CURRENT
          // catcode is LETTER.
          let mut chars = name.chars();
          let first = chars.next();
          if chars.next().is_some() {
            out.push(' ');
          } else if let Some(c) = first
            && lookup_catcode(c) == Some(Catcode::LETTER)
          {
            out.push(' ');
          }
        });
      },
      Catcode::SPACE => out.push(' '),
      Catcode::PARAM => t.with_str(|ts| {
        out.push_str(ts);
        out.push_str(ts);
      }),
      Catcode::ARG => {
        // B Book, 294. Reduce to param+integer
        out.push('#');
        t.with_str(|ts| out.push_str(ts));
      },
      _ => t.with_str(|ts| out.push_str(ts)),
    }
  }
  out
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
// keyset at once. These can be specified by giving comma-separated values in
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
// object. This can be achieved using comma-separated key values in
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

/// Use tex_glue::dimension_to_spaces for the precise Perl-matching algorithm.
/// This wrapper delegates to avoid breaking callers that import from here.
///
/// `Dimension::new` takes raw scaled-points (sp). `value_of()` returns the
/// canonical i64 (sp for Dimension/Glue, units for Number). The previous
/// implementation routed through `pt_value(None)` (which divides by UNITY)
/// and then `new_f64` (which does NOT multiply back by UNITY), losing a
/// factor of 65536 — so 2em (1310720 sp) became Dimension(20 sp) ≈ 0pt and
/// `tex_glue::dimension_to_spaces` produced an empty string. That made
/// `\hspace`'s `isSpace` Tbox skip the `if !s.is_empty()` gate at the
/// caller, dropping the math-mode space marker between `^{...}` and a
/// following `'` — surfacing as the false `unexpected:double-superscript`
/// in hep-th9601176 (`\Si^{\mu\nu}\hs{0.25}'(p)`).
///
/// Both callers (`\hglue`, `\hspace`) are `DefPrimitive`s, i.e. DIGEST time,
/// where `lookup_font()` already IS the font in effect — so the ambient read
/// is the right one and no font is threaded through.
pub fn dimension_to_spaces<T: NumericOps>(dimen: T) -> Cow<'static, str> {
  let dim = Dimension::new(dimen.value_of());
  Cow::Owned(super::tex_glue::dimension_to_spaces(dim, None))
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
  if qname == pin!("ltx:tag") {
  }
  // HACK
  else if !align.is_empty() && model::can_have_attribute(qname, pin!("align")) {
    node.set_attribute("align", align)?;
  } else if !class.is_empty() && model::can_have_attribute(qname, pin!("class")) {
    document.add_class(node, class)?;
  }
  Ok(())
}

/// Remove `\@spaces`/`\space` padding control sequences from a message
/// prefix, mirroring Perl `make_message`'s `s/(?:\\\@?spaces?)+//g`
/// (latex_constructs.pool.ltxml L5572). Longest-first so `\@spaces` is
/// never left as a stray `s`.
fn strip_message_spaces(s: &str) -> String {
  s.replace("\\@spaces", "")
    .replace("\\spaces", "")
    .replace("\\@space", "")
    .replace("\\space", "")
}

pub fn make_generic_message(cmd: &str, args: Vec<Tokens>, kind: &str) -> Result<()> {
  // Faithful port of Perl latex_constructs.pool.ltxml `make_message`
  // (L5569-5588). The first arg is the message PREFIX: its ToString,
  // stripped of `\@spaces`/`\space` runs, becomes the diagnostic `what`
  // ($type), falling back to the command name when the prefix is empty
  // (e.g. `\@latex@error`'s all-`\space` prefix). The REMAINING args form
  // the body — Perl `join(" ", map { ToString(Expand($_, \@spaces,
  // \@spaces)) } @args)`. Earlier the Rust port discarded the prefix and
  // pinned `what` to the command name, so package/class diagnostics lost
  // their `(pkgname)` $type and `\GenericWarning`/`\GenericInfo` even
  // leaked the prefix into the body. Witness: hep-th-style `\PackageError`
  // /`\GenericWarning` now match Perl byte-for-byte (`(mypkg)` $type, no
  // prefix in body).
  let mut args = args.into_iter();
  let lead = args.next().unwrap_or_default();
  // $type = ToString(lead) with `\@?spaces?` runs removed (Perl regex
  // `s/(?:\\\@?spaces?)+//g`); fall back to the command name when empty.
  let type_str = strip_message_spaces(&lead.to_string());
  let type_str: &str = if type_str.is_empty() { cmd } else { &type_str };

  bgroup();
  let_i(&T_CS!("\\protect"), &T_CS!("\\string"), None);
  let_i(
    &T_CS!("\\MessageBreak"),
    &T_CS!("\\ltx@hard@MessageBreak"),
    None,
  ); // tricky, we need Expand() to execute it
  let mut parts: Vec<String> = Vec::new();
  for arg in args {
    let mut arg_toks = arg.unlist();
    // Perl `make_message` pads each Expand input with two `\@spaces` tokens:
    //   ToString(Expand($_, T_CS('\@spaces'), T_CS('\@spaces')))
    // Comment in Perl: "expand padded with two \@spaces, to avoid pointless
    // errors. e.g. a trailing `\csq@noline` from csquotes.sty (let to
    // `\@gobble`) will produce a `gobble has no argument` error on every
    // message." Same fix needed here for etoc.sty's
    // `\PackageWarning{etoc}{...!\@gobbletwo}` — `\@gobbletwo` would otherwise
    // consume the do_expand-inserted T_END closer, leaving readBalanced
    // unbalanced.
    arg_toks.push(T_CS!("\\@spaces"));
    arg_toks.push(T_CS!("\\@spaces"));
    // Written as `\write`/`\message` write it (tex.web §262 print_cs): a control word is
    // followed by a space, so `\PackageWarning{p}{B \noexpand\foo c}` reads "B \foo c". Perl's
    // `ToString` (latex_constructs.pool.ltxml:5586-5587) runs it together ("B \fooc"; KPE #357).
    // Byte-mouth runs are decoded here (as `Tokens`' Display does), not in `writable_tokens`,
    // which also feeds `\scantokens`/`\detokenize` that need the byte spellings.
    // Expanded as `\immediate\write` expands (latex.ltx:8773-8799, `\GenericInfo`/`\GenericWarning`/
    // `\GenericError` print through it), i.e. as `\edef` and `\message`: what `\the`/`\unexpanded`
    // yield is not expanded again, so `\PackageWarning{p}{\unexpanded{\foo}}` prints "\foo" and
    // runs nothing. Perl `Expand`s fully (latex_constructs.pool.ltxml:5586-5587), executing an
    // undefined `\foo` there. Guard `perfect_kernel_gemini::package_warning_keeps_unexpanded_text`.
    let expanded = do_expand_partially(arg_toks)?;
    parts.push(mouth::decode_byte_mouth_runs(&writable_tokens(&expanded)).into_owned());
  }
  // Perl joins the body args with a single space (the `\MessageBreak`s
  // *within* an arg already became hard newlines via the let above).
  let message = parts.join(" ");

  egroup()?;
  // Downgrade vendor-class typesetting-only errors to Info. Publisher
  // classes routinely guard line widths, header heights, and other
  // PDF-layout concerns with `\PackageError`/`\GenericError`. We
  // produce XML/HTML, not PDF — these guards have no semantic value
  // in our output. See WISDOM #50 and
  // memory/feedback_size_layout_errors_moot.md.
  let effective_kind = if kind == "error" && is_typesetting_only_message(&message) {
    "info"
  } else {
    kind
  };
  //   return ('latex', $type, $stomach, $message);
  match effective_kind {
    "error" => {
      Error!("latex", type_str, message);
    },
    "warn" => {
      Warn!("latex", type_str, message);
    },
    "info" => {
      Info!("latex", type_str, message);
    },
    _other => panic!("Only call make_generic_message with error|warn|info message kinds."),
  };
  Ok(())
}

/// Heuristic classifier for vendor `\PackageError`/`\GenericError`
/// messages whose only concern is PDF typesetting (size, layout,
/// position, page-fit). These have no signal in XML/HTML output and
/// are downgraded to `Info:` per WISDOM #50.
fn is_typesetting_only_message(message: &str) -> bool {
  let lower = message.to_ascii_lowercase();
  // Phrase set tuned against the stage-1 sweep of the 100k warning
  // corpus. Conservative — every phrase here is purely about visual
  // layout or vendor-deprecation chatter, never about semantic
  // correctness. Examples:
  //   "Running heading author exceeds size limitations" (AISTATS)
  //   "Running heading title exceeds size limitations" (AISTATS)
  //   "Caption too wide for page" (various)
  //   "Heading breaks the line" (revtex, IEEEtran)
  //   "You are loading directly a language style" (babel: czech.sty,
  //     francais.sty, etc. unconditionally fire `\PackageError` to nag
  //     the user toward `\usepackage[<lang>]{babel}`; pdflatex shows
  //     the message but continues, and the document typesets normally
  //     — the message is informational, not a real failure)
  const PHRASES: &[&str] = &[
    "exceeds size limitations",
    "exceeds size limitation",
    "running heading",
    "running title",
    "running author",
    "breaks the line",
    "too wide for",
    "too tall for",
    "too long for",
    "too narrow for",
    "doesn't fit",
    "does not fit",
    "page overflow",
    "column overflow",
    "exceeds the page",
    "exceeds the column",
    "exceeds the line",
    "exceeds the textwidth",
    "exceeds \\textwidth",
    "exceeds \\columnwidth",
    "exceeds \\linewidth",
    "loading directly a language style",
    "syntax is deprecated",
    // babel TL2025: legacy `<lang>.ldf` files have been retired in
    // favour of `locale/<iso>/babel-<lang>.tex` (the ini-file
    // system). For papers that load `\usepackage[<lang>]{babel}`
    // without `provide=*`, babel fires:
    //   Package babel Error: Unknown option '<lang>'.
    //   Either you misspelled it or the language definition file
    //   <lang>.ldf was not found
    // The .ldf-missing case is benign: pdflatex shows the message
    // and proceeds — the document still typesets, just without
    // the language's captions/shorthands loaded. Same effective
    // outcome on our side: downgrade to Info, conversion continues.
    // Surpass-Perl: Perl raw-loads babel.sty and errors identically
    // on the same 58 papers in Round-27 Cluster D. This downgrade
    // closes the cluster (cannot reach 0-error AND load the proper
    // ini file without redesigning babel option processing).
    "either you misspelled it",
    // catoptions.sty (loaded transitively by many class/sty bundles)
    // calls `\@latex@error{Command \protect\\special_relax already
    // defined...}` because it `\def\special_relax{...}` and our engine
    // pre-registers `\special_relax` as an internal Gullet helper.
    // pdflatex shows the message and proceeds (cat's def takes over);
    // surpass-Perl: Perl also raw-loads catoptions and errors. The
    // resulting "Command ... already defined" cascade currently
    // produces 100+ errors per paper on 14 wp5 papers. Downgrade so
    // conversion continues. Same applies to "Command \end... illegal,
    // see p.192 of the manual" tail that LaTeX appends to the same
    // message ("\@latex@error" generic-error template).
    "already defined. or name",
    "command \\end... illegal",
    // amsfonts-not-installed: aims-class L: `\@latex@error{Package
    // `amsfonts' not installed, or version too old?}`. The class ships
    // its own font tables and only loads amsfonts as an enhancement;
    // pdflatex/Perl LaTeXML both proceed to typeset successfully when
    // amsfonts is missing. Our raw-load reports the message verbatim
    // but conversion is fine without the AMS msam/msbm fonts. Witness
    // 2202.13120.
    "amsfonts package will not be loaded",
    "package `amsfonts'",
    "package 'amsfonts'",
    // hrefhide-format-too-old: hrefhide.sty requires LaTeX format
    // 2022-11-01+; older formats trigger an error. The package is
    // strictly visual (hides hyperref colors on print), so the error
    // is moot for XML/HTML output. Witness 2202.03936.
    "newer latex format needed",
    "older hrefhide package",
  ];
  PHRASES.iter().any(|p| lower.contains(p))
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
  match document.get_element() {
    Some(context) => document::with_node_qname(&context, |qname| qname.starts_with("svg:")),
    _ => false,
  }
}

pub fn adjust_box_color(tbox: &Digested) -> Result<()> {
  use latexml_core::common::color;
  let color_opt = lookup_font().and_then(|f| f.get_color().cloned());
  if let Some(color) = color_opt
    && color != color::BLACK
  {
    let hex = color.to_attribute();
    adjust_box_color_rec(&hex, HashMap::default(), tbox);
  }
  Ok(())
}

fn adjust_box_color_rec(_color: &str, _props: HashMap<String, String>, _tbox: &Digested) {
  // Perl: adjustBoxColor recursively propagates color through box tree.
  // Currently a stub — color propagation is not yet critical for test passage.
}

// Hmm... I wonder, should getString itself be dealing with escapechar?
pub fn escapechar() -> String {
  let code: i64 = match lookup_register("\\escapechar", Vec::new()).unwrap() {
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

/// Whether `cs` is defined as a name noun (an expandable macro whose arguments are all optional, `is_name_noun`):
/// `\figurename`, not a drawing command that only shares the `\<counter>name` spelling (argumentation.sty:403
/// `\afname`).
pub fn is_name_noun_cs(cs: &str) -> bool {
  matches!(lookup_definition(&T_CS!(cs)), Ok(Some(def)) if is_name_noun(&def, 0))
}

/// Is this definition a NAME NOUN (`\figurename`): an expandable macro whose arguments are
/// all optional? ltcmd's `\NewDocumentCommand` defines `\foo` as the protected,
/// parameterless dispatcher `\__cmd_start_optimized: \foo code` (latex.ltx:1963-1972),
/// so follow that one hop to `\foo code` (zero-arg xparse nouns qualify, `m`-taking
/// drawing commands like argumentation.sty:403 `\afname` do not); the general
/// `\__cmd_start:nNNnnn` / `\__cmd_start_expandable:nNNNNn` dispatchers carry the
/// signature as written, which is read; a `\DeclareRobustCommand` wrapper is followed to
/// its inner macro.
fn is_name_noun(def: &Rc<dyn Definition>, depth: u8) -> bool {
  // A name is set before `~<number>\null` (hyperref's autoref) or in a tag's composition: optional arguments find
  // none and take their defaults, a required one would swallow what follows.
  if !def.is_expandable()
    || def
      .get_parameters()
      .is_some_and(|p| p.get_parameters().iter().any(|param| !param.optional))
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = def.get_expansion() else {
    return true;
  };
  let toks = body.clone().unlist();
  let Some(first) = toks.first() else {
    return true;
  };
  if first.get_catcode() != Catcode::CS {
    return true;
  }
  let head = first.to_string();
  // `\DeclareRobustCommand`'s wrapper `\protect \<cs>␣`: the name is the inner macro.
  if head == "\\protect" && toks.len() == 2 {
    return depth == 0
      && matches!(lookup_definition(&toks[1]), Ok(Some(inner)) if is_name_noun(&inner, 1));
  }
  if head == "\\__cmd_start_optimized:" {
    return depth == 0
      && toks.get(1).is_some_and(
        |code| matches!(lookup_definition(code), Ok(Some(code_def)) if is_name_noun(&code_def, 1)),
      );
  }
  if head == "\\__cmd_start:nNNnnn" || head == "\\__cmd_start_expandable:nNNNNn" {
    // ltcmd's general dispatcher carries the signature first: a name takes only optional arguments (`o`, `O{…}`,
    // `d`, `s`, `t`, `e`); `m`, `r`, `R`, `v`, `b`, `l`, `u` are required.
    return xparse_signature_is_optional(&toks[1..]);
  }
  true
}

/// Whether an ltcmd signature — the braced group at the start of `toks`, as written — has only optional argument
/// types: `m r R v b l u` are required; `o s g` optional, `t` with its token, `d` with two, `D` two and a default,
/// `O G` a default, `e` its tokens, `E` its tokens and defaults; `+ !` modify, `>` and `=` take a group. A type's
/// delimiters and defaults are skipped, so `t m` (an optional `m` token) or `O{with m}` stay optional; a braced type
/// is taken as required.
fn xparse_signature_is_optional(toks: &[Token]) -> bool {
  if toks
    .first()
    .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
  {
    return false;
  }
  // The signature's top-level items, a braced group counted as one.
  let mut items: Vec<Option<String>> = Vec::new();
  let mut depth = 0;
  for token in &toks[1..] {
    match token.get_catcode() {
      Catcode::BEGIN => {
        if depth == 0 {
          items.push(None);
        }
        depth += 1;
      },
      Catcode::END if depth == 0 => break,
      Catcode::END => depth -= 1,
      Catcode::SPACE if depth == 0 => {},
      _ if depth == 0 => items.push(Some(token.to_string())),
      _ => {},
    }
  }
  let mut items = items.into_iter();
  while let Some(item) = items.next() {
    let skip = match item.as_deref() {
      Some("m" | "r" | "R" | "v" | "b" | "l" | "u") => return false,
      Some("o" | "s" | "g" | "+" | "!") => 0,
      // A braced type (`{m}`): ltcmd reads its contents as the type (latex.ltx:2291); taken as required.
      None => return false,
      Some("t" | "O" | "G" | "e" | ">" | "=") => 1,
      Some("d" | "E") => 2,
      Some("D") => 3,
      Some(_) => return false,
    };
    for _ in 0..skip {
      items.next();
    }
  }
  true
}

#[cfg(test)]
mod author_split_tests {
  use latexml_core::state::{State, StateOptions, set_state};

  use super::*;

  fn setup() {
    // T_CS!/T_BEGIN! etc. and tokenize_internal need a thread-local State for
    // string interning.
    set_state(State::new(StateOptions::default()));
  }
  fn tk(s: &'static str) -> Tokens { mouth::tokenize_internal(s) }

  #[test]
  fn whole_line_cs_wrapper_detects_font_wrapper() {
    setup();
    let (cmd, inner) = whole_line_cs_wrapper(&tk("\\textbf{A, B}")).expect("is a wrapper");
    assert_eq!(cmd, T_CS!("\\textbf"));
    assert!(!inner.is_empty());
  }

  #[test]
  fn whole_line_cs_wrapper_rejects_non_wrappers() {
    setup();
    // No leading control sequence.
    assert!(whole_line_cs_wrapper(&tk("A, B")).is_none());
    // Content trails the group -> not a WHOLE-line wrapper.
    assert!(whole_line_cs_wrapper(&tk("\\textbf{A}x")).is_none());
  }

  #[test]
  fn split_author_line_splits_top_level_commas() {
    setup();
    assert_eq!(split_author_line(tk("Alice, Bob, Carol")).len(), 3);
  }

  #[test]
  fn split_author_line_splits_literal_and() {
    setup();
    assert_eq!(split_author_line(tk("Alice and Bob")).len(), 2);
  }

  #[test]
  fn split_author_line_unwraps_font_wrapper_per_author() {
    setup();
    // Beyond-Perl: brace-hidden commas would collapse to ONE creator in Perl;
    // we unwrap and re-apply the wrapper so each name is its own creator.
    let got = split_author_line(tk("\\textbf{Alice, Bob, Carol}"));
    assert_eq!(got.len(), 3);
    for piece in &got {
      let v = piece.unlist_ref();
      assert_eq!(v[0], T_CS!("\\textbf"));
      assert_eq!(v[1].get_catcode(), Catcode::BEGIN);
    }
  }

  #[test]
  fn split_author_line_leaves_single_wrapped_name_intact() {
    setup();
    // Only unwrap when there is actually an inner list to split.
    assert_eq!(split_author_line(tk("\\textbf{Alice}")).len(), 1);
  }

  // --- html_feedback#6637 helpers (OXIDIZED_DESIGN #52) ---

  #[test]
  fn normalize_hspace_becomes_quad_separator() {
    setup();
    let out = normalize_hspace_separators(tk("A \\hspace{1cm} B \\hspace*{2em} C \\hfill D"));
    // Every horizontal-space macro became a \quad separator…
    assert_eq!(position_of(&out, &[T_CS!("\\quad")]), Some(3));
    assert!(position_of(&out, &[T_CS!("\\hspace")]).is_none());
    assert!(position_of(&out, &[T_CS!("\\hfill")]).is_none());
    // …and the length arguments did not leak as text.
    let s = out.to_string();
    assert!(
      !s.contains("1cm") && !s.contains("2em"),
      "length leaked: {s}"
    );
  }

  #[test]
  fn footnote_symbol_operand_classification() {
    setup();
    // Note-marks (symbols) are recognised…
    for sym in ["*", "**", "\\dagger", "\\ddagger", "\\ast", "\\star", "\\S"] {
      assert!(
        is_footnote_symbol_operand(&tk(sym).unlist()),
        "{sym} should be a footnote symbol"
      );
    }
    // …affiliation marks (numeric/lettered) and non-symbol CS are NOT.
    for aff in ["1", "12", "a", "\\text", "\\inst"] {
      assert!(
        !is_footnote_symbol_operand(&tk(aff).unlist()),
        "{aff} must NOT be a footnote symbol"
      );
    }
    // An empty operand is not a mark.
    assert!(!is_footnote_symbol_operand(&[]));
  }

  #[test]
  fn rewrite_symbol_superscripts_recovers_marks() {
    setup();
    // $^{*}$, ${}^{\dagger}$, \textsuperscript{*}, $^{**}$ → keepsup sentinel, and
    // the superscript token is gone (no longer an affiliation-marker trigger).
    for marked in [
      "X$^{*}$",
      "X${}^{\\dagger}$",
      "X\\textsuperscript{*}",
      "X$^{**}$",
    ] {
      let out = rewrite_symbol_superscripts(tk(marked));
      assert!(
        position_of(&out, &[T_CS!("\\lx@frontmatter@keepsup")]).is_some(),
        "{marked} did not rewrite to keepsup: {out}"
      );
      assert!(
        position_of(&out, &[T_SUPER!()]).is_none(),
        "{marked} left a bare superscript trigger: {out}"
      );
    }
  }

  #[test]
  fn rewrite_symbol_superscripts_leaves_affiliation_and_math() {
    setup();
    // Numeric affiliation marks stay as superscript markers (untouched)…
    let aff = rewrite_symbol_superscripts(tk("X$^{1}$ \\and Y$^{1}$Institute"));
    assert!(
      position_of(&aff, &[T_SUPER!()]).is_some(),
      "numeric affiliation mark must be preserved: {aff}"
    );
    assert!(position_of(&aff, &[T_CS!("\\lx@frontmatter@keepsup")]).is_none());
    // …and real math (non-empty base) is not rewritten — including a SYMBOL
    // superscript on a base (`$a^{*}$`), which is genuine math, not a note-mark.
    for expr in ["$x^2$", "$a^{*}$"] {
      let math = rewrite_symbol_superscripts(tk(expr));
      assert!(
        position_of(&math, &[T_SUPER!()]).is_some(),
        "real math superscript lost in {expr}: {math}"
      );
      assert!(position_of(&math, &[T_CS!("\\lx@frontmatter@keepsup")]).is_none());
    }
  }
}
