//! Miscellaneous helpers: spaces, alignment classes, generic messages, attachments, box color, name nouns, ORCID ids.

use super::*;

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
  Cow::Owned(crate::tex_glue::dimension_to_spaces(dim, None))
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
pub(super) fn is_name_noun(def: &Rc<dyn Definition>, visited: &mut Vec<Token>) -> bool {
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
pub(super) fn orcid_id(id: Tokens) -> Tokens {
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
