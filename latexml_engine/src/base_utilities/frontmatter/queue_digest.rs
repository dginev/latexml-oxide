//! Digesting a queued front matter item: labels, the `Frontmatter` store, and the digested-entry seals.

use latexml_core::token::FieldLedgerScope;

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

/// The label keys of a frontmatter mark's operand — the request an `\inst{…}` or a name's `$^{…}$` makes, the label
/// an affiliation's leading mark sets — compared by their printed text (FIDELITY_CHARTER §4, 2026-10-29: refactor
/// stage 4, OXIDIZED_DESIGN_DIVERGENCES #919): [`printed_label`] when the operand prints through a command, else its
/// characters (which print themselves), each then parted and prefixed as [`clean_frontmatter_labels`] reads them. A
/// documented divergence from Perl's frontmatter API, which keys the source string (`cleanFrontmatterLabels`,
/// Base_Utility.pool.ltxml:365): aa's `\inst{\ESO}` with `\newcommand{\ESO}{2}` prints 2 and is entry 2's; `\dag` and
/// `$\dagger$` are one label, †.
pub(in crate::base_utilities) fn frontmatter_label_keys(
  operand: &Tokens,
  prefix: &str,
) -> Result<Vec<String>> {
  Ok(frontmatter_label_request(operand, prefix)?.0)
}

/// The label keys of a mark's operand ([`frontmatter_label_keys`]), and those of them only a picture's text prints
/// ([`PrintedLabel::outside_pictures`]): a request nothing answers is shown in the name (`insert::
/// show_unanswered_marks`), but not one a picture prints — the picture is no part of the name, and its text alone is
/// no mark (the TikZ ORCID icon's "iD" in `$^{1,\href{…}{\orcidicon}}$`, review rev92b_r3 S1), while it may answer a
/// label still (a circled `a`'s `a`, `\inst{\circled{1}}`'s 1).
pub(in crate::base_utilities) fn frontmatter_label_request(
  operand: &Tokens,
  prefix: &str,
) -> Result<(Vec<String>, Vec<String>)> {
  let keys = |printed: &str| -> Vec<String> {
    clean_frontmatter_labels(printed, prefix)
      .into_iter()
      .flat_map(parted_printed_label)
      .collect()
  };
  Ok(match printed_label(operand)? {
    Some(printed) => {
      let all = keys(&printed.text);
      let outside = keys(&printed.outside_pictures);
      let pictured = all
        .iter()
        .filter(|key| !outside.contains(key))
        .cloned()
        .collect();
      (all, pictured)
    },
    None => (
      clean_frontmatter_labels(&operand.to_string(), prefix),
      Vec::new(),
    ),
  })
}

/// A mark operand's printed text ([`printed_label`]), and the same with each pgf picture's text left out ([`label_ink`]:
/// a picture prints its text nodes, a label apart).
#[derive(Debug, Default)]
pub(in crate::base_utilities) struct PrintedLabel {
  /// What the operand prints.
  pub text:             String,
  /// What it prints outside its pictures, each picture a label separator.
  pub outside_pictures: String,
}

impl PrintedLabel {
  /// Write `c`, inside a picture or not.
  fn push(&mut self, c: char, in_picture: bool) {
    self.text.push(c);
    if !in_picture {
      self.outside_pictures.push(c);
    }
  }
}

/// A printed label read as the marks the page shows: a number or letters a symbol is glued to (`$^{\ICQD,\Phys\ast}$`
/// prints "1,4∗", 2412.09857) the whole, then each run apart (`4∗`, `4`, `∗`) — the number an affiliation's and the
/// symbol a note's —; a run of distinct symbols (`$^{\clubsuit\heartsuit}$` prints "♣♡", 1811.10830's spelled shape)
/// each symbol too (a doubled `∗∗` stays one), as the spelled run of symbol commands is (`SYMBOL_COMMAND_RUN`,
/// [`clean_frontmatter_labels`]); any other label as it is. A combining mark is part of the character before it
/// (`x̄`, `\={x}`'s, is one letter: no `x` and no mark of its own, review rev92b_r2 B1). Printed text alone (review
/// rev92b_r1 S2): a mark's spelling is parted by [`clean_frontmatter_labels`] only.
fn parted_printed_label(label: String) -> Vec<String> {
  let (prefix, bare) = match label.split_once(':') {
    Some((prefix, bare)) => (s!("{prefix}:"), bare.to_string()),
    None => (String::new(), label.clone()),
  };
  // (each run a kind — symbol or not — and its characters, a combining mark kept with the one before it)
  let mut runs: Vec<(bool, Vec<String>)> = Vec::new();
  for c in bare.chars() {
    if is_combining_mark(c) {
      match runs.last_mut().and_then(|(_, run)| run.last_mut()) {
        Some(character) => {
          character.push(c);
          continue;
        },
        None => return vec![label],
      }
    }
    let symbol = if c.is_alphanumeric() {
      false
    } else if is_printed_symbol(c) {
      true
    } else {
      return vec![label];
    };
    match runs.last_mut() {
      Some((kind, run)) if *kind == symbol => run.push(c.to_string()),
      _ => runs.push((symbol, vec![c.to_string()])),
    }
  }
  let mut parts: Vec<String> = Vec::new();
  for (symbol, run) in &runs {
    if runs.len() > 1 {
      parts.push(run.concat());
    }
    if *symbol && run.iter().any(|c| *c != run[0]) {
      parts.extend(run.iter().cloned());
    }
  }
  if parts.is_empty() {
    return vec![label];
  }
  std::iter::once(label)
    .chain(parts.into_iter().map(|part| s!("{prefix}{part}")))
    .collect()
}

/// Whether a printed character is a symbol a mark may be: a star, or a character no letter, digit, space, ASCII
/// punctuation, format character ([`is_format_char`]) or combining mark ([`is_combining_mark`], never a symbol on its
/// own: it is part of the character before it) — `†`, `⋆`, `♣` (`parted_printed_label`, `insert::unanswered_mark_text`).
pub(in crate::base_utilities) fn is_printed_symbol(c: char) -> bool {
  c == '*'
    || (!c.is_ascii()
      && !c.is_alphanumeric()
      && !c.is_whitespace()
      && !is_format_char(c)
      && !is_combining_mark(c))
}

/// The printed text of a frontmatter mark's operand that holds a command (a macro, a symbol, a font switch, math):
/// digested in a group, read as the page shows it ([`label_ink`]: its characters, a cross-reference as its key
/// `\ref{key}`, spacing a comma between labels); `None` — the operand's spelling, Perl's ToString, read instead — for
/// an operand of characters alone, one holding a note (`\inst{3\footnotemark[1]}`: the note is no label, and digesting
/// it here would queue it twice), one whose digestion errs or prints nothing. The speculative run's diagnostics are
/// discarded, an erring operand's error too (aa's `\inst{\undefinedmark}`: 0 errors, pdflatex's `! Undefined control
/// sequence`; Perl, which only stringifies the operand, reports none either — OXIDIZED_DESIGN_DIVERGENCES #919's
/// residuals). Inside an open assignment journal (a refused setter's replay, a title page's speculative run, the
/// observer's rehearsal: journals do not nest, `begin_assignment_journal`) no run opens and the spelling is read (#919's
/// residuals).
pub(in crate::base_utilities) fn printed_label(operand: &Tokens) -> Result<Option<PrintedLabel>> {
  let tokens = operand.unlist_ref();
  let plain = |t: &Token| {
    matches!(
      t.get_catcode(),
      Catcode::LETTER | Catcode::OTHER | Catcode::SPACE | Catcode::BEGIN | Catcode::END
    )
  };
  if tokens.iter().all(plain) || split_mark_notes(tokens).is_some() {
    return Ok(None);
  }
  if reading_label() {
    return Ok(None);
  }
  READING_LABEL.with(|r| r.set(true));
  let printed = speculative_label_ink(Tokens::new(tokens.to_vec()));
  READING_LABEL.with(|r| r.set(false));
  let printed = printed?;
  DebugFeature!(
    "frontmatter",
    "FRONT printed label {:?} of {}",
    printed,
    operand
  );
  Ok(
    printed
      .map(|p| PrintedLabel {
        text:             p.text.trim().to_string(),
        outside_pictures: p.outside_pictures.trim().to_string(),
      })
      .filter(|p| !p.text.is_empty()),
  )
}

thread_local! {
  /// While a label's operand is digested ([`printed_label`]): a frontmatter mark inside it (`\inst{\inst{1}}`) requests
  /// and sets nothing ([`reading_label`]).
  static READING_LABEL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether a label's operand is being digested ([`printed_label`]): the frontmatter label primitives run inside it
/// leave the entry being digested as it is.
pub(in crate::base_utilities) fn reading_label() -> bool {
  READING_LABEL.with(std::cell::Cell::get)
}

/// The printed text of `tokens` digested in a speculative run of their own (in a group, in a reading context of their
/// own, as sect05.rs `speculative_title_page_run` runs a title page): nothing of it stays — its assignments undone, its
/// diagnostics discarded, an argument read past its end (`\altaffilmark` handed `\let`) reading nothing of the
/// document's. `None` when no journal opens, the run is not undone, or it raised an error (a resource Fatal, or one
/// that could not be undone, propagates: Fatal stays Fatal).
fn speculative_label_ink(tokens: Tokens) -> Result<Option<PrintedLabel>> {
  let Some(journal) = begin_assignment_journal() else {
    return Ok(None);
  };
  let hold = util::logger::DiagnosticsHold::begin();
  let resource_fatal_before = resource_fatal_latched();
  let muted = SuppressLogOutputGuard::new(true);
  let depth = get_frame_depth();
  let boxes = box_lists_mark();
  let conditionals = if_stack_depth();
  // (the field ledger put back and the boxes it stamped unstamped, as the observer's rehearsal does: a `\usebox` in
  // the operand hands back a register's box, which the run's serial would otherwise stay on)
  let ledger_scope = FieldLedgerScope::begin();
  let result = reading_from_mouth(Mouth::default(), || -> Result<Digested> {
    bgroup();
    let digested = digest(tokens)?;
    egroup()?;
    Ok(digested)
  });
  let unwound = reading_from_mouth(Mouth::default(), || {
    new_local_box_list();
    while get_frame_depth() > depth {
      let before = get_frame_depth();
      let nobox = lookup_bool("groupNonBoxing");
      if pop_stack_frame(nobox).is_err() || get_frame_depth() >= before {
        break;
      }
    }
    expire_local_box_list();
    Ok(())
  });
  let printed = result.as_ref().ok().map(|digested| {
    let mut printed = PrintedLabel::default();
    label_ink(digested, &mut printed, false);
    printed
  });
  drop(ledger_scope);
  drop(muted);
  let errors = hold.errors_raised();
  let undone = journal.rollback();
  if let Err(err) = result {
    if !undone || (resource_fatal_latched() && !resource_fatal_before && !unlatch_runaway_fatal()) {
      hold.commit();
      return Err(err);
    }
    hold.discard();
    unwound?;
    unwind_box_lists_to(boxes);
    while if_stack_depth() > conditionals {
      pop_if_frame();
    }
    return Ok(None);
  }
  if undone {
    hold.discard();
  } else {
    hold.commit();
  }
  unwound?;
  Ok(printed.filter(|_| undone && errors == 0))
}

/// The text a digested label operand prints ([`printed_label`]): its boxes' characters, through lists and the arguments
/// a whatsit shows ([`shown_arguments`]); a cross-reference (`\ref`, its key resolved in post: it prints nothing yet)
/// written `\ref{key}`, as [`clean_frontmatter_labels`] reads a key. Spacing parts two labels as a comma does — any
/// white space in a box's text, and every item [`is_spacing_item`] reads as spacing, the reader 92e's [`first_ink`] uses
/// (`$^{1\,2}$`'s math glue, a box of a space; `\!`'s zero-width-space box; `\:`, `\>`, `\ `, `\hspace`, `\kern`,
/// `\mskip`): `$^{1\,2}$` is the labels 1 and 2 (2403.08629, 2601.02539, 1704.04058's `$^{* \, \dagger}$`; review
/// rev92b_r1 B1: one label `1_2`, nothing answered it, and the mark was lost). A format character prints no label, a
/// combining mark is part of the character before it ([`keeps_label_char`]: `\={x}`'s `x̄`), and a pgf/TikZ picture
/// prints only its text nodes ([`picture_text`]: a circled `a`'s `a`, never an SVG path's data), a label apart.
fn label_ink(d: &Digested, out: &mut PrintedLabel, in_picture: bool) {
  if is_spacing_item(d) {
    out.push(',', in_picture);
    return;
  }
  match d.data() {
    DigestedData::TBox(b) => {
      if let Ok(text) = b.borrow().get_string() {
        for c in text.chars() {
          if c.is_whitespace() {
            out.push(',', in_picture);
          } else if keeps_label_char(c, out.text.chars().next_back()) {
            out.push(c, in_picture);
          }
        }
      }
    },
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        label_ink(child, out, in_picture);
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      let cs = w.get_definition().get_cs_name().into_owned();
      if matches!(
        cs.trim_end(),
        "\\ref" | "\\autoref" | "\\eqref" | "\\lx@cref"
      ) {
        if let Some(key) = w.get_args().iter().rev().flatten().next() {
          for c in s!(",\\ref{{{}}},", key.to_string().trim()).chars() {
            out.push(c, in_picture);
          }
        }
        return;
      }
      // (a pgf picture prints only its text nodes, a label apart: 2605.02769's TikZ ORCID logo none, a circled `a` its
      // `a`; what it prints is kept apart, `PrintedLabel::outside_pictures`)
      if let Some(text) = picture_text(&w) {
        out.push(',', in_picture);
        for node in &text {
          label_ink(node, out, true);
        }
        out.push(',', in_picture);
        return;
      }
      if let Some(Stored::Digested(a)) = w.get_property("alignment").as_deref() {
        label_ink(a, out, in_picture);
      }
      for (arg, shown) in w.get_args().iter().zip(shown_arguments(&w)) {
        if shown && let Some(arg) = arg {
          label_ink(arg, out, in_picture);
        }
      }
      if let Ok(Some(body)) = w.get_body() {
        label_ink(&body, out, in_picture);
      }
      // (a math accent prints its glyph on its base: `$^{\bar{\dag}}$` is `†̄`, no `\dag` — review rev92b_r4 S2)
      for c in accent_combining_form(&w).unwrap_or_default().chars() {
        if keeps_label_char(c, out.text.chars().next_back()) {
          out.push(c, in_picture);
        }
      }
    },
    _ => {},
  }
}

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
///
/// Rust-only (OXIDIZED_DESIGN_DIVERGENCES #921; FIDELITY_CHARTER §4, user ruling 2026-10-30): a `name=` the call gives
/// is the label, and an EMPTY one (`name=`, `name={}`, or one that digests to nothing) is no label — the binding says
/// its class prints none (the kernel's title `\thanks`, a dedication, a report number set top right), and the role's
/// default does not stand in. Perl's `$name && scalar($name->unlist)` falls back to the default there. A call that
/// forwards an optional label (`\email[]{…}`, `name={#1}`) passes the role's default itself when none was given.
pub(in crate::base_utilities) fn get_frontmatter_name(
  name: Option<&String>,
  tag: &str,
  role: &str,
) -> Result<Option<String>> {
  let stag = tag.strip_prefix("ltx:").unwrap_or(tag);
  if let Some(name) = name {
    return Ok((!name.is_empty()).then(|| name.clone()));
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
/// (Perl holds a direct entry ref instead). An entry set after `\maketitle` placed the frontmatter — queued after
/// `\lx@frontmatterhere`'s flush, or digested in place after it (`{abstract}`) — is LATE ([`LATE_ENTRY`]): its element
/// is the one a late abstract's placement may move (`place_abstract_after_cover`, OXIDIZED_DESIGN_DIVERGENCES #903).
pub(in crate::base_utilities) fn frontmatter_push(tag: &str, mut entry: TagData) -> usize {
  let late = if lookup_bool("lx_frontmatter_flushing") {
    lookup_bool("lx_frontmatter_entry_late")
  } else {
    lookup_bool("lx_frontmatter_here_digested")
  };
  if late {
    entry
      .attr
      .insert(LATE_ENTRY.to_string(), "true".to_string());
    // (a late abstract carries its command's marker ordinal: [`LATE_MARK_ORDINAL`])
    if tag == "ltx:abstract" {
      let ordinal = if lookup_bool("lx_frontmatter_flushing") {
        lookup_string("lx_frontmatter_entry_mark")
      } else {
        take_late_mark()
      };
      if !ordinal.is_empty() {
        entry.attr.insert(LATE_MARK_ORDINAL.to_string(), ordinal);
      }
    }
  }
  // (handed on from a raw class's store, [`TITLE_PAGE_ENTRY`]: queued from it, or set from it in place)
  let from = if lookup_bool("lx_frontmatter_flushing") {
    lookup_string("lx_frontmatter_entry_from")
  } else {
    lookup_string(crate::frontmatter_stores::STORE_ENTRY_FROM)
  };
  if !from.is_empty() {
    entry.attr.insert(TITLE_PAGE_ENTRY.to_string(), from);
  }
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

/// Whether the frontmatter has an author creator, queued or digested: one a contact can annotate (a raw file's in-place
/// contact setter is the API's only then, `frontmatter_stores.rs` `\lx@class@contact`).
pub fn has_author_creators() -> bool { queued_author_count() > 0 || digested_author_count() > 0 }

/// Whether a creator entry — an author, or the pending stub of an annotation of one — is being digested: its first
/// content is still its `PlaceKeeper`, the annotations an `annotate=` contact inside it attached pushed after it (a
/// second `\affil` inside one `\author{…}`, 1805.12376's ci2018 shape, must read as the first did).
pub fn creator_entry_in_progress() -> bool {
  with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.get("ltx:creator").is_some_and(|list| {
      list
        .iter()
        .any(|e| matches!(e.content.first(), Some(TagContent::PlaceKeeper)))
    }),
    _ => false,
  })
}

/// Whether the author creators were placed — digested by a `\maketitle` or a frontmatter environment, not the fallback's
/// provisional flush (`note_digested_frontmatter`) — and no frontmatter entry is being digested now (a contact command
/// inside the author's own content runs while its creator digests).
pub fn author_creators_placed() -> bool {
  frontmatter_seal_key("ltx:creator", None, "author", false)
    .is_some_and(|key| lookup_mapping_int("frontmatter_digested", &key) >= FINAL_SEAL)
    && !frontmatter_entry_in_progress()
}

/// How many author creators stand, digested or queued: the role number (`author:N`, `num_ltx:creator`) the next one
/// will be given is one more.
pub fn author_creator_total() -> usize {
  queued_author_count() + usize::try_from(digested_author_count()).unwrap_or(0)
}

/// The queued author creators (their queue positions) whose names' marks request the label a mark's `operand` sets
/// (`requesting_authors`): what a rerouted setter's optional argument is read against (`frontmatter_stores.rs`
/// `optional_contact`: `\address[a]{…}` under `Ann$^a$` labels the address).
pub fn queued_authors_requesting(operand: &[Token]) -> Vec<usize> {
  requesting_authors(&queued_author_marks(), operand)
}

/// Whether an author creator of the flush being digested printed a mark requesting the affiliation label `key`: the
/// labels the marks of the authors the flush took from the queue request (`lx_frontmatter_requested_marks`, `place.rs`
/// `digest_front_matter`) — what [`queued_authors_requesting`] reads before the flush takes the queue. A class's
/// in-place `\affil[1]{Univ A}` inside `\author{ANN ABLE\textsuperscript{1} and BOB BAKER\textsuperscript{2} …}` runs
/// while its authors digest, the queue already taken (review rev94a r6 a19, on 94b r3's
/// `\lx@class@contact@optional`); only this flush's authors, not an earlier flush's whose creators the frontmatter still
/// holds (review rev94a r7 SF3).
pub fn digested_authors_requesting(key: &str) -> bool {
  let labels = clean_frontmatter_labels(key, "affiliation");
  !labels.is_empty()
    && lookup_bool("lx_frontmatter_flushing")
    && lookup_string("lx_frontmatter_requested_marks")
      .split('\u{1}')
      .any(|requested| labels.iter().any(|l| l == requested))
}

/// Whether an author's name shows a mark — a queued author's, or one of the flush being digested
/// ([`digested_authors_requesting`]'s): what a contact's leading mark is read against (`frontmatter_stores.rs`
/// `leads_with_unrequested_mark`) — with none, the marks the names print are not read.
pub fn authors_show_marks() -> bool {
  queued_author_marks().iter().any(|marks| !marks.is_empty())
    || with_value("frontmatter", |v| match v {
      Some(Stored::HashTagData(frontmatter)) => {
        frontmatter.get("ltx:creator").is_some_and(|creators| {
          creators
            .iter()
            .filter(|c| c.attr.get("role").map(String::as_str) == Some("author"))
            .any(|c| {
              ["_plainmarks", "_textmarks", "_picturemarks"]
                .iter()
                .any(|attr| {
                  c.attr
                    .get(*attr)
                    .is_some_and(|marks| !marks.trim().is_empty())
                })
            })
        })
      },
      _ => false,
    })
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

/// Whether a frontmatter entry is being digested: its first content still its `PlaceKeeper` (an `annotate=` contact
/// digested inside it pushes its annotation after it: [`creator_entry_in_progress`]).
pub fn frontmatter_entry_in_progress() -> bool {
  with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.values().any(|list| {
      list
        .iter()
        .any(|e| matches!(e.content.first(), Some(TagContent::PlaceKeeper)))
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
/// An abstract (`PRINTED_WHERE_THEY_STAND`) replaces only a store's entry released as no title
/// page's (`only_released`, [`RELEASED_STORE`]): the others of its name were printed where they stand, and are kept.
pub(in crate::base_utilities) fn frontmatter_clear_same_name(
  tag: &str,
  name: Option<&str>,
  lang: Option<&str>,
  store: Option<&str>,
  only_released: bool,
) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get_mut(tag)
    {
      list.retain(|e| {
        e.attr.get("name").map(String::as_str) != name
          || e.attr.get("xml:lang").map(String::as_str) != lang
          || e.attr.get("_store").map(String::as_str) != store
          || (only_released && !e.attr.contains_key(RELEASED_STORE))
      });
    }
  });
}

/// The flushes of the frontmatter queue so far (`digest_front_matter`).
pub(in crate::base_utilities) const FRONTMATTER_FLUSHES: &str = "lx_frontmatter_flushes";

/// The mark of an abstract given as `\abstract{…}` before `\maketitle` placed the frontmatter, which sect05.rs reads as
/// a group digested in place (`\lx@begin@abstract[command_form=true]`, [`takes_command_form`]) where Perl's
/// `\lx@add@abstract` queues it (latex_constructs.pool.ltxml:1136-1141): the flush count when it was added
/// ([`FRONTMATTER_FLUSHES`]). Until the next flush digests Perl's queue, a later abstract's `\lx@clear@frontmatter`
/// dequeues it there, so the next abstract of its name, language and store removes the entry here
/// ([`frontmatter_clear_command_form`]): a class's template `\abstract{…}` and the paper's own before `\maketitle` are
/// one abstract, the paper's, as pdflatex prints the last (sn-jnl, 2305.00493, 2609.14034). Two departures from Perl's
/// dequeue, which ignores the name and empties the queue filled after a flush too: an abstract of another name or
/// language is another element (a bilingual `\abstract{…}` and `{otherlanguage}`'s `{abstract}`, review probes q15,
/// q16), and one after `\maketitle` is printed where it stands (q01, q07), as pdflatex prints both and the base kept
/// them. OXIDIZED_DESIGN #154.
pub(in crate::base_utilities) const COMMAND_FORM: &str = "_command_form";

/// Take the `command_form` keyval of `\lx@add@frontmatter@until` (`\abstract{…}`, sect05.rs): whether the entry is the
/// abstract's command form ([`COMMAND_FORM`]).
pub(in crate::base_utilities) fn takes_command_form(options: &mut TagAttrs) -> bool {
  options
    .remove("command_form")
    .is_some_and(|value| value.trim() != "false")
}

/// Remove the entries of the abstract's command form not yet flushed ([`COMMAND_FORM`]) of the `name`, language and
/// store `key` of the abstract being added: Perl's `\lx@clear@frontmatter{ltx:abstract}` dequeue of them.
pub(in crate::base_utilities) fn frontmatter_clear_command_form(tag: &str, key: [Option<&str>; 3]) {
  let flushes = lookup_int(FRONTMATTER_FLUSHES).to_string();
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
      && let Some(list) = frnt.get_mut(tag)
    {
      list.retain(|e| {
        e.attr.get(COMMAND_FORM) != Some(&flushes)
          || ["name", "xml:lang", "_store"].map(|k| e.attr.get(k).map(String::as_str)) != key
          || matches!(e.content.as_slice(), [TagContent::PlaceKeeper])
      });
    }
  });
}

/// The mark of a store's entry no title page printed ([`frontmatter_release_store`]): an ordinary entry the document's
/// own of the same name and language replaces.
const RELEASED_STORE: &str = "_released_store";

/// Drop the abstract entries before (tag, index) of the same `name`, language and store as it, and its text (whitespace
/// aside): a repeat, not a second block — what a class or binding adds twice, which the clearing of a later same-named
/// entry used to absorb. The later copy stays, as the clearing kept it: a `\maketitle` placed the first, the second
/// is where the paper prints it (JAC2001's `\abstract{…}` before `\maketitle` and the same `{abstract}` after it,
/// physics0206049). A block with other text is a second one, kept. Not while an entry of the tag is still being
/// digested, whose index its digestion holds. OXIDIZED_DESIGN #154.
pub(in crate::base_utilities) fn frontmatter_drop_repeats(tag: &str, index: usize) {
  type Key = [Option<String>; 3];
  // (the boxes' text is read outside the borrow of the frontmatter: reading it consults the state)
  let entries: Vec<(Key, Option<Digested>)> = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt
      .get(tag)
      .map(|list| {
        list
          .iter()
          .map(|e| {
            let key = ["name", "xml:lang", "_store"].map(|k| e.attr.get(k).cloned());
            let content = match e.content.as_slice() {
              [TagContent::PlaceKeeper] => None,
              [TagContent::Box(d), ..] => Some(d.clone()),
              _ => None,
            };
            (key, content)
          })
          .collect()
      })
      .unwrap_or_default(),
    _ => Vec::new(),
  });
  if index >= entries.len() || frontmatter_has_open_placekeeper(tag) {
    return;
  }
  let text = |d: &Option<Digested>| {
    d.as_ref()
      .map(|d| {
        d.to_string()
          .split_whitespace()
          .collect::<Vec<_>>()
          .join(" ")
      })
      .filter(|t| !t.is_empty())
  };
  let (new_key, new_content) = &entries[index];
  let Some(new_text) = text(new_content) else {
    return;
  };
  let repeats: Vec<usize> = entries[..index]
    .iter()
    .enumerate()
    .filter(|(_, (key, content))| {
      key == new_key && text(content).as_deref() == Some(new_text.as_str())
    })
    .map(|(at, _)| at)
    .collect();
  for at in repeats.into_iter().rev() {
    frontmatter_remove(tag, at);
  }
}

/// The abstract and keywords entries of a raw class's `store` (`_store`, frontmatter_stores.rs `labelled_api`) whose
/// title page does not print it (`\hypersetup{pdfsubject={\@abst}}`, review probe `md`) become ordinary entries, which
/// the document's own of the same name and language replaces (`frontmatter_clear_same_name`, [`RELEASED_STORE`]): one
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
          list[i]
            .attr
            .insert(RELEASED_STORE.to_string(), "true".to_string());
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
/// Perl: fetchPendingEntry() — the last entry of a tag. Rust-only (OXIDIZED_DESIGN_DIVERGENCES #928 round 8): the
/// annotations its content made and deferred to their labels (`\lx@annotate@frontmatter@now`'s filled stubs) stand
/// after it and are passed over — a class's in-place `\affil[1]{Univ A}` inside `\author{\mbox{ANN
/// ABLE\textsuperscript{1}} \affil[1]{Univ A} \mbox{BOB BAKER\textsuperscript{2}} …}` is such an annotation (94b's
/// `\lx@class@contact@optional`), and the `\textsuperscript{2}` after it requested its label of no entry: the mark was
/// lost (review rev94a r7 SF3, o/o10). Perl reads the last entry alone: a `\label` inside an entry's content is
/// `\lx@set@frontmatter@label` (Base_Utility.pool.ltxml:330, :565), and the annotation it labels is lost there —
/// llncs's `\thanks{\protect\label{fn1}Funded.}` warns "Orphaned frontmatter annotation" and drops the text
/// (KNOWN_PERL_ERRORS #1204, review rev94a r8 pl/p02).
pub(in crate::base_utilities) fn with_pending_entry_attr(f: impl FnOnce(&mut TagAttrs)) {
  with_value_mut("frontmatter", |val_opt| {
    if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
      let mut tags: Vec<String> = frnt.keys().cloned().collect();
      tags.sort();
      for tag in tags {
        let deferred = |e: &TagData| {
          e.attr.get("role").map(String::as_str) == Some("pending")
            && !matches!(e.content.first(), Some(TagContent::PlaceKeeper))
        };
        if let Some(last) = frnt
          .get_mut(&tag)
          .and_then(|entries| entries.iter_mut().rev().find(|e| !deferred(e)))
          && matches!(last.content.first(), Some(TagContent::PlaceKeeper))
        {
          f(&mut last.attr);
          return;
        }
      }
    }
  });
}
