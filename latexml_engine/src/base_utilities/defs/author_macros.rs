//! `LoadDefinitions!` part 4: the author macros (`\lx@add@authors*`, affiliations, superscript marks), the default text
//! names, and `\maketitle` supersede/fallback.

use super::{
  frontmatter_api::{
    PRINTED_LABEL, affiliation_calls, affiliation_is_marked, binding_cut_list_attr,
    blank_line_marks_dropped, shared_affiliation_calls, unalias_mark_commands,
  },
  *,
};

// (the IJCAI sectioned block's split, a module of its own since Z18m deleted `author_block/`)
mod ijcai;
use ijcai::{ijcai_block, ijcai_split_call};

/// The key on the entry the frontmatter emitter is writing (its pending entry, `with_digesting_entry_attr`) saying it has read
/// its first mark, its label (`\lx@sup@setlabel@affiliation`; taken off as each emitted item starts,
/// `\lx@frontmatter@item@start`, and as each entry's line starts, `\lx@frontmatter@savesups`): a fact of the entry, kept on
/// it as Perl keeps its labels (Base_Utility.pool.ltxml:502-508).
pub(in crate::base_utilities) const EMITTED_LINE_MARK: &str = "_markread";

/// The key of a creator written whole, Perl's fallback, whose merge a Warning counts: `flag_merged_creators`' own mark of a
/// creator it checked (a tabular block is kept whole on purpose alike). The emitter sets it where its Warning counts the block
/// (OD #980): not on the emitter's queued entry. A block arriving while the frontmatter is digested (an `\author` in
/// the `\title`, 1502.01001, 1510.00875, hep-ph0207056, hep-th0111116; in an amsart `\thanks`, 0807.4720; in a re-digested
/// `{opening}`, astro-ph9808358) has its own flush, which reads it or writes it whole and counted (`\lx@frontmatter@late`).
pub(in crate::base_utilities) const KEPT_WHOLE: &str = "_merged_checked=1";

/// One whole block queued for the call (OD #980), as no reader cuts it at the call, which the emitter
/// takes at the flush (`observer::emitter::emit`): Perl's whole-block creator (`fallback::whole_block_author`). A `late`
/// call, made while a flush digests its queue after reading it (the ledger's `AuthorBlock::late`), queues it for a flush
/// of its own (`\lx@frontmatter@late`): Perl's `\lx@add@authors` there runs `@now`, split at the call
/// (Base_Utility.pool.ltxml:799-806).
fn whole_block_call(
  stuff: Tokens,
  replace: bool,
  keyvals: Option<Tokens>,
  late: bool,
) -> Result<Tokens> {
  if replace {
    replace_queued_authors();
  }
  let whole = whole_block_author(keyvals, stuff)?;
  if late {
    return Ok(Invocation!(T_CS!("\\lx@frontmatter@late"), vec![Some(
      whole
    )]));
  }
  Ok(whole)
}

pub(in crate::base_utilities) fn load() -> Result<()> {
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
    let replace = !lookup_bool("\\author:redefined");
    let ijcai = ijcai_block(&stuff)?;
    let late = record_author_call("\\lx@add@authors@adaptive", &stuff, replace, ijcai, None, false);
    if ijcai {
      return ijcai_split_call(stuff, replace, late);
    }
    whole_block_call(stuff, replace, None, late)
  });
  // The late block's flush (`whole_block_call`): its whole entry queued — the flush digesting it let the API to `@now`,
  // locally — then read by a flush of its own (re-entrant, `digest_front_matter`), which emits its split creators or
  // writes it whole with its Warning, digested here into the frontmatter (1502.01001, 0807.4720, astro-ph9808358).
  DefPrimitive!("\\lx@frontmatter@late{}", sub[(whole)] {
    bgroup();
    let_i(&T_CS!("\\lx@add@frontmatter"), &T_CS!("\\lx@frontmatter@queue@add"), None);
    let_i(&T_CS!("\\lx@annotate@frontmatter"), &T_CS!("\\lx@frontmatter@queue@annotate"), None);
    let queued = digest(whole);
    let closed = egroup();
    let queued = queued?;
    closed?;
    digest_front_matter()?;
    Ok(vec![queued])
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

  DefMacro!("\\lx@add@authors{}", sub[(stuff)] {
    let ijcai = ijcai_block(&stuff)?;
    let late = record_author_call("\\lx@add@authors", &stuff, true, ijcai, None, false);
    if ijcai {
      return ijcai_split_call(stuff, true, late);
    }
    whole_block_call(stuff, true, None, late)
  });
  // A class's own author command, called once per author or author group and each adding to the ones before
  // (informs4.cls:1198 `\AUTHOR{Ann Able$^{a}$, Bob Baker$^{b}$}`): the author-line parsing of `\lx@add@authors`
  // without its replacing, so the names' superscript marks request the affiliations labelled with them.
  // A per-author class's `\author` (amsart, aastex, revtex) parses its body as one more author block too: a name list
  // there names several people (`\author{Klemens Fellner and Bao Quoc Tang}`, 1708.01427; aastex's
  // `Name\altaffilmark{1}, Name\altaffilmark{2}`, 1010.1318), and a `\\` line under a name is its affiliation
  // (`\author{Masao Ishikawa\\ \small Faculty of Education, …}`, math0606082). The optional keyvals go to each author
  // (revtex's `annotations=`).
  DefMacro!("\\lx@add@authors@append[]{}", sub[(keyvals, stuff)] {
    let late = record_author_call("\\lx@add@authors@append", &stuff, false, false, keyvals.as_ref(), true);
    whole_block_call(stuff, false, keyvals, late)
  });
  // A class's own author-list command (informs4.cls:1198 `\AUTHOR{Ann Able$^{a}$, Bob Baker$^{b}$}`, documented as a
  // name list): appended like `\lx@add@authors@append`, but split at every separator as the article `\author` is.
  // (where the list starts in the queue: the row an address shared by its authors falls back to, `queued_row_start`)
  DefMacro!("\\lx@add@authors@list{}", sub[(stuff)] {
    assign_value("lx_authors_list_start", queued_author_count() as i64, Some(Scope::Global));
    let ijcai = ijcai_block(&stuff)?;
    let late = record_author_call("\\lx@add@authors@list", &stuff, false, ijcai, None, false);
    if ijcai {
      return ijcai_split_call(stuff, false, late);
    }
    whole_block_call(stuff, false, None, late)
  });
  // A name list where a binding has no lines or affiliations to read: each name the reading finds (comma,
  // " and ", " \& ") is an author of its own, the keyvals given to each (`\author{R. Braun\inst{1} and W. B.
  // Burton\inst{2}}`, astro-ph9810433; `\IEEEauthorblockN{Clemens Paul Zengler, Niels Troldborg, Mac Gaunaa.}`,
  // 2410.19527). A collaboration the list names beside its names is a contact on them, no part of a name (batch 103b:
  // webofc's `… on behalf of the KLOE-2 Collaboration` after the notes, 1502.05517). Queued whole, as the emitter reads it.
  DefMacro!("\\lx@add@author@split[]{}", sub[(keyvals, names)] {
    let late = record_author_call("\\lx@add@author@split", &names, false, false, keyvals.as_ref(), false);
    whole_block_call(names, false, keyvals, late)
  });
  // That class's affiliation command, under its authors (informs4.cls:1199 `\AFF{$^a$Univ A}`): when the queued authors
  // carry marks, its marked lines are affiliations labelled by their marks, which go to the authors requesting them
  // (`affiliation_calls`; 2609.17368, 22690, 28084); otherwise it is one affiliation, of the last author or as the
  // optional attributes place it (apa7's unnumbered list, `annotate=all`).
  DefMacro!("\\lx@add@affiliation@marked[]{}", sub[(attr, stuff)] {
    if affiliation_is_marked(&stuff) {
      // (its marked entries name the list, which the frontmatter API's Cut then cuts no second time: `CUT_LIST`)
      let named = Some(binding_cut_list_attr(&stuff));
      Ok(Tokens::new(affiliation_calls(None, named, None, stuff, true, true)?))
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
      Ok(Tokens::new(affiliation_calls(None, None, None, stuff, true, true)?))
    } else {
      Ok(Tokens::new(shared_affiliation_calls(attr, stuff)?))
    }
  });
  DefMacro!("\\lx@ijcai@names{}", sub[(stuff)] {
    let late = record_author_call("\\lx@ijcai@names", &stuff, true, false, None, false);
    whole_block_call(stuff, true, None, late)
  });

  // Shared "sectioned author block" machinery for the IJCAI author idiom
  // (ijcai97.sty and its derivatives): one `\author{}` holding names, then
  // `\affiliations`, then a comma-separated `\emails` list. Used both by the
  // `ijcai_sty` binding's `\author` override and by the `\lx@add@authors`
  // marker-branch above (so raw-loaded derivatives like ttm.sty work too).
  // `\lx@ijcai@authorsplit` reads the names up to `\affiliations` and runs them
  // through `\lx@ijcai@names` (`\lx@add@authors` without the marker branch), then splits the remainder into
  // affiliations (up to `\emails`) and the comma-separated emails. Ported from Perl ijcai.sty.ltxml (PR #2767).
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
  // The `\emails` list, which ijcai19.sty:127-130 prints as one line under the affiliations, attributing none
  // (`{aable, ccole}@x.edu, bob.baker@y.org`). Perl's ijcai.sty.ltxml:50-53 splits it at every comma and gives the
  // n-th piece to the n-th author (`labelseq=author`): a grouped address `\{a, b\}@x` was cut into `{a` and
  // `b}@x` and every piece after it went one author on (2105.02711, 1905.13566, 2308.08563; the ttm.sty derivative's
  // `\{creddy,wmgifford,jayant\}@us.ibm.com`, 2401.03955). Rust-only (OXIDIZED_DESIGN_DIVERGENCES #908,
  // KNOWN_PERL_ERRORS #1200): the list is a class's address list, as jheppub's `\emailAdd` is (`owner=sequence`,
  // `\lx@add@contact@list`): each address goes to the author it spells, else to the n-th author while the list is one
  // address per author in their order, else to the author block (`sequenced_contact_labels`); a grouped address is
  // written out, one address each (`\lx@split@addresses`), or kept whole where it is no plain list of addresses. The
  // section is one call of the list (`\lx@sequence@call`, `\lx@add@contact@list`'s), read line by line: a `\\` and a
  // further `\emails` start a line in the class (ijcai19.sty:127 `\def\emails{\egroup\par\bgroup\rm}`), so an
  // address after one is an address of its own (2002.11711's `1871161@stu.neu.edu.cn\\zelei.liu@ntu.edu.sg`;
  // 2609.15094's `\emails {$^{*}$Corresponding author}` after its list) — [`ijcai_email_lines`].
  DefMacro!("\\lx@ijcai@emails{}", sub[(list)] {
    let mut out =
      mouth::tokenize_internal("\\lx@clear@frontmatter{ltx:contact}[role=email]\\lx@sequence@call").unlist();
    for line in ijcai_email_lines(&list) {
      // (a line with no address, `$^{*}$Corresponding author`, is no place of the list and no e-mail: a note, as
      // jheppub's `\note` files its "Corresponding author." (`role=note`, notes.rs `is_note_macro`), placed by its own
      // evidence, else the author block's, as `\lx@add@contact@placed` places a contact; 2609.15094)
      if address_text(&Tokens::new(line.clone())).contains('@') {
        out.extend([T_CS!("\\lx@split@addresses"), T_BEGIN!(), T_CS!("\\lx@ijcai@email"), T_END!()]);
      } else {
        out.extend(mouth::tokenize_internal("\\lx@add@contact@placed[role=note]{}").unlist());
      }
      out.push(T_BEGIN!());
      out.extend(line);
      out.push(T_END!());
    }
    Ok(Tokens::new(out))
  });
  DefMacro!(
    "\\lx@ijcai@email{}",
    "\\lx@add@contact@placed[role=email,owner=sequence]{}{#1}"
  );

  // What an author reference mark (IEEEtran's `\IEEEauthorrefmark{n}`) prints for its operand, `\lx@author@refmark@symbol`:
  // the operand itself (`\lx@author@refmark@arabic`), unless the class binding declares its own (ieeetran_cls.rs: the
  // footnote symbol sequence, IEEEtran.cls:4543; IEEEoj's binding lets it back to the number, IEEEoj.cls:3186). A label reader (`queue_digest::printed_label`) reads its number: the symbols a class's reference
  // marks print are one to one with their numbers, so the marks pair among themselves as they print, and a label a
  // superscript sets in another form is told apart by the class's author block, which writes a reference mark mixed
  // with other marks as the symbol it prints (OXIDIZED_DESIGN_DIVERGENCES #968; 2312.10807).
  DefMacro!("\\lx@author@refmark@arabic{}", "#1");
  Let!("\\lx@author@refmark@symbol", "\\lx@author@refmark@arabic");
  DefMacro!("\\lx@author@refmark@printed{}", sub[(operand)] {
    Ok(if reading_label() {
      operand
    } else {
      let symbol = Invocation!(T_CS!("\\lx@author@refmark@symbol"), vec![Some(operand.clone())]);
      Invocation!(T_CS!("\\lx@author@refmark@shown"), vec![Some(operand), Some(symbol)])
    })
  });
  // (the symbol as it prints, its superscript keyed by the number, `_refmark`: the readers of the marks a name shows —
  // `relocate::shown_marks`, the Kind steps' — read the mark by its number, as its label is, where its symbol is no
  // label mark: 2311.03600's `Jakob Hollenstein\IEEEauthorrefmark{1}` answers `\IEEEauthorblockA{\IEEEauthorrefmark{1}…}`)
  DefConstructor!("\\lx@author@refmark@shown Semiverbatim {}", "#2",
  after_construct => sub[document, whatsit] {
    let key = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
    // (the superscript holding it, past the font wrappers inside it)
    let mut node = Some(document.get_node().clone());
    while let Some(at) = node.as_ref()
      && !matches!(at.get_name().as_str(), "sup" | "personname" | "contact" | "creator" | "note")
    {
      node = at.get_parent();
    }
    if let Some(mut sup) = node.filter(|at| at.get_name() == "sup")
      && !key.trim().is_empty()
    {
      let key = match sup.get_attribute("_refmark") {
        Some(before) => s!("{before},{}", key.trim()),
        None => key.trim().to_string(),
      };
      document.set_attribute(&mut sup, "_refmark", &key)?;
    }
  });

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
  // A note the operand holds (`\textsuperscript{1,2\thanks{…}}`) is no part of a label: it is read after the mark, the
  // note its author's or its line's (2512.05464; an author block takes it out before, `notes_out_of_marks`).
  DefPrimitive!(T_CS!("\\lx@sup@request@affiliation"), None, {
    let (operand, notes) = frontmatter_sup_operand_and_notes()?;
    if reading_label() {
      return Ok(Vec::new());
    }
    // (keyed by the operand's printed text: refactor stage 4, OXIDIZED_DESIGN_DIVERGENCES #919)
    let (label, pictured) = frontmatter_label_request(&operand, "affiliation")?;
    // (a requested asterisk shows as it prints: one requested in text is `*` — `show_unanswered_marks`' text marks —
    // unless the operand prints another form, which its key reads `*` too: icrc2009's `\IEEEauthorrefmark{1}` sets the
    // math `∗`, 0909.3754, 0912.0896, 2609.25589; `one_asterisk`, review rev100a B1)
    let in_text =
      !(label.iter().any(|l| l == "affiliation:*") && prints_another_asterisk(&operand)?);
    // (an author reference mark keys by its number and prints its symbol, which a request nothing answers shows:
    // 2408.02464's `Gus Gray\IEEEauthorrefmark{1}` stays "Gus Gray∗"; OXIDIZED_DESIGN_DIVERGENCES #968)
    let mut shown = None;
    if let [first, rest @ ..] = operand.unlist_ref().as_slice()
      && *first == T_CS!("\\lx@author@refmark@printed")
      && let Some(key) = label.first()
    {
      let mut symbol = vec![T_CS!("\\lx@author@refmark@symbol")];
      symbol.extend_from_slice(rest);
      if let Some(printed) = printed_label(&Tokens::new(symbol))? {
        shown = Some(s!("{key}={}", printed.text));
      }
    }
    add_annotation_labels(label.join(","), in_text, pictured);
    if let Some(shown) = shown {
      with_pending_entry_attr(move |attr| {
        let mut all = attr.get("_refmarkshown").cloned().unwrap_or_default();
        if !all.is_empty() {
          all.push(';');
        }
        all.push_str(&shown);
        attr.insert("_refmarkshown".to_string(), all);
      });
    }
    if let Some(notes) = notes {
      unread(notes);
    }
    Ok(Vec::new())
  });
  DefPrimitive!(T_CS!("\\lx@sup@setlabel@affiliation"), None, {
    // (whether the operand is a group, `^{1}`, or a token, `^*`: a shown mark is read again as it was written, so it
    // prints in the form its superscript gives it — TeX sets a braced script's box apart, and the float-script rewrite
    // makes `^{1}` italic where `^*` and `^1` stay upright, as the uncut contact had them: review rev99a2 S-c)
    skip_spaces()?;
    let braced = if_next(T_BEGIN!())?;
    let (operand, notes) = frontmatter_sup_operand_and_notes()?;
    if let Some(notes) = notes {
      unread(notes);
    }
    // A line's first mark is its label; a later superscript in it is its text, shown (`$^{2}$Laboratory for
    // $^{3}$He`: the isotope's, 63i review; repro affiliation_line_inner_superscript_is_its_text).
    let mut labelled = false;
    with_pending_entry_attr(|attr| labelled = attr.contains_key("_bymark"));
    // (an entry the frontmatter emitter writes has its label from the emitter, so no `_bymark` records its first mark: the
    // entry's mark read since its item or its line started (a key on the entry: the mark's own formula is a group) says a later
    // superscript is its text — `$^{146}$European
    // Centre … (ECT$^*$) )`, 1812.07638's institute, whose `*` the emitted entry lost: OD #980)
    if lookup_bool(EMITTING) {
      with_digesting_entry_attr(|attr| {
        labelled = attr
          .insert(EMITTED_LINE_MARK.to_string(), "true".to_string())
          .is_some();
      });
    }
    // (in an entry of the frontmatter API's Cut, which shows its marks as printed, a later superscript is the one it
    // writes, in its own mode: `$^{1}$Lab $\frac{a}{b^{1}}$ Lab`'s fraction keeps its script, review 90a r3 probe AFg10)
    let plain_sup = || {
      let plain = if lookup_bool("IN_MATH") {
        T_CS!("\\lx@frontmatter@plainsup")
      } else {
        T_CS!("\\lx@frontmatter@plaintextsuperscript")
      };
      (lookup_bool("lx_affiliation_mark_shown") && lookup_meaning(&plain).is_some())
        .then_some(plain)
    };
    // (read again as the plain superscript where it stands, so a script keeps its base: `b^{1}`; braced as written)
    let read_plain = |plain: Token, operand: Tokens| {
      let mut sup = vec![plain];
      if braced {
        sup.push(T_BEGIN!());
      }
      sup.extend(operand.unlist());
      if braced {
        sup.push(T_END!());
      }
      unread(Tokens::new(sup));
    };
    if labelled {
      if let Some(plain) = plain_sup() {
        read_plain(plain, operand);
        return Ok(Vec::new());
      }
      return Ok(vec![digest(Invocation!(
        T_CS!("\\lx@frontmatter@keepsup"),
        vec![Some(operand)]
      ))?]);
    }
    // (keyed by the operand's printed text, as the authors' requests are: refactor stage 4, OXIDIZED_DESIGN_DIVERGENCES
    // #919 — `$^{\dag}$Univ A` answers `Ann$^{\dagger}$`)
    let printed = printed_label(&operand)?;
    let label = set_label(&operand, printed.as_ref());
    // the marks it is (`\mathrm{a}`: a), as the authors' are read (relocate_annotations): from its printed text, else
    // its TeX spelling, which keeps the boundary after a control word (`{\rm a}`'s tokens join to `\rma`: 2008.00493)
    let keys = match &printed {
      Some(printed) => printed_marks(&printed.text).join(","),
      None => script_marks(operand.clone().untex_string().as_ref()).join(","),
    };
    // (an entry the frontmatter emitter writes is labelled by the reading: the mark it prints is recorded as the label that
    // prints, `PRINTED_LABEL`, which the Kind step for a legend reads, `insert::legend_mark` — OD #980, P2 NT)
    if lookup_bool(EMITTING)
      && let Some(label) = &label
    {
      with_digesting_entry_attr(|attr| {
        attr
          .entry(PRINTED_LABEL.to_string())
          .or_insert_with(|| label.clone());
      });
    }
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
    // (an entry the frontmatter API's Cut made keeps the mark it prints, as the observer writer's cut keeps it: label
    // text is markup, never dropped — FIDELITY_CHARTER §1, review rev98e S1; `_markshown`, set while
    // `\lx@annotate@frontmatter@now` digests the entry, and a later superscript in it as written, above)
    if let Some(plain) = plain_sup() {
      read_plain(plain, operand);
    }
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
    // (an entry the frontmatter emitter writes, a call nested in another's too, has read no mark yet:
    // `\lx@sup@setlabel@affiliation`; 1812.05961's `\IEEEauthorblockA{… \IEEEauthorblockA{\IEEEauthorrefmark{2}…}}`)
    if lookup_bool(EMITTING) {
      with_digesting_entry_attr(|attr| {
        attr.remove(EMITTED_LINE_MARK);
      });
    }
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
    if reading_label() {
      return Ok(Vec::new());
    }
    // (keyed by the symbol's printed text, as the entry's mark is: refactor stage 4, OXIDIZED_DESIGN_DIVERGENCES #919)
    let label = frontmatter_label_keys(&sym, "affiliation")?.join(",");
    with_pending_entry_attr(move |attr| {
      if attr.get("role").map(String::as_str) != Some("pending") && !label.is_empty() {
        let labels = attr.get("_annotations").cloned().unwrap_or_default();
        let newval = if labels.is_empty() { label.clone() } else { s!("{labels},{label}") };
        attr.insert("_annotations".to_string(), newval);
      }
    });
  });

  // Rust-only: a label of the creator being made (inside its `\lx@add@author`), for a contact of the author block
  // placed on it after the names are made, in the order its lines print (`placed_label`; batch 77p r9). Only on a
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

  // Rust-only: how many marked names a names line kept whole as one creator holds, when two or more
  // (`split_author_line_cautious`), for `flag_merged_creators` to report the merge (batch 97a, OXIDIZED_DESIGN_DIVERGENCES
  // #943). Only on a creator, as `\lx@author@placed@label`.
  DefPrimitive!("\\lx@author@merged@names{}", sub[(names)] {
    let names = names.to_string();
    with_pending_entry_attr(move |attr| {
      if attr.get("role").map(String::as_str) != Some("pending") {
        attr.insert("_mergednames".to_string(), names);
      }
    });
  });

  // Rust-only: a collaboration an author line names (`collaboration_split`, batch 97d, OXIDIZED_DESIGN_DIVERGENCES
  // #946), which the author block's reader makes a `contact role="collaboration"` of; read by no reader, it prints
  // where it stood.
  DefMacro!("\\lx@author@collaboration[]{}", "#2");

  // Rust-only: the separator a per-author class's names line prints before this name, which the cautious split parted
  // it at (`split_author_line_cautious`; hep-ph9710228's ", " and " and "), for the creator's `before` in place of the
  // binding's (`place.rs`; brief 97b part D). Only on a creator.
  DefPrimitive!("\\lx@author@printed@before{}", sub[(separator)] {
    let separator = separator.to_string();
    with_pending_entry_attr(move |attr| {
      if attr.get("role").map(String::as_str) != Some("pending") {
        attr.insert("_printedbefore".to_string(), separator);
      }
    });
  });

  DefMacro!("\\lx@add@affiliations[]{}", sub[(attr, stuff)] {
    dequeue_front_matter("ltx:contact", &[("role", "affiliation")]);
    Ok(Tokens::new(affiliation_calls(attr, None, None, stuff, true, true)?))
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
  // Empty: LaTeX sets a `\thanks` as a marked footnote with no label (latex.ltx `\thanks`/`\@thanks`; pdflatex "∗ X"),
  // so the note has none (FIDELITY_CHARTER §4, user ruling 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES #921; Perl
  // "Thanks:~"). A class that prints one sets it here (resphilosophica_cls.rs "Acknowledgments").
  DefMacro!("\\lx@pubnote@thanks@name", "");

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
  // (one label: `_annotations` is a comma list, and a name's text holds the commas of its marks — Perl's
  // Base_Utility.pool.ltxml:873 left `Gia Dvali$^{a,b}$` the labels `fuzzy:Gia_Dvalia` and `b`, which
  // `relocate_annotations` read as Dvali's own request of the `$^b$` line, so Pujolas, marked `b` too, lost it: 0806.3762)
  Tag!("ltx:creator", after_close => sub[document, creator] {
    if let Some(person) = document.findnode("ltx:personname", Some(&*creator)) {
      let label = clean_label(&person.get_content(), Some("fuzzy")).replace(',', "_");
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
  // The title page's printed words against the whole output, once the document is built: each word the document wrote
  // that it lost is counted (`title_page_check`; FIDELITY_CHARTER §4). A check: it writes nothing. Rust-only.
  Tag!("ltx:document", after_close_late => sub[document, _root] {
    check_title_page_words(document)?;
  });

  // A `\maketitle` that typesets authors an `\author` set after the last flush replaces
  // the ones that flush digested (`whole_block_call`'s replace; DIVERGENCES #406). First in
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
    // (a binding's `\maketitle` has no deposit: what the document gave the raw setters before it is read here,
    // `title_page_check::digest_raw_setter_calls`; aeb_emulateapj over the revtex binding, 1011.2770)
    digest_raw_setter_calls()?;
    // A `\maketitle`'s flush: it may replace what the fallback flushed provisionally ([`frontmatter_sealed`]).
    let outer = lookup_bool("lx_frontmatter_explicit");
    assign_value("lx_frontmatter_explicit", true, Some(Scope::Global));
    let flushed = digest_front_matter();
    assign_value("lx_frontmatter_explicit", outer, Some(Scope::Global));
    flushed?;
    assign_value("frontmatter_deferred", true, Some(Scope::Global));
    // An abstract queued from here on is LATE (`\lx@frontmatter@mark`'s `late`, OXIDIZED_DESIGN_DIVERGENCES #903).
    assign_value("lx_frontmatter_here_digested", true, Some(Scope::Global));
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
      // First flush only: promote an unambiguous hand-made title, then let the pass
      // wrap the rest of the cover (before the abstract's marker) as <titlepage>. Once
      // `\maketitle` placed the frontmatter, a later abstract is a LATE abstract: the
      // title-page text printed between `\maketitle` and it — the class's own title page
      // (the deposit), cover lines set apart, never body prose, a sectional unit, a TOC
      // or a float — is its cover, wrapped as <titlepage> with the abstract behind it at
      // its own marker (`\lx@frontmatter@mark`, `place_abstract_after_cover`, OXIDIZED_DESIGN_DIVERGENCES #903;
      // 2011.04405, 1302.5352); what is not cover stays body after the abstract, which
      // the placed frontmatter holds (or `insert_late_frontmatter` files, streamed).
      // (`frontmatter_done` is reset by that path itself.)
      let first_flush = !lookup_bool("frontmatter_done");
      if first_flush {
        maybe_promote_leading_title(document)?;
      }
      place_frontmatter(document, first_flush, FrontmatterAnchor::LeadingFrontmatter)?;
    } else {
      // Where the abstract sits among the document's in-place setters (`\lx@frontmatter@mark`), read before the flush
      // sweeps its marker: the root child after it, or the end.
      let abstract_bound = abstract_mark_bound(document);
      place_frontmatter(document, false, FrontmatterAnchor::Top)?;
      // The marked print of a field first (`\lx@field@mark`); the text match only for a copy no mark explains.
      if lift_marked_field_ink(document)? {
        assign_value("lx_title_ink_dedup_done", true, Some(Scope::Global));
      }
      maybe_dedup_leading_title_ink(document)?;
      // A document without `\maketitle` whose setters print in place before its abstract (arxiv.cls's `\affiliation`,
      // 1207.1161; ae100prg's `\address`, 1303.3004; newpasp's `\affil`, astro-ph0210637): their cover is wrapped and
      // the abstract follows it, as a late abstract does (OXIDIZED_DESIGN_DIVERGENCES #903).
      if let Some(bound) = abstract_bound {
        place_abstract_after_cover(document, bound.as_ref(), false)?;
      }
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
  Ok(())
}

/// The lines of an IJCAI `\emails` section (`\lx@ijcai@emails`): parted at the top level by a line break (`\\`, its
/// `*` and `[skip]`, `\newline`) and by a further section marker (`\emails`, `\affiliations`), each a new line of the
/// class's title block (ijcai19.sty:124-130); a line that prints nothing is none. The top level is outside a TeX group
/// and outside a printed one (`\{ … \}`, `\lbrace … \rbrace`, `\textbraceleft … \textbraceright`): `\{aable, bbaker,\\
/// ccole\}@x.edu` is one group of three addresses broken over two lines, which `\lx@split@addresses` writes out (review
/// of 88b r1, probe p1; r2, probe i7).
fn ijcai_email_lines(list: &Tokens) -> Vec<Vec<Token>> {
  let breaks = [
    T_CS!("\\\\"),
    T_CS!("\\newline"),
    T_CS!("\\emails"),
    T_CS!("\\affiliations"),
  ];
  let v = list.unlist_ref();
  let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  let mut printed = 0usize;
  let mut at = 0;
  while at < v.len() {
    let t = v[at];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if t == T_CS!("\\{") || t == T_CS!("\\lbrace") || t == T_CS!("\\textbraceleft") => {
        printed += 1
      },
      _ if t == T_CS!("\\}") || t == T_CS!("\\rbrace") || t == T_CS!("\\textbraceright") => {
        printed = printed.saturating_sub(1)
      },
      _ => {},
    }
    if depth == 0 && printed == 0 && breaks.contains(&t) {
      at += 1;
      if t == T_CS!("\\\\") {
        // (`\\*`, `\\[1ex]`: the break's own arguments)
        let mut next = at + v[at..].iter().take_while(|t| **t == T_SPACE!()).count();
        if v.get(next) == Some(&T_OTHER!("*")) {
          next += 1;
          at = next;
        }
        if v.get(next) == Some(&T_OTHER!("["))
          && let Some(close) = v[next..].iter().position(|t| *t == T_OTHER!("]"))
        {
          at = next + close + 1;
        }
      }
      lines.push(Vec::new());
      continue;
    }
    lines.last_mut().expect("a line").push(t);
    at += 1;
  }
  // (without the spaces around it, the end of the line before: `…,\\` then `wenqian@u.nus.edu`, 2304.11989)
  lines
    .into_iter()
    .map(|line| {
      let lead = line.iter().take_while(|t| **t == T_SPACE!()).count();
      let trail = line[lead..]
        .iter()
        .rev()
        .take_while(|t| **t == T_SPACE!())
        .count();
      line[lead..line.len() - trail].to_vec()
    })
    .filter(|line| !prints_nothing(&Tokens::new(line.clone())))
    .collect()
}
