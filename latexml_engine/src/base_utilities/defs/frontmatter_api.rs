//! `LoadDefinitions!` part 2: the frontmatter API (`\lx@personname`, the `Frontmatter` keyvals, `\lx@add@frontmatter*`,
//! `\lx@annotate@frontmatter*`, labels).

use super::*;
use crate::base_utilities::frontmatter::observer::block_creator_written;

mod cut;
pub(in crate::base_utilities) use cut::*;
mod kind;
use kind::*;
// (`pub(in crate::base_utilities)`: `base_utilities/mod.rs` re-exports by name the gates `frontmatter_stores` and the
// bindings call, Z18j)
pub(in crate::base_utilities) mod affiliations;
pub(in crate::base_utilities) use affiliations::*;
// (the e-mail line readers Kind and the cutter share, `frontmatter_api::email_lines`; not `classify::email_lines`, the
// text rules on an e-mail's printed ink — two modules of that name, each named by its parent; Z18m deleted the third,
// `author_block/email_lines.rs`, its `drop_empty_marks` now `classify::old_reader::marks`')
pub(in crate::base_utilities) mod email_lines;
pub(in crate::base_utilities) use email_lines::*;
mod line_marks;
pub(in crate::base_utilities) use line_marks::*;

/// The attribute (and `\lx@add@contact` option, a declared `Frontmatter` key) of a contact whose label its binding
/// prints before its line, as written: authblk's `\affil[x]` (`\AB@affilnote`, authblk.sty:142), jheppub's
/// `\affiliation[x]` (`\item[$^{#1}$]`), cas's letter (`cas_labelled_entry`); its value `true`. The Kind step for a
/// legend gives a note only a mark that prints (`insert::kind_legend_contacts`, OXIDIZED_DESIGN_DIVERGENCES #961): a
/// binding's key (icml's `\icmlaffiliation{intern}{…}`) is none.
pub(in crate::base_utilities) const LABEL_PRINTS: &str = "_labelprints";

/// The attribute (and option, a declared `Frontmatter` key) holding the label that prints, of an entry the frontmatter
/// emitter wrote, which the reading placed by its own label: the call's `label=` its binding prints ([`LABEL_PRINTS`],
/// the writer's `rekeyed`: authblk's `\affil[*]`, 2605.31330), or the first mark the entry's content printed
/// (`\lx@sup@setlabel@affiliation`'s record). Read by the Kind step for a legend as a printed mark
/// (`insert::legend_mark`; OD #980, P2 NT). A fact of the entry in a key of its own, as Perl keeps an entry's
/// (`_label`, `_annotations`).
pub(in crate::base_utilities) const PRINTED_LABEL: &str = "_printedlabel";

/// The contact roles the frontmatter API's Cut reads ([`cut_marked_contact`]): the ones a printed list of entries is
/// labelled by marks in (an affiliation, an alternate one, an address).
pub(in crate::base_utilities) const CUT_ROLES: [&str; 3] =
  ["affiliation", "altaffiliation", "address"];

/// The attribute naming the affiliation-kind entry an e-mail was digested inside, and the entry itself (its key, a count
/// of such entries: `\lx@annotate@frontmatter@now`), by which the Link step reads the entry's addresses and carriers
/// (`insert::link_entry_addresses`; OXIDIZED_DESIGN_DIVERGENCES #956).
pub(in crate::base_utilities) const ENTRY_KEY: &str = "_entry";

/// The transient attribute of a creator the reading marks the corresponding author, by a note or a legend
/// (`resolver::RCreator::corresponding`, the reading's one fact, written by the emitter's creator call): the Link step of an
/// entry's addresses reads it for the one address left of several carriers (`insert::link_entry_addresses`; repro
/// llncs_legend_corresponding_author_takes_the_left_address). Dropped at output, as every `_` attribute is.
pub(in crate::base_utilities) const CORRESPONDING: &str = "_corresponding";

/// The attribute saying a footnote symbol leads the entry [`ENTRY_KEY`] names (`$^\dagger$…`), whose addresses no
/// carrier spells are every carrier's (FIDELITY_CHARTER §4, "every symbol carrier").
pub(in crate::base_utilities) const ENTRY_SYMBOL: &str = "_entrysymbol";

/// The attribute of a contact labelled by its place among its role's entries (`labelseq`: svjour's and llncs's
/// `\institute{… \and …}`, aa's): the label `\lx@annotate@frontmatter@now` gave it, a number no mark printed — the
/// Kind step for a legend reads it as a mark only where a carrier's name shows or requests it
/// (`insert::kind_legend_contacts`, OXIDIZED_DESIGN_DIVERGENCES #961; 1507.04774's svjour_blank prints none). Any other
/// `affiliation:K` label was read off a printed mark (the marks path, the Cut, the observer's writer).
pub(in crate::base_utilities) const SEQUENCE_LABEL: &str = "_seqlabel";

/// The attribute of an e-mail the Kind step made of an entry's line or note ([`kind_entry_addresses`]), as against an
/// e-mail command the author wrote inside the entry, which its own call places where it holds one address
/// (FIDELITY_CHARTER §4 #420; `insert::link_entry_addresses`).
pub(in crate::base_utilities) const ENTRY_KINDED: &str = "_kinded";

/// The attribute of an entry of the Cut ([`cut_marked_contact`]) naming the printed list it came from
/// ([`cut_list_key`]): a copy of the list queued again adds no entries ([`cut_list_queued`]), and an entry relocated to
/// an author keeps the place the author's own uncut copy had (`relocate::at_queue_position`, [`CUT_COPY_PLACE`]).
pub(in crate::base_utilities) const CUT_LIST: &str = "_cutlist";

/// The attribute (and a declared `Frontmatter` key) of the author block's own creator, which the frontmatter emitter
/// writes below the names: Perl's one nameless creator holding only the block's contacts (`label=addresses:block`;
/// OXIDIZED_DESIGN #159, FIDELITY_CHARTER §4). No name of the row, it takes no place in its role's count — no
/// `_num`, no `author:N` label, no separator — and no pass that pairs names with contacts reads it as one
/// (`insert::coalesce_empty_creators`, `distribute_upfront_contacts`; `relocate_annotations` reads authors by their
/// `ltx:personname`). The frontmatter list indexes its entries by it: `TagList`, whose `tag::BLOCK_CREATOR` is this key
/// (the assertion below keeps the two copies equal at compile time).
pub(in crate::base_utilities) const BLOCK_CREATOR: &str = "_blockcreator";
const _: () = assert!(
  same_str(BLOCK_CREATOR, document::tag::BLOCK_CREATOR),
  "frontmatter_api::BLOCK_CREATOR and tag::BLOCK_CREATOR differ"
);

/// `a == b` for strings, in a `const` context.
const fn same_str(a: &str, b: &str) -> bool {
  let (a, b) = (a.as_bytes(), b.as_bytes());
  if a.len() != b.len() {
    return false;
  }
  let mut i = 0;
  while i < a.len() {
    if a[i] != b[i] {
      return false;
    }
    i += 1;
  }
  true
}

pub(in crate::base_utilities) fn load() -> Result<()> {
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
  // Rust-only: the abstract read from `\abstract{…}` (sect05.rs), which Perl's `\lx@add@abstract` queues
  // ([`COMMAND_FORM`]), honoured by `\lx@add@frontmatter@until` ([`takes_command_form`])
  DefKeyVal!("Frontmatter", "command_form", "");
  DefKeyVal!("Frontmatter", "name", "");
  // Rust-only: the RDFa pair of an `ltx:rdf` annotation ([`\lx@add@creator@rdf`]; OXIDIZED_DESIGN #479). Semiverbatim, so an
  // address's `_` and `~` are characters, not a script and a tie.
  DefKeyVal!("Frontmatter", "property", "Semiverbatim");
  DefKeyVal!("Frontmatter", "content", "Semiverbatim");
  // Rust-only: the store an unclaimed-text net note was handed on from (frontmatter_stores.rs `hand_net_stores`), read
  // where the note is placed (insert.rs `keep_unclaimed_net_notes`; OXIDIZED_DESIGN_DIVERGENCES #953).
  DefKeyVal!("Frontmatter", "_net", "Semiverbatim");
  // Rust-only: a contact whose label its binding prints with the line (authblk's `\affil[x]`, jheppub's
  // `\affiliation[x]`, cas's letter: [`LABEL_PRINTS`]), read by the Kind step for a legend (insert.rs
  // `kind_legend_contacts`; OXIDIZED_DESIGN_DIVERGENCES #961). Declared: an unknown key is reported (`Info:undefined`),
  // and that report changed what osajnl's speculative title-page run read (its `\vphantom{M}` was lost: 2203.08703,
  // 2206.12157, 2311.18459, 2411.07865).
  DefKeyVal!("Frontmatter", "_labelprints", "");
  // Rust-only: the label that prints, of an entry the frontmatter emitter wrote ([`PRINTED_LABEL`]; OD #980, P2 NT).
  DefKeyVal!("Frontmatter", "_printedlabel", "");
  // Rust-only: the author block's own creator, which the frontmatter emitter writes ([`BLOCK_CREATOR`]).
  DefKeyVal!("Frontmatter", "_blockcreator", "");
  // Rust-only: a creator the reading marks the corresponding author, which the frontmatter emitter writes and the Link step
  // of an entry's addresses reads ([`CORRESPONDING`]). Declared, as `_labelprints` is: an unknown key's report
  // (`Info:undefined`) changed the paragraphs after the title page (1512.06325's first paragraph lost `ltx_noindent`).
  DefKeyVal!("Frontmatter", "_corresponding", "");

  // Rust-only: `\lx@label@ref{<key>}`, the label a binding recorded for `key` once later input was read
  // ([`set_label_ref`]: cas's `\stmRef`, which LaTeX reads from the `.aux` of the run before), nothing for one never
  // recorded; read where a queued entry is digested, and by the flush's readers of the queue ([`resolve_label_refs`]).
  DefMacro!("\\lx@label@ref{}", sub[(key)] {
    let key: Tokens = key;
    let label = label_ref_text(&key.to_string()).unwrap_or_default();
    Ok(mouth::tokenize_internal(TeXString::assembled(label)))
  });

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
      drop_late_mark(tag_tks.to_string().trim());
      return Ok(Vec::new());
    }
    queue_add_frontmatter_now(keys_opt.as_ref(), &tag_tks, attrs_opt.as_ref(), Some(&tokens))?;
  });
  // (its queueing meaning, which a block arriving while a flush digests the queue queues with: `\lx@frontmatter@late`)
  Let!("\\lx@frontmatter@queue@add", "\\lx@add@frontmatter");

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
    // (a rehearsal's run notes the entry it would make, as the command a queued one is digested by: a setter inside a
    // block read while a flush digests its queue — `\date` in an `\author` nested in `\title`, 1411.6693 — runs `@now`, not
    // the queue, `rehearsal::note_frozen_entry`)
    if rehearsal::rehearsing() && tag != "ltx:creator" {
      rehearsal::note_frozen_entry(&tag, kv.as_ref(), &now_command(None, &tag_tks, kv.as_ref(), Some(&content))?);
    }
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
    // one replaces earlier ones. Ported from upstream `%ReplaceableFrontmatterTags`, which
    // upstream later removed (#2767; the vendored HEAD pushes unconditionally → duplicate
    // <title> when a document re-adds it, e.g. arXiv 2002.09766's appendix `\icmltitle`);
    // an abstract is printed where it stands, and kept (`PRINTED_WHERE_THEY_STAND`,
    // 1006.5288). OXIDIZED_DESIGN #154.
    // An `accumulate` entry adds to the ones before (acmart's `\received` history, 2307.05988).
    let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate;
    let printed_in_place = PRINTED_WHERE_THEY_STAND.contains(&tag.as_str());
    // …unless the document body began after the one it would replace: that one is the paper's, this a supplement's
    // ([`seal_digested_frontmatter`]), dropped — with the authors a supplement's frontmatter lists — before it is counted
    // (a later flush's last-author conjunction reads the count).
    let seal = frontmatter_seal_key(&tag, options.get("name").map(String::as_str), &role, replaceable);
    if let Some(ref key) = seal
      && frontmatter_sealed(key)
    {
      Info!("frontmatter", "sealed", s!("{tag} after the document body began is a supplement's; the paper's is kept"));
      drop_late_mark(&tag);
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
    if !role.is_empty() && options.contains_key(BLOCK_CREATOR) {
      // (the block's own creator is no name of the row: uncounted, unnumbered — [`BLOCK_CREATOR`])
      options.insert("role".to_string(), role.clone());
    } else if !role.is_empty() {
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
        ["name", "xml:lang", "_store", "role"].map(|k| entry.attr.get(k).map(String::as_str)),
        printed_in_place,
      );
    }
    let index = frontmatter_push(&tag, entry);
    // (the title-page check's watch over a frontmatter command's entry: #953 "106t")
    let watch = begin_frontmatter_entry_digestion(Some(&content), None);
    // REPLACE only 'place_keeper'!!
    let digested = digest_frontmatter_item(&tag, content);
    end_frontmatter_entry_digestion(watch, digested.as_ref().ok())?;
    let digested = digested?;
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #803, #916; user rulings 2026-10-21, 2026-10-30): a document's note that
    // prints nothing is none, whatever its label — one whose whole content our own extraction moved out (A&A's
    // `\offprints{\email{…}}` makes the address a contact and left an empty "Preprint: " note: astro-ph0405280,
    // astro-ph0608669) and one the author left empty or blank (`\title{…\thanks{}}`, an empty "Thanks: " note:
    // 2004.01582, 2401.15379; mdpi's `\externaleditor{~}`, 2609.00671; lipics's `\Copyright{\@empty}`, 1508.00510).
    // Only material known to print nothing counts as none ([`leaves_no_ink`]): a note keeping any other item stays —
    // text (`\offprints{O. Kochukhov, \email{…}}`, mdpi's `\corres{Correspondence: }`, 1609.03883), a command's own
    // text (`\offprints{\LaTeX}`, `\thanks{\dots}`), a rule or a frame, an argument the test cannot read (JHEP3's
    // `\preprint{\arXivid{…}}`, 0704.3719).
    if matches!(tag.as_str(), "ltx:pubnote" | "ltx:note") && leaves_no_ink(&digested) {
      DebugFeature!("frontmatter", "FRONT drop {tag}: it prints nothing");
      frontmatter_remove(&tag, index);
      return Ok(Vec::new());
    }
    frontmatter_set_first_content(&tag, index, TagContent::Box(digested));
    if replaceable && printed_in_place {
      frontmatter_drop_repeats(&tag, index);
    }
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
    // (the abstract's command form, which Perl queues — removable by the next abstract of its name and language until
    // a flush digests it — while `\maketitle` has not placed the frontmatter: [`COMMAND_FORM`])
    let command_form =
      takes_command_form(&mut options) && !lookup_bool("lx_frontmatter_here_digested");
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    let replaceable = REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()) && !accumulate;
    let printed_in_place = PRINTED_WHERE_THEY_STAND.contains(&tag.as_str());
    // (sealed whether or not the blocks accumulate: the IEEEtran and VTeX keyword blocks add to the ones before them,
    // and a supplement's after the body began is still its own — 2604.23289's `{IEEEkeywords}` under its "Supplemental
    // File" section, 77t)
    let seal = frontmatter_seal_key(
      &tag,
      options.get("name").map(String::as_str),
      &role,
      REPLACEABLE_FRONTMATTER_TAGS.contains(&tag.as_str()),
    );
    // An abstract (or keywords) after the document body began, where the paper's own came before it, is a
    // supplement's: its text stays where it is, as body paragraphs (the schema admits `ltx:abstract` only in a
    // document, sidebar, bibliography or titlepage, LaTeXML-structure.rnc:668-673), and the paper's stays at the head
    // ([`seal_digested_frontmatter`]; 2301.10468, 2401.11672, 2405.10566, 2503.16707).
    if let Some(ref key) = seal
      && frontmatter_sealed(key)
    {
      Info!("frontmatter", "sealed", s!("{tag} after the document body began is a supplement's; kept in place"));
      drop_late_mark(&tag);
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
    if command_form {
      options.insert(COMMAND_FORM.to_string(), lookup_int(FRONTMATTER_FLUSHES).to_string());
    }
    let entry = TagData {
      tag: tag.clone(),
      attr: options,
      content: vec![TagContent::PlaceKeeper], // (in case embedded)
    };
    // A replaceable tag keeps only one entry — an abstract, printed where it stands, replaces only a
    // store's entry no title page printed (`PRINTED_WHERE_THEY_STAND`; the vendored Perl keeps every one, 1006.5288)
    // — but ONLY clear when no same-tag `@until` is already in progress (open PlaceKeeper), so a nested/malformed
    // `\begin{abstract}\begin{abstract}…` isn't corrupted by clearing a parent's still-open entry.
    // OXIDIZED_DESIGN #154. Witness 2511.21969 (nested abstract env).
    // The abstract's command form before it of its name, language and store, not yet flushed, is what Perl's
    // `\lx@clear@frontmatter` dequeues ([`COMMAND_FORM`]).
    if replaceable && !frontmatter_has_open_placekeeper(&tag) {
      let key = ["name", "xml:lang", "_store"].map(|k| entry.attr.get(k).map(String::as_str));
      frontmatter_clear_same_name(
        &tag,
        [key[0], key[1], key[2], entry.attr.get("role").map(String::as_str)],
        printed_in_place,
      );
      if printed_in_place {
        frontmatter_clear_command_form(&tag, key);
      }
    }
    // (the document's frontmatter commands this entry is made in, for the title-page check: `queue_front_matter`)
    let checked = note_frontmatter_entry(&tag, &role, None);
    // (a rehearsal's run notes the entry it digests: `rehearsal::note_frozen_digest`)
    rehearsal::note_frozen_digest(&tag);
    let index = frontmatter_push(&tag, entry);
    // the body's own paragraph breaks are the field's (`frontmatter_paragraph_break`); the enclosing notes come back
    // after it
    let enclosing = (lookup_value("lx_frontmatter_par_item"), lookup_value("lx_frontmatter_par_depth"));
    note_frontmatter_paragraphs(&tag);
    // A terminal reached INSIDE a nested group (`\section`'s
    // `\@startsection@hook` inside an unbalanced `\abstract{…`) ends the
    // body from there: `digest_next_body` records the terminal and depth on
    // this frame, `until_terminal_inside_group` (stomach.rs) acts.
    // (the title-page check's watch over a frontmatter command's entry: #953 "106t")
    let watch = begin_frontmatter_entry_digestion(None, checked);
    let body = digest_next_body(Some(end));
    assign_value("lx_frontmatter_par_item", enclosing.0.unwrap_or(Stored::None), Some(Scope::Local));
    assign_value("lx_frontmatter_par_depth", enclosing.1.unwrap_or(Stored::None), Some(Scope::Local));
    let digested = body.map(|body| Digested::from(List::new(body)));
    // (an entry that typesets nothing is removed, below: no words of it are wanted)
    let kept = digested.as_ref().ok().filter(|digested| typesets_content(digested));
    end_frontmatter_entry_digestion(watch, kept)?;
    let digested = digested?;
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
    // (a repeat of an earlier block with the same text is no second one)
    if replaceable && printed_in_place {
      frontmatter_drop_repeats(&tag, index);
    }
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
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #954): the Cut, placement step 0 for every binding — an affiliation,
    // alternate affiliation or address holding two or more entries led by marks the queued names request is queued
    // as one contact per entry, each labelled by its mark ([`cut_marked_contact`]); read again from here, so the
    // frontmatter observer's ledger records the entries too (`queue_front_matter`).
    if let Some(calls) = cut_marked_contact(&parenttag, &tag, kv.as_ref(), &content)? {
      unread(calls);
      return Ok(Vec::new());
    }
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #956): the Kind step for the addresses an affiliation-kind entry prints, after
    // the Cut, each of its pieces once: its lines of addresses are e-mails digested inside it, the entry's addresses,
    // which the Link step places (`kind_entry_addresses`, `insert::link_entry_addresses`).
    let content = kind_entry_addresses(&tag, kv.as_ref(), &content)?.unwrap_or(content);
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
  Let!(
    "\\lx@frontmatter@queue@annotate",
    "\\lx@annotate@frontmatter"
  );

  // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #954): where an uncut copy of a printed list the Cut cut later stood in the
  // queue (`frontmatter_api::cut::place_uncut_copies`): its place, an annotation's ordinal, recorded for the authors it
  // reached, whose entry of the list keeps it (`relocate::at_queue_position`).
  DefPrimitive!("\\lx@cut@copy@place {}{}{}", sub[(key, authors, zone)] {
    let ordinal = lookup_int(QUEUE_ORDINAL_COUNTER) + 1;
    assign_value(QUEUE_ORDINAL_COUNTER, Stored::Int(ordinal), Some(Scope::Global));
    let place = zone_offset(Some(zone.to_string().as_str())) + ordinal;
    for author in authors.to_string().split(',') {
      assign_value(&s!("{CUT_COPY_PLACE}{key}:author:{author}"), Stored::Int(place), Some(Scope::Global));
    }
    Ok(Vec::new())
  });

  // Rust-only, the emitter's (OD #980), in the shape of Perl's `\\lx@author@withsup`: an emitted item's content, whose marks
  // label nothing, its labels the reading's, for the item's digestion (no group of its own: `\\begin{CJK*}` in a name, 2609.05616); `@itemonly`, where an API call in it means nothing more either, its contact an
  // item of its own. Only what records and prints nothing goes on.
  DefMacro!("\\lx@frontmatter@item{}", "\\lx@frontmatter@item@start #1");
  DefMacro!(
    "\\lx@frontmatter@itemonly{}",
    "\\bgroup\\lx@frontmatter@item@start\\lx@frontmatter@itemonly@start #1\\egroup"
  );
  DefPrimitive!("\\lx@frontmatter@item@start", sub[()] {
    assign_value(EMITTING, Stored::Bool(true), Some(Scope::Local));
    // (no mark of the item read yet: its first is its label, a later one its text, `\lx@sup@setlabel@affiliation`)
    with_digesting_entry_attr(|attr| {
      attr.remove(author_macros::EMITTED_LINE_MARK);
    });
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@frontmatter@itemonly@start", sub[()] {
    let_i(&T_CS!("\\lx@annotate@frontmatter"), &T_CS!("\\lx@frontmatter@itemnone"), None);
    Ok(Vec::new())
  });
  // (the next annotation's place in the queue, which an emitted one takes from its source's: `relocate::at_queue_position`)
  DefPrimitive!("\\lx@frontmatter@queued{}", sub[(place)] {
    let place: i64 = place.to_string().parse().unwrap_or(1);
    assign_value(QUEUE_ORDINAL_COUNTER, Stored::Int(place - 1), Some(Scope::Global));
    Ok(Vec::new())
  });
  // (an entry as `\\lx@annotate@frontmatter` queues it, its lines of addresses read, on `@now`)
  DefMacro!("\\lx@frontmatter@itemkinded {}{} OptionalKeyVals:Frontmatter {}", sub[(parent, tag, kv, content)] {
    let content = kind_entry_addresses(&tag, kv.as_ref(), &content)?.unwrap_or(content);
    let mut call = vec![T_CS!("\\lx@annotate@frontmatter@now"), T_BEGIN!()];
    call.extend(parent.unlist());
    call.extend([T_END!(), T_BEGIN!()]);
    call.extend(tag.unlist());
    call.push(T_END!());
    if let Some(kv) = &kv {
      call.push(T_OTHER!("["));
      call.extend(kv.revert()?.unlist());
      call.push(T_OTHER!("]"));
    }
    call.extend([T_BEGIN!(), T_CS!("\\lx@frontmatter@item"), T_BEGIN!()]);
    call.extend(content.unlist());
    call.extend([T_END!(), T_END!()]);
    Ok(Tokens::new(call))
  });
  DefMacro!("\\lx@frontmatter@itemnone {}{}[]{}", sub[(parent, tag, keys, content)] {
    if tag.to_string() != "ltx:rdf" {
      return Ok(Tokens::new(Vec::new()));
    }
    let mut call = vec![T_CS!("\\lx@annotate@frontmatter@now"), T_BEGIN!()];
    call.extend(parent.unlist());
    call.extend([T_END!(), T_BEGIN!()]);
    call.extend(tag.unlist());
    call.extend([T_END!(), T_OTHER!("[")]);
    call.extend(keys.map(Tokens::unlist).unwrap_or_default());
    call.extend([T_OTHER!("]"), T_BEGIN!()]);
    call.extend(content.unlist());
    call.push(T_END!());
    Ok(Tokens::new(call))
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
          let label = clean_label(&n.to_string(), Some(labelseq)).into_owned();
          // (a label of the entry's place, which no mark printed: `SEQUENCE_LABEL`, OD #961)
          options.insert(SEQUENCE_LABEL.to_string(), label.clone());
          labels.push(label);
        }
    }
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #805): a contact of a sequenced list (`\lx@add@contact@placed[owner=sequence]`)
    // goes where this flush placed it ([`sequenced_contact_labels`]), else where `\lx@add@contact@placed` did.
    if options.remove("_sequence").is_some() {
      let at = lookup_int("lx_frontmatter_sequence_at");
      assign_value("lx_frontmatter_sequence_at", Stored::Int(at + 1), Some(Scope::Global));
      let placed = lookup_string("lx_frontmatter_sequence_labels");
      if let Some(label) = placed
        .split('\u{1}')
        .nth(usize::try_from(at).unwrap_or(usize::MAX))
        .filter(|label| !label.is_empty())
      {
        labels = vec![label.to_string()];
      }
    }
    // (whether the call names its entry itself, which an entry digested inside it of its kind does not override: `_name`
    // on the stub below)
    let name_given = options.contains_key("name");
    match get_frontmatter_name(options.get("name"), &tag, &role)? {
      Some(name) => { options.insert("name".to_string(), name); },
      None => { options.remove("name"); },
    }
    options.insert("_label".to_string(), labels.join(","));
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #954): the annotation's place in the queue, which a relocated one keeps
    // among its creator's annotations (user ruling R8, print order: `relocate::at_queue_position`) — the queue is
    // digested in order, so the n-th annotation digested is the n-th queued
    let ordinal = lookup_int(QUEUE_ORDINAL_COUNTER) + 1;
    assign_value(QUEUE_ORDINAL_COUNTER, Stored::Int(ordinal), Some(Scope::Global));
    // (in the print zone its binding declares, `_zone`: `relocate::NAMES_ZONE`)
    let zone = zone_offset(options.remove("_zone").as_deref());
    let mut in_note = false;

    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #956): an affiliation-kind entry's key, which the e-mails digested inside it
    // inherit (an `\email` in an `\institute` entry, an address line `kind_entry_addresses` made one), so the Link step
    // reads them as that entry's addresses, its carriers the authors holding it (`insert::link_entry_addresses`); and
    // whether a footnote symbol leads the entry, whose unspelled addresses are every carrier's (FIDELITY_CHARTER §4).
    let entry_key = (tag == "ltx:contact" && CUT_ROLES.contains(&role.as_str())).then(|| {
      let key = lookup_int(ENTRY_KEY_COUNTER) + 1;
      assign_value(ENTRY_KEY_COUNTER, Stored::Int(key), Some(Scope::Global));
      key.to_string()
    });
    // (or a footnote symbol its call's label is, authblk's `\affil[*]{a@x, b@y}`: the entry the names showing `*` carry,
    // as the frontmatter observer's reading takes a symbol label — 1907.01388)
    let symbol_label = options.get("label").is_some_and(|label| {
      let label = label.trim();
      !label.is_empty() && label.chars().all(events::is_fnsymbol)
    });
    let entry_symbol = entry_key.is_some() && (led_by_footnote_symbol(&content) || symbol_label);
    if let Some(entry) = &entry_key {
      options.insert(ENTRY_KEY.to_string(), entry.clone());
    } else if role == "email" {
      // (an e-mail digested inside such an entry: the entry's, `ENTRY_DIGESTING`)
      let digesting = lookup_string(ENTRY_DIGESTING);
      if let Some((entry, symbol)) = digesting.split_once(':') {
        options.insert(ENTRY_KEY.to_string(), entry.to_string());
        if symbol == "symbol" {
          options.insert(ENTRY_SYMBOL.to_string(), "true".to_string());
        }
      }
    }
    // Snapshot the parent entries (those not role=pending), inherit labels
    // from an enclosing pending entry, and push a tentative stub entry —
    // in case digestion changes labels!
    // (nor the author block's own creator, which the emitter writes below the flush's names: no annotation's parent by
    // place, as no creator was before relocation made it — only a block item reaches it, by the orphan rules,
    // `relocate_annotations`; [`BLOCK_CREATOR`], 2009.10916. Left out only once a flush wrote it, `block_creator_written`;
    // the list keeps both indices as entries are added, so this is a read, not a scan over every entry: `TagList`)
    let block_creator = block_creator_written();
    let (parent_indices, stub_idx) = with_value_mut("frontmatter", |val_opt| {
      if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt {
        let list = frnt.entry(parenttag.clone()).or_default();
        let parent_indices = list.parent_indices(block_creator);
        // IF this item encountered WITHIN another frontmatter
        // we inherit that frontmatter's labels (even override!)
        if let Some(last) = list.last()
          && last.attr.get("role").map(String::as_str) == Some("pending")
            && matches!(last.content.first(), Some(TagContent::PlaceKeeper))
          {
            let inherited = last.attr.get("_annotations").cloned().unwrap_or_default();
            // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #901): an e-mail inside a contact every author shares (llncs's
            // one `\institute`, `share_row_contacts`) is no author's by that: it keeps the label its owner gave it,
            // else it is the author block's — never every author's (the 85b review's p19; 2410.04091's addresses had all gone to every author).
            let shares = inherited.split(',').any(|label| label == SHARED_ROW_LABEL);
            let label = match options.get("_label") {
              _ if !shares => inherited,
              Some(own) if role == "email" && !own.is_empty() => own.clone(),
              _ if role == "email" => "addresses:block".to_string(),
              _ => inherited,
            };
            options.insert("_label".to_string(), label);
            // Rust-only: a contact of the kind and role of the entry it is digested in, naming none itself, takes the
            // name that entry gave — the label the block prints before it, which the frontmatter emitter writes on the
            // item whose content is the class binding's own call (`E-mail: \email{…}` on JHEP3's affiliation line, its
            // `\lx@add@email` the contact kept, the empty one around it dropped: 0812.0465), never the default name
            if !name_given
              && last.attr.get("_for") == Some(&tag)
              && last.attr.get("_role") == Some(&role)
              && let Some(name) = last.attr.get("_name")
            {
              options.insert("name".to_string(), name.clone());
            }
            // (a contact inside a name's note, `\thanks{\email{…}}`, prints in the footer, after every entry of the
            // block: its place is the footer's, `FOOTER_ZONE`)
            in_note = last.attr.get("_for").map(String::as_str) == Some("ltx:note");
          }
        let mut stub_attr = TagAttrs::default();
        stub_attr.insert("role".to_string(), "pending".to_string());
        stub_attr.insert("_annotations".to_string(),
          options.get("_label").cloned().unwrap_or_default());
        // (what the stub annotates: a contact digested inside it knows it is a note's, `in_note` above)
        stub_attr.insert("_for".to_string(), tag.clone());
        // (and the name it gives, when it gives one: inherited above by an entry of its kind digested inside it)
        if name_given && let Some(name) = options.get("name") {
          stub_attr.insert("_role".to_string(), role.clone());
          stub_attr.insert("_name".to_string(), name.clone());
        }
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
    options.insert(
      QUEUE_ORDINAL.to_string(),
      (if in_note { FOOTER_ZONE } else { zone } + ordinal).to_string(),
    );
    // (the call's own: every creator its entry is given to, and every copy of it, keep it; and whether the call came
    // after `\maketitle` placed the frontmatter — `drop_repeated_contacts`)
    if tag == "ltx:contact" {
      options.insert(CONTACT_SOURCE.to_string(), ordinal.to_string());
      if frontmatter_entry_is_late() {
        options.insert(CONTACT_LATE.to_string(), "true".to_string());
      }
    }
    let nparents = parent_indices.len();
    // (a contact led by a mark of its own, which labels no one — a legend, `$^\dagger$Corresponding author`, or a mark
    // no author carries — is no answer to an author's label request, nor passes over the authors one answers)
    let legend = marker_leads(&content);
    // (an entry of the Cut shows the mark that labels it while it is digested: `\lx@sup@setlabel@affiliation`,
    // OXIDIZED_DESIGN_DIVERGENCES #954)
    let mark_shown = options.remove("_markshown").is_some();
    if mark_shown {
      assign_value("lx_affiliation_mark_shown", Stored::Bool(true), Some(Scope::Global));
    }
    // (the entry being digested, whose e-mails are its addresses: its key while its content is digested, OD #956)
    let enclosing_entry = lookup_string(ENTRY_DIGESTING);
    if let Some(entry) = &entry_key {
      let symbol = if entry_symbol { "symbol" } else { "" };
      assign_value(ENTRY_DIGESTING, Stored::String(pin(s!("{entry}:{symbol}"))), Some(Scope::Global));
    }
    // (the title-page check's watch over a frontmatter command's entry: #953 "106t")
    let watch = begin_frontmatter_entry_digestion(Some(&content), None);
    let xcontent = digest_frontmatter_item(&tag, content);
    end_frontmatter_entry_digestion(watch, xcontent.as_ref().ok())?;
    if entry_key.is_some() {
      assign_value(ENTRY_DIGESTING, Stored::String(pin(enclosing_entry)), Some(Scope::Global));
    }
    if mark_shown {
      assign_value("lx_affiliation_mark_shown", Stored::Bool(false), Some(Scope::Global));
    }
    let xcontent = xcontent?;
    // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #803, #916; user rulings 2026-10-21, 2026-10-30): a note that prints
    // nothing is none, whatever its label. `\fnsep\thanks{\email{…}}` (webofc; aa's `\fnmsep\thanks{\email{…}}`) moves
    // the e-mail into the author's contact and left an empty `<note role="thanks"/>`, a visible mark opening a note with
    // nothing in it (2409.01693; 20 of 21 webofc papers); an author's own empty or blank `\thanks{}` is no note either
    // (pdflatex prints a bare mark: 2004.01582, 2401.15379, astro-ph0608669). Only material known to print nothing
    // counts ([`leaves_no_ink`]: `\thanks{\dots}`, `\thanks{\LaTeX}`, `\thanks{$\vdots$}` and `\thanks{\fbox{}}` stay;
    // the frontmatter observer reads a note by the same test, `Event::inkless`). The rule covers `ltx:note`s only, not
    // contact-typed notes (elsarticle's `\fntext[f1]{\ead{…}}`).
    if tag == "ltx:note" && leaves_no_ink(&xcontent) {
      DebugFeature!("frontmatter", "FRONT drop {tag}: it prints nothing");
      with_value_mut("frontmatter", |val_opt| {
        if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
          && let Some(list) = frnt.get_mut(&parenttag)
          && list.get(stub_idx).is_some_and(|e| e.attr.get("role").map(String::as_str) == Some("pending"))
        {
          list.remove(stub_idx);
        }
      });
      return Ok(Vec::new());
    }
    // Reset if changed (eg. by \lx@set@frontmatter@label during digestion)!
    // (and the mark an entry the frontmatter emitter writes printed: `\lx@sup@setlabel@affiliation`'s record on the stub,
    // [`PRINTED_LABEL`], which the Kind step for a legend reads — OD #980, P2 NT; none off. A call whose binding prints
    // its label, [`LABEL_PRINTS`], has its own: the call's `label=`)
    let (stub_label, printed) = with_value("frontmatter", |v| {
      if let Some(Stored::HashTagData(frnt)) = v
        && let Some(e) = frnt.get(&parenttag).and_then(|l| l.get(stub_idx))
      {
        (e.attr.get("_annotations").cloned(), e.attr.get(PRINTED_LABEL).cloned())
      } else {
        (None, None)
      }
    });
    let stub_label = stub_label.unwrap_or_default();
    options.insert("_label".to_string(), stub_label.clone());
    if let Some(printed) = printed
      && !options.contains_key(LABEL_PRINTS)
    {
      options.entry(PRINTED_LABEL.to_string()).or_insert(printed);
    }
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
            // answered by a mark-picked contact of the role (`_bymark`, `\lx@affiliation@withsup`; or one its printed
            // mark labels, `_plainmarks`/`_picturemarks`, as `marked_at` below reads it — a class's in-place
            // `\affil{$^1$Univ A}`, so `\author{ANN ABLE$^1$ and BOB BAKER \affil{$^1$Univ A} \affil{Univ B}}` gives
            // Univ B to Bob Baker alone, as the observer's `Resolver::record_run`, review rev94a r7 SF2) — one deferred
            // until the labels are compared, or one this flush digests later (`lx_frontmatter_offered_labels`) — which
            // has not reached it yet: `\author{Ann$^{1}$}\affil{$^1$Univ X}\author{Bob \affil{Univ Y}}` gave Ann Y as
            // well, 77n r4 review probes e1, e4; a label nothing answers, `\author{Ann$^{1}$}\affil{$^{1-3}$Univ X}`, is none
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
                    && ["_bymark", "_plainmarks", "_picturemarks"]
                      .iter()
                      .any(|key| e.attr.contains_key(*key))
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
            // (nor is an author made before a deferred contact of the role a printed mark labels (`_bymark`, or the
            // marks it requested, `_plainmarks`/`_picturemarks`) new: the contact waits for its label, so its owner
            // holds no role yet — a class's in-place `\affil` in `\author{ANN ABLE$^1$ \affil{$^1$Univ A} BOB BAKER
            // \affil{Univ B}}` gave Ann Univ B too, IEEEtran's `\IEEEauthorblockA{\IEEEauthorrefmark{1}Univ X}` gave
            // the author above it the next block's `\IEEEauthorblockA{Univ Y}` (`annotate=new`; review rev94a r6 B1, the
            // observer's `Resolver::ends_a_run`). Not a legend's own walk, the whole run's, nor a `label=`'s: authblk's
            // `\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}` prints Univ Y under both (2605.25017). With no
            // author made since that contact the bound is none: every author of the run is answered, the contact the
            // whole run's — the walk below, 77n r5 review probes t02, r01.)
            let marked_at = if (run_new || newonly) && !legend {
              list
                .iter()
                .rposition(|e| {
                  e.attr.get("role").map(String::as_str) == Some("pending")
                    && ["_bymark", "_plainmarks", "_picturemarks"]
                      .iter()
                      .any(|key| e.attr.contains_key(*key))
                    && matches!(e.content.first(),
                      Some(TagContent::Entry(contact)) if contact.attr.get("role") == Some(&role))
                })
                .filter(|at| parent_indices.last().is_some_and(|last| last > at))
            } else {
              None
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
            // answers, so while every author it reached is answered it goes on: when every author of the run is
            // answered the contact is the whole run's as Perl's `_has{role}` test gives it — ptephy's
            // `\affil{}{Univ Z}` after `\affil{1}{…} \affil{2}{…}`, which the class prints under all of them; 77n r5
            // review probes t02, t04, o04. A last author answered does not make every author of the run answered:
            // `ANN ABLE$^1$ and BOB BAKER and CID COLE$^1$ \affil{$^1$Univ A} \affil{Univ B}` gives Univ B to CID
            // COLE too, before his Univ A — a known residual, review rev94a r8 u04, z14, RED catalog repro
            // `raw_class_affil_run_trailing_answered_name_takes_the_unmarked_line`)
            let mut reached_unanswered = false;
            let mut indices = parent_indices.clone();
            while let Some(pi) = indices.pop() {
              if nprev <= 0 {
                break;
              }
              nprev -= 1;
              let Some(mut parent) = list.get_mut(pi) else {
                continue;
              };
              attached += 1;
              parent.content.push(datum.clone());
              if !role.is_empty() {
                reached_unanswered |= !answered(&parent);
                parent.attr.insert(has_role_key.clone(), "1".to_string());
                drop(parent);
                let Some(&next_pi) = indices.last().filter(|_| newonly) else {
                  continue;
                };
                if list.get(next_pi).is_some_and(|next| next.attr.contains_key(&has_role_key))
                  || marked_at.is_some_and(|at| next_pi < at)
                {
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
  // A note the operand holds is no label: the class prints the operand in a superscript (`\def\inst#1{\unskip$^{#1}$}`,
  // llncs.cls:894, aa.cls:759-760), so its `\thanks` runs there, the note of the name it is printed on — read after the
  // request, as a name's `$^{…}$` mark reads one (`\lx@sup@request@affiliation`, [`split_mark_notes`]); `\fnmsep`
  // is a comma: the one aa prints there (aa.cls:807-808, an `\hbox` in `\inst`'s math); under llncs, a pdflatex error
  // there (llncs.cls:894-896, its `$^,$` inside `$^{…}$`; the rev98b review's pdx4), read the same. `\inst{2,\thanks{NASA Sagan Fellow}}` (aa 2303.08174,
  // 2311.02521, 2601.20473; llncs 1808.09829) had requested the note's text as a label and lost it (Perl the same,
  // aa.cls.ltxml:50; OXIDIZED_DESIGN_DIVERGENCES #951).
  DefPrimitive!("\\lx@request@frontmatter@annotation[]{}", sub[(prefix, label)] {
    let prefix = prefix.as_ref().map(ToString::to_string).unwrap_or_default();
    let (label, notes) = match split_mark_notes(label.unlist_ref()) {
      Some((mark, notes)) => (
        Tokens::new(
          mark
            .into_iter()
            .map(|t| if t == T_CS!("\\fnmsep") { T_OTHER!(",") } else { t })
            .collect(),
        ),
        Some(Tokens::new(notes)),
      ),
      None => (label, None),
    };
    // (keyed by the operand's printed text: FIDELITY_CHARTER §4, refactor stage 4, OXIDIZED_DESIGN_DIVERGENCES #919;
    // none while a label's own operand is read)
    if reading_label() {
      return Ok(Vec::new());
    }
    let (label, pictured) = frontmatter_label_request(
      &label,
      if prefix.is_empty() { "LABEL" } else { &prefix })?;
    add_annotation_labels(label.join(","), false, pictured);
    if let Some(notes) = notes {
      unread(notes);
    }
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
  Ok(())
}

/// Whether a name's last word is one its period belongs to: an initial (a single capital, `K`) or a suffix written with
/// a period (`Jr`, `Esq`, [`NAME_SUFFIXES`]). A two-letter word is a surname as often as an initial (`Wei Li.`, `Andrew
/// Ng.`: sentence punctuation), so its period goes, as Perl's.
pub(in crate::base_utilities) fn ends_with_abbreviation(name: &str) -> bool {
  let Some(word) = name.split_whitespace().next_back() else {
    return false;
  };
  let mut chars = word.chars();
  let initial = chars.next().is_some_and(char::is_uppercase) && chars.next().is_none();
  // (and "et al." ending a list, `is_name_qualifier`)
  initial || NAME_SUFFIXES.contains(&format!("{word}.").as_str()) || word == "al" || word == "et.al"
}
