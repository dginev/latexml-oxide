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
  // Rust-only: the RDFa pair of an `ltx:rdf` annotation ([`\lx@add@creator@rdf`]; OXIDIZED_DESIGN #479). Semiverbatim, so an
  // address's `_` and `~` are characters, not a script and a tie.
  DefKeyVal!("Frontmatter", "property", "Semiverbatim");
  DefKeyVal!("Frontmatter", "content", "Semiverbatim");

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
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    // Replaceable tags (title/toctitle/subtitle/date) keep only one entry — a later
    // one replaces earlier ones. Ported from upstream `%ReplaceableFrontmatterTags`
    // (the vendored copy pushes unconditionally → duplicate <title> when a document
    // re-adds it, e.g. arXiv 2002.09766's appendix `\icmltitle`). OXIDIZED_DESIGN #154.
    // An `accumulate` entry adds to the ones before (acmart's `\received` history, 2307.05988).
    let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate;
    // …unless the document body began after the one it would replace: that one is the paper's, this a supplement's
    // ([`seal_digested_frontmatter`]), dropped — with the authors a supplement's frontmatter lists — before it is counted
    // (a later flush's last-author conjunction reads the count).
    let seal = frontmatter_seal_key(&tag, options.get("name").map(String::as_str), &role, replaceable);
    if let Some(ref key) = seal
      && frontmatter_sealed(key)
    {
      Info!("frontmatter", "sealed", s!("{tag} after the document body began is a supplement's; the paper's is kept"));
      // …and the annotations this flush attaches to it, which would otherwise land on the paper's.
      let sealed_off = lookup_string("lx_frontmatter_sealed_off");
      assign_value(
        "lx_frontmatter_sealed_off",
        Stored::String(pin(s!("{sealed_off}\u{1}{tag}"))),
        Some(Scope::Global),
      );
      return Ok(Vec::new());
    }
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
    options.insert("_annotations".to_string(), labels.join(","));
    let entry = TagData {
      tag: tag.clone(),
      attr: options,
      content: vec![TagContent::PlaceKeeper], // (in case embedded)
    };
    DebugFeature!("frontmatter", "FRONT Add {}\n   for: {}",
      show_frontmatter(&entry), content);
    if replaceable {
      frontmatter_clear_same_name(
        &tag,
        entry.attr.get("name").map(String::as_str),
        entry.attr.get("xml:lang").map(String::as_str),
        entry.attr.get("_store").map(String::as_str),
      );
    }
    let index = frontmatter_push(&tag, entry);
    // REPLACE only 'place_keeper'!!
    let digested = digest_frontmatter_item(&tag, content)?;
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
    if let Some(key) = seal {
      note_digested_frontmatter(&key);
    }
    Ok(Vec::new())
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
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate;
    let seal = frontmatter_seal_key(&tag, options.get("name").map(String::as_str), &role, replaceable);
    // An abstract (or keywords) after the document body began, where the paper's own came before it, is a
    // supplement's: its text stays where it is, as body paragraphs (the schema admits `ltx:abstract` only in a
    // document, sidebar, bibliography or titlepage, LaTeXML-structure.rnc:668-673), and the paper's stays at the head
    // ([`seal_digested_frontmatter`]; 2301.10468, 2401.11672, 2405.10566, 2503.16707).
    if let Some(ref key) = seal
      && frontmatter_sealed(key)
    {
      Info!("frontmatter", "sealed", s!("{tag} after the document body began is a supplement's; kept in place"));
      // (read by what the body defines for itself: a class's in-abstract `\keywords` stays in place with it,
      // frontmatter_stores.rs `\lx@abstract@class@setters` — mtt.cls papers' English and Ukrainian abstracts at
      // their end: 1310.0585, 1310.0589, 1212.3890)
      assign_value("lx_frontmatter_kept_in_place", true, Some(Scope::Local));
      return digest_next_body(Some(end));
    }
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
    if replaceable && !frontmatter_has_open_placekeeper(&tag) {
      frontmatter_clear_same_name(
        &tag,
        entry.attr.get("name").map(String::as_str),
        entry.attr.get("xml:lang").map(String::as_str),
        entry.attr.get("_store").map(String::as_str),
      );
    }
    let index = frontmatter_push(&tag, entry);
    // the body's own paragraph breaks are the field's (`frontmatter_paragraph_break`); the enclosing notes come back
    // after it
    let enclosing = (lookup_value("lx_frontmatter_par_item"), lookup_value("lx_frontmatter_par_depth"));
    note_frontmatter_paragraphs(&tag);
    // A terminal reached INSIDE a nested group (`\section`'s
    // `\@startsection@hook` inside an unbalanced `\abstract{…`) ends the
    // body from there: `digest_next_body` records the terminal and depth on
    // this frame, `until_terminal_inside_group` (stomach.rs) acts.
    let body = digest_next_body(Some(end));
    assign_value("lx_frontmatter_par_item", enclosing.0.unwrap_or(Stored::None), Some(Scope::Local));
    assign_value("lx_frontmatter_par_depth", enclosing.1.unwrap_or(Stored::None), Some(Scope::Local));
    let digested = Digested::from(List::new(body?));
    // A body that typesets nothing adds no entry: an abstract (keywords, classification) with no authored content is
    // no frontmatter, whatever ended it — a heading that closed it before its first line (scrartcl's
    // `\begin{abstract}\section*{Abstract:} …`, 1507.08575; user ruling 2026-10-18, sect05.rs
    // `\lx@heading@ends@frontmatter`) or an environment left empty (pdflatex prints only its heading).
    if !typesets_content(&digested) {
      frontmatter_remove(&tag, index);
      return Ok(Vec::new());
    }
    DebugFeature!("frontmatter", "FRONT Add (until) {} for: {}", tag, digested);
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
    if let Some(key) = seal {
      note_digested_frontmatter(&key);
    }
    Ok(Vec::new())
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
    // An annotation of an entry this flush dropped as a supplement's ([`seal_digested_frontmatter`]) goes with it,
    // instead of landing on the paper's (a supplement's `\address[A]` on the paper's author labelled A).
    if lookup_string("lx_frontmatter_sealed_off").split('\u{1}').any(|sealed| sealed == parenttag) {
      return Ok(Vec::new());
    }
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
    // (a contact led by a mark of its own, which labels no one — a legend, `$^\dagger$Corresponding author`, or a mark
    // no author carries — is no answer to an author's label request, nor passes over the authors one answers)
    let legend = marker_leads(&content);
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
      // (`run:N`: the run, of the last N parents at most — authblk's `\AB@thenote` group, the authors since the last
      // `\affil`, `authblk_sty.rs`)
      let (annotate, cap) = match annotate.split_once(':') {
        Some((mode, cap)) => (mode.to_string(), cap.trim().parse::<i64>().ok()),
        None => (annotate, None),
      };
      let mut continued = false;
      // (the run's extent so far, which a contact continuing it keeps)
      let mut run_extent = 0i64;
      let (mut nprev, newonly): (i64, bool) = if annotate.is_empty() {
        (1, false)
      } else if annotate.chars().all(|c| c.is_ascii_digit()) {
        (annotate.parse().unwrap_or(1), false)
      } else if annotate == "all" {
        (nparents as i64, false)
      } else if annotate == "new" {
        (nparents as i64, true)
      } else if annotate == "run" {
        // `new`, and a second such contact before any new author (elsarticle's `\author{A}\author{B}\address{X}
        // \address{Y}`) the same run's as the one before it: past that one, the authors no longer read as new. The run is
        // keyed by the count of parents only, so a binding whose `\author` replaces the authors before it must not use it.
        let run_key = s!("{parenttag}|{role}|{nparents}");
        let last = match lookup_value("lx_frontmatter_new_run") {
          Some(Stored::String(last)) => with(last, |s| s.to_string()),
          _ => String::new(),
        };
        // (the run's record: the authors the contact before reached, and the ones Perl's `new` would have — those the
        // walk passed over as answered (below) included. A legend, `\affil{$^\dagger$Corresponding author}`, is the
        // whole run's: after `\affil{Univ Y}`, which an answered author's run passed over, it still reaches the author
        // showing its mark, a further `\affil{Univ W}` between them too (the extent is kept, below); 77n r5 review probe
        // m11, r6 review probe x1)
        let mut record = last.rsplitn(3, '|');
        match (record.next(), record.next(), record.next()) {
          (Some(extent), Some(n), Some(key)) if key == run_key => {
            continued = true;
            run_extent = extent.parse().unwrap_or(0);
            let n = if legend { extent } else { n };
            (n.parse().unwrap_or(nparents as i64), false)
          },
          _ => (nparents as i64, true),
        }
      } else {
        Info!("unexpected", &tag, s!("Frontmatter annotate '{annotate}' unrecognized"));
        (1, false)
      };
      // (a cap below 1 is none: a run of no authors would attach the contact to no one)
      if let Some(cap) = cap.filter(|cap| *cap >= 1) {
        nprev = nprev.min(cap);
      }
      DebugFeature!("frontmatter", "...adding to {nprev} previous{}",
        if newonly { " new" } else { "" });
      let (run_new, mut attached, mut passed) = (annotate == "run", 0i64, 0i64);
      let offered = if run_new && !legend {
        lookup_string("lx_frontmatter_offered_labels")
      } else {
        String::new()
      };
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
            // (a parent has the role already when it has such a contact; in a run, also when a mark of its own is
            // answered by a mark-picked contact of the role (`_bymark`, `\lx@affiliation@withsup`) — one deferred until
            // the labels are compared, or one this flush digests later (`lx_frontmatter_offered_labels`) — which has
            // not reached it yet: `\author{Ann$^{1}$}\affil{$^1$Univ X}\author{Bob \affil{Univ Y}}` gave Ann Y as well,
            // 77n r4 review probes e1, e4; a label nothing answers, `\author{Ann$^{1}$}\affil{$^{1-3}$Univ X}`, is none
            // (b3); a legend reads the contact only (`\affil{$^\dagger$Corresponding author}` after mark-picked ones
            // is every new author's, r3 review probe q04; 2605.11982's after `\maketitle`). A `label=` answers
            // nothing here: authblk's `\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}` prints Univ Y under
            // both (2605.25017, 77n r5 review probes q10, r07). Perl's `new` checks the contact only,
            // Base_Utility.pool.ltxml:530)
            let answering: Vec<String> = if run_new && !legend {
              list
                .iter()
                .filter(|e| {
                  e.attr.get("role").map(String::as_str) == Some("pending")
                    && e.attr.contains_key("_bymark")
                    && matches!(e.content.first(),
                      Some(TagContent::Entry(contact)) if contact.attr.get("role") == Some(&role))
                })
                .filter_map(|e| e.attr.get("_annotations"))
                .flat_map(|labels| labels.split(','))
                .map(str::to_string)
                .chain(offered.split('\u{1}').filter_map(|offer| {
                  offer
                    .split_once('\u{2}')
                    .filter(|(of, _)| *of == role)
                    .map(|(_, label)| label.to_string())
                }))
                .filter(|label| !label.is_empty())
                .collect()
            } else {
              Vec::new()
            };
            let answered = |parent: &TagData| {
              parent
                .attr
                .get("_annotations")
                .is_some_and(|labels| {
                  labels.split(',').any(|label| answering.iter().any(|a| a == label))
                })
            };
            // (the walk stops at an answered author only once it has attached the contact to an author no mark
            // answers: when the last author is answered too, every author of the run is, and the contact is the
            // whole run's as Perl's `_has{role}` test gives it — ptephy's `\affil{}{Univ Z}` after `\affil{1}{…}
            // \affil{2}{…}`, which the class prints under all of them; 77n r5 review probes t02, t04, o04)
            let mut reached_unanswered = false;
            let mut indices = parent_indices.clone();
            while let Some(pi) = indices.pop() {
              if nprev <= 0 {
                break;
              }
              nprev -= 1;
              if let Some(parent) = list.get_mut(pi) {
                attached += 1;
                parent.content.push(datum.clone());
                if !role.is_empty() {
                  reached_unanswered |= !answered(parent);
                  parent.attr.insert(has_role_key.clone(), "1".to_string());
                  let Some(&next_pi) = indices.last().filter(|_| newonly) else {
                    continue;
                  };
                  if list.get(next_pi).is_some_and(|next| next.attr.contains_key(&has_role_key)) {
                    break;
                  }
                  if reached_unanswered && list.get(next_pi).is_some_and(answered) {
                    // (the authors Perl's `new` reaches past here, for a legend of the run)
                    passed = indices
                      .iter()
                      .rev()
                      .take(nprev.max(0) as usize)
                      .take_while(|&&pi| {
                        list
                          .get(pi)
                          .is_some_and(|p| !p.attr.contains_key(&has_role_key))
                      })
                      .count() as i64;
                    break;
                  }
                }
              }
            }
          }
      });
      // (a contact that reached no one leaves the run as it was: `attached=0` would zero a later contact of the run; nor
      // does a legend within it, whose extent is not the authors the next contact continues with — one opening the run
      // starts it, `\affil{$^1$Univ X}` no author's mark answers before `\affil{Univ Y}`, r4 review probe d6. A contact
      // continuing the run keeps its extent: `\affil{Univ W}` after `\affil{Univ Y}`, which passed over the answered
      // author, reaches the unanswered ones only, and the legend after both still reaches the author showing its mark,
      // 77n r6 review probe x1)
      if run_new && attached > 0 && !(legend && continued) {
        let extent = if continued {
          run_extent.max(attached + passed)
        } else {
          attached + passed
        };
        assign_value(
          "lx_frontmatter_new_run",
          Stored::String(pin(s!("{parenttag}|{role}|{nparents}|{attached}|{extent}"))),
          Some(Scope::Global),
        );
      }
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
    add_annotation_labels(label, false);
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
    let mut out = if lookup_bool("\\author:redefined") {
      TokenizeInternal!(
        r"\def\lx@author@markup@unprovide{}\lx@author@markup@provide\prefix\@firstofone\lx@author@markup@provide\suffix\@firstofone\lx@author@markup@provide\particle\@firstofone\lx@author@markup@provide\fnms\@firstofone\lx@author@markup@provide\snm\@firstofone\lx@author@markup@provide\inits\@firstofone\lx@author@markup@provide\degs\@firstofone\lx@author@markup@provide\roles\@firstofone\lx@author@markup@provide\orcid\lx@author@orcid"
      )
    } else {
      TokenizeInternal!(r"\def\lx@author@markup@unprovide{}")
    }
    .unlist();
    // The author markup a class defines only where its title code reads `\@author` is in force for the author content
    // too: melba.cls:279-283 `{\def\aff{…}\def\name{…}\@author …}` (2405.09787, 2407.21368, 2105.14711), dmlr2e.sty:
    // 249-251 (2404.08403), amta2024.sty:77-78 (2405.16969), ssArxiv.sty:109-111 (2503.22625), a preamble's own
    // `\@maketitle` (2601.05137). In pdflatex `\@author` expands inside that group, while here the author content is
    // read at the frontmatter, before the title code runs. Each definition of a control sequence still undefined runs,
    // in source order, for the author content, which it is undefined again after, as the OmniBus author markup is
    // (#444); a definition already in force stands. Perl errs the same (`undefined:\name`).
    let definitions = title_code_author_definitions();
    let undefined: Vec<bool> = definitions
      .iter()
      .map(|(definee, _)| lookup_meaning(definee).is_none())
      .collect();
    for ((definee, statement), undefined) in definitions.into_iter().zip(undefined) {
      if !undefined {
        continue;
      }
      out.extend(statement);
      out.extend(
        TokenizeInternal!(r"\expandafter\def\expandafter\lx@author@markup@unprovide\expandafter{\lx@author@markup@unprovide\let")
          .unlist(),
      );
      out.extend([definee, T_CS!("\\@undefined"), T_END!()]);
    }
    Ok(Tokens::new(out))
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
  // Field provenance (frontmatter model M3, stage S1): `\lx@field@mark<prefix><store>{<field>}` — the store, a macro
  // without parameters, redefined (`<prefix>` `\def` or `\gdef`, the store's own scope at the setter) with the same
  // tokens, each now carrying out-of-band provenance "token <i> of field <field>" (`Token::prov`). Nothing is added to
  // the store: every token operation sees exactly the tokens pdflatex sees (`\ifx\@title\same`, `\equal`, xstring's
  // `\StrLen`/`\StrLeft`, `\pdfstrcmp`, `\@gobble`, `\@car`, `\csname`, `\write`), since token equality and every
  // character form ignore the provenance. What expands from a stamped token inherits it and what is digested from
  // one carries it (`fieldProv`), so the document's own print of the store is known for the field's ink by
  // construction (core `Document::absorb`'s field runs; [`lift_marked_field_ink`]). The kernel `\title` and `\author`
  // (sect05.rs) call it after the frontmatter took its copy. An empty or blank value is left as it is; anything but
  // a parameterless macro is left alone. OXIDIZED_DESIGN #715.
  DefMacro!("\\lx@field@mark Token DefToken {}", sub[(prefix, store, field)] {
    let field = field.to_string();
    let Some(field_id) = field_id(field.trim()) else {
      return Ok(Tokens::default());
    };
    let Some(defn) = lookup_definition(&store)? else {
      return Ok(Tokens::default());
    };
    if defn.get_parameters().is_some_and(|p| !p.get_parameters().is_empty()) {
      return Ok(Tokens::default());
    }
    let Some(ExpansionBody::Tokens(value)) = defn.get_expansion() else {
      return Ok(Tokens::default());
    };
    if value.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      return Ok(Tokens::default());
    }
    // Each call's positions follow the previous call's (`lx_field_base_<id>`), so a copy of an earlier value
    // (`\let\old\@title`, then `\title{Short}`) is never taken for a print of this one (review finding 7).
    let base = usize::try_from(lookup_int(&s!("lx_field_base_{field_id}"))).unwrap_or(0);
    let len = value.unlist_ref().len();
    if base + len > usize::from(FIELD_PROV_POS_MASK) {
      // out of positions: this value is not stamped, and no earlier copy can be whole of it
      assign_value(&s!("lx_field_extent_{field_id}"), String::new(), Some(Scope::Global));
      assign_value(&s!("lx_field_range_{field_id}"), String::new(), Some(Scope::Global));
      return Ok(Tokens::default());
    }
    assign_value(&s!("lx_field_base_{field_id}"), (base + len) as i64, Some(Scope::Global));
    // this call's positions: a print is of the value it sets only within them (an earlier call's print or copy, r12,
    // r13, is not joined with it, [`field_ink_runs`])
    assign_value(
      &s!("lx_field_range_{field_id}"),
      s!("{base}:{}", base + len),
      Some(Scope::Global),
    );
    // The field's extent: the position of every token of the value that bears ink or makes it (a character, a
    // control sequence, an active character; a macro's expansion and a primitive's arguments are covered from its
    // position, core `gullet::field_cover_of`), which a whole print covers, all of them: a fragment the document
    // takes of the value (`\StrLeft{\@title}{5}` of `About \nm`, `\@car`, `\@gobble`) leaves some out (review
    // finding 2). Not a space, a brace, `$`, `^`, `_`, `&`, `#`: what they make is covered by the tokens around them.
    // (as `start:end` intervals, end exclusive, joined by `,`: the form of a run's cover, [`field_ink_runs`])
    // The value's characters (a letter, an other character) are kept apart too: a print digests each once, so a run
    // that holds one twice read the value again ([`field_ink_runs`]).
    let mut extent: Vec<(usize, usize)> = Vec::new();
    let mut chars: Vec<(usize, usize)> = Vec::new();
    let push = |intervals: &mut Vec<(usize, usize)>, at: usize| match intervals.last_mut() {
      Some((_, end)) if *end == at => *end += 1,
      _ => intervals.push((at, at + 1)),
    };
    for (i, t) in value.unlist_ref().iter().enumerate() {
      let code = t.get_catcode();
      if matches!(
        code,
        Catcode::LETTER | Catcode::OTHER | Catcode::CS | Catcode::ACTIVE
      ) {
        push(&mut extent, base + i);
      }
      if matches!(code, Catcode::LETTER | Catcode::OTHER) {
        push(&mut chars, base + i);
      }
    }
    if extent.is_empty() {
      return Ok(Tokens::default());
    }
    let joined = |intervals: &[(usize, usize)]| {
      intervals
        .iter()
        .map(|(start, end)| s!("{start}:{end}"))
        .collect::<Vec<_>>()
        .join(",")
    };
    assign_value(&s!("lx_field_extent_{field_id}"), joined(&extent), Some(Scope::Global));
    assign_value(&s!("lx_field_chars_{field_id}"), joined(&chars), Some(Scope::Global));
    document::note_field_prov_stamped();
    let mut out = vec![prefix, store, T_BEGIN!()];
    for (i, t) in value.unlist_ref().iter().enumerate() {
      let stamped = Token { prov: field_prov(field_id, base + i), ..*t };
      // A `#` in the value is text here, not a parameter of the definition (as `\lx@deposit@field`).
      if t.get_catcode() == Catcode::PARAM {
        out.push(stamped);
      }
      out.push(stamped);
    }
    out.push(T_END!());
    Ok(Tokens::new(out))
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
  // (the body's first token, `\lx@abstract@class@setters`, defines there the keywords setters a raw class's refused
  // abstract defines for its body: frontmatter_stores.rs, 68n — wiley-article 1706.00599, astlb 1202.2889, mtt
  // 1310.0585)
  DefMacro!(
    "\\lx@begin@abstract[]",
    "\\lx@frontmatter@mark{ltx:abstract}\\lx@clear@frontmatter{ltx:abstract}\\lx@add@frontmatter@until{ltx:abstract}[#1]{\\lx@end@abstract}\\lx@abstract@class@setters"
  );

  // Like \let \relax, but \relax not def yet!
  // The until-body terminals: no-ops when the body's loop sees them; inside a
  // nested group they end the body from there (stomach.rs).
  // A keywords body a command form opened inside the abstract ends with it: ieeeconf's `\keywords` is a label with no
  // extent (`\begin{abstract} … \keywords \MyKeywords \end{abstract}`, 1401.5657), and read as the open body it kept the
  // abstract open past its `\end{abstract}`, so the whole document went into `<ltx:keywords>`.
  DefPrimitive!("\\lx@end@abstract", {
    if lookup_token("lx@until@terminal") == Some(T_CS!("\\lx@end@keywords")) {
      unread(Tokens!(
        T_CS!("\\lx@end@keywords"),
        T_CS!("\\lx@end@abstract")
      ));
      return Ok(Vec::new());
    }
    until_terminal_inside_group(&T_CS!("\\lx@end@abstract"))?;
  });

  DefMacro!(
    "\\lx@begin@keywords[]",
    "\\lx@clear@frontmatter{ltx:keywords}\\lx@add@frontmatter@until{ltx:keywords}[#1]{\\lx@end@keywords}"
  );

  DefPrimitive!("\\lx@end@keywords", {
    until_terminal_inside_group(&T_CS!("\\lx@end@keywords"))?;
  });

  // `\fnmsep` in frontmatter content (`digest_frontmatter_item`): nothing before a mark the frontmatter turns into a
  // link or a note, the class's comma otherwise.
  DefMacro!(
    "\\lx@frontmatter@fnmsep",
    r"\@ifnextchar\thanks{}{\@ifnextchar\inst{}{\@ifnextchar\footnote{}{\@ifnextchar\thanksref{}{\lx@saved@fnmsep}}}}"
  );

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
  // document-level `\inst` is the affiliation LIST, not a label). The author markup is in force here too: an
  // affiliation line of the author block is author content (`$^{3}$Chan Zuckerberg Biohub \\ \email Correspondence:
  // …` under auth_detailed.sty's `\@maketitle` definitions, 2406.07496).
  DefMacro!(
    "\\lx@affiliation@withinst{}",
    "\\let\\lx@saved@inst\\inst\\def\\inst##1{\\lx@sup@setlabel@affiliation{##1}}\\lx@author@markup@begin\\expandafter\\lx@author@markup@run\\expandafter{\\lx@author@markup@unprovide}{#1}\\let\\inst\\lx@saved@inst"
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
  // Rust-only: a contact a class's command gives apart from the names (OSA's `\email{x@y}` and `\homepage{…}` after
  // the author groups, opex3.sty:52/:57), placed among the queued authors by [`placed_contact_owner`] (`label=author:N`,
  // or the mark's own label, or `label=addresses:block`, the shared creator of #159). `#1` is the contact's keyvals
  // (its `role`, its `name`), `#2` the operand of the mark written on it (`$^*$x@y`: `*`; empty for none), `#3` its
  // content, already read.
  DefMacro!("\\lx@add@contact@placed[]{}{}", sub[(attr, mark, content)] {
    let mark: Tokens = mark;
    let content: Tokens = content;
    // (`owner=first` in the keyvals: a contact that names no one goes to the group's first author, not to the block —
    // AASTeX's `\authoremail` given before the authors, aas_support_sty.rs)
    let attr_text = attr.as_ref().map(ToString::to_string).unwrap_or_default();
    let owner_first = attr_text.split(',').any(|kv| kv.trim() == "owner=first");
    let attr_rest: Vec<&str> = attr_text.split(',').map(str::trim).filter(|kv| !kv.is_empty() && *kv != "owner=first").collect();
    // (`label=` and `_answers=` are fixed text; a value is a mark's label and is text: tokenized, `$^{1\#}$`'s `#` was
    // a parameter token and a `%` a comment, so each of its characters is taken as it is)
    let (value, answers) = match placed_contact_owner(&mark, &content) {
      ContactOwner::Mark(label) => (label, None),
      // (placed by a footnote-symbol mark its author requests: the contact answers that mark on that author,
      // `show_unanswered_marks`)
      ContactOwner::Author(author) => (s!("author:{author}"), answered_symbol_key(mark.unlist_ref())),
      ContactOwner::Block if owner_first => (s!("author:{}", frontmatter_group_first()), None),
      ContactOwner::Block => (s!("addresses:block"), None),
    };
    let mut keyvals: Vec<Token> = mouth::tokenize_internal("label=").unlist();
    keyvals.extend(text_tokens(&value));
    if let Some(key) = answers {
      keyvals.extend(mouth::tokenize_internal(",_answers=").unlist());
      keyvals.extend(text_tokens(&key));
    }
    if owner_first {
      if !attr_rest.is_empty() {
        keyvals.push(T_OTHER!(","));
        keyvals.extend(mouth::tokenize_internal(TeXString::assembled(attr_rest.join(","))).unlist());
      }
    } else if let Some(attr) = attr.as_ref().filter(|a| !a.is_empty()) {
      keyvals.push(T_OTHER!(","));
      keyvals.extend(attr.unlist_ref().iter().copied());
    }
    Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"), vec![
      Some(mouth::tokenize_internal("ltx:creator")),
      Some(mouth::tokenize_internal("ltx:contact")),
      Some(Tokens::new(keyvals)),
      Some(content)
    ]))
  });
  // Rust-only: the start of an author group of a class whose `\author` adds a group per call (OSA's, each typesetting
  // its names where it stands): a contact placed after it ([`placed_contact_owner`]) is the group's only author's
  // (`\author{D. L. Boiko}\address{…}\email{…}\author{N. J. Gunther}…`, 0906.4563; guard
  // `perfect_kernel_batch64u::osa_group_only_author_owns_the_email`).
  DefPrimitive!("\\lx@frontmatter@group@start", {
    let first = queued_author_marks().len() + 1;
    assign_value(
      "lx_frontmatter_group_first",
      Stored::Number(Number::new(first as i64)),
      Some(Scope::Global),
    );
  });
  // Rust-only (OXIDIZED_DESIGN #479): what the document says about an author but does not print (AASTeX 7's `\email`
  // without `[show]`, aastex701.cls:13341-13353; 2608.05283) — an `ltx:rdf` annotation of the creator, `property` the
  // RDFa term and `content` the value, hidden in HTML. An empty value records nothing (AASTeX 7's `\email{}` placeholder).
  DefMacro!("\\lx@add@creator@rdf [] {} Semiverbatim", sub[(attr, property, value)] {
    // The value as written, without the spaces around it (2608.13718 `\email{koberg@cfa.harvard.edu }`)
    let mut value = value.unlist();
    while value.last().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
      value.pop();
    }
    let lead = value.iter().take_while(|t| t.get_catcode() == Catcode::SPACE).count();
    value.drain(..lead);
    if value.is_empty() {
      return Ok(Tokens!());
    }
    let mut opts = mouth::tokenize_internal("property=").unlist();
    opts.push(T_BEGIN!());
    opts.extend(property.unlist());
    opts.push(T_END!());
    opts.extend(mouth::tokenize_internal(",content=").unlist());
    opts.push(T_BEGIN!());
    opts.extend(value);
    opts.push(T_END!());
    if let Some(a) = attr.as_ref().filter(|a| !a.is_empty()) {
      opts.push(T_OTHER!(","));
      opts.extend(a.unlist_ref().iter().copied());
    }
    Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"),
      vec![Some(mouth::tokenize_internal("ltx:creator")),
           Some(mouth::tokenize_internal("ltx:rdf")),
           Some(Tokens::new(opts)), Some(Tokens!())]))
  });
  DefMacro!(
    "\\lx@add@email@metadata [] Semiverbatim",
    "\\lx@add@creator@rdf[#1]{schema:email}{#2}"
  );
  DefMacro!(
    "\\lx@add@url [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=url,#1]{#2}"
  );
  // Rust-only: an email or web address set as a class sets it with `\url`, `\texttt` or `\ttfamily` — url.sty's
  // `\UrlFont` (url_sty.rs `\UrlFont`): the typewriter family, in the identity ASCII encoding. A class that reads the
  // address with `_` and `~` made other (revtex4.cls:4476-4493 `\sanitize@url` … `\href{mailto:#2}{#2}`, `\href` the
  // `\url` = `\texttt` of :4490-4493) prints them as cmtt does, '_' and '~'; the same characters in the roman OT1 font
  // are its dot and tilde accents (Perl TeX_Fonts.pool.ltxml:214), "ann˙able@a.edu" (1001.1076). A class binding
  // passes its address through this only where the class sets it in typewriter type.
  DefMacro!(
    "\\lx@address@urlfont{}",
    "{\\lx@fontencoding{ASCII}\\ttfamily #1}"
  );
  // Rust-only: an email or web address a class sets in the current (roman) font — revtex4-1/4-2's `\href`, which
  // without hyperref is `\@href` (revtex4-1.cls:1959-1961), AASTeX's, svjour3's, jmlr's — in that font, but in the
  // identity ASCII encoding: its `_` and `~` print as typed. In the OT1 font pdflatex prints them as the dot and tilde
  // accents ("ann˙able@a.edu", 2301.03570, 2301.03067); the address is the author's, not the accident of the font (user
  // ruling 2026-10-11).
  DefMacro!("\\lx@address@textfont{}", "{\\lx@fontencoding{ASCII}#1}");
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
  // An iD given as its orcid.org address (`https://orcid.org/0000-…`, with or without the scheme or `www.`) is the iD:
  // its link was `https://orcid.org/https://orcid.org/0000-…` (2104.12510, 2304.13458, 2407.18715 `\orcidauthor`;
  // DIVERGENCES #616). Perl's contact holds the text as given (Base_Utility.pool.ltxml:659-660).
  DefMacro!(
    "\\lx@add@orcid [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=orcid,#1]{\\expandafter\\lx@orcidlink\\expandafter{\\lx@orcid@id{#2}}{\\lx@orcidlogo}}"
  );
  DefMacro!("\\lx@orcid@id Semiverbatim", sub[(id)] { Ok(orcid_id(id)) });
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
        if spacing_end(seg.unlist_ref()) == 0 {
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
    // (a `\let` alias of a mark command is the command: `\let\up\textsuperscript`, 2209.14855; [`unalias_mark_commands`])
    let stuff = unalias_mark_commands(expand_separator_macros(expand_sole_author_macro(stuff)));
    // (an empty mark prints nothing and is no mark: `\textsuperscript{}Univ. Grenoble Alpes…` leaves the block unmarked,
    // its line every name's; [`drop_empty_marks`])
    let stuff = drop_empty_marks(stuff);
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
    let stuff = note_environments_as_commands(stuff);
    let stuff = math_active_scripts(stuff);
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
    let stuff = join_icons_to_links(stuff);
    let stuff = rewrite_symbol_superscripts(stuff);
    let stuff = rewrite_ordinal_superscripts(stuff);
    let stuff = rewrite_in_word_superscripts(stuff);
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
    } else if has_author_marks(&stuff) && (!cautious || answers_its_marks(&stuff)) {
      // (a per-author class binding keeps its marks visible on the names, as Perl does, and takes the unmarked path
      // below — its affiliation commands answer no mark (an IEEE block's `\IEEEauthorblockA`, amsart's `\address`) —
      // unless the block holds its marked affiliations itself (2401.14196's `\author[*]{… \\ $^1$DeepSeek-AI …}`))
      let mut entries: Vec<(AuthorLineKind, Tokens)> = Vec::new();
      // the entries that are notes naming the corresponding author, each its own mark-led line (`$^\spadesuit$
      // Corresponding author`, 1305.7027): the next labelled email line, unless an author or affiliation line comes
      // first, goes to whom its mark names ([`block_email_calls`])
      let mut correspondence_notes: Vec<usize> = Vec::new();
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
      // (a line of several links a line per link, [`link_run_lines`]: 2601.06979, the r7 re-review's finding 5)
      let group_lines = |group: Tokens| -> Vec<(Vec<Token>, Tokens)> {
        split_tokens_delimited(group, author_affil_splits())
          .into_iter()
          .flat_map(|(delimiter, line)| {
            link_run_lines(line)
              .into_iter()
              .enumerate()
              .map(move |(at, piece)| {
                let delimiter = if at == 0 {
                  delimiter.clone()
                } else {
                  vec![T_SPACE!()]
                };
                (delimiter, piece)
              })
          })
          .collect()
      };
      let block_lines: Vec<Tokens> = groups
        .iter()
        .flat_map(|group| group_lines(group.clone()))
        .map(|(_, line)| line)
        .collect();
      let mut position = 0;
      // where each `\and` group's entries start, for the creator a link line's URL contact goes to ([`link_line_owner`])
      let mut group_starts: Vec<usize> = Vec::new();
      for group in groups {
        // Marker-less lines may only continue an entry created WITHIN this
        // `\and` group — never one from a previous group.
        let group_start = entries.len();
        group_starts.push(group_start);
        let mut names_line = true;
        // the last line of names this group's authors were split from, which a line of more names continues
        let mut last_names: Option<Tokens> = None;
        for (delimiter, line) in group_lines(group) {
          position += 1;
          let beside = beside_prefix_marked_names(prefix_marked_names, &mut names_line, &delimiter);
          // (a piece that is only the word "and" separates the names beside it: `\quad \textnormal{and} \quad`,
          // 2608.24427)
          if line.is_empty() || is_and_piece(line.unlist_ref()) {
            continue;
          }
          match author_mark_position(&line) {
            None => {
              // A marker-less line that is purely a list of email addresses is a
              // SHARED email line (\texttt{a@x, b@y}) covering all the authors, not
              // a continuation of the previous affiliation — give it its own email
              // contact so it is shown once instead of being welded into an
              // affiliation's text (KNOWN_PERL_ERRORS #75, witness 2605.23553).
              if line_is_email_list(&line) {
                entries.push((AuthorLineKind::Email, line));
              } else if let Some(email_line) = labelled_email_line(&line)? {
                // A labelled email line (`Email: \email{x}`, JINST/JHEP author blocks) is the authors' emails, not a
                // continuation of the affiliation above it: under the last mark it went to that mark's authors
                // (0903.0326's `\llap{$^d$}… \\ Email: \email{demitri.muna@nyu.edu}` to the four authors marked d,
                // not D. Muna). Placed by the names the addresses spell ([`block_email_calls`]).
                entries.push((AuthorLineKind::LabelledEmail, email_line));
              } else if entries
                .last()
                .is_some_and(|(kind, _)| *kind == AuthorLineKind::Author)
                && last_names
                  .as_ref()
                  .is_some_and(|names| names_list_continues(names, &line))
              {
                // An unfinished name list goes on past the break, as in an unmarked block ([`names_continue`]):
                // `Gus Gray\IEEEauthorrefmark{1}, …, Ida Ivy,~\IEEEmembership{Fellow,~IEEE,}\\ Jon Jay, … and~Kim
                // Key` (2408.02464) is five authors, not Ida Ivy's name welded to the next line's.
                last_names = Some(line.clone());
                for author in split_author_line(line) {
                  entries.push((AuthorLineKind::Author, author));
                }
              } else if entries.len() > group_start
                && entries[group_start..]
                  .iter()
                  .all(|(kind, _)| *kind == AuthorLineKind::LabelledEmail)
              {
                // (only email lines before it in its `\and` group: its own entry, not a new author; a link line a URL
                // contact, [`is_link_line`])
                let kind = if is_link_line(&line) {
                  AuthorLineKind::Link
                } else {
                  AuthorLineKind::Affiliation
                };
                entries.push((kind, line));
              } else if let Some(continued) = entries[group_start..]
                .iter()
                .rposition(|(kind, _)| *kind != AuthorLineKind::LabelledEmail)
                .map(|at| group_start + at)
              {
                // continues an entry from THIS `\and` group; Append, with the delimiter that split the lines
                // (`\\` → break, `\quad`/`\qquad` → space) put back where it stood: Perl (Base_Utility.pool.ltxml:701-703 `Tokens($entries[-1][1],
                // $line)`) welds the two lines' words ("Department of PhysicsUniversity of Somewhere";
                // KNOWN_PERL_ERRORS #446, witness jacow-collaboration). Repro
                // sectioning-frontmatter/author_continuation_line_keeps_its_break.
                // A labelled email line between them is its own entry: the line continues the one before it
                // (1503.08624's `E-mail: \email{…}\\ \textbf{Proceeding of …}`, both lost when welded together).
                // …and after an email line under a name, its own entry: welded, it read as part of the name
                // (`A.~Able$^a$ \\ E-mail: \email{x} \\ More text`)
                // …and a legend, a footnote symbol and its text (`\\ \textsuperscript{*}\textit{Equal
                // contribution.}`, 2509.20748), is no part of the name before it, as it is no affiliation's
                // continuation in `\lx@add@affiliation` (llncs `\institute`, 2609.06094); and, under an affiliation
                // too, a symbol-led line that reads as an affiliation its symbol labels is its own entry, which the
                // symbol links to the authors carrying it (`symbol_labelled_affiliation`: 2601.05633's
                // `$^{\diamond}$ National University of Singapore` under the `$^{\heartsuit}$` line, not welded into it
                // and so given to the authors marked ♡, batch 77p r6); a note (`$^\dagger$ Deceased`, `$^*$
                // Corresponding author`) continues the affiliation above as before (batch 64k)
                // …and under an e-mail line, a line that is not addresses is its own entry, welded into neither the
                // e-mail line (the addresses no longer read as such, the line one contact on the last author: 2602.22394's
                // `{\footnotesize \url{…}}`, 2602.23802, 2602.09509's `Demo Code:~\url{…}`) nor the entry above it
                // (the r5 re-review's q3, "Univ Bhttps://…"; 2403.08629); and so is a link line ([`is_link_line`]) under
                // any entry, no part of an affiliation's or a name's text: a URL contact, not an affiliation (user ruling
                // 2026-10-20, batch 77p r7: 2605.06223, 2602.08439, 2604.12896), and a line under it is no part of it
                // (a line that prints nothing is no entry: `\large{~~}`, 2609.05430; a blank line's `\par`, 2606.00747;
                // `\tt\small` before a `\quad`, 2103.00887)
                if prints_nothing(&line) {
                  continue;
                }
                // (an address the line above leaves unfinished goes on here: `{\tt yoshikawa.masashi.yh8@}\\{\tt
                // is.naist.jp}`, 1906.01834)
                let finishes_an_address = entries[continued].0 == AuthorLineKind::Email
                  && email_addresses(&entries[continued].1)
                    .and_then(|addresses| addresses.last().cloned())
                    .is_some_and(|address| address.ends_with('@'));
                // (a labelled e-mail line between them is skipped, as before: 1503.08624's proceedings line goes on with
                // the affiliation above it, a name's own entry)
                let under_email = entries[continued..].iter().any(|(kind, _)| {
                  *kind == AuthorLineKind::Email
                    || (*kind == AuthorLineKind::LabelledEmail
                      && entries[continued].0 == AuthorLineKind::Author)
                });
                let carried: Vec<String> = entries
                  .iter()
                  .filter(|(kind, _)| *kind == AuthorLineKind::Author)
                  .flat_map(|(_, author)| symbol_marks(author))
                  .collect();
                let labelled = symbol_labelled_affiliation(&line, &carried);
                // (a link printing an institution too: placed as a link line is, an affiliation, [`url_contact_call`])
                if !finishes_an_address && is_link_line(&line) {
                  entries.push((AuthorLineKind::Link, line));
                  continue;
                }
                if !finishes_an_address
                  && (labelled.is_some()
                    || (entries[continued].0 == AuthorLineKind::Author && is_legend_line(&line))
                    || under_email
                    || entries[continued].0 == AuthorLineKind::Link)
                {
                  entries.push((AuthorLineKind::Affiliation, labelled.unwrap_or(line)));
                  continue;
                }
                let last = &mut entries[continued];
                let mut appended = last.1.clone().unlist();
                appended.extend(delimiter);
                appended.extend(line.unlist());
                last.1 = Tokens::new(appended);
              } else {
                // First line of this `\and` group and it has no marker → a NEW author (never merged
                // back into the previous group), whole as Perl keeps it ("safest to assume author?",
                // Base_Utility.pool.ltxml:705-706) — unless it reads as several names, then each its own
                // (`Aghil Alaee\footnote{…} \,\,and Hari K. Kunduri\footnote{…}\\ … $^a$ Department …`,
                // 1407.0988), as the merged-author check would otherwise report. A link line printing a URL there is
                // no name: a URL contact, as under an entry (`\url{…} \\ Ann Able$^1$ …`, `… \and \url{…}`; the r5
                // re-review's u4, u16); a linked name is its author ([`prints_a_url`]; 2608.24427).
                if prints_a_url(&line) {
                  entries.push((AuthorLineKind::Link, line));
                } else if name_count(&visible_name_text(line.unlist_ref())) >= 2 {
                  // …a names line an unmarked row of more names continues, as a later one does (`Ann Able$^{*}$, Bob
                  // Baker, \\ Cat Cole, Dan Dee, \\ Eve Eck$^{1}$`: the first line's symbol mark is no affiliation mark;
                  // 2411.13503, 2410.07701, 2401.05566, 2510.11639, 2304.07193).
                  last_names = Some(line.clone());
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
              // (a link line among them, as when it read as an affiliation)
              let affiliations_begun = entries.iter().any(|(kind, _)| {
                matches!(kind, AuthorLineKind::Affiliation | AuthorLineKind::Link)
              });
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
                last_names = Some(line.clone());
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
                // by number, instead of merging into one. The spacing that leads the line prints nothing and is no part
                // of its first institution (`\kern2pt$^{1}$Univ A`, `\vspace{0.1cm} $^{1}$ …`; 2407.15815).
                let line =
                  Tokens::new(line.unlist_ref()[leading_spacing_end(line.unlist_ref())..].to_vec());
                for seg in split_wrapped_affiliation_marks(line) {
                  if spacing_end(seg.unlist_ref()) == 0 {
                    continue;
                  }
                  if names_correspondence(&visible_name_text(seg.unlist_ref())) {
                    correspondence_notes.push(entries.len());
                  }
                  if let Some((AuthorLineKind::Affiliation | AuthorLineKind::Link, last)) =
                    entries.last_mut()
                  {
                    *last = Tokens::new(trim_trailing_separator(last.clone().unlist()));
                  }
                  entries.push((AuthorLineKind::Affiliation, seg));
                }
              }
            },
          }
        }
      }
      // A name line's trailing address is the name's email (`\name Ann Able$^{1}$ \email ann@uni.edu`, 2502.14381).
      let entries = entries
        .into_iter()
        .map(|(kind, line)| {
          Ok(if kind == AuthorLineKind::Author {
            let (name, email) = name_and_trailing_email(&line)?;
            (kind, name, email)
          } else {
            (kind, line, None)
          })
        })
        .collect::<Result<Vec<_>>>()?;
      let author_keys: Vec<NameKeys> = entries
        .iter()
        .filter(|(k, ..)| *k == AuthorLineKind::Author)
        .map(|(_, author, _)| NameKeys::of(author))
        .collect();
      let author_count = author_keys.len();
      // the authors an address on any e-mail line of the block spells, whom elimination gives no other address
      // ([`owners_by_elimination`]; 2605.06223's `kyle.min@…` under the `{treejhk, …}@…` line)
      let mut block_addresses: Vec<String> = Vec::new();
      // (and, by entry, how many of an e-mail line's addresses no name spells and no rule has placed yet: while another
      // line holds one, no name goes by elimination, [`owners_by_elimination`]; 2602.23777, the r5 re-review's q6)
      let mut unplaced: Vec<usize> = vec![0; entries.len()];
      for (at, (kind, line, _)) in entries.iter().enumerate() {
        if matches!(kind, AuthorLineKind::Email | AuthorLineKind::LabelledEmail) {
          let texts = line_address_texts(line)?;
          unplaced[at] = unspelled_addresses(&author_keys, &texts);
          block_addresses.extend(texts);
        }
      }
      let mut taken = names_spelled(&author_keys, &block_addresses);
      // the one author a `\thanks{Corresponding author.}` names, who an address spelling no name belongs to
      let corresponding = corresponding_author(
        entries
          .iter()
          .filter(|(k, ..)| *k == AuthorLineKind::Author)
          .map(|(_, author, _)| author),
      );
      // the addresses given to the authors in order so far; every email contact advances the `labelseq` count, so after
      // a line kept whole no later address is the next author's (2403.00801's `benhe@…` under two such lines)
      let mut sequenced = Some(0);
      // the mark of the note just read that names the corresponding author (`$^\spadesuit$ Corresponding author`,
      // 1305.7027): an email line after it that spells no one goes to whom that mark says
      let mut correspondence_mark: Option<String> = None;
      // the marks of the authors queued so far and of this call's, for the authors a note's mark names (a note in a later
      // `\author` call than its author, 1305.7027)
      let author_marks: Vec<Vec<Vec<Token>>> = queued_author_marks()
        .into_iter()
        .chain(
          entries
            .iter()
            .filter(|(k, ..)| *k == AuthorLineKind::Author)
            .map(|(_, author, _)| author_mark_operands(author.unlist_ref())),
        )
        .collect();
      // `author:N` counts every author numbered so far: those queued, and those an earlier flush digested and kept
      // (not when this call replaces them: `lx_authors_superseded`)
      let numbered_kept = if lookup_bool("lx_authors_superseded") {
        0
      } else {
        lookup_mapping_int("num_ltx:creator", "author") as usize
      };
      let numbered_before = numbered_kept + queued_author_count();
      let kinds: Vec<AuthorLineKind> = entries.iter().map(|(kind, ..)| *kind).collect();
      // Each link line's owner, and the authors (by their order in the block) it names: an author's or a column's
      // link goes to them by labels of their own (`placed_label`), placed after the marked affiliations above it, which
      // the labels place at the end — the contacts in the order the column prints them (the r5 re-review's u14).
      let made = |at: usize| {
        kinds[..at.min(kinds.len())]
          .iter()
          .filter(|kind| **kind == AuthorLineKind::Author)
          .count()
      };
      // (a name making its creator itself takes its link as before, the label of its `\lx@add@author` reaching no one:
      // [`makes_its_creator`])
      let own_creator: Vec<bool> = entries
        .iter()
        .filter(|(kind, ..)| *kind == AuthorLineKind::Author)
        .map(|(_, author, _)| makes_its_creator(author.unlist_ref()))
        .collect::<Result<Vec<_>>>()?;
      let link_authors: Vec<(LinkOwner, std::ops::Range<usize>)> = (0..kinds.len())
        .map(|index| {
          if kinds[index] != AuthorLineKind::Link {
            return (LinkOwner::Block, 0..0);
          }
          let (start, end) = group_bounds(&group_starts, kinds.len(), index);
          // (the authors of the line's `\and` column, and the one made last)
          let column = made(start)..made(end);
          let owner = link_line_owner(&kinds, &group_starts, index, replace);
          let authors = match owner {
            LinkOwner::Current if column.len() == 1 => column,
            LinkOwner::Current => made(index).saturating_sub(1)..made(index),
            LinkOwner::Group(_) => column,
            // (a link printing an institution, unmarked under the marked block, is every author's: user ruling R12
            // 2026-10-20, the r5 re-review's u38; a URL the block's)
            LinkOwner::Block if link_prints_an_institution(&entries[index].1) => {
              0..made(kinds.len())
            },
            LinkOwner::Block | LinkOwner::Author(_) => 0..0,
          };
          let authors = if authors
            .clone()
            .any(|author| own_creator.get(author).copied().unwrap_or(true))
          {
            0..0
          } else {
            authors
          };
          (owner, authors)
        })
        .collect();
      let mut placed: Vec<Option<String>> = Vec::new();
      let mut placed_calls: Vec<Vec<Token>> = vec![Vec::new(); made(kinds.len())];
      for (_, authors) in &link_authors {
        for author in authors.clone() {
          if let (_, Some(call)) = placed_label(&mut placed, author)? {
            placed_calls[author].extend(call);
          }
        }
      }
      let mut author_at = 0;
      for (index, (kind, mut line, email)) in entries.into_iter().enumerate() {
        match kind {
          AuthorLineKind::Author => {
            correspondence_mark = None;
            let mut content = Invocation!(T_CS!("\\lx@author@withsup"), vec![Some(line)]).unlist();
            content.extend(std::mem::take(&mut placed_calls[author_at]));
            author_at += 1;
            if let Some(email) = email {
              content
                .extend(Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(email)]).unlist());
              // (an email contact, which advances the `labelseq` count a later address line is placed by)
              sequenced = sequenced.map(|offset| offset + 1);
            }
            calls.extend(
              Invocation!(T_CS!("\\lx@add@author"), vec![
                keyvals.clone(),
                Some(Tokens::new(content))
              ])
              .unlist(),
            );
          },
          AuthorLineKind::Affiliation => {
            // a note's own mark, not that of an affiliation a symbol-marked note was welded into (`$^*$Corresponding
            // author` reads as a continuation: `rewrite_symbol_superscripts`)
            correspondence_mark = correspondence_notes
              .contains(&index)
              .then(|| entry_mark(line.unlist_ref()).and_then(|mark| mark_label(&mark)))
              .flatten();
            let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(line)]);
            calls.extend(
              Invocation!(T_CS!("\\lx@add@affiliation"), vec![None, Some(withsup)]).unlist(),
            );
          },
          AuthorLineKind::Link => {
            correspondence_mark = None;
            let (owner, authors) = link_authors[index].clone();
            let url = is_url_line(&line) || link_prints_a_handle(&line, &author_keys);
            if authors.is_empty() {
              calls.extend(url_contact_call(line, owner, url)?);
            } else {
              for author in authors {
                let (label, _) = placed_label(&mut placed, author)?;
                calls.extend(url_contact_call(
                  line.clone(),
                  LinkOwner::Author(label),
                  url,
                )?);
              }
            }
          },
          AuthorLineKind::LabelledEmail => {
            if let Some((label, addresses)) = labelled_emails(&line)? {
              calls.extend(block_email_calls(
                label.as_ref(),
                addresses,
                &author_keys,
                EmailOwners {
                  corresponding: corresponding.map(|author| numbered_before + author),
                  marked: correspondence_mark
                    .as_ref()
                    .map(|label| {
                      authors_requesting(&author_marks, label)
                        .into_iter()
                        .map(|author| numbered_kept + author)
                        .collect()
                    })
                    .unwrap_or_default(),
                  numbered_before,
                },
              )?);
            }
            // (a note's mark decides one labelled email line, the next one)
            correspondence_mark = None;
            // (its contacts are labelled, not sequenced; a later address line is not placed by the count)
            sequenced = None;
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
            // as one contact (the prior behavior), as when an address spells the name of an author other than the one
            // at its place (2406.06326's `{zhangxy, jyzhou, hmmeng}@…`, by institution; 63m). OXIDIZED_DESIGN #52(j).
            // (the authors carrying the footnote symbols that lead the line, whose its addresses are when no name spells
            // any — the marks then no part of them: `Ann Able$^{1,*}$ … \\ $^*$zq77@x.edu`; main-session decision 3 on
            // the 77p r11 questions, the r8 re-review's finding 6, probe t3. A spelled address keeps the mark it shows,
            // 1910.00401's `$^{*}$ghunt@wm.edu`, 65e)
            let carriers: Vec<usize> = {
              let marks: Vec<Vec<usize>> = leading_symbol_labels(&line)
                .iter()
                .map(|label| authors_requesting(&author_marks, &s!("affiliation:{label}")))
                .collect();
              marks
                .first()
                .map(|first| {
                  first
                    .iter()
                    .filter(|author| marks.iter().all(|carriers| carriers.contains(author)))
                    .map(|author| numbered_kept + author + 1)
                    .collect()
                })
                .unwrap_or_default()
            };
            let addresses = email_addresses(&line).unwrap_or_default();
            let placement = match sequenced {
              // (in order; and when the block lists an address for each author at most, no address there goes to an
              // author another address of the block spells or an earlier line was given: 2609.34459's `mayhhu@…` is
              // not Yuanyang Zhu's, whose own address is on a later line — where the block lists more, an author may
              // have two, `\{xx,yy\}@x.edu \\ ann.able@gmail.com` under Ann Able, Bob Baker, the r4 re-review's y1)
              // (every address spelling the author at its place: an address no name spells is never placed by its
              // position, user ruling R10 2026-10-20 — it goes by elimination where the order confirms it, else to the
              // block)
              Some(offset)
                if author_count >= 1
                  && addresses.len() <= author_count
                  && addresses_in_order(&author_keys, &addresses, offset)
                  && addresses.iter().enumerate().all(|(at, address)| {
                    spelled_owner(&author_keys, address) == Some(offset + at)
                  })
                  && (block_addresses.len() > author_count
                    || addresses.iter().enumerate().all(|(at, address)| {
                      !taken.get(offset + at).copied().unwrap_or(false)
                        || spelled_owner(&author_keys, address) == Some(offset + at)
                    })) =>
              {
                sequenced = Some(offset + addresses.len());
                for owner in offset..offset + addresses.len() {
                  if let Some(owner) = taken.get_mut(owner) {
                    *owner = true;
                  }
                }
                unplaced[index] = 0;
                AddressPlacement::Sequence
              },
              // (else each address to the author it spells, the rest by elimination — `owners_by_elimination`:
              // 2603.24730's `\{yuqihu,~ahna,~vasha\}@berkeley.edu \ \ \ \{vasha,~j8675309\}@mit.edu` under four
              // authors; the addresses no name settles are the block's, on its shared creator (user ruling R10
              // 2026-10-20; were one contact on the last author), the line's
              // ties and spacing commands printed as the spaces they are, not as "~~~" (2603.00717's
              // `{\tt\small D250201011@stu.cqupt.edu.cn ~~~ boliu@cqupt.edu.cn}`; 2605.06264's `cryexplorer@…`
              // between two spelled addresses with four names between their owners), and every address a name
              // settles goes to that name (main-session decision 2026-10-20, 77p r5: 2609.15028's `zihanxue@…`,
              // `serhii@…` beside an address no name spells, 2609.05913, 2609.37829, 2609.38070)
              _ => {
                sequenced = None;
                // (not while another e-mail line holds an address no name spells and no rule placed: 2602.23777)
                let elsewhere_open = unplaced
                  .iter()
                  .enumerate()
                  .any(|(at, count)| at != index && *count > 0);
                let owners =
                  owners_by_elimination(&author_keys, &addresses, &taken, elsewhere_open);
                // (a name an address went to leaves the pool of the lines after: 2407.00382)
                for owner in owners.iter().flatten() {
                  taken[*owner] = true;
                }
                unplaced[index] = owners
                  .iter()
                  .zip(&addresses)
                  .filter(|(owner, address)| owner.is_none() && !is_role_mailbox(address))
                  .count();
                if !addresses.is_empty() && owners.iter().all(Option::is_some) {
                  AddressPlacement::Authors(
                    owners
                      .into_iter()
                      .flatten()
                      .map(|owner| numbered_before + owner + 1)
                      .collect(),
                  )
                } else {
                  line = Tokens::new(
                    line
                      .unlist()
                      .into_iter()
                      .map(|t| {
                        if is_spacing_command(&t) {
                          T_SPACE!()
                        } else {
                          t
                        }
                      })
                      .collect(),
                  );
                  if owners.iter().any(Option::is_some) {
                    AddressPlacement::Settled(
                      owners
                        .into_iter()
                        .map(|owner| owner.map(|owner| numbered_before + owner + 1))
                        .collect(),
                    )
                  } else {
                    AddressPlacement::Shared
                  }
                }
              },
            };
            let placement = match placement {
              AddressPlacement::Shared if !carriers.is_empty() => {
                unplaced[index] = 0;
                line = without_leading_symbol_marks(&line);
                AddressPlacement::Marked(carriers)
              },
              placement => placement,
            };
            calls.extend(email_line_calls(line, placement)?);
          },
        }
      }
    } else {
      // No superscript markers. Split into author GROUPS on the \and family /
      // \quad only — NOT comma (author_group_splits). Within a group, the first
      // \\-delimited line is the author-name list (comma / " and "-split via
      // split_author_line); each remaining \\-line is an affiliation of every name
      // in that list, as LaTeX prints it under them (`annotate=N`, the group rule of
      // Perl's revtex `\affiliation`, `annotate=new`; 0911.0568's five LPT Orsay
      // names over one line, 63m). Splitting groups on comma shredded multi-part addresses
      // ("…Laboratory, Laurel, MD 20723") into fake authors and mislabeled the
      // trailing \email line as an affiliation. OXIDIZED_DESIGN #52 (surpass-Perl,
      // Perl's @authorsplits shares the comma); witness arXiv:2606.00315.
      // (a group: its names and the lines under them)
      type Group = (Vec<Tokens>, Vec<Tokens>);
      // Each block's groups, with whether only horizontal space (`\quad`, `\qquad`, the `\hspace`/`\hfill` and spacing
      // runs `normalize_hspace_separators` makes `\quad`) parts it from the block before.
      let mut blocks: Vec<(bool, Vec<Group>)> = Vec::new();
      // (the block's link lines no names stand over: the whole block's URL contacts, after its names)
      let mut block_links: Vec<Tokens> = Vec::new();
      // (the lines under such links, the block's affiliations)
      let mut block_lines: Vec<Tokens> = Vec::new();
      let mut spaced = true;
      for (delimiter, block) in split_tokens_delimited(stuff, author_group_splits()) {
        spaced &= delimiter
          .first()
          .is_some_and(|d| *d == T_CS!("\\quad") || *d == T_CS!("\\qquad"));
        let block = unwrap_alignment_environment(block);
        // (a block that is only the word "and", in any case or wrapper, separates the names beside it — never a name
        // or their affiliation: `\href{…}{…Eric Rossetto} \quad \textnormal{and} \quad \href{…}{…}`, 2608.24427)
        if block.is_empty() || is_and_piece(block.unlist_ref()) {
          continue;
        }
        let mut pairs = Vec::new();
        // Drop empty `\\`-pieces up front. A group that BEGINS with `\\` — because
        // the previous author line ended with a trailing `\quad\\` (or `\and\\`) that
        // leaked the line-break into this group — would otherwise give an EMPTY
        // names_line, demoting the group's real first author to an affiliation. The
        // first NON-empty piece is the name list; the rest are affiliations. Witness
        // arXiv 2507.06670 (acl): `…Zhiyuan Zhu\quad \\ \textbf{Ruiqi Li}\quad…`
        // rendered "Ruiqi Li" as an empty `<personname/>` + a bold "Ruiqi Li"
        // affiliation; now "Ruiqi Li" is an author. (A `\\`-only block still yields a
        // single empty author below, so its affiliations are not dropped.)
        // So is a line that prints nothing ([`spacing_end`]: `\author[2]{\vspace{2mm}\\O.~Ennis, U.~Chauhan, …}`,
        // 2504.09158, read as the name with the names below it its affiliation).
        let pieces = split_tokens(block, author_line_breaks());
        let opens_with_break = pieces.len() > 1
          && pieces
            .first()
            .is_some_and(|l| spacing_end(l.unlist_ref()) == 0);
        let mut lines: Vec<Tokens> = pieces
          .into_iter()
          .filter(|l| spacing_end(l.unlist_ref()) > 0)
          .flat_map(link_run_lines)
          .collect();
        // …but a block the line break opens whose first line is an institution a footnote symbol of the names before it
        // labels is lines under those names, no names (`Bob Baker$^\sharp$\footnote{…}\hspace{.2cm}\\ {\it \normalsize
        // $^\star$School of Mathematics …}\\ {\it \normalsize $^\sharp$Department of Biostatistics …}`, 2602.16540: a
        // creator named "⋆School of Mathematics …"; the r7 re-review's finding 1)
        // …and so is a block only horizontal space parts from a group's lines, the next institution on their printed
        // line (`Cat Cole$^{\dagger}$ \\ $^{\dagger}$Univ A \qquad $^{\ddagger}$Univ B`, the r7 re-review's g2, x13:
        // a creator "‡Univ B")
        let after_lines = spaced
          && blocks
            .last()
            .and_then(|(_, groups)| groups.last())
            .is_some_and(|(_, affils)| !affils.is_empty());
        // (an institution, a name led by its mark no part of it: `$^\dagger$Ann Able \\ $^\dagger$Univ A \quad
        // $^\ddagger$Bob Baker`)
        let earlier: Vec<Vec<String>> = blocks
          .iter()
          .flat_map(|(_, groups)| groups.iter())
          .flat_map(|(names, _)| names.iter().map(symbol_labels))
          .collect();
        let labelled = |line: &Tokens| {
          symbol_led_segments(line).iter().any(|segment| {
            labelled_institution(segment, &earlier)
              .is_some_and(|(_, rest)| reads_as_affiliation(&rest))
          })
        };
        if (opens_with_break || after_lines)
          && lines.first().is_some_and(labelled)
          && let Some((_, groups)) = blocks.last_mut()
          && let Some((_, affils)) = groups.last_mut()
        {
          // (only the lines down to a line of names, which with the lines under it is the next author's: `… \quad
          // $^\ddagger$Univ B \\ Cat Cole \\ Univ C`, the r8 re-review's finding 1, probes w1, w2 — Cat Cole had been an
          // affiliation, Univ C every name's of the row; no paper in its scan of 115k. A line reads as names there only
          // when it reads as names, no institution ([`reads_as_affiliation`] reads none in a line of names), and as no
          // note — correspondence, a contribution, funding ([`classify_thanks`]) —, no mark leads it (a legend or a
          // label: `$^*$Project Lead`, `$^\S$Google DeepMind`, the rev77y2 re-review's finding 2, probes m2, m4), and
          // either carries a symbol mark of its own after its text (w2's `Cat Cole$^{\S}$ \quad Dan Dorn$^{\S}$`), has
          // an institution under it that does not go on with its words (w1's Univ C; not `Shanghai Jiao Tong \\
          // University`, `Hong Kong \\ The Hong Kong University …`, [`continues_the_line`]), or an e-mail address its
          // names spell (`Cat Cole \\ \texttt{ccole@x.edu}`, [`addresses_spelling`]; the rev77y2 re-review's finding 4,
          // probes k12, k15, k17): a line of capitalized words alone under the row is no name — `Google DeepMind`,
          // `Hong Kong`, `Equal Contribution` stay the block's lines, the 77y review's finding 2, probes k1, k7, k9)
          let names_at = (1..lines.len())
            .find(|at| {
              let line = &lines[*at];
              let text = visible_name_text(line.unlist_ref());
              !labelled(line)
                && !line_is_email(line)
                && !is_link_line(line)
                && name_count(&text) > 0
                && classify_thanks(&text) == "note"
                && after_symbol_mark(line.unlist_ref()).len() == line.unlist_ref().len()
                && (!symbol_marks(line).is_empty()
                  || lines.get(at + 1).is_some_and(|under| {
                    (reads_as_institution(under) && !continues_the_line(line, under))
                      || addresses_spelling(line, under)
                  }))
            })
            .unwrap_or(lines.len());
          let names: Vec<Tokens> = lines.split_off(names_at);
          affils.append(&mut lines);
          if names.is_empty() {
            spaced = true;
            continue;
          }
          // (a line break, no space, parts the names from the lines above)
          lines = names;
          spaced = false;
        }
        // A link line printing a URL is no name ([`prints_a_url`]; a linked name is one, 2608.24427): one opening the
        // block goes with the lines under its names
        // (`\url{…} \\ Ann Able \\ Univ A`, the r5 re-review's u17), a block of link lines alone is the whole block's
        // (`Ann Able \\ Univ A \and \url{…}`, u16), not a creator named by its URL.
        let leading_links = (0..lines.len())
          .take_while(|at| prints_a_url(&lines[*at]))
          .count();
        if leading_links == lines.len() && !lines.is_empty() {
          block_links.append(&mut lines);
          continue;
        }
        // (and so is a column of links over an institution, no name under them: `\href{…}{Hugging Face Demo} \\ Univ
        // B`, the rev77y2 re-review's finding 3, probes l1-l3 — not a creator named by its institution)
        if leading_links > 0 && lines.get(leading_links).is_some_and(reads_as_institution) {
          block_lines.extend(lines.drain(leading_links..));
          block_links.append(&mut lines);
          continue;
        }
        let mut links: Vec<Tokens> = lines.drain(..leading_links).collect();
        for (names_line, mut affils) in name_groups(lines) {
          affils.splice(0..0, std::mem::take(&mut links));
          let mut names = split_names(names_line);
          if names.is_empty() {
            names.push(Tokens::default());
          }
          pairs.push((names, affils));
        }
        blocks.push((spaced, pairs));
        spaced = true;
      }
      // Names parted by horizontal space alone share the address lines under them, as names parted by a comma do in
      // one group: `Yuxi Fu \ \ \ Han Zhu \\ BASICS, Shanghai Jiao Tong University` gives the address to both, as
      // pdflatex prints it under the whole row (user ruling 2026-10-19, batch 77p r2; OXIDIZED_DESIGN #52(o); 1508.00093,
      // 1009.3124, 1202.3179, 1503.08895; 2304.13998 waits on its user-macro marks). Perl's `@authorsplits` parts the
      // names at `\quad` as at the comma and gives the lines to the last name (Base_Utility.pool.ltxml:676). A space before the line break ends no list
      // (`Zhiyuan Zhu\quad \\ \textbf{Ruiqi Li} \quad …\\ Zhejiang University`, 2507.06670), as a comma there does not
      // ([`names_continue`]). A name with lines of its own keeps them (`A \\ Univ A \quad B \\ Univ B`), and so does a
      // row of names whose address line is parted by space too (`A \quad B \\ Univ A \quad Univ B`, set by position;
      // nor does a piece of that row take the names after it, 2609.25833's `\texttt{a@x} \quad \texttt{b@y}`), and so
      // do names whose symbol marks label the lines (`Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\
      // $^{\dagger}$Univ A \\ $^{\ddagger}$Univ B`, 2505.23260, 2109.13296: the marks decide, not the row).
      let mut groups: Vec<Group> = Vec::new();
      // (by group, whether its lines' symbol marks kept it from the names before it: lines under the whole row, each
      // placed by its marks across the block, [`apart_line_calls`])
      let mut apart: Vec<bool> = Vec::new();
      // (and whether it opens a row of names — the block's first group, one after an `\and`, a names line under lines —
      // or goes on with the names before it on their printed line, joined by space or kept apart by its lines' marks: the
      // names an unmarked line under a marked row, and the symbol carriers of its marked lines, are the row's, the r6
      // re-review's x6)
      let mut row_starts: Vec<bool> = Vec::new();
      let next_spaced: Vec<bool> = blocks
        .iter()
        .skip(1)
        .map(|(spaced, _)| *spaced)
        .chain(std::iter::once(false))
        .collect();
      // (whether the last group's names may join the block after it: not a piece of a space-parted address row)
      let mut joinable = false;
      for ((spaced, mut pairs), next_spaced) in blocks.into_iter().zip(next_spaced) {
        let after_lines = groups.last().is_some_and(|(_, affils)| !affils.is_empty());
        let by_position = next_spaced && pairs.last().is_some_and(|(_, affils)| !affils.is_empty());
        let marked_lines = |earlier: &[Tokens], first: &Group| {
          let line_marks: Vec<String> = first.1.iter().flat_map(symbol_marks).collect();
          !line_marks.is_empty()
            && earlier
              .iter()
              .chain(&first.0)
              .flat_map(symbol_marks)
              .any(|mark| line_marks.contains(&mark))
        };
        let mut kept_apart = false;
        let mut opens_row = !spaced;
        if spaced
          && joinable
          && !by_position
          && let Some((earlier, _)) = groups.last()
          && let Some(first) = pairs.first()
        {
          kept_apart = marked_lines(earlier, first);
          if !kept_apart
            && let Some((mut names, _)) = groups.pop()
            && let Some((first_names, _)) = pairs.first_mut()
          {
            apart.pop();
            opens_row = row_starts.pop().unwrap_or(true);
            names.append(first_names);
            *first_names = names;
          }
        }
        joinable =
          !(spaced && after_lines) && pairs.last().is_some_and(|(_, affils)| affils.is_empty());
        apart.extend((0..pairs.len()).map(|at| at == 0 && kept_apart));
        row_starts.extend((0..pairs.len()).map(|at| at > 0 || opens_row));
        groups.extend(pairs);
      }
      // Every group's names are known before any line is placed, so a line of more addresses than its group has names
      // gives each to the name it spells in any group of the block (`\author{Rose Bohrer \and Ashe Neth\\ … \\
      // \texttt{\{rbohrer,aneth\}@wpi.edu}}`, 2409.18978; 2403.02271, 2402.00414, 63m); a line's addresses otherwise
      // look only at its group's names (2409.00286's `Chengxi Li\\ … \\ \texttt{chengxil@…}` is not Zexin Chen's).
      // A name line's trailing address is the name's email (`\name Ann Able \email ann@uni.edu`, the JMLR family).
      let groups = groups
        .into_iter()
        .map(|(names, affils)| {
          let names = names
            .iter()
            .map(name_and_trailing_email)
            .collect::<Result<Vec<_>>>()?;
          Ok((names, affils))
        })
        .collect::<Result<Vec<_>>>()?;
      let name_keys: Vec<NameKeys> = groups
        .iter()
        .flat_map(|(names, _)| names.iter().map(|(name, _)| NameKeys::of(name)))
        .collect();
      // (the footnote symbols each name carries, for the lines they label, [`apart_line_calls`])
      let name_symbols: Vec<Vec<String>> = groups
        .iter()
        .flat_map(|(names, _)| names.iter().map(|(name, _)| symbol_labels(name)))
        .collect();
      let mut bodies: Vec<Vec<Token>> = Vec::with_capacity(name_keys.len());
      // (the calls after each name's: an address of a name before its group's last, given to it by a label of its own
      // once the group's lines, which its last name's body gives to all of them, are placed — the contacts in the order
      // the lines print, `Ann Able, Bob Baker \\ Univ A \\ Correspondence: \href{mailto:bbaker@…}{…}`: Univ A, then the
      // address; the r5 re-review's u14)
      let mut after_bodies: Vec<Vec<Token>> = Vec::with_capacity(name_keys.len());
      // (the labels of the names a contact is placed on after the names are made, [`placed_label`])
      let mut placed: Vec<Option<String>> = Vec::new();
      let mut spans = Vec::with_capacity(groups.len());
      for (names, affils) in groups {
        let first = bodies.len();
        for (name, email) in names {
          let mut body = name.unlist();
          if let Some(email) = email {
            body.extend(Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(email)]).unlist());
          }
          bodies.push(body);
          after_bodies.push(Vec::new());
        }
        spans.push((first, bodies.len() - 1, affils));
      }
      // (the names an address on any e-mail line of the block spells, whom elimination gives no other address)
      let mut block_addresses: Vec<String> = Vec::new();
      // (and, by group and line, how many of a line's addresses no name spells and no rule has placed yet: while another
      // line holds one, no name goes by elimination, [`owners_by_elimination`] — another line of the group, for the
      // group's names; any line of the block, for all of them)
      let mut unplaced: Vec<(usize, usize)> = Vec::new();
      for (group, (_, _, affils)) in spans.iter().enumerate() {
        for line in affils {
          let texts = line_address_texts(line)?;
          unplaced.push((group, unspelled_addresses(&name_keys, &texts)));
          block_addresses.extend(texts);
        }
      }
      // (and those an earlier line's address went to by elimination: 2407.00382's `jw2423@…` and `ljcc0930@…` on lines of
      // their own are not both Joseph G. Wallwork's)
      let mut taken = names_spelled(&name_keys, &block_addresses);
      let mut line_at = 0;
      // (the first name of each group's row, [`row_starts`])
      let mut row_firsts: Vec<usize> = Vec::with_capacity(spans.len());
      for (group, (first, ..)) in spans.iter().enumerate() {
        let opens = group == 0 || row_starts.get(group).copied().unwrap_or(true);
        row_firsts.push(if opens { *first } else { row_firsts[group - 1] });
      }
      for (group, (first, last, affils)) in spans.into_iter().enumerate() {
        let kept_apart = apart.get(group).copied().unwrap_or(false);
        // (the names a line under a group kept apart from the names before it prints under: its whole row — an
        // unmarked line under a row whose only symbol line is a legend, every name's, user ruling R12 2026-10-20,
        // 2609.24299, 2603.06578; else the group's)
        let shared_first = if kept_apart { row_firsts[group] } else { first };
        // The names a line's addresses spell, or by elimination where the order confirms it ([`owners_by_elimination`]:
        // 1503.08895's `\{aszlam,jase,robfergus\}@fb.com`); an address under one name alone, printed in its column,
        // that name's; any other the block's, `None` (user ruling R10 2026-10-20: an address no name spells is never
        // the last name's).
        let owners_of = |addresses: &[&Tokens],
                         taken: &mut Vec<bool>,
                         unplaced: &mut Vec<(usize, usize)>,
                         line_at: usize|
         -> Vec<Option<usize>> {
          let texts: Vec<String> = addresses
            .iter()
            .map(|address| address_text(address))
            .collect();
          let whole_block = addresses.len() > last - first + 1 && name_keys.len() > 1;
          let elsewhere_open = unplaced
            .iter()
            .enumerate()
            .any(|(at, (of, count))| at != line_at && (whole_block || *of == group) && *count > 0);
          let owners: Vec<Option<usize>> = if whole_block {
            owners_by_elimination(&name_keys, &texts, taken, elsewhere_open)
          } else if last > first {
            owners_by_elimination(
              &name_keys[first..=last],
              &texts,
              &taken[first..=last],
              elsewhere_open,
            )
            .into_iter()
            .map(|owner| owner.map(|at| first + at))
            .collect()
          } else {
            vec![None; texts.len()]
          };
          for owner in owners.iter().flatten() {
            taken[*owner] = true;
          }
          if let Some((_, count)) = unplaced.get_mut(line_at) {
            *count = owners
              .iter()
              .zip(&texts)
              .filter(|(owner, address)| owner.is_none() && !is_role_mailbox(address))
              .count();
          }
          let alone = last == first && !kept_apart && !whole_block;
          owners
            .into_iter()
            .map(|owner| owner.or(alone.then_some(last)))
            .collect()
        };
        // (a group kept from the names before it whose lines' symbols label institutions: placed by them, a line at a
        // time, [`apart_line_calls`]; the symbols of the names of its row alone, an `\and` column's carriers no part of
        // it, the r6 re-review's x6, w28)
        let row = shared_first..=last;
        let row_symbols: Vec<Vec<String>> = name_symbols
          .iter()
          .enumerate()
          .map(|(at, symbols)| {
            if row.contains(&at) {
              symbols.clone()
            } else {
              Vec::new()
            }
          })
          .collect();
        // (and a group of names whose own symbols label its lines, `Ann Able$^\star$, and Bob Baker$^\sharp$ \\
        // $^\star$School … \\ $^\sharp$Department …`, 2602.16540: each line to the names carrying its symbol, not to
        // every name of the group; the r7 re-review's finding 1)
        let by_symbols = replace
          && (kept_apart || last > first)
          && affils.iter().any(|line| {
            symbol_led_segments(line)
              .iter()
              .any(|segment| labelled_institution(segment, &row_symbols).is_some())
          })
          && !bodies
            .iter()
            .map(|body| makes_its_creator(body))
            .collect::<Result<Vec<_>>>()?
            .contains(&true);
        // (the carriers a line goes on with, and whether the group's labelled institutions run on over unmarked lines,
        // [`welded_layout`])
        let mut continuing = Continuing {
          carriers:      Vec::new(),
          welded:        by_symbols && welded_layout(&affils, &row_symbols),
          labelled_left: affils
            .iter()
            .filter(|line| labelled_line(line, &row_symbols))
            .count(),
        };
        // (whether a line the group's names share, digested under its last name, came before: an address of an earlier
        // name after it is placed after it, by a label of its own, as the lines print — `Ann Able, Bob Baker \\ Univ A
        // \\ \{aable,bbaker\}@x.edu`; one before it, as an address row right under the names, stays in its name's
        // content, before it — 2602.05084's `\texttt{zuo.167@osu.edu} ~~~ \texttt{khalili.17@osu.edu}` over the
        // department)
        let mut shared_before = false;
        // (the block's, on its shared creator: an address no name spells, user ruling R10 2026-10-20)
        let block = s!("label=addresses:block");
        for line in &affils {
          let this_line = line_at;
          line_at += 1;
          let (line, annotations) = if cautious {
            take_author_annotations(line)
          } else {
            (line.clone(), Vec::new())
          };
          // (a line under a whole row of names, its marks those of names before its group: placed by them across the
          // block, [`apart_line_calls`]; 2601.02680)
          // (…but in a group of names not kept apart, whose own marks label its lines, an e-mail line goes by the
          // group's names as in any group, by spelling and the order that confirms it: 2603.07345's `\{mabansal, milind,
          // bgh, …\}@uber.com` under its comma-parted names, 77p r11)
          let group_email_line = by_symbols
            && !kept_apart
            && (line_labelled_emails(&line)?.is_some() || bare_email_addresses(&line)?.is_some());
          if group_email_line {
            continuing.carriers.clear();
          }
          if by_symbols && !group_email_line {
            shared_before = true;
            after_bodies[last].extend(apart_line_calls(
              &line,
              &name_keys,
              &row_symbols,
              row.clone(),
              &mut placed,
              &mut bodies,
              &mut continuing,
            )?);
            if let Some((_, count)) = unplaced.get_mut(this_line) {
              *count = 0;
            }
            bodies[last].extend(annotations);
            continue;
          }
          // An e-mail line, labelled (`E-mail: …`, `\textbf{Correspondence:} \href{mailto:…}{…}`) or bare
          // (\texttt{user@host}; `a@x, b@y`; `{a,b}@dom`), is the names' emails, not an affiliation (OXIDIZED_DESIGN #52;
          // witness arXiv:2606.00315): a shared one under several names gives each address to the one name it spells
          // (0911.0568's five PoS authors over one `E-mail:` line, 0911.0082's addresses in another order than the names,
          // 63k; 2401.15897, 2402.02746, 63m); an address naming no one, or several, is the block's (user ruling R10
          // 2026-10-20) — unless no address of the line names anyone and footnote symbols that names of the row carry
          // lead it: then its addresses, without the marks, are those names' ([`marked_email_line`]; main-session
          // decision 3 on the 77p r11 questions, probes s10, t4; a spelled address keeps the mark it shows, 65e).
          if let Some((label, addresses)) = line_emails(&line)? {
            let owners = owners_of(
              &addresses
                .iter()
                .map(|address| &address.address)
                .collect::<Vec<_>>(),
              &mut taken,
              &mut unplaced,
              this_line,
            );
            let marked = if owners.iter().all(Option::is_none) {
              match marked_email_line(&line, &row_symbols)? {
                Some((stripped, carriers)) => {
                  line_emails(&stripped)?.map(|found| (found, carriers))
                },
                None => None,
              }
            } else {
              None
            };
            let (label, placements): (Option<Tokens>, Vec<(EmailAddress, Vec<usize>)>) =
              match marked {
                Some(((label, addresses), carriers)) => {
                  if let Some((_, count)) = unplaced.get_mut(this_line) {
                    *count = 0;
                  }
                  (
                    label,
                    addresses
                      .into_iter()
                      .map(|address| (address, carriers.clone()))
                      .collect(),
                  )
                },
                None => (
                  label,
                  addresses
                    .into_iter()
                    .zip(owners)
                    .map(|(address, owner)| (address, owner.into_iter().collect()))
                    .collect(),
                ),
              };
            for (address, owners) in placements {
              if owners.is_empty() {
                after_bodies[last].extend(email_calls(
                  label.as_ref(),
                  vec![address],
                  Some(&block),
                )?);
                continue;
              }
              for owner in owners {
                let address = EmailAddress {
                  address: address.address.clone(),
                  call:    address.call.clone(),
                };
                if shared_before
                  && (shared_first..last + usize::from(kept_apart || by_symbols)).contains(&owner)
                  && !makes_its_creator(&bodies[owner])?
                {
                  let (placement, call) = placed_label(&mut placed, owner)?;
                  bodies[owner].extend(call.unwrap_or_default());
                  after_bodies[last].extend(email_calls(
                    label.as_ref(),
                    vec![address],
                    Some(&s!("label={placement}")),
                  )?);
                } else {
                  bodies[owner].extend(email_calls(label.as_ref(), vec![address], None)?);
                }
              }
            }
          } else if is_url_line(&line) || link_prints_a_handle(&line, &name_keys) {
            // A link line is a URL contact, not an affiliation of the names (user ruling 2026-10-20, batch 77p r7):
            // its one name's; the whole block's when the names it prints under are the document's whole author list
            // (a project's page, `Ann Able, Bob Baker \\ Univ A \\ \url{…}`); else its group's names', as the group's
            // other lines are — its row's, under a group kept apart from the names before it (R12).
            let owner = if last == shared_first {
              LinkOwner::Current
            } else if replace && shared_first == 0 && last + 1 == name_keys.len() {
              LinkOwner::Block
            } else {
              LinkOwner::Group(last - shared_first + 1)
            };
            shared_before |= matches!(owner, LinkOwner::Group(_));
            bodies[last].extend(url_contact_call(line, owner, true)?);
          } else if prints_nothing(&line) {
            // (a line that prints nothing is no entry, `~~~~` between the names and the institution: 2602.23649, the
            // r7 re-review's finding 6, as [`affiliation_calls`] skips it)
          } else if kept_apart && !line_is_email(&line) && spacing_end(line.unlist_ref()) > 0 {
            // (under a row whose only symbol line is a legend: an institution every name's of the row, a legend its
            // symbol's carriers', any other line the block's, as under a row whose symbols label its institutions —
            // [`row_line_calls`]; user ruling R12 2026-10-20, 2609.24299, 2603.06578, the r6 re-review's finding 7)
            after_bodies[last].extend(row_line_calls(
              &line,
              &row_symbols,
              row.clone(),
              &mut placed,
              &mut bodies,
              &[],
              &mut continuing,
            )?);
            shared_before = true;
          } else if spacing_end(line.unlist_ref()) > 0 {
            // (digested once, under the group's last name, and given to the group's names; an address line no list
            // reads, the block's, R10)
            let (cs, keyvals) = if line_is_email(&line) {
              let shared = (last > shared_first)
                .then(|| mouth::tokenize_internal(TeXString::assembled(block.clone())));
              (T_CS!("\\lx@add@email"), shared)
            } else {
              let shared = (last > shared_first).then(|| {
                mouth::tokenize_internal(TeXString::assembled(s!(
                  "annotate={}",
                  last - shared_first + 1
                )))
              });
              shared_before |= shared.is_some();
              (T_CS!("\\lx@add@affiliation"), shared)
            };
            bodies[last].extend(Invocation!(cs, vec![keyvals, Some(line)]).unlist());
          }
          bodies[last].extend(annotations);
        }
      }
      for (body, after) in bodies.into_iter().zip(after_bodies) {
        calls.extend(
          Invocation!(T_CS!("\\lx@add@author"), vec![
            keyvals.clone(),
            Some(Tokens::new(body))
          ])
          .unlist(),
        );
        calls.extend(after);
      }
      for link in block_links {
        let owner = if replace {
          LinkOwner::Block
        } else {
          LinkOwner::Current
        };
        calls.extend(url_contact_call(link, owner, true)?);
      }
      // (each by its kind: an address the block's e-mail, a link line printing a URL a URL contact, the rest
      // affiliations — `\href{…}{Project Page} \\ Univ B \\ \texttt{contact@x.org}`, the rev77y3 re-review's finding 3,
      // probe b9)
      for line in block_lines {
        if line_is_email(&line) {
          calls.extend(if replace {
            email_line_calls(line, AddressPlacement::Shared)?
          } else {
            Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(line)]).unlist()
          });
          continue;
        }
        // (a labelled address line, the block's e-mail under its label: `\\ Email: contact@x.org`, the rev77y4
        // re-review's finding 2, probe x2)
        if replace && let Some((label, addresses)) = line_labelled_emails(&line)? {
          calls.extend(block_email_calls(
            label.as_ref(),
            addresses,
            &[],
            EmailOwners {
              corresponding:   None,
              marked:          Vec::new(),
              numbered_before: 0,
            },
          )?);
          continue;
        }
        let owner = if replace {
          LinkOwner::Block
        } else {
          LinkOwner::Current
        };
        let url = is_url_line(&line);
        calls.extend(url_contact_call(line, owner, url)?);
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
  // (where the list starts in the queue: the row an address shared by its authors falls back to, `queued_row_start`)
  DefMacro!("\\lx@add@authors@list{}", sub[(stuff)] {
    assign_value("lx_authors_list_start", queued_author_count() as i64, Some(Scope::Global));
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
    if affiliation_is_marked(&stuff) {
      Ok(Tokens::new(affiliation_calls(None, None, stuff, true, true)?))
    } else {
      Ok(Invocation!(T_CS!("\\lx@add@affiliation"), vec![attr, Some(stuff)]))
    }
  });
  // Rust-only: an address a class prints under a row of names it shares (aistats's `\aistatsaddress` under
  // `\aistatsauthor{A \And B \And C}` when its columns are not the names', 1806.02924, 2510.09534): marked,
  // `\lx@add@affiliation@marked`'s; else each entry an affiliation of the authors the optional attributes place
  // (`annotate=run`), without its e-mail lines, whose addresses are the authors' e-mails as a marked list's are
  // (`shared_affiliation_calls`).
  DefMacro!("\\lx@add@affiliation@shared[]{}", sub[(attr, stuff)] {
    if affiliation_is_marked(&stuff) {
      Ok(Tokens::new(affiliation_calls(None, None, stuff, true, true)?))
    } else {
      Ok(Tokens::new(shared_affiliation_calls(attr, stuff)?))
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
    add_annotation_labels(label, true);
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

  // Rust-only: a label of the creator being made (inside its `\lx@add@author`), for a contact of the author block
  // placed on it after the names are made, in the order its lines print ([`placed_label`]; batch 77p r9). Only on a
  // creator, as `\lx@frontmatter@symbolmark`.
  DefPrimitive!("\\lx@author@placed@label{}", sub[(label)] {
    let label = label.to_string();
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
    // Not after the document body began over authors digested before it: those are the paper's, the queued ones a
    // supplement's, which the seal drops ([`seal_digested_frontmatter`]).
    if lookup_bool("lx_authors_superseded")
      && queued_author_count() > 0
      && lookup_mapping_int("frontmatter_sealed", &frontmatter_seal_key_of("ltx:creator", "author")) < FINAL_SEAL
    {
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
    // What the document printed of the marked fields before its `\maketitle` is their ink (`\lx@field@mark`).
    if lift_marked_field_ink(doc)? {
      assign_value("lx_title_ink_dedup_done", true, Some(Scope::Global));
    }
  },
  after_digest => {
    // A `\maketitle`'s flush: it may replace what the fallback flushed provisionally ([`frontmatter_sealed`]).
    let outer = lookup_bool("lx_frontmatter_explicit");
    assign_value("lx_frontmatter_explicit", true, Some(Scope::Global));
    let flushed = digest_front_matter();
    assign_value("lx_frontmatter_explicit", outer, Some(Scope::Global));
    flushed?;
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
      // The marked print of a field first (`\lx@field@mark`); the text match only for a copy no mark explains.
      if lift_marked_field_ink(document)? {
        assign_value("lx_title_ink_dedup_done", true, Some(Scope::Global));
      }
      maybe_dedup_leading_title_ink(document)?;
    }
  },
  after_digest => {
    // A raw class's stores the document set, read where the frontmatter is, before its queue is
    // digested (frontmatter_stores.rs): the authors' marks are still there to match.
    crate::frontmatter_stores::harvest_stores(false)?;
    // What the fallback flushes is sealed provisionally: a class that prints its frontmatter without `\maketitle`
    // (imsart's `{frontmatter}`) printed it, and a supplement's later frontmatter must not replace it; but where a
    // `\maketitle` follows a starred section, the PDF printed only that one ([`seal_digested_frontmatter`]).
    let outer = lookup_bool("lx_frontmatter_provisional");
    assign_value("lx_frontmatter_provisional", true, Some(Scope::Global));
    let flushed = digest_front_matter();
    assign_value("lx_frontmatter_provisional", outer, Some(Scope::Global));
    flushed?;
    assign_value("frontmatter_deferred", true, Some(Scope::Global));
  },
  // An internal hook (`\@startsection@hook`, the end-document hook): nothing of it belongs in a
  // reverted `tex=` (a math left open to `\end{document}` reverts through it).
  reversion => "");

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
    let mut lx_tags = vec![T_CS!("\\lx@tags"), T_BEGIN!()];
    for (_printed_here, form) in tag_forms(&ttype)? {
      lx_tags.extend(form.unlist());
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
  // The form is what latex.ltx:14963-14964 `\refstepcounter` stores in `\@currentlabel` and a `\ref` prints:
  // `\protected@edef` of `\csname p@<ctr>\expandafter\endcsname\csname the<ctr>\endcsname`, so `\p@<ctr>` is
  // applied once, to the expanded `\the<ctr>`, inside the definition's braces. Typed in place (Perl
  // Base_Utility.pool.ltxml:1028), a `\p@equation` taking nine arguments (`\renewcommand{\p@equation}
  // {\gobblesomeargs}`, a robust command, 1902.09345) read past the form's group: 18 errors where pdflatex has 0;
  // protected, it stays, and gobbles "Equation" of the expanded `\theequation` as the `\ref` does ("S1").
  // Only a `\p@<ctr>` that does something is applied so: with the kernel's empty one (`\@definecounter`, latex.ltx:10144) the
  // stored text is `\the<ctr>`'s expansion, which is typeset the same, and the number is typed as written — an
  // itemize's `\tikz` bullet (a number LaTeX never stores) does not survive an `\edef`.
  // OXIDIZED_DESIGN_DIVERGENCES #778; guards `perfect_kernel_batch77u::{p_form_applies_to_the_expanded_number,
  // p_forms_of_the_kernel_keep_their_reference}`, `perfect_kernel_batch56::itemize_label_is_used_as_written`.
  DefMacro!("\\lx@@p@the@@{}", sub[(ctr)] {
    let ctr = ctr.to_string();
    let mut form = vec![T_CS!("\\csname")];
    form.extend(Explode!(s!("p@{ctr}")));
    form.extend([T_CS!("\\expandafter"), T_CS!("\\endcsname"), T_CS!("\\csname")]);
    form.extend(Explode!(s!("the{ctr}")));
    form.push(T_CS!("\\endcsname"));
    let form = Tokens::new(form);
    let applies = match lookup_definition(&T_CS!(s!("\\p@{ctr}")))? {
      Some(defn) => !(defn.is_expandable()
        && defn.get_parameters().is_none_or(|params| params.get_num_args() == 0)
        && match defn.get_expansion() {
          None => true,
          Some(ExpansionBody::Tokens(body)) => body.is_empty(),
          Some(_) => false,
        }),
      None => false,
    };
    if applies {
      with_unexpandable_protect(|| do_expand_partially(form))?
    } else {
      form
    }
  });

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
      Ok(Some(def)) => is_name_noun(&def, &mut vec![t]),
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
  // A heading passes its title braced (sect04.rs, latex.ltx:17256 `#6{…}`); a caption does not.
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
  // A digestion that re-reads a source typeset where it stands (`tex_math::REREAD_COPY`): a reference form of a tag
  // (`\lx@make@tags`), the toc copy of a heading or caption without a `[short]`.
  DefPrimitive!("\\lx@reread@copy", {
    assign_value(crate::tex_math::REREAD_COPY, true, Some(Scope::Local));
  });
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
  // What an unnumbered heading's title format runs after the title (sect04.rs `\@@unnumbered@section`), as a
  // numbered one's `\format@title@<type>` does: titlesec's after-code (titlesec.sty:756, :780, :816; the numberless
  // title is the same `\ttlf@<type>`), which closes what its format opened (tufte-common.def:1613-1619
  // `\begin{fullwidth}` … `\end{fullwidth}`; 1905.13490's run-in `\subsection*` its `[.]`). Rust-only (Perl's starred title runs the font alone,
  // latex_constructs.pool.ltxml:666-670, and titlesec.sty.ltxml:42-57 drops the after-code; OXIDIZED_DESIGN_DIVERGENCES #785).
  DefMacro!(
    r"\lx@format@title@after@@{}",
    r"\@ifundefined{format@title@after@#1}{}{\csname format@title@after@#1\endcsname}"
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
  let stuff = rewrite_ordinal_superscripts(strip_linebreak_options(unalias_mark_commands(stuff)));
  let with_sup = marks_are_labels && author_mark_position(&stuff).is_some();
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
    // Mirrors \lx@add@authors. So is a line that prints nothing ([`prints_nothing`]: IEEEtran's
    // `\end{@IEEEauthorhalign}\hfill\strut`, 2309.15562, an empty affiliation of the last author).
    if line.is_empty() || prints_nothing(&line) {
      continue;
    }
    if with_sup {
      let across_and = delimiter.iter().any(|t| {
        author_and_splits()
          .iter()
          .any(|d| matches!(d, SplitDelim::Token(a) if a == t))
      });
      if authors_queued && author_mark_position(&line).is_none() && line_is_email_list(&line) {
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
        && author_mark_position(&line).is_none()
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
        if spacing_end(seg.unlist_ref()) == 0 {
          continue;
        }
        if let Some(last) = marked_entries.last_mut() {
          *last = trim_trailing_separator(std::mem::take(last));
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
  let email_calls = place_address_runs(address_runs, &mut marked_entries, 0)?;
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

/// Whether an affiliation a class gives after its authors is a marked list ([`affiliation_calls`]): it has a mark, and the
/// queued authors carry marks — or none is queued yet, when the marks are labels for the authors to come (an affiliation
/// without its label had no one to go to).
fn affiliation_is_marked(stuff: &Tokens) -> bool {
  (queued_creators_have_marks() || queued_creator_count() == 0)
    && author_mark_position(stuff).is_some()
}

/// The calls for an unmarked address shared by the authors before it (`\lx@add@affiliation@shared`): each of its
/// entries (separated by the `\and` family, `\And` the column break of the conference templates) one affiliation,
/// attributed by `attr` — one for entries printed alike (1302.5348's `\UMDAddress \\ … \And \UMDAddress \\ …`) —
/// without its e-mail lines, whose addresses, all of them, are one run placed as a marked list's are
/// ([`place_address_runs`]: one for each author of the row, in their order, else the block's, one contact an address —
/// the row the authors queued after the last affiliation, as `annotate=run` attaches one: 2302.10910's four e-mails of
/// a later row go to its four authors, not to the block for not being one for each queued author) — not lines of the
/// affiliation (1806.02924's `Carnegie Mellon University\\ \{asuggala, adarshp, vaishnavh, pradeepr\}@cs.cmu.edu`,
/// 1311.1894's `\texttt{\small \{honda,takemura\}@…}`), as the authors' own addresses are in a paired block
/// (2610.11580).
fn shared_affiliation_calls(attr: Option<Tokens>, stuff: Tokens) -> Result<Vec<Token>> {
  let authors_queued = queued_creator_count() > 0;
  let mut entries: Vec<Vec<Token>> = Vec::new();
  let mut run = AddressRun { lines: Vec::new(), under: None };
  for entry in split_tokens(strip_linebreak_options(stuff), author_and_splits()) {
    let mut kept: Vec<Token> = Vec::new();
    for (delimiter, line) in split_tokens_delimited(entry, author_line_breaks()) {
      if authors_queued && line_is_email_list(&line) {
        run.lines.push((delimiter, line));
      } else if spacing_end(line.unlist_ref()) > 0 {
        if !kept.is_empty() {
          kept.extend(delimiter);
        }
        kept.extend(line.unlist());
      }
    }
    let lead = kept
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count();
    kept.drain(..lead);
    while kept
      .last()
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      kept.pop();
    }
    if !kept.is_empty() && !entries.contains(&kept) {
      entries.push(kept);
    }
  }
  let mut calls: Vec<Token> = Vec::new();
  for entry in entries {
    calls.extend(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        attr.clone(),
        Some(Tokens::new(entry))
      ])
      .unlist(),
    );
  }
  if !run.lines.is_empty() {
    calls.extend(place_address_runs(vec![run], &mut [], queued_row_start())?);
  }
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
///   `\institute{$^1$A \\ $^2$B \\ \email{a@x} \\ \email{b@y}}`), any other is one contact an address on the shared
///   creator below the authors (#159) — by order, 2609.06094's four addresses for six authors had given Jindong
///   Gu's to Runjia Li, and by the mark of the affiliation they follow, three of them to the wrong author.
///
/// `row_start` is the queue position the authors the list belongs to start after (0: every queued author).
fn place_address_runs(
  runs: Vec<AddressRun>,
  entries: &mut [Vec<Token>],
  row_start: usize,
) -> Result<Vec<Token>> {
  let authors = queued_author_marks();
  let row_start = row_start.min(authors.len());
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
      _ if total == authors.len() - row_start && run_in_author_order(&run.lines, row_start) => {
        let mut positions = row_start + 1..;
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

/// The queue positions of the authors whose marks request the affiliation label `mark` sets. An author's mark reads as
/// its spelling (a symbol answered by its own line: OSA's `A. Able,$^{1,*}$` and `$^*$e-mail: …`, guard
/// `perfect_kernel_batch64u::osa_email_line_mark_shapes`) and as its request reads it ([`label_pieces`]): a glued
/// `1*`/`1\dagger` asks for label 1 too (77n r7 review probe g8: `Bob$^{2*}$` answered by `\affil{$^2$Y}`, not by
/// authblk's run).
fn requesting_authors(authors: &[Vec<Vec<Token>>], mark: &[Token]) -> Vec<usize> {
  let Some(label) = mark_label(mark) else {
    return Vec::new();
  };
  authors
    .iter()
    .enumerate()
    .filter(|(_, marks)| {
      marks.iter().any(|operand| {
        let reads = |operand: Vec<Token>| {
          clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation")
            .contains(&label)
        };
        reads(operand.clone()) || label_pieces(operand).is_some_and(reads)
      })
    })
    .map(|(i, _)| i + 1)
    .collect()
}

/// The name a queued author entry holds: the content of its `\lx@personname` (inside `\lx@author@withinst`), not the
/// `\lx@add@frontmatter@now{ltx:creator}[role=author]` around it, whose words read as surnames ("author" spelled in
/// `Corresponding author: …`; guard `perfect_kernel_batch64u::osa_style_address_email_line`).
fn queued_name(entry: &Tokens) -> Tokens {
  let tokens = entry.unlist_ref();
  for cs in ["\\lx@author@withinst", "\\lx@personname"] {
    if let Some(at) = tokens.iter().position(|t| *t == T_CS!(cs))
      && tokens
        .get(at + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      let mut depth = 0usize;
      for (j, t) in tokens.iter().enumerate().skip(at + 1) {
        match t.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth == 0 {
              return Tokens::new(tokens[at + 2..j].to_vec());
            }
          },
          _ => {},
        }
      }
    }
  }
  entry.clone()
}

/// The keys ([`NameKeys`]) of each queued author's name ([`queued_name`]), in queue order (the n-th digests
/// with `author:n`).
fn queued_author_name_keys() -> Vec<NameKeys> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .map(|entry| NameKeys::of(&queued_name(&entry.2)))
      .collect(),
    _ => Vec::new(),
  })
}

/// Where a contact placed apart from the names goes ([`placed_contact_owner`]).
enum ContactOwner {
  /// to the authors requesting the mark's label, which pairs them as an affiliation's mark does (and so the mark,
  /// answered, is not left standing on the name)
  Mark(String),
  /// to the author at this queue position (from 1)
  Author(usize),
  /// to the whole block, the shared creator below the authors (#159)
  Block,
}

/// The owner of a contact placed apart from the names (`\lx@add@contact@placed`), in this order:
/// 1. the mark written on the contact itself (`$^*$x@y`), when one queued author requests it — the contact's own mark
///    is the strongest evidence, stronger than the spelling (`*b@x` under `A$^*$` and `B` is A's); when several
///    request it, the one of them the address spells (1312.7046);
/// 2. the author whose surname the address's local part spells ([`spelled_owners`]: 0901.2914);
/// 3. the only author of the current group (`\lx@frontmatter@group@start`; the whole queue when no class marks
///    groups);
/// 4. else the block's.
///
/// An author block's labelled email lines rank otherwise ([`block_email_calls`]): there the spelling outranks the mark
/// of a correspondence NOTE before the line (1710.00802's `aleder@` under another author's "Corresponding Author"),
/// a mark on a nearby note being weaker evidence than one on the address.
fn placed_contact_owner(mark: &Tokens, content: &Tokens) -> ContactOwner {
  let authors = queued_author_marks();
  let keys = queued_author_name_keys();
  let requesting = if mark.is_empty() {
    Vec::new()
  } else {
    requesting_authors(&authors, mark.unlist_ref())
  };
  let spelled: Vec<usize> = address_owners(&keys, content)
    .into_iter()
    .map(|at| at + 1)
    .collect();
  if let [author] = requesting.as_slice() {
    // (a footnote-symbol mark stays shown on the name and pairs by no label of its own: the author's position)
    return match mark_label(mark.unlist_ref()) {
      Some(label) if !is_symbol_mark(mark.unlist_ref()) => ContactOwner::Mark(label),
      _ => ContactOwner::Author(*author),
    };
  }
  if let [author] = requesting
    .iter()
    .copied()
    .filter(|author| spelled.contains(author))
    .collect::<Vec<_>>()
    .as_slice()
  {
    return ContactOwner::Author(*author);
  }
  if let [author] = spelled.as_slice() {
    return ContactOwner::Author(*author);
  }
  let first = frontmatter_group_first();
  if authors.len() == first {
    return ContactOwner::Author(first);
  }
  ContactOwner::Block
}

/// `text` as tokens of its characters, each as it is: an ASCII letter a letter, anything else other (no `#` parameter, no
/// `%` comment, no `{`/`}` group: a keyval value that is text).
fn text_tokens(text: &str) -> Vec<Token> {
  text
    .chars()
    .map(|c| {
      let c = c.to_string();
      if c.chars().all(|c| c.is_ascii_alphabetic()) {
        T_LETTER!(&c)
      } else if c == " " {
        T_SPACE!()
      } else {
        T_OTHER!(&c)
      }
    })
    .collect()
}

/// The key a footnote-symbol mark (`$^\dagger$` on `$^\dagger$baker@y.edu`) answers on the author it places a contact
/// for ([`symbol_mark_key`]). `None` for any other mark.
fn answered_symbol_key(mark: &[Token]) -> Option<String> {
  if mark.is_empty() || !is_symbol_mark(mark) {
    return None;
  }
  let label = mark_label(mark)?;
  symbol_mark_key(label.strip_prefix("affiliation:").unwrap_or(&label))
}

/// A symbol mark's key, made of its glyph (`unanswered_mark_text`: `\dag` and `\dagger` alike, `\ast` and `*` alike)
/// as hexadecimal code points (`sym-2020`): only letters, digits and `-`, so it passes through a keyval's tokenization
/// as it is (a `#`, `\#`'s glyph, read back as a parameter token raised `Error:misdefined:#`).
fn symbol_mark_key(mark: &str) -> Option<String> {
  let glyph = unanswered_mark_text(mark)?.replace('\u{2217}', "*");
  (!glyph.chars().all(char::is_alphanumeric)).then(|| {
    s!(
      "sym-{}",
      glyph
        .chars()
        .map(|c| format!("{:x}", c as u32))
        .collect::<Vec<_>>()
        .join("-")
    )
  })
}

/// The queue position of the current author group's first author (`\lx@frontmatter@group@start`; 1 when no class marks
/// groups).
fn frontmatter_group_first() -> usize {
  match lookup_value("lx_frontmatter_group_first") {
    Some(Stored::Number(n)) => usize::try_from(n.value_of()).unwrap_or(1).max(1),
    _ => 1,
  }
}

/// Where the addresses of a line in an author block go.
enum AddressPlacement {
  /// The n-th address to the n-th author, counted over the emails so far (`labelseq=author`).
  Sequence,
  /// Each address to the author at that queue position, by the `author:N` label the creator digests with.
  Authors(Vec<usize>),
  /// Each address a name settles to the author at that queue position, as [`AddressPlacement::Authors`]; each other
  /// the block's, as [`AddressPlacement::Shared`] places it (2609.15028's `qq@…`-like address no name spells beside
  /// `zihanxue@…`, `serhii@…`; user ruling R10 2026-10-20). The line itself when no address is settled.
  Settled(Vec<Option<usize>>),
  /// The block's, on the shared creator below the authors (#159): its label names no creator. One contact an address
  /// when the line lists several (1802.07182's `\{jrr41,$\,$wct23,…\}@cam.ac.uk`, whose raw line a Semiverbatim
  /// contact showed as `{jrr41,$ $wct23,…}`), else the line.
  Shared,
  /// The line's, no address of which a name settles, to every author at these queue positions, those carrying the marks
  /// that led it (`$^*$zq77@x.edu` under `Ann Able$^{1,*}$`; main-session decision 3 on the 77p r11 questions;
  /// 2605.01818's `$^*$nbes@itu.dk` on the ∗ author).
  Marked(Vec<usize>),
}

/// The `\lx@add@email` calls for a line of addresses in an author block (`a@x, b@y`, `{a,b}@dom`), placed as
/// `placement` says: one call per address, a whole-line wrapper repeated on each, or the line itself when it holds one
/// address (`\href{mailto:a@x}{a@x}` stays a link); else the line as one contact. OXIDIZED_DESIGN #52(j).
fn email_line_calls(line: Tokens, placement: AddressPlacement) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  // A wrapper is kept on each address (`\texttt`, `\small`, `\url`), but an email command's (`\email`, `\mailto`,
  // `\emailaddr`) is the contact `\lx@add@email` stands for — nested, it made a second (empty) contact and advanced
  // the `labelseq` count twice, so the second address found no author (llncs `\institute{… \\ \email{a@x, b@y}}`).
  // (the address set as the command sets it, [`email_command_address`])
  let unwrapped = |line: Tokens| match whole_line_cs_wrapper(&line) {
    Some((cmd, inner)) if is_email_wrapper(&cmd) => email_command_address(&cmd, inner),
    _ => Ok(line),
  };
  let addrs = email_addresses(&line).unwrap_or_default();
  if let AddressPlacement::Marked(carriers) = &placement
    && !addrs.is_empty()
  {
    // (one address: the line itself, as written)
    let addresses = if addrs.len() == 1 {
      vec![unwrapped(line.clone())?]
    } else {
      address_tokens(&line, addrs)?
    };
    for address in addresses {
      for author in carriers {
        calls.extend(
          Invocation!(T_CS!("\\lx@add@email"), vec![
            Some(mouth::tokenize_internal(TeXString::assembled(s!(
              "label=author:{author}"
            )))),
            Some(address.clone())
          ])
          .unlist(),
        );
      }
    }
    return Ok(calls);
  }
  let placement = match placement {
    AddressPlacement::Settled(owners) if !addrs.is_empty() && owners.len() == addrs.len() => {
      if owners.iter().all(Option::is_none) {
        AddressPlacement::Shared
      } else if owners.iter().all(Option::is_some) {
        AddressPlacement::Authors(owners.into_iter().flatten().collect())
      } else {
        // (the settled ones to their authors, each other the block's, in the line's order)
        for (address, owner) in address_tokens(&line, addrs)?.into_iter().zip(owners) {
          let placement = match owner {
            Some(n) => s!("label=author:{n}"),
            None => s!("label=addresses:block"),
          };
          calls.extend(
            Invocation!(T_CS!("\\lx@add@email"), vec![
              Some(mouth::tokenize_internal(TeXString::assembled(placement))),
              Some(address)
            ])
            .unlist(),
          );
        }
        return Ok(calls);
      }
    },
    AddressPlacement::Settled(_) | AddressPlacement::Marked(..) => AddressPlacement::Shared,
    placement => placement,
  };
  let options: Vec<Option<Tokens>> = match placement {
    AddressPlacement::Sequence if !addrs.is_empty() => addrs
      .iter()
      .map(|_| Some(mouth::tokenize_internal("labelseq=author")))
      .collect(),
    AddressPlacement::Shared if addrs.len() > 1 => addrs
      .iter()
      .map(|_| Some(mouth::tokenize_internal("label=addresses:block")))
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
        .extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(unwrapped(line)?)]).unlist());
      return Ok(calls);
    },
  };
  if let [opts] = options.as_slice() {
    calls.extend(
      Invocation!(T_CS!("\\lx@add@email"), vec![
        opts.clone(),
        Some(unwrapped(line)?)
      ])
      .unlist(),
    );
    return Ok(calls);
  }
  for (address, opts) in address_tokens(&line, addrs)?.into_iter().zip(options) {
    calls.extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(address)]).unlist());
  }
  Ok(calls)
}

/// Whether `cmd` is an email command (`\email`, `\mailto`, `\emailaddr`): the contact `\lx@add@email` stands for.
fn is_email_wrapper(cmd: &Token) -> bool {
  cmd.with_str(|name| name.to_lowercase().contains("mail"))
}

/// The address of a call of the email command `cmd` (`\email{x}`) as that command sets it: in the font the class's
/// command gives its address, `\lx@address@urlfont` (llncs's `\email`, `{\tt#1}` at llncs.cls:897; amsart's, JHEP's)
/// or `\lx@address@textfont` (svjour3's, REVTeX 4.1's), else as written — so a contact the author block makes of the
/// call (`\institute{… \\ \email{a@x}}`, `E-mail: \email{x}`) is set as the command's own is. Only the command's own
/// expansion body is read, one level: a command reaching the address macro through another macro is not seen.
fn email_command_address(cmd: &Token, address: Tokens) -> Result<Tokens> {
  let fonts = [
    T_CS!("\\lx@address@urlfont"),
    T_CS!("\\lx@address@textfont"),
  ];
  let font = match lookup_definition(cmd)?
    .as_ref()
    .and_then(|def| def.get_expansion())
  {
    Some(ExpansionBody::Tokens(body)) => body
      .unlist_ref()
      .iter()
      .find(|t| fonts.contains(t))
      .copied(),
    _ => None,
  };
  Ok(match font {
    Some(font) => Invocation!(font, vec![Some(address)]),
    None => address,
  })
}

/// The addresses `addrs` of an email `line` as tokens, the line's whole-line wrapper (`\texttt`, `\small`, `\url`)
/// repeated on each — but not an email command's, the contact itself.
fn address_tokens(line: &Tokens, addrs: Vec<String>) -> Result<Vec<Tokens>> {
  let line_wrapper = whole_line_cs_wrapper(line).map(|(cmd, _)| cmd);
  let wrapper = line_wrapper.filter(|cmd| !is_email_wrapper(cmd));
  // (an email command's address set as the command sets it, [`email_command_address`])
  let email_command = line_wrapper.filter(is_email_wrapper);
  // (the line's links to a mailbox, by the address they reach: each address keeps its link, `\texttt{\{}\href{mailto:
  // Xinhao.Zhang@…}{…xinhao.zhang,} \href{mailto:maxime.peyrard@…}{…}\texttt{\}@…}`, 2604.19440)
  let v = line.unlist_ref();
  let links: Vec<(String, Vec<Token>)> = (0..v.len())
    .filter(|at| v[*at] == T_CS!("\\href"))
    .filter_map(|at| {
      let open = at + 1 + v[at + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      let target = v[open..skip_group(v, open).clamp(open, v.len())].to_vec();
      let (text, _) = link_texts(&v[at..], 0);
      let address = text.strip_prefix("mailto:")?.trim().to_lowercase();
      Some((address, target))
    })
    .collect();
  addrs
    .into_iter()
    .map(|addr| {
      let mut toks = mouth::tokenize(TeXString::assembled(addr.replace('_', "\\_"))).unlist();
      if let Some(cmd) = wrapper {
        let mut wrapped = vec![cmd, T_BEGIN!()];
        wrapped.append(&mut toks);
        wrapped.push(T_END!());
        toks = wrapped;
      }
      if email_command.is_none()
        && let Some((_, target)) = links
          .iter()
          .find(|(address, _)| *address == addr.to_lowercase())
      {
        let mut linked = vec![T_CS!("\\href")];
        linked.extend(target.iter().copied());
        linked.push(T_BEGIN!());
        linked.append(&mut toks);
        linked.push(T_END!());
        toks = linked;
      }
      match email_command {
        Some(cmd) => email_command_address(&cmd, Tokens::new(toks)),
        None => Ok(Tokens::new(toks)),
      }
    })
    .collect()
}

/// The end of `tokens` before the line break and spacing they end with, which print nothing: spaces, `\\`, `\hfill`,
/// `\smallskip`…, `\protect`, `\vspace{…}`, `\hspace*{…}` (2409.06765's `\email vye@berkeley.edu\\ \vspace{-0.6cm}`,
/// read as one line; 2609.08102's `\author[Mech]{\protect\\Johann Guilleminot}`); 0 for a line of only those.
fn spacing_end(tokens: &[Token]) -> usize {
  let spacing = |t: &Token, names: &[&str]| {
    t.get_catcode() == Catcode::CS && t.with_str(|s| names.contains(&s))
  };
  let mut end = tokens.len();
  loop {
    while end > 0 && tokens[end - 1] == T_SPACE!() {
      end -= 1;
    }
    if end > 0
      && spacing(&tokens[end - 1], &[
        "\\\\",
        "\\hfill",
        "\\smallskip",
        "\\medskip",
        "\\bigskip",
        "\\newline",
        "\\protect",
      ])
    {
      end -= 1;
      continue;
    }
    if end > 0 && tokens[end - 1].get_catcode() == Catcode::END {
      let mut depth = 0usize;
      let mut open = None;
      for j in (0..end).rev() {
        match tokens[j].get_catcode() {
          Catcode::END => depth += 1,
          Catcode::BEGIN => {
            depth -= 1;
            if depth == 0 {
              open = Some(j);
              break;
            }
          },
          _ => {},
        }
      }
      if let Some(open) = open {
        let mut cs = open;
        if cs > 0 && tokens[cs - 1] == T_OTHER!("*") {
          cs -= 1;
        }
        if cs > 0 && spacing(&tokens[cs - 1], &["\\vspace", "\\hspace"]) {
          end = cs - 1;
          continue;
        }
      }
    }
    return end;
  }
}

/// A name line and the address it ends with, as the JMLR family writes an author (`\name Ann Able \email ann@uni.edu`,
/// jmlr2e.sty:271-273; 2405.13980, 2407.04153, 2411.04991, 2502.14381, 2511.15722): the name, and the line's last run
/// without a space, the class's email markup with it (`\email ann@uni.edu`), when that run prints one plain address and a
/// name stands before it, its markup leading the address. A run with a note or a command that adds to the frontmatter
/// itself (`\thanks{ann@uni.edu}`, `\footnote{…}`, a class's `\email{x}` expanding to `\lx@add@email`, aa's `\mail`
/// adding a contact) is the author's as it stands, and a line that is only an address is kept whole.
fn name_and_trailing_email(line: &Tokens) -> Result<(Tokens, Option<Tokens>)> {
  let tokens = line.unlist_ref();
  let end = spacing_end(tokens);
  // the last run without a space outside braces
  let mut start = end;
  let mut depth = 0usize;
  while start > 0 {
    match tokens[start - 1].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN if depth == 0 => break,
      Catcode::BEGIN => depth -= 1,
      Catcode::SPACE if depth == 0 => break,
      _ => {},
    }
    start -= 1;
  }
  let whole = || Ok((line.clone(), None));
  if start == end || depth != 0 {
    return whole();
  }
  // the markup leads the address (`\email ann@uni.edu`, `\texttt{ann@uni.edu}`): a command or group after printed text
  // is not an address's (`Peng\thanks{bp2601@columbia.edu}`, 2402.08164, is a name and its note)
  let mut printed = false;
  for t in &tokens[start..end] {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => printed = true,
      Catcode::CS
        if printed
          && !matches!(
            t.with_str(str::to_string).as_str(),
            "\\_" | "\\textunderscore"
          ) =>
      {
        return whole();
      },
      Catcode::BEGIN | Catcode::ACTIVE if printed => return whole(),
      _ => {},
    }
  }
  let run = Tokens::new(tokens[start..end].to_vec());
  // (a name annotation other than the address's markup glued to it, `\orcidlink{…}ann@x.org`, `\cite{x}ann@x.org`, is
  // the name's, printed beside it: the run is no address, 77n r5 review probes n02, n04)
  if run.unlist_ref().iter().any(|t| {
    t.code == Catcode::CS
      && t.with_str(|name| name != "\\email" && NAME_ANNOTATIONS.contains(&name))
  }) {
    return whole();
  }
  let commands = run
    .unlist_ref()
    .iter()
    .filter(|t| t.code == Catcode::CS)
    .map(adds_to_frontmatter)
    .collect::<Result<Vec<bool>>>()?;
  let plain = |address: &str| {
    address.split_once('@').is_some_and(|(local, domain)| {
      !local.is_empty()
        && local
          .chars()
          .all(|c| c.is_alphanumeric() || "._%+-".contains(c))
        && domain.contains('.')
        && domain
          .chars()
          .all(|c| c.is_alphanumeric() || ".-".contains(c))
    })
  };
  let one_address =
    email_addresses(&run).is_some_and(|addresses| addresses.len() == 1 && plain(&addresses[0]));
  let name = Tokens::new(tokens[..start].to_vec());
  if commands.contains(&true)
    || !one_address
    || visible_name_text(name.unlist_ref()).trim().is_empty()
  {
    return whole();
  }
  let mut name = name.unlist();
  while name.last() == Some(&T_SPACE!()) {
    name.pop();
  }
  Ok((Tokens::new(name), Some(run)))
}

/// The addresses of an author-block line that is only email addresses (`\texttt{a@x, b@y}`, `{a,b}@dom`), each as
/// tokens ([`address_tokens`]); a line of one address is itself (`\href{mailto:a@x}{a@x}` stays a link). `None` for
/// any other line.
fn bare_email_addresses(line: &Tokens) -> Result<Option<Vec<Tokens>>> {
  let Some(addrs) = email_addresses(line).filter(|addrs| !addrs.is_empty()) else {
    return Ok(None);
  };
  if addrs.len() == 1 {
    return Ok(line_is_email(line).then(|| vec![line.clone()]));
  }
  address_tokens(line, addrs).map(Some)
}

/// The definitions in force where a class's title code reads `\@author`: the statements of the groups still open there
/// in its `\@maketitle` (else a dropped `\maketitle`'s) body — `\def`, `\gdef`, `\let`, `\newcommand`, `\providecommand`
/// — with a global one kept past its group, in source order; each with its definee.
fn title_code_author_definitions() -> Vec<(Token, Vec<Token>)> {
  [T_CS!("\\@maketitle"), T_CS!("\\lx@dropped@maketitle")]
    .iter()
    .find_map(|cs| match lookup_meaning(cs) {
      Some(Stored::Expandable(definition)) => match definition.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => definitions_at_author(body.unlist_ref()),
        _ => None,
      },
      _ => None,
    })
    .unwrap_or_default()
}

/// The definition statements of `body`'s groups open at its first `\@author` ([`title_code_author_definitions`]),
/// in source order; `None` when it reads no `\@author`. Groups are brace groups, `\bgroup`/`\egroup`,
/// `\begingroup`/`\endgroup` and environments. A statement runs locally where it is replayed: a `\gdef` is a `\def`
/// there, and the `\global` before a definition is left out. `\edef`/`\xdef` are not replayed (expanded now, not at
/// the title code), nor `\renewcommand` (of a command already defined, which stands). A statement whose shape is not
/// read whole (a `\def` without its body's `{`, a `\newcommand` without a body) is left out, and so is one that closes
/// a group it did not open (2601.13359's `\def\affiliations{\egroup\par\Large\bgroup\rm}`, `\let\x\egroup`): the
/// title code's layout between its own groups, which the author content is not inside.
fn definitions_at_author(body: &[Token]) -> Option<Vec<(Token, Vec<Token>)>> {
  // the index after the balanced group starting at `i` (a BEGIN), else after the one token there
  let group_end = |i: usize| -> usize {
    if body.get(i).map(|t| t.get_catcode()) != Some(Catcode::BEGIN) {
      return (i + 1).min(body.len());
    }
    let mut depth = 0usize;
    for (j, t) in body.iter().enumerate().skip(i) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return j + 1;
          }
        },
        _ => {},
      }
    }
    body.len()
  };
  let skip_spaces = |mut j: usize| -> usize {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  // each statement with the group it is made in (None: global)
  let mut statements: Vec<(Token, Vec<Token>, Option<usize>)> = Vec::new();
  let mut open: Vec<usize> = vec![0];
  let mut groups = 0usize;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    let name = t.with_str(str::to_string);
    let opens = t.get_catcode() == Catcode::BEGIN
      || matches!(name.as_str(), "\\bgroup" | "\\begingroup" | "\\begin");
    let closes = t.get_catcode() == Catcode::END
      || matches!(name.as_str(), "\\egroup" | "\\endgroup" | "\\end");
    if opens {
      groups += 1;
      open.push(groups);
      i += 1;
      continue;
    }
    if closes {
      if open.len() > 1 {
        open.pop();
      }
      i += 1;
      continue;
    }
    if t == T_CS!("\\@author") {
      return Some(
        statements
          .into_iter()
          .filter(|(_, _, group)| group.is_none_or(|g| open.contains(&g)))
          .map(|(definee, statement, _)| (definee, statement))
          .collect(),
      );
    }
    let global = name == "\\gdef" || (i > 0 && body[i - 1] == T_CS!("\\global"));
    // the definee, where the statement ends, and its first token as replayed
    let read: Option<(Option<Token>, usize, Token)> = match name.as_str() {
      "\\def" | "\\gdef" if t.get_catcode() == Catcode::CS => {
        // the parameter text runs to the body's `{`
        let mut j = i + 2;
        while j < body.len() && body[j].get_catcode() != Catcode::BEGIN {
          j += 1;
        }
        (j < body.len()).then(|| (body.get(i + 1).copied(), group_end(j), T_CS!("\\def")))
      },
      "\\let" if t.get_catcode() == Catcode::CS => {
        let mut j = skip_spaces(i + 2);
        if body.get(j) == Some(&T_OTHER!("=")) {
          j += 1;
        }
        j = skip_spaces(j);
        body
          .get(j)
          .filter(|target| !matches!(target.get_catcode(), Catcode::BEGIN | Catcode::END))
          .map(|_| (body.get(i + 1).copied(), j + 1, t))
      },
      "\\newcommand" | "\\providecommand" if t.get_catcode() == Catcode::CS => {
        let mut j = i + 1;
        if body.get(j) == Some(&T_OTHER!("*")) {
          j += 1;
        }
        j = skip_spaces(j);
        let definee = match body.get(j) {
          Some(b) if b.get_catcode() == Catcode::BEGIN => body.get(j + 1).copied(),
          other => other.copied(),
        };
        j = skip_spaces(group_end(j));
        while body.get(j) == Some(&T_OTHER!("[")) {
          while j < body.len() && body[j] != T_OTHER!("]") {
            j += 1;
          }
          j = skip_spaces(j + 1);
        }
        body
          .get(j)
          .filter(|b| b.get_catcode() != Catcode::END)
          .map(|_| (definee, group_end(j), t))
      },
      _ => None,
    };
    match read {
      Some((Some(definee), stop, head))
        if matches!(definee.get_catcode(), Catcode::CS | Catcode::ACTIVE)
          && stop <= body.len()
          && closes_only_its_own_groups(&body[i + 2..stop]) =>
      {
        let mut statement = vec![head];
        statement.extend_from_slice(&body[i + 1..stop]);
        statements.push((
          definee,
          statement,
          (!global).then(|| *open.last().unwrap_or(&0)),
        ));
        i = stop;
      },
      _ => i += 1,
    }
  }
  None
}

/// Whether `tokens` (a definition's meaning) close only the groups and environments they open (`\bgroup`,
/// `\begingroup`, `\begin`): an author separator `\def\sep{\end{tabular}\begin{tabular}{c}}` does not.
fn closes_only_its_own_groups(tokens: &[Token]) -> bool {
  let mut depth = 0usize;
  for t in tokens {
    if t.get_catcode() != Catcode::CS {
      continue;
    }
    match t.with_str(str::to_string).as_str() {
      "\\bgroup" | "\\begingroup" | "\\begin" => depth += 1,
      "\\egroup" | "\\endgroup" | "\\end" => match depth.checked_sub(1) {
        Some(d) => depth = d,
        None => return false,
      },
      _ => {},
    }
  }
  true
}

/// The marks that request an affiliation label in an author (`\lx@author@withsup`, llncs `\inst`) and set it in an
/// affiliation (`\lx@affiliation@withsup`, `\lx@affiliation@withinst`): the author marks ([`is_author_mark`]), llncs's
/// `\inst` and the symbol mark.
fn is_affiliation_mark(t: &Token) -> bool {
  is_author_mark(t) || *t == T_CS!("\\inst") || *t == T_CS!("\\lx@frontmatter@symbolmark")
}

/// The operand of each mark in `tokens`, in order, read as `read_frontmatter_sup_operand` reads it.
fn mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  let mut operands = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if is_affiliation_mark(&tokens[i])
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

/// The queue position of the first author of the row an address shared by a row of names belongs to
/// ([`shared_affiliation_calls`]): the authors queued after the last affiliation queued — on its own, or in an author's
/// entry (a paired row's `Ann Able\lx@add@affiliation{Univ A}`) —, those an `annotate=run` affiliation goes to; when
/// every queued author has one, the authors of the last author list (`\lx@add@authors@list`: 2010.08573's second
/// `\aistatsaddress`, the e-mails under a row its first address was paired with).
fn queued_row_start() -> usize {
  let affiliation = T_CS!("\\lx@add@affiliation");
  let (authors, start) = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => {
      let (mut authors, mut start) = (0, 0);
      for (tag, attr, content) in queue {
        let role = attr.get("role").map(String::as_str);
        if tag == "ltx:creator" && role == Some("author") {
          authors += 1;
          if content.unlist_ref().contains(&affiliation) {
            start = authors;
          }
        } else if tag == "ltx:contact" && role == Some("affiliation") {
          start = authors;
        }
      }
      (authors, start)
    },
    _ => (0, 0),
  });
  if start < authors {
    return start;
  }
  match lookup_value("lx_authors_list_start") {
    Some(Stored::Int(list_start)) if (list_start as usize) < authors => list_start as usize,
    _ => start,
  }
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

/// Whether the addresses of `lines` may be the queued authors' in order from the one at `offset`
/// ([`addresses_in_order`]).
fn run_in_author_order(lines: &[(Vec<Token>, Tokens)], offset: usize) -> bool {
  let keys = queued_author_name_keys();
  let addresses: Vec<String> = lines
    .iter()
    .flat_map(|(_, line)| email_addresses(line).unwrap_or_default())
    .collect();
  addresses_in_order(&keys, &addresses, offset)
}

/// Whether `addresses` may be the authors' in order from the one at `offset` (names keyed by `keys`, [`NameKeys`]):
/// none spells the name of an author other than the one at its place. 2509.10377's
/// `{12421181,ziyuzhao.cs,wu\_zhiliang,yangyics,…}@zju.edu.cn` lists one address for each author, by institution —
/// "yangyics" at Zhiliang Wu's place spells Yi Yang (2406.06326: "jyzhou" at Baolin Peng's, 63m).
fn addresses_in_order(keys: &[NameKeys], addresses: &[String], offset: usize) -> bool {
  addresses
    .iter()
    .enumerate()
    .all(|(at, address)| spelled_owner(keys, address).is_none_or(|owner| owner == offset + at))
}

/// `tokens` without the marks that print nothing: a mark command's empty argument (`\textsuperscript{}`, [`mark_commands`])
/// and an empty math superscript (`$^{}$`), spaces and `\relax` aside, when the block holds no other mark. An empty mark is no mark
/// (user ruling R11 2026-10-20): it makes no block a marked one (2604.19440's `\textsuperscript{}Univ. Grenoble Alpes,
/// …` under four unmarked names, which pdflatex prints with no mark, an affiliation of all four). In a block other marks
/// make a marked one it stays, as before: it labels nothing there, and each name it ends is an author's piece of its
/// own (2604.22129's `Robert Maier\textsuperscript{}~~~Aljaz Bozic\textsuperscript{}` beside `David
/// Recasens\textsuperscript{\rm 1}`, which a marked block's marker-less pieces would weld into one creator).
fn drop_empty_marks(tokens: Tokens) -> Tokens {
  let original = tokens.clone();
  let v = tokens.unlist();
  // (a `\relax` in it prints nothing either: `Ann Able\textsuperscript{\relax}, Bob Baker`, the r7 re-review's n10)
  let blank = |range: &[Token]| {
    range.iter().all(|t| {
      matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::SPACE
      ) || *t == T_CS!("\\relax")
    })
  };
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    let (operand, after) = if mark_commands().contains(&v[i]) {
      let open = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      (open, 0)
    } else if v[i].get_catcode() == Catcode::MATH && v.get(i + 1) == Some(&T_SUPER!()) {
      (i + 2, 1)
    } else {
      out.push(v[i]);
      i += 1;
      continue;
    };
    let close = skip_group(&v, operand);
    if v
      .get(operand)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && blank(&v[operand..close.min(v.len())])
      && (after == 0
        || v
          .get(close)
          .is_some_and(|t| t.get_catcode() == Catcode::MATH))
    {
      i = close + after;
    } else {
      out.push(v[i]);
      i += 1;
    }
  }
  let out = Tokens::new(out);
  if has_author_marks(&rewrite_symbol_superscripts(out.clone())) {
    original
  } else {
    out
  }
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

/// The tags `\lx@make@tags{type}` collects into an `ltx:tags` (Perl Base_Utility.pool.ltxml:975-983), one per declared
/// role (`type_tag_formatter`), in role order, each with whether it is printed where it is built: the role-less tag is
/// the target's own label (`\item[Espa~na]`, a bibliography entry's `\@biblabel`), the others are the forms only a
/// reference prints (refnum, typerefnum, autoref, the cleveref forms). A caller that typesets those only where a
/// reference prints them (sect11.rs `\lx@bibitem`, the `\b@key` a `\cite` prints) takes them apart here.
pub fn tag_forms(ttype: &Tokens) -> Result<Vec<(bool, Tokens)>> {
  // Pull the (role -> formatter) pairs out of HashStored via with_value
  // so we don't clone the whole hashmap envelope; the per-role tokens
  // are Copy and the per-role String arm just dereferences a SymStr.
  let role_formatters: Vec<(String, Option<Token>)> =
    with_value("type_tag_formatter", |v| match v {
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
              Some(Stored::String(sym)) => Some(Token!(sym * sym, Catcode::CS)),
              _ => None,
            };
            (role, ft)
          })
          .collect()
      },
      _ => Vec::new(),
    });
  let mut forms = Vec::new();
  for (role, formatter_opt) in role_formatters {
    let Some(formatter_token) = formatter_opt else {
      continue;
    };
    // A tag only a reference prints (`type_tag_deferred`) holds its diagnostics (`\lx@tag@intags@held`).
    let constructor = if with_mapping("type_tag_deferred", &role, |held| held.is_some()) {
      "\\lx@tag@intags@held"
    } else {
      "\\lx@tag@intags"
    };
    let invoked_tag = Invocation!(T_CS!(constructor), vec![
      Tokens!(T_OTHER!(role.as_str())),
      build_invocation(formatter_token, vec![Some(ttype.clone())])?
    ]);
    // A tag a reference prints (refnum, typerefnum, autoref, the cleveref forms) is built where its target is (an
    // eqnarray row's in its math group, where a document may rebind `~`: vdm.sty:135-138, 202-205
    // `\everymath{\let~\hook}`, a `\vbox{\ialign…}` that re-stepped the equation, an endless recursion on 1601.02132)
    // and printed in text. Its `~` is the kernel's no-break space, LaTeX's `\nobreakspace`, which babel's `~` also is
    // except in a language that makes it a shorthand; such a shorthand, or a document's own text `~`, no longer reaches
    // these tags (OXIDIZED_DESIGN_DIVERGENCES #98 addendum). The role-less tag is the target's own label, printed where
    // it is built (`\item[Espa~na]`), and keeps the document's `~`. Guard
    // `perfect_kernel_batch63::autoref_tag_tilde_is_the_kernel_space`.
    // A reference form re-reads the label's source, which the role-less tag typesets (`\lx@reread@copy`): an
    // `\item[$\hat\mathbf{e}$]`'s typerefnum, a `\tag{$\hat\mathbf{t}$}`'s refnum give no second
    // OXIDIZED_DESIGN_DIVERGENCES #721 Warning (guard `perfect_kernel_batch71a::undelimited_argument_warning_once_per_occurrence`).
    if role.is_empty() {
      forms.push((true, invoked_tag));
    } else {
      let mut wrapped = vec![
        T_CS!("\\begingroup"),
        T_CS!("\\lx@reread@copy"),
        T_CS!("\\let"),
        T_ACTIVE!('~'),
        T_CS!("\\lx@tag@texttilde"),
      ];
      wrapped.extend(invoked_tag.unlist());
      wrapped.push(T_CS!("\\endgroup"));
      forms.push((false, Tokens::new(wrapped)));
    }
  }
  Ok(forms)
}

/// The authors the frontmatter has digested so far (`num_ltx:creator{author}`): the last one digests with the label
/// `author:N` for this N, by which a contact digested after it can name it or the ones before it (amsppt's per-block
/// addresses, amsppt_sty.rs `\lx@amsppt@paired`).
pub fn digested_author_count() -> i64 { lookup_mapping_int("num_ltx:creator", "author") }

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

/// Whether digested material typesets something a reader sees: a character that is not a space, an image (a whatsit
/// carrying a `graphic` or image `candidates`), or a cross-reference or citation, whose text the post-processor
/// fills in (`fills_generated_text`), anywhere in it — through lists, the whatsit arguments it shows
/// (`shown_arguments`, as [`ink_text`] reads them) and bodies, and alignment cells. Paragraph and spacing commands,
/// rules and empty boxes are not content, nor is an argument LaTeX only tests: the position letter of
/// `\begin{tabular}[t]`, `\parbox[t]`, `{minipage}[t]`, `\makebox[w][l]` or `\shortstack[l]` (a `Position` parameter,
/// base_parameter_types.rs) — counted, the deproc title code's `\begin{tabular}[t]{l}\bf\rule{\z@}{24pt}…` with an
/// emptied `\@author` (iclr*_conference.sty:83-93, arxiv.sty, colm, corl, nips) kept its `\@maketitle` deposit, an empty
/// strut table before the first section (HF14; 94 of 6,000 sampled papers, 2610.20409, 2310.10873).
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
      // (an image: graphics.sty's `\@includegraphics` records its `graphic`, graphicx's `\@includegraphicx` its
      // `path`, both the image's `candidates`; its file name is no shown argument)
      w.get_property("graphic").is_some()
        || w.get_property("candidates").is_some()
        || fills_generated_text(&w)
        // A tabular or `\halign` keeps its rows in its `alignment` property (tex_tables.rs): a class's `\maketitle`
        // that is only a table typesets it (simplecv's header, brandeis-problemset's title table; dropped since 59p).
        || matches!(w.get_property("alignment").as_deref(), Some(Stored::Digested(a)) if typesets_content(a))
        || w
          .get_args()
          .iter()
          .zip(shown_arguments(&w))
          .any(|(arg, shown)| shown && arg.as_ref().is_some_and(typesets_content))
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

/// Whether the whatsit `w` is a cross-reference or a citation, which prints text the post-processor generates (the
/// label's number, the cited work's name and year) though its arguments are keys, never shown ([`shown_arguments`]):
/// a reference carries the `label` its `labelref` names (`\ref`, `\pageref`, `\eqref`, `\autoref`, `\nameref`,
/// cleveref's `\lx@cref`, ntheorem's `\thref`), a citation its `bibrefs` (`\@@bibref`, which every `\cite` form reaches).
/// `\label` records its `label` too, but prints nothing: an alignment may skip it (`alignmentSkippable`). A class's
/// `\@maketitle` printing only `\ref{s1}` keeps its deposit, as pdflatex prints "1" (77o review probe `m1_depref`).
fn fills_generated_text(w: &Whatsit) -> bool {
  w.get_property("bibrefs").is_some()
    || (w.get_property("label").is_some() && w.get_property("alignmentSkippable").is_none())
}

/// The text digested material typesets, as [`typesets_content`] walks it — characters of its boxes, through lists,
/// whatsit arguments and bodies, alignment rows and cells; never a whatsit's reversion (its source), and only the
/// arguments it shows (`shown_arguments`): an image's file name and options are not text on the page
/// (`\includegraphics{\@logo}`), nor is a link's target (`\href{\@url}{Home}` prints "Home"). What a speculative
/// title-page run printed (sect05.rs `speculative_title_page_run`; review probe `logo`).
pub fn ink_text(d: &Digested, out: &mut String) {
  match d.data() {
    DigestedData::TBox(b) => {
      if let Ok(s) = b.borrow().get_string() {
        out.push_str(&s);
      }
    },
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        ink_text(child, out);
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      if let Some(Stored::Digested(a)) = w.get_property("alignment").as_deref() {
        ink_text(a, out);
      }
      for (arg, shown) in w.get_args().iter().zip(shown_arguments(&w)) {
        if shown && let Some(arg) = arg {
          ink_text(arg, out);
        }
      }
      if let Ok(Some(body)) = w.get_body() {
        ink_text(&body, out);
      }
    },
    DigestedData::Alignment(a) => {
      for row in a.borrow().rows() {
        for cell in row.get_columns() {
          if let Some(boxes) = cell.boxes.as_ref() {
            ink_text(boxes, out);
          }
        }
      }
    },
    _ => {},
  }
}

/// The constructors that print a verbatim argument as it is — a URL, an e-mail address, a DOI —, each with which of its
/// arguments it shows (by position), for [`shown_arguments`]: llncs.cls's `\url` (llncs_cls.rs), hyperref's
/// `\nolinkurl`, aas's `\@@email`, the OmniBus `\lx@doi`, natbib's `\harvardurl`. A constructor's replacement is
/// compiled, so what it puts in its element cannot be read off the whatsit; a verbatim argument is otherwise a key
/// (`\ref`, `\pageref`, `\eqref`, `\cite`, `\nameref`, `\autoref`, `\label`), a link target or a file name, which the
/// page does not show. url.sty's and hyperref's `\url`/`\path` and the bindings' `\email`s are macros: what they print
/// reaches a constructor of its own, whose text argument is shown (`\lx@url@url@nolink`'s, `\lx@hyper@url@`'s `#5`), as
/// aas's `\doi{…}`'s is. A constructor printing one of two verbatim arguments names it: `("\\href", &[false, true])` would show the
/// text, not the URL (jhep's two-verbatim `\href`, a binding class's, which never meets the raw-class stores).
const VERBATIM_PRINTERS: &[(&str, &[bool])] = &[
  ("\\url", &[true]),
  ("\\nolinkurl", &[true]),
  ("\\@@email", &[true]),
  ("\\lx@doi", &[true]),
  ("\\harvardurl", &[true]),
];

/// Which arguments of the whatsit `w` it shows as text, for [`ink_text`], by their declared parameter types: none of an
/// image (a whatsit carrying a `graphic` or image `candidates`: its arguments are the file and its options); else a
/// verbatim argument (`Semiverbatim`, `Verbatim`, `HyperVerbatim`: a file name, a URL, a label) only when the whatsit
/// is one that prints it ([`VERBATIM_PRINTERS`]: `\url{…}`, `\nolinkurl{…}`, the arguments its entry names) — never a
/// reference's or a citation's key (`\ref{\@x}` prints the label's number, review probe `ctx3`), and
/// `\href{<url>}{<text>}` and `\hypertarget{<name>}{<text>}` print the text —, and never keyvals (options) nor a
/// position keyword (`Position`, `OptionalPosition`: the `[t]` of `\begin{tabular}[t]`, base_parameter_types.rs). Every
/// argument when its parameters cannot be matched to them.
fn shown_arguments(w: &Whatsit) -> Vec<bool> {
  let args = w.get_args();
  // (graphics.sty's `\@includegraphics` records its `graphic`, graphicx's `\@includegraphicx` its `path`; both the
  // image's `candidates`)
  if w.get_property("graphic").is_some() || w.get_property("candidates").is_some() {
    return vec![false; args.len()];
  }
  let definition = w.get_definition();
  let types: Vec<SymStr> = definition
    .get_parameters()
    .map(|params| {
      params
        .get_parameters()
        .into_iter()
        .filter(|p| !p.novalue)
        .map(|p| p.name)
        .collect()
    })
    .unwrap_or_default();
  if types.len() != args.len() {
    return vec![true; args.len()];
  }
  // (0: keyvals or a position keyword, 1: verbatim, 2: shown text)
  let kinds: Vec<u8> = types
    .iter()
    .map(|t| {
      with(*t, |name| {
        if name.contains("KeyVals") || name.ends_with("Position") {
          0
        } else if name.contains("erbatim") {
          1
        } else {
          2
        }
      })
    })
    .collect();
  let cs = definition.get_cs();
  let shown: &[bool] = VERBATIM_PRINTERS
    .iter()
    .find(|(name, _)| *cs == T_CS!(*name))
    .map_or(&[], |(_, shown)| shown);
  kinds
    .iter()
    .enumerate()
    .map(|(i, kind)| *kind == 2 || (*kind == 1 && shown.get(i).copied().unwrap_or(false)))
    .collect()
}

/// While a raw class's own frontmatter setter replays for its side effects (`replay_class_setter`, OXIDIZED_DESIGN
/// #482), the frontmatter is the kernel's alone: nothing is queued (an entry, an annotation) or removed
/// (`\lx@clear@frontmatter`, which the kernel's `\lx@add@title`, `\lx@add@date`, `\lx@add@abstract` and
/// `\lx@add@keywords` run first), whatever path a body takes to the queue.
fn frontmatter_frozen_for_replay() -> bool { lookup_bool("lx_replaying_class_setter") }

/// frontmatter_raw contains the undigested commands to create frontmatter,
/// along with the tag & attributes that would be created.
/// Digestion is deferred until \maketitle, or something similar,
/// to avoid extra side-effects, particularly when entries
/// (eg. \title, \author) get redefined & replaced.
/// Use dequeue_front_matter or \lx@clear@frontmatter if there should be only 1 entry of $tag
/// Perl: queueFrontMatter($stomach, $tag, $attr, $command).
pub fn queue_front_matter(tag: &str, attr: Option<&KeyVals>, command: Tokens) {
  if frontmatter_frozen_for_replay() {
    return;
  }
  let queue = frontmatter_queue_name();
  note_document_sets_front_matter(
    tag,
    &attr
      .and_then(|kv| kv.get_value("role"))
      .map(|role| role.to_string())
      .unwrap_or_default(),
  );
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
  let missing = with_value(queue, |v| !matches!(v, Some(Stored::FrontmatterRaw(_))));
  if missing {
    assign_value(
      queue,
      Stored::FrontmatterRaw(Vec::new()),
      Some(Scope::Global),
    );
  }
  with_value_mut(queue, |val_opt| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
      queue.push((tag.to_string(), attr_hash, command));
    }
  });
}

/// The queue a frontmatter entry goes to: the document's (`frontmatter_raw`), or, while a stand-in binding's copy of
/// its author file is read (`binding::content::read_stand_in_copy`, OXIDIZED_DESIGN_DIVERGENCES #731), the copy's
/// defaults (`frontmatter_copy_defaults`): in TeX a class's own setter call (atlasdoc.cls:189 `\author{The ATLAS
/// Collaboration}`, :190 `\date{\today}`) only stores the default the document's own call replaces, which its title
/// page prints if the document makes none ([`hand_copy_frontmatter_defaults`]). A removal (`\lx@clear@frontmatter`,
/// which `\date` runs first) applies to the same queue: the copy's later `\date{}` (atlasdoc.cls:824 under
/// `cernpreprint`) clears its `\today`.
fn frontmatter_queue_name() -> &'static str {
  if definition::origin::copy_provides() != 0 {
    "frontmatter_copy_defaults"
  } else {
    "frontmatter_raw"
  }
}

/// A frontmatter entry `tag` of role `role` (`""`: every role) is set — queued or removed — outside a stand-in copy's
/// read ([`frontmatter_queue_name`]): the document's (or a binding's) setter call, whatever its value, replaces the
/// class default the copy stored, as `\def\@date{#1}` does in TeX ([`hand_copy_frontmatter_defaults`]): `\date{}`
/// removes the class's `\date{\today}`, and `\author{}`/`\title{}` theirs, though they queue nothing.
fn note_document_sets_front_matter(tag: &str, role: &str) {
  if definition::origin::copy_provides() != 0 {
    // (the copy stored a default: from now on its kernel field's setter is observed, as a binding's `\author` may
    // queue nothing for an empty call — inst_support's accumulates)
    if !definition::origin::setter_watch() {
      definition::origin::watch_setter_calls(&COPY_DEFAULT_SETTERS.map(|(setter, ..)| setter));
    }
  } else {
    assign_value(
      &s!("frontmatter_set:{tag}:{role}"),
      true,
      Some(Scope::Global),
    );
  }
}

/// The kernel's fields a stand-in copy's setter stores a default of: setter, tag, role, store.
const COPY_DEFAULT_SETTERS: [(&str, &str, &str, &str); 3] = [
  ("\\author", "ltx:creator", "author", "\\@author"),
  ("\\date", "ltx:date", "creation", "\\@date"),
  ("\\title", "ltx:title", "", "\\@title"),
];

/// Did the document set a frontmatter entry `tag` of role `role` — call its setter (`state::setter_called`, the
/// kernel's fields: whatever binding defines `\author`), or queue or remove one ([`note_document_sets_front_matter`];
/// a role-less removal sets every role)?
fn document_set_front_matter(tag: &str, role: &str) -> bool {
  lookup_bool(&s!("frontmatter_set:{tag}:{role}"))
    || (!role.is_empty() && lookup_bool(&s!("frontmatter_set:{tag}:")))
    || COPY_DEFAULT_SETTERS
      .iter()
      .any(|(setter, t, r, _)| *t == tag && *r == role && setter_called(&T_CS!(setter)))
}

/// At `\maketitle` (`\lx@store@defaults`): hand a stand-in copy's frontmatter defaults (`frontmatter_queue_name`) to
/// the document's frontmatter, each only when the document's setter made no call of its kind (tag and role: its own
/// `\author` replaces the class's, an empty `\date{}` too, `document_set_front_matter`), and, for the kernel's fields,
/// only when the copy's own title page prints the store (`frontmatter_stores::copy_title_page_prints`; the binding's
/// title page prints them when the copy defines none). Five ATLAS papers had the class's `<creator>` beside the
/// document's (1805.01649, 1806.07355, 1807.02012, 1902.05892, 1904.03536) and a `<date>` the class does not print
/// (1902.05892, 1904.03536).
pub fn hand_copy_frontmatter_defaults() {
  let defaults: Vec<RawFrontmatter> = match remove_value("frontmatter_copy_defaults") {
    Some(Stored::FrontmatterRaw(defaults)) => defaults,
    _ => return,
  };
  for (tag, attr, command) in defaults {
    let role = attr.get("role").cloned().unwrap_or_default();
    if document_set_front_matter(&tag, &role) || has_front_matter_with_role(&tag, &role) {
      continue;
    }
    let store = COPY_DEFAULT_SETTERS
      .iter()
      .find(|(_, t, r, _)| *t == tag && (*r == role || r.is_empty()))
      .map(|(.., store)| *store);
    if let Some(store) = store
      && crate::frontmatter_stores::copy_title_page_prints(&T_CS!(store)) == Some(false)
    {
      Info!(
        "frontmatter",
        tag,
        s!("The class's default {tag} is not printed by its title page: not shown")
      );
      continue;
    }
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
        queue.push((tag, attr, command));
      }
    });
  }
}

/// Is a frontmatter entry `tag` of role `role` (`""`: any) queued or digested ([`has_front_matter`])?
fn has_front_matter_with_role(tag: &str, role: &str) -> bool {
  let matches =
    |attr: &TagAttrs| role.is_empty() || attr.get("role").map(String::as_str) == Some(role);
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .any(|entry| entry.0 == tag && matches(&entry.1)),
    _ => false,
  });
  queued
    || with_value("frontmatter", |v| match v {
      Some(Stored::HashTagData(frnt)) => frnt
        .get(tag)
        .is_some_and(|list| list.iter().any(|e| matches(&e.attr))),
      _ => false,
    })
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

/// Is `affiliation`'s first mark (its label, `entry_mark`) one a queued author carries — an affiliation number, not a
/// footnote symbol (`is_symbol_mark`: `$^*$Equal contribution` is the legend of the authors' stars)? Only then do the
/// authors showing that mark answer the affiliation: a mark no author carries (a typo, or authors without marks after
/// marked ones: `\author{Ann$^1$}\affil{$^1$X}\author{Bob}\affil{$^2$Y}`) would label an affiliation no one requests,
/// which the frontmatter places on an author of its own without a name. Nor a combined mark
/// (`\author{Ann$^{1}$}\author{Bob$^{2}$}\affil{$^{1,2}$Univ X}`, 77n r3 review probe w1): the affiliation is labelled
/// by its first label only (`mark_label`), so Bob lost it; it is the run's.
pub fn queued_authors_carry_entry_mark(affiliation: &Tokens) -> bool {
  entry_mark(affiliation.unlist_ref()).is_some_and(|mark| {
    !is_symbol_mark(&mark)
      && clean_frontmatter_labels(&Tokens::new(mark.clone()).to_string(), "affiliation").len() == 1
      && !requesting_authors(&queued_author_marks(), &mark).is_empty()
  })
}

/// Does a queued creator carry superscript marks (`\author{A\textsuperscript{1}}`) for affiliations
/// to be matched against?
pub fn queued_creators_have_marks() -> bool {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .any(|entry| entry.0 == "ltx:creator" && author_mark_position(&entry.2).is_some()),
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
  if frontmatter_frozen_for_replay() {
    return;
  }
  note_document_sets_front_matter(
    tag,
    attr
      .iter()
      .find(|(key, _)| *key == "role")
      .map_or("", |(_, role)| role),
  );
  with_value_mut(frontmatter_queue_name(), |val_opt| {
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

/// A `\par` digested in the frontmatter item `item` (its whatsit's `frontmatter_item`), constructed where the schema is
/// known: when the item's element holds only inline content (`ltx:acknowledgements`, `ltx:keywords`, `ltx:date`, …: no
/// `ltx:p`, but `ltx:break`) and is reached from the insertion point without passing an element that holds paragraphs
/// of its own, the paragraph break is a line break of the item, where text mode would merge the paragraphs (user ruling
/// 2026-10-18; apa.cls's two-paragraph `\acknowledgements`, OXIDIZED_DESIGN_DIVERGENCES #766). A break only parts
/// lines: none before the item's first content, none after another (a run of `\par`s), and those left at its end are
/// dropped when it closes ([`drop_trailing_paragraph_breaks`]). Returns whether the `\par` was the item's (it then
/// closes nothing); otherwise it is the ordinary `\par`.
pub(crate) fn frontmatter_paragraph_break(document: &mut Document, item: &str) -> Result<bool> {
  if document::can_contain_qname(item, "ltx:p") || !document::can_contain_qname(item, "ltx:break") {
    return Ok(false);
  }
  let Some(current) = document.get_element() else {
    return Ok(false);
  };
  let item_sym = pin(item);
  let mut open = Vec::new();
  let mut node = Some(current.clone());
  let item_node = loop {
    let Some(n) = node else {
      return Ok(false);
    };
    if document::get_node_qname(&n) == item_sym {
      break n;
    }
    if document::can_contain(&n, "ltx:p") || document::get_node_qname(&n) == pin!("ltx:p") {
      return Ok(false);
    }
    node = n.get_parent();
    open.push(n);
  };
  open.push(item_node.clone());
  if last_line_end(&item_node, &open) == Some(false) && document::can_contain(&current, "ltx:break")
  {
    document.insert_element(
      "ltx:break",
      Vec::new(),
      Some(map!(PARAGRAPH_BREAK_MARK.to_string() => "true".to_string())),
    )?;
  }
  Ok(true)
}

/// Note that the `\par`s digested from here on, at this mode frame, are paragraph breaks of the frontmatter field `tag`
/// ([`frontmatter_paragraph_break`]): a box, an alignment cell or math inside it begins a deeper mode frame, where a
/// `\par` is the ordinary one (in a box or cell it ends nothing, tex.web §1096 `end_graf`), while a group (`{\bfseries …}`)
/// begins none.
fn note_frontmatter_paragraphs(tag: &str) {
  assign_value(
    "lx_frontmatter_par_item",
    tag.to_string(),
    Some(Scope::Local),
  );
  assign_value(
    "lx_frontmatter_par_depth",
    Stored::Int(mode_frame_depth()),
    Some(Scope::Local),
  );
}

/// How many frames bind the mode (`MODE`): each mode switch — a box, a cell, math, a frontmatter item's text — binds it
/// in a frame of its own.
pub(crate) fn mode_frame_depth() -> i64 { with_stacked_values("MODE", |modes| modes.len() as i64) }

/// The frontmatter field whose paragraph break a `\par` digested now is (what [`note_frontmatter_paragraphs`] noted, at
/// the same mode frame), if any.
pub(crate) fn frontmatter_paragraph_field() -> Option<String> {
  let item = lookup_string("lx_frontmatter_par_item");
  (!item.is_empty() && lookup_int("lx_frontmatter_par_depth") == mode_frame_depth()).then_some(item)
}

/// The transient mark on the `ltx:break`s [`frontmatter_paragraph_break`] makes (dropped with the other `_`
/// attributes when the document is finalized).
const PARAGRAPH_BREAK_MARK: &str = "_paragraph_break";

/// Does `node`'s content end with a paragraph break the frontmatter made (`Some(true)`), with other content
/// (`Some(false)`), or is it empty (`None`)? Whitespace counts for nothing, and so does an element still `open` (where
/// the content goes on) that holds nothing yet.
fn last_line_end(node: &Node, open: &[Node]) -> Option<bool> {
  let mut child = node.get_last_child();
  while let Some(n) = child {
    match n.get_type() {
      Some(NodeType::TextNode) if n.get_content().trim().is_empty() => {},
      Some(NodeType::ElementNode) if n.has_attribute(PARAGRAPH_BREAK_MARK) => return Some(true),
      Some(NodeType::ElementNode) => match last_line_end(&n, open) {
        Some(found) => return Some(found),
        None if open.iter().any(|o| o.to_hashable() == n.to_hashable()) => {},
        None => return Some(false),
      },
      _ => return Some(false),
    }
    child = n.get_prev_sibling();
  }
  None
}

/// Drop the paragraph breaks [`frontmatter_paragraph_break`] left at the end of the frontmatter element `node` (in its
/// last group too): a `\par` that ends an item ends no line of it.
fn drop_trailing_paragraph_breaks(document: &mut Document, node: &Node) {
  let mut child = node.get_last_child();
  while let Some(n) = child {
    let previous = n.get_prev_sibling();
    match n.get_type() {
      Some(NodeType::TextNode) if n.get_content().trim().is_empty() => {},
      Some(NodeType::ElementNode) if n.has_attribute(PARAGRAPH_BREAK_MARK) => {
        document.remove_node(n)
      },
      Some(NodeType::ElementNode) => return drop_trailing_paragraph_breaks(document, &n),
      _ => return,
    }
    child = previous;
  }
}

/// Digest the content for a frontmatter item, disabling or masking certain commands.
/// Note that we shouldn't be digesting any frontmatter until within document, so inPreamble unnec.
/// See clean_trailing_break for cleanup of misused \\.
/// Perl: digestFrontmatterItem($stomach, $tag, $item).
fn digest_frontmatter_item(tag: &str, item: Tokens) -> Result<Digested> {
  bgroup();
  // the field being made, for the commands a class's own setter of it defines for its argument
  // (frontmatter_stores.rs `refused_environment_definition`: vmsta.cls's `\author` and its `\fnm`)
  assign_value("lx_frontmatter_item", tag.to_string(), Some(Scope::Local));
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
  // `\fnmsep` sets the comma between two footnote marks (`U.~Hopp\inst{1}\fnmsep\thanks{…}`, aa/llncs); before a
  // mark that becomes a link or a note here it would stand alone in the name ("U. Hopp,": astro-ph0001054,
  // astro-ph0611016), so it sets nothing there; between two marks that stay visible it is still their comma.
  // (once: an item digested within another — a `\thanks` in an author — would save the wrapper as itself, and a
  // `\fnmsep` there not before a mark would call itself without end)
  if !lookup_bool("lx_frontmatter_fnmsep") && lookup_definition(&T_CS!("\\fnmsep"))?.is_some() {
    let_i(&T_CS!("\\lx@saved@fnmsep"), &T_CS!("\\fnmsep"), None);
    let_i(&T_CS!("\\fnmsep"), &T_CS!("\\lx@frontmatter@fnmsep"), None);
    assign_value("lx_frontmatter_fnmsep", Stored::Bool(true), None);
  }
  // `DigestText`, with the item's own paragraph breaks noted in its text mode's frame (`frontmatter_paragraph_break`)
  begin_mode("text")?;
  note_frontmatter_paragraphs(tag);
  let digested = digest(item);
  end_mode("text")?;
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

/// A mark's font switch, no part of it: `$^{\bf{1}}$`, 1109.3726; `\textsuperscript{\normalfont{1},}` inside a bold
/// name, 2604.24532; Perl strips `\rm` only, Base_Utility.pool.ltxml:365 — OXIDIZED_DESIGN #34.
static FONT_SWITCH: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
  Regex::new(
    r"\\(?:rm|bf|it|sf|mathbf|mathrm|mathsf|textbf|textrm|textsf|textit|textmd|textup|textnormal|normalfont|upshape|mdseries|bfseries|rmfamily)([^a-zA-Z]|$)",
  )
  .unwrap()
});

/// A symbol unit of a mark — a control word or a star — and a mark's operand that is two or more of them alone
/// (`\spadesuit\heartsuit`; `\dagger*`, `\dagger *`, the r6 re-review's x1, w10, w11), which [`clean_frontmatter_labels`]
/// reads as a label each when they differ (`**`, a doubled star, stays one).
static SYMBOL_COMMAND: std::sync::LazyLock<Regex> =
  std::sync::LazyLock::new(|| Regex::new(r"\\[a-zA-Z]+|\*").unwrap());
static SYMBOL_COMMAND_RUN: std::sync::LazyLock<Regex> =
  std::sync::LazyLock::new(|| Regex::new(r"^(?:(?:\\[a-zA-Z]+|\*)\s*){2,}$").unwrap());

/// The spacing a mark's operand may carry, which prints no label character and parts the labels it stands between as a
/// comma does: a tie, the control space, the math kerns `\!`, `\,`, `\:`, `\;`, `\>`, and the named spaces
/// (`\up{~~1}` is the label 1 and `\up{\!1,2}` the labels 1 and 2, 2209.14855; `$^{1,2\,*}$` the labels 1, 2 and the
/// symbol, 2403.08629).
static SPACING_IN_LABEL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
  Regex::new(r"~|\\[!,:;> ]|\\(?:quad|qquad|enspace|enskip|thinspace|negthinspace)([^a-zA-Z]|$)")
    .unwrap()
});

/// Perl: cleanFrontmatterLabels($labels, $prefix). Rust also parts the labels at the spacing ([`SPACING_IN_LABEL`]), on
/// both sides of the match: the mark an author requests and the one an affiliation sets (OXIDIZED_DESIGN #52(o)).
fn clean_frontmatter_labels(labels: &str, prefix: &str) -> Vec<String> {
  let labels = FONT_SWITCH.replace_all(labels, "$1");
  let labels = SPACING_IN_LABEL.replace_all(&labels, ",$1");
  let mut cleaned = Vec::new();
  let mut pieces: Vec<&str> = labels.split(',').collect();
  // Perl split drops trailing empty fields
  while pieces.last().is_some_and(|p| p.is_empty()) {
    pieces.pop();
  }
  // (a run of distinct symbol commands is a label each, as `$^{1,2}$` is two: `$^{\spadesuit\heartsuit}$` under
  // `$^\spadesuit$Univ A \\ $^\heartsuit$Univ B`, 1811.10830)
  let pieces: Vec<&str> = pieces
    .into_iter()
    .flat_map(|piece| match SYMBOL_COMMAND_RUN.is_match(piece.trim()) {
      true => {
        let symbols: Vec<&str> = SYMBOL_COMMAND
          .find_iter(piece)
          .map(|m| m.as_str())
          .collect();
        // (the run whole first, then each symbol: a line labelled by the run — `$^{*\dag}$Project lead`, which
        // `\lx@sup@setlabel@affiliation` labels by its first label — is answered by a name carrying the whole run,
        // not every name carrying one of its symbols, 2602.19190; a name's run requests each symbol too,
        // 2606.15609's `$^{*\spadesuit}$`)
        if symbols.iter().skip(1).any(|symbol| *symbol != symbols[0]) {
          std::iter::once(piece).chain(symbols).collect()
        } else {
          vec![piece]
        }
      },
      false => vec![piece],
    })
    .collect();
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

/// Remove the entry at (tag, index) of frontmatter{tag}.
fn frontmatter_remove(tag: &str, index: usize) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get_mut(tag)
      && index < list.len()
    {
      list.remove(index);
    }
  });
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

/// Whether a frontmatter entry's options ask it to `accumulate` (any value but `false`), the key taken out of them so it
/// is no attribute.
fn takes_accumulate(options: &mut TagAttrs) -> bool {
  options
    .remove("accumulate")
    .is_some_and(|value| value.trim() != "false")
}

/// Seal the frontmatter digested so far: called where the document body begins — a sectioning unit
/// (`\@@numbered@section`, `\@@unnumbered@section`, digested after `\@startsection@hook`'s flush) or a
/// `thebibliography`. A replaceable entry (a title, date, abstract, keywords) or the author list digested before then is
/// the paper's; a later one under the same name is a supplement's, which `\lx@add@frontmatter@now` drops (with the
/// annotations its flush attaches to it) and `\lx@add@frontmatter@until` keeps in place as body text (user ruling
/// 2026-10-09: keep the main paper's frontmatter, as page 1 of the PDF; OXIDIZED_DESIGN_DIVERGENCES #154). Only what was
/// digested is sealed: a starred section before the first `\maketitle` seals nothing it did not flush (2412.04315,
/// 2609.05314), and what the fallback flushed there (a preamble `\title`) only provisionally — a later `\maketitle` still
/// replaces it, as the PDF prints only that one; nothing is sealed while a frontmatter entry is still being digested (a
/// `\section{Keywords:}` inside an abstract, 2401.13568).
/// Witnesses 2301.10468, 2401.11672, 2503.07022 (imsart supplement frontmatter), 2405.10566, 2503.16707, 2510.20036.
pub fn seal_digested_frontmatter() {
  // (nor a sectioning command a replayed class setter's body typesets: `replay_class_setter`, sect05.rs)
  if frontmatter_entry_in_progress() || lookup_bool("lx_replaying_class_setter") {
    return;
  }
  let keys = with_mapping_keys("frontmatter_digested", |keys| {
    keys.into_iter().map(to_string).collect::<Vec<_>>()
  });
  for key in keys {
    let level = lookup_mapping_int("frontmatter_digested", &key);
    if lookup_mapping_int("frontmatter_sealed", &key) < level {
      assign_mapping("frontmatter_sealed", &key, Some(Stored::Int(level)));
    }
  }
}

/// The key a frontmatter entry is sealed under ([`seal_digested_frontmatter`]): a replaceable one by its tag, `name` and
/// role (an "Abstract" and a "摘要" are two, as are a `published` and a `copyrightyear` date: acmart's copyright year,
/// added after the body, 2503.04003, 2601.12690), an author creator by its role; any other is never sealed.
fn frontmatter_seal_key(
  tag: &str,
  name: Option<&str>,
  role: &str,
  replaceable: bool,
) -> Option<String> {
  if replaceable {
    Some(frontmatter_seal_key_of(
      tag,
      &s!("{}\u{2}{role}", name.unwrap_or("")),
    ))
  } else if tag == "ltx:creator" && role == "author" {
    Some(frontmatter_seal_key_of(tag, "author"))
  } else {
    None
  }
}

fn frontmatter_seal_key_of(tag: &str, name: &str) -> String { s!("{tag}\u{1}{name}") }

/// Whether a new `tag` entry under `role`, without a `name`, would now be turned away as a supplement's
/// ([`seal_digested_frontmatter`]): a title (replaceable), or an author creator.
pub fn frontmatter_entry_sealed(tag: &str, role: &str) -> bool {
  let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag);
  frontmatter_seal_key(tag, None, role, replaceable).is_some_and(|key| frontmatter_sealed(&key))
}

/// How firmly a frontmatter entry is held: `PROVISIONAL_SEAL` for what the fallback flushed at the first section,
/// which a later `\maketitle` may still replace (the PDF printed it only where the class prints its frontmatter
/// without one); `FINAL_SEAL` for what a `\maketitle` or a frontmatter environment digested.
const PROVISIONAL_SEAL: i64 = 1;
const FINAL_SEAL: i64 = 2;

fn note_digested_frontmatter(key: &str) {
  let level = if lookup_bool("lx_frontmatter_provisional") {
    PROVISIONAL_SEAL
  } else {
    FINAL_SEAL
  };
  if lookup_mapping_int("frontmatter_digested", key) < level {
    assign_mapping("frontmatter_digested", key, Some(Stored::Int(level)));
  }
}

/// Whether a new entry under `key` is a supplement's: sealed finally, or provisionally where no `\maketitle` flushes
/// it (`lx_frontmatter_explicit`, set around `\lx@frontmatterhere`'s flush).
fn frontmatter_sealed(key: &str) -> bool {
  match lookup_mapping_int("frontmatter_sealed", key) {
    FINAL_SEAL => true,
    PROVISIONAL_SEAL => !lookup_bool("lx_frontmatter_explicit"),
    _ => false,
  }
}

/// Whether a frontmatter entry is being digested (its content still a lone `PlaceKeeper`).
fn frontmatter_entry_in_progress() -> bool {
  with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.values().any(|list| {
      list
        .iter()
        .any(|e| matches!(e.content.as_slice(), [TagContent::PlaceKeeper]))
    }),
    _ => false,
  })
}

/// Drop the `frontmatter{tag}` entries that carry the same `name` as a new one (Perl:
/// `$$frontmatter{$tag} = []`), so a later `REPLACEABLE_FRONTMATTER_TAGS` entry
/// replaces a re-emission of itself (arXiv 2002.09766's second `\icmltitle`). An
/// entry under another name is another element and stays: a bilingual document's
/// "Abstract" and "摘要" abstracts (beamertheme-mirage-doc lost its English one). So is one in another language
/// (`xml:lang`): smfart's `{altabstract}`, named "Abstract" as the kernel names the document's abstract written
/// before `\begin{document}` sets the class's captions (smfart_cls.rs; 2506.20120, math0201262).
/// OXIDIZED_DESIGN #154.
/// Drop the `tag` entries a new one of the same `name` and `lang` replaces. A raw class's store's entry (`_store`, the
/// store: frontmatter_stores.rs `labelled_api`) is replaced only by another of the same store: the class's title page
/// prints it beside the document's own abstract (`\begin{quote}\itshape\@resumen\end{quote}` and the document's
/// `{abstract}`, review probe `d_nolab`; silent-loss #6a) — one it does not print is released
/// ([`frontmatter_release_store`]).
fn frontmatter_clear_same_name(
  tag: &str,
  name: Option<&str>,
  lang: Option<&str>,
  store: Option<&str>,
) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get_mut(tag)
    {
      list.retain(|e| {
        e.attr.get("name").map(String::as_str) != name
          || e.attr.get("xml:lang").map(String::as_str) != lang
          || e.attr.get("_store").map(String::as_str) != store
      });
    }
  });
}

/// The abstract and keywords entries of a raw class's `store` (`_store`, frontmatter_stores.rs `labelled_api`) whose
/// title page does not print it (`\hypersetup{pdfsubject={\@abst}}`, review probe `md`) become ordinary entries, which
/// the document's own of the same name and language replaces as it replaces any (`frontmatter_clear_same_name`): one
/// already there drops the store's.
pub(crate) fn frontmatter_release_store(store: &str) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
      for tag in ["ltx:abstract", "ltx:keywords"] {
        let Some(list) = frnt.get_mut(tag) else {
          continue;
        };
        let released: Vec<usize> = (0..list.len())
          .filter(|&i| list[i].attr.get("_store").map(String::as_str) == Some(store))
          .collect();
        let mut dropped: Vec<usize> = Vec::new();
        for &i in &released {
          list[i].attr.remove("_store");
          let (name, lang) = (list[i].attr.get("name"), list[i].attr.get("xml:lang"));
          let replaced = list.iter().enumerate().any(|(j, e)| {
            j != i
              && !released.contains(&j)
              && !e.attr.contains_key("_store")
              && e.attr.get("name") == name
              && e.attr.get("xml:lang") == lang
          });
          if replaced {
            dropped.push(i);
          }
        }
        for i in dropped.into_iter().rev() {
          list.remove(i);
        }
      }
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
  // The entries a seal drops in this flush, whose annotations go with them ([`seal_digested_frontmatter`]) — this
  // flush's own, the outer flush's restored after it (a queued entry may flush again, `\maketitle` in an abstract).
  let outer_sealed_off = lookup_string("lx_frontmatter_sealed_off");
  assign_value(
    "lx_frontmatter_sealed_off",
    Stored::String(pin("")),
    Some(Scope::Global),
  );
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
  // The labels the mark-picked contacts digested here answer, `role` and label (a marked line's first mark,
  // `\lx@affiliation@withsup`): an author requesting one has its contact of that role even before the contact is
  // digested, which a run's `new` reads (`\lx@annotate@frontmatter@now`; `\author{Ann$^{1}$}\author{Bob$^{2}$}
  // \affil{$^1$Univ X}\author{Cat \affil{Univ Z}}\affil{$^2$Univ Y}`: Cat's `\affil` is digested with her, before
  // Univ Y, 77n r4 review probe e4). Not a `label=` (`\affil[1]`): authblk prints the unlabelled `\affil` after it
  // under its `\AB@thenote`, both authors' (`\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}`, 2605.25017's
  // e-mail line, 77n r5 review probes q10, r07), as Perl's `annotate=new` (authblk.sty.ltxml:57) gives it.
  let offered: Vec<String> = commands
    .iter()
    .filter(|entry| entry.0 == "ltx:contact")
    .flat_map(|entry| {
      let role = entry.1.get("role").cloned().unwrap_or_default();
      let tokens = entry.2.unlist_ref();
      tokens
        .iter()
        .position(|t| *t == T_CS!("\\lx@affiliation@withsup"))
        .and_then(|at| entry_mark(&tokens[at + 1..]))
        .and_then(|mark| mark_label(&mark))
        .map(move |label| s!("{role}\u{2}{label}"))
    })
    .collect();
  assign_value(
    "lx_frontmatter_offered_labels",
    Stored::String(pin(offered.join("\u{1}"))),
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
  assign_value(
    "lx_frontmatter_sealed_off",
    Stored::String(pin(&outer_sealed_off)),
    Some(Scope::Global),
  );
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
  // A demoted cover no longer stands in for the document's `<ltx:title>`, which the HTML then displays: the class's
  // own printing `\title` run on it would be a second copy (OXIDIZED_DESIGN #629).
  drop_cover_title_ink(document, &titlepage);
  normalize_titlepage_layout_children(document, &titlepage)?;
  let mut para = document.rename_node(titlepage, "ltx:para", true)?;
  document.add_class(&mut para, "ltx_titlepage")?;
  Ok(())
}

/// The marks around the class's own printing `\title` run on a cover (sect05.rs `\lx@cover@title@begin`/`@end`,
/// `titlepage_class_run`): empty internal elements, admitted anywhere by the model (`_Capture_`-suffixed), never in the
/// output (removed here; a stray one the core finalize drops).
pub const COVER_TITLE_BEGIN: &str = "ltx:_CoverTitleBegin_Capture_";
/// See [`COVER_TITLE_BEGIN`].
pub const COVER_TITLE_END: &str = "ltx:_CoverTitleEnd_Capture_";

/// The class's own printing `\title` run on a cover inside `scope` (sect05.rs `titlepage_class_run`): everything that
/// lies between each pair of cover-title marks in document order — elements and bare text (`\def\title#1{#1}` in the
/// cover's open paragraph) — removed with the marks, and the wrappers it leaves empty (a `center`'s paragraph). Called
/// where a `{titlepage}` stops standing in for the document's `<ltx:title>`, so the HTML displays that title
/// (LaTeXML-structure-xhtml.xsl:374-375 hides it only beside an `ltx:titlepage`): `\unwind@titlepage` (a `\maketitle`
/// inside the cover), [`demote_stranded_titlepage`], and [`settle_cover_title_ink`]. An unmatched mark is removed
/// alone. OXIDIZED_DESIGN #629.
pub fn drop_cover_title_ink(document: &mut Document, scope: &Node) {
  let begin_q = pin(COVER_TITLE_BEGIN);
  let marks = cover_title_marks(document, scope);
  if marks.is_empty() {
    return;
  }
  let mut doomed: Vec<Node> = Vec::new();
  let mut chains: Vec<Vec<Node>> = Vec::new();
  let mut open: Option<Node> = None;
  for mark in marks {
    chains.push(ancestors_below(&mark, scope));
    if document::get_node_qname(&mark) == begin_q {
      if let Some(stray) = open.replace(mark) {
        doomed.push(stray);
      }
    } else if let Some(begin) = open.take() {
      doomed.extend(marked_range(scope, &begin, &mark));
      doomed.push(begin);
      doomed.push(mark);
    } else {
      doomed.push(mark);
    }
  }
  doomed.extend(open);
  remove_marked_ink(document, doomed, chains, |_| true);
}

/// Remove the `doomed` nodes (marks and the ink between them), then the marks' ancestors (`chains`) the removal leaves
/// with no content (a `center`'s paragraph holding only the print), innermost first, until none more empties — each a
/// wrapper `prunable` admits. Everything is decided before anything is removed: `remove_node` frees the subtree it
/// takes out (`discard_subtree`), so no handle into it may be used afterwards.
fn remove_marked_ink(
  document: &mut Document,
  doomed: Vec<Node>,
  chains: Vec<Vec<Node>>,
  prunable: impl Fn(&Node) -> bool,
) {
  // the nodes that go, by identity (`to_hashable`: a `Node` is no hash key, its handle is mutable)
  let mut gone: HashSet<usize> = doomed.iter().map(Node::to_hashable).collect();
  // is `node` or one of its ancestors going?
  let goes_with = |node: &Node, gone: &HashSet<usize>| {
    let mut n = Some(node.clone());
    while let Some(cur) = n {
      if gone.contains(&cur.to_hashable()) {
        return true;
      }
      n = cur.get_parent();
    }
    false
  };
  // an ancestor of a mark inside another range goes with that range
  let mut wrappers: Vec<Node> = Vec::new();
  for node in chains.into_iter().flatten() {
    if !goes_with(&node, &gone) && prunable(&node) && !wrappers.contains(&node) {
      wrappers.push(node);
    }
  }
  loop {
    let emptied: Vec<Node> = wrappers
      .iter()
      .filter(|w| !gone.contains(&w.to_hashable()))
      .filter(|w| {
        w.get_child_nodes().iter().all(|c| {
          gone.contains(&c.to_hashable())
            || (c.get_type() != Some(NodeType::ElementNode) && c.get_content().trim().is_empty())
        })
      })
      .cloned()
      .collect();
    if emptied.is_empty() {
      break;
    }
    gone.extend(emptied.iter().map(Node::to_hashable));
  }
  // each removed whole, once: the outermost of the nodes that go (no ancestor of it goes), in a stable order
  let mut victims: Vec<Node> = Vec::new();
  for n in doomed.iter().chain(wrappers.iter()) {
    if gone.contains(&n.to_hashable())
      && !n.get_parent().is_some_and(|p| goes_with(&p, &gone))
      && !victims.contains(n)
    {
      victims.push(n.clone());
    }
  }
  for victim in victims {
    document.remove_node(victim);
  }
}

/// The cover-title marks inside `scope`, in document order.
fn cover_title_marks(document: &mut Document, scope: &Node) -> Vec<Node> {
  document.findnodes(
    "descendant::ltx:_CoverTitleBegin_Capture_ | descendant::ltx:_CoverTitleEnd_Capture_",
    Some(scope),
  )
}

/// `node`'s ancestors strictly below `scope`, innermost first.
fn ancestors_below(node: &Node, scope: &Node) -> Vec<Node> {
  let mut out = Vec::new();
  let mut p = node.get_parent();
  while let Some(n) = p {
    if &n == scope {
      break;
    }
    p = n.get_parent();
    out.push(n);
  }
  out
}

/// The nodes after `begin` and before `end` in document order, each taken whole: the following siblings of `begin`
/// and of each of its ancestors up to the one on the path to `end`, then the children before that path down to `end`.
fn marked_range(scope: &Node, begin: &Node, end: &Node) -> Vec<Node> {
  let mut end_path = ancestors_below(end, scope);
  end_path.push(end.clone());
  let mut range: Vec<Node> = Vec::new();
  let mut cur = begin.clone();
  let mut fork: Option<Node> = None;
  'up: loop {
    let mut sib = cur.get_next_sibling();
    while let Some(s) = sib {
      if end_path.contains(&s) {
        fork = Some(s);
        break 'up;
      }
      sib = s.get_next_sibling();
      range.push(s);
    }
    match cur.get_parent() {
      Some(p) if &p != scope => cur = p,
      _ => break,
    }
  }
  while let Some(node) = fork.take() {
    if &node == end {
      break;
    }
    for child in node.get_child_nodes() {
      if end_path.contains(&child) {
        fork = Some(child);
        break;
      }
      range.push(child);
    }
  }
  range
}

/// At `\end{document}`, the cover-title marks still in the tree (OXIDIZED_DESIGN #629), by the XSLT's own test
/// (LaTeXML-structure-xhtml.xsl:374-375): while the document has a top-level `ltx:titlepage`, its `<ltx:title>` is not
/// displayed and the class's print is the title the page shows — kept, its marks removed, wherever the cover's
/// typesetting put it (a `\vbox` print the absorber set beside an emptied cover: hep-th0010274); without one (a
/// titlepage nested in a section or a box, every cover unwound or demoted) the title is displayed and the print goes
/// ([`drop_cover_title_ink`]).
pub fn settle_cover_title_ink(document: &mut Document) {
  let Some(root) = document.get_document().get_root_element() else {
    return;
  };
  if document
    .findnode("/ltx:document/ltx:titlepage", None)
    .is_some()
  {
    for mark in cover_title_marks(document, &root) {
      document.remove_node(mark);
    }
  } else {
    drop_cover_title_ink(document, &root);
  }
}

/// The field runs' marks (core `Document::absorb`, [`document::FIELD_MARK_TARGET`]): processing instructions
/// `<?ltx-field-mark begin <field> <id>?>` … `<?ltx-field-mark end <field> <id> <min> <max>?>` around a run of ink
/// digested from a frontmatter field's stored tokens (their out-of-band provenance, `\lx@field@mark`). Not elements: a
/// processing instruction is no node of content (`content_nodes`, text content), so no block is built around it.
/// Never in the output: removed by [`lift_marked_field_ink`] and [`settle_field_marks`], and by the core finalize.
pub const FIELD_MARK_TARGET: &str = document::FIELD_MARK_TARGET;
/// The fields the kernel setters stamp, by their provenance id (`Token::prov`'s field).
const FIELD_MARK_FIELDS: &[(&str, u16)] = &[("title", 1), ("author", 2)];

/// The provenance id of a stamped field (`title`, `author`).
fn field_id(field: &str) -> Option<u16> {
  FIELD_MARK_FIELDS
    .iter()
    .find(|(name, _)| *name == field)
    .map(|(_, id)| *id)
}

/// One run of a field's ink, from its two marks ([`FIELD_MARK_TARGET`]).
struct FieldInkRun {
  field:       u16,
  begin:       Node,
  end:         Node,
  /// the field-ledger serials of the boxes it absorbed, inclusive intervals (core `token::field_ledger_cover`), and
  /// of those an alignment took between its parts (a joined run's cells' `\\`)
  serials:     Vec<(u32, u32)>,
  /// the positions of the current call's value (`lx_field_range_<id>`) the run consumed: sorted disjoint
  /// `start..end` intervals
  cover:       Vec<(u16, u16)>,
  /// does the run consume positions of the current call's value only? A print of an earlier call's value (`\title{A}`
  /// printed, then `\title{B}`; an `\edef` copy) is joined with no other run (re-review finding 2, probes r12, r13).
  in_range:    bool,
  /// is the run preceded, in the document's content order, by body text ([`field_runs_after_body_text`])? Then it
  /// is not in the document's title block.
  body_before: bool,
  /// does the run hold a box of one character of the value twice? A print digests each character once, so it read
  /// the value again from an earlier position the way a macro's expansion does (all its boxes at the macro's
  /// position): `\StrLeft{\@title}{1}\@title`'s two `T`s. No whole print.
  reread:      bool,
}

impl FieldInkRun {
  fn min_serial(&self) -> u32 {
    self
      .serials
      .iter()
      .map(|&(first, _)| first)
      .min()
      .unwrap_or(0)
  }
  fn max_serial(&self) -> u32 {
    self
      .serials
      .iter()
      .map(|&(_, last)| last)
      .max()
      .unwrap_or(0)
  }
}

/// The `start:end` intervals of a field's extent (`lx_field_extent_<id>`), a call's range (`lx_field_range_<id>`) or a
/// run's serials (its end mark).
fn parse_intervals<T: FromStr>(text: &str) -> Vec<(T, T)> {
  text
    .split(',')
    .filter_map(|iv| {
      let (start, end) = iv.split_once(':')?;
      Some((start.trim().parse().ok()?, end.trim().parse().ok()?))
    })
    .collect()
}

/// The positions of field `field`'s stored value a whole print covers (`\lx@field@mark`'s extent).
fn field_extent(field: u16) -> Vec<(u16, u16)> {
  parse_intervals(&lookup_string(&s!("lx_field_extent_{field}")))
}

/// The positions of the current call's value of field `field` (`\lx@field@mark`'s range): `start..end`.
fn field_range(field: u16) -> Option<(u16, u16)> {
  parse_intervals(&lookup_string(&s!("lx_field_range_{field}")))
    .first()
    .copied()
}

/// `cover` within `range` (sorted disjoint intervals stay so).
fn clip_cover(cover: &[(u16, u16)], range: Option<(u16, u16)>) -> Vec<(u16, u16)> {
  let Some((start, end)) = range else {
    return Vec::new();
  };
  cover
    .iter()
    .filter_map(|&(s, e)| {
      let (s, e) = (s.max(start), e.min(end));
      (s < e).then_some((s, e))
    })
    .collect()
}

/// Does `cover` cover every position of `extent`, so the run is a whole print of the field (review finding 2: every
/// position, a macro's too, not the first and the last character)? Both are sorted disjoint intervals, the cover's
/// merged where they touch, so each extent interval lies in one cover interval.
fn covers_extent(cover: &[(u16, u16)], extent: &[(u16, u16)]) -> bool {
  !extent.is_empty()
    && extent
      .iter()
      .all(|&(s, e)| cover.iter().any(|&(cs, ce)| cs <= s && e <= ce))
}

/// Is `node` inside invisible ink (`\phantom`, `\vphantom`, `\hphantom` in text: `<ltx:text class="ltx_phantom">`)?
fn in_phantom(node: &Node) -> bool {
  let mut p = node.get_parent();
  while let Some(n) = p {
    if n
      .get_attribute("class")
      .is_some_and(|c| c.split_whitespace().any(|c| c == "ltx_phantom"))
    {
      return true;
    }
    p = n.get_parent();
  }
  false
}

/// Do the boxes with serials `serials` (inclusive intervals) hold a box of one character of field `field`'s value
/// (`lx_field_chars_<id>`) twice?
fn rereads_a_char(field: u16, serials: &[(u32, u32)]) -> bool {
  let chars: Vec<(u16, u16)> = parse_intervals(&lookup_string(&s!("lx_field_chars_{field}")));
  let mut seen: HashSet<u16> = HashSet::default();
  for &(first, last) in serials {
    for serial in first..=last {
      if let Some(entry) = field_ledger_entry(serial)
        && entry.cover.field == field
        && entry.cover.end == entry.cover.start + 1
        && chars
          .iter()
          .any(|&(s, e)| s <= entry.cover.start && entry.cover.start < e)
        && !seen.insert(entry.cover.start)
      {
        return true;
      }
    }
  }
  false
}

/// The serials of the field-ledger entries of field `field` an alignment took with serials `from..to` (exclusive), as
/// inclusive intervals (core `token::field_ledger_take`).
fn taken_serials(field: u16, from: u32, to: u32) -> Vec<(u32, u32)> {
  (from..to)
    .filter(|&serial| field_ledger_entry(serial).is_some_and(|e| e.taken && e.cover.field == field))
    .map(|serial| (serial, serial))
    .collect()
}

/// The field runs inside `scope`, each with its own two marks (paired by the run's id, so never with another print's),
/// in document order of their begin marks; and every mark node. A run's cover is what the boxes it absorbed consumed
/// of the current call's value (and what an alignment took between and right after them): never what a box absorbed
/// elsewhere (in another run, in a formula) or never absorbed (an unused `\setbox`/`\sbox`/`{lrbox}`) consumed
/// (re-review finding 1, probes r19, r20). A run set invisibly (`\phantom`) is no print.
fn field_ink_runs(document: &mut Document, scope: &Node) -> (Vec<FieldInkRun>, Vec<Node>) {
  let marks = field_marks(document, scope);
  if marks.is_empty() {
    return (Vec::new(), marks);
  }
  let body_before = field_runs_after_body_text(scope);
  let mut begins: Vec<(u32, u16, usize)> = Vec::new();
  let mut runs: Vec<(usize, FieldInkRun)> = Vec::new();
  for (at, mark) in marks.iter().enumerate() {
    let content = mark.get_content();
    let parts: Vec<&str> = content.split_whitespace().collect();
    match parts.as_slice() {
      ["begin", field, id] => {
        if let (Ok(field), Ok(id)) = (field.parse(), id.parse()) {
          begins.push((id, field, at));
        }
      },
      ["end", field, id, serials] => {
        let (Ok(field), Ok(id)) = (field.parse::<u16>(), id.parse::<u32>()) else {
          continue;
        };
        let serials: Vec<(u32, u32)> = parse_intervals(serials);
        if let Some(i) = begins
          .iter()
          .position(|(bid, bfield, _)| *bid == id && *bfield == field)
        {
          let (_, _, begin_at) = begins.swap_remove(i);
          if in_phantom(&marks[begin_at]) {
            continue;
          }
          let range = field_range(field);
          let cover = field_ledger_cover(field, &serials);
          let in_range =
            range.is_some_and(|(start, end)| cover.iter().all(|&(s, e)| start <= s && e <= end));
          let reread = rereads_a_char(field, &serials);
          runs.push((begin_at, FieldInkRun {
            field,
            begin: marks[begin_at].clone(),
            end: mark.clone(),
            reread,
            serials,
            cover: clip_cover(&cover, range),
            in_range,
            body_before: body_before.contains(&id),
          }));
        }
      },
      _ => {},
    }
  }
  runs.sort_by_key(|(at, _)| *at);
  // One print split by the structure it is set in (a cover `tabular`'s cells, `\and`'s `\end{tabular}…
  // \begin{tabular}`): a run of a field that goes on with the field's later positions, with no ink between, is one run
  // with the one before when, together with what the alignment took between them (the cells' `\\`, its row-end
  // marker never absorbed, taken once the alignment is shown, core `document::field_ink_taken_serials`), they cover one contiguous stretch of the field's extent.
  // Both must be prints of the current call's value. A second print starts over, and two slices of the value with a
  // gap between (`\StrLeft…\par\StrRight…`, nothing of the gap consumed) stay two runs, neither whole (review finding
  // 2).
  let mut merged: Vec<FieldInkRun> = Vec::new();
  for (_, run) in runs {
    if let Some(last) = merged.last_mut()
      && last.field == run.field
      && last.in_range
      && run.in_range
      && !last.reread
      && !run.reread
      && run.min_serial() > last.max_serial()
      && let (Some(&(_, last_end)), Some(&(run_start, _))) = (last.cover.last(), run.cover.first())
      && run_start >= last_end
      && marked_range(scope, &last.end, &run.begin)
        .iter()
        .all(is_inkless)
    {
      let mut serials = last.serials.clone();
      serials.extend(taken_serials(
        run.field,
        last.max_serial() + 1,
        run.min_serial(),
      ));
      serials.extend(run.serials.iter().copied());
      let joined = clip_cover(
        &field_ledger_cover(run.field, &serials),
        field_range(run.field),
      );
      let extent = field_extent(run.field);
      // every extent position from where the first stopped to where the second starts is consumed between them
      let contiguous = extent
        .iter()
        .flat_map(|&(s, e)| s.max(last_end)..e.min(run_start))
        .all(|p| joined.iter().any(|&(cs, ce)| cs <= p && p < ce));
      if contiguous {
        last.end = run.end;
        last.serials = serials;
        last.cover = joined;
        continue;
      }
    }
    merged.push(run);
  }
  // What the alignment took right after a run's last box (a value's closing `\\`, its row-end marker absorbed in no
  // run, 1605.00182) is the run's too: the ledger's next entries of the field an alignment took, while they continue
  // its cover (no position of the extent skipped), past entries that cover no position of the extent (the space the
  // cell dropped before it). Nothing else: an entry absorbed elsewhere or never absorbed is no part of this print.
  for run in &mut merged {
    let Some(&(_, mut end)) = run.cover.last() else {
      continue;
    };
    let extent = field_extent(run.field);
    let limit = field_range(run.field).map_or(end, |(_, e)| e);
    let mut serial = run.max_serial() + 1;
    while let Some(next) = field_ledger_entry(serial)
      && next.cover.field == run.field
    {
      let continues =
        next.cover.start <= end || !extent.iter().any(|&(s, e)| s < next.cover.start && e > end);
      if next.taken && continues {
        end = end.max(next.cover.end).min(limit);
      } else if next.taken
        || extent
          .iter()
          .any(|&(s, e)| s < next.cover.end && next.cover.start < e)
      {
        break;
      }
      // (an entry of no extent position the alignment dropped, the space before a cell's end, is stepped over: it
      // adds nothing to the cover)
      serial += 1;
    }
    if let Some(last) = run.cover.last_mut() {
      last.1 = last.1.max(end);
    }
  }
  (merged, marks)
}

/// The ids of the field runs inside `scope` that body text precedes in the document's content order (re-review finding
/// 3: a letter that ends `\noindent\@author` signs it; pdflatex prints the name there and only there). Body text is
/// any text of the document before the run's begin mark, the text before it in the run's own block too, except:
/// - a field's ink (between a run's two marks: a title before its author);
/// - what is set apart from the page's flow: a frontmatter element ([`FRONTMATTER_ELEMENTS`]), a note, a float, an
///   error's markup, invisible ink (`\phantom`), the internal `_Capture_` marks;
/// - the text of a TeX box that does not hold the run: a cover's header box (a preprint-number `\parbox`, 0706.0979,
///   1605.00182; `\rightline{LA-UR-…}`, `\makebox[\textwidth][r]{…}`, `\mbox{preprint}`, `\colorbox`,
///   `\fcolorbox`, `\scalebox`) is no body. A box is decided by the TeX mechanism, not by the element it builds
///   (review r4 finding 3), and marks only the element its own constructor opened, at the point it opens it (review r5
///   finding 1; core `Document::mark_field_box`'s `_fieldbox`): a constructor declared `restricted_horizontal`
///   outside math (`\hbox`, so `\rightline`/`\leftline`/`\centerline` and `\colorbox`; `\mbox`, `\makebox`,
///   `\raisebox`, `\fcolorbox`, `\scalebox`/`\resizebox`/`\rotatebox`/`\reflectbox`), `\framebox`/`\fbox`, the
///   capture of a `\vbox`, `\vtop`, `\parbox`, minipage, `p` column or framed box ([`insert_block`]), an
///   environment whose begin code opens a box group, an alignment's container. Material a `\vbox` absorbs straight
///   into its context marks nothing.
///
/// A list environment is no box: text in `{flushright}`, `{center}`, `quote` before the print is body text, and the
/// print stays (the header cases OXIDIZED_DESIGN #715 lists).
fn field_runs_after_body_text(scope: &Node) -> HashSet<u32> {
  /// Is `node` set apart from the page's flow, its text no body of the document?
  fn set_apart(node: &Node) -> bool {
    node.get_name().ends_with("_Capture_")
      || node
        .get_attribute("class")
        .is_some_and(|c| c.split_whitespace().any(|c| c == "ltx_phantom"))
      || with(document::get_node_qname(node), |q| {
        FRONTMATTER_ELEMENTS.contains(&q)
          || matches!(
            q,
            "ltx:note" | "ltx:figure" | "ltx:table" | "ltx:float" | "ltx:ERROR" | "ltx:indexmark"
          )
      })
  }
  /// Was `node` opened by a TeX box's constructor (core `Document::mark_field_box`): its text is no body for a run
  /// outside it?
  fn is_box(node: &Node) -> bool { node.has_attribute(document::FIELD_BOX_ATTRIBUTE) }
  #[derive(Default)]
  struct Walk {
    /// the ids of the runs open at this point of the walk (their marks seen, the end not yet)
    open:  Vec<u32>,
    /// body text seen outside every box
    body:  bool,
    /// the boxes around this point, innermost last, each with whether body text was seen in it
    boxes: Vec<bool>,
    /// the runs body text precedes
    after: HashSet<u32>,
  }
  fn walk(node: &Node, w: &mut Walk) {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::ElementNode) => {
          if set_apart(&child) {
            continue;
          }
          let boxed = is_box(&child);
          if boxed {
            w.boxes.push(false);
          }
          walk(&child, w);
          if boxed {
            w.boxes.pop();
          }
        },
        Some(NodeType::TextNode) => {
          if w.open.is_empty() && !child.get_content().trim().is_empty() {
            match w.boxes.last_mut() {
              Some(seen) => *seen = true,
              None => w.body = true,
            }
          }
        },
        Some(NodeType::PiNode) if child.get_name() == FIELD_MARK_TARGET => {
          let content = child.get_content();
          match content.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["begin", _, id, ..] => {
              if let Ok(id) = id.parse::<u32>() {
                if w.body || w.boxes.iter().any(|seen| *seen) {
                  w.after.insert(id);
                }
                w.open.push(id);
              }
            },
            ["end", _, id, ..] => {
              if let Ok(id) = id.parse::<u32>() {
                w.open.retain(|open| *open != id);
              }
            },
            _ => {},
          }
        },
        _ => {},
      }
    }
  }
  let mut w = Walk::default();
  walk(scope, &mut w);
  w.after
}

/// Does `node` hold no ink: whitespace, a mark, a line break, or elements holding only those?
fn is_inkless(node: &Node) -> bool {
  match node.get_type() {
    Some(NodeType::ElementNode) => {
      document::get_node_qname(node) == pin_static("ltx:break")
        || (node.get_first_child().is_some() && node.get_child_nodes().iter().all(is_inkless))
    },
    Some(NodeType::TextNode) => node.get_content().trim().is_empty(),
    _ => true,
  }
}

/// The field marks inside `scope`, in document order.
fn field_marks(document: &mut Document, scope: &Node) -> Vec<Node> {
  document.findnodes(
    &s!("descendant::processing-instruction('{FIELD_MARK_TARGET}')"),
    Some(scope),
  )
}

/// The elements a printed field's ink may sit in to be the document's title block: paragraphs and their wrappers
/// (`\begin{center}`, a cover minipage, a font group, `\emph`), and a cover's `tabular` (`\begin{tabular}[t]{c}\@author
/// \end{tabular}`). Not a float, a note, a list, a theorem, a sectional unit, a `{titlepage}` or a frontmatter element:
/// what is set there is the author's content (or the frontmatter itself) even when it prints the field.
fn is_field_ink_wrapper(q: SymStr) -> bool {
  with(q, |q| {
    matches!(
      q,
      "ltx:para"
        | "ltx:p"
        | "ltx:logical-block"
        | "ltx:inline-logical-block"
        | "ltx:block"
        | "ltx:inline-block"
        | "ltx:inline-para"
        | "ltx:text"
        | "ltx:emph"
        | "ltx:tabular"
        | "ltx:thead"
        | "ltx:tbody"
        | "ltx:tfoot"
        | "ltx:tr"
        | "ltx:td"
    )
  })
}

/// Does `node` hold content besides the nodes in `gone`, empty internal marks (`_Capture_`) and line breaks? A
/// processing instruction (a field mark) is none.
fn holds_content_besides(node: &Node, gone: &[Node]) -> bool {
  node.get_child_nodes().iter().any(|c| {
    if gone.contains(c) {
      return false;
    }
    match c.get_type() {
      Some(NodeType::ElementNode) => {
        if c.get_name().ends_with("_Capture_") && c.get_first_element_child().is_none() {
          return false;
        }
        // a line's break (`\centerline`'s, a `\\` after the print) ends the line, it holds nothing
        if document::get_node_qname(c) == pin_static("ltx:break") {
          return false;
        }
        // an element with no children is content of its own (a `<break/>`, a graphic, a rule)
        c.get_first_child().is_none() || holds_content_besides(c, gone)
      },
      Some(NodeType::TextNode) => !c.get_content().trim().is_empty(),
      _ => false,
    }
  })
}

/// The printed ink of each field the frontmatter displays, removed — the field's print by construction: the ink
/// digested from the field's stored tokens (their out-of-band provenance, `\lx@field@mark`; the title-print study's
/// rule R3, frontmatter model M3 stage S1, OXIDIZED_DESIGN #715), one run of it (`field_ink_runs`) at a time, with the
/// title-block wrappers it leaves empty (not a `\label` target, which is the author's own). Called once, where the
/// frontmatter is placed (the `\maketitle` flush `\lx@frontmatterhere`, the fallback at the first sectional unit or
/// `\end{document}`), so only what was printed before it is the field's ink; every mark goes, and later prints are not
/// marked (`lx_field_ink_settled`, `Document::settle_field_ink`). A run is lifted only:
/// - when it is the whole field: what its own boxes consumed of the current call's value covers every position of
///   the field's extent (`lx_field_extent_<id>`, `covers_extent`), a macro's too, so a fragment the document takes of
///   it (`\StrLeft{\@title}{5}`, also of `About \nm`; `\@car`, `\@gobble`) stays, also beside the rest of the value
///   set in a box never shown, in a formula or in a `\phantom` (`field_ink_runs`);
/// - where the frontmatter displays the field: a document-level `<ltx:title>`, or a document-level author
///   `<ltx:creator>`, with no `<ltx:titlepage>` beside it (beside one the XSLT shows neither, LaTeXML-structure-xhtml.xsl
///   :374-375 = Perl :358-359 and :687-692, and the print is the displayed copy: 1304.0681, 1503.03277);
/// - in the document's title block: no body text before it in the document's content order
///   (`field_runs_after_body_text`: a letter's closing `\noindent\@author` stays), and `is_field_ink_wrapper` for every ancestor of both marks — never inside a
///   `{titlepage}`, a float, a note, a frontmatter element or a frame;
/// - when the fields' prints fill their lines: the paragraph or table cell around each mark holds nothing but lifted
///   ink, so a mention in prose (`Title: \thetitle`) keeps its words and a title and author set on one line go
///   together.
///
/// Every whole print in the title block is lifted, a title the document prints twice too: it shows once, in the
/// frontmatter element (user ruling 2026-10-17, "lift every whole print"; OXIDIZED_DESIGN #715).
///
/// Returns whether a title print was lifted: the text-matching dedup (`maybe_dedup_leading_title_ink`) then has
/// nothing left to explain. P2 witnesses: 1605.00182, 2402.14614, 1603.09631 (lrec `\maketitleabstract`), 1503.06457.
pub fn lift_marked_field_ink(document: &mut Document) -> Result<bool> {
  if lookup_bool("lx_field_ink_settled") {
    return Ok(false);
  }
  assign_value("lx_field_ink_settled", true, Some(Scope::Global));
  document.settle_field_ink()?;
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(false);
  };
  let (runs, marks) = field_ink_runs(document, &root);
  if marks.is_empty() {
    return Ok(false);
  }
  let no_titlepage = document
    .findnode("/ltx:document/ltx:titlepage", None)
    .is_none();
  let title_shown = no_titlepage && document.findnode("/ltx:document/ltx:title", None).is_some();
  let creators_shown = no_titlepage
    && document
      .findnode("/ltx:document/ltx:creator[@role='author']", None)
      .is_some();
  // a framed wrapper (`\fbox{\@title}`, a framed cover minipage) is the author's own ink around the print: emptied, it
  // would leave a frame the PDF does not have (as the dedup, 68s review finding 4)
  let in_title_block = |mark: &Node| {
    ancestors_below(mark, &root)
      .iter()
      .all(|a| is_field_ink_wrapper(document::get_node_qname(a)) && !a.has_attribute("framed"))
  };
  // the line a mark sits in: its nearest paragraph or table cell
  let line_of = |mark: &Node| {
    ancestors_below(mark, &root).into_iter().find(|a| {
      let q = document::get_node_qname(a);
      q == pin_static("ltx:p") || q == pin_static("ltx:td")
    })
  };
  // the whole prints of a displayed field in the title block, with their ink and their lines
  let mut candidates: Vec<(&FieldInkRun, Vec<Node>, Vec<Node>)> = Vec::new();
  for run in &runs {
    let Some((name, _)) = FIELD_MARK_FIELDS.iter().find(|(_, id)| *id == run.field) else {
      continue;
    };
    let shown = if *name == "title" {
      title_shown
    } else {
      creators_shown
    };
    let whole = !run.reread && covers_extent(&run.cover, &field_extent(run.field));
    if !shown
      || !whole
      || run.body_before
      || !in_title_block(&run.begin)
      || !in_title_block(&run.end)
    {
      continue;
    }
    let range = marked_range(&root, &run.begin, &run.end);
    if range.is_empty() {
      continue;
    }
    let lines: Vec<Node> = [&run.begin, &run.end]
      .into_iter()
      .filter_map(line_of)
      .collect();
    candidates.push((run, range, lines));
  }
  // the prints that fill their lines with lifted ink alone, all fields together (finding 5: a title and its author on
  // one line), until no more drop out
  loop {
    let gone: Vec<Node> = candidates
      .iter()
      .flat_map(|(_, range, _)| range.iter())
      .chain(marks.iter())
      .cloned()
      .collect();
    let before = candidates.len();
    candidates.retain(|(_, _, lines)| !lines.iter().any(|line| holds_content_besides(line, &gone)));
    if candidates.len() == before {
      break;
    }
  }
  let mut doomed: Vec<Node> = marks.clone();
  let mut chains: Vec<Vec<Node>> = Vec::new();
  let mut lifted_title = false;
  for (run, mut range, lines) in candidates {
    let name = FIELD_MARK_FIELDS
      .iter()
      .find(|(_, id)| *id == run.field)
      .map_or("", |(name, _)| *name);
    // the lines' breaks go with the print they end
    for line in &lines {
      range.extend(document.findnodes("descendant::ltx:break", Some(line)));
    }
    let text = normalize_frontmatter_text(
      &range
        .iter()
        .filter(|n| n.get_type() != Some(NodeType::PiNode))
        .map(|n| n.get_content())
        .collect::<Vec<_>>()
        .join(" "),
    );
    Info!(
      "frontmatter",
      "field_ink",
      s!("lifted the printed \\@{name} (\"{text}\"): the frontmatter displays it")
    );
    lifted_title |= name == "title";
    chains.push(ancestors_below(&run.begin, &root));
    chains.push(ancestors_below(&run.end, &root));
    doomed.extend(range);
  }
  remove_marked_ink(document, doomed, chains, |w| {
    is_field_ink_wrapper(document::get_node_qname(w)) && !w.has_attribute("labels")
  });
  Ok(lifted_title)
}

/// At `\end{document}`, the field marks still in the tree (made with no placement to settle them, `\lx@field@mark`)
/// removed; their ink stays.
pub fn settle_field_marks(document: &mut Document) {
  let Some(root) = document.get_document().get_root_element() else {
    return;
  };
  for mark in field_marks(document, &root) {
    document.remove_node(mark);
  }
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

/// A sectional unit's element (`ltx:section` … `ltx:glossary`): what ends the document-level
/// title block for `maybe_promote_leading_title` and `maybe_dedup_leading_title_ink`.
fn is_sectional_qname(q: SymStr) -> bool {
  with(q, |q| {
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
  })
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
  if is_sectional_qname(anchor_q) {
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
/// paragraph of the document-level title block (not inside a float, a titlepage or any
/// other container) — so an unrelated centered display block is never removed — and at
/// most once per document, at the first flush that sees the structured title (68s;
/// guard `perfect_kernel_batch68s`). Reuses the same detection helpers as the promote path.
///
/// It is the LAST RESORT of the title placement rules (OXIDIZED_DESIGN #146): a text match
/// where no print event explains the copy — a print of the marked `\@title` is lifted by its marks first
/// ([`lift_marked_field_ink`], OXIDIZED_DESIGN #715), and then this does not run (`lx_title_ink_dedup_done`).
/// Never in a document with an `<ltx:titlepage>`: there the XSLT does not display the structured
/// title (LaTeXML-structure-xhtml.xsl, the `ltx:title` template's
/// `not(parent::*/child::ltx:titlepage)`; Perl's copy :358-359) — the cover IS the displayed title,
/// and deleting its copy left none (1304.0681, 1503.03277: no `<h1>`, Perl shows the title once).
fn maybe_dedup_leading_title_ink(document: &mut Document) -> Result<()> {
  // Needs a structured document title to prioritize over.
  let Some(title) = document.findnode("/ltx:document/ltx:title", None) else {
    return Ok(());
  };
  let title_text = normalize_frontmatter_text(&title.get_content());
  if title_text.is_empty() {
    return Ok(());
  }
  // Once per document: the hand-typeset title block is judged at the first flush that sees the
  // structured title — the fallback runs at EVERY sectioning command (`\@startsection@hook`) and
  // at `\end{document}`, and a second look removed a second copy each time (one per section), down
  // into the body. A copy still there after that first look is the author's ink, not the title.
  if lookup_bool("lx_title_ink_dedup_done") {
    return Ok(());
  }
  assign_value("lx_title_ink_dedup_done", true, Some(Scope::Global));
  // A titlepage beside the title: the structured title is metadata and the cover the display.
  if document
    .findnode("/ltx:document/ltx:titlepage", None)
    .is_some()
  {
    return Ok(());
  }
  let nominal = {
    let v = lookup_float("NOMINAL_FONT_SIZE").map_or(0.0, |f| f.0);
    if v > 0.0 { v } else { 10.0 }
  };
  // Leading centered paragraphs of the document-level title block (before the first sectional
  // unit, reached only through title-block wrappers — a para, a titlepage, a cover minipage;
  // never inside a figure or any other container, whose content is the author's).
  // The nodes come from a walk of the live DOM from the absolutely queried root, so
  // they support child traversal (shared-node caveat in `maybe_promote_leading_title`).
  let Some(root) = document.findnode("/ltx:document", None) else {
    return Ok(());
  };
  let candidates = leading_centered_paragraphs(&root);
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
  // Remove the redundant title ink, pruning any title-block wrapper it leaves empty (para →
  // logical-block, a minipage that held only the title and the paragraph around it) —
  // mirroring the promote path's cleanup. Sibling `<p>`s (e.g. the author block) keep their
  // wrapper non-empty and survive. A wrapper that carries `labels` (a `\label` target) or a
  // frame (`\fbox{\begin{minipage}…}`, `framed`) is kept, empty: removing it would leave a
  // dangling reference or drop ink of its own (68s review probes p7_fbox_cover, p8).
  let mut victim = victim;
  loop {
    let parent = victim.get_parent();
    document.remove_node(victim);
    match parent {
      Some(p)
        if is_title_block_wrapper(document::get_node_qname(&p))
          && !has_element_child(&p)
          && p.get_content().trim().is_empty()
          && !p.has_attribute("labels")
          && !p.has_attribute("framed") =>
      {
        victim = p;
      },
      _ => break,
    }
  }
  Ok(())
}

/// The containers a hand-typeset title block is set in at document level: paragraph wrappers
/// (`ltx:para`, `ltx:logical-block`), a display block (`ltx:block`, a `{center}` holding a logo),
/// a cover minipage (`ltx:inline-block`, inside the `ltx:p` of its paragraph — entered only
/// through a cover paragraph, [`is_cover_box_paragraph`]). Not a float, a list, a theorem, a
/// table or a sectional unit: what is set there is the author's content even when it repeats
/// the title. Not a `{titlepage}` (`ltx:titlepage`) either: the cover is the displayed title
/// there, and the structured title is metadata ([`maybe_dedup_leading_title_ink`]).
fn is_title_block_wrapper(q: SymStr) -> bool {
  with(q, |q| {
    matches!(
      q,
      "ltx:para" | "ltx:logical-block" | "ltx:block" | "ltx:inline-block" | "ltx:p"
    )
  })
}

/// A non-centered `ltx:p` holding only boxes: its element children are all `ltx:inline-block`s
/// and its text is whitespace — the `\noindent\begin{minipage}…\end{minipage}` cover paragraph
/// (2010.07703). A paragraph of running prose with a `\parbox` in it is not: the box's words
/// are the sentence's (68s review probe p2_parbox_only, pdflatex prints them mid-sentence).
fn is_cover_box_paragraph(p: &Node) -> bool {
  let mut has_box = false;
  for c in p.get_child_nodes() {
    match c.get_type() {
      Some(NodeType::ElementNode) => {
        if document::get_node_qname(&c) != pin_static("ltx:inline-block") {
          return false;
        }
        has_box = true;
      },
      Some(NodeType::TextNode) if !c.get_content().trim().is_empty() => return false,
      _ => {},
    }
  }
  has_box
}

/// The `<ltx:p align="center">` paragraphs of `node` reached only through title-block wrappers
/// ([`is_title_block_wrapper`]), in document order; a centered paragraph is taken whole (a
/// minipage inside it is part of its text), any other `ltx:p` is entered only when it is a
/// cover paragraph of boxes ([`is_cover_box_paragraph`]).
fn collect_title_block_paragraphs(node: &Node, out: &mut Vec<Node>) {
  // A framed wrapper (`\fbox{\begin{minipage}…}`, `framed`) is the author's own ink around the title: removing the copy
  // inside left an empty frame the PDF does not have, so it is never searched (main-session ruling 2026-10-16, until
  // the last-resort step promotes the author's block in place; OXIDIZED_DESIGN #146).
  if node.has_attribute("framed") {
    return;
  }
  let q = document::get_node_qname(node);
  if q == pin_static("ltx:p") {
    if node.get_attribute("align").as_deref() == Some("center") {
      out.push(node.clone());
      return;
    }
    if !is_cover_box_paragraph(node) {
      return;
    }
  }
  if !is_title_block_wrapper(q) {
    return;
  }
  for c in node.get_child_nodes() {
    if c.get_type() == Some(NodeType::ElementNode) {
      collect_title_block_paragraphs(&c, out);
    }
  }
}

/// The `<ltx:p align="center">` paragraphs of the document-level title block: those reached from
/// `root`'s children only through title-block wrappers ([`collect_title_block_paragraphs`]) and
/// before its first sectional child (a back-matter list is passed over), in document order. A
/// paragraph inside a float, a list, a theorem, a titlepage or any other container, or inside a
/// chapter or part, is not the leading title block: the walk once took every centered paragraph
/// before the first `<ltx:section>`, so a figure's centered line and a chapter's opening block were
/// removed as the title (68p review probe `float.tex`; 2608.10928 the witness of the rule; a cover
/// minipage 2010.07703; the titlepage covers 1304.0681, 1503.03277, 1601.07172, 2606.13892 are kept
/// whole since 68s round 2). Only the document's children are visited and the walk stops at the
/// first sectional unit, so it stays linear in the leading block (it runs once per document; the
/// old XPath scanned `preceding::` for every centered paragraph at every sectioning command —
/// 2208.14314: 121 of 133 s; 2108.10297: 113 of 146 s; PERFORMANCE.md Principle 6).
fn leading_centered_paragraphs(root: &Node) -> Vec<Node> {
  let mut out = Vec::new();
  let mut cur = root.get_first_child();
  while let Some(node) = cur {
    if node.get_type() == Some(NodeType::ElementNode) {
      let q = document::get_node_qname(&node);
      // WORKAROUND, not a rule of the title block: a bibliography, index or glossary list placed
      // ahead of the cover does not end it. 2010.07703's bibliography (`\bibliography` at
      // thesis.tex:480, under `\part{Listings and Bibliography}`) is built at the head of the XML,
      // before its cover minipage, by bibentry's `\nobibliography*` in the preamble
      // (texpreamble.tex:820) — a separate defect (RED index-bib/nobibliography_star_prints_nothing,
      // SYNC_STATUS); once that is fixed this skip has nothing to pass over.
      let back_matter = with(q, |q| {
        matches!(q, "ltx:bibliography" | "ltx:index" | "ltx:glossary")
      });
      if is_sectional_qname(q) && !back_matter {
        break;
      }
      collect_title_block_paragraphs(&node, &mut out);
    }
    cur = node.get_next_sibling();
  }
  out
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
  let answered = answered_labels(document);
  relocate_annotations(document)?;
  show_unanswered_marks(document, &answered)?;
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
/// Moves every annotation, `<ltx:contact>`, `<ltx:note>` and `<ltx:rdf>`: an author `\thanks` is a
/// marked `<ltx:note role="thanks">` (OXIDIZED_DESIGN #156), so a trailing `\thanks` on
/// a nameless comma-split creator would otherwise be dropped with the empty creator —
/// witness 1510.02728 (`\author{Sani,~\IEEEmembership{…} Vosoughi,~\IEEEmembership{…}%
/// \thanks{…NSF…}}`), where the note must land on the last real author, as a contact did;
/// and an author's metadata (OXIDIZED_DESIGN #479: an AASTeX 7 `\email` after an `\author` whose name prints nothing).
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
              q == "ltx:contact" || q == "ltx:note" || q == "ltx:rdf"
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
  // …and no other author is to receive one of their kind: an author whose mark requests a label a pending contact of a
  // role the last author's contacts hold carries gets it from `relocate_annotations`, after this — the last author's
  // contacts are then the run's, not a pile (`\author{Ann Able$^{1}$}\author{Bob Baker}\affil{$^1$Univ X}\affil{Univ Y}
  // \affil{Univ W}` handed Univ Y to Ann and kept Univ W alone on Bob; 77n r5 review probes g01-g06, m11). A pending
  // contact of another role leaves the pile one (2605.30274's e-mail lines on the last of two authors whose affiliations
  // are mark-picked).
  let last_roles: HashSet<String> = contacts_of(&creators[n - 1])
    .iter()
    .map(|contact| contact.get_attribute("role").unwrap_or_default())
    .collect();
  let mut pending_labels: HashSet<String> = HashSet::default();
  for pending in document.findnodes(".//*[@role='pending']", None) {
    for note in element_nodes(&pending) {
      if !last_roles.contains(&note.get_attribute("role").unwrap_or_default()) {
        continue;
      }
      let labels = [
        note.get_attribute("_label").unwrap_or_default(),
        pending.get_attribute("_annotations").unwrap_or_default(),
      ];
      for label in labels.iter().flat_map(|labels| labels.split(',')) {
        if !label.is_empty() {
          pending_labels.insert(label.to_string());
        }
      }
    }
  }
  let answered_later = |creator: &Node| {
    creator
      .get_attribute("_annotations")
      .unwrap_or_default()
      .split(',')
      .any(|label| pending_labels.contains(label))
  };
  if creators[..n - 1].iter().any(answered_later) {
    return Ok(());
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
  drop_trailing_paragraph_breaks(document, &opened);
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

/// The labels the frontmatter entries carry (`_label`), each also without its prefix, as [`relocate_annotations`]
/// matches them (`institute:1` answers `affiliation:1`), and every mark an affiliation printed (`_bymark`: llncs's
/// `\institute{$^{1,3}$ Univ A}` answers a `\inst{3}` too, though only its first mark labels it): what an author's mark
/// request can find. A creator's own role-sequence label (`author:N`) answers no mark.
fn answered_labels(document: &mut Document) -> HashSet<String> {
  let mut answered = HashSet::default();
  for node in document.findnodes(".//*[@_bymark]", None) {
    for mark in node.get_attribute("_bymark").unwrap_or_default().split(',') {
      if !mark.is_empty() {
        answered.insert(mark.to_string());
        answered.insert(s!("affiliation:{mark}"));
      }
    }
  }
  for node in document.findnodes(".//*[@_label][not(self::ltx:creator)]", None) {
    for label in node.get_attribute("_label").unwrap_or_default().split(',') {
      // (nor does a contact's placement by that label, `label=author:N`: 2403.00801's `jiawei2020@…` placed on the
      // second author answered every mark "2")
      if label.is_empty() || label.starts_with("author:") {
        continue;
      }
      answered.insert(label.to_string());
      if let Some(pos) = label.find(':') {
        answered.insert(label[pos + 1..].to_string());
      }
    }
  }
  answered
}

/// An author's affiliation mark that no entry answers stays in the name, as LaTeX prints it (`Ann Able$^*$ and Bob
/// Bee$^1$` with no `$^1$` line; `Gus Gray\IEEEauthorrefmark{1}` whose legend is a `\thanks`, 2408.02464): read as
/// a request, it was taken out of the name to link an affiliation that is not there. A mark the name already shows
/// is not repeated. (Perl drops it too; repro sectioning-frontmatter/author_unmatched_numeric_mark.)
fn show_unanswered_marks(document: &mut Document, answered: &HashSet<String>) -> Result<()> {
  for creator in document.findnodes(".//ltx:creator[@_annotations]", None) {
    let Some(mut personname) = document
      .findnodes("ltx:personname", Some(&creator))
      .into_iter()
      .next()
    else {
      continue;
    };
    let shown = shown_marks(document, &creator);
    let shown_sups: Vec<String> = document
      .findnodes("ltx:personname//ltx:sup", Some(&creator))
      .iter()
      .map(|sup| sup.get_content())
      .collect();
    // (a symbol mark nothing answers is shown too — 1807.01584, 1805.02699, 2203.01270 — except one that placed one of
    // this author's contacts and was dropped from it, `$^\dagger$baker@y.edu`: answered on this author only,
    // `answered_symbol_key`; one the author's email shows as well stays on the name too, as printed: 1910.00401)
    let answered_here: Vec<String> = document
      .findnodes(".//*[@_answers]", Some(&creator))
      .iter()
      .flat_map(|node| {
        node
          .get_attribute("_answers")
          .unwrap_or_default()
          .split(',')
          .map(str::to_string)
          .collect::<Vec<_>>()
      })
      .collect();
    let text_marks: Vec<String> = creator
      .get_attribute("_textmarks")
      .unwrap_or_default()
      .split(',')
      .map(str::to_string)
      .collect();
    let mut marks: Vec<String> = Vec::new();
    for label in creator
      .get_attribute("_annotations")
      .unwrap_or_default()
      .split(',')
    {
      let Some(mark) = label.strip_prefix("affiliation:") else {
        continue;
      };
      if answered.contains(label) || answered.contains(mark) {
        continue;
      }
      let Some(mut text) = unanswered_mark_text(mark) else {
        continue;
      };
      // (the glyph follows the mark's mode: a `*` requested in text is `*`, in math `∗`)
      if mark == "*" && text_marks.iter().any(|m| m == label) {
        text = "*".to_string();
      }
      if symbol_mark_key(mark).is_some_and(|key| answered_here.contains(&key)) {
        continue;
      }
      // (a symbol the name shows in a superscript of its own is shown: a `$^*$`, `\lx@frontmatter@keepsup`, also
      // requests; `*` and `∗` are one mark)
      let unify = |text: &str| text.replace('\u{2217}', "*");
      let symbol_shown = !text.chars().all(char::is_alphanumeric)
        && shown_sups
          .iter()
          .any(|sup| unify(sup).contains(&unify(&text)));
      if symbol_shown || shown.contains(&text) || marks.contains(&text) {
        continue;
      }
      marks.push(text);
    }
    if marks.is_empty() {
      continue;
    }
    let mut sup = Node::new("sup", None, document.get_document())?;
    if let Some(namespace) = personname.get_namespace() {
      let _ = sup.set_namespace(&namespace);
    }
    sup.append_text(&marks.join(","))?;
    personname.add_child(&mut sup)?;
  }
  Ok(())
}

/// Add `label` (comma separated) to the requests of the frontmatter entry being digested (`_annotations`), and, for a
/// superscript requested in text (`\textsuperscript{1,*}`, not `$^{1,*}$`: `superscript`), to its text-mode marks
/// (`_textmarks`): such a `*` prints as `*`, a math one as `∗` (`show_unanswered_marks`). A mark command (`\inst`,
/// `\IEEEauthorrefmark`) sets its mark in math (llncs.cls `\def\inst#1{\unskip$^{#1}$}`), whatever mode it is read in.
fn add_annotation_labels(label: String, superscript: bool) {
  let in_text = superscript && !lookup_bool("IN_MATH");
  with_pending_entry_attr(move |attr| {
    let labels = attr.get("_annotations").cloned().unwrap_or_default();
    let newval = if labels.is_empty() {
      label.clone()
    } else {
      s!("{labels},{label}")
    };
    DebugFeature!("frontmatter", "FRONT add annotation label {label}");
    attr.insert("_annotations".to_string(), newval);
    if in_text {
      let text_marks = attr.get("_textmarks").cloned().unwrap_or_default();
      let newval = if text_marks.is_empty() {
        label.clone()
      } else {
        s!("{text_marks},{label}")
      };
      attr.insert("_textmarks".to_string(), newval);
    }
  });
}

/// How an unanswered affiliation mark prints: a number or letter as it is, a superscript footnote symbol (`$^{\dag}$`,
/// which labels affiliations as a mark, `is_symbol_mark`; its label keeps the control sequence) as its glyph; `None`
/// for anything else (a fuzzy or composite label).
fn unanswered_mark_text(mark: &str) -> Option<String> {
  if !mark.is_empty() && mark.chars().all(char::is_alphanumeric) {
    return Some(mark.to_string());
  }
  let glyph = match mark {
    "\\dag" | "\\dagger" => "\u{2020}",
    "\\ddag" | "\\ddagger" => "\u{2021}",
    // (pdflatex sets a math `*` as `∗`; one requested in text stays `*`, `show_unanswered_marks`)
    "\\ast" | "*" => "\u{2217}",
    "\\star" => "\u{22C6}",
    "\\S" => "\u{A7}",
    "\\P" => "\u{B6}",
    "\\bullet" => "\u{2219}",
    "\\sharp" => "\u{266F}",
    "\\|" => "\u{2016}",
    "\\#" | "#" => "#",
    "\u{2020}" | "\u{2021}" | "\u{2217}" | "\u{A7}" | "\u{B6}" => mark,
    _ => return None,
  };
  Some(glyph.to_string())
}

/// What a frontmatter annotation says: its text, or an `ltx:rdf`'s `content`/`resource` (OXIDIZED_DESIGN #479), which
/// has no text.
fn annotation_value(note: &Node) -> String {
  if note.get_name() == "rdf" {
    note
      .get_attribute("content")
      .or_else(|| note.get_attribute("resource"))
      .unwrap_or_else(|| note.get_content())
  } else {
    note.get_content()
  }
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
  // A label's targets: the exact index, then the prefix-stripped fallbacks for a misused prefix — creators only, the
  // annotations a prefix is misused on: a title's `tnote:t1` must not take an author's orphaned `fn:t1` (63h review,
  // `elsarticle_orphan_fntext_stays_off_the_title`).
  let indexed_targets = |label: &str| -> Option<Vec<Node>> {
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
  // A superscript mark and a bracketed label printed as the same superscript are one label (authblk sets
  // `\author[1]{Bob}` and `\affil[1]{Univ X}` as a `\textsuperscript{1}`, `\AB@authnote`/`\AB@affilnote`,
  // authblk.sty:98-99, :142, :162): the label a mark set (`\affil{$^1$Univ X}`, `affiliation:1` for the authors
  // showing that mark) also reaches the authors whose label is its mark (`\author[1]{Bob}`, `1`), and a bracketed
  // label (`\affil[1]{Univ X}`, `1`) the authors requesting it by a mark (`\author{Bob$^{1}$}`, whose answered mark
  // the name no longer shows; `\inst{1}`: `affiliation:1`): Bob lost Univ X to the author whose spelling matched
  // (77n r6 review probes f2, f7). Only authblk's names and `\inst` request `affiliation:N` from a printed label, so
  // a class that renumbers its labels (elsarticle prints `\author[1]` as "a") is untouched (r7 review: elsarticle,
  // cas, llncs, revtex, aastex, IEEE probes and 12 elsarticle/cas papers unchanged).
  let find_targets = |label: &str| -> Option<Vec<Node>> {
    let mut targets = indexed_targets(label).unwrap_or_default();
    let respelt: Vec<Node> = if let Some(keys) = by_mark.get(label) {
      keys
        .iter()
        .filter_map(|key| labeltable.get(key.as_str()))
        .flatten()
        .filter(|target| target.get_name() == "creator")
        .cloned()
        .collect()
    } else if !label.is_empty() && !label.contains(':') {
      labeltable
        .get(&s!("affiliation:{label}"))
        .into_iter()
        .flatten()
        .filter(|target| target.get_name() == "creator")
        .cloned()
        .collect()
    } else {
      Vec::new()
    };
    for target in respelt {
      if !targets.contains(&target) {
        targets.push(target);
      }
    }
    (!targets.is_empty()).then_some(targets)
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
  // An orphan's owners among the authors (below), by its label and role.
  let owned_by = |label: &str, note: &Node| {
    owners.get(label).filter(|named| {
      named.len() == 1 || is_institute_role(&note.get_attribute("role").unwrap_or_default())
    })
  };
  // The annotations an author takes, by kind and text, with their labels: a copy of one under another label that no
  // author takes is no orphan to share (`\address{$^{1,5}$Lab …}` with no author marked 5, osa_journal_sty.rs
  // `lines_per_label`, 1101.5841). Read before placing any, as the orphan copy may come first (`$^{5,1}$`).
  let mut taken: Vec<(String, Option<String>, String, String)> = Vec::new();
  for pending in &pending_nodes {
    for note in element_nodes(pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      let content = annotation_value(&note).trim().to_string();
      if !label.is_empty()
        && !content.is_empty()
        && (find_targets(&label).is_some() || owned_by(&label, &note).is_some())
      {
        taken.push((note.get_name(), note.get_attribute("role"), content, label));
      }
    }
  }
  for mut pending in pending_nodes {
    let mut promote_this = false;
    // the stub's notes placed elsewhere, which a promoted stub must not keep a second time
    let mut placed: Vec<Node> = Vec::new();
    for note in element_nodes(&pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      // An annotation given before any author with no label to find one by (cas's `\affiliation{…}` above the first
      // `\author`, which cas-common.sty:1252-1290 prints under the authors) is an orphan like the labelled ones below:
      // a sole author's, else shared. Perl drops it unplaced. Guard:
      // `perfect_kernel_batch63::cas_affiliation_before_author`.
      let unlabelled = label.is_empty();
      // (an empty one is nothing to keep: it made a nameless creator of its own; nor one an author already holds — a
      // store set between two `\maketitle`s is handed on to the authors a later `\author` names, and queued too:
      // `perfect_kernel_batch59::store_set_before_a_superseding_author`)
      let empty = annotation_value(&note).trim().is_empty() && element_nodes(&note).is_empty();
      // (metadata, an `ltx:rdf`, is held by an author who has its value in any annotation: 2608.17008's `\email{x}` ahead
      // of the authors, then `\email[show]{x}` on the first, is that author's displayed address)
      let held = || {
        let (role, content) = (note.get_attribute("role"), annotation_value(&note));
        let metadata = note.get_name() == "rdf";
        authors.iter().any(|author| {
          element_nodes(author).iter().any(|c| {
            let same_kind = c.get_name() == note.get_name() && c.get_attribute("role") == role;
            (same_kind || (metadata && matches!(c.get_name().as_str(), "contact" | "rdf")))
              && annotation_value(c).trim() == content.trim()
          })
        })
      };
      if unlabelled && (pending.get_name() != "creator" || empty || held()) {
        continue;
      }
      let targets = if unlabelled {
        match authors.as_slice() {
          [author] => Some(vec![author.clone()]),
          _ => None,
        }
      } else {
        find_targets(&label)
      };
      if let Some(targets) = targets {
        placed.push(note.clone());
        for target in targets {
          // (nor twice under two labels: an entry marked with several labels is one copy a label — OSA's
          // `\address{$^{1,2}$Univ X}`, osa_journal_sty.rs `lines_per_label`, 1406.0824 — which reaches an author
          // marked with two of them once; two entries under one label are two, as the PDF prints them; an annotation
          // without text, an icon's link, is compared by nothing. Guards
          // `perfect_kernel_batch64u::osa_several_labels_one_line_each`,
          // `perfect_kernel_batch61::marked_affiliation_review_shapes`)
          let (role, content) = (note.get_attribute("role"), annotation_value(&note));
          if !content.trim().is_empty()
            && element_nodes(&target).iter().any(|c| {
              c.get_name() == note.get_name()
                && c.get_attribute("role") == role
                && c.get_attribute("_label").is_some_and(|l| l != label)
                && annotation_value(c).trim() == content.trim()
            })
          {
            continue;
          }
          DebugFeature!("frontmatter", "FRONT Moving annotation for {label}");
          let mut target = target;
          document.append_clone(&mut target, vec![note.clone()])?;
          contact_in_title_as_note(document, &target)?;
        }
      } else {
        let content = annotation_value(&note);
        if !unlabelled
          && owned_by(&label, &note).is_none()
          && taken.iter().any(|(name, role, text, other)| {
            *name == note.get_name()
              && *role == note.get_attribute("role")
              && *text == content.trim()
              && *other != label
          })
        {
          placed.push(note.clone());
          continue;
        }
        if !unlabelled && !shared_orphan(&note) {
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
        if let Some(named) = owned_by(&label, &note) {
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
      // below the authors, where a shared note prints: a stub queued before the first author stood above them
      // (cas's `\affiliation{…}` above its `\author`s; aastex's `\correspondingauthor`+`\email` first, 2408.07136)
      if let Some(last) = authors.last() {
        move_content_free_after(std::slice::from_ref(&pending), last.clone())?;
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
/// the 2), past braces and font and size switches (`\rm a`, `\mathrm{a}`, `{\rm 2}`, `\footnotesize 1`: the a, the 2,
/// the 1), and before a symbol or closing parenthesis after it (`2*`, `1\dagger`, `1†`, `1)`: the 2, the 1; the `1)` of
/// 2306.17039, as `clean_frontmatter_labels` reads its label). A symbol alone or other markup (`\footnote{…}`) is no
/// mark. Both the marks an affiliation is labelled by and the marks an author shows are read so (63i review).
fn script_marks(script: &str) -> Vec<String> {
  // (the whole script under its switches, `\footnotesize\ensuremath{1,2}`: the 1 and the 2; 2411.14110)
  let script = past_mark_switches(script);
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
      let item: String = past_mark_switches(item)
        .chars()
        .filter(|c| !matches!(c, '{' | '}') && !c.is_whitespace())
        .collect();
      let lead: String = item.chars().take_while(|c| c.is_alphanumeric()).collect();
      let rest = &item[lead.len()..];
      let symbol_after = rest.is_empty()
        || rest.starts_with(['*', '\\', ')'])
        || rest.starts_with(|c: char| !c.is_ascii() && !c.is_alphanumeric());
      (!lead.is_empty() && symbol_after).then_some(lead)
    })
    .collect()
}

/// The font and size switches, and the math wrapper, a mark is printed under (`{\rm 2}`, `\mathrm{a}`,
/// `\textsuperscript{\footnotesize\ensuremath{1}}`, 2502.16662 and 2411.14110's `\IEEEauthorrefmark`), which neither
/// an affiliation's mark nor its leading text is.
const MARK_SWITCHES: [&str; 32] = [
  "\\rm",
  "\\it",
  "\\bf",
  "\\sl",
  "\\sf",
  "\\tt",
  "\\em",
  "\\mathrm",
  "\\mathit",
  "\\mathbf",
  "\\mathsf",
  "\\textrm",
  "\\textit",
  "\\textbf",
  "\\textsf",
  "\\textup",
  "\\textnormal",
  "\\itshape",
  "\\upshape",
  "\\bfseries",
  "\\rmfamily",
  "\\normalfont",
  "\\tiny",
  "\\scriptsize",
  "\\footnotesize",
  "\\small",
  "\\normalsize",
  "\\large",
  "\\Large",
  "\\ensuremath",
  "\\scriptstyle",
  "\\textstyle",
];

/// `text` past the switches it opens with ([`MARK_SWITCHES`]), with the braces and spaces around them.
fn past_mark_switches(text: &str) -> &str {
  let mut rest = text;
  loop {
    rest = rest.trim_start_matches(|c: char| c == '{' || c.is_whitespace());
    let Some(after) = rest.strip_prefix('\\') else {
      return rest;
    };
    let len = after
      .find(|c: char| !c.is_ascii_alphabetic())
      .unwrap_or(after.len());
    if len == 0 || !MARK_SWITCHES.contains(&&rest[..=len]) {
      return rest;
    }
    rest = &after[len..];
  }
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
///
/// The family, series and shape are likewise the document's `\familydefault`, `\seriesdefault` and `\shapedefault`
/// that `\reset@font` selects (`\edef\f@family{\familydefault}`, latex.ltx:14115-14117), where they name a font
/// LaTeXML knows: a `\renewcommand\familydefault{\sfdefault}` document's footnotes and tags are sans, as its PDF
/// (CMSS8 footnote text), not Perl's serif `textDefault`. OXIDIZED_DESIGN_DIVERGENCES #779; guard
/// `perfect_kernel_batch77u::family_default_sets_the_body`.
pub fn neutralize_font() {
  let mut font = Font::text_default();
  if let Some(encoding) = encoding_named("\\encodingdefault") {
    font.encoding = Some(Cow::Owned(encoding));
  }
  let components = [
    font_code_named("\\familydefault").and_then(|code| font::lookup_font_family(&code)),
    font_code_named("\\seriesdefault").and_then(|code| font::lookup_font_series(&code)),
    font_code_named("\\shapedefault").and_then(|code| font::lookup_font_shape(&code)),
  ];
  for component in components.into_iter().flatten() {
    font = font.merge_ref(component);
  }
  sync_nfss_font_codes(&font);
  assign_value("font", font, Some(Scope::Local));
  assign_value("mathfont", Font::math_default(), Some(Scope::Local));
}

/// The NFSS code the macro `cs` (`\encodingdefault`, `\familydefault`, …) names, expanded as `\fontencoding` or
/// `\edef\f@family{…}` expands it. The common cases are read without expanding, as they are on every tag: a literal
/// name (`OT1`), or a chain of parameterless macros ending in one (`\familydefault` → `\rmdefault` → `cmr`). `None`
/// when undefined (plain TeX) or empty.
fn font_code_named(cs: &str) -> Option<String> {
  let mut token = T_CS!(cs);
  let mut literal = None;
  for _ in 0..8 {
    let defn = lookup_definition(&token).ok()??;
    let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
      break;
    };
    if defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
    {
      break;
    }
    let tokens = body.unlist_ref();
    if tokens
      .iter()
      .all(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
    {
      literal = Some(body.to_string());
      break;
    }
    match tokens[..] {
      [next] if matches!(next.get_catcode(), Catcode::CS) => token = next,
      _ => break,
    }
  }
  let code = match literal {
    Some(code) => code,
    None => do_expand(T_CS!(cs)).ok()?.to_string(),
  };
  let code = code.trim();
  (!code.is_empty()).then(|| code.to_string())
}

/// The encoding the macro `cs` (`\encodingdefault`, `\latinencoding`) names ([`font_code_named`]). `None` when
/// undefined (plain TeX) or not an encoding name.
fn encoding_named(cs: &str) -> Option<String> {
  font_code_named(cs).filter(|encoding| encoding != "ASCII" && !encoding.contains('\\'))
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
    // A vertical box's own list (`VBoxContents`), where `\unvbox` puts a box's lines back (tex_box.rs `vlist_of`).
    if mode == "internal_vertical" {
      assign_frame_value("lx@vbox@list", true);
    }
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
  insert_block_as(document, contents, block_attr, false, true)
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
    return insert_block_as(document, contents, block_attr, in_paragraph, true);
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
  let mut nodes = insert_block_as(document, contents, block_attr, in_paragraph, true)?;
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

/// Whether the element `node` was built from one thing that shows: its box is a whatsit whose last argument — a
/// constructor's body (`\rotatebox`, `\scalebox`, `\resizebox`, `\fbox`, `\colorbox`) — has exactly one item,
/// nested groups flattened, that shows something ([`typesets_content`]). Any other box refuses. An alignment
/// environment around the transform is read by [`environment_is_sole_content`] instead.
fn box_is_sole_content(document: &Document, node: &Node) -> bool {
  fn shown(d: &Digested) -> usize {
    match d.data() {
      DigestedData::List(l) => l.borrow().boxes.iter().map(shown).sum(),
      _ => usize::from(typesets_content(d)),
    }
  }
  let Some(whatsit) = document.get_node_box(node) else {
    return false;
  };
  let DigestedData::Whatsit(w) = whatsit.data() else {
    return false;
  };
  let w = w.borrow();
  matches!(w.get_args().last(), Some(Some(body)) if shown(body) == 1)
}

/// Whether the environment an `ltx:_CaptureBlock_` is being built for is an alignment wrapper (`{center}`,
/// `{flushleft}`, `{flushright}`: its box is the environment's whatsit, whose body is what it holds between `\begin`
/// and `\end`) holding one thing that shows: the transform alone, not a transform and a tabular after it. Any other
/// environment refuses (`{minipage}`, `{framed}`).
fn environment_is_sole_content(document: &Document, capture: &Node) -> bool {
  fn shown(d: &Digested) -> usize {
    match d.data() {
      DigestedData::List(l) => l.borrow().boxes.iter().map(shown).sum(),
      _ => usize::from(typesets_content(d)),
    }
  }
  let Some(whatsit) = document.get_node_box(capture) else {
    return false;
  };
  let DigestedData::Whatsit(w) = whatsit.data() else {
    return false;
  };
  let w = w.borrow();
  // only the alignment wrappers, which make no box of their own (a `{minipage}` keeps its width and class as a panel,
  // which the caption would leave: two minipages side by side shrank to the rotated box; a `{framed}` frame would move
  // onto the float)
  [
    "\\begin{center}",
    "\\begin{flushleft}",
    "\\begin{flushright}",
  ]
  .contains(&w.get_definition().get_cs_name().as_ref())
    && matches!(w.get_body(), Ok(Some(body)) if shown(&body) == 1)
}

/// Whether a box's closing caption may be moved from inside the transform `context` to the float `home` its climb
/// found ([`insert_block_as`]): every element from `context` up to `home` was built from this box alone
/// ([`box_is_sole_content`]: no content beside or after it would follow the moved insertion point out of a frame
/// or a transform) and none is a paragraph, and `home` is a float whose own content holds no other caption (one
/// `\lx@@caption`: a second would make the two one float, with one number for two `\label`s — two `\resizebox`ed
/// captioned panels, a float's own `\caption` after the box, `\subcaption` panels). Otherwise Perl's container
/// stands. Witnesses heria/heria-proposal (heria.cls:737-745 `SidewaysFigure`), 1810.08269; guards
/// `perfect_kernel_batch65a::caption_in_a_rotated_parbox_stays_with_its_figure`,
/// `caption_hoist_never_merges_two_captions`, `caption_hoist_never_moves_content_beside_the_box`.
/// OXIDIZED_DESIGN #529.
fn caption_hoist_is_safe(document: &Document, context: &Node, home: &Node) -> bool {
  fn captions(d: &Digested) -> usize {
    match d.data() {
      DigestedData::List(l) => l.borrow().boxes.iter().map(captions).sum(),
      DigestedData::Whatsit(w) => {
        let w = w.borrow();
        usize::from(w.get_definition().get_cs_name() == "\\lx@@caption")
          + w.get_args().iter().flatten().map(captions).sum::<usize>()
          + w
            .get_body()
            .ok()
            .flatten()
            .map_or(0, |body| captions(&body))
      },
      _ => 0,
    }
  }
  let is_float = |n: &Node| with(document::get_node_qname(n), |q| PANEL_FLOATS.contains(&q));
  // The home an alignment environment's capture block (`{center}`, `{flushleft}`, `{flushright}` around the
  // transform: the caption lands in what the environment builds, which its float then holds): the environment holds
  // the transform alone, and the float right above it is the one whose captions count.
  let float = if is_float(home) {
    home.clone()
  } else if with(document::get_node_qname(home), |q| {
    q == "ltx:_CaptureBlock_"
  }) && environment_is_sole_content(document, home)
    && let Some(parent) = home.get_parent().filter(is_float)
  {
    parent
  } else {
    return false;
  };
  let mut node = Some(context.clone());
  while let Some(n) = node {
    if &n == home {
      return document
        .get_node_box(&float)
        .is_some_and(|float| captions(&float) == 1);
    }
    // A paragraph on the way is open to what follows the box in it (`\fbox{\rotatebox…} trailing text`): its
    // node box is only what opened it, so nothing tells the rest of it is not there.
    if with(document::get_node_qname(&n), |q| {
      q == "ltx:p" || q == "ltx:para"
    }) {
      return false;
    }
    if !box_is_sole_content(document, &n) {
      return false;
    }
    node = n.get_parent();
  }
  false
}

/// Where the content a box cannot hold is placed after it ([`insert_block_as`]'s hoist): the nearest ancestor of
/// `context` that holds all of `tail_tags`, climbing only through text containers, never out of a list, an alignment
/// or a float, and the node it follows there (the box itself, or the ancestor's child the climb came from).
fn tail_home(
  context: &Node,
  container: &Node,
  tail_tags: &[SymStr],
  kept_empty: bool,
) -> Option<(Node, Node)> {
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
  loop {
    if tail_tags
      .iter()
      .all(|t| with(*t, |s| document::can_contain(&ancestor, s)))
    {
      return Some((anchor, ancestor));
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
  None
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
///
/// `boxed`: the block is a TeX box (a `\vbox`, `\vtop`, `\parbox`, a minipage, a `p` column, a framed box), its
/// capture marked for a frontmatter field's print (core `Document::mark_field_box`, OXIDIZED_DESIGN #715); not an
/// aligning list environment's (`{center}`, `{flushright}`: [`aligning_environment`]).
fn insert_block_as(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
  in_paragraph: bool,
  boxed: bool,
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
  // the box's own element: the mark rides the capture's rename, an attribute-taking lone child, a paragraph-move; an
  // unwrapped capture leaves its material in the context, unmarked
  if boxed {
    document.mark_field_box(&mut container);
  }
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
  // A box no candidate holds whole only because its paragraphs are wrapped in `ltx:para` — a caption beside a
  // `\noindent` paragraph (adjustbox's collectbox; `\caption{First} \noindent x` in a minipage, 2312.04535, 2209.02973,
  // 2404.07198, 2404.18532, 2406.00259, 2502.04073) — holds them unwrapped, the `ltx:p`s a candidate takes beside the
  // caption (the #244 demotion). Perl warns and falls back to an `ltx:block` the caption and the para are invalid in.
  let para = pin_static("ltx:para");
  if !candidate_set
    .iter()
    .any(|c| node_tags.iter().all(|t| holds(*c, *t)))
    && node_tags.contains(&para)
  {
    let unwrapped: Vec<SymStr> = nodes
      .iter()
      .flat_map(|n| {
        if document::get_node_qname(n) == para {
          content_nodes(n)
            .iter()
            .map(document::get_node_qname)
            .collect::<Vec<_>>()
        } else {
          vec![document::get_node_qname(n)]
        }
      })
      .collect();
    if candidate_set
      .iter()
      .any(|c| unwrapped.iter().all(|t| holds(*c, *t)))
    {
      for wrapper in nodes.iter().filter(|n| document::get_node_qname(n) == para) {
        // (what the paragraph carried goes to the elements it held, its id going with it: a text paragraph (`ltx:p`)
        // takes it as the para had it; a box (an adjustbox's `ltx:inline-block`, a picture) is centred as a box
        // (`ltx_centering`: an `align` on it centres only its own text), and the paragraph's indentation is not its)
        let attributes = wrapper.get_attributes();
        for mut child in wrapper.get_child_elements() {
          let tag = document::get_node_qname(&child);
          let text_block = tag == pin_static("ltx:p");
          for (key, value) in &attributes {
            if key == "id" || key == "xml:id" || key.starts_with('_') {
              continue;
            }
            if key == "class" {
              for class in value.split_whitespace() {
                if text_block || (class != "ltx_indent" && class != "ltx_noindent") {
                  document.add_class(&mut child, class)?;
                }
              }
            } else if key == "align" && !text_block {
              // (a box's other alignments are not carried: no class places a box right; a residual)
              if value == "center" {
                document.add_class(&mut child, "ltx_centering")?;
              }
            } else if !child.has_attribute(key) && document::sym_can_have_attribute(tag, pin(key)) {
              document.set_attribute(&mut child, key, value)?;
            }
          }
        }
        document.unwrap_nodes(wrapper.clone())?;
      }
      nodes = content_nodes(&container);
      node_tags = nodes.iter().map(document::get_node_qname).collect();
    }
  }
  let holds_all = |c: SymStr| node_tags.iter().all(|t| holds(c, *t));
  // Perl's order (TeX_Box.pool.ltxml:503-509): the first candidate that holds all the content and that the context
  // holds, else the first that holds all the content.
  let allowed = |c: SymStr| document::can_contain_qsym(context_tag, c);
  let fits_all = candidate_set
    .iter()
    .copied()
    .find(|c| holds_all(*c) && allowed(*c))
    .or_else(|| candidate_set.iter().copied().find(|c| holds_all(*c)));
  let container_tag = match fits_all {
    // One the context does not hold (a figure for a `\parbox` with a closing `\caption` in a `\rotatebox`'s
    // `ltx:inline-block`, LaTeXML-misc.rnc:28: heria.cls:737-745 `SidewaysFigure`, schema-invalid) gives way to one it
    // holds for the box's leading content, when what that one cannot hold is the caption closing the box and it has
    // a home after it ([`tail_home`]: the caption goes to the figure the box is in, read in the same order; 1810.08269).
    // A box that opens with its caption (`\rotatebox{90}{\begin{minipage}…\caption…\begin{tabular}…}`) keeps Perl's
    // choice: hoisting the caption would put it after the content it precedes, and an empty box would be left in the
    // transform; so does one whose closing part holds more than a caption, one beside other content in the
    // transform or a frame around it (`\scalebox{…}{minipage \hfill minipage}`, 2605.06364: the hoist moves the
    // insertion point, so the next panel would leave the transform), and one whose float holds another caption
    // ([`caption_hoist_is_safe`]: two captions would become one float).
    Some(c) if !allowed(c) => candidate_set
      .iter()
      .copied()
      .filter(|alt| allowed(*alt))
      .find(|alt| {
        let kept = node_tags.iter().take_while(|t| holds(*alt, **t)).count();
        let tail = &node_tags[kept..];
        kept > 0
          && !tail.is_empty()
          && tail
            .iter()
            .all(|t| !holds(*alt, *t) && with(*t, |q| q == "ltx:caption" || q == "ltx:toccaption"))
          && tail_home(
            &context,
            &container,
            tail,
            nodes[..kept]
              .iter()
              .all(|n| n.get_content().trim().is_empty()),
          )
          .is_some_and(|(_, home)| caption_hoist_is_safe(document, &context, &home))
      })
      .or(Some(c)),
    Some(c) => Some(c),
    None => node_tags
      .first()
      .and_then(|first| candidate_set.iter().copied().find(|c| holds(*c, *first))),
  };
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
    let home = tail_home(&context, &container, &tail_tags, kept_empty);
    if let Some((anchor, ancestor)) = home {
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
      // the lone child takes the box's attributes, and so its mark (OXIDIZED_DESIGN #715)
      if container.has_attribute(document::FIELD_BOX_ATTRIBUTE) {
        document.mark_field_box(&mut nodes[0]);
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

/// Whether an `ltx:XMath` of `mathnode` holds real math: Perl TeX_Math.pool.ltxml:197-201
/// `findnodes('ltx:XMath/ltx:*[local-name() != "XMText" and local-name() != "XMHint" and
/// not(local-name() = "XMTok" and (@role="PUNCT" or @role="PERIOD") and not(preceding-sibling::*)
/// and not(following-sibling::*))]')` is non-empty. Evaluated as one walk over the children: the
/// sibling axes, evaluated per child, made the query quadratic in the XMath's width — a math left
/// open to `\end{document}` holds the rest of the document (1907.09124: 265 s of building).
fn has_real_math(mathnode: &Node) -> bool {
  mathnode
    .get_child_elements()
    .iter()
    .filter(|xmath| document::with_node_qname(xmath, |qname| qname == "ltx:XMath"))
    .any(|xmath| {
      let children = xmath.get_child_elements();
      let alone = children.len() == 1;
      children.iter().any(|child| {
        document::with_node_qname(child, |qname| match qname {
          "ltx:XMText" | "ltx:XMHint" => false,
          "ltx:XMTok" => {
            !(alone
              && matches!(
                child.get_attribute("role").as_deref(),
                Some("PUNCT" | "PERIOD")
              ))
          },
          other => other.starts_with("ltx:"),
        })
      })
    })
}

pub fn cleanup_math(document: &mut Document, mathnode: Node) -> Result<()> {
  float_meta_out_of_math(document, &mathnode)?;
  // Cleanup ltx:Math elements; particularly if they aren't "really" math.
  // But record the oddity with class=ltx_markedasmath

  // If the Math ONLY contains XMath/XMText and XMHint, it apparently isn't math at all!?!
  // Single token PUNCTs can also be taken out of math.
  if !has_real_math(&mathnode) {
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

/// An author block's math-active `^` and `_` (catcode 12, mathcode "8000: mathstyle.sty, which breqn loads through
/// flexisym.sty:418, sets them so — :233-236 its default `mathactivechars` option, `\AtBeginDocument{\catcode`\^=12
/// …}`, and :251-258 `\mathcode`\^="8000` with the active `^` `\let` to `\sp`; latexml_contrib's breqn_sty.rs:138-163) written as the
/// superscript and subscript they are in math, where TeX reads the active character's meaning, `\sp`/`\sb` (tex.web
/// §1151): the marks of `{Mohamed Benzaghta$^{\star}$, …, David
/// López-Pérez}$^{\sharp}$` (2504.02633) are marks, as they are with `^` of catcode 7.
fn math_active_scripts(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let active = |c: &str| lookup_mathcode(c) == Some(0x8000);
  let (sup, sub) = (
    v.contains(&T_OTHER!("^")) && active("^"),
    v.contains(&T_OTHER!("_")) && active("_"),
  );
  if !(sup || sub) {
    return stuff;
  }
  let mut in_math = false;
  Tokens::new(
    v.iter()
      .map(|t| {
        if t.get_catcode() == Catcode::MATH {
          in_math = !in_math;
        }
        if in_math && sup && *t == T_OTHER!("^") {
          T_SUPER!()
        } else if in_math && sub && *t == T_OTHER!("_") {
          T_SUB!()
        } else {
          *t
        }
      })
      .collect(),
  )
}

/// An author block's `\begin{footnote}{text}…\end{footnote}` as LaTeX reads it: `\begin{footnote}` runs `\footnote`,
/// which takes the braced text after it as its argument, so what follows up to `\end{footnote}` is the author line's
/// (`Velibor \v{Z}eli\begin{footnote}{Linn\'{e} FLOW Centre, …},\end{footnote} Du\v{s}an Zorica\begin{footnote}…`,
/// 1703.02032: the comma printed after the first note's mark parts the names). Written `\footnote{text}` and the rest,
/// the note is the name's and its comma a separator; read as one unit, the environment hid it. Only a `{footnote}` whose
/// braced text and `\end{footnote}` follow. The environment's group (`\begin` is `\begingroup`, `\end` its
/// `\endgroup`, latex.ltx) is not kept: it would open in one name and close in the next. So a declaration between the
/// note and `\end{footnote}` is no longer confined to it; in an author block that span is the separator after the mark.
fn note_environments_as_commands(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let name_is = |at: usize, name: &str| -> Option<usize> {
    if v.get(at)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let close = (at + 1..v.len()).find(|&k| v[k].get_catcode() == Catcode::END)?;
    let read: String = v[at + 1..close].iter().map(ToString::to_string).collect();
    (read == name).then_some(close + 1)
  };
  if !v.iter().any(|t| *t == T_CS!("\\begin")) {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    if v[i] == T_CS!("\\begin")
      && let Some(mut j) = name_is(i + 1, "footnote")
    {
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      let text_end =
        (v.get(j).is_some_and(|t| t.get_catcode() == Catcode::BEGIN)).then(|| skip_group(v, j));
      let end = text_end.and_then(|from| {
        (from..v.len()).find_map(|k| {
          (v[k] == T_CS!("\\end"))
            .then(|| name_is(k + 1, "footnote").map(|e| (k, e)))
            .flatten()
        })
      });
      if let (Some(text_end), Some((end_at, past_end))) = (text_end, end)
        && text_end <= v.len()
        && !v[text_end..end_at].iter().any(|t| *t == T_CS!("\\begin"))
      {
        out.push(T_CS!("\\footnote"));
        out.extend_from_slice(&v[j..text_end]);
        out.extend_from_slice(&v[text_end..end_at]);
        i = past_end;
        changed = true;
        continue;
      }
    }
    out.push(v[i]);
    i += 1;
  }
  if changed { Tokens::new(out) } else { stuff }
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
/// The split proceeds by hierarchy (a separator at a wrapper's edge, or a wrapper holding only one, first moved out to
/// the line: `hoist_wrapped_separators`):
///  1. the name separators ("," and " and ", " \& ") at top level, each piece that is one wrapper holding names with
///     only marks and notes after it split into them (`names_in_wrapper`, KNOWN_PERL_ERRORS #552); if that already
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
  // (a byline's "by" before the names is no name: `\author{by T. Gisiger and M. B. Paranjape}`, hep-ph9507223,
  // hep-ph9404319)
  let line = without_leading_by(&line).unwrap_or(line);
  if separated_name_count(&line) >= 2 {
    split_author_line(line)
  } else {
    // (one name too opens with the "and" before it: jheppub's `\author[a,2]{and Fourth}`)
    vec![without_leading_and(&line).unwrap_or(line)]
  }
}

/// How many names an author line reads as ([`name_count`]) with its separators written as the split reads them
/// ([`spaced_name_separators`]): `{\bf Ann Able\break and Bob Baker}`, `Ann Able\thanks{…}and Bob Baker` are two, as
/// pdflatex prints them, where their text alone ("Ann Ableand Bob Baker") reads as one; so is `\parbox[t]{\textwidth}{
/// Philip F.~Hopkins\altaffilmark{1}, \&\ Jonathan Squire\altaffilmark{1}}` (1801.10166).
fn separated_name_count(line: &Tokens) -> usize {
  name_count(&visible_name_text(
    spaced_name_separators(line.clone()).unlist_ref(),
  ))
}

/// [`split_author_line`] before a qualifier rejoins its name: each piece with the separator written before it (none
/// for the first). An empty piece — between ", " and " and " in "A, B, and C" — names no one and is dropped, before a
/// wrapper is put back on the pieces: kept, it became an empty creator that took a copy of every affiliation given to
/// all the authors (`annotate=new`) and merged it onto B.
fn split_author_names(line: Tokens) -> Vec<(Vec<Token>, Tokens)> {
  // Name-level separators: comma, the literal word " and " ("Alice and Bob"), " \& " (`Zheng Zheng \& Jordi
  // Miralda-Escud\'e`, astro-ph0201275; 0908.0757) with its ties, and the `\and` family inside a wrapper; an "and" or
  // `\&` with no space token beside it (`\break and`, svmult's `and\\`) and a comma in a mark's math are written so
  // first ([`spaced_name_separators`]).
  let split = |tokens: Tokens| -> Vec<(Vec<Token>, Tokens)> {
    let mut pieces: Vec<(Vec<Token>, Tokens)> = Vec::new();
    // the separators of a dropped empty piece go on to the next (", and")
    let mut carried: Vec<Token> = Vec::new();
    let mut separators = vec![
      SplitDelim::Token(T_OTHER!(",")),
      literal_and(),
      literal_and_tie(),
      tie_and_literal(),
      tie_and_tie(),
      group_and(),
      literal_ampersand(),
      // (inside a wrapper, where the top level's `\and` split cannot reach: `{\bf {\large A \and B}}`, 2401.03555;
      // `\large{A \and B}`, 2402.15967)
      SplitDelim::Token(T_CS!("\\and")),
      SplitDelim::Token(T_CS!("\\And")),
      SplitDelim::Token(T_CS!("\\AND")),
    ];
    separators.extend(tied_ampersands());
    for (delimiter, piece) in split_tokens_delimited(tokens, separators) {
      carried.extend(delimiter);
      // (a doubled separator: the "and" after `\&` is the separator too, `Barrientos, \& and Omar López-Cruz`,
      // astro-ph9612009)
      let piece = if pieces.is_empty() {
        None
      } else {
        without_leading_and(&piece)
      }
      .unwrap_or(piece);
      if !piece.is_empty() {
        pieces.push((std::mem::take(&mut carried), piece));
      }
    }
    pieces
  };
  // a names line opening with "and" opens with the separator before its first name (mnras's `Bob Baker$^{2}$
  // \newauthor and Cat Cole$^{1}$`, 2401.11878); a byline's "By" before the names is none of them
  // (`By~~{\scshape Vitaly~Bergelson}~~and~~{\scshape Florian~K.~Richter}`, 2603.25922, 2604.12864)
  let line = without_leading_and(&line).unwrap_or(line);
  let line = without_leading_by(&line).unwrap_or(line);
  let line = marks_before_glued_commas(spaced_name_separators(hoist_wrapped_separators(line)));
  let top = split(line.clone());
  // a piece that is one wrapper holding names, its marks or notes after it (`\textbf{Cat Cole$^1$, Dan Dorn$^2$}\thanks{…}`,
  // 2303.16563; `\scalebox{.9}{…}`, 2412.05271; `\href{…}{A, B}`, 2312.07592): its names, each in the wrapper
  let descended: Vec<(Vec<Token>, Tokens)> = top
    .iter()
    .flat_map(|(delimiter, piece)| names_in_wrapper(delimiter, piece))
    .collect();
  if descended.len() > top.len() {
    return descended;
  }
  if top.len() > 1 {
    return top;
  }
  if let Some((cmd, inner)) = whole_line_cs_wrapper(&line) {
    // the marks after commas inside the wrapper are its names' too (`\textbf{Todd M. Tripp,\altaffilmark{2} …}`)
    let inner_split = split(marks_before_glued_commas(spaced_name_separators(inner)));
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
  // A line that is one group (`{\bf Daniel Boyanovsky, Da- Shin Lee}`, hep-th9212083; `{\bf Jianfei Ma$^{2}$,
  // \enspace Emmanuele Chersoni$^{2}$, …}`, 2608.16650; with no declarations, mnras's `\newauthor{…}` group in
  // 2409.07518, 2312.07625) is the same when what it holds reads as two or more names: each name in the group and its
  // declarations. A group holding one name together (`\author{{Smith, Jr., John}}`) reads as none.
  if let Some((declarations, inner)) = whole_line_group(&line)
    && separated_name_count(&inner) >= 2
  {
    let inner_split = split(marks_before_glued_commas(spaced_name_separators(inner)));
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

/// An author argument that is one parameterless macro, alone or with an empty group (`\author{\Authors}`,
/// 2401.13568; `\IEEEauthorblockN{\hpcaauthors{}}`, 2407.02944), read as TeX typesets it: expanded once, the macro's body
/// in its place, so its names and separators are seen; a body that is a `\newif` toggle through the branch TeX takes, any
/// other conditional body left whole (KNOWN_PERL_ERRORS #552).
fn expand_sole_author_macro(stuff: Tokens) -> Tokens {
  let content: Vec<Token> = stuff
    .unlist_ref()
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  let cs = match content.as_slice() {
    [cs] => *cs,
    [cs, open, close]
      if open.get_catcode() == Catcode::BEGIN && close.get_catcode() == Catcode::END =>
    {
      *cs
    },
    _ => return stuff,
  };
  if cs.get_catcode() != Catcode::CS || lookup_conditional(&cs).is_some() {
    return stuff;
  }
  let Ok(Some(defn)) = lookup_definition(&cs) else {
    return stuff;
  };
  // (no parameter text at all: `\def\Authors.{…}` reads a delimiter TeX would demand)
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return stuff;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return stuff;
  };
  // a body holding a conditional is read through the branch a `\newif` toggle takes (`\ifanon Anonymous\else Ann Able,
  // Bob Baker\fi`, as the PDF shows it); any other is left whole: split at its commas, the `\if` and its `\else`/`\fi`
  // landed in different authors (`Error:expected:\fi`, the other branch's names leaking)
  let body = body.unlist_ref();
  let read = if body.iter().any(|t| lookup_conditional(t).is_some()) {
    match toggled_branch(body) {
      Some(branch) => branch,
      None => return stuff,
    }
  } else {
    body.to_vec()
  };
  if read.iter().any(|t| t.get_catcode() == Catcode::LETTER) {
    Tokens::new(read)
  } else {
    stuff
  }
}

/// An author line's parameterless macros that print only a name separator — `\newcommand{\authcomma}{\textmd{,}
/// \hskip0.5em}` (2403.07809), tibop-article.cls:41's `\lastand` = `, and ` (2401.13365) — written as that separator,
/// where the split reads it: hidden in the macro, the names around it were one author. The separator is written as the
/// macro prints it, a space where its body has spacing and no more (`AT\amp T Labs` stays "AT&T", `NSF\comma NIH`
/// "NSF,NIH"), and an empty group ending the macro's name (`\amp{}`) prints nothing.
fn expand_separator_macros(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  if !v.iter().any(|t| t.get_catcode() == Catcode::CS) {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    i += 1;
    let Some(sep) = separator_macro_text(&t) else {
      out.push(t);
      continue;
    };
    changed = true;
    if sep.lead && out.last() != Some(&T_SPACE!()) {
      out.push(T_SPACE!());
    }
    for c in sep.text.chars() {
      out.push(match c {
        ' ' => T_SPACE!(),
        '&' => T_CS!("\\&"),
        ',' => T_OTHER!(","),
        _ => T_LETTER!(&c.to_string()),
      });
    }
    if v.get(i) == Some(&T_BEGIN!()) && v.get(i + 1) == Some(&T_END!()) {
      i += 2;
    }
    if sep.trail {
      out.push(T_SPACE!());
      while v.get(i) == Some(&T_SPACE!()) {
        i += 1;
      }
    }
  }
  if changed { Tokens::new(out) } else { stuff }
}

/// The font commands that set the upright roman text a separator prints anyway: around a separator's group they
/// print nothing of their own (`\authcomma` = `\textmd{,}\hskip0.5em`, 2403.07809).
const PLAIN_FONT_COMMANDS: [&str; 8] = [
  "\\textmd",
  "\\textup",
  "\\textrm",
  "\\textnormal",
  "\\normalfont",
  "\\rm",
  "\\rmfamily",
  "\\upshape",
];

/// The index in `body` past the glue specification starting at `i` (after `\hskip` or `\kern`): a dimension — a
/// number and a unit, two letters or `fil`, `fill`, `filll` — then `plus` and `minus` dimensions (`1em plus 1fil`).
fn past_glue(body: &[Token], i: usize) -> usize {
  let letter_at = |j: usize, c: char| {
    body.get(j).is_some_and(|t| {
      t.get_catcode() == Catcode::LETTER && t.with_str(|s| s.len() == 1 && s.starts_with(c))
    })
  };
  let past_spaces = |mut j: usize| {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  let past_dimension = |j: usize| {
    let mut j = past_spaces(j);
    while body
      .get(j)
      .is_some_and(|u| u.with_str(|s| s.chars().all(|c| c.is_ascii_digit() || ".+-".contains(c))))
    {
      j += 1;
    }
    j = past_spaces(j);
    if letter_at(j, 'f') && letter_at(j + 1, 'i') && letter_at(j + 2, 'l') {
      j += 3;
      while letter_at(j, 'l') {
        j += 1;
      }
    } else {
      for _ in 0..2 {
        if body
          .get(j)
          .is_some_and(|u| u.get_catcode() == Catcode::LETTER)
        {
          j += 1;
        }
      }
    }
    j
  };
  let mut i = past_dimension(i);
  for keyword in ["plus", "minus"] {
    let j = past_spaces(i);
    if keyword
      .chars()
      .enumerate()
      .all(|(k, c)| letter_at(j + k, c))
    {
      i = past_dimension(j + keyword.len());
    }
  }
  i
}

/// What a separator macro prints: `text` (`,`, `and`, `&` or `, and`), and whether spacing comes before (`lead`) and
/// after it (`trail`) in the macro's body — written as one space.
struct SeparatorText {
  text:  &'static str,
  lead:  bool,
  trail: bool,
}

/// The separator a parameterless macro prints, read through its body past spacing (`\hskip0.5em`, `\quad`, ties), the
/// font commands around a group (`\textmd{,}`) and `\ignorespaces`, its letters in the case a `\MakeLowercase`,
/// `\MakeUppercase`, `\lowercase` or `\uppercase` gives its group (cimento's `\atque`); `None` for any other macro.
fn separator_macro_text(cs: &Token) -> Option<SeparatorText> {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return None;
  }
  // (an author-block delimiter is split on itself, whatever it prints: a class's `\renewcommand{\and}{{\normalfont
  // and}}`, ecca.cls:192, still parts its groups, as Perl splits the unexpanded `\and`, Base_Utility.pool.ltxml:682)
  let delimiter = |d: SplitDelim| matches!(d, SplitDelim::Token(t) if t == *cs);
  if author_group_splits()
    .into_iter()
    .chain(author_and_splits())
    .chain(author_affil_splits())
    .any(delimiter)
  {
    return None;
  }
  let defn = lookup_definition(cs).ok().flatten()?;
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return None;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return None;
  };
  let body = body.unlist_ref();
  let mut text = String::new();
  let (mut lead, mut spaced) = (false, false);
  // the case changes in force, each with the group depth it ends at (`\MakeLowercase{and\ \ignorespaces}`), and the
  // one a case command just asked of the group after it
  let mut cases: Vec<(usize, bool)> = Vec::new();
  let mut pending_case: Option<bool> = None;
  let mut depth = 0usize;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    i += 1;
    // (a case command's argument that is no group is its one token: `\MakeUppercase a`)
    let single_case = if t.get_catcode() == Catcode::BEGIN {
      None
    } else {
      pending_case.take()
    };
    let spacing = if t.get_catcode() == Catcode::BEGIN {
      depth += 1;
      if let Some(upper) = pending_case.take() {
        cases.push((depth, upper));
      }
      continue;
    } else if t.get_catcode() == Catcode::END {
      if cases.last().is_some_and(|(at, _)| *at == depth) {
        cases.pop();
      }
      depth = depth.saturating_sub(1);
      continue;
    } else if t == T_CS!("\\ignorespaces") {
      // (prints nothing: cimento.cls:1525 `\atque` = `{\normalfont\MakeLowercase{and\ \ignorespaces}}`, every shipped
      // copy; 0711.2584, 0801.3557, 1402.1844, 2109.08173, 2309.14694, hep-th0703055)
      continue;
    } else if let Some(upper) = t.with_str(|s| match s {
      "\\MakeLowercase" | "\\lowercase" => Some(false),
      "\\MakeUppercase" | "\\uppercase" => Some(true),
      _ => None,
    }) {
      // (the case its group's letters print in)
      pending_case = Some(upper);
      continue;
    } else if is_spacing(&t) {
      true
    } else if t == T_CS!("\\hskip") || t == T_CS!("\\kern") {
      i = past_glue(body, i);
      true
    } else if t == T_CS!("\\hspace") {
      if body.get(i) == Some(&T_OTHER!("*")) {
        i += 1;
      }
      i = skip_group(body, i);
      true
    } else {
      false
    };
    if spacing {
      if text.is_empty() {
        lead = true;
      } else {
        spaced = true;
      }
      continue;
    }
    if t.get_catcode() == Catcode::CS && t != T_CS!("\\&") {
      // an upright-roman font command around the separator's group (`\textmd{,}`, `\rm`) prints nothing of its own;
      // a visible one (`\textbf{\&}`, `\large`) is the separator's look, kept as written
      if !t.with_str(|s| PLAIN_FONT_COMMANDS.contains(&s)) {
        return None;
      }
      continue;
    }
    if !(t == T_CS!("\\&") || matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)) {
      // (an alignment `&`, a parameter, math: no printed separator)
      return None;
    }
    if std::mem::take(&mut spaced) {
      text.push(' ');
    }
    if t == T_CS!("\\&") {
      text.push('&');
    } else {
      t.with_str(
        |s| match single_case.or(cases.last().map(|(_, upper)| *upper)) {
          Some(true) => text.push_str(&s.to_uppercase()),
          Some(false) => text.push_str(&s.to_lowercase()),
          None => text.push_str(s),
        },
      );
    }
  }
  let text = match text.as_str() {
    "," => ",",
    "and" => "and",
    "&" => "&",
    ", and" => ", and",
    _ => return None,
  };
  Some(SeparatorText { text, lead, trail: spaced })
}

/// The branch of `body` = `\ifX … [\else …] \fi` whose `\ifX` means `\iftrue` or `\iffalse` (a `\newif` toggle) that TeX
/// takes, when that branch holds no conditional of its own; `None` for any other shape.
fn toggled_branch(body: &[Token]) -> Option<Vec<Token>> {
  let first = body.iter().position(|t| *t != T_SPACE!())?;
  let last = body.iter().rposition(|t| *t != T_SPACE!())?;
  let test = lookup_definition(&body[first]).ok().flatten()?;
  let truth = if Some(&test)
    == lookup_definition(&T_CS!("\\iftrue"))
      .ok()
      .flatten()
      .as_ref()
  {
    true
  } else if Some(&test)
    == lookup_definition(&T_CS!("\\iffalse"))
      .ok()
      .flatten()
      .as_ref()
  {
    false
  } else {
    return None;
  };
  if last <= first || lookup_conditional(&body[last]) != Some(ConditionalType::Fi) {
    return None;
  }
  let inner = &body[first + 1..last];
  let otherwise = inner
    .iter()
    .position(|t| lookup_conditional(t) == Some(ConditionalType::Else));
  let branch = match (truth, otherwise) {
    (true, Some(e)) => &inner[..e],
    (true, None) => inner,
    (false, Some(e)) => &inner[e + 1..],
    (false, None) => &[][..],
  };
  if branch.iter().any(|t| lookup_conditional(t).is_some()) {
    return None;
  }
  Some(branch.to_vec())
}

/// The names a piece of an author line holds inside one wrapper — a command over its last braced argument
/// (`\textbf{A, B}`, `\scalebox{0.9}{A, B}`, `\href{url}{A, B}`) or a group opening with declarations (`{\bf A, B}`) —
/// with only marks and notes after it: each name in the wrapper, the marks and notes after the last. Perl's
/// `SplitTokens` passes over groups (Base_Utility.pool.ltxml:152-165), so such a piece was one author holding them all.
/// A group with no declarations is a wrapper too (`{Benedikt Steinar Magnússon, Ragnar Sigurðsson}`, 2306.02486;
/// 2312.04684, 2501.04001): a piece reading as fewer than two names stays — a group holding one name together,
/// `{Smith, Jr., John}`, reads as none ("Smith" alone is no name) (KNOWN_PERL_ERRORS #552, #554).
fn names_in_wrapper(delimiter: &[Token], piece: &Tokens) -> Vec<(Vec<Token>, Tokens)> {
  let unchanged = || vec![(delimiter.to_vec(), piece.clone())];
  let v = piece.unlist_ref();
  // (spacing before the wrapper — `\ \textbf{…}`, `\vspace{0.2cm} \textbf{…}`: 2508.05004, 2512.22234 — stays with
  // the first name)
  let start = leading_spacing_end(v);
  if start >= v.len() {
    return unchanged();
  }
  let (open, close) = if v[start].get_catcode() == Catcode::CS {
    if is_footnote_mark(&v[start]) || is_name_annotation(&v[start]) || is_author_mark(&v[start]) {
      return unchanged();
    }
    let (mut i, mut last) = (start + 1, None);
    loop {
      match v.get(i) {
        Some(t) if *t == T_OTHER!("[") => match v[i..].iter().position(|u| *u == T_OTHER!("]")) {
          Some(k) => i += k + 1,
          None => return unchanged(),
        },
        Some(t) if t.get_catcode() == Catcode::BEGIN => {
          let end = skip_group(v, i);
          if end > v.len()
            || v
              .get(end - 1)
              .is_none_or(|u| u.get_catcode() != Catcode::END)
          {
            return unchanged();
          }
          last = Some((i, end - 1));
          i = end;
        },
        _ => break,
      }
    }
    match last {
      Some(group) => group,
      None => return unchanged(),
    }
  } else if v[start].get_catcode() == Catcode::BEGIN {
    let end = skip_group(v, start);
    if end > v.len()
      || v
        .get(end - 1)
        .is_none_or(|u| u.get_catcode() != Catcode::END)
    {
      return unchanged();
    }
    (start, end - 1)
  } else {
    return unchanged();
  };
  let tail = &v[close + 1..];
  // (spacing after it — `\textbf{…} \vspace{0.05in}`: 2404.00511, 2404.02905, 2408.13296, 2412.05271; between its notes,
  // with a paragraph break: `\emph{Wenzhuo Ouyang, Atilla Eryilmaz, and Ness B. Shroff} \vspace{-9pt}\par
  // \vspace{-5pt} \thanks{…}`, 1108.3768 — stays with the last)
  let marks = without_spacing(tail);
  // (the list's closing punctuation before them: `\uppercase{…, and Cat Cole}.$^{2}$`, 2201.07394)
  let marks = match marks.first() {
    Some(t) if *t == T_OTHER!(".") || *t == T_OTHER!(";") => &marks[1..],
    _ => &marks[..],
  };
  if !(marks.iter().all(|t| *t == T_SPACE!()) || only_marks_and_notes(marks)) {
    return unchanged();
  }
  let inner = &v[open + 1..close];
  // A box that only lays the names out (`\parbox[c]{\textwidth}{\centering A, B}`, 2504.03146; `\parbox{\textwidth}{A,
  // and B}`, mn2e 1205.5573) is no styling of theirs: each name stands without it, its paragraph layout (`\centering`)
  // left out with it, as a parbox per name a text width wide would lay them out anew. Its other opening declarations
  // style every name (`\mbox{\large A, B}`, `\parbox{…}{\centering\bfseries A, B}`): each name in a group of them.
  let layout_box = open != start && LAYOUT_BOXES.iter().any(|cs| v[start] == T_CS!(*cs));
  let declarations = if open == start || layout_box {
    opening_declarations(&Tokens::new(inner.to_vec()))
  } else {
    Vec::new()
  };
  let names = &inner[declarations.len()..];
  // (a layout box's styling: its declarations but its paragraph layout)
  let box_style: Vec<Token> = if layout_box {
    let style: Vec<Token> = declarations
      .iter()
      .copied()
      .filter(|t| !PARAGRAPH_LAYOUT.iter().any(|cs| *t == T_CS!(*cs)))
      .collect();
    if style.iter().all(|t| *t == T_SPACE!()) {
      Vec::new()
    } else {
      style
    }
  } else {
    Vec::new()
  };
  if separated_name_count(&Tokens::new(names.to_vec())) < 2 {
    return unchanged();
  }
  // a piece of marks and notes only is the name's before it, as after the wrapper (`Chia-Hsun Chuang,\thanks{…}
  // \thanks{…} and Yun Wang`, 1205.5573; `{Ann Able},~\IEEEmembership{Senior Member, IEEE},`, 2112.09769, 2603.10769)
  let pieces = notes_to_the_name_before(split_author_names(Tokens::new(names.to_vec())));
  if pieces.len() < 2
    || pieces
      .iter()
      .any(|(_, name)| visible_name_text(name.unlist_ref()).trim().is_empty())
  {
    return unchanged();
  }
  let declarations = if layout_box { Vec::new() } else { declarations };
  let count = pieces.len();
  pieces
    .into_iter()
    .enumerate()
    .map(|(k, (inner_delimiter, name))| {
      let mut wrapped = if layout_box {
        if k == 0 {
          v[..start].to_vec()
        } else {
          Vec::new()
        }
      } else if k == 0 {
        v[..=open].to_vec()
      } else {
        v[start..=open].to_vec()
      };
      if !box_style.is_empty() {
        wrapped.push(T_BEGIN!());
        wrapped.extend(box_style.iter().copied());
      }
      wrapped.extend(declarations.iter().copied());
      wrapped.extend(name.unlist());
      if !box_style.is_empty() {
        wrapped.push(T_END!());
      }
      if !layout_box {
        wrapped.push(v[close]);
      }
      if k + 1 == count {
        wrapped.extend_from_slice(tail);
      }
      (
        if k == 0 {
          delimiter.to_vec()
        } else {
          inner_delimiter
        },
        Tokens::new(wrapped),
      )
    })
    .collect()
}

/// Where a piece's leading spacing ends: spaces, ties, space commands (`\ `, `\quad`) and `\vspace`/`\hspace` with
/// their length.
fn leading_spacing_end(v: &[Token]) -> usize {
  let mut i = 0;
  while i < v.len() {
    if is_spacing(&v[i]) || PARAMETERLESS_SPACING.iter().any(|cs| v[i] == T_CS!(*cs)) {
      i += 1;
    } else if v[i] == T_CS!("\\penalty") {
      // an integer follows (tex.web §1103 `\penalty` scans an int: `\penalty10000`, `\penalty -50`)
      let mut j = i + 1;
      while v
        .get(j)
        .is_some_and(|t| *t == T_SPACE!() || *t == T_OTHER!("-") || *t == T_OTHER!("+"))
      {
        j += 1;
      }
      while v.get(j).is_some_and(|t| {
        t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit()))
      }) {
        j += 1;
      }
      i = j;
    } else if ["\\vskip", "\\hskip", "\\kern"]
      .iter()
      .any(|cs| v[i] == T_CS!(*cs))
    {
      // a glue or dimension follows (tex.web §1057, §1061: `\kern2pt`, `\vskip 1em plus 1fil`)
      i = past_glue(v, i + 1);
    } else if v[i] == T_CS!("\\rule") {
      // latex.ltx:16359 `\rule[raise]{width}{height}`, a rule box, no glyph
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("[")) {
        while j < v.len() && v[j] != T_OTHER!("]") {
          j += 1;
        }
        j += 1;
      }
      for _ in 0..2 {
        if v.get(j).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
          break;
        }
        j = skip_group(v, j).min(v.len());
      }
      i = j;
    } else if v[i] == T_CS!("\\vspace") || v[i] == T_CS!("\\hspace") || v[i] == T_CS!("\\addvspace")
    {
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("*")) {
        j += 1;
      }
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      // (only with its braced length: `\vspace\baselineskip` stops here, its length no wrapper)
      if v.get(j).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
        break;
      }
      i = skip_group(v, j).min(v.len());
    } else {
      break;
    }
  }
  i
}

/// The boxes that lay a names line out without styling it ([`names_in_wrapper`]): latex.ltx `\parbox[pos]{width}{…}`,
/// `\mbox{…}`, `\makebox[width][pos]{…}`, and the primitive `\hbox{…}`.
const LAYOUT_BOXES: [&str; 4] = ["\\parbox", "\\mbox", "\\makebox", "\\hbox"];

/// The paragraph-layout declarations a layout box's content may open with, which go with the box.
const PARAGRAPH_LAYOUT: [&str; 5] = [
  "\\centering",
  "\\raggedright",
  "\\raggedleft",
  "\\noindent",
  "\\par",
];

/// `tokens` without the spacing ([`leading_spacing_end`]) and paragraph breaks in them, which print nothing.
fn without_spacing(tokens: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(tokens.len());
  let mut i = 0;
  while i < tokens.len() {
    let past = i + leading_spacing_end(&tokens[i..]);
    if past > i {
      i = past;
    } else {
      if tokens[i] != T_CS!("\\par") {
        out.push(tokens[i]);
      }
      i += 1;
    }
  }
  out
}

/// Name-list pieces with each piece of marks and notes only ([`only_marks_and_notes`]) joined to the piece before it,
/// without the separator written between them, which separated nothing visible; and a line break at a piece's edge
/// (`{Bob Baker},~\IEEEmembership{Member, IEEE},\\ {Cat Cole}`, 2112.09769) left out, as the layout between two names.
fn notes_to_the_name_before(pieces: Vec<(Vec<Token>, Tokens)>) -> Vec<(Vec<Token>, Tokens)> {
  let mut out: Vec<(Vec<Token>, Tokens)> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let v = piece.unlist_ref();
    let mut start = 0;
    let mut end = v.len();
    loop {
      let past = separator_gap_end(&v[..end], start, false);
      if past == start {
        break;
      }
      start = past;
    }
    while end > start
      && (is_spacing(&v[end - 1])
        || SEPARATOR_LINE_BREAKS
          .iter()
          .any(|cs| v[end - 1] == T_CS!(*cs)))
    {
      end -= 1;
    }
    let trimmed = &v[start..end];
    match out.last_mut() {
      Some((_, name)) if only_marks_and_notes(trimmed) => {
        let mut joined = name.clone().unlist();
        joined.extend_from_slice(trimmed);
        *name = Tokens::new(joined);
      },
      _ => out.push((delimiter, Tokens::new(trimmed.to_vec()))),
    }
  }
  out
}

/// A name separator written at the edge of a wrapper is the line's (`\textbf{Hang Xu$^{2}$,}`, 2310.10477;
/// `\textbf{, Jinyang Guo}`, 2505.02214), and a wrapper holding only one (`\textbf{,}`, 2403.11004; `{\rm and}`,
/// 2401.03205) is that separator: the font of a comma or an "and" is not a name's (KNOWN_PERL_ERRORS #552).
fn hoist_wrapped_separators(line: Tokens) -> Tokens {
  let v = line.unlist();
  let and = [T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")];
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut in_math = false;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    // (a separator in math or in a script's argument is the formula's: `$^{\dagger, 1}$`, 2301.11477)
    if t.get_catcode() == Catcode::MATH {
      in_math = !in_math;
    }
    // (nor is a group after a command it does not wrap: a mark's argument, `\textsuperscript{\faRuler,\faBalanceScale}`,
    // 2503.01996)
    let script = t.get_catcode() == Catcode::BEGIN
      && out
        .last()
        .is_some_and(|p| matches!(p.get_catcode(), Catcode::SUPER | Catcode::SUB | Catcode::CS));
    if in_math || t.get_catcode() == Catcode::MATH || script {
      let end = if t.get_catcode() == Catcode::BEGIN {
        skip_group(&v, i).min(v.len())
      } else {
        i + 1
      };
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    }
    let command = t.get_catcode() == Catcode::CS
      && v
        .get(i + 1)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
      && !is_footnote_mark(&t)
      && !is_name_annotation(&t)
      && !is_author_mark(&t);
    let open = if command {
      i + 1
    } else if t.get_catcode() == Catcode::BEGIN {
      i
    } else {
      out.push(t);
      i += 1;
      continue;
    };
    let end = skip_group(&v, open);
    if end > v.len()
      || v
        .get(end - 1)
        .is_none_or(|u| u.get_catcode() != Catcode::END)
    {
      out.push(t);
      i += 1;
      continue;
    }
    let close = end - 1;
    let inner = &v[open + 1..close];
    let declarations = if command {
      Vec::new()
    } else {
      opening_declarations(&Tokens::new(inner.to_vec()))
    };
    if !command && declarations.iter().all(|u| *u == T_SPACE!()) {
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    }
    let body: Vec<Token> = inner[declarations.len()..].to_vec();
    // (a tie or a space command beside the separator is spacing: `\textbf{Jiaming Ji$^{\diamond}$,~}`, 2505.20214;
    // `\textbf{Yuanpei Chen$^5$, \quad}`, 2407.15815)
    let spacing = is_spacing;
    let first = body.iter().position(|u| !spacing(u));
    let last = body.iter().rposition(|u| !spacing(u));
    let (Some(first), Some(last)) = (first, last) else {
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    };
    let trimmed = &body[first..=last];
    let rewrap = |out: &mut Vec<Token>, content: &[Token]| {
      out.extend_from_slice(&v[i..=open]);
      out.extend(declarations.iter().copied());
      out.extend_from_slice(content);
      out.push(v[close]);
    };
    if trimmed == [T_OTHER!(",")] {
      out.extend([T_OTHER!(","), T_SPACE!()]);
    } else if trimmed == and {
      out.push(T_SPACE!());
      out.extend(and);
      out.push(T_SPACE!());
    } else if trimmed.len() > 1 && trimmed[trimmed.len() - 1] == T_OTHER!(",") {
      rewrap(&mut out, &trimmed[..trimmed.len() - 1]);
      out.push(T_OTHER!(","));
    } else if trimmed.len() > 1 && trimmed[0] == T_OTHER!(",") {
      out.extend([T_OTHER!(","), T_SPACE!()]);
      rewrap(&mut out, &trimmed[1..]);
    } else if trimmed.len() > 4 && trimmed[..3] == and && spacing(&trimmed[3]) {
      // (the word "and" opening it, a word of its own: `\mbox{and {Thomas~P.} Krichbaum $^{5}$}`, mdpi 2108.09054)
      let rest = trimmed[3..]
        .iter()
        .position(|u| !spacing(u))
        .map_or(trimmed.len(), |k| 3 + k);
      if is_name_qualifier(&visible_name_text(&trimmed[rest..])) {
        // (…but not before what names no one, which stays with the name before it in its wrapper's font: `Ann Able
        // \emph{and others}`, 77n r2 review)
        out.extend_from_slice(&v[i..end]);
      } else {
        out.push(T_SPACE!());
        out.extend(and);
        out.push(T_SPACE!());
        rewrap(&mut out, &trimmed[rest..]);
      }
    } else if trimmed.len() > 4
      && trimmed[trimmed.len() - 3..] == and
      && spacing(&trimmed[trimmed.len() - 4])
    {
      let name_end = trimmed[..trimmed.len() - 3]
        .iter()
        .rposition(|u| !spacing(u))
        .map_or(0, |k| k + 1);
      rewrap(&mut out, &trimmed[..name_end]);
      out.push(T_SPACE!());
      out.extend(and);
      out.push(T_SPACE!());
    } else {
      out.extend_from_slice(&v[i..end]);
    }
    i = end;
  }
  Tokens::new(out)
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

/// What an author line's top level is made of, for [`spaced_name_separators`]: the spacing or line breaks between two
/// words (a gap), the word "and" or `\&`, a brace group, a math span, a span the split keeps whole (balanced
/// parentheses, an environment), or one token.
#[derive(Clone, Copy, PartialEq)]
enum LinePart {
  Gap(usize, usize),
  And(usize),
  Ampersand(usize),
  Group(usize, usize),
  Math(usize, usize),
  Span(usize, usize),
  Token(usize),
}

/// The line breaks an author line's printed separator may stand beside, with their optional `*` and `[…]`: a break
/// before the word or after it is no name's (`Ann Able\orcidID{…} and\\ Bob Baker`, svmult; `Badr~Kaoui$^{3,4}$\break
/// and Jens~Harting`, jfm 1311.3650; `\linebreak and`, mdpi 2509.22118), nor is a control return — a line ending in `\`,
/// which latex.ltx:558 `\def\^^M{\ }` prints as a space (2004.02831, 2404.12084).
const SEPARATOR_LINE_BREAKS: [&str; 5] = ["\\\\", "\\break", "\\linebreak", "\\allowbreak", "\\\r"];

/// The index past the spacing that starts at `v[i]` (`i` when none): spaces, ties, space commands and lengths
/// ([`leading_spacing_end`]), the line breaks of [`SEPARATOR_LINE_BREAKS`], and — when `macros` — a parameterless
/// macro whose body is only spacing (jfm.cls:1025 `\newcommand\ns{\kern.5em\relax}`: `Chang Liu \ns and Dennice F.
/// Gayme`, 1912.02168).
fn separator_gap_end(v: &[Token], mut i: usize, macros: bool) -> usize {
  while i < v.len() {
    let past = i + leading_spacing_end(&v[i..]);
    if past > i {
      i = past;
      continue;
    }
    let t = v[i];
    if SEPARATOR_LINE_BREAKS.iter().any(|cs| t == T_CS!(*cs)) {
      i += 1;
      if t == T_CS!("\\\\") && v.get(i) == Some(&T_OTHER!("*")) {
        i += 1;
      }
      if (t == T_CS!("\\\\") || t == T_CS!("\\linebreak"))
        && v.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = v[i..].iter().position(|u| *u == T_OTHER!("]"))
      {
        i += close + 1;
      }
    } else if macros && prints_only_spacing(&t) {
      i += 1;
    } else {
      break;
    }
  }
  i
}

/// Whether `cs` is a parameterless macro whose body prints only spacing (`\kern.5em\relax`, `\hskip1em`, `\quad`).
fn prints_only_spacing(cs: &Token) -> bool {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return false;
  }
  let Ok(Some(defn)) = lookup_definition(cs) else {
    return false;
  };
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  let body = body.unlist_ref();
  body.iter().any(|t| {
    !matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) && *t != T_CS!("\\relax")
  }) && only_spacing(body)
}

/// Whether `v` holds only spacing ([`separator_gap_end`]), `\relax` and groups of the same (`{\kern.5em}`); a spacing
/// command's braced argument is its own (`\hspace{1em}`, `\rule{0pt}{1ex}`).
fn only_spacing(v: &[Token]) -> bool {
  let mut i = 0;
  while i < v.len() {
    let past = separator_gap_end(v, i, false);
    if past > i {
      i = past;
    } else if v[i] == T_CS!("\\relax") {
      i += 1;
    } else if v[i].get_catcode() == Catcode::BEGIN {
      let end = skip_group(v, i);
      if end > v.len()
        || end < i + 2
        || v[end - 1].get_catcode() != Catcode::END
        || !only_spacing(&v[i + 1..end - 1])
      {
        return false;
      }
      i = end;
    } else {
      return false;
    }
  }
  true
}

/// The parts of an author line's top level ([`LinePart`]).
fn line_parts(v: &[Token]) -> Vec<LinePart> {
  let and = [T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")];
  let mut parts = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    let gap = separator_gap_end(v, i, true);
    let (part, end) = if gap > i {
      (LinePart::Gap(i, gap), gap)
    } else if t.get_catcode() == Catcode::BEGIN {
      let end = skip_group(v, i).min(v.len());
      (LinePart::Group(i, end), end)
    } else if t.defined_as(&T_MATH!()) {
      let end = (i + 1..v.len())
        .find(|&j| v[j].defined_as(&T_MATH!()))
        .map_or(v.len(), |j| j + 1);
      (LinePart::Math(i, end), end)
    } else if v[i..].starts_with(&and) {
      (LinePart::And(i), i + 3)
    } else if t == T_CS!("\\&") {
      (LinePart::Ampersand(i), i + 1)
    } else if t == T_OTHER!("(")
      && paren_closes_ahead(&v[i + 1..].iter().copied().collect::<VecDeque<Token>>())
    {
      let mut level = 0i32;
      let mut end = i;
      while end < v.len() {
        if v[end] == T_OTHER!("(") {
          level += 1;
        } else if v[end] == T_OTHER!(")") {
          level -= 1;
        }
        end += 1;
        if level == 0 {
          break;
        }
      }
      (LinePart::Span(i, end), end)
    } else if t == T_CS!("\\begin")
      && let Some(span) =
        environment_closes_ahead(&v[i + 1..].iter().copied().collect::<VecDeque<Token>>())
    {
      (LinePart::Span(i, i + 1 + span), i + 1 + span)
    } else {
      (LinePart::Token(i), i + 1)
    };
    parts.push(part);
    i = end;
  }
  parts
}

/// Whether `t` is a letter of a word: a letter, or a letter command (`\L`, `\ss`, `\dj`) or accent written in it.
fn letterlike(t: &Token) -> bool {
  t.get_catcode() == Catcode::LETTER
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|s| letter_command(s).is_some() || ACCENT_COMMANDS.contains(&s)))
}

/// Whether `t` is a control word that is no letter of the word beside it: a name of two characters or more, or of one
/// letter (tex.web §354-356 — a control symbol's name is one non-letter), after which TeX skips the spaces, so the
/// command itself parts the words (`\rm`, `\break`, `\P`). A control symbol skips none: `\-` (a discretionary
/// hyphen) and `\/` (an italic correction) print inside a word (`Rol\-and Smith`, `Bertr\/and Smith` print "Roland
/// Smith", "Bertrand Smith"); a spacing one (`\,`, `\ `) is a gap of its own ([`separator_gap_end`]).
fn parts_words(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && !letterlike(t)
    && t.with_str(|s| {
      let mut name = s.strip_prefix('\\').unwrap_or(s).chars();
      match (name.next(), name.next()) {
        (Some(_), Some(_)) => true,
        (Some(c), None) => c.is_alphabetic(),
        _ => false,
      }
    })
}

/// Whether the part before a separator word ends the word before it: a gap; a control word that is no letter (`\L`,
/// `\ss`) or accent ([`parts_words`]); a note's or mark's argument (`Dan Dorn\thanks{…}and Eve Ess`, jfm 2005.00317),
/// or a group whose text does not end in a letter or a control symbol (`{\bf Ann Able}and` prints "Ann Ableand",
/// `{Rol\-}and` "Roland"); a math span holding only a superscript, a mark (`Bao-zhi Sun$^\ast$\&Shao-Ming Fei`,
/// 1707.03934); a span the split keeps whole.
fn ends_a_word(v: &[Token], parts: &[LinePart], at: usize) -> bool {
  match parts[at] {
    LinePart::Gap(..) | LinePart::Span(..) => true,
    LinePart::Token(i) => parts_words(&v[i]),
    LinePart::Group(start, end) => {
      let owner = at.checked_sub(1).and_then(|before| match parts[before] {
        LinePart::Token(k) if v[k].get_catcode() == Catcode::CS => Some(v[k]),
        _ => None,
      });
      if let Some(owner) = owner {
        if is_name_annotation(&owner) || is_footnote_mark(&owner) {
          return true;
        }
        if letterlike(&owner) {
          return false;
        }
      }
      // (the group's last token: a letter, a letter command, a control symbol printing no space, or an inner group's
      // end continue the word)
      let last = v[start + 1..end.saturating_sub(1)]
        .iter()
        .rev()
        .find(|t| **t != T_SPACE!());
      !last.is_some_and(|t| {
        letterlike(t)
          || t.get_catcode() == Catcode::END
          || (t.get_catcode() == Catcode::CS
            && !parts_words(t)
            && separator_gap_end(std::slice::from_ref(t), 0, false) == 0)
      })
    },
    LinePart::Math(start, end) => {
      v.get(start + 1) == Some(&T_SUPER!())
        && sup_operand_at(v, start + 2).is_some_and(|(_, next)| {
          let mut next = next;
          while v.get(next) == Some(&T_SPACE!()) {
            next += 1;
          }
          next + 1 == end
        })
    },
    LinePart::And(_) | LinePart::Ampersand(_) => false,
  }
}

/// Whether the part after a separator word begins the next one: a gap (`Eve Ess and\;Fay Fox`, 2502.11348), or a
/// control word that is no letter or accent ([`parts_words`]: `and\rm`, not `and\-`).
fn begins_a_word(v: &[Token], part: LinePart) -> bool {
  match part {
    LinePart::Gap(..) => true,
    LinePart::Token(i) => parts_words(&v[i]),
    _ => false,
  }
}

/// An author line's name separators as the split reads them, at its top level (the split's re-reads of a wrapper's
/// content see the content's own):
///  - the word "and" between two word boundaries, and `\&` after one, written ` and `, ` \& `, the line
///    breaks and spacing commands beside them dropped. TeX skips the spaces after a control word, so `\space and`,
///    `\break and`, `\ns and` hold no space token; nor do `\thanks{…}and`, `$^\ast$\&`, `and\;`, svmult's `and\\` —
///    each "and" the author wrote between two names, whether pdflatex prints it apart or, after a control word, glued to
///    it ("ATLAS∗and", OXIDIZED_DESIGN_DIVERGENCES #765: a reading of the author's intent) (1311.3650, 1503.04190, 1505.06278, 2005.00317, 1912.02168,
///    2509.22118, 1504.03105, 1607.04619, 2004.02831, 2404.12084, 2502.11348, astro-ph9704233, 1707.03934, 2401.17237,
///    2506.15327, 2509.11693, 2403.07875, 2512.18788, 2603.27055). Perl's only such separator is " and " between
///    spaces (Base_Utility.pool.ltxml:678), so it keeps these names one author (OXIDIZED_DESIGN_DIVERGENCES #765). A
///    letter or a letter or accent command beside the word keeps it in its name ("Rand", "Anders", `Zo\"{e}and`), and
///    "AT\&T" stays one word;
///  - a comma at the edge of a math span that holds only a superscript mark, printed between the names, is written
///    outside it, where the split sees it: `Wittor$^{1}, $Brüggen` (1610.00129), `Iacobuta$,^{5,6,7}$ Mabel`
///    (2108.04461), `Manoj$^{2},$ Chandra` (2402.11182) — a mark after the comma then goes to the name before it
///    ([`marks_before_glued_commas`]).
fn spaced_name_separators(line: Tokens) -> Tokens {
  let v = line.unlist_ref();
  let parts = line_parts(v);
  let single_space = |part: Option<&LinePart>| match part {
    Some(LinePart::Gap(a, b)) => b - a == 1 && (v[*a] == T_SPACE!() || v[*a] == T_ACTIVE!('~')),
    _ => false,
  };
  let mut out: Vec<Token> = Vec::with_capacity(v.len() + 4);
  let mut changed = false;
  // the gap a separator just written took in (dropped), and the length of the gap last written, at the end of `out`
  let mut skip_next_gap = false;
  let mut written_gap: Option<usize> = None;
  // the place and length in `out` of the gap last written when only declarations that typeset nothing follow it
  // (`\sc Ann Able \rm and \sc Bob Baker`: the gap before `\rm` is the separator's too, no name's; not before
  // `\LaTeX`, which prints a word)
  let mut gap_before_layout: Option<(usize, usize)> = None;
  for (at, part) in parts.iter().enumerate() {
    let (word, start, len) = match *part {
      LinePart::And(i) => (true, i, 3),
      LinePart::Ampersand(i) => (false, i, 1),
      LinePart::Gap(start, end) => {
        if std::mem::take(&mut skip_next_gap) {
          (written_gap, gap_before_layout) = (None, None);
        } else {
          gap_before_layout = Some((out.len(), end - start));
          out.extend_from_slice(&v[start..end]);
          written_gap = Some(end - start);
        }
        continue;
      },
      LinePart::Math(start, end) => {
        (skip_next_gap, written_gap, gap_before_layout) = (false, None, None);
        if let Some(moved) = comma_out_of_mark_math(&v[start..end]) {
          out.extend(moved);
          changed = true;
        } else {
          out.extend_from_slice(&v[start..end]);
        }
        continue;
      },
      LinePart::Group(start, end) | LinePart::Span(start, end) => {
        (skip_next_gap, written_gap, gap_before_layout) = (false, None, None);
        out.extend_from_slice(&v[start..end]);
        continue;
      },
      LinePart::Token(i) => {
        (skip_next_gap, written_gap) = (false, None);
        if !crate::frontmatter_stores::LAYOUT
          .iter()
          .any(|cs| v[i] == T_CS!(*cs))
        {
          gap_before_layout = None;
        }
        out.push(v[i]);
        continue;
      },
    };
    // ("and" needs both boundaries; `\&`, no letter, only the one before it: `$^\ast$\&Shao-Ming Fei`, but `AT\&T`)
    let before = at.checked_sub(1).is_some_and(|b| ends_a_word(v, &parts, b));
    let after = parts
      .get(at + 1)
      .is_some_and(|next| begins_a_word(v, *next));
    let separates = before && (after || !word);
    // (between two plain spaces or ties the split's own separators match as written)
    let plain = at
      .checked_sub(1)
      .is_some_and(|b| single_space(parts.get(b)))
      && single_space(parts.get(at + 1));
    if !separates || plain {
      (skip_next_gap, written_gap, gap_before_layout) = (false, None, None);
      out.extend_from_slice(&v[start..start + len]);
      continue;
    }
    changed = true;
    if let Some(gap) = written_gap.take() {
      out.truncate(out.len() - gap);
    } else if let Some((place, gap)) = gap_before_layout {
      out.drain(place..place + gap);
    }
    gap_before_layout = None;
    out.push(T_SPACE!());
    out.extend_from_slice(&v[start..start + len]);
    out.push(T_SPACE!());
    skip_next_gap = true;
  }
  if changed { Tokens::new(out) } else { line }
}

/// A math span holding only a superscript mark and a comma at its edge (`$^{1}, $`, `$,^{5}$`), with the comma written
/// outside it: `$^{1}$,`, `,$^{5}$`. `None` for any other span.
fn comma_out_of_mark_math(span: &[Token]) -> Option<Vec<Token>> {
  let (open, close) = (*span.first()?, *span.last()?);
  if span.len() < 4 || !close.defined_as(&T_MATH!()) {
    return None;
  }
  let inner: Vec<Token> = span[1..span.len() - 1]
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  let comma = T_OTHER!(",");
  let mark_only = |mark: &[Token]| {
    mark.first() == Some(&T_SUPER!())
      && sup_operand_at(mark, 1).is_some_and(|(_, next)| next == mark.len())
  };
  if inner.last() == Some(&comma) && mark_only(&inner[..inner.len() - 1]) {
    let mut out = vec![open];
    out.extend_from_slice(&inner[..inner.len() - 1]);
    out.extend([close, comma]);
    Some(out)
  } else if inner.first() == Some(&comma) && mark_only(&inner[1..]) {
    let mut out = vec![comma, open];
    out.extend_from_slice(&inner[1..]);
    out.push(close);
    Some(out)
  } else {
    None
  }
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
    } else if first == T_OTHER!("*") {
      // a run of bare asterisks, a footnote mark written after the comma (`S. F. Pereira,* Z. Y. Ou,** and …`,
      // quant-ph0003094)
      Some(
        i + tokens[i..]
          .iter()
          .take_while(|t| **t == T_OTHER!("*"))
          .count(),
      )
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
fn literal_and_tie() -> SplitDelim { word_and_between(T_SPACE!(), T_ACTIVE!('~')) }
/// "~and " and "~and~", the "and" tied to the name before it (`Yasuhiro Ohta\ddag~and Kenji Kajiwara`, nlin0101056).
fn tie_and_literal() -> SplitDelim { word_and_between(T_ACTIVE!('~'), T_SPACE!()) }
fn tie_and_tie() -> SplitDelim { word_and_between(T_ACTIVE!('~'), T_ACTIVE!('~')) }
/// `{\ and}`, an "and" given its own space in a group (`Huw Price {\ and} Ken Wharton`, 1508.01140).
fn group_and() -> SplitDelim {
  let mut tks = vec![T_BEGIN!(), T_CS!("\\ ")];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(T_END!());
  SplitDelim::Tokens(Tokens::new(tks))
}
/// The word "and" between `before` and `after`.
fn word_and_between(before: Token, after: Token) -> SplitDelim {
  let mut tks = vec![before];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(after);
  SplitDelim::Tokens(Tokens::new(tks))
}
/// " \& " between two names, its spaces kept.
fn literal_ampersand() -> SplitDelim {
  SplitDelim::Tokens(Tokens::new(vec![T_SPACE!(), T_CS!("\\&"), T_SPACE!()]))
}
/// `\&` with a tie on either side (`Kai Han~\&~Yunhe Wang`, 2601.13599), as the "and" ties are.
fn tied_ampersands() -> [SplitDelim; 3] {
  let amp = |before: Token, after: Token| {
    SplitDelim::Tokens(Tokens::new(vec![before, T_CS!("\\&"), after]))
  };
  [
    amp(T_ACTIVE!('~'), T_ACTIVE!('~')),
    amp(T_ACTIVE!('~'), T_SPACE!()),
    amp(T_SPACE!(), T_ACTIVE!('~')),
  ]
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
/// The commands that print a mark raised, its operand their argument: `\textsuperscript`, and IEEEtran's author
/// reference mark (`Ann Able\IEEEauthorrefmark{1}` over `\IEEEauthorblockA{\IEEEauthorrefmark{1}Univ A}`, IEEEtran.cls's
/// `\IEEEauthorrefmark` a superscript symbol; 2406.11437, 2104.02493). Every reader of an author block's marks counts
/// them alike: as labels, in the operands an address is placed by, and where a line splits into its affiliations
/// (`\IEEEauthorrefmark{2}Google, \IEEEauthorrefmark{3}Google DeepMind`, 2310.02368).
fn mark_commands() -> [Token; 3] {
  [
    T_CS!("\\textsuperscript"),
    T_CS!("\\IEEEauthorrefmark"),
    T_CS!("\\authorrefmark"),
  ]
}

/// The mark command `t` means, by TeX's `\ifx` (`Token::defined_as`, which follows `\let`): one of [`mark_commands`]
/// itself, or an alias a document made of it (`\let\up\textsuperscript`, 2209.14855's macros/custom.tex:23, `Ann
/// Able\up{1}`). The one test every reader of an author block's marks makes: a literal comparison took the alias for
/// name text, so the block read as unmarked and its `\up{2}Univ B` line became an author (batch 77p, HF14 M2(b)).
fn mark_command_meant(t: &Token) -> Option<Token> {
  if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
    return None;
  }
  mark_commands().into_iter().find(|mark| t.defined_as(mark))
}

/// Is `t` a mark command, or an alias of one ([`mark_command_meant`])?
fn is_mark_command(t: &Token) -> bool { mark_command_meant(t).is_some() }

/// Is `t` one of the marks an author line links its names and affiliations by: a superscript, or a mark command
/// ([`is_mark_command`])?
fn is_author_mark(t: &Token) -> bool { *t == T_SUPER!() || is_mark_command(t) }

/// The 1-based position of the first author mark in `tokens`, as [`position_of`] counts: a superscript, `\textsuperscript`
/// or an author reference mark (`\IEEEauthorrefmark`, `\authorrefmark`), or a `\let` alias of one
/// (`is_author_mark`).
pub fn author_mark_position(tokens: &Tokens) -> Option<usize> {
  tokens
    .unlist_ref()
    .iter()
    .position(is_author_mark)
    .map(|i| i + 1)
}

/// `tokens` with every alias of a mark command written as the command it means ([`mark_command_meant`]), so the
/// readers of an author or affiliation block that match a command by name (`\textsuperscript`'s symbol and ordinal
/// rewrites) see the alias as the command.
fn unalias_mark_commands(tokens: Tokens) -> Tokens {
  if !tokens.unlist_ref().iter().any(|t| {
    matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)
      && !mark_commands().contains(t)
      && is_mark_command(t)
  }) {
    return tokens;
  }
  Tokens::new(
    tokens
      .unlist()
      .into_iter()
      .map(|t| mark_command_meant(&t).unwrap_or(t))
      .collect(),
  )
}

/// How many distinct marks (by operand) `stuff` shows outside its notes. Two or more make an address a key the names'
/// marks index (aistats' marked address over a marked row, 2312.06786, 1901.02427); a single mark is a note's
/// (`$^*$equal contribution`, `Univ A$^*$` twice), and the address still pairs by column (77o r3 review: 1905.11190,
/// 2405.18281, 2011.03885, 2301.11862, 2410.15516; probes m_lead, m_star).
pub fn distinct_author_marks(stuff: &Tokens) -> usize {
  let mut seen: Vec<String> = Vec::new();
  for operand in author_mark_operands(stuff.unlist_ref()) {
    let key = Tokens::new(operand).to_string();
    if !seen.contains(&key) {
      seen.push(key);
    }
  }
  seen.len()
}

/// Does an author block mark its names (`$^{a}$`, `\textsuperscript{1}`)? A superscript inside a `\footnote` or
/// `\thanks` is the note's own text, not a mark: 0911.0568's `LPT Orsay (CNRS)~\footnote{$^a$Laboratoire …}` line
/// made its PoS author block a marked list, its affiliation and email line read as a person (63k).
pub fn has_author_marks(stuff: &Tokens) -> bool {
  let tokens = stuff.unlist_ref();
  let mut i = 0;
  while i < tokens.len() {
    let t = &tokens[i];
    if *t == T_CS!("\\footnote") || *t == T_CS!("\\thanks") {
      i += 1;
      while i < tokens.len() && tokens[i] == T_SPACE!() {
        i += 1;
      }
      if i < tokens.len() && tokens[i] == T_OTHER!("[") {
        while i < tokens.len() && tokens[i] != T_OTHER!("]") {
          i += 1;
        }
        i += 1;
      }
      if i < tokens.len() && tokens[i].code == Catcode::BEGIN {
        let mut depth = 0usize;
        while i < tokens.len() {
          match tokens[i].code {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
          i += 1;
        }
      }
    } else if is_author_mark(t) {
      return true;
    }
    i += 1;
  }
  false
}

/// The name annotations ([`NAME_ANNOTATIONS`]) printed beside the name, not taken into a note: an address, an ORCID
/// link, a membership grade, a citation mark, a superscript (a mark or text). The one classification of those commands
/// ([`is_name_note_command`]): `author_marks_end_name` reads what follows a name's mark by it, the email split of a
/// name line (`adds_to_frontmatter`) the markup of an address (77n r4 review, which found the two lists apart).
const PRINTED_IN_PLACE: [&str; 8] = [
  "\\email",
  "\\textsuperscript",
  "\\orcidlink",
  "\\orcidID",
  "\\lx@aas@checkorcid",
  "\\IEEEmembership",
  "\\lx@IEEE@membership",
  "\\cite",
];

/// A name annotation command ([`NAME_ANNOTATIONS`]) whose argument goes into a note, not printed beside the name: all
/// but [`PRINTED_IN_PLACE`] (`\thanks`, `\footnote`, `\inst`, `\altaffilmark`, `\corref`, ecai's `\institute`, …).
fn is_name_note_command(name: &str) -> bool {
  NAME_ANNOTATIONS.contains(&name) && !PRINTED_IN_PLACE.contains(&name)
}

/// Does a superscript's operand read as mark labels (`1`, `1,2`, `1, 3`, `a`, `\dagger`, `\mathrm{c}`, `*`), not as
/// text shown raised (`\textsuperscript{equal contrib.}`, `\textsuperscript{(*)}`, `$^{\rm (a)}$`,
/// `\textsuperscript{ (equal)}`)? Labels hold no word of three or more letters, and [`clean_frontmatter_labels`] gives
/// each back as it is (its font switches and braces aside): it drops parentheses and writes a space between the
/// letters of one as `_`, so a mark read from such text loses what it shows (77n r4 review probes d1, d3, d4, c4, d6).
fn reads_as_labels(operand: &[Token]) -> bool {
  let mut word = 0;
  for t in operand {
    word = if t.get_catcode() == Catcode::LETTER {
      word + 1
    } else {
      0
    };
    if word >= 3 {
      return false;
    }
  }
  let text = Tokens::new(
    operand
      .iter()
      .filter(|t| {
        !matches!(t.get_catcode(), Catcode::BEGIN | Catcode::END)
          && !(t.get_catcode() == Catcode::CS && t.with_str(|name| FONT_SWITCH.is_match(name)))
      })
      .copied()
      .collect(),
  )
  .to_string();
  let pieces: Vec<&str> = text
    .split(',')
    .map(str::trim)
    .filter(|piece| !piece.is_empty())
    .collect();
  clean_frontmatter_labels(&text, "") == pieces
}

/// Do an author's marks end what it prints (`Ann Able$^{1}$`, `Ann$^{1}$\thanks{…}`): after the first of them nothing
/// but further marks, spacing, braces and math shifts (`${}^{1}$`), and notes with their arguments — `\thanks`,
/// `\footnote`, the note commands of `is_name_note_command`, a document's note macro, and the binding's `contacts`,
/// whose argument goes to the frontmatter (authblk's `\affil` inside `\author`, 2605.03796)? Not when a line or
/// anything visible follows them: `Ann Able$^{1}$ \\ Univ X`, `Ann Able$^{a}$ (ann@x.org)`, `Ann Able$^{1}$
/// \orcidlink{…}` (77n r3 review probes m2, m9, m8). A superscript is a mark only when its operand reads as labels
/// (`reads_as_labels`): `Ann Able$^{1}$\textsuperscript{equal contrib.}`, `\textsuperscript{(*)}`, `$^{\rm (a)}$` show
/// text, which the list path would have cut to a label or dropped (77n r4 review probes d1, d3, d4, c4, d6). Public for
/// authblk's `\author` (latexml_package), which reads a single marked name as a list only then: the list path shows a
/// mark nothing answers after the name's last text, and drops one before a line break (m7).
pub fn author_marks_end_name(stuff: &Tokens, contacts: &[Token]) -> bool {
  let tokens = outside_notes(stuff.unlist_ref());
  // (the index past a mark at `i`: `$^{1}$`, `^{1}`, `\textsuperscript{1}` and the mark commands)
  let mark_end = |i: usize| -> Option<usize> {
    let t = tokens[i];
    if t == T_MATH!() && tokens.get(i + 1) == Some(&T_SUPER!()) {
      sup_operand_at(&tokens, i + 2)
        .filter(|(operand, next)| tokens.get(*next) == Some(&T_MATH!()) && reads_as_labels(operand))
        .map(|(_, next)| next + 1)
    } else if t == T_SUPER!() || is_mark_command(&t) {
      sup_operand_at(&tokens, i + 1)
        .filter(|(operand, _)| reads_as_labels(operand))
        .map(|(_, next)| next)
    } else {
      None
    }
  };
  let mut marked = false;
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    if let Some(next) = mark_end(i) {
      marked = true;
      i = next;
    } else if !marked
      || is_spacing(&t)
      || matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::MATH
      )
    {
      // (the name before its first mark; after it, spacing, braces and math shifts)
      i += 1;
    } else if t.with_str(is_name_note_command) || is_note_macro(&t) || contacts.contains(&t) {
      // (its optional argument and its arguments)
      i += 1;
      if tokens.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[i..].iter().position(|t| *t == T_OTHER!("]"))
      {
        i += close + 1;
      }
      while let Some(t) = tokens.get(i) {
        match t.get_catcode() {
          Catcode::SPACE => i += 1,
          Catcode::BEGIN => i = skip_group(&tokens, i),
          _ => break,
        }
      }
    } else {
      return false;
    }
  }
  marked
}

/// The marks an author's name shows that read as labels (`reads_as_labels`), outside its notes and not inside a word
/// (`rewrite_in_word_superscripts`: `Craig M$^{\mathrm{c}}$Rae` spells the name): what the name requests, kept as it
/// prints. For authblk's `\author` kept whole on `\lx@add@creator` because text follows its marks (`Bob
/// Baker$^{1}$\textsuperscript{equal contrib.}`, `Bob Baker$^{1}$ (bob@x.org)`), which still requests the affiliation
/// its mark picks, as a marked name read as a list does — it requested nothing, and lost `$^1$Univ X` to the author
/// before it (77n r5 review probes k1, k2, k5). The binding's `contacts` (authblk's `\affil` inside `\author`) are left
/// out with their arguments: their marks label them. Only the labels are requested (`label_pieces`): a footnote symbol
/// (`$^{*}$`, `$^{1,\dagger}$`'s dagger, the `\ast` glued to `1\ast`) is the name's own text, which a request nothing
/// answered showed a second time after it (`Bob Baker$^{*}$ (bob@x.org)` printed "Bob Baker∗ (bob@x.org)∗"; 2605.10201's
/// `$^{1\ast}$`, `$^{1\dagger}$` gained "∗,†" in its name; 77n r6 review probes s1, s3, s5, z1).
pub fn author_label_marks(stuff: &Tokens, contacts: &[Token]) -> Vec<Tokens> {
  let tokens = rewrite_in_word_superscripts(stuff.clone());
  let v = tokens.unlist_ref();
  let mut name = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    if contacts.contains(&v[i]) {
      i += 1;
      if v.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = v[i..].iter().position(|t| *t == T_OTHER!("]"))
      {
        i += close + 1;
      }
      while let Some(t) = v.get(i) {
        match t.get_catcode() {
          Catcode::SPACE => i += 1,
          Catcode::BEGIN => i = skip_group(v, i),
          _ => break,
        }
      }
    } else {
      name.push(v[i]);
      i += 1;
    }
  }
  author_mark_operands(&name)
    .into_iter()
    .filter(|operand| reads_as_labels(operand))
    .filter_map(|operand| label_pieces(&operand))
    .map(Tokens::new)
    .collect()
}

/// A mark's operand without its footnote symbols (`is_symbol_mark`), comma-separated pieces kept apart: `1,\dagger` is
/// `1`, a number or letter glued to a symbol (`1\ast`, `1{\dagger}`: `split_glued_symbol`) its label; `None` when only
/// symbols are left (`*`, `\dagger`).
fn label_pieces(operand: &[Token]) -> Option<Vec<Token>> {
  let mut pieces: Vec<&[Token]> = Vec::new();
  let (mut start, mut i) = (0, 0);
  while i < operand.len() {
    if operand[i].get_catcode() == Catcode::BEGIN {
      i = skip_group(operand, i);
      continue;
    }
    if operand[i] == T_OTHER!(",") {
      pieces.push(&operand[start..i]);
      start = i + 1;
    }
    i += 1;
  }
  pieces.push(&operand[start.min(operand.len())..]);
  let mut kept: Vec<Token> = Vec::with_capacity(operand.len());
  for piece in pieces {
    let piece = {
      let lead = piece.iter().take_while(|t| **t == T_SPACE!()).count();
      let trail = piece[lead..]
        .iter()
        .rev()
        .take_while(|t| **t == T_SPACE!())
        .count();
      &piece[lead..piece.len() - trail]
    };
    let unbraced = match piece {
      [open, inner @ .., close]
        if open.get_catcode() == Catcode::BEGIN
          && close.get_catcode() == Catcode::END
          && skip_group(piece, 0) == piece.len() =>
      {
        inner
      },
      _ => piece,
    };
    let label = if unbraced.is_empty() || is_symbol_mark(unbraced) {
      continue;
    } else if let Some((label, _)) = split_glued_symbol(piece) {
      label
    } else {
      piece.to_vec()
    };
    if !kept.is_empty() {
      kept.push(T_OTHER!(","));
    }
    kept.extend(label);
  }
  (!kept.is_empty()).then_some(kept)
}

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
/// matched only as a balanced OTHER-catcode bracket group following the `\\` and
/// the spaces after it, which `\@ifnextchar` skips (`\\ [0.5em]`, 2604.20841), so
/// ordinary bracketed content elsewhere in a name/affiliation is untouched.
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
      // (the spaces before the optional argument skipped, as `\@ifnextchar` skips them: latex.ltx's `\@xnewline`,
      // `\@xarraycr`; `\\ [0.5em]`, 2604.20841, the r7 re-review's finding 8. Not before a star: `\\ * Equal
      // contribution` stays a legend line, though `\@ifstar` skips them too and pdflatex prints no star there —
      // 2608.23637, a SYNC_STATUS residual)
      let after_spaces = |from: usize| {
        from
          + src[from.min(n)..]
            .iter()
            .take_while(|t| **t == T_SPACE!())
            .count()
      };
      if j < n && src[j] == T_OTHER!("*") {
        j += 1;
        consumed = true;
      }
      if after_spaces(j) < n && src[after_spaces(j)] == T_OTHER!("[") {
        j = after_spaces(j);
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
///
/// A run of two or more spacing commands ([`is_spacing_command`]) between two names at the block's top level is such a
/// separator too (user ruling 2026-10-19, batch 77p): `\textbf{Ann Able}\textsuperscript{1}\;\;\textbf{Bob Baker}`
/// (2610.20409), `Ann Able$^{1}$ \ \ \ \ Bob Baker` (2310.10873), ACL's `Shanshan Wang$^1$~~~~ Derek F. Wong$^1$`
/// (2603.20851; `~~`, 2605.30274), 1702.05821, 1508.00093, 2009.12421 (OXIDIZED_DESIGN #52(o)). One spacing command
/// stays inside a name (`J.~Smith`, `Ann\ Able`, `van~der~Berg`); see [`spacing_run_separator`] for what counts as the
/// names on either side. A run or `\quad` inside a group parts nothing here: neurips_2024's 2406.09417 (one tie between
/// spaces) and 2603.20775 (`{\large \bf A \quad B}`), named with the ruling, are that shape.
fn normalize_hspace_separators(tokens: Tokens) -> Tokens {
  // (marked as the branch choice reads it, a footnote-symbol mark no mark: `\textbf{Vaibhav Singh}$^{*}$`, 2603.09835)
  let marked_block = has_author_marks(&rewrite_symbol_superscripts(tokens.clone()));
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  // (the group depth and math mode at `src[i]`: a run is a separator only at the top level, outside math; whether a
  // name letter — outside math, outside a mark command's argument, at the top level or inside a group that shows its
  // letters as name text ([`shows_name_text`]) — came since the last top-level separator, so a run after a
  // line-leading mark is the mark's space, `$\ast$~~Department of Physics`, hep-ph9708397, and one after an icon is
  // the icon's, `\raisebox{-0.3ex}{\includegraphics[height=1.1em]{daedalus.jpg}}\;\;Daedalus Research Team`,
  // 2604.10005; the depth of the mark argument being read)
  let mut depth = 0usize;
  let mut in_math = false;
  let mut named = false;
  let mut mark_argument: Option<usize> = None;
  // (for each open group, whether its letters are name text; and the depth of an optional argument being read, whose
  // letters are not, `\includegraphics[height=0.7em]`)
  let mut shown: Vec<bool> = Vec::new();
  let mut closed_shown = true;
  let mut optional_argument = false;
  // (whether the line opens with a mark — an affiliation line, `$^1$The University of Hong Kong~~~~~~$^2$Zhejiang Lab`,
  // 2310.15160 —, where a run before the next mark parts two entries; `None` before its first token)
  let mut mark_led: Option<bool> = None;
  // (whether this is a line of names — the block's first, or the first after an `\and`; a line break ends it — and
  // where in `out` the piece since the last top-level separator starts)
  let mut names_line = true;
  let mut piece_start = 0usize;
  // (in an unmarked block, the line break that began this line, where in `out` it stands, after a line of names: a
  // run that parts two names on this line makes it a row of names going on with the one before, the break a `\quad`
  // — `\textbf{Pratham Grover}\\ \textbf{Ramakrishna B.~Bairi}\ \ \textbf{Ritabrata Maiti}`, 2603.09835, where
  // "Ramakrishna B. Bairi" had read as an address line of Pratham Grover; a marked block reads its rows by their marks)
  let mut row_break: Option<usize> = None;
  // (where in `out` the names line starts, and where in `src` the line after it does: a row of names goes on with the
  // names line only when it lists another number of names than it, or a line that is no row of as many entries
  // follows it — [`row_of_names`])
  let mut names_start = 0usize;
  let mut row_start = 0usize;
  let mut i = 0;
  while i < n {
    let t = src[i];
    if mark_led.is_none() && t.get_catcode() != Catcode::SPACE && t != T_CS!("\\par") {
      mark_led = Some(starts_with_mark(after_leading_markup(&src[i..])));
    }
    match t.get_catcode() {
      Catcode::BEGIN => {
        depth += 1;
        if mark_argument.is_none() && out.last().is_some_and(is_mark_command) {
          mark_argument = Some(depth);
        }
        let outer = shown.last().copied().unwrap_or(true);
        shown.push(outer && !optional_argument && shows_name_text(out.last(), closed_shown));
      },
      Catcode::END => {
        if mark_argument == Some(depth) {
          mark_argument = None;
        }
        depth = depth.saturating_sub(1);
        closed_shown = shown.pop().unwrap_or(true);
      },
      Catcode::MATH => in_math = !in_math,
      Catcode::LETTER
        if !in_math
          && mark_argument.is_none()
          && !optional_argument
          && shown.last().copied().unwrap_or(true) =>
      {
        named = true
      },
      Catcode::OTHER
        if t == T_OTHER!("[") && out.last().is_some_and(|p| p.get_catcode() == Catcode::CS) =>
      {
        optional_argument = true
      },
      Catcode::OTHER if t == T_OTHER!("]") => optional_argument = false,
      _ if depth == 0 && is_line_separator(&t) => {
        (named, mark_led) = (false, None);
        piece_start = out.len() + 1;
        if t == T_CS!("\\\\") || t == T_CS!("\\newline") {
          row_break = (names_line && !marked_block).then_some(out.len());
          row_start = i + 1;
          names_line = false;
        } else if t == T_CS!("\\and") || t == T_CS!("\\And") || t == T_CS!("\\AND") {
          (names_line, row_break) = (true, None);
          names_start = out.len() + 1;
        }
      },
      _ => {},
    }
    // (a line opens with a mark only in a marked block, a mark being what the branch choice reads as one: in a block
    // marked only with footnote symbols — no mark there, [`rewrite_symbol_superscripts`] — a symbol-led line is an
    // affiliation line read as any other line off the names line, `$^{*}$\daplablogo\ Columbia University\ \ \
    // $^{\dagger}$University at Buffalo` (2609.35807; was a creator "†University at Buffalo", 77p r4 re-review))
    let entry_mark_led = marked_block && mark_led == Some(true);
    if depth == 0
      && !in_math
      && named
      && is_spacing_command(&t)
      && let Some(run) = spacing_run_separator(&src, &out, i, RunContext {
        mark_led: entry_mark_led,
        names_line,
        piece_start,
      })
      // (in an unmarked block, off the names line, a run between two names parts them only on a row of names going on
      // with the names line — the line right after it, [`row_of_names`]: not institutions under the names, `Ann
      // Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins`, nor names under an affiliation, `Ann Able \\ Univ A
      // \\ Bob Baker~~~~Cat Cole`, which read as the affiliation lines they stand as, nor on a line a footnote symbol
      // leads, `$^{*}$Univ A~~~~$^{\dagger}$Univ B` (2609.35807); a marked block reads its lines by their marks)
      && (names_line
        || marked_block
        || row_break
          .is_some_and(|at| row_of_names(&out[names_start.min(at)..at], &src[row_start..])))
    {
      if !names_line
        && !entry_mark_led
        && let Some(at) = row_break.take()
      {
        out[at] = T_CS!("\\quad");
        names_line = true;
      }
      if run.and_word {
        out.extend([
          T_SPACE!(),
          T_LETTER!("a"),
          T_LETTER!("n"),
          T_LETTER!("d"),
          T_SPACE!(),
        ]);
      } else {
        out.push(T_CS!("\\quad"));
      }
      (named, mark_led) = (false, None);
      piece_start = out.len();
      i = run.end;
      continue;
    }
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

/// Whether the line after a line of names `names` (as rewritten so far, the rows it already goes on with included) —
/// `rest`, from just after the line break —, which a spacing run parts into names, is a row of more names going on with
/// them rather than the institutions under them, one under each name (`Ann Able~~~~Bob Baker \\ Carnegie
/// Mellon~~~~Johns Hopkins`, a column per author; `Ann Able \\ Carnegie Mellon~~~~Johns Hopkins`). The names line holds
/// two or more names, and the row lists another number of names (2603.09835, 2607.07318, 2603.07624, 2603.16856: rows
/// of five over three, two over four), or a line follows it that is no row of as many entries — the address under all
/// the rows (2605.24414's six names over six over three, then `\textit{JIUTIAN Research}`; 2608.21409's three over
/// three over its e-mail line; 2607.09029's five over five over `LiAuto Inc.`). A row of institutions under the names
/// ends the block, or has a row of as many entries under it (their e-mails). Markup does not tell the two apart:
/// 2607.07318's and 2603.16856's second rows open with `{\bf`/`\bf` under plain first rows.
///
/// A line of addresses under the row says whose they are ([`row_spelled_by_addresses`]): the row is names only when an
/// address spells one of its pieces, or the addresses are as many as the names of both rows together — not institutions
/// over the authors' addresses, `Ann Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins \\
/// \texttt{\{aa,bb\}@x.edu}`, nor three over three over one address (probes v1, v3: were creators "Carnegie Mellon",
/// "Johns Hopkins", 77p r4 re-review).
fn row_of_names(names: &[Token], rest: &[Token]) -> bool {
  let names_count = row_name_count(names);
  if names_count < 2 {
    return false;
  }
  let rest = after_line_break_options(rest);
  let mut at = rest.iter().position(is_row_break).unwrap_or(rest.len());
  let row = &rest[..at];
  let row_count = row_name_count(row);
  // (the next line that prints something, past the lines of spacing alone: `\vspace{2mm}\\`)
  let mut under: Option<&[Token]> = None;
  while rest
    .get(at)
    .is_some_and(|t| *t == T_CS!("\\\\") || *t == T_CS!("\\newline"))
  {
    let start = rest.len() - after_line_break_options(&rest[at + 1..]).len();
    let end = start
      + rest[start..]
        .iter()
        .position(is_row_break)
        .unwrap_or(rest.len() - start);
    let line = &rest[start..end];
    if spacing_end(line) > 0 {
      under = Some(line);
      break;
    }
    at = end;
  }
  if let Some(addresses) = under.and_then(|line| email_addresses(&Tokens::new(line.to_vec()))) {
    return addresses.len() == names_count + row_count
      || row_spelled_by_addresses(names, row, &addresses);
  }
  if row_count != names_count {
    return true;
  }
  under.is_some_and(|line| row_entry_count(line) != names_count)
}

/// Whether one of `addresses` spells a piece of `row` ([`spelled_owners`], the names line's pieces `names` spelled
/// against too, so an address spells the row's piece only when it spells it best).
fn row_spelled_by_addresses(names: &[Token], row: &[Token], addresses: &[String]) -> bool {
  let pieces = |tokens: &[Token]| -> Vec<NameKeys> {
    let mut keys = Vec::new();
    let mut i = 0;
    while i < tokens.len() && !is_row_break(&tokens[i]) {
      let piece = next_name_piece(&tokens[i..]);
      if piece.is_empty() {
        i += 1;
        continue;
      }
      if has_name_letter(piece) {
        keys.push(NameKeys::of(&Tokens::new(outside_notes(piece))));
      }
      i += piece.len();
    }
    keys
  };
  let mut keys = pieces(names);
  let row_from = keys.len();
  keys.extend(pieces(row));
  addresses.iter().any(|address| {
    spelled_owners(&keys, address)
      .iter()
      .any(|owner| *owner >= row_from)
  })
}

/// `tokens` after the star and the optional `[…]` argument of the line break before them, and the spaces before each
/// (`\\[0.2em]`, `\\*`).
fn after_line_break_options(tokens: &[Token]) -> &[Token] {
  let mut i = tokens
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  if tokens.get(i) == Some(&T_OTHER!("*")) {
    i += 1;
  }
  if tokens.get(i) == Some(&T_OTHER!("["))
    && let Some(close) = tokens[i..].iter().position(|t| *t == T_OTHER!("]"))
  {
    i += close + 1;
  }
  &tokens[i..]
}

/// How many entries an author line lists up to its first line break: the pieces between its separators
/// ([`next_name_piece`]) that print text (`\texttt{a@x}~~~~\texttt{b@y}` two).
fn row_entry_count(tokens: &[Token]) -> usize {
  let mut count = 0;
  let mut i = 0;
  while i < tokens.len() && !is_row_break(&tokens[i]) {
    let piece = next_name_piece(&tokens[i..]);
    if piece.is_empty() {
      i += 1;
      continue;
    }
    if !visible_name_text(piece).trim().is_empty() {
      count += 1;
    }
    i += piece.len();
  }
  count
}

/// A token that ends an author line's row of names: a line break, or the `\and` family.
fn is_row_break(t: &Token) -> bool {
  t.with_str(|s| matches!(s, "\\\\" | "\\newline" | "\\and" | "\\And" | "\\AND"))
}

/// How many names an author line lists up to its first line break ([`is_row_break`]): each piece between its
/// separators ([`next_name_piece`]) with a name letter counts the names its text lists ([`name_count`], "Ann Able and
/// Bob Baker" two), one at least.
fn row_name_count(tokens: &[Token]) -> usize {
  let mut count = 0;
  let mut i = 0;
  while i < tokens.len() && !is_row_break(&tokens[i]) {
    let piece = next_name_piece(&tokens[i..]);
    if piece.is_empty() {
      i += 1;
      continue;
    }
    if has_name_letter(piece) {
      count += name_count(visible_name_text(&outside_notes(piece)).trim()).max(1);
    }
    i += piece.len();
  }
  count
}

/// The spacing commands a run of which parts two author names (user ruling 2026-10-19): the control space `\ ` — also
/// as the control return a line ending in `\ ` leaves, TeX dropping the line's trailing space (latex.ltx's `\^^M` is
/// `\ `; cs0207008's `K.V. Hindriks \affila\ \ ⏎ W. van der Hoek`) —, the medium and thick spaces `\:`, `\>`, `\;`,
/// the tie `~` and `\enspace`.
fn is_spacing_command(t: &Token) -> bool {
  *t == T_ACTIVE!('~')
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|s| matches!(s, "\\ " | "\\\r" | "\\:" | "\\>" | "\\;" | "\\enspace")))
}

/// `tokens` after the markup a line or an entry may open with before its mark: spaces, group openings and the font and
/// size commands of [`NAME_TEXT_COMMANDS`] (`\normalsize{$^1$I3A - Universidad de Zaragoza} ~~~~~~
/// \normalsize{$^2$École Polytechnique Fédérale de Lausanne}`, 2308.10525).
fn after_leading_markup(tokens: &[Token]) -> &[Token] {
  let lead = tokens
    .iter()
    .take_while(|t| {
      matches!(t.get_catcode(), Catcode::SPACE | Catcode::BEGIN)
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s)))
    })
    .count();
  &tokens[lead..]
}

/// Whether `tokens` open with an author mark: `$^…$` or `\(^…\)` (a math shift before a superscript), a superscript, a
/// mark command ([`is_author_mark`]).
fn starts_with_mark(tokens: &[Token]) -> bool {
  match tokens {
    [t, ..] if is_author_mark(t) => true,
    // (`$^1$`, and `\(^{1}\)`: 2604.12896's `\(^{1}\)Huawei Technologies, Canada~~\(^2\)University of Alberta`)
    [open, sup, ..] => {
      (open.get_catcode() == Catcode::MATH || *open == T_CS!("\\(")) && *sup == T_SUPER!()
    },
    _ => false,
  }
}

/// A token that ends an author line or a name, before any spacing run reads it so: the `\and` family, `\quad`,
/// `\qquad`, a line break, the horizontal spaces [`normalize_hspace_separators`] makes `\quad`. Not `\par`: an author
/// block is set in a `c` column (latex.ltx's `\@maketitle`, `\begin{tabular}[t]{c}\@author\end{tabular}`; ACL's the
/// same), restricted horizontal mode, where a blank line's `\par` ends no paragraph and prints nothing (tex.web §1094
/// `hmode+par_end` calls `end_graf`, §1096 acts only when `mode=hmode`): 2009.12421's `Ehsan
/// Shareghi$^\spadesuit$$^\clubsuit$ <blank line> \ \ Nigel Collier$^\clubsuit$` prints the two names on one row.
fn is_line_separator(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && (author_affil_splits()
      .iter()
      .chain(author_and_splits().iter())
      .chain(author_line_breaks().iter())
      .any(|d| matches!(d, SplitDelim::Token(d) if d == t))
      || t.with_str(|s| matches!(s, "\\qquad" | "\\hfill" | "\\hfil" | "\\hspace")))
}

/// Whether a group opened after `before` shows its letters as name text, for a spacing run to part names after them: a
/// group of its own (`{\scshape Vitaly~Bergelson}`, also after a separator or a spacing command: `~~ {Kyungtae Lim}`
/// once the run before it is `\quad`, 2605.30545) or a font command's argument (`\textbf{Ann Able}`); not another
/// command's argument (`\raisebox{-0.3ex}{\includegraphics[…]{daedalus.jpg}}`, 2604.10005; `\mbox{\orcidlink{…}}`,
/// 2605.06250; `\url{…}`, 2605.28131). A group right after another (`}{`) is shown as that one was (`closed_shown`):
/// the next argument of the same command.
fn shows_name_text(before: Option<&Token>, closed_shown: bool) -> bool {
  match before {
    Some(t) if t.get_catcode() == Catcode::CS => {
      t.with_str(|s| {
        NAME_TEXT_COMMANDS.contains(&s) || matches!(s, "\\par" | "\\relax" | "\\noindent")
      }) || is_line_separator(t)
        || is_spacing_command(t)
    },
    Some(t) if *t == T_END!() || *t == T_OTHER!("]") => closed_shown,
    _ => true,
  }
}

/// The font and size commands an author line styles a name or an entry with: their argument, or the group after the
/// declaration, prints as the name it styles ([`shows_name_text`]), and an entry may open with them before its mark
/// ([`after_leading_markup`]).
const NAME_TEXT_COMMANDS: &[&str] = &[
  "\\footnotesize",
  "\\scriptsize",
  "\\tiny",
  "\\LARGE",
  "\\huge",
  "\\Huge",
  "\\rm",
  "\\sf",
  "\\tt",
  "\\em",
  "\\sl",
  "\\upshape",
  "\\itshape",
  "\\slshape",
  "\\scshape",
  "\\bfseries",
  "\\mdseries",
  "\\rmfamily",
  "\\sffamily",
  "\\ttfamily",
  "\\normalfont",
  "\\textbf",
  "\\textit",
  "\\textsc",
  "\\textsl",
  "\\textup",
  "\\textmd",
  "\\textrm",
  "\\textsf",
  "\\textnormal",
  "\\emph",
  "\\bf",
  "\\it",
  "\\sc",
  "\\large",
  "\\Large",
  "\\normalsize",
  "\\small",
];

/// What a spacing run's place on its author block tells [`spacing_run_separator`].
struct RunContext {
  /// the line of a marked block opens with a mark (an affiliation line, `$^1$Univ A~~~~$^2$Univ B`, 2310.15160)
  mark_led:    bool,
  /// the line is a line of names: the block's first, or the first after an `\and` (a line break ends it)
  names_line:  bool,
  /// where in the tokens before the run the piece since the last top-level separator starts
  piece_start: usize,
}

/// A spacing run that parts two names ([`spacing_run_separator`]): where it ends, and whether the word "and" stands in
/// it (`~~AND~~`, `~~{and}~~`), which then is the separator, written as the plain word between spaces that every author
/// split reads (`Cat Cole~~\textrm{and}~~Dan Dorn` → `Cat Cole and Dan Dorn`; a marked block's line welds a piece a
/// `\quad` parts off into the entry before it).
struct RunSeparator {
  end:      usize,
  and_word: bool,
}

/// Where a run of spacing commands that starts at `src[start]` ends, when it parts two names: two or more spacing
/// commands ([`is_spacing_command`], spaces and a blank line's `\par` between them), after a name — `out` holds what
/// precedes it, a name letter is on its line (the caller's test), and it does not end with a separator already (`\\`,
/// `\and`, `\quad`, a comma or `;`), nor with an initial (`R.~~Beyer`, 2603.17646; `L.~~Caballero-Ontanaya`,
/// 2604.25437; `Rafa\l ~R.\ ~Suszek`, 2603.20914: one person) — and before the next one, which starts with a letter, a
/// group or a command that is no name annotation (a run before `\thanks{…}`, `\textsuperscript{1}` or a `$^1$` mark
/// leaves the mark on its name) and has a name letter before the next separator (not `~~\JMorcid,`, 2605.12398;
/// `~~\mbox{\orcidlink{…}}`, 2605.06250); or, on a line that opens with a mark (`ctx.mark_led`), with the next mark.
/// Off the names line (`ctx.names_line`: after a line break, before the next `\and`) both sides must read as names, not
/// as an address, an e-mail or a link (`Ann Able \\ Department of Physics~~~~University of X`; `\url{…}~~~ \url{…}`,
/// 2605.28131). A run before the word "and" — in any case, alone or in one wrapper ([`and_word_end`]: `~~AND~~`,
/// `~~{and}~~`, `\;\;\textnormal{AND}\;\;`, 2606.27261, 2607.03677) — parts with it, the word the separator
/// ([`RunSeparator`]). `None` for a single spacing command, inside one name, and for a run after the word "and", or
/// after a byline's "By" (`By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922, 2604.12864, 2603.27258, 2604.07186),
/// which separate the names themselves.
fn spacing_run_separator(
  src: &[Token],
  out: &[Token],
  start: usize,
  ctx: RunContext,
) -> Option<RunSeparator> {
  let mut end = start;
  let mut commands = 0;
  while let Some(t) = src.get(end) {
    if is_spacing_command(t) {
      commands += 1;
    } else if t.get_catcode() != Catcode::SPACE && *t != T_CS!("\\par") {
      break;
    }
    end += 1;
  }
  if commands < 2 {
    return None;
  }
  let unprinted = |t: &&Token| t.get_catcode() == Catcode::SPACE || **t == T_CS!("\\par");
  let mut before_run = out.iter().rev().skip_while(unprinted);
  let before = *before_run.next()?;
  if is_line_separator(&before)
    || before == T_CS!("\\&")
    || matches!(before.get_catcode(), Catcode::OTHER if before.with_str(|s| s == "," || s == ";"))
  {
    return None;
  }
  // (an initial: one letter and its period, after no letter)
  if before == T_OTHER!(".")
    && before_run
      .next()
      .is_some_and(|t| t.get_catcode() == Catcode::LETTER)
    && before_run
      .next()
      .is_none_or(|t| t.get_catcode() != Catcode::LETTER)
  {
    return None;
  }
  // (a run after the word "and" — in any case, alone or in one wrapper — leaves the separating to it: `Cat
  // Cole and~~Dan Dorn`; so does one after a byline's "by")
  let word_before: String = word_letters(out.iter().rev()).chars().rev().collect();
  if and_word_end(&out.iter().rev().copied().collect::<Vec<_>>(), true).is_some()
    || word_before.eq_ignore_ascii_case("by")
  {
    return None;
  }
  // (a run before the word "and" — `~~and~~`, `~~AND~~`, `~~{and}~~`, `\;\;\textnormal{AND}\;\;`, 2606.27261,
  // 2607.03677 — is one separator with it and the space after it: what follows must open a name as after a run)
  let (end, and_word) = match and_word_end(&src[end..], false) {
    Some(word_end) => (
      end
        + word_end
        + src[end + word_end..]
          .iter()
          .take_while(|t| {
            t.get_catcode() == Catcode::SPACE || is_spacing_command(t) || **t == T_CS!("\\par")
          })
          .count(),
      true,
    ),
    None => (end, false),
  };
  let after = src.get(end)?;
  if *after == T_CS!("\\&") {
    return None;
  }
  // (on a mark-led line, a run before the next mark parts its entries: `$^1$Univ A~~~~$^2$Univ B`; the word "and" in
  // it too, off the names line as the `\quad` that parts affiliation entries, `$^1$Univ A~~and~~$^2$Univ B` (probe
  // w10, was "Univ A and" an entry))
  if ctx.mark_led && starts_with_mark(after_leading_markup(&src[end..])) {
    return Some(RunSeparator {
      end,
      and_word: and_word && ctx.names_line,
    });
  }
  let starts_name = match after.get_catcode() {
    Catcode::LETTER | Catcode::BEGIN => true,
    Catcode::OTHER => after.with_str(|s| s.chars().next().is_some_and(char::is_alphabetic)),
    Catcode::CS => {
      !is_line_separator(after)
        && !is_name_annotation(after)
        && !is_footnote_mark(after)
        && !is_mark_command(after)
    },
    _ => false,
  };
  if !starts_name {
    return None;
  }
  let next_piece = next_name_piece(&src[end..]);
  if !has_name_letter(next_piece) {
    return None;
  }
  if !ctx.names_line
    && !(reads_as_name_piece(&out[ctx.piece_start.min(out.len())..])
      && reads_as_name_piece(next_piece))
  {
    return None;
  }
  Some(RunSeparator { end, and_word })
}

/// Whether `tokens` print only the word "and", in any case and wrapper (`and`, `\textnormal{AND}`, `{and}`): the
/// separator between two names, never a name or an affiliation.
fn is_and_piece(tokens: &[Token]) -> bool {
  visible_name_text(tokens).trim().eq_ignore_ascii_case("and")
}

/// The letters of the word `tokens` open with, after their spaces.
fn word_letters<'a>(tokens: impl Iterator<Item = &'a Token>) -> String {
  tokens
    .skip_while(|t| t.get_catcode() == Catcode::SPACE)
    .take_while(|t| t.get_catcode() == Catcode::LETTER)
    .map(|t| t.with_str(str::to_string))
    .collect()
}

/// Where the word "and" that `tokens` open with ends (after their spaces), in any case (`and`, `And`, `AND`), written
/// alone or as the only word of one wrapper — a group, or a font command's argument (`{and}`, `\textrm{and}`,
/// `\textnormal{AND}`: 2606.27261, 2607.03677, 2608.24427). `reversed`: `tokens` run backwards (the tokens before a
/// run), the wrapper's closing brace first. `None` when they open with anything else (`Andrew`, `Anders`).
fn and_word_end(tokens: &[Token], reversed: bool) -> Option<usize> {
  let (open, close) = if reversed {
    (T_END!(), T_BEGIN!())
  } else {
    (T_BEGIN!(), T_END!())
  };
  let mut i = tokens
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  // (a font command before its argument: read forwards, it opens the wrapper; read backwards, it stands after it)
  if !reversed
    && tokens.get(i).is_some_and(|t| {
      t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s))
    })
    && tokens.get(i + 1) == Some(&open)
  {
    i += 1;
  }
  let wrapped = tokens.get(i) == Some(&open);
  if wrapped {
    i += 1;
  }
  let word_start = i
    + tokens[i..]
      .iter()
      .take_while(|t| wrapped && t.get_catcode() == Catcode::SPACE)
      .count();
  let word: String = word_letters(tokens[word_start..].iter());
  let word: String = if reversed {
    word.chars().rev().collect()
  } else {
    word
  };
  if !word.eq_ignore_ascii_case("and") {
    return None;
  }
  i = word_start + 3;
  if wrapped {
    i += tokens[i..]
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count();
    if tokens.get(i) != Some(&close) {
      return None;
    }
    i += 1;
  }
  // (a letter or a letter command after it is a longer word's: `and\o`; the closing word of a reversed read must not
  // be glued to one before it either, `Rand`)
  tokens
    .get(i)
    .is_none_or(|t| !parts_words_after(t))
    .then_some(i)
}

/// Whether `t`, after a word, goes on with it: a letter, or a letter written as a command (`\o`, `\ss`).
fn parts_words_after(t: &Token) -> bool {
  t.get_catcode() == Catcode::LETTER
    || (t.get_catcode() == Catcode::CS && t.with_str(letter_command).is_some())
}

/// The tokens from the start of `tokens` to the next top-level separator: a line separator ([`is_line_separator`]), a
/// comma or `;`, a run of two or more spacing commands, or the mark that ends a name (`Rui Wang$^{*, 2}$ <blank line>
/// \vspace{2pt} $^1$Ant Group`, 2603.19223: the name, not the lines after it).
fn next_name_piece(tokens: &[Token]) -> &[Token] {
  let mut depth = 0usize;
  let mut lettered = false;
  for (j, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => return &tokens[..j],
      Catcode::END => depth -= 1,
      Catcode::LETTER => lettered = true,
      _ if depth > 0 => {},
      _ if lettered && starts_with_mark(&tokens[j..]) => return &tokens[..j],
      _ if is_line_separator(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";") => {
        return &tokens[..j];
      },
      _ if is_spacing_command(t)
        && tokens[j + 1..]
          .iter()
          .find(|u| u.get_catcode() != Catcode::SPACE)
          .is_some_and(is_spacing_command) =>
      {
        return &tokens[..j];
      },
      _ => {},
    }
  }
  tokens
}

/// Whether `tokens` hold a letter of name text: outside math, outside a mark command's argument and an optional
/// argument, at the top level or in a group that shows its letters ([`shows_name_text`]).
fn has_name_letter(tokens: &[Token]) -> bool {
  let mut shown: Vec<bool> = Vec::new();
  let mut closed_shown = true;
  let mut in_math = false;
  let mut optional_argument = false;
  let mut previous: Option<&Token> = None;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => {
        let outer = shown.last().copied().unwrap_or(true);
        shown.push(
          outer
            && !optional_argument
            && !previous.is_some_and(is_mark_command)
            && shows_name_text(previous, closed_shown),
        );
      },
      Catcode::END => {
        closed_shown = shown.pop().unwrap_or(true);
      },
      Catcode::MATH => in_math = !in_math,
      Catcode::LETTER
        if !in_math && !optional_argument && shown.last().copied().unwrap_or(true) =>
      {
        return true;
      },
      Catcode::OTHER
        if *t == T_OTHER!("[") && previous.is_some_and(|p| p.get_catcode() == Catcode::CS) =>
      {
        optional_argument = true
      },
      Catcode::OTHER if *t == T_OTHER!("]") => optional_argument = false,
      _ => {},
    }
    previous = Some(t);
  }
  false
}

/// Whether a piece of an author line off its names line reads as a name, for a spacing run there to part two: its text
/// is names ([`name_count`]), not an affiliation ([`reads_as_affiliation`]), and it holds no address (`@`) or link
/// (`\url`, `\href`, `\texttt`).
fn reads_as_name_piece(piece: &[Token]) -> bool {
  // (a name's notes are no part of it: `Xinlei He\textsuperscript{1}\thanks{… \href{mailto:…}{…}}`, 2605.30366)
  let piece = outside_notes(piece);
  let text = visible_name_text(&piece);
  !text.contains('@')
    && !piece
      .iter()
      .any(|t| t.with_str(|s| matches!(s, "\\url" | "\\href" | "\\texttt" | "\\tt")))
    && name_count(&text) > 0
    && !reads_as_affiliation(&Tokens::new(piece))
}

/// The footnote-SYMBOL characters (`\fnsymbol`-style) and control sequences that,
/// as a superscript on an author, mark an equal-contribution / corresponding /
/// note relation — NEVER an affiliation (which is numbered). Kept deliberately to
/// pure symbols so a numeric (`1`) or lettered (`a`) affiliation mark is never
/// misread as a note. Mirrors the note-vs-affiliation split already documented in
/// `starts_with_affiliation_mark`. The ptephy binding normalizes the same symbols (latexml_contrib ptephy_cls.rs
/// `footnote_symbol`): keep the two lists in step.
fn is_footnote_symbol_operand(sym: &[Token]) -> bool {
  use FOOTNOTE_SYMBOL_CS as SYMBOL_CS;
  const SYMBOL_CHARS: &[&str] = &[
    "*", "\u{2217}", "\u{2020}", "\u{2021}", "\u{A7}", "\u{B6}", "\u{22C6}", "\u{2605}",
    "\u{2022}", "\u{25E6}", "\u{2666}",
  ];
  let mut saw_symbol = false;
  for t in sym {
    match t.get_catcode() {
      Catcode::SPACE => continue,
      Catcode::CS => {
        if !t.with_str(|s| SYMBOL_CS.contains(&s)) && !is_symbol_command(t, 1) {
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

/// The footnote-symbol commands ([`is_footnote_symbol_operand`]).
const FOOTNOTE_SYMBOL_CS: &[&str] = &[
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

/// Whether `t` is a footnote-symbol command beyond [`FOOTNOTE_SYMBOL_CS`]: one LaTeX's own footnote marks print — a
/// command in the body of `\@fnsymbol` (latex.ltx `\def\@fnsymbol#1{\ifcase#1\or \TextOrMath\textasteriskcentered *\or
/// \TextOrMath \textdagger \dagger\or … \TextOrMath \textsection \mathsection\or …}`, as the document's class may
/// redefine it) that prints a symbol, a math symbol or a primitive printing one (`\mathsection`, `\textparagraph`),
/// not the conditionals around them; or a macro without parameters whose body is only such commands
/// (`\newcommand{\eq}{\mathsection}`), `depth` macros deep. A superscript holding only such commands is a symbol mark,
/// as one holding `\dagger` is (main-session decision 2026-10-20, batch 77p r6: 2601.02680's `$^\mathsection$` and
/// `$^{\mathsection}$`, beside its `$^{\dagger}$`). Letters and digits are none, nor is a text-mode `\dag`/`\ddag`,
/// the affiliation mark it is today ([`is_symbol_mark`]); and a math symbol outside `\@fnsymbol` (a suit, `\clubsuit`)
/// stays the affiliation mark it labels its lines with (2601.05633's `$^{\clubsuit \heartsuit}$`).
fn is_symbol_command(t: &Token, depth: usize) -> bool {
  if t.get_catcode() != Catcode::CS || *t == T_CS!("\\dag") || *t == T_CS!("\\ddag") {
    return false;
  }
  // (a command that prints a symbol: a math symbol, or a primitive printing a character that is no letter or digit)
  let prints_a_symbol = |u: &Token| match lookup_definition_stored(u) {
    Ok(Some(Stored::MathPrimitive(math))) => math.nargs.unwrap_or(0) == 0,
    Ok(Some(Stored::Primitive(primitive))) => matches!(
      &primitive.replacement,
      Some(PrimitiveBody::String(text))
        if with(*text, |text| {
          !text.is_empty() && !text.chars().any(|c| c.is_alphanumeric() || c.is_whitespace())
        })
    ),
    _ => false,
  };
  let in_fnsymbol = |u: &Token| {
    matches!(lookup_definition_stored(&T_CS!("\\@fnsymbol")),
      Ok(Some(Stored::Expandable(fnsymbol)))
        if matches!(fnsymbol.get_expansion(), Some(ExpansionBody::Tokens(body))
          if body.unlist_ref().contains(u)))
  };
  if in_fnsymbol(t) && prints_a_symbol(t) {
    return true;
  }
  depth > 0
    && matches!(lookup_definition_stored(t), Ok(Some(Stored::Expandable(macro_def)))
    if macro_def
      .get_parameters()
      .is_none_or(|p| p.get_parameters().is_empty())
      && matches!(macro_def.get_expansion(), Some(ExpansionBody::Tokens(body))
        if body.unlist_ref().iter().any(|u| *u != T_SPACE!())
          && body.unlist_ref().iter().all(|u| {
            *u == T_SPACE!()
              || u.with_str(|s| FOOTNOTE_SYMBOL_CS.contains(&s))
              || is_symbol_command(u, depth - 1)
          })))
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

/// A legend-shaped `line` that is an affiliation its symbol labels: led by a symbol mark [`keepsup`] made
/// (`$^{\diamond}$ National University of Singapore`), the symbol one an author of the block carries (`carried`, the
/// authors' [`symbol_marks`]), and the rest reading as an affiliation ([`reads_as_affiliation`]): the line with that mark
/// written back as the superscript it was, which `\lx@affiliation@withsup` labels the affiliation by, so it goes to the
/// authors carrying the symbol as a numbered one goes to the authors carrying the number (2601.05633's ⋄ line under its ♡
/// line, to Yang Zhang, Zhiyong Huang and Tat-Seng Chua, not the last author; batch 77p r6). A legend
/// (`$^{*}$ Equal contribution`) or a symbol no author carries stays as it is.
fn symbol_labelled_affiliation(line: &Tokens, carried: &[String]) -> Option<Tokens> {
  let v = line.unlist_ref();
  let lead = v.len() - after_leading_markup(&v[leading_spacing_end(v)..]).len();
  if v.get(lead) != Some(&T_CS!("\\lx@frontmatter@keepsup")) {
    return None;
  }
  let shown_end = skip_group(v, lead + 1);
  if v.get(shown_end) != Some(&T_CS!("\\lx@frontmatter@symbolmark")) {
    return None;
  }
  let end = skip_group(v, shown_end + 1);
  let mark = symbol_marks(&Tokens::new(v[shown_end..end].to_vec()));
  let rest = Tokens::new(v[end..].to_vec());
  if !mark.first().is_some_and(|mark| carried.contains(mark)) || !reads_as_affiliation(&rest) {
    return None;
  }
  let mut out = v[..lead].to_vec();
  out.push(T_CS!("\\textsuperscript"));
  out.extend_from_slice(&v[lead + 1..shown_end]);
  out.extend(rest.unlist());
  Some(Tokens::new(out))
}

/// Whether `line` is a legend: a footnote symbol ([`starts_with_footnote_symbol`]), also as the visible superscript
/// [`rewrite_symbol_superscripts`] makes it (`\textsuperscript{*}\textit{Equal contribution.}`, 2509.20748), and text
/// after it.
fn is_legend_line(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let v = after_leading_markup(&v[leading_spacing_end(v)..]);
  (starts_with_footnote_symbol(&Tokens::new(v.to_vec()))
    || v.first() == Some(&T_CS!("\\lx@frontmatter@keepsup")))
    && has_name_letter(v)
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
/// A number or letter glued to such a symbol (`$^{1*}$`, `\textsuperscript{a\dagger}`) is the affiliation mark it is
/// and the symbol after it (`split_glued_symbol`): the mark requests affiliation 1, the symbol is shown, as for
/// `$^{1,*}$` (else the one request `affiliation:1*` matched no affiliation, and affiliation 1 went to no one; authblk's
/// `\author{Ann$^{1*}$}`, 77n r3; the neurips fixture's Wenqi Shao, 2507.06670). The symbol is shown as written, a text
/// `*` as for a lone `$^{*}$`, where the unsplit `$^{1,*}$` keeps its math `∗` (2210.14772, 2508.09827).
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
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((operand, next)) = read_operand(i + 1)
      && let Some((mark, sym)) = split_glued_symbol(&operand)
    {
      out.extend([t, T_BEGIN!()]);
      out.extend(mark);
      out.push(T_END!());
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
      if p < close
        && src[p] == T_SUPER!()
        && let Some((operand, next)) = read_operand(p + 1)
        && next == close
        && let Some((mark, sym)) = split_glued_symbol(&operand)
      {
        out.extend_from_slice(&src[i..=p]);
        out.push(T_BEGIN!());
        out.extend(mark);
        out.extend([T_END!(), T_MATH!()]);
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

/// A mark's operand of a number or letter glued to a footnote symbol after it (`1*`, `a\dagger`, `2\ast`; the symbol
/// braced, `1{\dagger}`, 2605.01693): the mark and the symbol ([`rewrite_symbol_superscripts`]).
fn split_glued_symbol(operand: &[Token]) -> Option<(Vec<Token>, Vec<Token>)> {
  let lead = operand
    .iter()
    .take_while(|t| {
      t.get_catcode() == Catcode::LETTER
        || (t.get_catcode() == Catcode::OTHER
          && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit())))
    })
    .count();
  let sym = &operand[lead..];
  // (a group holding the symbol alone is the symbol)
  let unbraced = match sym {
    [open, inner @ .., close]
      if open.get_catcode() == Catcode::BEGIN
        && close.get_catcode() == Catcode::END
        && skip_group(sym, 0) == sym.len() =>
    {
      inner
    },
    _ => sym,
  };
  (lead > 0 && !unbraced.is_empty() && is_symbol_mark(unbraced))
    .then(|| (operand[..lead].to_vec(), sym.to_vec()))
}

/// `\lx@frontmatter@keepsup{<sym>}` as a token list: a superscript shown as it is, requesting nothing.
fn visible_sup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = vec![T_CS!("\\lx@frontmatter@keepsup"), T_BEGIN!()];
  v.extend(sym);
  v.push(T_END!());
  v
}

/// The footnote-symbol marks [`keepsup`] wrote into `tokens` (the `\lx@frontmatter@symbolmark` operands, as text), at
/// any depth: `Ann Able$^{\dagger}$` and `$^{\dagger}$Univ A` both hold `\dagger`.
fn symbol_marks(tokens: &Tokens) -> Vec<String> {
  let v = tokens.unlist_ref();
  let mut marks = Vec::new();
  for (i, t) in v.iter().enumerate() {
    if *t == T_CS!("\\lx@frontmatter@symbolmark") && v.get(i + 1) == Some(&T_BEGIN!()) {
      let mut depth = 0usize;
      let mut operand: Vec<Token> = Vec::new();
      for &u in &v[i + 1..] {
        match u.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ => {},
        }
        if depth == 0 {
          break;
        }
        if !(depth == 1 && u.get_catcode() == Catcode::BEGIN) {
          operand.push(u);
        }
      }
      let mark = Tokens::new(operand).to_string();
      let mark = mark.trim();
      if !mark.is_empty() {
        marks.push(mark.to_string());
      }
    }
  }
  marks
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
      .is_some_and(is_digit)
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

/// Rewrite a superscript inside a word, a letter on each side of it (`Craig M$^{\mathrm{c}}$Rae`, 2505.09041;
/// `M\textsuperscript{c}Donald`; `I$^2$R`, `NLP$^2$C`), onto a visible superscript: it is the name's spelling, not an
/// affiliation mark, which follows a name — read as one, its letter was a request and left the name ("Craig MRae", the
/// "c" shown after it). Not a mark list (`$^{1,3}$`: a list of marks is never spelling) nor a mark glued to the word
/// "and" after it (`Hiroshi Ito$^{1,3}$and Tetsuya Ogata`, 2605.00471; `Bipasha Kundu$^{1}$and Cristian Linte`,
/// 2604.27101): those are the name's marks, which kept as its text lost their affiliations (77n r3 review).
fn rewrite_in_word_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let letter = |t: Option<&Token>| t.is_some_and(|t| t.get_catcode() == Catcode::LETTER);
  // (the operand a mark's: a list of marks, or the word after it "and", the separator before the next name; `Andrew`
  // is a word, 77n r4 review probe a8. Only "and" itself: `Ann Able$^{1}$And Bob Baker$^{2}$`, `AND`, stays one name,
  // which `flag_merged_creators` reports — read as a mark, its "And" merged the names silently, 77n r5 review probes
  // a1, a6)
  let marks = |operand: &[Token], after: usize| {
    operand.contains(&T_OTHER!(","))
      || (src.get(after..after + 3) == Some(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")][..])
        && !letter(src.get(after + 3)))
  };
  // (the letters before it a word, not a length's unit: `\kern2pt$^{1}$Univ A` is a spacing, then a mark; 2407.15815)
  let after_a_word = |out: &[Token]| {
    let word = out
      .iter()
      .rev()
      .take_while(|t| t.get_catcode() == Catcode::LETTER)
      .count();
    word > 0
      && out[..out.len() - word].last().is_none_or(|t| {
        !(t.get_catcode() == Catcode::OTHER
          && t.with_str(|s| {
            s.chars()
              .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
          }))
      })
  };
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    if after_a_word(&out) {
      if t == T_CS!("\\textsuperscript")
        && let Some((operand, next)) = sup_operand_at(&src, i + 1)
        && letter(src.get(next))
        && !marks(&operand, next)
      {
        out.extend(visible_sup(operand));
        i = next;
        continue;
      }
      if t == T_MATH!()
        && src.get(i + 1) == Some(&T_SUPER!())
        && let Some((operand, next)) = sup_operand_at(&src, i + 2)
        && src.get(next) == Some(&T_MATH!())
        && letter(src.get(next + 1))
        && !marks(&operand, next + 1)
      {
        let mut math = vec![T_MATH!()];
        math.extend(operand);
        math.push(T_MATH!());
        out.extend(visible_sup(math));
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
  let switch = |t: &Token| MARK_SWITCHES.iter().any(|cs| *t == T_CS!(*cs));
  let tokens: Vec<&Token> = content
    .unlist_ref()
    .iter()
    .filter(|t| **t != T_SPACE!())
    .skip_while(|t| **t == T_BEGIN!() || **t == T_CS!("\\noindent") || switch(t))
    .collect();
  let script = match tokens.as_slice() {
    [t, rest @ ..] if is_mark_command(t) || **t == T_SUPER!() => rest,
    [t, s, rest @ ..] if t.get_catcode() == Catcode::MATH && **s == T_SUPER!() => rest,
    _ => return false,
  };
  script
    .iter()
    .find(|t| ***t != T_BEGIN!() && !switch(t))
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
  split_marked_pieces(tokens, true)
}

/// An institution without the SEPARATOR that clings to its end when the affiliations are comma-joined
/// (`\textsuperscript{1}Univ A, \textsuperscript{2}Univ B`) rather than space-joined: without this the affiliation
/// contact reads "Univ A," with a stray trailing comma (html_feedback#6588, arXiv:2606.01317). Only trailing
/// whitespace + a SINGLE trailing comma (or semicolon, `$^1$Univ A; $^2$Univ B`, 2609.22690) are removed, so a comma
/// INSIDE an institution name ("Dept X, Univ Y, City") is untouched. Applied where a marked piece is followed by
/// another: between two on a line ([`split_marked_pieces`]), and at a line's end when the next line opens a new one
/// (`…Univ C,\\ \textsuperscript{4}Univ D`); before a line continuing it the comma is the institution's own, as
/// pdflatex prints it ("Université Laval,\\ pavillon …", 1507.08653).
fn trim_trailing_separator(mut toks: Vec<Token>) -> Vec<Token> {
  while toks.last() == Some(&T_SPACE!()) {
    toks.pop();
  }
  if toks.last() == Some(&T_OTHER!(",")) || toks.last() == Some(&T_OTHER!(";")) {
    toks.pop();
    while toks.last() == Some(&T_SPACE!()) {
      toks.pop();
    }
  }
  toks
}

/// [`split_before_affiliation_marks`] of a line (`line_start`) or of what follows a wrapper in one (`{…} \vspace{1mm}
/// $^3$Univ C`), where a lead that prints nothing is a separator, not the start of the first institution.
fn split_marked_pieces(tokens: Tokens, line_start: bool) -> Vec<Tokens> {
  let toks = tokens.unlist();
  let mut segments: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  // A boundary only outside every group: a cut inside `{…}` would leave the pieces unbalanced (a mark
  // list inside a font group is split by `split_wrapped_affiliation_marks`, which repeats the group).
  let mut depth = 0usize;
  for (i, t) in toks.iter().enumerate() {
    let dollar_super = *t == T_MATH!() && toks.get(i + 1).is_some_and(|n| *n == T_SUPER!());
    let is_mark_start = dollar_super || is_mark_command(t);
    // Only treat a mark as an affiliation boundary when it is preceded by
    // whitespace (institutions are space-separated). A mark glued to the
    // preceding text — e.g. a superscript INSIDE an institution name,
    // "Center for R$^2$ Studies" — is not a boundary, so the name is not
    // wrongly split (reviewer-flagged). The first mark (current empty) always
    // opens segment 0.
    // A lead that prints nothing before the first mark — declarations (`\color{blue} $^1$…`), spacing (`\small \vspace{1mm}
    // $^{1}$Univ A`, 2407.15815) — is no segment: it opens the first one.
    // (a bare space is no lead: `\textit{$^1$A, $^2$B} $^3$C` keeps its boundary before `$^3$`)
    let prints_nothing = || {
      line_start
        && segments.is_empty()
        && current.iter().any(|t| t.get_catcode() == Catcode::CS)
        && leading_unprinted_end(&current) == current.len()
    };
    if is_mark_start && depth == 0 && current.last() == Some(&T_SPACE!()) {
      if prints_nothing() {
        // …nor is the space between it and the mark the institution's
        while current.last() == Some(&T_SPACE!()) {
          current.pop();
        }
      } else {
        segments.push(Tokens::new(trim_trailing_separator(std::mem::take(
          &mut current,
        ))));
      }
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    current.push(*t);
  }
  // (the last piece keeps a comma closing it until the next line shows whether it separates: [`trim_trailing_separator`])
  while current.last() == Some(&T_SPACE!()) {
    current.pop();
  }
  if !current.is_empty() {
    segments.push(Tokens::new(current));
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
    // A bare group's opening declarations (`\small`, `\it`, `\color{blue}` with its argument, among the spacing that
    // prints nothing: `{\small\vspace{1mm}\noindent\it $^{1}$…}`, 2407.15815), repeated in each later piece.
    let opening = if cmd.is_none() {
      lead_declarations(inner.unlist_ref())
    } else {
      Vec::new()
    };
    let mut pieces = split_wrapped_affiliation_marks(inner);
    if pieces.len() > 1 {
      // (a comma closing the wrapped list separates it from what follows the wrapper: `\textit{$^1$A, $^2$B,} $^3$C`)
      if let Some(last) = pieces.last_mut() {
        *last = Tokens::new(trim_trailing_separator(std::mem::take(last).unlist()));
      }
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
      let mut after = split_marked_pieces(Tokens::new(trailing), false).into_iter();
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
  // What the line prints from its first glyph on: the spacing and declarations before it print nothing, nor do the unit
  // letters of their lengths, which read as a name before the mark (`\\ \vspace{0.1cm} $^{1}$ Univ A`: Univ A … read as
  // authors, 2407.15815; `\kern2pt$^{1}$Univ A`).
  let printed = |tokens: &[Token]| Tokens::new(tokens[leading_unprinted_end(tokens)..].to_vec());
  let line = &printed(line.unlist_ref());
  let probe = match leading_wrapper(line) {
    Some((_, inner, _)) => printed(inner.unlist_ref()),
    None => line.clone(),
  };
  match author_mark_position(&probe) {
    Some(p) => !name_precedes_marker(&probe, p),
    // No mark inside the wrapper (`{}$^1$Univ A`, `\noindent{}$^1$…`): the whole line decides, as before.
    None => author_mark_position(line).is_some_and(|p| !name_precedes_marker(line, p)),
  }
}

/// The spacing commands a line's unprinted lead may hold that take no argument (latex.ltx:9391-9393 `\smallskip` …
/// `\vspace\…amount`; `\hfill`, `\newline`, `\protect`).
const PARAMETERLESS_SPACING: [&str; 7] = [
  "\\smallskip",
  "\\medskip",
  "\\bigskip",
  "\\hfill",
  "\\hfil",
  "\\newline",
  "\\protect",
];

/// The declarations among a line's unprinted lead, its spacing left out (`\small\vspace{1mm}\noindent\it` → `\small`,
/// `\noindent`, `\it`; [`leading_unprinted_end`]; 2407.15815).
fn lead_declarations(v: &[Token]) -> Vec<Token> {
  let mut declarations = Vec::new();
  let mut i = 0;
  loop {
    let start = i;
    i += leading_spacing_end(&v[i..]);
    let found = opening_declarations(&Tokens::new(v[i..].to_vec()));
    i += found.len();
    declarations.extend(found);
    if i == start {
      // (the space after them is no institution's: `{\color{blue} $^1$A, $^2$B}`)
      while declarations.last() == Some(&T_SPACE!()) {
        declarations.pop();
      }
      return declarations;
    }
  }
}

/// Where a line's unprinted lead ends: its spacing (`leading_spacing_end`) and its declarations
/// (`opening_declarations`), as often as they interleave (`\vspace*{2mm}\noindent`, `\small\vspace{1mm}`). Public
/// for authblk's `\affil` (latexml_package), which reads whether a line opens with a mark past it.
pub fn leading_unprinted_end(v: &[Token]) -> usize {
  let mut i = 0;
  loop {
    let start = i;
    i += leading_spacing_end(&v[i..]);
    i += opening_declarations(&Tokens::new(v[i..].to_vec())).len();
    if i == start {
      return i;
    }
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
    // (spacing that takes a length or a rule's dimensions is no declaration: [`leading_spacing_end`] reads it whole)
    if [
      "\\vspace",
      "\\hspace",
      "\\addvspace",
      "\\vskip",
      "\\hskip",
      "\\kern",
      "\\penalty",
      "\\rule",
    ]
    .iter()
    .any(|cs| t == T_CS!(*cs))
    {
      break;
    }
    if t == T_SPACE!() {
      out.push(t);
      i += 1;
    } else if t.get_catcode() == Catcode::CS
      && !is_mark_command(&t)
      && (with_arguments || !braced)
      // (a command over an optional argument is content too: `{\parbox[c]{\textwidth}{…}}`, 2504.03146)
      && v.get(i + 1) != Some(&T_OTHER!("["))
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
      if is_mark_command(&v[start]) {
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
  // IEEEtran's compsoc notes after the names (`\IEEEcompsocitemizethanks{\IEEEcompsocthanksitem …}`, 2408.01902)
  "\\IEEEcompsocitemizethanks",
  // IEEEtran's grade of a name line, made a contact (ieeetran_cls.rs `name_memberships`)
  "\\lx@IEEE@membership",
  // a footnote-symbol mark, rewritten before the parse (`rewrite_symbol_superscripts`: `$^{*}$`)
  "\\lx@frontmatter@keepsup",
  "\\lx@frontmatter@symbolmark",
  // REVTeX 3's note mark on a name (`Itay Furman \cite{itay} and Ofer Biham \cite{ofer}`, cond-mat9611093;
  // `A. V. Germanenko, \cite{cont}`, cond-mat9912430): the cited bibitem is the author's note
  "\\cite",
  // a note's text written among the names (`Bob Baker \footnotetext{On leave from …} $^{1a}$`, revtex4-1 2201.11098;
  // amsppt's `D. Logachev\footnotemark \footnotetext{E-mail: …}`, 2209.04044), elsarticle's note references
  // (`Susarla Raghuram and Anil Bhardwaj \corref{cor1}`, 1201.6291), and a class's per-name note (ecai.cls:450
  // `\newcommand{\institute}[1]{\thanks{#1}}`: `Kenneth Skiba\institute{…} \and Matthias Thimm$^1$`, 2008.02735)
  "\\footnotetext",
  "\\corref",
  "\\fnref",
  "\\tnoteref",
  "\\institute",
];

/// Whether `t` is a note or mark command an author name carries: one of [`NAME_ANNOTATIONS`], or a document's own note
/// command ([`is_note_macro`]).
fn is_name_annotation(t: &Token) -> bool {
  t.with_str(|s| NAME_ANNOTATIONS.contains(&s)) || is_note_macro(t)
}

/// A macro that is a note command: it takes arguments and every one of them goes into a `\thanks` or `\footnote` in its
/// body, so what it prints beside a name is that note's mark (`\newcommand{\blackthanks}[1]{\begingroup
/// \renewcommand{\thefootnote}{…}\unskip \thanks{#1}\unskip\endgroup}`: `Yuke Peng\textsuperscript{1,}\blackthanks{Co-first
/// authors.}\ ,`, 2506.03876). A macro that prints an argument itself (`\newcommand{\auth}[2]{#1\thanks{#2}}`) is none.
fn is_note_macro(cs: &Token) -> bool {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return false;
  }
  let Ok(Some(defn)) = lookup_definition(cs) else {
    return false;
  };
  if !defn.is_expandable() || defn.get_num_args() == 0 {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  let body = body.unlist_ref();
  let mut notes = 0;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    if t == T_CS!("\\thanks") || t == T_CS!("\\footnote") {
      let mut j = i + 1;
      while body.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      if body
        .get(j)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
      {
        notes += 1;
        i = skip_group(body, j);
        continue;
      }
    }
    if t.get_catcode() == Catcode::ARG {
      return false;
    }
    i += 1;
  }
  notes > 0
}

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
    // (T1 / LY1 `\dj` đ, `\DJ` Đ, `\dh` ð, `\DH` Ð, `\th` þ, `\TH` Þ, `\ng` ŋ, `\NG` Ŋ: t1enc.def)
    "\\dj" | "\\dh" => Some('d'),
    "\\DJ" | "\\DH" => Some('D'),
    "\\th" => Some('t'),
    "\\TH" => Some('T'),
    "\\ng" => Some('n'),
    "\\NG" => Some('N'),
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
        } else if t == T_CS!("\\and") || t == T_CS!("\\And") || t == T_CS!("\\AND") {
          // (an author separator inside a wrapper reads as the word it prints: `{\bf {\large A \and B}}`, 2401.03555)
          text.push_str(" and ");
        } else if let Some(letter) = t.with_str(letter_command) {
          // a letter written as a command (`{\L}ukasz`, `\O stergaard`) is that letter, the spaces ending the
          // command's name aside
          text.push(letter);
          while tokens.get(i + 1) == Some(&T_SPACE!()) {
            i += 1;
          }
        }
        let annotation = is_name_annotation(&t);
        i += 1;
        // an accent's spaces before its letter are not text (`Plech\' a\v{c}`, cond-mat9705101; tex.web §1123
        // `\accent` skips them)
        if t.with_str(|s| ACCENT_COMMANDS.contains(&s)) {
          while tokens.get(i) == Some(&T_SPACE!()) {
            i += 1;
          }
        }
        // a control word's optional argument is no text (`\parbox[c]{\textwidth}{…}`, 2504.03146), nor is glue
        // (`and Yael Shadmi$^{d}$ \vskip 0.3in`, hep-ph0107182)
        if !annotation
          && parts_words(&t)
          && tokens.get(i) == Some(&T_OTHER!("["))
          && let Some(close) = tokens[i..].iter().position(|u| *u == T_OTHER!("]"))
        {
          i += close + 1;
        }
        if ["\\vskip", "\\hskip", "\\kern"]
          .iter()
          .any(|cs| t == T_CS!(*cs))
        {
          i = past_glue(tokens, i);
          if t == T_CS!("\\hskip") {
            text.push(' ');
          }
          continue;
        }
        // an optional `[...]` (`\footnotemark[2]` has nothing more), then a note's or mark's braced argument
        if annotation || is_footnote_mark(&t) {
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
          continue;
        }
        // another command's argument with no letter in it is an identifier or a mark, no name text
        // (`\orcidicon{0000-0001-…}`, `\authorrefmark{1, 3}`: 2104.02493, 2407.03625); a letter is one of any script
        // (`\textbf{Μαρία}`, its letters OTHER under pdfTeX's catcodes)
        if !annotation
          && tokens
            .get(i)
            .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
        {
          let end = skip_group(tokens, i).min(tokens.len());
          let lettered = tokens[i..end].iter().any(|u| match u.get_catcode() {
            Catcode::LETTER | Catcode::OTHER => u.with_str(|s| s.chars().any(char::is_alphabetic)),
            Catcode::CS => u.with_str(letter_command).is_some(),
            _ => false,
          });
          // (nor is a graphic's file name or a link's address: `\href{https://…}{\includegraphics[scale=0.06]{orcid.pdf}
          // \hspace{1mm}Hossein Bahak, Farzaneh Taheri}`, 2312.07592)
          if !lettered || t == T_CS!("\\includegraphics") || t == T_CS!("\\href") {
            i = end;
          }
        }
        continue;
      },
      Catcode::MATH => {
        // a word set in math is name text — a surname given in math to carry its mark (`M. ${\rm Schreiber}^{*}$`,
        // cond-mat9402093): a run of two or more letters, broken by anything but a command or a brace; a script and its
        // operand (`$^{a}$`, `$^\mathrm{ab}$`) and a lone letter (`$e^+e^-$`) are not
        i += 1;
        let mut word = String::new();
        let end_word = |word: &mut String, text: &mut String| {
          if word.chars().count() >= 2 {
            text.push_str(word);
          }
          word.clear();
        };
        while i < tokens.len() && tokens[i].get_catcode() != Catcode::MATH {
          match tokens[i].get_catcode() {
            Catcode::SUPER | Catcode::SUB => {
              end_word(&mut word, &mut text);
              i = skip_script_operand(tokens, i + 1);
              continue;
            },
            Catcode::LETTER => tokens[i].with_str(|s| word.push_str(s)),
            Catcode::CS | Catcode::BEGIN | Catcode::END => {},
            _ => end_word(&mut word, &mut text),
          }
          i += 1;
        }
        end_word(&mut word, &mut text);
      },
      Catcode::SUPER | Catcode::SUB => {
        i = skip_script_operand(tokens, i + 1);
        continue;
      },
      // a run of bare asterisks after a name is its mark (`S. F. Pereira,* Z. Y. Ou,** and H. J. Kimble`,
      // quant-ph0003094)
      Catcode::OTHER if t == T_OTHER!("*") => {},
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

/// The index after a script's operand at `i`: a brace group, a command with its brace group (`^\mathrm{ab}`), or one
/// token.
fn skip_script_operand(tokens: &[Token], i: usize) -> usize {
  if tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::CS)
    && tokens
      .get(i + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    skip_group(tokens, i + 1)
  } else {
    skip_group(tokens, i)
  }
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
/// `\footnotemark` or a mark manyfoot declares beside it (`\DeclareNewFootnote{AAffil}` makes `\footnotemarkAAffil`,
/// manyfoot.sty:253-257; 2309.13063): a mark with an optional `[n]` and no argument.
fn is_footnote_mark(t: &Token) -> bool { t.with_str(|s| s.starts_with("\\footnotemark")) }

/// Acronyms of institutions and societies, in capitals only: "Meta AI", "ESA" are none of them a name, though Qingyao
/// Ai and Esa Räsänen are.
const NON_NAME_ACRONYMS: &[&str] = &[
  "AI", "IEEE", "ACM", "INFN", "CNRS", "INAF", "CSIC", "NASA", "ESA", "ESO", "CERN", "DESY",
  "RIKEN", "KEK", "IPMU", "MIT", "UCLA", "SISSA", "TU",
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
  // (a list's closing `;` too: sn-jnl's `\author[1]{A.~Able, B.~Baker, C.~Cole;}`, 2504.09158)
  let stripped = text.trim_end_matches(|c: char| c.is_ascii_digit() || "*∗†‡§¶♯#,;".contains(c));
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
  // (a surname in capitals too, of three letters or more: `M.CACCIA`, hep-ex9910019; not an abbreviation's last letter,
  // `I.N.F.N`)
  let capitals_surname =
    rest.chars().count() >= 3 && rest.chars().all(|c| c.is_uppercase() || "'’-".contains(c));
  if parts.is_empty() || !(rest.chars().skip(1).any(char::is_lowercase) || capitals_surname) {
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
  // (and "et al." ending a list, `is_name_qualifier`)
  initial || NAME_SUFFIXES.contains(&format!("{word}.").as_str()) || word == "al" || word == "et.al"
}

/// The societies whose membership grade follows an author's name (`Senior Member, IEEE`).
const NAME_GRADE_SOCIETIES: &[&str] = &[
  "IEEE", "ACM", "OSA", "SPIE", "IET", "IEICE", "AAAI", "SIAM", "APS", "IAPR", "IFAC", "ASCE",
  "ASME",
];
/// The abbreviated grades glued to a society's name (`M.ASCE`, `F.ASCE`, `A.M.ASCE`: ascelike's own
/// `\author[1]{Matthew R. Kuhn, M.ASCE}`, ascexmpl.tex:48; ASCE's Student, Associate, full and Distinguished Members
/// and Fellows).
const NAME_GRADE_ABBREVIATIONS: &[&str] = &["S.M.", "A.M.", "M.", "F.", "Dist.M."];
/// Generational suffixes and academic degrees written after a name, comma-separated (`John Smith, Jr.`, `Jane Doe,
/// MD, PhD`).
const NAME_SUFFIXES: &[&str] = &[
  "Jr", "Jr.", "Sr", "Sr.", "Jun.", "Sen.", "II", "III", "IV", "PhD", "Ph.D.", "Ph.D", "MD",
  "M.D.", "MSc", "M.Sc.", "BSc", "B.Sc.", "MA", "M.A.", "MS", "M.S.", "MBA", "MPH", "MBBS",
  "DPhil", "D.Phil.", "DSc", "D.Sc.", "FRS", "RN", "Esq.", "P.E.",
];

/// Whether an author-list part qualifies the name before it instead of naming a person: a generational suffix or a
/// degree ([`NAME_SUFFIXES`]), or a professional membership grade (`Member, IEEE`, `Senior Member`, or the society
/// alone once the list is split at the grade's comma), which LaTeX prints after the name, comma-separated.
fn is_name_qualifier(text: &str) -> bool {
  let text = strip_glued_marks(text);
  if NAME_SUFFIXES.contains(&text) {
    return true;
  }
  // (the rest of a list left unnamed is no one: `Ann Able and others`, `\emph{and others}`, `Ann Able, et al.`)
  let words: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
  if ["others", "et al.", "et al", "et. al.", "et.al.", "et.al"]
    .contains(&words.to_lowercase().as_str())
  {
    return true;
  }
  // an abbreviated grade before its society, glued or spaced (`M.ASCE`, `M. ASCE`, `A. M. ASCE`)
  let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
  if NAME_GRADE_ABBREVIATIONS.iter().any(|grade| {
    compact
      .strip_prefix(grade)
      .is_some_and(|society| NAME_GRADE_SOCIETIES.contains(&society))
  }) {
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
    // (a separator alone, of a doubled one: `Barrientos, \& and Omar López-Cruz`, astro-ph9612009)
    .filter(|part| !part.is_empty() && part != "&" && part != "and")
    .collect()
}

/// How many parts of `text` are names when every part is a name or qualifies one ([`is_name_qualifier`]); 0 when a
/// part is neither (an institution, a place).
fn name_count(text: &str) -> usize {
  let parts = name_parts(text);
  if parts.iter().all(|p| name_shaped(p) || is_name_qualifier(p)) {
    // (a qualifier is no name, whatever its shape: `M.ASCE` reads as a glued initial and surname)
    parts
      .iter()
      .filter(|p| name_shaped(p) && !is_name_qualifier(p))
      .count()
  } else {
    0
  }
}

/// Whether `line` continues the author names `names` instead of starting their affiliations: styled as the names are,
/// all of it names (or their qualifiers), and either the names read as an unfinished list — ending in a comma, "and"
/// or "&" (`Joshua N.\ Winn\altaffilmark{1}, Andrew W.\ Howard\altaffilmark{2},\\ Avi Shporer\altaffilmark{1}`,
/// 1010.1318) — or the line opens with "and" (`…, \emph{Member, IEEE},\\ and Warren J. Gross, \emph{Senior Member,
/// IEEE}`, 1611.04834) — or the line is a comma list of two or more names and nothing else (`…, Wenhan Liu\\ Haonan
/// Chen, Zheng Liu, Zhicheng Dou, and Ji-Rong Wen`, 2308.07107; 2312.11514, 2406.10513). "and Max--Planck Institut für
/// Mathematik, Bonn" (math0208081), `John Smith,\\ {\it Bell Labs, Murray Hill}` and `Pierre Fernandez\\ Meta FAIR \&
/// Inria Rennes` (2402.14904: joined by "&" alone, as institutions are) stay affiliations.
pub fn names_continue(names: &Tokens, line: &Tokens) -> bool {
  leading_markup(line) == leading_markup(names) && names_list_continues(names, line)
}

/// [`names_continue`]'s text test alone, whatever markup either line opens with: in a block whose authors carry marks,
/// a row of names after names continues them however it is styled (`Ann Able$^{1}$, Bob Baker$^{2}$,\\
/// \textbf{Cat Cole}$^{\dagger}$, \textbf{Dan Dee}$^{\dagger}$`, 2307.00040; `\textbf{Cat Cole, Dan Dee,}`, 2510.11639) —
/// its words tell names from places there, as the unmarked block's styled-affiliation alternation needs the markup too
/// (hep-ph9306253, 0811.1526).
fn names_list_continues(names: &Tokens, line: &Tokens) -> bool {
  // (a separator in a wrapper of its own is the line's: `\textbf{Seth Ebner}$^{\diamondsuit}$\textrm{,}\\ \textbf{Brandon
  // Smock}`, 2512.18601)
  let names_text = visible_name_text(hoist_wrapped_separators(names.clone()).unlist_ref());
  let names_text = names_text.trim_end();
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim();
  let unfinished =
    names_text.ends_with(',') || names_text.ends_with(" and") || names_text.ends_with('&');
  let count = name_count(text);
  ((unfinished || text.starts_with("and ") || text.starts_with("& ")) && count > 0)
    || (count >= 2 && text.contains(',') && name_count(names_text) >= 2)
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

/// Whether a name carries a mark after it, outside its notes: a superscript or mark command
/// ([`is_affiliation_mark`]: IEEEtran's `\IEEEauthorrefmark`) or aastex's `\altaffilmark` after its first letter
/// (`Bob Baker$^{2}$`; not `$^{2}$Univ B`).
fn marked_after_name(name: &Tokens) -> bool {
  let outside = outside_notes(name.unlist_ref());
  let Some(first_letter) = outside
    .iter()
    .position(|t| t.get_catcode() == Catcode::LETTER)
  else {
    return false;
  };
  outside[first_letter..]
    .iter()
    .any(|t| is_affiliation_mark(t) || *t == T_CS!("\\altaffilmark"))
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
///    name) stays an affiliation;
///  - after affiliations, a line of names carrying an author's note (`\thanks`, `\footnote`) opens the next pair
///    when the block's first names carry the same note command ([`author_note_of`]; hep-lat9404004's `C.
///    Michael\thanks{…},`).
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
    // (the author's note the block's first names carry, below)
    let first_names_note = groups.first().and_then(|(first, _)| author_note_of(first));
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
      // a line of names after a finished list goes on after a comma, where the line break parted them
      let names_text = visible_name_text(names.unlist_ref());
      let names_text = names_text.trim_end();
      if !(names_text.ends_with(',')
        || names_text.ends_with(" and")
        || names_text.ends_with('&')
        || and_led.is_some())
      {
        joined.push(T_OTHER!(","));
      }
      joined.push(T_SPACE!());
      // (an "and" opening the line — inside its group too, `{A, B} \\ {and C}`, 2306.02486, 2411.12606 — is the
      // separator, written where the split reads it)
      match and_led.and_then(|_| without_leading_and(line)) {
        Some(rest) => {
          joined.extend([T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d"), T_SPACE!()]);
          joined.extend(rest.unlist());
        },
        None => joined.extend(line.clone().unlist()),
      }
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
    } else if name_count(text) > 0
      && first_names_note.is_some()
      && author_note_of(line) == first_names_note
    {
      // after affiliations, names carrying an author's note open the next pair, however they are styled, when the
      // block's first names carry the same note command — the block's shape (`A.M. Green\thanks{…},\\ Research
      // Institute …\\ C. Michael\thanks{…},\\ DAMTP …`, hep-lat9404004, 1706.02584; `\footnote` throughout, 1410.7277,
      // 1403.1404); an affiliation with a note under plain names stays one (`Ann Able and Bob Baker\\ Department of
      // Physics\\ Ohio State\footnote{Visiting.}`), and so does one with another note command than the names'
      // (`Ann Able\thanks{x}\\ Dept\\ Ohio State\footnote{Visiting}\\ Bob Baker\thanks{y}`)
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

/// The author's note command `line` carries at its top level, `\thanks` or `\footnote` (the first one), if any.
fn author_note_of(line: &Tokens) -> Option<Token> {
  let mut depth = 0i32;
  line.unlist_ref().iter().copied().find(|t| {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    depth == 0 && (*t == T_CS!("\\thanks") || *t == T_CS!("\\footnote"))
  })
}

/// `line` without the byline word "by" (in any case) that opens it, when names follow it (see [`without_leading_and`]), and the
/// spaces after it (ties and control spaces too: `By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922).
fn without_leading_by(line: &Tokens) -> Option<Tokens> {
  let v = line.unlist_ref();
  let first = v.iter().position(|t| t.get_catcode() == Catcode::LETTER)?;
  let spacing = |t: Option<&Token>| t.is_some_and(|t| *t == T_SPACE!() || is_spacing_command(t));
  // (in any case: "by", "By", "BY")
  if !(v[first] == T_LETTER!("b") || v[first] == T_LETTER!("B"))
    || !(v.get(first + 1) == Some(&T_LETTER!("y")) || v.get(first + 1) == Some(&T_LETTER!("Y")))
    || !spacing(v.get(first + 2))
  {
    return None;
  }
  let mut rest = first + 2;
  while spacing(v.get(rest)) {
    rest += 1;
  }
  let mut out = v[..first].to_vec();
  out.extend_from_slice(&v[rest..]);
  let out = Tokens::new(out);
  (name_count(&visible_name_text(out.unlist_ref())) >= 1).then_some(out)
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
/// its arguments ([`NAME_ANNOTATIONS`], `\footnotemark[…]`, the mark commands of [`mark_commands`]:
/// `\textsuperscript`, `\IEEEauthorrefmark`, `\authorrefmark`), a superscript or subscript, or math holding only those
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
      // (a mark command too: ieeeaccess's `\authorrefmark{2}` after a list's closing period, 2201.07394)
      Catcode::CS if is_footnote_mark(&t) || is_name_annotation(&t) || is_mark_command(&t) => {
        any = true;
        i += 1;
        if tokens.get(i) == Some(&T_OTHER!("[")) {
          while i < tokens.len() && tokens[i] != T_OTHER!("]") {
            i += 1;
          }
          i += 1;
        }
        if !is_footnote_mark(&t) {
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

/// Whether a personname's child is a mark: an `<ltx:sup>`, or a formula that is only a superscript (`$^{1}$`, its `tex`
/// `{}^{1}` until the math is rewritten into an `<ltx:sup>`; [`shown_marks`]).
fn is_name_mark(node: &Node, qname: SymStr) -> bool {
  with(qname, |q| q == "ltx:sup")
    || (with(qname, |q| q == "ltx:Math")
      && node.get_attribute("tex").is_some_and(|tex| {
        tex
          .trim()
          .trim_start_matches("{}")
          .trim_start()
          .starts_with('^')
      }))
}

/// The text a `<ltx:personname>` shows, without its marks and notes (`ltx:sup`, `ltx:note`, math); an element
/// (a `<ltx:break/>`, a cell) parts the words around it. With `marks_part`, each mark ([`is_name_mark`]) reads as a
/// comma, as an author's mark after a name ends it.
fn personname_text(node: &Node, marks_part: bool) -> String {
  let mut text = String::new();
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) => text.push_str(&child.get_content()),
      Some(NodeType::ElementNode) => {
        text.push(' ');
        let qname = document::get_node_qname(&child);
        if marks_part && is_name_mark(&child, qname) {
          text.push(',');
        } else if !with(qname, |q| {
          q == "ltx:sup" || q == "ltx:note" || q == "ltx:Math" || q == "ltx:contact"
        }) {
          text.push_str(&personname_text(&child, marks_part));
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
    let text = personname_text(&person, false);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    // (a mark after a name ends it, as an author's mark stands: names parted only by marks and spacing,
    // `FLORIAN REETZ$^1$ \ns PRIYA SUBRAMANIAN$^{2,3}$`, 1911.02873, read as one name without them)
    let names = match name_count(&text) {
      n if n >= 2 => n,
      n => n.max(name_count(&personname_text(&person, true))),
    };
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

/// An author-block line holding email addresses behind the author's own label, or given by the class's email command,
/// as its label and addresses:
/// `E-mail: \email{x}` (JHEP.cls/PoS.cls author blocks, astro-ph0611258), `E-mail: \texttt{x@y}`, `\email{x}`. Each
/// address becomes `\lx@add@email`, the label (as written, "E-mail:") its `name`, so it stays markup a stylesheet can
/// show or hide (as the default "Email: " is). Before, the label stood as an affiliation of its own ("E-mail: ") and
/// a lone `\email{x}` line nested its own `\lx@add@email` in another, leaving an empty email contact. A link line to a
/// mailbox is one too, under whatever label it prints (`\textbf{Correspondence:} \href{mailto:…}{…}`, [`mailto_line`]).
/// `None` for any other line.
fn labelled_emails(line: &Tokens) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  // (a footnote symbol before the label is the line's mark, no part of it: `$^*$Corresponding author:
  // \href{mailto:…}{…}`, the r5 re-review's u23)
  let line = &Tokens::new(after_symbol_mark(line.unlist_ref()).to_vec());
  let (label, rest) = split_email_label(line);
  if let Some(calls) = email_command_calls(&rest)? {
    return Ok(Some((label, calls)));
  }
  let commands = rest
    .unlist_ref()
    .iter()
    .filter(|t| t.code == Catcode::CS)
    .map(is_email_command)
    .collect::<Result<Vec<bool>>>()?;
  if label.is_some() && line_is_email(&rest) && !commands.contains(&true) {
    return Ok(Some((label, vec![EmailAddress {
      address: rest,
      call:    None,
    }])));
  }
  // (several, a comma or a semicolon between them, each an address or a link to one: `Emails:
  // \href{mailto:a@x}{a@x}, \href{mailto:b@y}{b@y}`, the r5 re-review's u22)
  if label.is_some() && !commands.contains(&true) {
    let items = split_top_level_list(rest.unlist_ref());
    if items.len() > 1
      && items
        .iter()
        .all(|item| line_is_email(item) || mailto_line(item).is_some())
    {
      return Ok(Some((
        label,
        items
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
          .collect(),
      )));
    }
    // (or a list the address reader parts, a brace list among them: `Email: aable@x.edu; \{bbaker, ccole\}@y.edu`,
    // 2604.10316; the r8 re-review's §B, probes t1, t2 — had been one affiliation)
    if let Some(addresses) = email_addresses(&rest).filter(|addresses| addresses.len() > 1) {
      return Ok(Some((
        label,
        address_tokens(&rest, addresses)?
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
          .collect(),
      )));
    }
  }
  // …and a link line to a mailbox, under any label (`\textbf{Correspondence:} \href{mailto:…}{…}`, [`mailto_line`]),
  // an address each of a brace list it prints ([`mailto_addresses`]); and several such links after one label, a comma or
  // a semicolon between them (`\textbf{Correspondence:} \href{mailto:a@x}{a@x}, \href{mailto:c@x}{c@x}`, 2604.25444;
  // the r8 re-review's §B, probe t6 — had been one affiliation)
  if let Some((label, address)) = mailto_line(line) {
    return Ok(Some((label, mailto_addresses(address))));
  }
  let items = split_top_level_list(line.unlist_ref());
  if items.len() < 2 {
    return Ok(None);
  }
  let mut label = None;
  let mut addresses = Vec::new();
  for (at, item) in items.iter().enumerate() {
    match mailto_line(item) {
      Some((first, address)) if at == 0 => {
        label = first;
        addresses.extend(mailto_addresses(address));
      },
      Some((None, address)) => addresses.extend(mailto_addresses(address)),
      _ => return Ok(None),
    }
  }
  Ok(Some((label, addresses)))
}

/// The addresses a mailbox link prints ([`mailto_line`]): itself, or, when it prints a list of them, each address of it
/// ([`email_addresses`]: `\href{mailto:julia.belikova@skol.tech}{\{julia.belikova, a.panchenko\}@skol.tech}`,
/// 2602.12235; the r7 re-review's finding 7) — the one its target reaches kept the link, the others as they print.
fn mailto_addresses(link: EmailAddress) -> Vec<EmailAddress> {
  let v = link.address.unlist_ref();
  let Some(at) = v.iter().position(|t| *t == T_CS!("\\href")) else {
    return vec![link];
  };
  let (open, close) = link_printed_argument(v, at);
  let Some(addresses) =
    email_addresses(&Tokens::new(v[open..close].to_vec())).filter(|a| a.len() > 1)
  else {
    return vec![link];
  };
  let ((target_open, target_close), _) = link_arguments(v, at);
  let (target, _) = link_texts(v, at);
  let reached = target.trim_start_matches("mailto:").trim().to_lowercase();
  addresses
    .into_iter()
    .map(|address| {
      let printed = mouth::tokenize(TeXString::assembled(address.replace('_', "\\_"))).unlist();
      let tokens = if address.to_lowercase() == reached {
        let mut linked = vec![T_CS!("\\href")];
        linked.extend_from_slice(&v[target_open..target_close]);
        linked.push(T_BEGIN!());
        linked.extend(printed);
        linked.push(T_END!());
        linked
      } else {
        printed
      };
      EmailAddress {
        address: Tokens::new(tokens),
        call:    None,
      }
    })
    .collect()
}

/// `tokens` after the footnote symbol mark that leads them, as [`rewrite_symbol_superscripts`] writes it
/// (`\lx@frontmatter@keepsup{*}\lx@frontmatter@symbolmark{*}`), a size or font before it and the spaces after it;
/// `tokens` when none does.
fn after_symbol_mark(tokens: &[Token]) -> &[Token] {
  let whole = tokens;
  // (through a group around the whole piece: `{\small $^{\dagger}$\,University of Toronto}`, 2606.28166;
  // `{\it \normalsize $^\sharp$Department of Biostatistics, …}`, 2602.16540 — the r7 re-review's finding 1)
  let tokens = unwrapped(tokens);
  // (and a size or font before it, `\normalsize $^{\dagger}$Department of Statistics`, 2505.23260)
  let lead = tokens
    .iter()
    .take_while(|t| {
      **t == T_SPACE!()
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s)))
    })
    .count();
  if tokens.get(lead) != Some(&T_CS!("\\lx@frontmatter@keepsup")) {
    return whole;
  }
  let mut at = skip_group(tokens, lead + 1);
  if tokens.get(at) == Some(&T_CS!("\\lx@frontmatter@symbolmark")) {
    at = skip_group(tokens, at + 1);
  }
  let at = at.min(tokens.len());
  &tokens[at
    + tokens[at..]
      .iter()
      .take_while(|t| **t == T_SPACE!())
      .count()..]
}

/// `tokens` inside the groups around the whole of them, a one-argument font command's too (`\textit{…}`), and the spaces
/// around those (`{\small $^{\dagger}$\,Univ A \qquad $^{\ddagger}$\,EPFL}`, 2606.28166): what a wrapper group holds,
/// its font aside, is the piece; `tokens` when no group holds them whole.
fn unwrapped(tokens: &[Token]) -> &[Token] {
  let mut v = tokens;
  loop {
    let start = v.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = v
      .iter()
      .rposition(|t| *t != T_SPACE!())
      .map_or(start, |at| at + 1);
    let inner = &v[start..end.max(start)];
    // (and a one-argument font command around them, as a group: `\textit{$^{\dagger}$Univ A \qquad $^{\ddagger}$Univ
    // B}`, the r8 re-review's finding 8, probe s7)
    let font = usize::from(
      inner.len() >= 3
        && inner[0].get_catcode() == Catcode::CS
        && inner[0].with_str(|s| s.starts_with("\\text") || s == "\\emph")
        && inner[0].with_str(|s| NAME_TEXT_COMMANDS.contains(&s)),
    );
    let group = &inner[font..];
    if group
      .first()
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && skip_group(group, 0) == group.len()
      && group.len() >= 2
    {
      v = &group[1..group.len() - 1];
    } else {
      return v;
    }
  }
}

/// `tokens` before the footnote symbol marks that end them after their text (`Carnegie Mellon University, Pittsburgh, PA,
/// USA$^{\dagger}$`, `\texttt{mazeyuwhu@gmail.com}$^{\dagger}$`, 2109.13296; the r7 re-review's finding 1), inside a
/// group around the whole of them ([`unwrapped`]), without the spaces before the marks, and the marks as written
/// (`\lx@frontmatter@keepsup{…}\lx@frontmatter@symbolmark{…}`, [`keepsup`]): a mark after its text labels that text, as
/// one before it does. `None` when no mark ends them, or nothing stands before it.
fn before_trailing_symbol_mark(tokens: &[Token]) -> Option<(&[Token], &[Token])> {
  let v = unwrapped(tokens);
  let last = v.iter().rposition(|t| *t != T_SPACE!())? + 1;
  let mut end = last;
  while let Some(open) = group_opening(v, end)
    && open >= 1
    && v[open - 1] == T_CS!("\\lx@frontmatter@symbolmark")
    && let Some(sup) = group_opening(v, open - 1)
    && sup >= 1
    && v[sup - 1] == T_CS!("\\lx@frontmatter@keepsup")
  {
    end = sup - 1;
  }
  if end == last {
    return None;
  }
  let text = v[..end].iter().rposition(|t| *t != T_SPACE!())? + 1;
  Some((&v[..text], &v[end..last]))
}

/// Where the group that the `}` at `end - 1` of `v` closes opens; `None` when `v[end - 1]` closes none.
fn group_opening(v: &[Token], end: usize) -> Option<usize> {
  if end == 0 || v.get(end - 1)?.get_catcode() != Catcode::END {
    return None;
  }
  let mut depth = 0usize;
  for at in (0..end).rev() {
    match v[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => {
        depth -= 1;
        if depth == 0 {
          return Some(at);
        }
      },
      _ => {},
    }
  }
  None
}

/// The items of `tokens` at its commas and semicolons outside groups, each without the spaces around it; empty items
/// dropped.
fn split_top_level_list(tokens: &[Token]) -> Vec<Tokens> {
  let mut items: Vec<Tokens> = Vec::new();
  let mut item: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  let mut finish = |item: &mut Vec<Token>| {
    let start = item.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = item
      .iter()
      .rposition(|t| *t != T_SPACE!())
      .map_or(0, |at| at + 1);
    if start < end {
      items.push(Tokens::new(item[start..end].to_vec()));
    }
    item.clear();
  };
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && (*t == T_OTHER!(",") || *t == T_OTHER!(";")) => {
        finish(&mut item);
        continue;
      },
      _ => {},
    }
    item.push(*t);
  }
  finish(&mut item);
  items
}

/// One address of an email line, and the class's email command that gave it (`\email{x}`), if any.
struct EmailAddress {
  address: Tokens,
  call:    Option<Tokens>,
}

/// The words that can stand for a person in an email address ([`spelled_owners`]), lowercased, of the name as printed,
/// its notes aside (1706.03762's `Ashish Vaswani\thanks{… Noam proposed …}` is not "noam").
#[derive(Clone, Debug, Default)]
struct NameKeys {
  /// the name's words after the first that have three letters or more (`Ph.~Boucaud` "boucaud",
  /// `J.~Rodr\'iguez-Quintero` "rodriguez", "quintero")
  surnames: Vec<String>,
  /// its first word, when it has three letters or more (`Ahna R. Girshick` "ahna"; not the initial of `J.~Smith`)
  given:    Option<String>,
  /// the spellings an address opens with that join two of its words: the first word's initial and a later word of two
  /// letters or more (`Prathamjyot Singh` "psingh", 2608.10670's `psingh1_be22@…` among three Singhs; `Tianjian Li`
  /// "tli", 2606.23525's `tli104@…`; `Karamvir Singh Batra` "kbatra"), a later word before the whole first, the
  /// family name first (`Fei Wu` "wufei", `Yu Cheng` "chengyu": 2509.10377), or after it (`Di Wu` "diwu", 2604.01354;
  /// `Mo Yu` "moyu", 2608.18082 — 77p r10, where an address no name spells is the block's), and the initials of the
  /// words before the last with the last (`Kai-Wei Chang` "kwchang", 2604.01354); and the initials of every word, two or more
  /// (`Ann Able` "aa"), or of the first and the last when more stand between (`Joseph G. Wallwork` "jgw" and "jw",
  /// 2407.00382's `jw2423@cam.ac.uk`), which only a local part of those letters alone spells
  joined:   Vec<String>,
  initials: Vec<String>,
}

impl NameKeys {
  fn of(name: &Tokens) -> NameKeys {
    let text = visible_name_text(name.unlist_ref());
    let words: Vec<String> = text
      .split(|c: char| !c.is_alphabetic())
      .filter(|word| !word.is_empty())
      .map(str::to_lowercase)
      .collect();
    let Some((first, later)) = words.split_first() else {
      return NameKeys::default();
    };
    let initial: String = first.chars().take(1).collect();
    let mut joined: Vec<String> = Vec::new();
    for word in later.iter().filter(|word| word.chars().count() >= 2) {
      joined.push(format!("{initial}{word}"));
      if first.chars().count() >= 2 {
        joined.push(format!("{word}{first}"));
        joined.push(format!("{first}{word}"));
      }
    }
    // (the initials of every word before the last, then the last: `Kai-Wei Chang` "kwchang", 2604.01354)
    if let Some((last, before)) = words.split_last()
      && before.len() >= 2
      && last.chars().count() >= 2
    {
      let initials: String = before
        .iter()
        .filter_map(|word| word.chars().next())
        .collect();
      joined.push(format!("{initials}{last}"));
    }
    let initials: String = words
      .iter()
      .filter_map(|word| word.chars().next())
      .collect();
    NameKeys {
      surnames: later
        .iter()
        .filter(|word| word.chars().count() >= 3)
        .cloned()
        .collect(),
      given: Some(first.clone()).filter(|word| word.chars().count() >= 3),
      joined,
      initials: [
        Some(initials.clone()),
        (words.len() >= 3).then(|| {
          initials
            .chars()
            .take(1)
            .chain(initials.chars().last())
            .collect()
        }),
      ]
      .into_iter()
      .flatten()
      .filter(|initials: &String| initials.chars().count() >= 2)
      .collect(),
    }
  }
}

/// An address's letters and other characters, as [`spelled_owners`] reads them.
fn address_text(address: &Tokens) -> String {
  // (a colour, a font, a scale is no letter of it, [`without_non_printing_arguments`]: 2604.19440, 2609.25669)
  let printed = without_non_printing_arguments(address.unlist_ref());
  let v = printed.as_slice();
  let text = |v: &[Token]| -> String {
    v.iter()
      .filter(|t| matches!(t.code, Catcode::LETTER | Catcode::OTHER))
      .map(|t| t.to_string())
      .collect()
  };
  // `\href{mailto:t}{p}`: the address it prints, not its target and the printed one run together; its target's, past
  // `mailto:`, when it prints none (a placeholder target, `\href{mailto:email@domain}{yuxi.xia@univie.ac.at}`,
  // 2601.07974)
  if let Some(at) = v.iter().position(|t| *t == T_CS!("\\href")) {
    let open = at + 1 + v[at + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
    let close = skip_group(v, open).min(v.len());
    let target = text(&v[open..close]);
    let printed = text(&v[..at]) + &text(&v[close..]);
    return if printed.contains('@') {
      printed
    } else {
      target.trim_start_matches("mailto:").to_string()
    };
  }
  text(v)
}

/// Every name (index into `name_keys`) whose surname the local part of `address` spells ([`spelled_owners`]).
fn address_owners(name_keys: &[NameKeys], address: &Tokens) -> Vec<usize> {
  spelled_owners(name_keys, &address_text(address))
}

/// The names (indices into `name_keys`) the local part of `address` spells best: each name scores the letters of its
/// keys the local part holds — a surname whole, or its first six letters (`leyaouan@…` for "A.~Le~Yaouanc"); the given
/// name it opens with, or holds when the given name has four letters or more (2603.24730's
/// `\{yuqihu,~ahna,~vasha\}@berkeley.edu` for Yuqi Hu, Ahna R. Girshick, Vasha DuTell; 2603.20851's
/// `nlp2ct.shanshan@…` for Shanshan Wang, `lidiasc@…` for Lidia S. Chao), or a joined spelling it opens with when
/// that scores better (the better of the two, never their sum: 2602.07835) — and the highest scores own it: a longer
/// spelling outranks a key it holds (`fucheng.fuc@…` is Cheng Fu's, not Jiawei Chen's, 2403.00801; `wangchiew@…`
/// Wang-Chiew Tan's, not Xiaolan Wang's, 2104.01371; `yipengzhang97@…` Yipeng Zhang's, not Xiaoying Zhang's,
/// 2406.06326), a tie going to the surname's spelling. No name when none scores.
fn spelled_owners(name_keys: &[NameKeys], address: &str) -> Vec<usize> {
  let Some(local) = address.split('@').next() else {
    return Vec::new();
  };
  let local: String = local
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect::<String>()
    .to_lowercase();
  let surname_score = |key: &String| {
    if local.contains(key.as_str()) {
      key.chars().count()
    } else if key.chars().count() >= 6 && local.contains(&key.chars().take(6).collect::<String>()) {
      6
    } else {
      0
    }
  };
  let given_score = |given: &String| {
    let length = given.chars().count();
    if local.starts_with(given.as_str()) || (length >= 4 && local.contains(given.as_str())) {
      length
    } else {
      0
    }
  };
  // (a joined spelling the local part opens with, its length; the initials, the local part's letters alone)
  // (or its first six letters or more, as a surname's: a truncated family-first spelling, `zhuzhenh22@` for Zhenhao
  // Zhu, `chenyuxi22@` for Yuxiang Chen, 2602.03328, 2609.11716; the r7 re-review's finding 9 — scored the letters it
  // holds less one, below a spelling whole of as many: `wangyu@` is Yu Wang's "wangyu", not Yuxin Wang's "wangyuxin",
  // `liyang@` Yang Li's, not Yangfan Li's; the r8 re-review's finding 3 — where the local part ends there; one going on
  // past a shared start scores its six letters, as before)
  let joined_score = |keys: &NameKeys| {
    keys
      .joined
      .iter()
      .filter_map(|key| {
        if local.starts_with(key.as_str()) {
          Some(key.chars().count())
        } else {
          let held = local
            .chars()
            .zip(key.chars())
            .take_while(|(a, b)| a == b)
            .count();
          // (a truncation only when the local part's letters are used up, else its first six letters: `zhangweiwei@`
          // parts from "zhangweiwen" and is Wei Zhang's "zhangwei", the 77y review's finding 5, probe n1)
          if held == local.chars().count() {
            (held >= 6).then(|| held - 1)
          } else {
            (held >= 6).then_some(6)
          }
        }
      })
      .max()
      .unwrap_or(0)
      + keys
        .initials
        .iter()
        .find(|initials| **initials == local)
        .map_or(0, |initials| initials.chars().count())
  };
  // (the score, then its surname part: a tie goes to the surname, `chen@…` Wei Chen's before Chen Li's). A name scores
  // the better of its words spelled apart and a joined spelling, never both added: one spelling of the letters is
  // counted once (`khan.muhammad@ieee.org` scores 12 for Khan Muhammad, "muhammad" and "khan" apart, and 12 for
  // Muhammad Haris Khan, "khanmuhammad" joined, and goes to the surname's spelling, 2602.07835).
  let scores: Vec<(usize, usize)> = name_keys
    .iter()
    .map(|keys| {
      let surname: usize = keys.surnames.iter().map(surname_score).sum();
      (
        (surname + keys.given.as_ref().map_or(0, given_score)).max(joined_score(keys)),
        surname,
      )
    })
    .collect();
  let best = scores.iter().copied().max().unwrap_or((0, 0));
  if best.0 == 0 {
    return Vec::new();
  }
  (0..scores.len()).filter(|at| scores[*at] == best).collect()
}

/// The owner (index into `name_keys`) of each of a line's `addresses`: the one name it spells ([`spelled_owner`]); and,
/// by elimination, an address no name spells goes to a name no address spells — on this line, or on another e-mail line
/// of the block (`taken`, by index; 2605.06223's `kyle.min@…` on the next line leaves `treejhk@…` Junhyuk Kwon's) — only
/// where the order is confirmed (user decision 2026-10-20, batch 77p r4):
///  - exactly one address and one name are left (`\{lzhang,zhangw,smith\}@…` under Wei Zhang, Li Zhang, Bob Smith:
///    "zhangw" Wei Zhang's once "lzhang" is Li Zhang's); or
///  - the addresses the names spell stand in name order (one at least), and the addresses left are as many as the names
///    left (2605.26032's `\{jzxchen,chenzhuo,archerdw,gore,congyued,billf,soljacic\}@mit.edu`), or those between two
///    of them as many as the names between their owners (never a run before the first or after the last, which no
///    spelled address bounds: user ruling R10 2026-10-20, 2509.10377), every one of those
///    names counted, spelled on another line or not, and none of them spelled or given another address (`\{aszlam,jase,
///    robfergus\}@fb.com`: "jase" to Jason Weston, 1503.08895; 2603.24730's `j8675309@mit.edu` to Jennifer E. Corbett;
///    2303.11225's `xuta@…`, `musclewu@…` between `tianyuhe@…`, `tabaltru@…`, `runnan.li@…`).
///
/// A name left over is no one's to give while another e-mail line of the block holds an address no name spells
/// (`elsewhere_open`): that address may be the name's, so neither the one-left nor the as-many-left rule places one
/// (the r5 re-review's q6, `\{aable,q77,ddorn\}@x.edu \\ \{bbaker,z99\}@y.edu`: `q77@` is not Cat Cole's;
/// 2602.23777's `dongsli@` on the next line). The run between two spelled owners is read by the order alone, every
/// author between them counted, spelled on another line or not: 2602.23777's `nnwang@` between `xu_zhipeng@` and
/// `dcheng@` has Zilong Wang, Xinyang Jiang and Dongsheng Li between Zhipeng Xu and De Cheng, so it goes to none of
/// them; 2303.11225's `xuta@` has Xu Tan alone between Tianyu He and Tadas Baltrušaitis, as its PDF prints the list
/// in the authors' order (and its `\thanks` names `xuta@microsoft.com` Xu Tan's), though `ztk21@` on the line above
/// spells no one (batch 77p r6).
///
/// Not in order — a list by institution (2509.10377), or none spelled (`\{bb,aa\}@x.edu` reads by the initials, not
/// the order) — the rest stay `None`, as does a role mailbox (RFC 2142's `info@`, `support@`, …, [`is_role_mailbox`]),
/// which is no person's: the block's, on its shared creator (user ruling R10 2026-10-20).
fn owners_by_elimination(
  name_keys: &[NameKeys],
  addresses: &[String],
  taken: &[bool],
  elsewhere_open: bool,
) -> Vec<Option<usize>> {
  let mut owners: Vec<Option<usize>> = addresses
    .iter()
    .map(|address| spelled_owner(name_keys, address))
    .collect();
  let free = |name: usize| !taken.get(name).copied().unwrap_or(false);
  let left: Vec<usize> = (0..name_keys.len())
    .filter(|name| !owners.contains(&Some(*name)) && free(*name))
    .collect();
  let open: Vec<usize> = (0..owners.len())
    .filter(|at| owners[*at].is_none() && !is_role_mailbox(&addresses[*at]))
    .collect();
  if let ([at], [name]) = (open.as_slice(), left.as_slice())
    && !elsewhere_open
  {
    owners[*at] = Some(*name);
    return owners;
  }
  let spelled: Vec<usize> = owners.iter().flatten().copied().collect();
  if open.is_empty() || spelled.is_empty() || !spelled.windows(2).all(|pair| pair[0] < pair[1]) {
    return owners;
  }
  // (as many left as there are names left: in order, 2605.26032's `jzxchen@`, `billf@`, `soljacic@`)
  if open.len() == left.len() && !elsewhere_open {
    for (at, name) in open.into_iter().zip(left) {
      owners[at] = Some(name);
    }
    return owners;
  }
  // (each run of open addresses, with the owners of the spelled addresses around it, and every name between them)
  let mut run: Vec<usize> = Vec::new();
  let mut below: Option<usize> = None;
  for at in 0..=owners.len() {
    let bound = if at == owners.len() {
      Some(name_keys.len())
    } else {
      owners[at]
    };
    match bound {
      Some(above) => {
        let between = below.map_or(0, |below| below + 1)..above;
        // (a run before the first spelled owner or after the last is bounded by the list's end alone, which no spelled
        // address confirms: no position placement at a list's end (user ruling R10 2026-10-20; 2509.10377's `12421181@`
        // first in its list, the r5 re-review's q7 `z99@`); a run between two spelled owners is read by the order,
        // 2303.11225)
        let at_an_end = below.is_none() || at == owners.len();
        if !run.is_empty() && between.len() == run.len() && between.clone().all(free) && !at_an_end
        {
          for (address, name) in run.iter().zip(between) {
            owners[*address] = Some(name);
          }
        }
        run.clear();
        below = Some(above);
      },
      None if open.contains(&at) => run.push(at),
      None => {},
    }
  }
  owners
}

/// How many of a line's `addresses` no name spells ([`spelled_owner`]), a role mailbox aside: the addresses that keep
/// the names of the other e-mail lines of the block from going by elimination ([`owners_by_elimination`]).
fn unspelled_addresses(name_keys: &[NameKeys], addresses: &[String]) -> usize {
  addresses
    .iter()
    .filter(|address| spelled_owner(name_keys, address).is_none() && !is_role_mailbox(address))
    .count()
}

/// Whether `line` prints nothing: spacing ([`leading_spacing_end`], `\vspace{2pt}`), a paragraph break, and the groups, font and size
/// commands around them (`\large{~~}`, 2609.05430); a strut, an alignment declaration, an environment's `\begin{…}` or
/// `\end{…}` itself (`\end{@IEEEauthorhalign}\hfill\strut`, 2309.15562's IEEE block, whose environment name had read as an
/// address, then as an empty affiliation).
fn prints_nothing(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let mut i = 0;
  while i < v.len() {
    let t = &v[i];
    if matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) || *t == T_CS!("\\par")
      || is_spacing_command(t)
      || (t.get_catcode() == Catcode::CS
        && t.with_str(|s| {
          NAME_TEXT_COMMANDS.contains(&s)
            || matches!(
              s,
              "\\strut" | "\\centering" | "\\raggedright" | "\\raggedleft"
            )
        }))
    {
      i += 1;
    } else if *t == T_CS!("\\begin") || *t == T_CS!("\\end") {
      i = skip_group(v, i + 1).max(i + 1);
    } else if *t == T_CS!("\\vspace") || *t == T_CS!("\\hspace") {
      // (its length, after a star)
      let mut at = i + 1;
      while v
        .get(at)
        .is_some_and(|u| *u == T_SPACE!() || *u == T_OTHER!("*"))
      {
        at += 1;
      }
      i = skip_group(v, at);
    } else {
      match leading_spacing_end(&v[i..]) {
        0 => return false,
        skipped => i += skipped,
      }
    }
  }
  true
}

/// Whether `line` is only a link, `\href{…}{…}` or `\url{…}`, with the groups, size or font it is set in, the spacing
/// that prints nothing (`\href{https://jnnan.github.io/trumans/}{…} \vspace{-21pt}`, 2403.08629;
/// `{\footnotesize \url{…}}`, 2602.22394; `{\tt{\small{\url{…}}}}`, 2602.23802), and a label ending in a colon before it
/// (`Demo Code:~\url{…}`, 2602.09509).
fn is_link_line(line: &Tokens) -> bool { link_line_link(line).is_some() }

/// Where the one link of a link line ([`is_link_line`]) stands among its tokens, from its command to the end of its
/// arguments; `None` for any other line.
fn link_line_link(line: &Tokens) -> Option<(usize, usize)> {
  let v = line.unlist_ref();
  let (start, end) = (leading_spacing_end(v), spacing_end(v));
  // (a line of spacing alone is no link: 2510.07961, 2607.21993)
  if start >= end {
    return None;
  }
  let v = &v[start..end];
  let mut links = 0;
  let mut link = (0, 0);
  // (the last visible character of a label before the link)
  let mut label: Option<char> = None;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if matches!(
      t.get_catcode(),
      Catcode::SPACE | Catcode::BEGIN | Catcode::END
    ) || is_spacing_command(&t)
      || is_spacing(&t)
      || (t.get_catcode() == Catcode::CS
        && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s) || s == "\\texttt"))
    {
      i += 1;
    } else if let Some((end, _)) = non_printing_end(v, i) {
      // (a colour, a font, a scale, an icon the link is set in or beside prints no word: `{\color{orange}\textbf{\url{…}}}`,
      // `\textcolor{blue}{\url{…}}`, [`NON_PRINTING_ARGUMENTS`])
      i = end.max(i + 1);
    } else if t == T_CS!("\\href") || t == T_CS!("\\url") {
      if label.is_some_and(|last| last != ':') {
        return None;
      }
      // (a link that prints nothing is no link of the line: 2601.02680's `\href{…}{…} \url{}`)
      let (open, close) = link_printed_argument(v, i);
      if v[open..close].iter().all(|t| {
        matches!(
          t.get_catcode(),
          Catcode::BEGIN | Catcode::END | Catcode::SPACE
        )
      }) {
        i = close.max(i + 1);
        continue;
      }
      links += 1;
      let at = i;
      // (its arguments, the spaces before each skipped as TeX skips them: `\href{…}⏎{…}`, 2605.06223; none after a
      // bare `\href` that ends the line)
      let argument = |at: usize| {
        let at = at
          + v.get(at..).map_or(0, |rest| {
            rest.iter().take_while(|t| **t == T_SPACE!()).count()
          });
        skip_group(v, at)
      };
      i = argument(i + 1);
      if t == T_CS!("\\href") {
        i = argument(i);
      }
      link = (start + at, start + i.min(v.len()));
    } else if links == 0 && matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
      label = t.with_str(|s| s.chars().last());
      i += 1;
    } else {
      return None;
    }
  }
  (links == 1).then_some(link)
}

/// `tokens` with each `\quad`/`\qquad` that parts an icon from the link after it a space: a piece of an author block
/// that is only an icon — what prints no word, an icon macro or a graphic ([`without_non_printing_arguments`]) — before
/// a link line ([`is_link_line`]) is that link's, one link line with it, a URL contact ([`is_url_line`]), not an
/// affiliation holding the logo alone (`\github \quad \url{…}`, 2601.17879's
/// `\def\github{\raisebox{-1.5pt}{\includegraphics…}}`; the r8 re-review's finding 9).
fn join_icons_to_links(tokens: Tokens) -> Tokens {
  let mut v = tokens.unlist();
  let separator = |t: &Token| {
    [
      T_CS!("\\\\"),
      T_CS!("\\newline"),
      T_CS!("\\and"),
      T_CS!("\\And"),
      T_CS!("\\AND"),
      T_CS!("\\quad"),
      T_CS!("\\qquad"),
    ]
    .contains(t)
  };
  let mut delimiters = Vec::new();
  let mut depth = 0usize;
  for (at, t) in v.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && separator(t) => delimiters.push(at),
      _ => {},
    }
  }
  let icon_only = |piece: &[Token]| {
    let piece = Tokens::new(piece.to_vec());
    !prints_nothing(&piece)
      && prints_nothing(&Tokens::new(without_non_printing_arguments(
        piece.unlist_ref(),
      )))
  };
  for (index, at) in delimiters.iter().enumerate() {
    let before = if index == 0 {
      0
    } else {
      delimiters[index - 1] + 1
    };
    let after = delimiters.get(index + 1).copied().unwrap_or(v.len());
    if (v[*at] == T_CS!("\\quad") || v[*at] == T_CS!("\\qquad"))
      && icon_only(&v[before..*at])
      && is_link_line(&Tokens::new(v[at + 1..after].to_vec()))
    {
      v[*at] = T_SPACE!();
    }
  }
  Tokens::new(v)
}

/// The links of a line of several, each a link line of its own ([`is_url_line`]) with the icon or label before it, the
/// spacing and the separators (`|`, `·`, `,`) between them dropped (`\faGithub~\href{…}{Code} | \yticon~\href{…}{Demo
/// Video}`, 2601.06979; the r7 re-review's finding 5): a URL contact each. The line whole when it holds one link, or a
/// piece of it is no such link line (a row of linked names).
fn link_run_lines(line: Tokens) -> Vec<Tokens> {
  let v = line.unlist_ref();
  let mut ends = Vec::new();
  let mut depth = 0usize;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if depth == 0 && (t == T_CS!("\\href") || t == T_CS!("\\url")) {
      let (_, (_, after_second)) = link_arguments(v, i);
      let (_, close) = link_printed_argument(v, i);
      i = if t == T_CS!("\\href") {
        after_second
      } else {
        close
      }
      .max(i + 1);
      ends.push(i);
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    i += 1;
  }
  if ends.len() < 2 {
    return vec![line];
  }
  let separator = |t: &Token| {
    t.get_catcode() == Catcode::MATH
      || is_spacing(t)
      || is_spacing_command(t)
      || [",", ";", "|", "·", "/", "•"]
        .iter()
        .any(|mark| *t == T_OTHER!(*mark))
      || *t == T_CS!("\\textbar")
      || *t == T_CS!("\\textbullet")
      || *t == T_CS!("\\cdot")
  };
  let mut pieces = Vec::with_capacity(ends.len());
  let mut start = 0;
  for (at, end) in ends.iter().enumerate() {
    let end = if at + 1 == ends.len() { v.len() } else { *end };
    let mut from = start;
    while from < end && separator(&v[from]) {
      from += 1;
    }
    pieces.push(Tokens::new(v[from..end].to_vec()));
    start = end;
  }
  if pieces.iter().all(is_url_line) {
    pieces
  } else {
    vec![line]
  }
}

/// The words that name what a link leads to — a page, a project, its code or data — and no institution or person
/// (`Project Page`, `GitHub Repository`, `Hugging Face Demo`, `Substack Article`: 2602.22227, 2604.24697, 2601.18810,
/// 2604.15404, the r6 re-review's finding 2; `Paper Website`, the r8 re-review's finding 2, probe u3), as [`link_words`]
/// reads them.
const LINK_WORDS: &[&str] = &[
  "page",
  "webpage",
  "homepage",
  "website",
  "site",
  "url",
  "link",
  "code",
  "repository",
  "repo",
  "github",
  "demo",
  "dataset",
  "data",
  "article",
  "blog",
  "video",
  "model",
  "project",
  "paper",
];

/// Whether `text` holds a word of [`LINK_WORDS`], in any case, singular or plural.
fn link_words(text: &str) -> bool {
  text
    .split(|c: char| !c.is_alphanumeric())
    .map(str::to_lowercase)
    .any(|word| {
      let word = word
        .strip_suffix('s')
        .filter(|_| word.len() > 3)
        .unwrap_or(&word);
      LINK_WORDS.contains(&word)
    })
}

/// Whether the web address `target` reaches a page below its site's root: a path after its host (`https://x.org/proj`,
/// `github.com/a/b`), not the bare domain an institution's link is (`https://www.google.com`, `https://x.ai/`).
fn target_has_path(target: &str) -> bool {
  let rest = target.split_once("://").map_or(target, |(_, rest)| rest);
  rest
    .split_once('/')
    .is_some_and(|(_, path)| !path.trim_matches(['/', '#', '?']).is_empty())
}

/// Whether `line` is a link line ([`is_link_line`]) to a web page, a URL contact of the author block
/// ([`url_contact_call`]), by what the link says and where it goes, not by the shape of its words: not to a mailbox
/// (`\textbf{Correspondence:} \href{mailto:…}{…}`, 2601.00240, 2605.18563, an address and no URL: an e-mail line,
/// [`mailto_line`]); a `\url`, or an `\href` printing a web address or a link word before or in its text
/// ([`link_words`]: `\href{…}{Project Page}`, `Demo Code:~\href{…}{here}`, 2602.09509); not one printing an institution,
/// that institution linked, an affiliation (`\href{https://launchsafe.com}{\textbf{LaunchSafe Research}}`, 2606.26479,
/// the r5 re-review's u32, u33, u35), nor a person's name, that name (`\textit{Author:} \href{…}{Masashi Sekine}`,
/// 2603.22058); else a link printing no word (an icon) or to a page below a site's root is one, and a link to a bare
/// domain is the institution it prints (`\href{https://www.google.com}{Google}`, `\href{https://x.ai}{xAI}`, the r6 re-review's x10, w8).
fn is_url_line(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  let (target, printed) = link_texts(v, at);
  if links_a_mailbox(&target, &printed) {
    return false;
  }
  if v[at] == T_CS!("\\url") || reads_as_web_address(&printed) {
    return true;
  }
  let label = visible_name_text(&printed_words(&v[..at]));
  let (open, close) = link_printed_argument(v, at);
  let printed_tokens = Tokens::new(printed_words(&v[open..close]));
  // (an institution holding a link word is that institution linked: `\href{…}{Data Science Institute}`, the r7
  // re-review's finding 4, probe n6)
  if reads_as_affiliation(&printed_tokens) {
    return false;
  }
  if link_words(&printed) || link_words(&label) {
    return true;
  }
  if name_count(&visible_name_text(printed_tokens.unlist_ref())) > 0 {
    return false;
  }
  // (an icon, a link printing no word, is one: 2604.10930; and one printing a word that names no organization, `here`,
  // the r7 re-review's finding 10)
  !printed.chars().any(char::is_alphanumeric)
    || target_has_path(&target)
    || !names_an_organization(&visible_name_text(printed_tokens.unlist_ref()), &target)
}

/// Whether `line` is a link line printing an institution, no URL line ([`is_url_line`]): its printed text reads as an
/// affiliation (`\href{https://launchsafe.com}{\textbf{LaunchSafe Research}}`, 2606.26479), or it links a bare domain
/// and prints no name (`\href{https://www.google.com}{Google}`, the r6 re-review's x9, x10) — not a linked name
/// (`Advisor: \href{…}{Carl Doe}`, the r6 re-review's w5).
fn link_prints_an_institution(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  let (target, printed) = link_texts(v, at);
  if is_url_line(line) || links_a_mailbox(&target, &printed) {
    return false;
  }
  let (open, close) = link_printed_argument(v, at);
  let printed = Tokens::new(printed_words(&v[open..close]));
  let text = visible_name_text(printed.unlist_ref());
  reads_as_affiliation(&printed)
    || (name_count(&text) == 0
      && names_an_organization(&text, &target)
      && !target_has_path(&target))
}

/// Whether a bare domain's link text names its organization, as such a name is written — each word with a capital
/// (`Google`, `xAI`) — or as the domain `target` it links spells it in a handle
/// (`\href{https://www.deepseek.com}{deepseek-ai}`, the r8 re-review's finding 9, probe s14; not a homepage's bare
/// `{annable}`, the 77y review's finding 4), not a word of the sentence around it (`\href{https://ann.org}{here}`, a
/// link: the r7 re-review's finding 10, probes n13, p9).
fn names_an_organization(text: &str, target: &str) -> bool {
  let capitalized = !text.trim().is_empty()
    && text
      .split_whitespace()
      .all(|word| word.chars().any(char::is_uppercase));
  // (the domain's name, the label before its top-level one: "deepseek" of `https://www.deepseek.com`)
  let host = target
    .split_once("://")
    .map_or(target, |(_, rest)| rest)
    .split(['/', '?', '#'])
    .next()
    .unwrap_or_default()
    .to_lowercase();
  let labels: Vec<&str> = host.split('.').filter(|label| !label.is_empty()).collect();
  let spelled: String = text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .collect::<String>()
    .to_lowercase();
  // (an organization's handle, written with a mark no word holds — `deepseek-ai` —, not a person's: a homepage's link
  // printing the name its domain spells, `\href{https://www.annable.com}{annable}`, is a URL contact; the 77y review's
  // finding 4, probes d1, d2)
  // (the label's letters and digits, as the text's are read: `ml-group.org`, `{ml-group}`)
  let label: String = labels
    .len()
    .checked_sub(2)
    .map(|at| labels[at].chars().filter(|c| c.is_alphanumeric()).collect())
    .unwrap_or_default();
  let domain_named = label.chars().count() >= 3
    && spelled.contains(label.as_str())
    && text
      .chars()
      .any(|c| !c.is_alphabetic() && !c.is_whitespace());
  capitalized || domain_named
}

/// Whether `line` is a link printing one word whose letters are exactly a spelling of an author's name (`keys`: a joined
/// spelling or the initials, [`NameKeys`]): the author's own handle, a homepage — `\href{https://annable.com}{ann-able}`,
/// `\href{https://www.annable42.com}{annable42}`, `{AnnAble}` under Ann Able (the rev77y2 re-review's finding 5, probes
/// d5, d9, d12; `deepseek-ai` no name spells stays an affiliation). A name's words running inside an organization's
/// name are no spelling of it (`{Huawei}` under Hua Wu and Xiaoming Wei, `{Samsung}` under Sam Smith, `{able-lab}`
/// under Ann Able: affiliations, the rev77y3 re-review's finding 2, probes h1-h9). An organization whose name is exactly
/// an author's spelling (`{Huawei}` under Hua Wei, `{HW}` under Hua Wu) reads as that author's handle: the link stays on
/// the name it prints under, as a URL contact (rev77y4 probes h20, h21; SYNC residual).
fn link_prints_a_handle(line: &Tokens, keys: &[NameKeys]) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let (_, printed) = link_texts(&line.unlist_ref()[..end], at);
  let printed = printed.trim();
  let letters: String = printed
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect::<String>()
    .to_lowercase();
  !letters.is_empty()
    && !printed.contains(char::is_whitespace)
    && !printed.contains('@')
    && keys
      .iter()
      .any(|keys| keys.joined.contains(&letters) || keys.initials.contains(&letters))
}

/// Whether `line` is a link line ([`is_url_line`]) that prints a web address, a link word or a label before its link:
/// a `\url`, an `\href` printing a URL, `\href{…}{Project Page}`, `Project page: \href{…}{here}`. A link line opening an
/// author block or an `\and` column is only one so, a linked name there its author
/// (`\href{https://orcid.org/…}{\includegraphics{orcid.pdf}\hspace{1mm}Eric Rossetto}`, 2608.24427, 2602.15968,
/// 2603.04124); a column of a "Project Page" link alone is the block's, no author (the r6 re-review's w22). What the
/// link prints decides, never the line under it: a name whose surname is a link word stays a name
/// ([`link_words_beyond_a_surname`]; the rev77y2 re-review's finding 3).
fn prints_a_url(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  let (_, printed) = link_texts(v, at);
  // (a printed text that reads as a name is that name linked, unless it shows a scheme or `www.`, and no word of it is a
  // link word but a name's surname: `\href{…}{Lucy Page}`, `\href{…}{Edward A.G. Schuur}`, 2601.06085;
  // the r7 re-review's findings 2 and 4 — not `\href{…}{Project Page}`, `{Model Card}`, `{Paper Website}`, `{Hugging
  // Face Demo}`, the r8 re-review's finding 2, probes u1, u3, u4)
  if name_count(&printed) > 0
    && !printed.contains("://")
    && !printed
      .split_whitespace()
      .any(|word| word.starts_with("www."))
    && !link_words_beyond_a_surname(&printed)
  {
    return false;
  }
  is_url_line(line)
    && (v[at] == T_CS!("\\url")
      || reads_as_web_address(&printed)
      || link_words(&printed)
      || v[..at]
        .iter()
        .any(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)))
}

/// `v` without what prints no word ([`without_non_printing_arguments`]): a colour's arguments, a graphic with its
/// arguments (an ORCID icon, `\includegraphics[scale=0.06]{orcid.pdf}`), a skip's length (`\hspace{1mm}`, a space).
fn printed_words(v: &[Token]) -> Vec<Token> { without_non_printing_arguments(v) }

/// Where the printed argument of the link at `at` in `v` stands: an `\href`'s second, a `\url`'s one.
fn link_printed_argument(v: &[Token], at: usize) -> (usize, usize) {
  let (target, printed) = link_arguments(v, at);
  if v.get(at) == Some(&T_CS!("\\href")) {
    printed
  } else {
    target
  }
}

/// Where the first two arguments after the link command at `at` in `v` stand, the spaces before each skipped as TeX
/// skips them; empty past the end.
fn link_arguments(v: &[Token], at: usize) -> ((usize, usize), (usize, usize)) {
  let end = v.len();
  let argument = |from: usize| -> (usize, usize) {
    let from = from.min(end);
    let open = from + v[from..].iter().take_while(|t| **t == T_SPACE!()).count();
    (open, skip_group(v, open).clamp(open, end))
  };
  let first = argument(at + 1);
  (first, argument(first.1))
}

/// The target and the printed text of the link at `at` in `v` (`\href{target}{printed}`; a `\url`'s argument is both),
/// as their letters, other characters and spaces read, what prints no word aside ([`printed_words`]: a colour, an ORCID
/// icon's `\includegraphics[scale=0.06]{orcid.pdf}`, a skip's length).
fn link_texts(v: &[Token], at: usize) -> (String, String) {
  let text = |v: &[Token]| -> String {
    let mut text = String::new();
    for t in printed_words(v) {
      match t.get_catcode() {
        Catcode::LETTER | Catcode::OTHER => t.with_str(|s| text.push_str(s)),
        Catcode::SPACE => text.push(' '),
        Catcode::ACTIVE if t == T_ACTIVE!('~') => text.push(' '),
        _ => {},
      }
    }
    text.trim().to_string()
  };
  let ((open, close), _) = link_arguments(v, at);
  let target = text(&v[open..close]);
  let (open, close) = link_printed_argument(v, at);
  (target, text(&v[open..close]))
}

/// Whether `text` is a bare e-mail address, a mailbox and no web page: a local part, an `@`, a domain with a dot, and
/// no `/` (`binzhu@smu.edu.sg` as the ACL template's `\href{binzhu@smu.edu.sg}{binzhu@smu.edu.sg}` writes it, without
/// `mailto:`; 2604.04847, 2605.29064, 2605.29893, 2606.01600) — not a handle (`@aable`, the r6 re-review's w21), nor a
/// list of them (`{a,b}@x.edu` is no one address, [`email_addresses`] reads it: 2609.00588).
fn is_bare_address(text: &str) -> bool {
  let Some((local, domain)) = text.trim().split_once('@') else {
    return false;
  };
  !local.is_empty()
    && !local.contains([',', ';', '{', '}', ' '])
    && !text.contains('/')
    && domain.split('.').filter(|part| !part.is_empty()).count() >= 2
    && !domain.contains(['@', ' ', ','])
}

/// Whether a link to `target` printing `printed` reaches a mailbox: to `mailto:`, or a bare address
/// ([`is_bare_address`]) either way, its target no web page (`\href{https://x.com/aable}{@aable}` is a profile's, the r6
/// re-review's w21).
fn links_a_mailbox(target: &str, printed: &str) -> bool {
  target.starts_with("mailto:")
    || (!target.contains("://") && (is_bare_address(target) || is_bare_address(printed)))
}

/// Whether `text` prints a web address: a word with `://`, opening with `www.`, with a `/` between two letters or digits
/// (`github.io/traj_evo_search`), or with a `.` between two whose last label is a top-level domain of two letters or
/// more (`x.org`), and no `@` — not a run of dotted initials (`Edward A.G. Schuur`, 2601.06085; `T.J.C. Bezerra`,
/// 2606.03745; the r7 re-review's finding 2), nor a version number (`v1.2`).
fn reads_as_web_address(text: &str) -> bool {
  text.split_whitespace().any(|word| {
    let between = |c: u8| {
      word
        .as_bytes()
        .windows(3)
        .any(|w| w[0].is_ascii_alphanumeric() && w[1] == c && w[2].is_ascii_alphanumeric())
    };
    let initials = word
      .as_bytes()
      .windows(3)
      .any(|w| w[0].is_ascii_uppercase() && w[1] == b'.' && w[2].is_ascii_uppercase());
    let domain = word
      .trim_end_matches(|c: char| !c.is_alphanumeric())
      .rsplit_once('.')
      .is_some_and(|(_, last)| last.len() >= 2 && last.chars().all(|c| c.is_ascii_alphabetic()));
    !word.contains('@')
      && (word.contains("://")
        || word.starts_with("www.")
        || between(b'/')
        || (between(b'.') && domain && !initials))
  })
}

/// Whether `text` holds a link word ([`LINK_WORDS`]) other than a name's surname: `Model Card`, `Paper Website`,
/// `Hugging Face Demo`, `Interactive Online Demo`, `Project Page` (the r8 re-review's finding 2, probes u1, u3, u4; the
/// rev77y2 re-review's finding 3, probes l1-l3), wherever the column puts it. A word before the last is one whole, as
/// spaces part it (`Anne-Page Smith` a name, probe l7); the last is the surname, no link word, where the name reads as
/// one around it — two words (`Lucy Page`, `Mark Link`, `Hans-Peter Link`, `Lucy Page-Smith`: the r7 re-review's
/// finding 4), an initial before it (`Lawrence E. Page`), or more words before a link word that is also a surname
/// ([`SURNAME_LINK_WORDS`]: `Mary Ann Code`, `Lawrence Edward Page`, `Juan Carlos de la Page`, alone in their column or
/// over an e-mail or a place: the 77y review's finding 1, probes a1-a3, a7, a8; the rev77y2 re-review's finding 3,
/// probes l4-l6; no paper in either scan). A two-word text stays a name (`Source Code`, `Online Demo` alone in a column:
/// no corpus witness, probes a5, a6; a SYNC residual), and so does a three-word one ending in such a surname (`Full
/// Source Code`, `Google Scholar Page`, `Hugging Face Link`: the rev77y3 re-review's finding 5, probes s1, s3, s4, s7;
/// no author block linking such a text in the 2025-2026 scan; the same residual). The surname is read as a link word is,
/// a plural's `s` dropped (`Mary Ann Pages`), after a name suffix is set aside ([`NAME_SUFFIXES`]: `Lawrence Page Jr.`;
/// the rev77y3 re-review's findings 4 and 6, probes s9, s8).
fn link_words_beyond_a_surname(text: &str) -> bool {
  let mut words: Vec<&str> = text.split_whitespace().collect();
  // (a name suffix after the surname is no surname: `Lawrence Page Jr.`, the rev77y3 re-review's finding 6, probe s8)
  if words.len() > 2
    && words
      .last()
      .is_some_and(|word| NAME_SUFFIXES.contains(&word.trim_end_matches(',')))
  {
    words.pop();
  }
  let Some((last, before)) = words.split_last() else {
    return false;
  };
  let initial = |word: &&str| {
    word.ends_with('.')
      && word.chars().next().is_some_and(char::is_uppercase)
      && word.chars().all(|c| c.is_alphabetic() || c == '.')
      && word.chars().filter(|c| c.is_alphabetic()).count() <= 2
  };
  // (a word whole, as spaces part it: no part of `Anne-Page`; lowercased, a plural's `s` dropped)
  let stem = |word: &str| {
    let word = word
      .trim_matches(|c: char| !c.is_alphanumeric())
      .to_lowercase();
    match word.strip_suffix('s') {
      Some(stem) if word.len() > 3 => stem.to_string(),
      _ => word,
    }
  };
  let link_word = |word: &str| LINK_WORDS.contains(&stem(word).as_str());
  // (the last word's link words, a part of it each, `Page-Smith`, all surnames, read as a link word is: `Mary Ann
  // Pages`, the rev77y3 re-review's finding 4, probe s9)
  let surname = || {
    last
      .split(|c: char| !c.is_alphanumeric())
      .filter(|part| link_word(part))
      .all(|part| SURNAME_LINK_WORDS.contains(&stem(part).as_str()))
  };
  before.iter().any(|word| link_word(word))
    || (link_words(last) && !(before.len() == 1 || before.iter().any(initial) || surname()))
}

/// The link words ([`LINK_WORDS`]) that are also surnames (`Lawrence Edward Page`, `Mary Ann Code`, `Hans Link`): a
/// name's last word, never a link's ([`link_words_beyond_a_surname`]).
const SURNAME_LINK_WORDS: &[&str] = &["page", "link", "code"];

/// A link line ([`link_line_link`]) to a mailbox (`\textbf{Correspondence:} \href{mailto:x@y}{x@y}`, `Email:
/// \href{mailto:…}{…}`), read as a labelled e-mail line (user ruling 2026-10-20, batch 77p r8: 2601.00240, 2601.07974,
/// 2604.23002, 2606.03014, 2605.18563): the text before the link, as it prints without its font ("Correspondence:"),
/// is the label, and the link, kept a link, the address ([`address_text`] reads the one it prints). `None` for any
/// other line.
fn mailto_line(line: &Tokens) -> Option<(Option<Tokens>, EmailAddress)> {
  let (at, end) = link_line_link(line)?;
  let v = line.unlist_ref();
  // (the link's target and printed text: to a mailbox, or a bare address without `mailto:`, the ACL template's
  // `\textbf{Correspondence:} \href{binzhu@smu.edu.sg}{binzhu@smu.edu.sg}`, 2604.04847, 2605.29064, 2605.29893,
  // 2606.01600)
  let (target, printed) = link_texts(&v[..end], at);
  if !links_a_mailbox(&target, &printed) {
    return None;
  }
  let mut label: Vec<Token> = Vec::new();
  for t in &without_non_printing_arguments(&v[..at]) {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => label.push(*t),
      _ if (t.get_catcode() == Catcode::SPACE || is_spacing(t))
        && label.last().is_some_and(|last| *last != T_SPACE!()) =>
      {
        label.push(T_SPACE!())
      },
      _ => {},
    }
  }
  while label.last() == Some(&T_SPACE!()) {
    label.pop();
  }
  Some((
    (!label.is_empty()).then(|| Tokens::new(label)),
    EmailAddress {
      address: Tokens::new(v[at..end].to_vec()),
      call:    None,
    },
  ))
}

/// The e-mail addresses of an author block's line, as text: a line of addresses (`a@x, b@y`, `{a,b}@dom`) or a
/// labelled one (`Email: \email{a@x}`); none for any other line.
fn line_address_texts(line: &Tokens) -> Result<Vec<String>> {
  if let Some(addresses) = email_addresses(line) {
    return Ok(addresses);
  }
  Ok(
    line_labelled_emails(line)?
      .map(|(_, addresses)| {
        addresses
          .iter()
          .map(|address| address_text(&address.address))
          .collect()
      })
      .unwrap_or_default(),
  )
}

/// Which names (by index into `name_keys`) one of `addresses` spells ([`spelled_owner`]).
fn names_spelled(name_keys: &[NameKeys], addresses: &[String]) -> Vec<bool> {
  let mut spelled = vec![false; name_keys.len()];
  for address in addresses {
    if let Some(owner) = spelled_owner(name_keys, address) {
      spelled[owner] = true;
    }
  }
  spelled
}

/// Whether `address` is a role mailbox, no person's (RFC 2142 "Mailbox Names for Common Services, Roles and
/// Functions", and `contact@`, `office@`, `admin@`): `info@x.edu` beside `bbaker@x.edu` is not Ann Able's.
fn is_role_mailbox(address: &str) -> bool {
  let local = address
    .split('@')
    .next()
    .unwrap_or_default()
    .trim()
    .to_lowercase();
  matches!(
    local.as_str(),
    "info"
      | "marketing"
      | "sales"
      | "support"
      | "abuse"
      | "noc"
      | "security"
      | "postmaster"
      | "hostmaster"
      | "usenet"
      | "news"
      | "webmaster"
      | "www"
      | "uucp"
      | "ftp"
      | "contact"
      | "office"
      | "admin"
  )
}

/// Whether the local part of the e-mail address `address` ([`address_text`]) holds a list the address reader could not
/// part (`\{luoyanyan,…,fengjunlan\}it@chinamobile.com`, 2605.27805; the r8 re-review's finding 4, probe t5): its
/// letters run together spell one of the names it lists, so it spells no one, the block's (user ruling R10).
fn lists_local_parts(address: &str) -> bool {
  address
    .split_once('@')
    .is_some_and(|(local, _)| local.contains([',', ';', '|', '{', '}']))
}

/// The one name (index into `name_keys`) the local part of `address` spells ([`spelled_owners`]), or `None` when no name or
/// several do.
fn spelled_owner(name_keys: &[NameKeys], address: &str) -> Option<usize> {
  if lists_local_parts(address) {
    return None;
  }
  match spelled_owners(name_keys, address).as_slice() {
    [owner] => Some(*owner),
    _ => None,
  }
}

/// The email calls for `addresses`: under the author's `label`, `\lx@add@email` named by it; without one, the class's
/// own email command as written (`\email{x}`), or `\lx@add@email`; placed by the label `placement` gives when it gives
/// one (`label=author:N`, the creator that numbers, placed once the authors are made, in the order the lines are written;
/// `label=addresses:block`, the block's).
fn email_calls(
  label: Option<&Tokens>,
  addresses: Vec<EmailAddress>,
  placement: Option<&str>,
) -> Result<Vec<Token>> {
  let mut kv: Vec<Token> = Vec::new();
  if let Some(label) = label {
    kv.extend(mouth::tokenize_internal("name=").unlist());
    kv.push(T_BEGIN!());
    kv.extend(label.unlist_ref().iter().copied());
    kv.push(T_SPACE!());
    kv.push(T_END!());
  }
  if let Some(placement) = placement {
    if !kv.is_empty() {
      kv.push(T_OTHER!(","));
    }
    kv.extend(mouth::tokenize_internal(TeXString::assembled(placement.to_string())).unlist());
  }
  let keyvals = (!kv.is_empty()).then(|| Tokens::new(kv));
  let mut calls = Vec::new();
  for EmailAddress { address, call } in addresses {
    match (&keyvals, call) {
      // no label of the author's: the class's own command, with the name it gives (amsart's "Email address: ")
      (None, Some(call)) => calls.extend(call.unlist()),
      // under the label, the address as the class's command sets it (JHEP's `E-mail: \email{x}`)
      (Some(_), Some(call)) => {
        let address = match call.unlist_ref().first() {
          Some(cmd) => email_command_address(cmd, address)?,
          None => address,
        };
        calls.extend(
          Invocation!(T_CS!("\\lx@add@email"), vec![
            keyvals.clone(),
            Some(address)
          ])
          .unlist(),
        )
      },
      _ => calls.extend(
        Invocation!(T_CS!("\\lx@add@email"), vec![
          keyvals.clone(),
          Some(address)
        ])
        .unlist(),
      ),
    }
  }
  Ok(calls)
}

/// The authors (indices into `author_marks`) whose marks request the affiliation label `label` ([`mark_label`]).
fn authors_requesting(author_marks: &[Vec<Vec<Token>>], label: &str) -> Vec<usize> {
  author_marks
    .iter()
    .enumerate()
    .filter(|(_, marks)| {
      marks.iter().any(|operand| {
        clean_frontmatter_labels(&Tokens::new(operand.clone()).to_string(), "affiliation")
          .iter()
          .any(|requested| requested == label)
      })
    })
    .map(|(at, _)| at)
    .collect()
}

/// Who the addresses of a labelled email line may belong to besides the authors they spell ([`block_email_calls`]).
struct EmailOwners {
  /// the one author a `\thanks` names corresponding (as `author:N` numbers it, from 0)
  corresponding:   Option<usize>,
  /// the authors (numbered so) the mark of a note just before the line names, a note naming the corresponding author
  marked:          Vec<usize>,
  /// the authors numbered before this call's (`author:N` counts them)
  numbered_before: usize,
}

/// The `\lx@add@email` calls for a labelled email line of a marked author block ([`labelled_emails`]): each address to
/// the author whose surname its local part spells (`label=author:N`, [`address_owner`]), as an unmarked block's lines
/// give theirs (counted among all numbered authors, those of earlier calls too); one spelling no name, after a note
/// naming the corresponding author, to the one author that note's mark names; a lone one to the one author a `\thanks`
/// names corresponding (JINST's
/// `S.~Bityukov$^a$\thanks{Corresponding author.}` over `E-mail: \email{Serguei.Bitioukov@cern.ch}`, 1302.2651); else a
/// contact of the whole block (`label=addresses:block`, the shared creator of #159). The label as written is the
/// contact's `name`. (Here the spelling outranks a note's mark; a mark on the address itself outranks the spelling for a
/// contact placed apart from the names, [`placed_contact_owner`].)
fn block_email_calls(
  label: Option<&Tokens>,
  addresses: Vec<EmailAddress>,
  author_keys: &[NameKeys],
  owners: EmailOwners,
) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  let lone = addresses.len() == 1;
  for EmailAddress { address, call } in addresses {
    // the address itself, not the class's email command around it (`\email{x}` it does not count as one): the contact
    // is what that command stands for, and a class that kills it once its title page is typeset (JINST copies as
    // `arxiv.cls`, `\global\let\email\@gobble`) left the contact empty (1111.1180, 1210.0427); set as the command
    // sets it ([`email_command_address`])
    let (address, command) = match whole_line_cs_wrapper(&address) {
      Some((command, inner)) if is_email_wrapper(&command) => (inner, Some(command)),
      _ => (
        address,
        call.and_then(|call| call.unlist_ref().first().copied()),
      ),
    };
    let mut keyvals = Vec::new();
    if let Some(label) = label {
      keyvals.extend(mouth::tokenize_internal("name=").unlist());
      keyvals.push(T_BEGIN!());
      keyvals.extend(label.unlist_ref().iter().copied());
      keyvals.push(T_SPACE!());
      keyvals.push(T_END!());
      keyvals.push(T_OTHER!(","));
    }
    // the author the address spells; spelling no one, a lone address is the corresponding author's; spelling several
    // (`X.~Wang`, `Y.~Wang`), the corresponding author's only if among them
    let spelled: Vec<usize> = address_owners(author_keys, &address)
      .into_iter()
      .filter(|_| !lists_local_parts(&address_text(&address)))
      .map(|author| owners.numbered_before + author)
      .collect();
    let owner = match spelled.as_slice() {
      [owner] => Some(*owner),
      [] => owners.corresponding.filter(|_| lone),
      several => owners
        .corresponding
        .filter(|author| several.contains(author)),
    };
    // spelled by one name: that author's, though a note marked otherwise precedes it (1710.00802's `aleder@` is A.
    // Leder's, under "Corresponding Author"); else after a note naming the corresponding author, the one author its
    // mark names (1305.7027's `$^\spadesuit$`)
    // (a lone address only, and among several it spells only one of them)
    let owner = match (spelled.len(), owners.marked.as_slice()) {
      (1, _) => owner,
      (0, [marked]) if lone => Some(*marked),
      (_, [marked]) if lone && spelled.contains(marked) => Some(*marked),
      _ => owner,
    };
    let placement = match owner {
      // (`author:N` counts every numbered author, those of an earlier `\author` call too: 1805.09245's collaboration)
      Some(author) => s!("label=author:{}", author + 1),
      None => s!("label=addresses:block"),
    };
    keyvals.extend(mouth::tokenize_internal(TeXString::assembled(placement)).unlist());
    let address = match command {
      Some(command) => email_command_address(&command, address)?,
      None => address,
    };
    calls.extend(
      Invocation!(T_CS!("\\lx@add@email"), vec![
        Some(Tokens::new(keyvals)),
        Some(address)
      ])
      .unlist(),
    );
  }
  Ok(calls)
}

/// A labelled email line ([`labelled_emails`]) behind what prints nothing before it — spacing, declarations, the
/// paragraph break of a blank line in the block (1112.1037's `…Davis,\\ <blank line> E-mail: \email{…}`) — without
/// that lead or a like tail, or `None`.
fn labelled_email_line(line: &Tokens) -> Result<Option<Tokens>> {
  let toks = line.unlist_ref();
  let mut lead = 0;
  loop {
    let start = lead;
    while toks.get(lead).is_some_and(|t| *t == T_CS!("\\par")) {
      lead += 1;
    }
    lead += leading_unprinted_end(&toks[lead..]);
    if lead == start {
      break;
    }
  }
  // …and a blank line or spacing after it (1311.3535's `E-mail: \email{…}<blank line>}`)
  let mut end = toks.len();
  while end > lead && (toks[end - 1] == T_CS!("\\par") || is_spacing(&toks[end - 1])) {
    end -= 1;
  }
  // …and a group around the whole of it, with what leads it there (`\tt\small {Email Address:
  // songyuxinbb@outlook.com}`, 2603.19228)
  // (`\small {…}` reads as a command over its argument, which a size or font command does not take)
  loop {
    let open = if toks.get(lead).is_some_and(|t| {
      t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s))
    }) {
      lead + 1
    } else {
      lead
    };
    if end > open + 1 && toks[open].get_catcode() == Catcode::BEGIN && skip_group(toks, open) == end
    {
      lead = open + 1 + leading_unprinted_end(&toks[open + 1..end - 1]);
      end -= 1;
    } else {
      break;
    }
  }
  let rest = Tokens::new(toks[lead..end].to_vec());
  Ok(labelled_emails(&rest)?.is_some().then_some(rest))
}

/// The label and addresses of a labelled e-mail line ([`labelled_emails`]) behind what prints nothing before it
/// ([`labelled_email_line`]); `None` for any other line.
fn line_labelled_emails(line: &Tokens) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  match labelled_email_line(line)? {
    Some(rest) => labelled_emails(&rest),
    None => Ok(None),
  }
}

/// Whether a note's text names its author the corresponding one ("Corresponding author.", "Correspondence to …",
/// "to whom correspondence should be addressed"; not a "corresponding member" of an academy).
fn names_correspondence(text: &str) -> bool {
  let text = text.to_lowercase();
  text.contains("corresponding author") || text.contains("correspondence")
}

/// The one author (index among `authors`) whose own `\thanks`/`\footnote` note names them the corresponding author
/// ("Corresponding author.", "Correspondence to …"), or `None` when no author's or several authors' do.
fn corresponding_author<'a>(authors: impl Iterator<Item = &'a Tokens>) -> Option<usize> {
  let notes = [T_CS!("\\thanks"), T_CS!("\\footnote")];
  let mut owners = authors.enumerate().filter(|(_, author)| {
    let toks = author.unlist_ref();
    toks.iter().enumerate().any(|(i, t)| {
      // (past an optional argument: `\thanks[1]{…}`)
      let mut at = i + 1;
      if notes.contains(t) && toks.get(at) == Some(&T_OTHER!("[")) {
        at += toks[at..]
          .iter()
          .position(|t| *t == T_OTHER!("]"))
          .map_or(0, |close| close + 1);
      }
      notes.contains(t)
        && toks
          .get(at)
          .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
        && names_correspondence(&visible_name_text(&toks[at..skip_group(toks, at)]))
    })
  });
  let (owner, _) = owners.next()?;
  owners.next().is_none().then_some(owner)
}

/// A line's leading email label — "E-mail:", "Email:", "e-mails:", "E-mail address:", up to its colon, or the same
/// words without one right before a command — and the rest of the line.
fn split_email_label(line: &Tokens) -> (Option<Tokens>, Tokens) {
  let tokens = line.unlist_ref();
  let mut text = String::new();
  let mut end = None;
  for (i, t) in tokens.iter().enumerate() {
    match t.code {
      Catcode::LETTER | Catcode::OTHER => {
        let ch = t.to_string();
        if ch == ":" {
          end = Some(i + 1);
          break;
        }
        text.push_str(&ch);
      },
      Catcode::SPACE => text.push(' '),
      Catcode::CS => {
        end = Some(i);
        break;
      },
      _ => break,
    }
  }
  let is_label = {
    let words: String = text
      .chars()
      .filter(|c| !c.is_whitespace() && *c != '-')
      .collect::<String>()
      .to_lowercase();
    matches!(
      words.as_str(),
      "email" | "emails" | "emailaddress" | "emailaddresses"
    )
  };
  match end {
    Some(end) if is_label => {
      let label: Vec<Token> = tokens[..end]
        .iter()
        .copied()
        .skip_while(|t| *t == T_SPACE!())
        .collect();
      let label_end = label
        .iter()
        .rposition(|t| !is_spacing(t))
        .map_or(0, |at| at + 1);
      (
        Some(Tokens::new(label[..label_end].to_vec())),
        Tokens::new(
          tokens[end..]
            .iter()
            .copied()
            .skip_while(is_spacing)
            .collect(),
        ),
      )
    },
    _ => (None, line.clone()),
  }
}

/// The addresses of a line made only of the class's email commands (a macro whose expansion is `\lx@add@email…`,
/// JHEP's `\email`, amsart's) and the commas, semicolons or spaces between them, each with its command call; `None`
/// for any other line.
fn email_command_calls(line: &Tokens) -> Result<Option<Vec<EmailAddress>>> {
  let tokens = line.unlist_ref();
  let mut calls = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    let t = &tokens[i];
    if is_spacing(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";") {
      i += 1;
      continue;
    }
    if t.code != Catcode::CS || !is_email_command(t)? {
      return Ok(None);
    }
    let command = i;
    i += 1;
    while i < tokens.len() && tokens[i] == T_SPACE!() {
      i += 1;
    }
    // the command's optional argument, part of the call (AASTeX's `\email[show]{x}`, aastex701.cls:13341)
    if i < tokens.len() && tokens[i] == T_OTHER!("[") {
      let Some(close) = tokens[i..].iter().position(|t| *t == T_OTHER!("]")) else {
        return Ok(None);
      };
      i += close + 1;
      while i < tokens.len() && tokens[i] == T_SPACE!() {
        i += 1;
      }
    }
    if i >= tokens.len() || tokens[i].code != Catcode::BEGIN {
      return Ok(None);
    }
    let start = i + 1;
    let mut depth = 0usize;
    while i < tokens.len() {
      match tokens[i].code {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            break;
          }
        },
        _ => {},
      }
      i += 1;
    }
    if depth != 0 {
      return Ok(None);
    }
    calls.push(EmailAddress {
      address: Tokens::new(tokens[start..i].to_vec()),
      call:    Some(Tokens::new(tokens[command..=i].to_vec())),
    });
    i += 1;
  }
  Ok(if calls.is_empty() { None } else { Some(calls) })
}

/// Space between an email label and its addresses, or between addresses: a space, a tie, or a spacing command
/// (`E-mail:~\email{x}`, `E-mail:\ \email{x}`).
fn is_spacing(t: &Token) -> bool {
  *t == T_SPACE!()
    || *t == T_ACTIVE!('~')
    || (t.code == Catcode::CS
      && matches!(
        t.to_string().as_str(),
        "\\ "
          | "\\,"
          | "\\;"
          | "\\:"
          | "\\quad"
          | "\\qquad"
          | "\\enskip"
          | "\\enspace"
          | "\\thinspace"
          | "\\space"
      ))
}

/// A note or mark command an author name carries (`\thanks`, `\footnote`, `\inst`, …: [`is_name_note_command`], the
/// [`NAME_ANNOTATIONS`] but those printed in place, [`PRINTED_IN_PLACE`]; elsarticle's note references), or a macro
/// whose expansion adds to the frontmatter itself (`\lx@add@…`, `\lx@annotate@…`, `\@add@frontmatter`, `\lx@request@…`:
/// aa's `\mail`, achemso's `\email`, JHEP's `\email`).
fn adds_to_frontmatter(cs: &Token) -> Result<bool> {
  let note = cs.with_str(|name| {
    is_name_note_command(name)
      || matches!(
        name,
        "\\footnotemark"
          | "\\footnotetext"
          | "\\fnref"
          | "\\corref"
          | "\\tnoteref"
          | "\\thankstext"
      )
  });
  if note {
    return Ok(true);
  }
  // a macro whose expansion begins with one of those, or with a macro that does (quantumarticle's `\ead` → `\email`);
  // a macro computed by code (imsart's `\ead`) is taken to add itself
  let frontmatter_call = |t: &Token| {
    t.with_str(|name| {
      [
        "\\lx@add@",
        "\\lx@annotate@",
        "\\@add@frontmatter",
        "\\lx@request@",
      ]
      .iter()
      .any(|prefix| name.starts_with(prefix))
    })
  };
  let Some(def) = lookup_definition(cs)? else {
    return Ok(false);
  };
  Ok(match def.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => match body.unlist_ref().first() {
      Some(first) if frontmatter_call(first) => true,
      Some(first) if first.code == Catcode::CS => lookup_definition(first)?.is_some_and(|inner| {
        matches!(inner.get_expansion(), Some(ExpansionBody::Tokens(body))
          if body.unlist_ref().first().is_some_and(frontmatter_call))
      }),
      _ => false,
    },
    Some(_) => true,
    None => false,
  })
}

/// A class's email command: a macro whose expansion begins with a frontmatter email call — `\lx@add@email`
/// (JHEP.cls.ltxml's `\email`), or one that decides how to record the address (AASTeX 7's `\lx@add@aas@email`, the
/// `\lx@add@email@metadata` of an address the PDF does not print).
fn is_email_command(cs: &Token) -> Result<bool> {
  let Some(def) = lookup_definition(cs)? else {
    return Ok(false);
  };
  Ok(
    matches!(def.get_expansion(), Some(ExpansionBody::Tokens(body))
    if body.unlist_ref().first().is_some_and(|first| first.with_str(|name| {
      name.strip_prefix("\\lx@add@").is_some_and(|rest| rest.contains("email"))
    }))),
  )
}

/// A marker-less line that is *purely* a list of email addresses — a shared
/// `\texttt{a@x, b@y,}` / `\email{...}` / `\{a,b\}@x` line covering all authors. Like
/// [`line_is_email`] but tolerates a comma-separated list (and the trailing
/// comma authors often leave), as [`email_addresses`] reads it: every item a full
/// address or a braced list of local parts before its domain. A prose affiliation
/// line ("Dept. of Foo, University of Pisa, Italy") is rejected, so this never
/// misclassifies an address as an email.
fn line_is_email_list(line: &Tokens) -> bool { email_addresses(line).is_some() }

/// The commands whose arguments print no letter, each with those arguments as an xparse-style signature — `s` a star,
/// `o` an optional `[…]` argument, `m` a mandatory one — and whether the command prints a space: what the command
/// prints, if anything, is the argument after them. One table for every reader of an author block's printed text
/// ([`email_addresses`], [`address_text`], [`printed_words`], [`link_line_link`], [`mailto_line`]'s label): a colour, a
/// font's name or size, a scale, an angle, a raise, a graphic's file, a skip's length are no letters of an address, a
/// label or a link's text (`\href{mailto:…}{\color{black}…}`, 2604.19440; `\texttt{\fontfamily{pcr}\selectfont
/// xw109@illinois.edu}`, 2609.25669, 2609.35216; `\scalebox{0.9}{…}`, `\fontsize{9}{11}`, the r6 re-review's w14,
/// w15). The signatures are those of xcolor.sty (`\fcolorbox[model]{frame}[model]{background}{text}`, `\colorlet`,
/// `\definecolor`), latex.ltx's font selection (`\fontsize{size}{skip}`, `\usefont{enc}{family}{series}{shape}`) and
/// boxes (`\raisebox{lift}[height][depth]{text}`, `\hspace*{len}`), and graphicx.sty (`\scalebox{h}[v]{text}`,
/// `\rotatebox[keys]{angle}{text}`, `\resizebox*{width}{height}{text}`, `\includegraphics*[…][…]{file}`, all of whose
/// arguments print no letter).
const NON_PRINTING_ARGUMENTS: &[(&str, &str, bool)] = &[
  ("\\color", "om", false),
  ("\\textcolor", "om", false),
  ("\\colorbox", "om", false),
  ("\\fcolorbox", "omom", false),
  ("\\pagecolor", "om", false),
  ("\\colorlet", "omom", false),
  ("\\definecolor", "ommm", false),
  ("\\fontfamily", "m", false),
  ("\\fontseries", "m", false),
  ("\\fontshape", "m", false),
  ("\\fontencoding", "m", false),
  ("\\fontsize", "mm", false),
  ("\\usefont", "mmmm", false),
  ("\\scalebox", "mo", false),
  ("\\rotatebox", "om", false),
  ("\\raisebox", "moo", false),
  ("\\resizebox", "smm", false),
  ("\\includegraphics", "soom", false),
  ("\\hspace", "sm", true),
  ("\\vspace", "sm", true),
  // (latex.ltx's `\rule[raise]{width}{height}` and `\phantom{…}` print a box, no letter: the r7 re-review's finding 11,
  // probes p1, p2, p4)
  ("\\rule", "omm", false),
  ("\\phantom", "m", true),
  ("\\hphantom", "m", true),
  ("\\vphantom", "m", false),
  // (TeX's `\kern`, `\hskip`, `\vskip` take a dimension or glue, `d`/`g`, no group: probes p5, p6)
  ("\\kern", "d", true),
  ("\\hskip", "g", true),
  ("\\vskip", "g", false),
  // (an icon prints its glyph, no letter: fontawesome5's `\faIcon[style]{name}`, fontawesome's `\faicon[…]{name}` and
  // `\faIconFromMacro{name}`, which their `\faGithub`-style commands expand to, [`icon_macro_options`]; 2601.03661,
  // 2601.06979, the r7 re-review's finding 5)
  ("\\faIcon", "om", false),
  ("\\faicon", "om", false),
  ("\\faIconFromMacro", "m", false),
];

/// Where the command at `at` and its arguments that print no letter end ([`NON_PRINTING_ARGUMENTS`]), the spaces before
/// each argument skipped as TeX skips them, and whether it prints a space; `None` for any other token.
fn non_printing_end(v: &[Token], at: usize) -> Option<(usize, bool)> {
  non_printing_end_within(v, at, 0)
}

/// [`non_printing_end`], `depth` macros deep in the expansion of a command that prints no word
/// ([`icon_macro_options`]).
fn non_printing_end_within(v: &[Token], at: usize, depth: usize) -> Option<(usize, bool)> {
  let t = v.get(at)?;
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  let found = t.with_str(|name| {
    NON_PRINTING_ARGUMENTS
      .iter()
      .find(|(command, ..)| *command == name)
      .map(|(_, signature, space)| (*signature, *space))
  });
  // (a macro of optional arguments alone whose expansion prints no word, an icon: `\faGithub`, a user's
  // `\newcommand{\yticon}{\textcolor{youtubeRed}{\faYoutube}}`)
  let owned;
  let (signature, space) = match found {
    Some(found) => found,
    None => {
      owned = "o".repeat(icon_macro_options(t, depth)?);
      (owned.as_str(), false)
    },
  };
  let mut i = at + 1;
  for kind in signature.chars() {
    let next = i
      + v[i.min(v.len())..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count();
    match kind {
      's' if v.get(next) == Some(&T_OTHER!("*")) => i = next + 1,
      'o' if v.get(next) == Some(&T_OTHER!("[")) => {
        i = next
          + v[next..]
            .iter()
            .position(|t| *t == T_OTHER!("]"))
            .map_or(v.len() - next, |close| close + 1);
      },
      'm' if next < v.len() => i = skip_group(v, next).max(next + 1),
      'd' | 'g' => i = next + dimension_length(&v[next.min(v.len())..], kind == 'g'),
      _ => {},
    }
  }
  Some((i.min(v.len()), space))
}

/// How many tokens of `v` a dimension takes (`2em`, `-1.5pt`, `\fill`, `0.5\textwidth`), the spaces in it, and with
/// `glue` its `plus`/`minus` parts (`2em plus 1fil`) — TeX's ⟨dimen⟩ and ⟨glue⟩ (tex.web §448, §461), read as written
/// in an author line.
fn dimension_length(v: &[Token], glue: bool) -> usize {
  let text_of = |t: &Token| -> Option<char> {
    matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)
      .then(|| t.with_str(|s| s.chars().next()))
      .flatten()
  };
  // (a sign, digits and a point, then a unit of two letters or a register)
  let dimen = |from: usize| -> usize {
    let mut i = from;
    while v
      .get(i)
      .is_some_and(|t| *t == T_SPACE!() || matches!(text_of(t), Some('+' | '-')))
    {
      i += 1;
    }
    let digits = i;
    while v
      .get(i)
      .is_some_and(|t| matches!(text_of(t), Some(c) if c.is_ascii_digit() || c == '.' || c == ','))
    {
      i += 1;
    }
    while v.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    match v.get(i) {
      Some(t) if t.get_catcode() == Catcode::CS => i + 1,
      Some(_) if i > digits => {
        // (a unit of two letters, `true` before it; `fil`, `fill`, `filll` in glue)
        let letters: String = v[i..]
          .iter()
          .take_while(|t| t.get_catcode() == Catcode::LETTER)
          .take(6)
          .filter_map(text_of)
          .collect();
        let unit = if let Some(more) = letters.strip_prefix("fil") {
          3 + more.chars().take_while(|c| *c == 'l').count().min(2)
        } else if letters.starts_with("true") && letters.len() >= 6 {
          6
        } else if letters.len() >= 2 {
          2
        } else {
          return from;
        };
        // (and the one optional space after a unit, tex.web §443)
        i + unit + usize::from(v.get(i + unit) == Some(&T_SPACE!()))
      },
      _ => from,
    }
  };
  let mut end = dimen(0);
  if end == 0 {
    return 0;
  }
  if glue {
    for keyword in ["plus", "minus"] {
      let mut at = end;
      while v.get(at) == Some(&T_SPACE!()) {
        at += 1;
      }
      let word: String = v[at.min(v.len())..]
        .iter()
        .take(keyword.len())
        .filter_map(text_of)
        .collect();
      if word == keyword {
        let after = dimen(at + keyword.len());
        if after > at + keyword.len() {
          end = after;
        }
      }
    }
  }
  end
}

/// How many optional arguments the command `t` takes when it is a macro taking no other whose expansion prints no
/// word, only commands that print none ([`NON_PRINTING_ARGUMENTS`], an icon: fontawesome5's `\faGithub[]` is
/// `\faIcon[#1]{github}`; 2601.03661's `\faGithub \, \href{…}{…}`, 2601.06979's `\yticon~\href{…}{…}`, the r7
/// re-review's finding 5) and the groups, spaces and spacing around them; `None` for any other command, a macro that
/// expands to nothing among them.
fn icon_macro_options(t: &Token, depth: usize) -> Option<usize> {
  if depth > 3 {
    return None;
  }
  let definition = lookup_definition(t).ok()??;
  let Some(ExpansionBody::Tokens(body)) = definition.get_expansion() else {
    return None;
  };
  let parameters = definition
    .get_parameters()
    .map(|parameters| parameters.get_parameters())
    .unwrap_or_default();
  if parameters
    .iter()
    .any(|parameter| !parameter.optional && !parameter.novalue)
  {
    return None;
  }
  let v = body.unlist_ref();
  let mut icon = false;
  let mut i = 0;
  while i < v.len() {
    let u = &v[i];
    match u.get_catcode() {
      Catcode::BEGIN
      | Catcode::END
      | Catcode::SPACE
      | Catcode::PARAM
      | Catcode::ARG
      | Catcode::MARKER => i += 1,
      _ if is_spacing_command(u) || is_spacing(u) => i += 1,
      Catcode::CS => {
        let (end, _) = non_printing_end_within(v, i, depth + 1)?;
        icon = true;
        i = end.max(i + 1);
      },
      _ => return None,
    }
  }
  icon.then_some(
    parameters
      .iter()
      .filter(|parameter| parameter.optional)
      .count(),
  )
}

/// `v` without the arguments that print no letter ([`NON_PRINTING_ARGUMENTS`]), each command with them gone, or a
/// space where it prints one (a skip). What a command prints, its argument after them, stays.
fn without_non_printing_arguments(v: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    match non_printing_end(v, i) {
      Some((end, space)) => {
        if space {
          out.push(T_SPACE!());
        }
        i = end.max(i + 1);
      },
      None => {
        out.push(v[i]);
        i += 1;
      },
    }
  }
  out
}

/// `v` with each `\url`'s argument as the command prints it, verbatim: its braces and underscores characters
/// (`\{`, `\}`, `\_`), not a group and a subscript (`\texttt{\url{{co_gy,lyu,funakoshi,oku}@lr.first.iir.isct.ac.jp}}`,
/// a brace list of local parts, 2609.00588).
fn url_arguments_printed(v: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    out.push(v[i]);
    i += 1;
    if v[i - 1] != T_CS!("\\url") {
      continue;
    }
    while v.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    if v.get(i).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
      continue;
    }
    let close = skip_group(v, i).min(v.len());
    out.push(v[i]);
    for t in &v[i + 1..close.saturating_sub(1).max(i + 1)] {
      out.push(match t.get_catcode() {
        Catcode::BEGIN => T_CS!("\\{"),
        Catcode::END => T_CS!("\\}"),
        Catcode::SUB => T_CS!("\\_"),
        _ => *t,
      });
    }
    if close > i + 1 {
      out.push(v[close - 1]);
    }
    i = close;
  }
  out
}

/// Parse an email line into its individual address strings, or `None` if the line
/// is not an email list. Two forms are recognized, and mixed (after skipping
/// wrapper commands and grouping — the visible letter/other/space chars are read,
/// with the printed braces `\{ \}` and underscores `\_`):
/// - **Distributed** — `a@x, b@y, c@z`: every comma item carries its own `@`.
/// - **Grouped brace-expansion** — `\{a,b,c\}@dom`: the local parts in the braces are
///   each expanded against the domain after them → `a@dom, b@dom, c@dom`. This is the
///   compact form co-located authors use for one shared address per person; a line
///   may hold several groups (`{a,b}@x, {c,d}@y`, 2402.02746; `{a,b}@x, c@y`,
///   2410.19160).
///
/// A prose affiliation line ("Dept. of Foo, University of Pisa, Italy") has no `@`,
/// and a place before an address (`Berlin, Germany, foo@bar.de`) is a bare word, so
/// either is rejected and an address is never invented.
fn email_addresses(line: &Tokens) -> Option<Vec<String>> {
  // (a colour command's colour is no letter of the address: `\href{mailto:…}{\color{black}\texttt{xinhao.zhang},}`,
  // 2604.19440, read `blackxinhao.zhang@…`; a font's name, a scale, a skip's length neither, `\fontfamily{pcr}`,
  // 2609.25669; [`without_non_printing_arguments`])
  // (and a skip's length, printing a space: `xiaobing.gxb@alibaba-inc.com \hspace{25pt}zixuanze@andrew.cmu.edu`,
  // 2107.06049; `\vspace{-1.25ex}`, 2602.09494; a `\url`'s braces print, `\url{{a,b}@x}`, [`url_arguments_printed`])
  let printed = url_arguments_printed(&without_non_printing_arguments(line.unlist_ref()));
  let mut visible = String::new();
  // `\href`'s first argument is its link, not printed (`\href{mailto:a@x}{a@x}`).
  let mut skip_group = false;
  let mut depth = 0usize;
  // (a command's optional arguments are no text: `\makebox[\textwidth][c]{\tt haiyang.wang@huawei.com ~
  // lue.fan@ia.ac.cn}`, 2602.10975)
  let mut optional = false;
  let mut after_command = false;
  for t in &printed {
    if optional {
      optional = *t != T_OTHER!("]");
      continue;
    }
    if after_command && *t == T_OTHER!("[") {
      optional = true;
      continue;
    }
    after_command = t.get_catcode() == Catcode::CS || (after_command && *t == T_SPACE!());
    if skip_group {
      match t.code {
        Catcode::SPACE if depth == 0 => {},
        Catcode::OTHER if depth == 0 && *t == T_OTHER!("*") => {},
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
      // (a space, and the spacing that parts addresses printed in a row: `\texttt{jl-tong@sjtu.edu.cn}~~~~~
      // \texttt{xyshen@eitech.edu.cn}`, 2603.04592)
      Catcode::SPACE => visible.push(' '),
      Catcode::ACTIVE if *t == T_ACTIVE!('~') => visible.push(' '),
      Catcode::CS if is_spacing_command(t) || is_line_separator(t) => visible.push(' '),
      Catcode::CS if *t == T_CS!("\\href") => skip_group = true,
      // the printed braces of a list of local parts, and an underscore (`wu\_zhiliang`, 2509.10377)
      Catcode::CS => t.with_str(|name| match name {
        "\\{" | "\\lbrace" | "\\textbraceleft" => visible.push('{'),
        "\\}" | "\\rbrace" | "\\textbraceright" => visible.push('}'),
        "\\_" | "\\textunderscore" => visible.push('_'),
        _ => {},
      }),
      _ => {},
    }
  }
  let v = visible.trim();
  if !v.contains('@') {
    return None;
  }
  // Items at commas outside the braces: each a full address, or a braced list of local parts taking the domain after
  // its `}` (`{a,b,c}@dom` → a@dom, b@dom, c@dom). A bare word is no address, so a place before an address
  // (`Berlin, Germany, foo@bar.de`) leaves the line no email list.
  let is_part =
    |p: &str| !p.is_empty() && !p.contains(['@', '{', '}']) && !p.chars().any(char::is_whitespace);
  let mut out = Vec::new();
  let mut rest = v;
  while !rest.trim().is_empty() {
    rest = rest.trim_start();
    let (item, next) = match rest.strip_prefix('{') {
      Some(list) => {
        let close = list.find('}')?;
        let after = &list[close + 1..];
        // (the domain ends at a comma, a semicolon or a space: `{a,b}@x.edu \ \ \ {c,d}@y.edu`, 2603.24730)
        let (domain, next) = match after.find([',', ';', ' ']) {
          Some(end) => (
            &after[..end],
            after[end..].trim_start().trim_start_matches([',', ';']),
          ),
          None => (after, ""),
        };
        // (a sentence's `;` or `.` after it is no part of it: `\{lizixu.cs, …\}@gmail.com;`, 2605.24470)
        let domain = domain.trim().trim_end_matches([';', '.']);
        if !domain.starts_with('@') || !is_part(&domain[1..]) {
          return None;
        }
        // (a `;` parts them as a comma does: `\{hjjang; dychoi; …\}@vclab.kaist.ac.kr`, 2607.16926)
        for local in list[..close]
          .split([',', ';'])
          .map(str::trim)
          .filter(|p| !p.is_empty())
        {
          if !is_part(local) {
            return None;
          }
          out.push(format!("{local}{domain}"));
        }
        (None, next)
      },
      // (an item ends at a comma or a semicolon: `aable@x.edu; \{bbaker, ccole\}@y.edu`, 2604.10316; the r8
      // re-review's §B, probes t1, t2)
      None => {
        let (item, next) = rest.split_once([',', ';']).unwrap_or((rest, ""));
        (Some(item.trim().trim_start_matches("mailto:")), next)
      },
    };
    // (an item is taken whole, as the line prints it: 2410.07147's `vn72@cornell.edu\nsone sj597@…` has no space
    // between its addresses; addresses a space parts are items each, `a@x~~~b@y`, every one of them an address)
    if let Some(item) = item.filter(|item| !item.is_empty()) {
      for item in item.split_whitespace() {
        let item = item.trim_end_matches([';', '.']);
        // (a local part before the `@`: a handle, `@aable`, is no address, the r6 re-review's w4)
        if !item.contains('@') || item.starts_with('@') || item.contains(['{', '}']) {
          return None;
        }
        out.push(item.to_string());
      }
    }
    rest = next;
  }
  (!out.is_empty()).then_some(out)
}

/// Line role within a superscript-labeled author block (see `\lx@add@authors`).
#[derive(Clone, Copy, PartialEq)]
enum AuthorLineKind {
  Author,
  Affiliation,
  Email,
  /// An email line behind a label or the class's email command (`Email: \email{x}`, [`labelled_emails`]).
  LabelledEmail,
  /// A line that is only a link ([`is_link_line`]): a URL contact, or an affiliation when the link prints an institution
  /// ([`is_url_line`], [`url_contact_call`]).
  Link,
}

/// The creator a link line of an author block goes to ([`url_contact_call`]).
#[derive(Clone, Debug, PartialEq)]
enum LinkOwner {
  /// the whole block's (`label=addresses:block`, the shared creator of OXIDIZED_DESIGN #159; a sole author's)
  Block,
  /// the author being made, the last of the line's group
  Current,
  /// the line's group of that many names, as the group's affiliation lines are (`annotate=N`)
  Group(usize),
  /// the author a label of its own names (`label=…`, [`placed_label`]), given once the block's marked affiliations are:
  /// a marked block's link after them in its contacts, as the column prints it (the r5 re-review's u14)
  Author(String),
}

/// Whose a link line of a marked author block is ([`url_contact_call`]): in an `\and` group of authors while other
/// groups hold authors, that group's (its one author's, or its names' as its affiliation lines are), the link printed in
/// that column; else the block's when the call makes
/// the whole author list (`replace`, the document's `\author`), the project's page of 2605.06223, 2602.08439,
/// 2604.12896 (user ruling 2026-10-20, batch 77p r7); else, a per-author class's call, its author's. `kinds` are the
/// block's entries, `group_starts` where each group's begin, `index` the line's entry.
fn link_line_owner(
  kinds: &[AuthorLineKind],
  group_starts: &[usize],
  index: usize,
  replace: bool,
) -> LinkOwner {
  let (start, end) = group_bounds(group_starts, kinds.len(), index);
  let authors = |kinds: &[AuthorLineKind]| {
    kinds
      .iter()
      .filter(|kind| **kind == AuthorLineKind::Author)
      .count()
  };
  let here = authors(&kinds[start..end.max(start)]);
  if here == 1 && authors(kinds) > here {
    LinkOwner::Current
  } else if here > 1 && authors(kinds) > here {
    // (a column of several names: theirs, as the column's affiliation lines are, `Ann Able$^1$, Bob Baker$^1$ \\
    // $^1$Univ A \\ \url{…} \and Cat Cole$^2$ …`, the r5 re-review's u6m; ruling 9)
    LinkOwner::Group(here)
  } else if replace {
    LinkOwner::Block
  } else {
    LinkOwner::Current
  }
}

/// The calls placing a line under a row of names parted by space whose footnote symbols label the institutions under
/// them (a group of the unmarked branch kept from the names before it by its lines' marks, and one of its lines an
/// institution a symbol labels, [`labelled_institution`]: `Dinghong Song$^{\dagger}$ \ \ \ \ Zhiwei Xu$^\mathsection$ …
/// Dong Li$^{\dagger}$ \\ $^\mathsection$ Tsinghua University \ \ \ \ $^\dagger$ University of California, Merced \\
/// \texttt{dinghongsong21@gmail.com}`, 2601.02680, 2505.23260, 2608.12337, the r5 re-review's r10, r11, r14, r16, r18):
/// the lines print under the whole row, not under its last name. `row` is the row's names (indices into `name_keys`),
/// `name_symbols` the symbols each name of it carries (none for a name of another row: an `\and` column's carriers are
/// no part of it, the r6 re-review's x6, w28). Each address of an e-mail line goes to the name it spells anywhere in
/// the block, and one no name spells to the block (`label=addresses:block`, the shared creator of #159) — no
/// elimination; a link line is the block's; an institution a symbol labels goes to every name carrying it
/// (`label=author:N`), without the symbol, each such institution of the line its own (`$^\mathsection$ Tsinghua
/// University \ \ \ \ $^\dagger$ …`). An unmarked line goes on with the institution above it only where that line is
/// one such institution alone, ending in a comma, and it reads as an institution too (`continuing`, the carriers of
/// that one: `$^{\dagger}$State Key Laboratory …, \\ School of Computer Science, Peking University`, 2608.12337); an
/// unmarked institution otherwise is every name's of the row, as it prints under the whole row (user ruling R12
/// 2026-10-20: `… \\ Department of Physics`, the r6 re-review's y8); a legend goes to the names carrying its symbol;
/// any other piece — a note, a place, a symbol no name of the row carries, an address line no list reads — is the
/// block's (`Equal contribution`, the r6 re-review's y7; 2607.16926's address line under all five names) —
/// [`row_line_calls`]. None lands on one name for its place (batch 77p r9, r10). A line that prints nothing is none.
fn apart_line_calls(
  line: &Tokens,
  name_keys: &[NameKeys],
  name_symbols: &[Vec<String>],
  row: std::ops::RangeInclusive<usize>,
  placed: &mut Vec<Option<String>>,
  bodies: &mut [Vec<Token>],
  continuing: &mut Continuing,
) -> Result<Vec<Token>> {
  // (a name's label of its own, given to it in its content, [`placed_label`])
  let mut author = |owner: usize| -> Result<String> {
    let (label, call) = placed_label(placed, owner)?;
    if let Some(body) = bodies.get_mut(owner) {
      body.extend(call.unwrap_or_default());
    }
    Ok(s!("label={label}"))
  };
  let block = s!("label=addresses:block");
  let mut calls = Vec::new();
  if prints_nothing(line) {
    return Ok(calls);
  }
  // (the carriers the line above leaves this one to go on with: none unless set again below)
  let above = std::mem::take(&mut continuing.carriers);
  // (a mark after an address is no letter of it: `\texttt{mazeyuwhu@gmail.com}$^{\dagger}$`, 2109.13296, the address
  // of the name it spells; the r7 re-review's findings 1 and 9)
  let (addresses_line, trailing_carriers) = match before_trailing_symbol_mark(line.unlist_ref()) {
    Some((text, marks)) => (
      Tokens::new(text.to_vec()),
      symbol_carriers(&symbol_labels(&Tokens::new(marks.to_vec())), name_symbols),
    ),
    None => (line.clone(), Vec::new()),
  };
  if let Some((label, addresses)) = line_emails(&addresses_line)? {
    // (a line no address of which a name spells goes to the names carrying its marks, the marks no part of its addresses
    // — [`marked_email_line`] for those that lead it; main-session decision 3 on the 77p r11 questions, the r8
    // re-review's finding 6, probes s10, t4 —, else to the block; a spelled address keeps the mark it shows, 65e)
    let unspelled = addresses
      .iter()
      .all(|address| spelled_owner(name_keys, &address_text(&address.address)).is_none());
    let (label, addresses, carriers) = match (unspelled, marked_email_line(line, name_symbols)?) {
      (true, Some((stripped, carriers))) => match line_emails(&stripped)? {
        Some((label, addresses)) => (label, addresses, carriers),
        None => (label, addresses, Vec::new()),
      },
      (true, None) => (label, addresses, trailing_carriers),
      (false, _) => (label, addresses, Vec::new()),
    };
    for address in addresses {
      let owners = match spelled_owner(name_keys, &address_text(&address.address)) {
        Some(owner) => vec![owner],
        None => carriers.clone(),
      };
      if owners.is_empty() {
        calls.extend(email_calls(label.as_ref(), vec![address], Some(&block))?);
      } else {
        for owner in owners {
          let placement = author(owner)?;
          calls.extend(email_calls(
            label.as_ref(),
            vec![EmailAddress {
              address: address.address.clone(),
              call:    address.call.clone(),
            }],
            Some(&placement),
          )?);
        }
      }
    }
    return Ok(calls);
  }
  if is_url_line(line) {
    return url_contact_call(line.clone(), LinkOwner::Block, true);
  }
  calls.extend(row_line_calls(
    line,
    name_symbols,
    row,
    placed,
    bodies,
    &above,
    continuing,
  )?);
  Ok(calls)
}

/// An e-mail line's label and addresses: a labelled one's ([`line_labelled_emails`]), else a bare one's
/// ([`bare_email_addresses`]); `None` for any other line.
fn line_emails(line: &Tokens) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  Ok(match line_labelled_emails(line)? {
    Some(found) => Some(found),
    None => bare_email_addresses(line)?.map(|addresses| {
      (
        None,
        addresses
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
          .collect(),
      )
    }),
  })
}

/// `line` without the footnote symbol marks that lead it ([`after_symbol_mark`], each of a run).
fn without_leading_symbol_marks(line: &Tokens) -> Tokens {
  let mut after_marks: &[Token] = line.unlist_ref();
  while after_symbol_mark(after_marks).len() < after_marks.len() {
    after_marks = after_symbol_mark(after_marks);
  }
  Tokens::new(after_marks.to_vec())
}

/// An e-mail line its footnote symbol marks lead (`$^\dagger$zq77@x.edu`), when names of the row (`name_symbols`, the
/// symbols each carries, none for a name of another row) carry them: the line without them, and those names, whose an
/// address of it no name spells is (main-session decision 3 on the 77p r11 questions, as a marked line ties an
/// affiliation to its carriers, 63i; the r8 re-review's finding 6, probes s10, t4; as the numbered branch places one,
/// [`AddressPlacement::Marked`]: 2605.01818). `None` for any other line; a mark no name carries stays, as it prints.
fn marked_email_line(
  line: &Tokens,
  name_symbols: &[Vec<String>],
) -> Result<Option<(Tokens, Vec<usize>)>> {
  let stripped = without_leading_symbol_marks(line);
  if stripped.unlist_ref().len() == line.unlist_ref().len() {
    return Ok(None);
  }
  let carriers = symbol_carriers(&leading_symbol_labels(line), name_symbols);
  Ok(
    (!carriers.is_empty()
      && (line_labelled_emails(&stripped)?.is_some()
        || bare_email_addresses(&stripped)?.is_some()))
    .then_some((stripped, carriers)),
  )
}

/// The names a line under a row of symbol-marked names goes on with, from the line above it ([`row_line_calls`]).
struct Continuing {
  /// the carriers of the institution above, which the line goes on with; none unless set again by the line
  carriers:      Vec<usize>,
  /// whether the group's labelled institutions run on over unmarked lines ([`welded_layout`])
  welded:        bool,
  /// the group's lines holding an institution a symbol labels yet to come ([`labelled_line`]): past the last, a row
  /// with an unmarked name welds no line (user ruling R12)
  labelled_left: usize,
}

/// Whether `line` holds an institution a footnote symbol of the names (`name_symbols`) labels
/// ([`labelled_institution`]).
fn labelled_line(line: &Tokens, name_symbols: &[Vec<String>]) -> bool {
  symbol_led_segments(line)
    .iter()
    .any(|segment| labelled_institution(segment, name_symbols).is_some())
}

/// Whether an unmarked line may go on with the labelled institution above it where a group welds them
/// ([`welded_layout`]): it prints, holds no address (`@`) and is no link line, is no note — correspondence, a
/// contribution, an address, funding ([`classify_thanks`]) — and reads as an institution ([`reads_as_institution`]:
/// `Ball State University`) or a place ([`reads_as_place`]: `Muncie, IN 47306`), 2602.05059. Not `Corresponding Author:
/// \texttt{…}` (2604.02904), `Email: (…)@tu-bs.de, boche@tum.de` (2602.03421), `Preprint. Under review.`, `Work done
/// during an internship at Google`: the 77y review's finding 3, probes c1, c3, x4.
fn weldable_line(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  !prints_nothing(line)
    && !text.contains('@')
    && !is_link_line(line)
    && classify_thanks(&text) == "note"
    && (reads_as_institution(line) || reads_as_place(line))
}

/// Whether `line` reads as a place: no name ([`name_count`]), no address, label (`:`) or note ([`classify_thanks`]),
/// parts a comma parts, each word of them opening with a capital or a digit, or a particle ([`NAME_PARTICLES`])
/// (`Muncie, IN 47306`, `Braunschweig, Germany`, `Hong Kong SAR, China`, `Campus de Santiago, 3810-183 Aveiro,
/// Portugal`, 2605.19581).
fn reads_as_place(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim();
  text.contains(',')
    && !text.contains(['@', ':'])
    && name_count(text) == 0
    && classify_thanks(text) == "note"
    && text.split_whitespace().all(|word| {
      let word = word.trim_matches(|c: char| !c.is_alphanumeric());
      word.is_empty()
        || NAME_PARTICLES.contains(&word)
        || word
          .chars()
          .next()
          .is_some_and(|c| c.is_uppercase() || c.is_ascii_digit())
    })
}

/// Whether the lines under a row of symbol-marked names (`affils`, the names' symbols `name_symbols`) set each labelled
/// institution over unmarked lines of its own: an unmarked line that may go on with one ([`weldable_line`]: an
/// institution or a place) stands between two lines holding an institution a symbol labels ([`labelled_line`]) —
/// `$^\dagger$Department of Computer Science \\ Ball State University \\ Muncie, IN 47306 \\ … \\ $^\ddagger$School of
/// Artificial Intelligence …`, 2602.05059 (the r8 re-review's finding 5, probe t7). There such an unmarked line goes on
/// with the institution above it, as a numbered block welds it (x12, main-session decision 3 on the 77p r10 questions)
/// — a note, an address or a link line not (the 77y review's finding 3: 2604.02904, 2602.03421), nor, in a row with an
/// unmarked name, a line past the last labelled institution, which is every name's of the row (R12; probe c4); else
/// an unmarked institution under the last labelled one is every name's of the row (user ruling R12: `$^\dagger$Univ A
/// \\ $^\ddagger$Univ B \\ Department of Physics`, probe y8).
fn welded_layout(affils: &[Tokens], name_symbols: &[Vec<String>]) -> bool {
  let (mut seen_labelled, mut unmarked_since) = (false, false);
  for line in affils {
    if labelled_line(line, name_symbols) {
      if unmarked_since {
        return true;
      }
      seen_labelled = true;
    } else if seen_labelled && symbol_marks(line).is_empty() && weldable_line(line) {
      unmarked_since = true;
    }
  }
  false
}

/// The calls placing a line under a row of names that is no address and no link ([`apart_line_calls`]; and a line
/// under a row whose only symbol line is a legend, 2609.24299, 2603.06578): each symbol-led piece of it
/// ([`symbol_led_segments`]) that is an institution its symbol labels to the row's names carrying the symbol
/// (`name_symbols`, by index, none for a name of another row), without it; a legend (`$^*$Corresponding author: …`) to
/// the names carrying its symbol, whole, its mark naming them; an unmarked institution ([`reads_as_institution`]) to
/// the carriers of the line above when that was one labelled institution alone ending in a comma (`above`), else to
/// every name of the `row` (user ruling R12 2026-10-20); anything else — a note, a place, a symbol no name of the row
/// carries — to the block (`label=addresses:block`, the shared creator of #159), never to one name for its place. Where
/// the group's labelled institutions run on over unmarked lines (`continuing.welded`, [`welded_layout`]), an unmarked
/// institution or place ([`weldable_line`]) goes on with the carriers of the line above (2602.05059) — a note, an
/// address or a link line not (2604.02904, 2602.03421) — and past the group's last labelled institution only where
/// every name of the row carries a mark: under a row with an unmarked name an unmarked line under it goes on with it
/// only as one under an institution ending in a comma does, else it is every name's of the row (R12, literally; the
/// 77y review's finding 3, probe c4; the rev77y2 re-review's finding 1, probe w1). `continuing` is set to the carriers
/// a line under this one goes on with.
fn row_line_calls(
  line: &Tokens,
  name_symbols: &[Vec<String>],
  row: std::ops::RangeInclusive<usize>,
  placed: &mut Vec<Option<String>>,
  bodies: &mut [Vec<Token>],
  above: &[usize],
  continuing: &mut Continuing,
) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  let mut affiliation = |owner: Option<usize>, content: Tokens| -> Result<Vec<Token>> {
    let placement = match owner {
      Some(owner) => {
        // (a name's label of its own, given to it in its content, [`placed_label`])
        let (label, call) = placed_label(placed, owner)?;
        if let Some(body) = bodies.get_mut(owner) {
          body.extend(call.unwrap_or_default());
        }
        s!("label={label}")
      },
      None => s!("label=addresses:block"),
    };
    Ok(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        Some(mouth::tokenize_internal(TeXString::assembled(placement))),
        Some(content)
      ])
      .unlist(),
    )
  };
  let segments = symbol_led_segments(line);
  let alone = segments.len() == 1;
  if labelled_line(line, name_symbols) {
    continuing.labelled_left = continuing.labelled_left.saturating_sub(1);
  }
  // (a line past the group's last labelled institution welds on only where every name of the row carries a mark: one
  // under a row with an unmarked name is every name's of the row, R12 — the 77y review's finding 3, probe c4)
  let welds = continuing.welded
    && (continuing.labelled_left > 0
      || row.clone().all(|at| {
        name_symbols
          .get(at)
          .is_some_and(|symbols| !symbols.is_empty())
      }));
  for segment in segments {
    // (a mark anywhere in it, a group around it or after its text, makes it no unmarked line: the r7 re-review's
    // finding 1, 2606.28166, 2602.16540, 2109.13296)
    let unmarked = symbol_marks(&segment).is_empty();
    let (owners, content): (Vec<usize>, Tokens) = match labelled_institution(&segment, name_symbols)
    {
      Some((carriers, rest)) => {
        // (one institution alone on its line, ending in a comma, goes on on the line under it; so does any, where the
        // group's labelled institutions run on over unmarked lines, [`welded_layout`], but the last of them in a row
        // with an unmarked name, `welds`: `$^\ddagger$School of Law, University B \\ Department of Physics, University
        // C` gives the last line to every name of the row, R12 — the rev77y2 re-review's finding 1, probe w1)
        if alone
          && (welds
            || visible_name_text(rest.unlist_ref())
              .trim_end()
              .ends_with(','))
        {
          continuing.carriers = carriers.clone();
        }
        (carriers, rest)
      },
      // (an unmarked line under a labelled institution whose group runs its institutions on over unmarked lines goes on
      // with it, a place too, and so does the line under it: `$^\dagger$Department of Computer Science \\ Ball State
      // University \\ Muncie, IN 47306`, 2602.05059 — the x12 weld of a numbered block, main-session decision 3 on
      // the 77p r10 questions, applied to symbol rows; the r8 re-review's finding 5, probe t7) — an institution or a
      // place, no note, address or link line (`Corresponding Author: \texttt{…}`, 2604.02904; `Email: (…)@tu-bs.de,
      // boche@tum.de`, 2602.03421: the 77y review's finding 3)
      None if unmarked && alone && welds && !above.is_empty() && weldable_line(&segment) => {
        continuing.carriers = above.to_vec();
        (above.to_vec(), segment)
      },
      // (and an unmarked piece before the labelled institutions of its line, no note and no address, is the
      // institution of the names no mark sends elsewhere, as it prints in their row: `Uber Technologies \ \ \
      // $^\dagger$University of British Columbia`, 2603.07345 — 77p r11; not one `\quad` parts onto a line of its own,
      // which reads alone: 2509.20478's `UC Berkeley \quad $^\dagger$\unskip Princeton University`)
      None
        if unmarked
          && (reads_as_institution(&segment)
            || link_prints_an_institution(&segment)
            || (!alone && institution_beside_labelled(&segment))) =>
      {
        if alone && !above.is_empty() {
          (above.to_vec(), segment)
        } else {
          (row.clone().collect(), segment)
        }
      },
      // (a marked piece no institution reads: the names carrying every mark of it, else the block's — never the row's)
      None if !unmarked => (
        symbol_carriers(&symbol_labels(&segment), name_symbols),
        segment,
      ),
      None => (Vec::new(), segment),
    };
    if owners.is_empty() {
      calls.extend(affiliation(None, content)?);
    } else {
      for owner in owners {
        calls.extend(affiliation(Some(owner), content.clone())?);
      }
    }
  }
  Ok(calls)
}

/// Whether an unmarked piece of a line of symbol-led pieces ([`symbol_led_segments`]) reads as their fellow
/// institution: letters, no address, no note — correspondence, a contribution, funding ([`classify_thanks`]) — and no link
/// line (`Uber Technologies` beside `$^\dagger$University of British Columbia`, 2603.07345).
fn institution_beside_labelled(segment: &Tokens) -> bool {
  let text = visible_name_text(segment.unlist_ref());
  has_name_letter(segment.unlist_ref())
    && !text.contains('@')
    && classify_thanks(&text) == "note"
    && !is_link_line(segment)
}

/// Whether the institution `under` goes on with the words of `line` above it as one name of one institution, no
/// institution of its own under a line of names: it is one generic institution noun ([`NON_NAME_WORDS`]: `Shanghai Jiao
/// Tong \\ University`), opens with a connective (`… \\ of Science and Technology`), or holds every word of the line
/// (`Hong Kong \\ The Hong Kong University of Science and Technology`) — the rev77y2 re-review's finding 4, probes k12,
/// k17. An institution's own name is none (`Cat Cole \\ CERN`, `\\ RIKEN`, `\\ the University of Tokyo`, `\\ eBay
/// Inc.`, `\\ Max-Planck-Institut`: the rev77y3 re-review's finding 1, probes k20, k26, k31, k40-k42; rev77y4's, c2,
/// c5). A name over a one-word generic noun or a connective-led institution of its own (`Cat Cole \\ Physics`, `\\ at
/// Univ C`) is read as the line's continuation — a SYNC residual (rev77y4 probes c1, c3, c4, c8, k27, k30).
fn continues_the_line(line: &Tokens, under: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  let under = visible_name_text(under.unlist_ref());
  let words: Vec<&str> = under.split_whitespace().collect();
  // (the whole word, never a hyphen part of an institution's own name: `Max-Planck-Institut`, `Helmholtz-Zentrum` — the
  // rev77y4 re-review's finding 1, probes c2, c5)
  let generic_noun = |word: &str| {
    word_forms(word)
      .next()
      .is_some_and(|bare| NON_NAME_WORDS.contains(&bare.as_str()))
  };
  matches!(words.as_slice(), [word] if generic_noun(word))
    || words
      .first()
      .is_some_and(|word| ["of", "and", "for", "at", "in"].contains(word))
    || text.split_whitespace().all(|word| words.contains(&word))
}

/// Whether `under` is an e-mail line holding an address the names of `line` spell ([`spelled_owners`]): `Cat Cole \\
/// \texttt{ccole@x.edu}` — the rev77y2 re-review's finding 4, probe k15.
fn addresses_spelling(line: &Tokens, under: &Tokens) -> bool {
  let keys = [NameKeys::of(line)];
  email_addresses(under).is_some_and(|addresses| {
    addresses
      .iter()
      .any(|address| !spelled_owners(&keys, address).is_empty())
  })
}

/// Whether `line` reads as an institution: as an affiliation ([`reads_as_affiliation`]: an institution word, no name,
/// no `@`), and no note — correspondence, a contribution, an address, funding ([`classify_thanks`]).
fn reads_as_institution(line: &Tokens) -> bool {
  reads_as_affiliation(line) && classify_thanks(&visible_name_text(line.unlist_ref())) == "note"
}

/// The footnote symbols [`symbol_marks`] reads in `tokens`, one each of a run of distinct ones (`$^{\dagger\ddagger}$`:
/// two, 2608.12337), as the labels they request are read ([`clean_frontmatter_labels`]).
fn symbol_labels(tokens: &Tokens) -> Vec<String> {
  symbol_marks(tokens)
    .iter()
    .flat_map(|mark| clean_frontmatter_labels(mark, ""))
    .collect()
}

/// A piece of a line ([`symbol_led_segments`]) that is an institution its footnote symbol labels: led by a symbol mark
/// some names carry (`name_symbols`), the rest no note — not correspondence, a contribution, an address or funding
/// ([`classify_thanks`]), and no e-mail address (`$^\mathsection$ Tsinghua University`, `$^{\ddagger}$LLM-Core Xiaomi`,
/// 2601.02680, 2608.12337; not `$^*$ Corresponding author`) —: the names carrying the symbol, and the piece without it.
fn labelled_institution(
  segment: &Tokens,
  name_symbols: &[Vec<String>],
) -> Option<(Vec<usize>, Tokens)> {
  let after = after_symbol_mark(segment.unlist_ref());
  let (rest, marks) = if after.len() == segment.unlist_ref().len() {
    // (a mark after its text labels it as one before it does: `Carnegie Mellon University, …$^{\dagger}$`, 2109.13296;
    // not a text holding marks of its own, a row of marked names, `\textbf{~Pavan Thodima$^{\dagger}$, Aniket
    // Rege$^{\dagger}$}`, 2601.18077, whose last mark is a name's, no line's label)
    let (text, marks) = before_trailing_symbol_mark(segment.unlist_ref())?;
    if !symbol_marks(&Tokens::new(text.to_vec())).is_empty() {
      return None;
    }
    (
      Tokens::new(text.to_vec()),
      symbol_labels(&Tokens::new(marks.to_vec())),
    )
  } else {
    (Tokens::new(after.to_vec()), leading_symbol_labels(segment))
  };
  if marks.is_empty() {
    return None;
  }
  let text = visible_name_text(rest.unlist_ref());
  // (under the names, a symbol-led line is no name: `$^{\ddagger}$LLM-Core Xiaomi` reads as one)
  let institution = reads_as_affiliation(&rest)
    || (classify_thanks(&text) == "note"
      && !text.contains('@')
      && has_name_letter(rest.unlist_ref()));
  let carriers = symbol_carriers(&marks, name_symbols);
  (institution && !carriers.is_empty()).then_some((carriers, rest))
}

/// The labels of the footnote symbol marks that lead `segment` before its text ([`after_symbol_mark`], each mark of a
/// run, [`symbol_marks`]; one each of a compound mark's symbols, [`clean_frontmatter_labels`]: `$^{*\dag}$Project lead:
/// …` gives `*` and `\dag`, 2602.19190).
fn leading_symbol_labels(segment: &Tokens) -> Vec<String> {
  // (each symbol, not a run whole: [`clean_frontmatter_labels`] gives both)
  let mut labels: Vec<String> = Vec::new();
  let mut v: &[Token] = segment.unlist_ref();
  loop {
    let rest = after_symbol_mark(v);
    if rest.len() == v.len() {
      break;
    }
    let lead = Tokens::new(v[..v.len() - rest.len()].to_vec());
    // (the run whole is the first label of several; a doubled mark, `$^{**}$`, `$^{\dagger\dagger}$`, is one label,
    // its own: the r7 re-review's finding 3, probes n1, n2 — a comma list, `$^{\dagger,\ddagger}$`, has no run whole,
    // its first label a symbol of its own: the r8 re-review's finding 10)
    for label in symbol_marks(&lead).iter().flat_map(|mark| {
      let labels = clean_frontmatter_labels(mark, "");
      let skip = usize::from(
        labels.len() > 1
          && labels
            .first()
            .is_some_and(|run| SYMBOL_COMMAND_RUN.is_match(&run.replace('_', " "))),
      );
      labels.into_iter().skip(skip)
    }) {
      if !labels.contains(&label) {
        labels.push(label);
      }
    }
    v = rest;
  }
  labels
}

/// The names (by index into `name_symbols`, the symbols each carries) carrying every one of `marks`: a line's compound
/// mark names those marked with all its symbols (`$^{*\dag}$Project lead: Yi Yang` is Yi Yang's, `$^{*\dag}$`, not
/// every starred name's, 2602.19190), a single symbol every name carrying it (`$^\dagger$` reaches a name marked
/// `$^{\dagger\ddagger}$`, 2608.12337).
fn symbol_carriers(marks: &[String], name_symbols: &[Vec<String>]) -> Vec<usize> {
  (0..name_symbols.len())
    .filter(|at| !marks.is_empty() && marks.iter().all(|mark| name_symbols[*at].contains(mark)))
    .collect()
}

/// The pieces of `line` each footnote symbol mark ([`keepsup`]) leads, at the top level after a space or spacing
/// (`$^\mathsection$ Tsinghua University \ \ \ \ $^\dagger$ University of California, Merced`: two), without the spacing
/// around them; the line whole when none does.
fn symbol_led_segments(line: &Tokens) -> Vec<Tokens> {
  // (through a group around the whole line, and at a group a mark opens: `{\small $^{\dagger}$\,University of
  // Toronto \qquad $^{\ddagger}$\,EPFL}`, 2606.28166; the r7 re-review's finding 1, probes g1, g3)
  let v = unwrapped(line.unlist_ref());
  let mut cuts = vec![0];
  let mut depth = 0usize;
  for (i, t) in v.iter().enumerate() {
    let after_space = i > 0 && (v[i - 1] == T_SPACE!() || is_spacing(&v[i - 1]));
    if depth == 0
      && after_space
      && (*t == T_CS!("\\lx@frontmatter@keepsup")
        || (t.get_catcode() == Catcode::BEGIN
          && after_leading_markup(&v[i..]).first() == Some(&T_CS!("\\lx@frontmatter@keepsup"))))
    {
      cuts.push(i);
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
  }
  cuts.push(v.len());
  // (a piece of marks alone is no piece: the mark after a space ends the text before it, which it labels,
  // [`before_trailing_symbol_mark`] — `Univ A $^{\dagger}$`, the r8 re-review's finding 7, probe s3)
  let marks_only = |piece: &[Token]| {
    let mut after = piece;
    while after_symbol_mark(after).len() < after.len() {
      after = after_symbol_mark(after);
    }
    after.len() < piece.len() && spacing_end(after) == 0
  };
  let mut at = 1;
  while at + 1 < cuts.len() {
    if marks_only(&v[cuts[at]..cuts[at + 1]]) {
      cuts.remove(at);
    } else {
      at += 1;
    }
  }
  cuts
    .windows(2)
    .filter_map(|pair| {
      let piece = &v[pair[0]..pair[1]];
      let piece = &piece[leading_spacing_end(piece)..];
      let mut end = spacing_end(piece);
      // (and the run that parts it from the next, `\ \ \ \ `)
      while end > 0 && is_spacing(&piece[end - 1]) {
        end -= 1;
      }
      let piece = unwrapped(&piece[..end]);
      (!piece.is_empty()).then(|| Tokens::new(piece.to_vec()))
    })
    .collect()
}

/// Whether a name's content holds a command that makes its creator itself (`\lx@add@creator`: PoS's `\speaker{Piotr
/// Korcyl}`, 1307.5080), which a label of the `\lx@add@author` around it would not reach ([`placed_label`]): a contact of
/// that name is placed in its content, as the name is made.
fn makes_its_creator(name: &[Token]) -> Result<bool> {
  for t in name.iter().filter(|t| t.get_catcode() == Catcode::CS) {
    if let Some(def) = lookup_definition(t)?
      && let Some(ExpansionBody::Tokens(body)) = def.get_expansion()
      && body.unlist_ref().first() == Some(&T_CS!("\\lx@add@creator"))
    {
      return Ok(true);
    }
  }
  Ok(false)
}

/// The label of the author `author` of a block (an index among its names) that a contact placed on it after the names
/// are made reaches it by, `labels` holding those given so far: a new one, `author:placed<N>` (counted over the
/// document, so no two blocks share one; `author:` so no misprefixed institute label's number reaches it), given to the
/// author by `\lx@author@placed@label` in its content (`label`, the call to add there, `Some` when new). Not the
/// `author:N` sequence label: a block whose content makes creators of its own (PoS's `\speaker`, 1307.5080) numbers them
/// otherwise.
fn placed_label(
  labels: &mut Vec<Option<String>>,
  author: usize,
) -> Result<(String, Option<Vec<Token>>)> {
  if labels.len() <= author {
    labels.resize(author + 1, None);
  }
  if let Some(label) = &labels[author] {
    return Ok((label.clone(), None));
  }
  let count = lookup_int("lx_author_placed_labels") + 1;
  assign_value(
    "lx_author_placed_labels",
    Stored::Int(count),
    Some(Scope::Global),
  );
  let label = s!("author:placed{count}");
  labels[author] = Some(label.clone());
  let call = Invocation!(T_CS!("\\lx@author@placed@label"), vec![Some(
    mouth::tokenize_internal(TeXString::assembled(label.clone()))
  )])
  .unlist();
  Ok((label, Some(call)))
}

/// Where the `\and` group of a marked author block's entry `index` begins and ends among its `count` entries
/// (`group_starts`, where each begins).
fn group_bounds(group_starts: &[usize], count: usize, index: usize) -> (usize, usize) {
  let group = group_starts
    .iter()
    .rposition(|start| *start <= index)
    .unwrap_or(0);
  let start = group_starts.get(group).copied().unwrap_or(0);
  let end = group_starts.get(group + 1).copied().unwrap_or(count);
  (start, end.max(start))
}

/// The call making a link line of an author block ([`is_link_line`]) a URL contact (`\lx@add@contact[role=url]`, as a
/// class's `\homepage` is), not an affiliation: the line is its content, a label before the link with it (`Demo
/// Code:~\url{…}`, 2602.09509), in the author block's markup as an affiliation line is (`\lx@affiliation@withinst`);
/// placed as `owner` says (user ruling 2026-10-20, batch 77p r7). A link printing an institution
/// ([`is_url_line`] false: `\href{https://launchsafe.com}{\textbf{LaunchSafe Research}}`, 2606.26479) is an affiliation
/// placed so, the link inside it (`url` false; the r5 re-review's u38).
fn url_contact_call(line: Tokens, owner: LinkOwner, url: bool) -> Result<Vec<Token>> {
  let placement = match owner {
    LinkOwner::Block => s!("label=addresses:block"),
    LinkOwner::Current => String::new(),
    LinkOwner::Group(names) => s!("annotate={names}"),
    LinkOwner::Author(label) => s!("label={label}"),
  };
  if !url {
    return Ok(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        (!placement.is_empty()).then(|| mouth::tokenize_internal(TeXString::assembled(placement))),
        Some(line)
      ])
      .unlist(),
    );
  }
  let keyvals = if placement.is_empty() {
    s!("role=url")
  } else {
    s!("role=url,{placement}")
  };
  let mut content = mouth::tokenize_internal(
    r"\lx@author@markup@begin\expandafter\lx@author@markup@run\expandafter{\lx@author@markup@unprovide}",
  )
  .unlist();
  content.push(T_BEGIN!());
  // (a link that prints nothing is none of it: 2601.02680's `\url{}` after its repository link)
  let v = line.unlist_ref();
  let mut i = 0;
  while i < v.len() {
    if v[i] == T_CS!("\\url") || v[i] == T_CS!("\\href") {
      let (_, (_, after_second)) = link_arguments(v, i);
      let (open, close) = link_printed_argument(v, i);
      let end = if v[i] == T_CS!("\\href") {
        after_second
      } else {
        close
      };
      if v[open..close].iter().all(|t| {
        matches!(
          t.get_catcode(),
          Catcode::BEGIN | Catcode::END | Catcode::SPACE
        )
      }) {
        i = end.max(i + 1);
        continue;
      }
    }
    content.push(v[i]);
    i += 1;
  }
  content.push(T_END!());
  Ok(
    Invocation!(T_CS!("\\lx@add@contact"), vec![
      Some(mouth::tokenize_internal(TeXString::assembled(keyvals))),
      Some(Tokens::new(content))
    ])
    .unlist(),
  )
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
    // the case changes the character, not the token's out-of-band field provenance (`Token::prov`)
    Token {
      prov: token.prov,
      ..Token::new(result, token.get_catcode())
    }
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
    // (a list environment, no TeX box: its text is body text for a field's print, OXIDIZED_DESIGN #715)
    for mut node in insert_block_as(document, body, HashMap::default(), false, false)?.into_iter() {
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
  let name = T_CS!(cs);
  matches!(lookup_definition(&name), Ok(Some(def)) if is_name_noun(&def, &mut vec![name]))
}

/// Is this definition a NAME NOUN (`\figurename`): an expandable macro whose arguments are
/// all optional? ltcmd's `\NewDocumentCommand` defines `\foo` as the protected,
/// parameterless dispatcher `\__cmd_start_optimized: \foo code` (latex.ltx:1963-1972),
/// so follow that one hop to `\foo code` (zero-arg xparse nouns qualify, `m`-taking
/// drawing commands like argumentation.sty:403 `\afname` do not); the general
/// `\__cmd_start:nNNnnn` / `\__cmd_start_expandable:nNNNNn` dispatchers carry the
/// signature as written, which is read; a `\DeclareRobustCommand` wrapper is followed to
/// its inner macro, and a `\protect\<cs>` body to `\<cs>` (`\protect` is `\relax` in a typeset tag,
/// latex.ltx `\@typeset@protect`): `\renewcommand\figurename{\protect\rb}` with a robust `\rb` is two hops,
/// the body to the wrapper and the wrapper to its inner macro, and was cut at the first, so the tag lost its name
/// (pdflatex "Fx 1", ours "1"; `[figurename=\rb]{caption}` stores the same body). `visited` holds the macros
/// followed so far: a hop back to one of them is a loop (`\def\x{\protect\x}`, on which pdflatex never ends) and
/// is not a name; a chain of any length otherwise is followed (pdflatex "Ex 1" through five `\protect` hops), with a
/// fuse of `MAX_HOPS` distinct macros. Guard `perfect_kernel_batch68h::protected_robust_name_is_a_name_noun`.
fn is_name_noun(def: &Rc<dyn Definition>, visited: &mut Vec<Token>) -> bool {
  const MAX_HOPS: usize = 32;
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
  let mut follow = |next: &Token| {
    if visited.len() >= MAX_HOPS || visited.contains(next) {
      return false;
    }
    visited.push(*next);
    matches!(lookup_definition(next), Ok(Some(next_def)) if is_name_noun(&next_def, visited))
  };
  if head == "\\protect" && toks.len() == 2 {
    return follow(&toks[1]);
  }
  if head == "\\__cmd_start_optimized:" {
    return toks.get(1).is_some_and(follow);
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

/// An ORCID iD written as its orcid.org address — `https://orcid.org/<iD>`, `http://`, `www.`, or the bare
/// `orcid.org/<iD>`, a trailing `/` — is the iD; anything else, and an argument holding a macro, is kept as written.
fn orcid_id(id: Tokens) -> Tokens {
  if id
    .unlist_ref()
    .iter()
    .any(|t| t.get_catcode() == Catcode::CS)
  {
    return id;
  }
  let text = id.to_string();
  let trimmed = text.trim();
  let lower = trimmed.to_ascii_lowercase();
  let mut rest = lower.as_str();
  for scheme in ["https://", "http://"] {
    if let Some(r) = rest.strip_prefix(scheme) {
      rest = r;
      break;
    }
  }
  rest = rest.strip_prefix("www.").unwrap_or(rest);
  match rest.strip_prefix("orcid.org/") {
    Some(r) => {
      let start = trimmed.len() - r.len();
      let bare = trimmed[start..].trim_end_matches('/');
      Tokens::new(
        bare
          .chars()
          .map(|c| {
            if c == ' ' {
              T_SPACE!()
            } else {
              T_OTHER!(c.to_string())
            }
          })
          .collect(),
      )
    },
    None => id,
  }
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

  /// The replayed statements of a title code `body`, as strings (`definitions_at_author`).
  fn replayed(body: &'static str) -> Vec<String> {
    definitions_at_author(tk(body).unlist_ref())
      .expect("reads \\@author")
      .into_iter()
      .map(|(_, statement)| Tokens::new(statement).to_string())
      .collect()
  }

  #[test]
  fn title_code_definitions_open_at_author() {
    setup();
    // a spaced `\newcommand [1] {…}` is read whole; `\let`; the last of two `\def`s comes last; a group closed before
    // `\@author` is not replayed
    assert_eq!(
      replayed(
        r"{\let\addr\itshape\newcommand\note [1] {[#1]}\def\dup{a}{\def\dup{b}\begingroup\def\gone{G}\endgroup\@author}}"
      ),
      [
        r"\let\addr\itshape",
        r"\newcommand\note[1] {[#1]}",
        r"\def\dup{a}",
        r"\def\dup{b}"
      ]
    );
  }

  #[test]
  fn title_code_definitions_global_local_and_skipped() {
    setup();
    // a `\gdef` (and `\global\def`) made in a closed group is kept, replayed as a local `\def`; `\edef`,
    // `\renewcommand` and a definition closing a group it did not open (2601.13359) are not replayed
    assert_eq!(
      replayed(
        r"{\gdef\g{1}\global\def\h{2}}{\edef\e{3}\renewcommand\r{4}\def\affiliations{\egroup\par\bgroup}\let\x\egroup\def\sep{\end{tabular}\begin{tabular}{c}}\def\ok{\begin{center}c\end{center}}\@author}"
      ),
      [
        r"\def\g{1}",
        r"\def\h{2}",
        r"\def\ok{\begin{center}c\end{center}}"
      ]
    );
    assert!(definitions_at_author(tk(r"{\def\name{N}}").unlist_ref()).is_none());
  }

  /// The name and trailing email `name_and_trailing_email` reads in `line`, as strings.
  fn name_email(line: &'static str) -> (String, Option<String>) {
    let (name, email) = name_and_trailing_email(&tk(line)).expect("reads the line");
    (name.to_string(), email.map(|e| e.to_string()))
  }

  #[test]
  fn name_line_trailing_email_splits_the_address() {
    setup();
    // the JMLR family's name line, its markup with the address; a line break and spacing after it (2409.06765)
    assert_eq!(
      name_email(r"\name Ann Able \email ann@uni.edu"),
      (
        r"\nameAnn Able".to_string(),
        Some(r"\emailann@uni.edu".to_string())
      )
    );
    assert_eq!(
      name_email(r"Ann Able$^{1}$ \texttt{ann\_a@uni.edu}\\ \vspace{-0.6cm}"),
      (
        r"Ann Able$^{1}$".to_string(),
        Some(r"\texttt{ann\_a@uni.edu}".to_string())
      )
    );
  }

  #[test]
  fn name_line_trailing_email_keeps_other_lines_whole() {
    setup();
    // a note's address, glued to the surname (2402.08164) or not; an address alone, a bracketed one, no address
    for line in [
      r"Binghui Peng\thanks{bp2601@columbia.edu}",
      r"Ann Able \thanks{ann@uni.edu}",
      r"Ann Able \footnote{ann@uni.edu}",
      r"ann@uni.edu",
      r"Ann Able (ann@uni.edu)",
      r"Ann Able",
    ] {
      assert_eq!(name_email(line).1, None, "{line}");
    }
    assert_eq!(spacing_end(tk(r"\vspace*{-1cm}\\ \hfill").unlist_ref()), 0);
  }

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

  #[test]
  fn is_link_line_bare_trailing_href() {
    setup();
    // a bare `\href` ending the line, or one whose second argument is missing, reads no argument past the line (was a
    // slice past its end, 77p r4 re-review) and prints nothing, so is no link line (77p r9: 2601.02680's `\url{}`); a
    // whole link is a link line, a line of spacing alone none
    for (line, link) in [
      ("\\href", false),
      ("\\href ", false),
      ("\\href{https://x.org}", false),
      ("\\href{https://x.org}{x.org} \\url{}", true),
      ("\\small \\href{https://x.org}{x.org}", true),
      ("\\url{https://x.org} text", false),
      ("\\vspace{2mm}", false),
      // (in groups and sizes, after a label ending in a colon: 2602.22394, 2602.23802, 2602.09509, 77p r6)
      ("{\\footnotesize \\url{https://x.org}}", true),
      ("{\\tt{\\small{\\url{https://x.org}}}}", true),
      ("Demo Code:~\\url{https://x.org}", true),
      ("Demo Code \\url{https://x.org}", false),
      ("\\url{https://x.org} \\url{https://y.org}", false),
    ] {
      assert_eq!(is_link_line(&tk(line)), link, "{line}");
    }
  }

  #[test]
  fn dotted_initials_are_no_web_address() {
    // (77p r11, the r7 re-review's finding 2: 2601.06085, 2606.03745)
    for (text, web) in [
      ("Edward A.G. Schuur", false),
      ("T.J.C. Bezerra", false),
      ("J.Smith", false),
      ("version 1.2", false),
      ("x.org", true),
      ("ai.meta.com", true),
      ("github.io/traj_evo_search", true),
      ("www.x", true),
      ("https://a.b", true),
      ("aable@x.edu", false),
    ] {
      assert_eq!(reads_as_web_address(text), web, "{text}");
    }
  }

  #[test]
  fn is_url_line_not_a_mailbox() {
    setup();
    // a link line to a web page is a URL contact, one to a mailbox not, an e-mail line ([`mailto_line`]; 2601.00240's
    // `\textbf{Correspondence:} \href{mailto:…}{…}`, 77p r7, r8)
    for (line, url) in [
      ("Code:~\\url{https://x.org}", true),
      ("\\href{https://x.org}{x.org}", true),
      (
        "\\textbf{Correspondence:} \\href{mailto:a@x.org}{a@x.org}",
        false,
      ),
      ("Univ A", false),
      // (77p r9: a phrase, a colour around the link, an empty `\url{}` beside it; a linked institution or name is
      // none, nor a link to a bare address, 2606.26479, 2604.04847)
      ("\\href{https://x.org}{Project page}", true),
      ("{\\color{orange}\\textbf{\\url{https://x.org}}}", true),
      ("\\href{https://x.org}{x.org} \\url{}", true),
      (
        "\\href{https://launchsafe.com}{\\textbf{LaunchSafe Research}}",
        false,
      ),
      ("\\href{https://www.mit.edu}{MIT}", false),
      ("\\href{https://x.org}{Davide Grossi}", false),
      ("\\textbf{Correspondence:} \\href{b@x.org}{b@x.org}", false),
      // (77p r10: the link's wording and target decide, not the shape of its words — a link word is a page
      // (2602.22227, 2601.18810), a bare domain an institution (the r6 re-review's x10, w8), a handle's profile a page
      // (w21), a person's name that name (2603.22058), an icon a link)
      ("\\href{https://x.org/p}{Project Page}", true),
      (
        "\\href{https://x.substack.com/p/a}{\\textcolor{blue}{Substack Article}}",
        true,
      ),
      ("\\href{https://github.com/x/y}{GitHub Repository}", true),
      ("\\href{https://huggingface.co/x}{Hugging Face Demo}", true),
      ("\\href{https://www.google.com}{Google}", false),
      ("\\href{https://x.ai/}{xAI}", false),
      ("Contact: \\href{https://x.com/aable}{@aable}", true),
      (
        "\\textit{Author:} \\href{https://x.org/ms}{Masashi Sekine}",
        false,
      ),
      (
        "\\href{https://github.com/x}{\\includegraphics{gh.png}}",
        true,
      ),
    ] {
      assert_eq!(is_url_line(&tk(line)), url, "{line}");
    }
  }

  #[test]
  fn bare_address_needs_a_local_part_and_a_dotted_domain() {
    // (77p r10, the r6 re-review's finding 6: a handle, a list, a web page are no one address)
    for (text, bare) in [
      ("binzhu@smu.edu.sg", true),
      ("@aable", false),
      ("aable@localhost", false),
      ("{a,b}@x.edu", false),
      ("x.com/aable@b.org", false),
    ] {
      assert_eq!(is_bare_address(text), bare, "{text}");
    }
    assert!(!links_a_mailbox("https://x.com/aable", "@aable"));
    assert!(links_a_mailbox("mailto:a@x.org", "a@x.org"));
  }

  #[test]
  fn non_printing_arguments_are_no_letters() {
    setup();
    // every command of the one table ([`NON_PRINTING_ARGUMENTS`]) drops its non-printing arguments and keeps what it
    // prints (77p r10, the r6 re-review's finding 5: 2609.25669's `\fontfamily{pcr}`, w14's `\scalebox{0.9}`)
    for (line, printed) in [
      ("\\color[rgb]{0,0,1}aable", "aable"),
      ("\\textcolor{blue}{aable}", "aable"),
      ("\\colorbox{red}{aable}", "aable"),
      ("\\fcolorbox[rgb]{1,0,0}[gray]{0.5}{aable}", "aable"),
      ("\\pagecolor{white}aable", "aable"),
      ("\\colorlet{mine}[rgb]{red}aable", "aable"),
      ("\\definecolor{mine}{rgb}{1,0,0}aable", "aable"),
      ("\\fontfamily{pcr}\\selectfont aable", "aable"),
      ("\\fontseries{b}aable", "aable"),
      ("\\fontshape{it}aable", "aable"),
      ("\\fontencoding{T1}aable", "aable"),
      ("\\fontsize{9}{11}aable", "aable"),
      ("\\usefont{T1}{pcr}{m}{n}aable", "aable"),
      ("\\scalebox{0.9}[0.8]{aable}", "aable"),
      ("\\rotatebox[origin=c]{90}{aable}", "aable"),
      ("\\raisebox{-1pt}[0pt][0pt]{aable}", "aable"),
      ("\\resizebox*{2cm}{!}{aable}", "aable"),
      ("\\includegraphics*[scale=0.1]{icon.pdf}aable", "aable"),
      ("x\\hspace*{2pt}aable", "x aable"),
      ("x\\vspace{1ex}aable", "x aable"),
      // (77p r11, the r7 re-review's finding 11: probes p1, p2, p4, p5, p6)
      ("\\rule{0pt}{2ex}aable", "aable"),
      ("\\rule[-1pt]{1em}{1em}aable", "aable"),
      ("x\\phantom{xx}aable", "x aable"),
      ("x\\hphantom{xx}aable", "x aable"),
      ("\\vphantom{Xy}aable", "aable"),
      ("x\\kern2em aable", "x aable"),
      ("x\\kern-1.5pt aable", "x aable"),
      ("x\\hskip 2em plus 1fil aable", "x aable"),
      ("x\\hskip\\fill aable", "x aable"),
      ("x\\vskip 1ex minus 2pt aable", "xaable"),
      ("x\\kern2emaable", "x aable"),
    ] {
      let text: String = without_non_printing_arguments(tk(line).unlist_ref())
        .iter()
        .filter(|t| {
          matches!(
            t.get_catcode(),
            Catcode::LETTER | Catcode::OTHER | Catcode::SPACE
          )
        })
        .map(|t| t.to_string())
        .collect();
      assert_eq!(text, printed, "{line}");
    }
    assert_eq!(
      email_addresses(&tk(
        "\\texttt{\\fontfamily{pcr}\\selectfont xw109@illinois.edu, tunder@illinois.edu}"
      )),
      Some(vec![s!("xw109@illinois.edu"), s!("tunder@illinois.edu")])
    );
    // (a `\url`'s braces print: a brace list of local parts, 2609.00588; a `;` parts them, 2607.16926)
    assert_eq!(
      email_addresses(&tk("\\texttt{\\url{{co_gy,lyu}@x.ac.jp}}")),
      Some(vec![s!("co_gy@x.ac.jp"), s!("lyu@x.ac.jp")])
    );
    assert_eq!(
      email_addresses(&tk("{\\tt\\small \\{hjjang; dychoi\\}@vclab.kaist.ac.kr}")),
      Some(vec![
        s!("hjjang@vclab.kaist.ac.kr"),
        s!("dychoi@vclab.kaist.ac.kr")
      ])
    );
  }

  #[test]
  fn empty_marks_and_compound_symbol_labels() {
    setup();
    // an empty mark is no mark (user ruling R11 2026-10-20, 2604.19440)
    assert_eq!(
      drop_empty_marks(tk("Ann Able \\\\ \\textsuperscript{}Univ A $^{ }$B")).to_string(),
      tk("Ann Able \\\\ Univ A B").to_string()
    );
    assert_eq!(
      drop_empty_marks(tk("Ann$^{1}$")).to_string(),
      tk("Ann$^{1}$").to_string()
    );
    // (in a block other marks make a marked one it stays: 2604.22129)
    let marked = "Ann$^{1}$~~~Bob\\textsuperscript{}";
    assert_eq!(
      drop_empty_marks(tk(marked)).to_string(),
      tk(marked).to_string()
    );
    // a compound symbol mark is its run whole and a label per symbol, a doubled star one (the r6 re-review's x1, w10,
    // w11; 2602.19190, 2606.15609)
    assert_eq!(clean_frontmatter_labels("\\dagger*", ""), [
      "\\dagger*",
      "\\dagger",
      "*"
    ]);
    assert_eq!(clean_frontmatter_labels("\\dagger *", ""), [
      "\\dagger_*",
      "\\dagger",
      "*"
    ]);
    assert_eq!(clean_frontmatter_labels("**", ""), ["**"]);
  }

  #[test]
  fn leading_symbol_labels_skip_only_a_run_whole() {
    setup();
    // a run of distinct symbols is its run whole first, then a label per symbol: the run is no label of the line, its
    // symbols are (the r7 re-review's finding 3); a comma list gives a label per symbol and no run, so none is skipped
    // (the r8 re-review's finding 10)
    let labels = |s: &'static str| leading_symbol_labels(&tk(s));
    assert_eq!(
      labels("\\lx@frontmatter@keepsup{x}\\lx@frontmatter@symbolmark{\\dagger\\ddagger}Univ A"),
      ["\\dagger", "\\ddagger"]
    );
    assert_eq!(
      labels("\\lx@frontmatter@keepsup{x}\\lx@frontmatter@symbolmark{\\dagger,\\ddagger}Univ A"),
      ["\\dagger", "\\ddagger"]
    );
    assert_eq!(
      labels("\\lx@frontmatter@keepsup{x}\\lx@frontmatter@symbolmark{\\dagger *}Univ A"),
      ["\\dagger", "*"]
    );
    assert_eq!(
      labels("\\lx@frontmatter@keepsup{x}\\lx@frontmatter@symbolmark{**}Univ A"),
      ["**"]
    );
  }

  #[test]
  fn weldable_lines_are_institutions_and_places() {
    setup();
    // a symbol row's unmarked line goes on with the labelled institution above it only as an institution or a place —
    // a particle in a place too (2605.19581) — never a note, an address or a label (the 77y review's finding 3:
    // 2604.02904, 2602.03421, probes c1, c3, x4)
    let weldable = |s: &'static str| weldable_line(&tk(s));
    for line in [
      "Ball State University",
      "Muncie, IN 47306",
      "Campus de Santiago, 3810-183 Aveiro, Portugal",
      "{\\small Braunschweig, Germany}",
    ] {
      assert!(weldable(line), "{line}");
    }
    for line in [
      "Corresponding Author: \\texttt{aliwazirjam@gmail.com}",
      "Email: (hadi.aghaee, christian.deppe)@tu-bs.de, boche@tum.de",
      "Preprint. Under review.",
      "Work done during an internship at Google",
      "Corresponding author: Ann Able",
      "Ann Able, Bob Baker",
    ] {
      assert!(!weldable(line), "{line}");
    }
  }

  #[test]
  fn mailto_line_is_a_labelled_email_line() {
    setup();
    // a link line to a mailbox is an e-mail line under the text before it, whatever it says, and its address the one
    // the link prints, its target's when it prints none (2601.00240, 2601.07974's placeholder target, 77p r8)
    for (line, label, address) in [
      (
        "\\small{ \\textbf{Correspondence:} \\href{mailto:email@domain}{a.b@x.org} }",
        Some("Correspondence:"),
        "a.b@x.org",
      ),
      ("\\href{mailto:a@x.org}{Write to us}", None, "a@x.org"),
      (
        "Contact:~\\href{mailto:a@x.org}{a@x.org}",
        Some("Contact:"),
        "a@x.org",
      ),
      // (77p r9: a link to a bare address without `mailto:`, the ACL template's, 2604.04847; a footnote symbol before
      // the label)
      (
        "\\textbf{Correspondence:} \\href{b@x.org}{b@x.org}",
        Some("Correspondence:"),
        "b@x.org",
      ),
      (
        "\\lx@frontmatter@keepsup{*}\\lx@frontmatter@symbolmark{*}Corresponding author: \\href{mailto:a@x.org}{a@x.org}",
        Some("Corresponding author:"),
        "a@x.org",
      ),
    ] {
      let (got_label, addresses) = labelled_emails(&tk(line)).unwrap().expect(line);
      assert_eq!(
        got_label.map(|label| label.to_string()).as_deref(),
        label,
        "{line}"
      );
      let texts: Vec<String> = addresses.iter().map(|a| address_text(&a.address)).collect();
      assert_eq!(texts, [address], "{line}");
    }
    for line in [
      "\\href",
      "Mail: \\href",
      "\\textbf{Code:} \\href{https://x.org}{x.org}",
      "Write \\href{mailto:a@x.org}{a@x.org}",
    ] {
      assert!(labelled_emails(&tk(line)).unwrap().is_none(), "{line}");
    }
  }

  #[test]
  fn email_addresses_skip_colour_arguments() {
    setup();
    // a colour command's colour is no letter of an address (2604.19440's `\href{mailto:…}{\color{black}\texttt{…}}`,
    // the r5 re-review's c1-c6; 77p r9); what it prints stays
    for line in [
      "\\texttt{\\{\\color{black}aable, bbaker\\}@x.edu}",
      "\\textcolor{blue}{\\texttt{aable@x.edu, bbaker@x.edu}}",
      "\\textcolor[rgb]{0,0,1}{aable@x.edu, bbaker@x.edu}",
      "\\fcolorbox{red}{white}{aable@x.edu, bbaker@x.edu}",
      "\\texttt{\\{}\\href{mailto:aable@x.edu}{\\color{black}\\texttt{aable},} \\href{mailto:bbaker@x.edu}{\\color{black}\\texttt{bbaker}}\\texttt{\\}@x.edu}",
    ] {
      assert_eq!(
        email_addresses(&tk(line)),
        Some(vec![s!("aable@x.edu"), s!("bbaker@x.edu")]),
        "{line}"
      );
    }
    // (several linked addresses behind a label, the r5 re-review's u22)
    let (label, addresses) = labelled_emails(&tk(
      "Emails: \\href{mailto:a@x.org}{a@x.org}, \\href{mailto:b@y.org}{b@y.org}",
    ))
    .unwrap()
    .expect("a labelled list");
    assert_eq!(
      label.map(|label| label.to_string()).as_deref(),
      Some("Emails:")
    );
    let texts: Vec<String> = addresses.iter().map(|a| address_text(&a.address)).collect();
    assert_eq!(texts, ["a@x.org", "b@y.org"]);
  }

  #[test]
  fn email_addresses_skip_optional_arguments() {
    setup();
    // a command's optional arguments are no address text (2602.10975's `\makebox[\textwidth][c]{\tt …}`, 77p r6)
    assert_eq!(
      email_addresses(&tk(
        "\\makebox[\\textwidth][c]{\\tt haiyang.wang@huawei.com ~ lue.fan@ia.ac.cn}"
      )),
      Some(vec![s!("haiyang.wang@huawei.com"), s!("lue.fan@ia.ac.cn")])
    );
  }

  #[test]
  fn prints_nothing_reads_spacing_fonts_and_paragraphs() {
    setup();
    // (2609.05430's `\large{~~}`, 2606.00747's blank line, 2103.00887's `\tt\small`; 77p r6)
    for (line, nothing) in [
      ("\\large{~~}", true),
      ("\\vspace{2pt}", true),
      ("{\\small \\vspace{2pt}}", true),
      ("\\par", true),
      ("\\tt\\small ", true),
      ("\\large{Univ A}", false),
      ("\\url{https://x.org}", false),
    ] {
      assert_eq!(prints_nothing(&tk(line)), nothing, "{line}");
    }
  }

  #[test]
  fn rewrite_symbol_superscripts_splits_a_glued_symbol() {
    setup();
    // `$^{1*}$` and `\textsuperscript{a\dagger}`: the mark stays a superscript marker, the symbol is shown…
    for (marked, mark) in [
      ("X$^{1*}$", "X$^{1}$"),
      ("X\\textsuperscript{a\\dagger}", "X\\textsuperscript{a}"),
    ] {
      let out = rewrite_symbol_superscripts(tk(marked)).to_string();
      assert!(
        out.starts_with(&tk(mark).to_string()) && out.contains("\\lx@frontmatter@keepsup"),
        "{marked}: {out}"
      );
    }
    // …and a list of marks or a number alone is no glued symbol.
    for whole in ["X$^{12}$", "X$^{a,b}$", "X$^{1,*}$"] {
      let out = rewrite_symbol_superscripts(tk(whole));
      assert!(
        position_of(&out, &[T_CS!("\\lx@frontmatter@keepsup")]).is_none(),
        "{whole}: {out}"
      );
    }
  }
}
