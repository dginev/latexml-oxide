//! Digesting a queued front matter item: labels, the `Frontmatter` store, and the digested-entry seals.

use super::*;

/// Digest the content for a frontmatter item, disabling or masking certain commands.
/// Note that we shouldn't be digesting any frontmatter until within document, so inPreamble unnec.
/// See clean_trailing_break for cleanup of misused \\.
/// Perl: digestFrontmatterItem($stomach, $tag, $item).
pub(in crate::base_utilities) fn digest_frontmatter_item(
  tag: &str,
  item: Tokens,
) -> Result<Digested> {
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
  // `\fnmsep` sets the comma between two footnote marks (`U.~Hopp\inst{1}\fnmsep\thanks{…}`, aa/llncs; webofc's
  // `\fnsep`, webofc.cls:217); before a mark that becomes a link or a note here it would stand alone in the name
  // ("U. Hopp,": astro-ph0001054, astro-ph0611016; "Andrea Shindler,": 2301.07438), so it sets nothing there; between
  // two marks that stay visible it is still their comma.
  // (once: an item digested within another — a `\thanks` in an author — would save the wrapper as itself, and a
  // `\fnmsep` there not before a mark would call itself without end)
  if !lookup_bool("lx_frontmatter_fnmsep") {
    for (sep, saved, wrapper) in [
      ("\\fnmsep", "\\lx@saved@fnmsep", "\\lx@frontmatter@fnmsep"),
      ("\\fnsep", "\\lx@saved@fnsep", "\\lx@frontmatter@fnsep"),
    ] {
      if lookup_definition(&T_CS!(sep))?.is_some() {
        let_i(&T_CS!(saved), &T_CS!(sep), None);
        let_i(&T_CS!(sep), &T_CS!(wrapper), None);
      }
    }
    assign_value("lx_frontmatter_fnmsep", Stored::Bool(true), None);
  }
  // `DigestText`, with the item's own paragraph breaks noted in its text mode's frame (`frontmatter_paragraph_break`)
  begin_mode("text")?;
  note_frontmatter_paragraphs(tag);
  // What the item leaves open closes with it (`digest_in_own_group`), as Perl's `egroup` after the item closes its group
  // whatever the item's own `Error`s: the ACM SIG template's `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}`
  // in `\author` — which the class sets inside its own tabular — opens a tabular here that nothing closes, so the
  // text mode could not end, this group was never popped, and its `\label` (`\lx@set@frontmatter@label`) took every
  // later `\label` of the document: each reference dangled (1704.03168, 1301.5612, 1401.6981, 0712.1016, sig-alternate).
  // Guard: `perfect_kernel_batch79l::an_author_left_open_keeps_the_documents_labels`.
  let (digested, _) = digest_in_own_group(|| digest(item))?;
  end_mode("text")?;
  egroup()?;
  digested
}

/// Perl: cleanTrailingBreak($document, $node) — remove trailing whitespace
/// text nodes and ltx:break elements.
pub(in crate::base_utilities) fn clean_trailing_break(
  document: &mut Document,
  node: &mut Node,
) -> Result<()> {
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
pub(in crate::base_utilities) fn unwrap_whole_name_bold(
  document: &mut Document,
  node: &Node,
) -> Result<()> {
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
pub(in crate::base_utilities) fn read_frontmatter_sup_operand() -> Result<Tokens> {
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

/// [`read_frontmatter_sup_operand`], parted into the mark and the notes it holds ([`split_mark_notes`]): the notes'
/// tokens, if any, for the reader to read after the mark.
pub(in crate::base_utilities) fn frontmatter_sup_operand_and_notes()
-> Result<(Tokens, Option<Tokens>)> {
  let operand = read_frontmatter_sup_operand()?;
  Ok(match split_mark_notes(operand.unlist_ref()) {
    Some((mark, notes)) => (Tokens::new(mark), Some(Tokens::new(notes))),
    None => (operand, None),
  })
}

/// A mark's font switch, no part of it: `$^{\bf{1}}$`, 1109.3726; `\textsuperscript{\normalfont{1},}` inside a bold
/// name, 2604.24532; Perl strips `\rm` only, Base_Utility.pool.ltxml:365 — OXIDIZED_DESIGN #34.
pub(in crate::base_utilities) static FONT_SWITCH: std::sync::LazyLock<Regex> =
  std::sync::LazyLock::new(|| {
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
pub(in crate::base_utilities) static SYMBOL_COMMAND_RUN: std::sync::LazyLock<Regex> =
  std::sync::LazyLock::new(|| Regex::new(r"^(?:(?:\\[a-zA-Z]+|\*)\s*){2,}$").unwrap());

/// The spacing a mark's operand may carry, which prints no label character and parts the labels it stands between as a
/// comma does: a tie, the control space, the math kerns `\!`, `\,`, `\:`, `\;`, `\>`, and the named spaces
/// (`\up{~~1}` is the label 1 and `\up{\!1,2}` the labels 1 and 2, 2209.14855; `$^{1,2\,*}$` the labels 1, 2 and the
/// symbol, 2403.08629) — written so, or digested, as a class's optional `\author[*\,b,c]{…}` labels reach it: the
/// no-break space, the thin, medium, thick, en and em spaces (2404.00867's "*\u{2009}b" was one label `*_b`, so Paul J.
/// Steinhardt lost his affiliation b).
static SPACING_IN_LABEL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
  Regex::new(concat!(
    r"~|[\u{00A0}\u{2002}-\u{2009}\u{202F}]|\\[!,:;> ]",
    r"|\\(?:quad|qquad|enspace|enskip|thinspace|negthinspace)([^a-zA-Z]|$)"
  ))
  .unwrap()
});

/// Perl: cleanFrontmatterLabels($labels, $prefix). Rust also parts the labels at the spacing ([`SPACING_IN_LABEL`]), on
/// both sides of the match: the mark an author requests and the one an affiliation sets (OXIDIZED_DESIGN #52(o)).
pub(in crate::base_utilities) fn clean_frontmatter_labels(
  labels: &str,
  prefix: &str,
) -> Vec<String> {
  let labels = FONT_SWITCH.replace_all(labels, "$1");
  // (Perl's `s/\\rm//g`, Base_Utility.pool.ltxml:365, which strips `\rm` glued to the label it switches, too: the
  // operand `{\rm a,b}` reads "\rma,b", where the tokenizer skipped the space after `\rm` — `$^{\rm a,b}$` requested
  // "\rma" and "b", so no `${\rm ^a}$` line answered the first, 0905.3658)
  let labels = labels.replace("\\rm", "");
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
pub(in crate::base_utilities) fn get_frontmatter_name(
  name: Option<&String>,
  tag: &str,
  role: &str,
) -> Result<Option<String>> {
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
pub(in crate::base_utilities) fn frontmatter_push(tag: &str, entry: TagData) -> usize {
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
pub(in crate::base_utilities) fn frontmatter_remove(tag: &str, index: usize) {
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
pub(in crate::base_utilities) fn queued_author_count() -> usize {
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
pub(in crate::base_utilities) fn supersede_digested_authors() -> bool {
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
pub(in crate::base_utilities) fn takes_accumulate(options: &mut TagAttrs) -> bool {
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
pub(in crate::base_utilities) fn frontmatter_seal_key(
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

pub(in crate::base_utilities) fn frontmatter_seal_key_of(tag: &str, name: &str) -> String {
  s!("{tag}\u{1}{name}")
}

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
pub(in crate::base_utilities) const FINAL_SEAL: i64 = 2;

pub(in crate::base_utilities) fn note_digested_frontmatter(key: &str) {
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
pub(in crate::base_utilities) fn frontmatter_sealed(key: &str) -> bool {
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
pub(in crate::base_utilities) fn frontmatter_clear_same_name(
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
pub(in crate::base_utilities) fn frontmatter_has_open_placekeeper(tag: &str) -> bool {
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
pub(in crate::base_utilities) fn frontmatter_set_first_content(
  tag: &str,
  index: usize,
  content: TagContent,
) {
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
pub(in crate::base_utilities) fn with_pending_entry_attr(f: impl FnOnce(&mut TagAttrs)) {
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
