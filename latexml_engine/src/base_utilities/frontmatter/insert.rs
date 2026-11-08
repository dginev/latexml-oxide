//! Inserting the front matter into the document: creators, contacts, late frontmatter, unanswered marks.

use super::*;
use crate::base_utilities::defs::frontmatter_api::{LABEL_PRINTS, PRINTED_LABEL};

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
  link_entry_addresses(document, None)?;
  respell_emails(document)?;
  let unshown_marks = show_unanswered_marks(document, &answered)?;
  drop_empty_contacts(document)?;
  drop_repeated_orcids(document);
  // (FM S2b: a written family's paper, where it differs from the observer's reading by the writer's operations alone,
  // written as the reading has it — under the shadow switch only; OXIDIZED_DESIGN_DIVERGENCES #837 "S2b")
  write_observed_frontmatter(document)?;
  count_lost_marks(document, &unshown_marks);
  // (an entry's line of addresses, read once every route has placed the entry and the observer's writing is done: its
  // addresses the entry's e-mails, placed by the Link step; OD #966)
  join_split_address_contacts(document)?;
  let kinded = kind_address_lines(document)?;
  if !kinded.is_empty() {
    link_entry_addresses(document, Some(&kinded))?;
    drop_empty_contacts(document)?;
  }
  // (a list of addresses one contact prints for several people, each its owner's e-mail: census138 ADDR-b, OD #972)
  distribute_address_lists(document)?;
  // (an e-mail's correspondence label, read once every address is placed: its owner's note, review rev107c N4)
  write_role_notes(document)?;
  if !kinded.is_empty() {
    show_line_marks(document)?;
  }
  // (an affiliation's dedication lines, read once every route has placed it: the paper's dedication, 107c)
  kind_dedication_lines(document)?;
  // (an affiliation that is a legend, read once every route has placed it: its carriers' note, OD #961)
  kind_legend_contacts(document)?;
  // (an affiliation that is a link or a membership grade, read where the legend's is: the block's link, its carriers'
  // grade, OD #969)
  kind_link_and_grade_contacts(document)?;
  // (a contact a creator holds twice, once every route and Kind step has placed it: its first, OD #969)
  drop_repeated_contacts(document);
  // (the authors as written: a name the writer re-parted holds one person, OD #928)
  flag_merged_creators(document)?;
  // (the unclaimed-text net's notes: their lines no frontmatter field and no text already set holds)
  keep_unclaimed_net_notes(document)?;
  // (the content invariant on the placed output, under the emitter's `=2`: every printed word placed once or counted, OD #980)
  check_placed_content(document);
  // (the census's read-back of the final document against the reading: the plan's definition of done, item 5. Every pass
  // after the writer is classed by what it may change in `perfect_kernel_batche0`'s allow-list, a debt: read that before
  // adding one)
  read_back_frontmatter(document);
  Ok(())
}

/// The words a note line counts for [`keep_unclaimed_net_notes`] (the frontmatter scorer's, fm_scorer/score_tiers.py
/// `content_words`, and its stop words): its letter runs of two letters or more outside the stop words and its digit
/// runs of three or more, folded — accents dropped ([`resolver::base_letters`]), lower-cased — in printed order. Beyond
/// the scorer's ASCII runs, a run of any script's letters is a word (`Москва`), and each Han ideograph or kana, of
/// scripts written without spaces, is a word of its own (2206.01007's `北京师范大学天文系`, 2601.14329's
/// `北京航空航天大学`; 105a r2).
pub(super) fn net_content_words(text: &str) -> Vec<String> {
  const STOPWORDS: &[&str] = &[
    "a", "an", "the", "of", "and", "or", "in", "on", "at", "for", "to", "from", "by", "with",
    "via", "de", "la", "le", "les", "du", "des", "del", "della", "di", "da", "do", "dos", "das",
    "e", "y", "et", "und", "von", "van", "der", "den", "zu", "im", "am", "al", "el", "en", "is",
    "are", "be", "as", "this", "that", "its", "it", "we", "our", "their", "his", "her", "no",
    "not",
  ];
  let folded = resolver::base_letters(text).to_lowercase();
  let mut words = Vec::new();
  let mut run = String::new();
  let mut letters = false;
  let flush = |run: &mut String, letters: bool, words: &mut Vec<String>| {
    let keep = if letters {
      run.chars().count() >= 2 && !STOPWORDS.contains(&run.as_str())
    } else {
      run.len() >= 3
    };
    if keep {
      words.push(std::mem::take(run));
    } else {
      run.clear();
    }
  };
  for c in folded.chars() {
    if unspaced_script(c) {
      if !run.is_empty() {
        flush(&mut run, letters, &mut words);
      }
      words.push(c.to_string());
      continue;
    }
    let kind = if c.is_alphabetic() {
      Some(true)
    } else if c.is_ascii_digit() {
      Some(false)
    } else {
      None
    };
    if kind != Some(letters) && !run.is_empty() {
      flush(&mut run, letters, &mut words);
    }
    if let Some(kind) = kind {
      letters = kind;
      run.push(c);
    }
  }
  if !run.is_empty() {
    flush(&mut run, letters, &mut words);
  }
  words
}

/// Is `c` a Han ideograph or a kana — a letter of a script written without spaces between its words, each counted as a
/// word of its own ([`net_content_words`])?
fn unspaced_script(c: char) -> bool {
  matches!(c,
    '\u{3040}'..='\u{30FF}' // Hiragana, Katakana
    | '\u{31F0}'..='\u{31FF}' // Katakana phonetic extensions
    | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
    | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
    | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
    | '\u{20000}'..='\u{3134F}' // CJK Unified Ideographs Extensions B-G
  )
}

/// The text of `node` without what the nodes `skip` hold.
fn text_outside(node: &Node, skip: &[Node], out: &mut String) {
  for child in node.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) {
      if !skip.contains(&child) {
        text_outside(&child, skip, out);
        out.push(' ');
      }
    } else {
      out.push_str(&child.get_content());
    }
  }
}

/// The text of `node` without the text of an `ltx:ERROR` at any depth in it (an error's text is no printed word: an
/// undefined `\songti`'s name in the group `{\songti\zihao{-4}#2}`, physics_report.sty, 2601.14329).
fn text_without_errors(node: &Node, out: &mut String) {
  if node.get_type() == Some(NodeType::ElementNode) {
    if node.get_name() != "ERROR" {
      for child in node.get_child_nodes() {
        text_without_errors(&child, out);
      }
    }
  } else {
    out.push_str(&node.get_content());
  }
}

/// A note's lines ([`keep_unclaimed_net_notes`]): what stands between its `ltx:break`s, each `ltx:p` apart, with the
/// break that ends it; the text of an `ltx:ERROR` in it, at any depth, left out ([`text_without_errors`]).
fn net_note_lines(parent: &Node, lines: &mut Vec<(Vec<Node>, String)>) {
  let mut line: (Vec<Node>, String) = (Vec::new(), String::new());
  for child in parent.get_child_nodes() {
    let element = child.get_type() == Some(NodeType::ElementNode);
    if element && child.get_name() == "break" {
      line.0.push(child);
      lines.push(std::mem::take(&mut line));
    } else if element && child.get_name() == "p" {
      if !line.0.is_empty() {
        lines.push(std::mem::take(&mut line));
      }
      net_note_lines(&child, lines);
    } else {
      text_without_errors(&child, &mut line.1);
      line.1.push(' ');
      line.0.push(child);
    }
  }
  if !line.0.is_empty() {
    lines.push(line);
  }
}

/// Remove the `ltx:break`s a note starts or ends with, blank text aside (the lines [`keep_unclaimed_net_notes`] took out
/// leave them: `\\AB@affillist` starts with its `\\\\`).
fn trim_edge_breaks(document: &mut Document, note: &Node) {
  let blank =
    |n: &Node| n.get_type() != Some(NodeType::ElementNode) && n.get_content().trim().is_empty();
  for from_end in [false, true] {
    loop {
      let mut children = note.get_child_nodes();
      if from_end {
        children.reverse();
      }
      match children.into_iter().find(|n| !blank(n)) {
        Some(edge)
          if edge.get_type() == Some(NodeType::ElementNode) && edge.get_name() == "break" =>
        {
          document.remove_node(edge);
        },
        _ => break,
      }
    }
  }
}

/// The classification codes `text` prints, folded ([`resolver::base_letters`], lower-cased), which the net keeps below
/// its floor ([`keep_unclaimed_net_notes`]): a piece of digits and letters, a digit in it, its letters at most 3 to a
/// run, parted by dots or dashes or run together with the digits — a PACS code `98.80.Cq` (europhys `\Pacs`,
/// cond-mat9812194), an MSC code `20N05` (ourlematema `\MSC`, 1909.06690; basm `\MSC`, 1601.01922), an ACM class
/// `H.2.4`; not a bare number (OD #953, 105a).
fn code_pieces(text: &str) -> Vec<String> {
  resolver::base_letters(text)
    .to_lowercase()
    .split(|c: char| c.is_whitespace() || ",;:()[]".contains(c))
    .map(|piece| piece.trim_matches(|c: char| c == '.' || c == '-'))
    .filter(|piece| {
      let runs: Vec<&str> = piece
        .split(['.', '-'])
        .filter(|run| !run.is_empty())
        .collect();
      let letter_runs_short = runs.iter().all(|run| {
        run
          .split(|c: char| c.is_ascii_digit())
          .all(|letters| letters.len() <= 3)
      });
      piece.chars().any(|c| c.is_ascii_digit())
        && piece
          .chars()
          .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && letter_runs_short
        && (runs.len() > 1 || piece.chars().any(|c| c.is_ascii_alphabetic()))
    })
    .map(ToString::to_string)
    .collect()
}

/// Is the content word `word` ([`net_content_words`], folded) a date's — a month's name, a year — or part of a code
/// `codes` holds ([`code_pieces`])? The words the net keeps below its floor ([`keep_unclaimed_net_notes`]).
fn code_or_date_word(word: &str, codes: &[String]) -> bool {
  date_word(word) || codes.iter().any(|code| code.contains(word))
}

/// The label a code's or a date's line leads with, of its content words `words` ([`net_content_words`], in printed
/// order): a dateline's (received, accepted, revised, published, submitted; any case, a colon after it or not) before a
/// date's word ([`date_word`]: "Received: 2021", "Accepted 2021"), or a classification's (MSC, PACS; a year after it or
/// not) on a line that prints a code `codes` holds ("MSC 2020: 05C12"). Kept below the floor with the line's code or
/// date words ([`keep_unclaimed_net_notes`]); a leading word of no such list is none ("ICML 2021", "Room 101"; 105a r2).
fn leading_label<'w>(words: &'w [String], codes: &[String]) -> Option<&'w str> {
  const DATELINE: &[&str] = &["received", "accepted", "revised", "published", "submitted"];
  let (first, rest) = words.split_first()?;
  let label = if DATELINE.contains(&first.as_str()) {
    rest.iter().any(|word| date_word(word))
  } else {
    matches!(first.as_str(), "msc" | "pacs") && !codes.is_empty()
  };
  label.then_some(first.as_str())
}

/// Is the content word `word` ([`net_content_words`], folded) a date's: an English month's name, whole or shortened, or
/// a year (1800-2100)? A month of another language is none (`mars 2021`, counted; the RED repro
/// `net_non_english_month_line_red`).
fn date_word(word: &str) -> bool {
  const MONTHS: &[&str] = &[
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
    "jan",
    "feb",
    "mar",
    "apr",
    "jun",
    "jul",
    "aug",
    "sep",
    "sept",
    "oct",
    "nov",
    "dec",
  ];
  MONTHS.contains(&word) || (word.len() == 4 && matches!(word.parse::<u32>(), Ok(1800..=2100)))
}

/// The unclaimed-text net's notes (frontmatter_stores.rs `hand_net_stores`: what a raw class's title page prints of a
/// store no frontmatter owner claims, OXIDIZED_DESIGN_DIVERGENCES #953), once the frontmatter is placed: each note line
/// (split at `ltx:break` and `ltx:p`, [`net_note_lines`]) is read by its distinct content words
/// ([`net_content_words`]) that neither the frontmatter nor the text set before it holds — the whole document's text but
/// the net's notes: the deposit's paragraphs, the creators (`\AB@names` repeats the names), the abstract — nor the
/// class's own code in the store (`net_class_text`). A line of e-mail addresses alone ([`resolver::only_addresses`]) is
/// the e-mail contacts of the note's creator — the block's nameless one or the sole author (OD #159), no invented owner
/// — each address no text kept holds, whether or not its words are used elsewhere (aaskaiid.sty's `\email@toks`,
/// 2606.27706, 2607.25721). Else a line with no such word goes (it repeats what is kept); a line is kept whole with 3 or
/// more, or with fewer when they are all a code's or a date's ([`code_or_date_word`]) or the label such a line leads
/// with ([`leading_label`]); otherwise it goes, its words counted. A note left without a line goes, with a nameless
/// creator it leaves empty. A store's kept lines raise one Warning, and its lost words one more, in printed order
/// (FIDELITY_CHARTER §4: printed text no owner claims, kept beside the frontmatter or lost, counted — 2103.14934's
/// "corresponding authors" below the floor; 105a).
fn keep_unclaimed_net_notes(document: &mut Document) -> Result<()> {
  let notes = document.findnodes("//ltx:note[@_net]", None);
  if notes.is_empty() {
    return Ok(());
  }
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(());
  };
  let mut claimed_text = String::new();
  text_outside(&root, &notes, &mut claimed_text);
  let claimed: HashSet<String> = net_content_words(&claimed_text).into_iter().collect();
  let mut written: Vec<String> = resolver::addresses_in(&claimed_text);
  let claimed_folded = resolver::base_letters(&claimed_text).to_lowercase();
  let email_name = get_frontmatter_name(None, "ltx:contact", "email")?;
  for mut note in notes {
    let store = note.get_attribute("_net").unwrap_or_default();
    // (and the words of the class's own code in the store, frontmatter_stores.rs `net_class_text`, as a multiset: each
    // occurrence in the class's text accounts for one in the line, so a label's `单位` does not take the document's
    // own `单位` with it — each Han ideograph is a word, `作者单位：#1` around `单位甲`; review rev105a3 finding 1)
    let mut class: HashMap<String, usize> = HashMap::default();
    for word in net_content_words(&lookup_string(&s!("lx_net_class_text_{store}"))) {
      *class.entry(word).or_default() += 1;
    }
    let mut lines = Vec::new();
    net_note_lines(&note, &mut lines);
    let mut kept = 0usize;
    let mut lost: Vec<String> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();
    for (nodes, text) in lines {
      let words = net_content_words(&text);
      // (distinct, in printed order)
      let mut unmatched: Vec<String> = Vec::new();
      let mut class_left = class.clone();
      for word in &words {
        if claimed.contains(word) {
          continue;
        }
        if let Some(left) = class_left.get_mut(word)
          && *left > 0
        {
          *left -= 1;
          continue;
        }
        if !unmatched.contains(word) {
          unmatched.push(word.clone());
        }
      }
      let listed = resolver::addresses_in(&text);
      // (its codes no text kept prints: an MSC code's letters and digits are no content words, `20N05`)
      let codes: Vec<String> = code_pieces(&text)
        .into_iter()
        .filter(|code| !claimed_folded.contains(code.as_str()))
        .collect();
      let label = leading_label(&words, &codes);
      // (an address line: its addresses no kept text holds, whether or not their words are used elsewhere — a title's
      // word in the domain; one every address of which is written repeats what is kept)
      if !listed.is_empty() && resolver::only_addresses(&text) {
        for address in listed {
          if !written.contains(&address) {
            written.push(address.clone());
            addresses.push(address);
          }
        }
      } else if unmatched.len() >= 3
        || ((!unmatched.is_empty() || !codes.is_empty())
          && unmatched
            .iter()
            .all(|word| Some(word.as_str()) == label || code_or_date_word(word, &codes)))
      {
        kept += 1;
        continue;
      } else {
        for word in unmatched {
          if !lost.contains(&word) {
            lost.push(word);
          }
        }
      }
      for node in nodes {
        document.remove_node(node);
      }
    }
    // (the breaks left at either end of a kept line's run, and the paragraphs left empty)
    for p in document.findnodes(".//ltx:p", Some(&note)) {
      if element_nodes(&p).is_empty() && p.get_content().trim().is_empty() {
        document.remove_node(p);
      }
    }
    let creator = note.get_parent();
    // (the addresses, the note's creator's e-mail contacts where the note stands)
    if let Some(creator) = creator.as_ref() {
      let saved = document.get_node().clone();
      document.set_node(creator);
      for address in &addresses {
        let mut attributes = HashMap::default();
        attributes.insert("role".to_string(), "email".to_string());
        if let Some(name) = email_name.as_ref() {
          attributes.insert("name".to_string(), name.clone());
        }
        let mut contact = document.open_element("ltx:contact", Some(attributes), None)?;
        contact.append_text(address).ok();
        document.close_element("ltx:contact")?;
        contact.unlink_node();
        note.add_prev_sibling(&mut contact).ok();
      }
      document.set_node(&saved);
    }
    if !lost.is_empty() {
      // (named once: the title-page check counts no word named here again, `title_page_check`)
      note_net_counted(&lost);
      Warn!(
        "frontmatter",
        "unclaimed",
        s!(
          "The title page prints \\{store} with {} word(s) no frontmatter field holds, too few to keep as a note: {}",
          lost.len(),
          lost.join(" ")
        )
      );
    }
    if kept == 0 {
      document.remove_node(note);
      if addresses.is_empty()
        && let Some(creator) = creator
        && creator.get_name() == "creator"
        && element_nodes(&creator).is_empty()
      {
        document.remove_node(creator);
      }
      continue;
    }
    trim_edge_breaks(document, &note);
    note.remove_attribute("_net").ok();
    Warn!(
      "frontmatter",
      "unclaimed",
      s!(
        "The title page prints {kept} line(s) of \\{store} that no frontmatter field holds: kept as a note of the author block"
      )
    );
  }
  Ok(())
}

/// The Kind step for a legend (FIDELITY_CHARTER §4, author annotations are notes; census129 cluster 6;
/// OXIDIZED_DESIGN_DIVERGENCES #961), where every route that makes an affiliation meets — once the frontmatter is
/// placed, its addresses linked and the frontmatter observer's writing done: authblk's labelled `\affil[*]{Corresponding
/// author}` (under article, wlscirep, googledeepmind, osajnl), an `\institute` entry the Cut made
/// (`\textsuperscript{$\dagger$}Joint first authors.`, 2608.22772), an author block's line (`$^\ast$ Corresponding
/// author.`, 2310.15190, 2111.01415; `$\dagger$Project lead`, 2609.18514, 2609.19719), IEEEtran's
/// `\IEEEauthorblockA{\IEEEauthorrefmark{4}These authors contributed equally}` (2410.12184) — a creator's affiliation
/// contact whose whole text, its addresses aside, is a legend ([`entry_legend_kind`]: correspondence, a contribution, an
/// author deceased; no institution) is its carriers' note in its place (`ltx:note role=thanks`, `ltx_thanks_<kind>`), as
/// a legend line's note is (`queue_digest::legend_note_kind`, `resolver::is_legend_text`). Its mark is one that prints:
/// the superscript its text opens with, else a label it was placed by (`_label`) that a carrier's name shows
/// (IEEEtran's `\IEEEauthorrefmark{4}`'s `4`), that a printed mark gave, or that its binding prints ([`LABEL_PRINTS`]:
/// authblk's `*`); a binding's key (icml's `intern`, 2106.09776) or an entry's place ([`SEQUENCE_LABEL`]: svjour's
/// `affiliation:2`, 1507.04774) is no mark (review rev103c2 B-A). Its carriers stay the contact's, and so do the
/// e-mails its line printed (2409.05386's `ago@`, the author's e-mail contact as before). Left as they are: a legend
/// that prints no mark
/// (`\affil[ ]{Corresponding author: …}`, 2412.14916, 2305.02090: a note shows its text only behind its mark, so an
/// unmarked one is unreachable on the rendered page; review rev103c B1), a contact the class names otherwise than an
/// affiliation (a `name` but the role's default), an institution named with a legend's words ("Department of Equal
/// Contribution Studies, Univ C"), a mixed line (`Facebook (work done while at JHU/APL)`, 1702.02684; `DeepAuto.ai († :
/// Equal advising)`, 2609.34327; "Equal contribution, Google DeepMind", a name no carrier has). Guards
/// `perfect_kernel_batch103c`.
fn kind_legend_contacts(document: &mut Document) -> Result<()> {
  let contacts = document.findnodes("//ltx:creator/ltx:contact[@role='affiliation']", None);
  if contacts.is_empty() {
    return Ok(());
  }
  let default_name = get_frontmatter_name(None, "ltx:contact", "affiliation")?;
  // (its printed text, a line break at any depth parting its lines, a mark set apart, a note inside it — a
  // `\footnotemark`'s — none of it: `entry_legend_kind` reads it without its addresses, which stay where they print)
  let mut read = Vec::new();
  for contact in contacts {
    if contact.get_attribute("name") != default_name {
      continue;
    }
    let mut text = String::new();
    legend_text(&contact, &mut text);
    if text.chars().any(char::is_alphabetic) {
      read.push((contact, text));
    }
  }
  // (the entry's carriers, each creator holding a copy of its text: their personnames as printed, which a name in the
  // legend must spell, review rev103c S2; a nameless creator — an author block's own, holding the block's legends —
  // carries for every author: "corresponding author(s): Xiaofan Xu, Hua Zhu", 2609.08556; 1507.04774, 2503.10710)
  let authors: Vec<String> = document
    .findnodes("//ltx:creator/ltx:personname", None)
    .iter()
    .map(Node::get_content)
    .collect();
  // (the author creators, a nameless creator's carriers)
  let named_creators = document.findnodes("//ltx:creator[ltx:personname]", None);
  let mut carriers: HashMap<&str, Vec<String>> = HashMap::default();
  for (contact, text) in &read {
    let name = contact.get_parent().and_then(|creator| {
      element_nodes(&creator)
        .into_iter()
        .find(|child| child.get_name() == "personname")
    });
    let entry = carriers.entry(text.as_str()).or_default();
    match name {
      Some(personname) => entry.push(personname.get_content()),
      None => entry.extend(authors.iter().cloned()),
    }
  }
  // (each printed text read once: every holder's copy of an entry has its text and its carriers — a collaboration's
  // authblk `\affil[N]` under hundreds of names, 2005.08745; P2r2's cost gate)
  let mut kinds: HashMap<&str, Option<&'static str>> = HashMap::default();
  for (contact, text) in &read {
    let mark = legend_mark(document, contact, &named_creators);
    // (a legend that prints no mark stays the affiliation it was: a note's text shows only behind its mark, and an empty
    // one shows nothing to reach it by — `\affil[ ]{Corresponding author: …}`, 2412.14916, 2305.02090; review rev103c
    // B1, main-session ruling 2026-11-04)
    if mark.is_empty() {
      continue;
    }
    // (or a dedication's words behind a name's mark, which the Kind step for a dedication leaves to it: the carriers'
    // note, as "Deceased" is — `$^\dagger$In memory of Bob Baker, …` under Bob Baker$^{1,\dagger}$, review rev107c S1)
    let Some(kind) = *kinds.entry(text.as_str()).or_insert_with(|| {
      entry_legend_kind(text, &carriers[text.as_str()])
        .or_else(|| opens_a_dedication(text).then_some("note"))
    }) else {
      continue;
    };
    DebugFeature!(
      "frontmatter",
      "FRONT kind affiliation: a legend, its carriers' {kind} note: {text}"
    );
    let mut note = document.rename_node(contact.clone(), "ltx:note", true)?;
    note.remove_attribute("name").ok();
    document.set_attribute(&mut note, "role", "thanks")?;
    document.set_attribute(
      &mut note,
      "class",
      &s!("ltx_note_frontmatter ltx_thanks_{kind}"),
    )?;
    document.set_attribute(&mut note, "mark", &mark)?;
  }
  Ok(())
}

/// The mark a legend contact prints ([`kind_legend_contacts`]; empty for none): the superscript its text opens with,
/// else a key of a label it was placed by that a printed mark gave, that a carrier's name shows or requests, or that its
/// binding prints ([`LABEL_PRINTS`]) — `named_creators` the author creators, a nameless creator's carriers. Read by the
/// Kind step for a dedication too ([`kind_dedication_lines`]: a line a name's mark points to is no dedication).
fn legend_mark(document: &mut Document, contact: &Node, named_creators: &[Node]) -> String {
  // (the superscript its text opens with, as printed)
  let printed = element_nodes(contact)
    .first()
    .filter(|first| {
      first.get_name() == "sup"
        && contact
          .get_first_child()
          .is_some_and(|child| child == **first || child.get_content().trim().is_empty())
    })
    .map(|sup| sup.get_content().trim().to_string())
    .filter(|mark| !mark.is_empty());
  // (else a key of a label it was placed by that a printed mark gave or that prints, whole — `*†` of 2602.19190's
  // `$^{*\dag}$Project lead: Yi Yang (…)`; review rev103c2 B-A): an `affiliation:K` label read off a printed
  // superscript (the marks path, the Cut, the observer's writer: 2609.03342, 2503.10710) but not one of the entry's
  // place (`SEQUENCE_LABEL`: svjour's `\institute{… \and …}`, 1507.04774), a key a carrier's name shows or requests
  // by its superscript (IEEEtran's `\IEEEauthorrefmark{4}` as `Tayal<sup>4</sup>`, 2410.12184), or a label its
  // binding prints before the line (`LABEL_PRINTS`: authblk's `\affil[*]`, jheppub's `\affiliation[*]`, 2204.11919;
  // cas's letter, 2303.06763) — the carrier's own among several (`\affil[1,2]`: Ann's "1", Bob's "2", 2301.13111).
  // A binding's key prints no mark (icml's `\icmlaffiliation{intern}{…}`, whose number icml prints: 2106.09776): no
  // mark, so it stays an affiliation.
  printed
    .or_else(|| {
      // (each key, and whether a printed mark gave it: an `affiliation:K` label not of the entry's place)
      let place = contact.get_attribute(SEQUENCE_LABEL).unwrap_or_default();
      let mut keys: Vec<(String, bool)> =
        clean_frontmatter_labels(&contact.get_attribute("_label")?, "")
          .into_iter()
          .filter_map(|label| match label.split_once(':') {
            Some(("affiliation", key)) => Some((label_key(key), label != place)),
            None => Some((label_key(&label), false)),
            _ => None,
          })
          .filter(|(key, _)| !key.is_empty())
          .collect();
      let creator = contact.get_parent()?;
      let printed_label = contact.get_attribute(PRINTED_LABEL);
      let prints = printed_label.is_some() || contact.get_attribute(LABEL_PRINTS).is_some();
      // (an entry the frontmatter emitter wrote is placed by the reading's label, so the label that prints is its own
      // key, `PRINTED_LABEL`: the call's `label=` its binding prints (authblk's `\affil[*]`, 2605.31330), or the first
      // mark the entry's content printed (`\thanks{\textsuperscript{1}These authors contributed equally …}`); OD #980,
      // P2 NT)
      if let Some(printed) = printed_label {
        keys.splice(
          0..0,
          clean_frontmatter_labels(&printed, "")
            .into_iter()
            .map(|label| label_key(label.split_once(':').map_or(label.as_str(), |(_, key)| key)))
            .filter(|key| !key.is_empty())
            .map(|key| (key, true)),
        );
      }
      let named = element_nodes(&creator)
        .iter()
        .any(|child| child.get_name() == "personname");
      let holders = if named {
        std::slice::from_ref(&creator)
      } else {
        named_creators
      };
      // (the marks the carriers' names show, and those they request by a printed superscript — `affiliation:K`, read
      // as the name prints it, `\lx@sup@request@affiliation`; a binding's `annotations={…}` may be keys)
      let mut shown: Vec<String> = Vec::new();
      // (an author reference mark's number, as its label keys it, and the symbol it prints: `relocate::sup_marks`;
      // IEEEtran's `\IEEEauthorrefmark{4}` keys 4 and prints `§`, the note's mark, 2410.12184)
      let mut refmarks: Vec<(String, String)> = Vec::new();
      for holder in holders {
        // (each of a superscript's marks: `Bridgeford<sup>1,†</sup>`, 2503.10710)
        for sup in document.findnodes("ltx:personname//ltx:sup", Some(holder)) {
          shown.extend(sup.get_content().split(',').map(mark_form));
          if let Some(key) = sup.get_attribute("_refmark") {
            shown.push(mark_form(&key));
            refmarks.push((mark_form(&key), sup.get_content().trim().to_string()));
          }
        }
        let requests = holder.get_attribute("_annotations").unwrap_or_default();
        shown.extend(
          requests
            .split(',')
            .filter_map(|label| label.strip_prefix("affiliation:"))
            .map(mark_form),
        );
      }
      // (the creator's own requests, which pick its mark among several)
      let requested: Vec<String> = creator
        .get_attribute("_annotations")
        .unwrap_or_default()
        .split(',')
        .map(|label| label_key(label.split_once(':').map_or(label, |(_, key)| key)))
        .collect();
      let marks: Vec<&String> = keys
        .iter()
        .filter(|(key, marked)| *marked || prints || shown.contains(&mark_form(key)))
        .map(|(key, _)| key)
        .collect();
      marks
        .iter()
        .find(|key| requested.contains(key))
        .or(marks.first())
        .map(|key| {
          refmarks
            .iter()
            .find(|(number, printed)| *number == mark_form(key) && !printed.is_empty())
            .map_or_else(|| (*key).clone(), |(_, printed)| printed.clone())
        })
    })
    .unwrap_or_default()
}

/// The Kind step for a dedication (FIDELITY_CHARTER §4: no printed word lost, each kept where it belongs; census138
/// LEGEND: 107c), where the legend's is ([`kind_legend_contacts`]): an affiliation-kind contact's lines from the first
/// that opens a dedication ([`opens_a_dedication`]: "This work is dedicated to Professor Roger Temam" set under the
/// author block, `\centerline{…}\\ \centerline{on the occasion of his seventieth birthday}`, 0910.1386) to the first
/// later line that reads as something else ([`ends_a_dedication`]: an address, a mark's line, a note's kind, an
/// affiliation; review rev107c S2) are the paper's dedication — not an affiliation of its holders and no note of theirs
/// — written once, as a class's `\dedication` writes one (`ltx:pubnote role=dedication`, aa_support_sty.rs; no label, as
/// the block prints none), after the last creator, its lines' breaks one each (review rev107c N3); every holder's copy
/// gives them up, and a contact left empty goes. A line led by a mark a name shows or requests — its own leading mark,
/// or the contact's ([`legend_mark`]) where it is the contact's first line — is no dedication: it is its carriers'
/// note, as "Deceased" is (`$^\dagger$In memory of Bob Baker, …` under Bob Baker$^{1,\dagger}$; [`kind_legend_contacts`],
/// review rev107c S1).
fn kind_dedication_lines(document: &mut Document) -> Result<()> {
  let contacts: Vec<Node> = document
    .findnodes("//ltx:creator/ltx:contact", None)
    .into_iter()
    .filter(|contact| {
      contact
        .get_attribute("role")
        .is_some_and(|role| CUT_ROLES.contains(&role.as_str()))
    })
    .collect();
  let line_text = |nodes: &[Node]| -> String { nodes.iter().map(Node::get_content).collect() };
  let opening: Vec<(Node, Vec<EntryLine>)> = contacts
    .into_iter()
    .map(|contact| {
      let lines = entry_lines(&contact);
      (contact, lines)
    })
    .filter(|(_, lines)| {
      lines
        .iter()
        .any(|(nodes, _)| opens_a_dedication(&line_text(nodes)))
    })
    .collect();
  if opening.is_empty() {
    return Ok(());
  }
  let named_creators = document.findnodes("//ltx:creator[ltx:personname]", None);
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  let author_marks = author_mark_keys(document, &authors);
  let glued_marks = glued_name_marks(document, &authors);
  let mut written: Vec<String> = Vec::new();
  for (contact, lines) in opening {
    let contact_mark = legend_mark(document, &contact, &named_creators);
    let first_printed = lines
      .iter()
      .position(|(nodes, _)| nodes.iter().any(|n| !n.get_content().trim().is_empty()));
    // (a line a name's mark points to: its leading mark one a name answers, or the contact's mark on its first line)
    let requested = |at: usize| -> bool {
      line_leading_mark(&lines[at].0)
        .is_some_and(|mark| !answering_or_glued(&author_marks, &glued_marks, &mark).is_empty())
        || (Some(at) == first_printed && !contact_mark.is_empty())
    };
    let Some(from) =
      (0..lines.len()).find(|&at| opens_a_dedication(&line_text(&lines[at].0)) && !requested(at))
    else {
      continue;
    };
    let to = (from + 1..lines.len())
      .find(|&at| ends_a_dedication(&lines[at].0))
      .unwrap_or(lines.len());
    // (every node it takes out, and the breaks between its lines)
    let mut dedication: Vec<Node> = Vec::new();
    for (at, (nodes, before)) in lines.iter().enumerate().take(to).skip(from) {
      if at > from {
        dedication.extend(before.clone());
      }
      dedication.extend(nodes.iter().cloned());
    }
    // (its lines for the note: one break between two, none at its edges — `\centerline{…}\\ \centerline{…}` printed
    // two, the `\\`'s and the centred line's, review rev107c N3)
    let blank = |node: &Node| {
      node.get_type() != Some(NodeType::ElementNode) && node.get_content().trim().is_empty()
    };
    let is_break =
      |node: &Node| node.get_type() == Some(NodeType::ElementNode) && node.get_name() == "break";
    let mut inner: Vec<Node> = Vec::new();
    let mut after_break = true;
    for node in &dedication {
      if is_break(node) {
        if after_break {
          continue;
        }
        after_break = true;
      } else if !blank(node) {
        after_break = false;
      }
      inner.push(node.clone());
    }
    let text = line_text(&inner)
      .split_whitespace()
      .collect::<Vec<_>>()
      .join(" ");
    DebugFeature!(
      "frontmatter",
      "FRONT kind affiliation: a dedication: {text}"
    );
    if !written.contains(&text) {
      written.push(text);
      let Some(last) = document.findnodes("//ltx:creator", None).last().cloned() else {
        continue;
      };
      let Some(parent) = last.get_parent() else {
        continue;
      };
      let saved = document.get_node().clone();
      document.set_node(&parent);
      let mut attributes: HashMap<String, String> = HashMap::default();
      attributes.insert("role".to_string(), "dedication".to_string());
      let mut note = document.open_element("ltx:pubnote", Some(attributes), None)?;
      document.close_element("ltx:pubnote")?;
      document.set_node(&saved);
      document.append_clone(&mut note, inner)?;
      note.unlink_node();
      let mut last = last;
      last.add_next_sibling(&mut note)?;
      trim_edge_breaks(document, &note);
    }
    let mut parents: Vec<Node> = Vec::new();
    if let Some((_, Some(before))) = lines.get(from) {
      parents.extend(before.get_parent());
      document.remove_node(before.clone());
    }
    for node in dedication {
      parents.extend(node.get_parent());
      document.remove_node(node);
    }
    drop_emptied_wrappers(document, parents);
    // (and the blank lines it leaves at the contact's edges: `\mbox{ }\\` before it, 0910.1386)
    while let Some(last) = contact.get_last_child()
      && (last.get_name() == "break"
        || (last.get_content().trim().is_empty() && element_nodes(&last).is_empty()))
    {
      document.remove_node(last);
    }
    trim_edge_breaks(document, &contact);
  }
  drop_empty_contacts(document)
}

/// The mark leading a line's nodes ([`kind_dedication_lines`]): its first printing node a superscript or a formula read
/// as a mark ([`is_mark_text`]), else the mark glyphs a text opening it prints ([`is_mark_glyph`]); `None` for none.
fn line_leading_mark(nodes: &[Node]) -> Option<String> {
  fn first_printing(node: &Node) -> Option<Node> {
    match node.get_type() {
      Some(NodeType::TextNode) => (!node.get_content().trim().is_empty()).then(|| node.clone()),
      Some(NodeType::ElementNode) if node.get_name() == "text" => {
        node.get_child_nodes().iter().find_map(first_printing)
      },
      Some(NodeType::ElementNode) => Some(node.clone()),
      _ => None,
    }
  }
  let first = nodes.iter().find_map(first_printing)?;
  let content = first.get_content();
  if first.get_type() == Some(NodeType::ElementNode) {
    return (matches!(first.get_name().as_str(), "sup" | "Math") && is_mark_text(&content))
      .then(|| content.trim().to_string());
  }
  let glyphs: String = content
    .trim_start()
    .chars()
    .take_while(|c| is_mark_glyph(*c))
    .collect();
  (!glyphs.is_empty()).then_some(glyphs)
}

/// Whether a line after a dedication's first reads as something else, so the dedication ends before it
/// ([`kind_dedication_lines`], review rev107c S2): it prints an address or a link (`@`, `http`, `www.`), a mark leads it
/// ([`line_leading_mark`]), its text is a note of another kind ([`classify_thanks`]: "Present address: …", "Supported by
/// NSF grant 123", "Corresponding author"), or an affiliation ([`reads_as_affiliation`]) — not a dedication's own next
/// line ("on the occasion of his seventieth birthday").
fn ends_a_dedication(nodes: &[Node]) -> bool {
  let text: String = nodes.iter().map(Node::get_content).collect();
  if text.trim().is_empty() {
    return false;
  }
  let lower = text.to_lowercase();
  lower.contains('@')
    || lower.contains("http")
    || lower.contains("www.")
    || line_leading_mark(nodes).is_some()
    || classify_thanks(&text) != "note"
    || reads_as_affiliation(&Tokens::new(ExplodeText!(&text)))
}

/// The Kind step for a link and a membership grade (FIDELITY_CHARTER §4; census132 LEGEND; OXIDIZED_DESIGN_DIVERGENCES
/// #969), where the legend's is ([`kind_legend_contacts`]): a creator's affiliation contact carrying the role's
/// default name whose whole printed text is
/// - one link ([`sole_link`]: `{\texttt{\url{https://github.com/…}}}` under IEEEtran's `\IEEEauthorblockA{…}\\`, which
///   the binding wraps into a block of its own, 2511.18438) is a URL contact (`role=url`, as an author block's link line
///   is one, `author_block::links::is_link_line`): the block's own creator's, once, when it is the block's line — no
///   mark labels it and every author holds a copy —, else each holder's (a line a mark labels is its owners'); a short
///   label before it that names a link (`Project website:`, `\textbf{Website: }`: [`link_label`]) is the contact's
///   name, as printed (2609.03591, 2502.19820: 107c); an entry whose every line is one such link is a contact per line
///   ([`split_link_lines`]), one to `orcid.org` an ORCID contact (IEEEtran's `\IEEEauthorblockA{\href{mailto:…}{…}\\
///   \url{https://orcid.org/…}\\ \url{https://members.loria.fr/AClement}}`, 2206.10577: 107c);
/// - a membership grade ([`reads_as_membership_grade`]: "Senior Member, IEEE" behind IEEE_lsens's
///   `\IEEEauthorieeemembermark{2}`, a raw class's mark, 2310.02678, 2505.15137) is its carriers' `membership` contact,
///   as IEEEtran's `\IEEEmembership` makes one (ieeetran_cls.rs `\lx@IEEE@membership`).
///
/// Its printed text stays, and so do its carriers. Guards `perfect_kernel_batch106b`.
fn kind_link_and_grade_contacts(document: &mut Document) -> Result<()> {
  let contacts = document.findnodes("//ltx:creator/ltx:contact[@role='affiliation']", None);
  if contacts.is_empty() {
    return Ok(());
  }
  let default_name = get_frontmatter_name(None, "ltx:contact", "affiliation")?;
  // (an entry of link lines, a contact per line)
  let mut read: Vec<Node> = Vec::with_capacity(contacts.len());
  for contact in contacts {
    let copies = if contact.get_attribute("name") == default_name {
      split_link_lines(document, &contact)?
    } else {
      Vec::new()
    };
    read.push(contact);
    read.extend(copies);
  }
  // (each link's copies, in print order)
  let mut links: Vec<(String, Vec<Node>)> = Vec::new();
  for mut contact in read {
    if contact.get_parent().is_none() || contact.get_attribute("name") != default_name {
      continue;
    }
    // (a label naming the link before it: the contact's name, as printed)
    if let Some((label, link)) = link_label(document, &contact) {
      strip_before_link(document, &contact, &link);
      document.set_attribute(&mut contact, LINK_LABEL, &s!("{label} "))?;
    }
    if let Some(link) = sole_link(document, &contact) {
      match links.iter_mut().find(|(href, _)| *href == link) {
        Some((_, copies)) => copies.push(contact),
        None => links.push((link, vec![contact])),
      }
      continue;
    }
    let mut text = String::new();
    legend_text(&contact, &mut text);
    if reads_as_membership_grade(&text) {
      DebugFeature!(
        "frontmatter",
        "FRONT kind affiliation: a membership grade: {text}"
      );
      document.set_attribute(&mut contact, "role", "membership")?;
      set_role_name(document, &mut contact, "membership")?;
    }
  }
  if links.is_empty() {
    return Ok(());
  }
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  for (link, copies) in links {
    DebugFeature!(
      "frontmatter",
      "FRONT kind affiliation: a link, {} copies: {link}",
      copies.len()
    );
    let mut holders: Vec<Node> = Vec::new();
    for holder in copies.iter().filter_map(Node::get_parent) {
      if !holders.contains(&holder) {
        holders.push(holder);
      }
    }
    // (the block's line: no mark labels it, and every author holds it — 2511.18438's line under all four names; a line a
    // mark labels is its owners', `\IEEEauthorblockA{\IEEEauthorrefmark{1}\url{…}}` under the names marked 1)
    let unlabelled = copies.iter().all(|copy| {
      copy
        .get_attribute("_label")
        .is_none_or(|label| label.is_empty())
    });
    let kept =
      if unlabelled && authors.len() > 1 && authors.iter().all(|author| holders.contains(author)) {
        let Some(mut block) = block_creator(document, &authors)? else {
          continue;
        };
        document.append_clone(&mut block, vec![copies[0].clone()])?;
        let Some(placed) = element_nodes(&block).last().cloned() else {
          continue;
        };
        for copy in copies {
          document.remove_node(copy);
        }
        vec![placed]
      } else {
        copies
      };
    let role = if link.contains("orcid.org/") {
      "orcid"
    } else {
      "url"
    };
    for mut copy in kept {
      document.set_attribute(&mut copy, "role", role)?;
      match copy.get_attribute(LINK_LABEL) {
        Some(label) => {
          document.set_attribute(&mut copy, "name", &label)?;
          copy.remove_attribute(LINK_LABEL).ok();
        },
        None => set_role_name(document, &mut copy, role)?,
      }
    }
  }
  Ok(())
}

/// The attribute of a link contact whose label [`kind_link_and_grade_contacts`] read off its text ([`link_label`]):
/// the label as printed, the contact's name once it is placed.
const LINK_LABEL: &str = "_linklabel";

/// The words a link's label ends in ([`link_label`]): "Project website:", "Homepage:", "Code:", "Project page:".
const LINK_LABEL_WORDS: &[&str] = &[
  "website",
  "websites",
  "site",
  "homepage",
  "page",
  "url",
  "web",
  "code",
  "github",
  "repository",
  "repo",
  "demo",
  "project",
  "data",
  "dataset",
];

/// The label printed before the one link a contact's text holds, on one line — one to three words, the last naming a
/// link, and a colon ([`is_link_label`]): `Project website: {\small\href{…}{https://…}}` (2609.03591), `\textbf{Website: }
/// \href{…}{https://…}` (2502.19820) — where the link prints as a URL does and nothing follows it but a stop: the label
/// as printed, and the link. `None` for any other text.
fn link_label(document: &mut Document, contact: &Node) -> Option<(String, Node)> {
  if !document.findnodes(".//ltx:break", Some(contact)).is_empty() {
    return None;
  }
  let refs = document.findnodes(".//ltx:ref[@href]", Some(contact));
  let [link] = refs.as_slice() else {
    return None;
  };
  let label = printed_link_label(&contact.get_content(), link)?;
  is_link_label(&label).then(|| (label, link.clone()))
}

/// The text printed before `link` in `text` (a line's), its spacing collapsed, where the link prints as a URL does
/// ([`prints_as_url`]) and nothing follows it but a stop; `None` otherwise.
fn printed_link_label(text: &str, link: &Node) -> Option<String> {
  let href = link.get_attribute("href")?;
  let shown = link.get_content();
  let shown = shown.trim();
  let at = text.find(shown)?;
  let after = text[at + shown.len()..].trim();
  (!shown.is_empty()
    && after.trim_end_matches(['.', ',']).is_empty()
    && prints_as_url(shown, &href))
  .then(|| text[..at].split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Whether `label` names a link ([`link_label`]): one to three words of letters, the last one of [`LINK_LABEL_WORDS`],
/// and a colon.
fn is_link_label(label: &str) -> bool {
  let Some(words) = label.strip_suffix(':') else {
    return false;
  };
  let words: Vec<&str> = words.split_whitespace().collect();
  (1..=3).contains(&words.len())
    && words
      .iter()
      .all(|word| word.chars().all(|c| c.is_alphabetic() || c == '-'))
    && words
      .last()
      .is_some_and(|word| LINK_LABEL_WORDS.contains(&word.to_lowercase().as_str()))
}

/// Removes from `parent` everything printed before `link` (the label [`link_label`] read): each node before it, and
/// before it inside each wrapper holding it.
fn strip_before_link(document: &mut Document, parent: &Node, link: &Node) {
  let mut holders: Vec<Node> = Vec::new();
  let mut at = link.get_parent();
  while let Some(node) = at {
    if node == *parent {
      break;
    }
    at = node.get_parent();
    holders.push(node);
  }
  for child in parent.get_child_nodes() {
    if child == *link {
      return;
    }
    if holders.contains(&child) {
      strip_before_link(document, &child, link);
      return;
    }
    document.remove_node(child);
  }
}

/// Whether a link printing `shown` with target `href` prints as a URL does: `https://…`, `www.…`, or its target without
/// the scheme — not a department's name set as a link (1812.03371's `\href{…}{Department of Mathematics, Harvey Mudd
/// College}`).
fn prints_as_url(shown: &str, href: &str) -> bool {
  let bare = |text: &str| {
    text
      .trim_start_matches("https://")
      .trim_start_matches("http://")
      .trim_start_matches("www.")
      .trim_end_matches('/')
      .to_lowercase()
  };
  shown.starts_with("http://")
    || shown.starts_with("https://")
    || shown.starts_with("www.")
    || bare(shown) == bare(href)
}

/// An affiliation contact's link lines (each of its children between two line breaks one link printing as a URL does,
/// [`prints_as_url`], behind a link's label or none, [`is_link_label`], and nothing else), parted off: those ending it
/// after a line of other text (`\IEEEauthorblockA{\textsuperscript{2}Univ B\\ Project website: {\href{…}{…}}}`: 107c),
/// or, where every line is one and there are two or more, each but the first (an author's own block of links,
/// `\url{https://orcid.org/…}\\ \url{https://members.loria.fr/AClement}`, 2206.10577: 107c) — a copy of the contact
/// per line, holding it, after the contact in print order. Returns the copies made.
fn split_link_lines(document: &mut Document, contact: &Node) -> Result<Vec<Node>> {
  let mut lines: Vec<Vec<Node>> = vec![Vec::new()];
  for child in contact.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) && child.get_name() == "break" {
      lines.push(Vec::new());
    } else if (child.get_type() != Some(NodeType::TextNode)
      || !child.get_content().trim().is_empty())
      && let Some(line) = lines.last_mut()
    {
      line.push(child);
    }
  }
  lines.retain(|line| !line.is_empty());
  let link_line = |document: &mut Document, line: &[Node]| -> bool {
    let mut refs: Vec<Node> = Vec::new();
    for node in line {
      if node.get_type() == Some(NodeType::ElementNode) && node.get_name() == "ref" {
        refs.push(node.clone());
      } else if node.get_type() == Some(NodeType::ElementNode) {
        refs.extend(document.findnodes(".//ltx:ref[@href]", Some(node)));
      }
    }
    let [link] = refs.as_slice() else {
      return false;
    };
    let text: String = line.iter().map(Node::get_content).collect();
    printed_link_label(&text, link).is_some_and(|label| label.is_empty() || is_link_label(&label))
  };
  let links: Vec<bool> = lines.iter().map(|line| link_line(document, line)).collect();
  let first = links.iter().rposition(|link| !link).map_or(1, |at| at + 1);
  if lines.len() < 2 || first >= lines.len() {
    return Ok(Vec::new());
  }
  let Some(mut creator) = contact.get_parent() else {
    return Ok(Vec::new());
  };
  let mut copies: Vec<Node> = Vec::new();
  let mut previous = contact.clone();
  for keep in first..lines.len() {
    document.append_clone(&mut creator, vec![contact.clone()])?;
    let Some(mut copy) = element_nodes(&creator).last().cloned() else {
      continue;
    };
    copy.unlink_node();
    previous.add_next_sibling(&mut copy)?;
    keep_only_lines(document, &copy, keep..keep + 1);
    previous = copy.clone();
    copies.push(copy);
  }
  keep_only_lines(document, contact, 0..first);
  Ok(copies)
}

/// Removes from `contact` every printed line outside `keep` ([`split_link_lines`]), and the line breaks of the lines it
/// removes and at its edges.
fn keep_only_lines(document: &mut Document, contact: &Node, keep: std::ops::Range<usize>) {
  let mut line = 0;
  let mut printed = false;
  for child in contact.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) && child.get_name() == "break" {
      if printed {
        line += 1;
        printed = false;
      }
      // (a break inside the lines kept stays: one before their first line or after their last goes)
      if !(keep.start < line && line < keep.end) {
        document.remove_node(child);
      }
      continue;
    }
    let prints =
      child.get_type() != Some(NodeType::TextNode) || !child.get_content().trim().is_empty();
    printed |= prints;
    if !keep.contains(&line) && prints {
      document.remove_node(child);
    }
  }
  trim_edge_breaks(document, contact);
}

/// `contact` named as its class names a contact of `role` (`get_frontmatter_name`; no name when the class gives none).
fn set_role_name(document: &mut Document, contact: &mut Node, role: &str) -> Result<()> {
  match get_frontmatter_name(None, "ltx:contact", role)? {
    Some(name) => document.set_attribute(contact, "name", &name)?,
    None => {
      contact.remove_attribute("name").ok();
    },
  }
  Ok(())
}

/// The target of the one link a contact's whole printed text is (an `ltx:ref` with an `href`, in whatever text, font
/// or line break wrappers, printing all the contact prints but its spacing and a closing stop) and that prints as a URL
/// does (its target, or `http(s)://…`, `www.…`); `None` for any other.
fn sole_link(document: &mut Document, contact: &Node) -> Option<String> {
  let refs = document.findnodes(".//ltx:ref[@href]", Some(contact));
  let [link] = refs.as_slice() else {
    return None;
  };
  let printed = |text: String| {
    text
      .chars()
      .filter(|c| !c.is_whitespace())
      .collect::<String>()
      .trim_end_matches(['.', ','])
      .to_string()
  };
  let shown = printed(link.get_content());
  let href = link.get_attribute("href")?;
  // (a link printing its own target, as a URL prints: [`prints_as_url`])
  (!shown.is_empty() && prints_as_url(&shown, &href) && shown == printed(contact.get_content()))
    .then_some(href)
}

/// Whether a contact's text ([`legend_text`]) is a membership grade in a society, its marks aside: "Senior Member,
/// IEEE", "Graduate Student Member, IEEE", "Life Fellow, IEEE", "Member, ACM" — grade words, then "Member" or "Fellow",
/// a comma and a society whose grades follow a name (`NAME_GRADE_SOCIETIES`, the names reader's: IEEE, ACM, OSA, ASCE,
/// …) — one per line ("Graduate Student Member, IEEE | Senior Member, IEEE"). Not "Member of Technical Staff, Bell
/// Labs", "Fellow, Trinity College", nor a position at an institute ("Senior Fellow, IAS": review rev106b S3).
fn reads_as_membership_grade(text: &str) -> bool {
  const GRADES: &[&str] = &[
    "life",
    "senior",
    "graduate",
    "student",
    "associate",
    "honorary",
    "distinguished",
    "affiliate",
  ];
  let grade = |line: &str| {
    let Some((grade, society)) = line.split_once(',') else {
      return false;
    };
    let words: Vec<&str> = grade
      .split(|c: char| !c.is_alphanumeric())
      .filter(|w| !w.is_empty())
      .collect();
    let society: Vec<&str> = society
      .split(|c: char| !c.is_alphanumeric())
      .filter(|w| !w.is_empty())
      .collect();
    matches!(words.split_last(), Some((last, before))
      if matches!(last.to_lowercase().as_str(), "member" | "fellow")
        && before.iter().all(|w| GRADES.contains(&w.to_lowercase().as_str())))
      && matches!(society.as_slice(), [society] if NAME_GRADE_SOCIETIES.contains(society))
  };
  let lines: Vec<&str> = text
    .split('|')
    .map(str::trim)
    .filter(|line| line.chars().any(char::is_alphanumeric))
    .collect();
  !lines.is_empty() && lines.iter().all(|line| grade(line))
}

/// The attribute naming the source call a contact was made by — an annotation's, its queue ordinal when it was digested
/// (`\lx@annotate@frontmatter@now`), else the entry's own when it is inserted (`insert_frontmatter_entry`) — which
/// every copy of it keeps (the entry given to several creators, a relocated annotation's `append_clone`, the writer's
/// copies, a creator coalesced into another) and the e-mails the Kind step makes of its address lines take
/// ([`kind_address_lines`]): [`drop_repeated_contacts`].
pub(in crate::base_utilities) const CONTACT_SOURCE: &str = "_source";

/// The attribute of a contact a call made after `\maketitle` placed the frontmatter (`queue_digest::
/// frontmatter_entry_is_late`): a supplement's `\author{…}\affiliation{…}` written again, which no title prints
/// (2311.16348, 1407.0849): [`drop_repeated_contacts`].
pub(in crate::base_utilities) const CONTACT_LATE: &str = "_latecall";

/// The state counter of the [`CONTACT_SOURCE`] of a contact no annotation made.
const CONTACT_SOURCE_COUNTER: &str = "lx_frontmatter_contact_source";

/// Rust-only (OXIDIZED_DESIGN_DIVERGENCES #969): a creator holding one entry twice — two contacts of one role and one
/// name printing the same text (its spacing aside; case, marks and notes kept) — keeps the first, in print order, where
/// the two are one entry:
/// - copies of one source call ([`CONTACT_SOURCE`]), which two routes placed on it: PoS's "on behalf of the Fermi-LAT
///   Collaboration", given to every name, twice on the last when the empty name after it is coalesced into it
///   (1909.05984); a revtex group's affiliations doubled (1207.1783, 2006.15775);
/// - lines of one label (`_label`): a marked line written again under a later name, easychair's `$^{1}$Tsinghua
///   University` under each of two names, which the first name's mark 1 requests twice (1204.6725);
/// - a call made after `\maketitle` placed the frontmatter ([`CONTACT_LATE`]) restating one the creator holds: a
///   supplement's `\author{…}\affiliation{…}` written again, which no title prints (2311.16348, 1407.0849).
///
/// Two printed lines that share their text are two entries, both kept: an address's "Groningen" and the next one's
/// (0808.4092, 1106.1785), amsart's `\email{Ann.Able@x.org}` and `\email{ann.able@x.org}`, a second address carrying a
/// footnote of its own, aa's `\thanks{\email{x}}` beside the institute's `\email{x}` (2005.03567, each printed), aa's
/// `\mail{x}` "Correspondence to" beside the `\email{x}` (two names, perfect_kernel_batch97b). A copy on another creator
/// stays: a shared entry is each holder's. Perl writes each call's contact (Base_Utility.pool.ltxml :659-660). Guards
/// `perfect_kernel_batch106b`.
fn drop_repeated_contacts(document: &mut Document) {
  for creator in document.findnodes("//ltx:creator", None) {
    // (each entry held: its identity, role, name and printed text)
    let mut seen: Vec<(String, Option<String>, Option<String>, String)> = Vec::new();
    for contact in element_nodes(&creator) {
      if contact.get_name() != "contact" {
        continue;
      }
      let text = contact
        .get_content()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
      if text.is_empty() {
        continue;
      }
      let identities: Vec<String> = [
        contact
          .get_attribute(CONTACT_SOURCE)
          .map(|source| s!("source:{source}")),
        contact
          .get_attribute("_label")
          .filter(|label| !label.is_empty())
          .map(|label| s!("label:{label}")),
      ]
      .into_iter()
      .flatten()
      .collect();
      let (role, name) = (contact.get_attribute("role"), contact.get_attribute("name"));
      let late = contact.has_attribute(CONTACT_LATE);
      let held = |identity: Option<&String>| {
        seen.iter().any(|(i, r, n, t)| {
          identity.is_none_or(|identity| i == identity) && *r == role && *n == name && *t == text
        })
      };
      if (late && held(None)) || identities.iter().any(|identity| held(Some(identity))) {
        DebugFeature!(
          "frontmatter",
          "FRONT a contact held twice ({identities:?}): {text}"
        );
        document.remove_node(contact);
      } else if identities.is_empty() {
        seen.push((String::new(), role, name, text));
      } else {
        for identity in identities {
          seen.push((identity, role.clone(), name.clone(), text.clone()));
        }
      }
    }
  }
}

/// The printed text of a contact for the Kind step for a legend ([`kind_legend_contacts`]): its text, a line break
/// (`ltx:break`, at any depth: `{\small corresponding author \\ SSE, CUHKSZ}`, 2303.06042) written `|`, a superscript
/// set apart by spaces, a note inside it (`\footnotemark`'s tags, 2504.19724) left out.
fn legend_text(node: &Node, text: &mut String) {
  for child in node.get_child_nodes() {
    if child.get_type() != Some(NodeType::ElementNode) {
      text.push_str(&child.get_content());
      continue;
    }
    match child.get_name().as_str() {
      "break" => text.push_str(" | "),
      "note" => {},
      "sup" => {
        text.push(' ');
        legend_text(&child, text);
        text.push(' ');
      },
      _ => legend_text(&child, text),
    }
  }
}

/// A label's key as a mark prints it, for the Kind step for a legend ([`kind_legend_contacts`]): its math shifts and
/// the spaces a label's words are joined by (`_`) left out (`$†$`: †).
fn label_key(label: &str) -> String { label.chars().filter(|c| *c != '$' && *c != '_').collect() }

/// A mark as [`kind_legend_contacts`] compares it with the marks a name shows: its label key ([`label_key`]), trimmed,
/// the math asterisk read as the text one (`$^{1,*}$` prints `∗`, the label is `*`: 2503.10710).
fn mark_form(mark: &str) -> String { label_key(mark.trim()).replace('∗', "*") }

/// A contact of no content — a template's `\curraddr{}`, `\urladdr{}`, `\dedicatory{}`, aastex's `\nocollaboration` —
/// is no contact: its name label printed over nothing ("Current address:"), where pdflatex prints nothing (58 of the
/// 2,973 ams census papers: 2206.14950, 2203.07822, 2001.01741, 2003.03018; 7 of the 688 aas). Removed once every entry
/// is placed — an empty entry still counts in today's pile (`perfect_kernel_batchfms2a2r4::an_empty_entry_is_in_todays_pile`)
/// — with a nameless creator it leaves empty. Guard `perfect_kernel_batch81::empty_contacts_print_nothing`.
fn drop_empty_contacts(document: &mut Document) -> Result<()> {
  // (empty: no text, and no element but an empty `text` or a `break` — an icon, a link, a formula is content)
  let empty = |contact: &Node| {
    contact.get_content().trim().is_empty()
      && element_nodes(contact).iter().all(|child| {
        matches!(child.get_name().as_str(), "text" | "break") && element_nodes(child).is_empty()
      })
  };
  for creator in document.findnodes("//ltx:creator", None) {
    let dropped: Vec<Node> = element_nodes(&creator)
      .into_iter()
      .filter(|child| child.get_name() == "contact" && empty(child))
      .collect();
    if dropped.is_empty() {
      continue;
    }
    for contact in dropped {
      document.remove_node(contact);
    }
    // (a nameless creator holding only them is no creator either; the scope is the creators this pass emptied)
    if element_nodes(&creator).is_empty() {
      document.remove_node(creator);
    }
  }
  Ok(())
}

/// Rust-only (OXIDIZED_DESIGN_DIVERGENCES #960): a creator holding one ORCID iD twice keeps its first contact of it —
/// acmart's `\orcid{0000-0001-9164-7747}` beside the name's `\orcidicon{0000-0001-9164-7747}` link, which
/// `\lx@author@href` made the same contact (2501.02380; review rev103b S2). The iD is the contact's link, else its text;
/// two iDs are two contacts. Perl keeps a repeated `\orcid` twice (Base_Utility.pool.ltxml:659-660).
fn drop_repeated_orcids(document: &mut Document) {
  for creator in document.findnodes("//ltx:creator", None) {
    let mut seen: Vec<String> = Vec::new();
    for contact in element_nodes(&creator) {
      if contact.get_name() != "contact"
        || contact.get_attribute("role").as_deref() != Some("orcid")
      {
        continue;
      }
      let id = document
        .findnodes(".//ltx:ref[@href]", Some(&contact))
        .first()
        .and_then(|link| link.get_attribute("href"))
        .unwrap_or_else(|| contact.get_content())
        .trim()
        .to_string();
      if id.is_empty() {
        continue;
      }
      if seen.contains(&id) {
        document.remove_node(contact);
      } else {
        seen.push(id);
      }
    }
  }
}

/// The Link step for the addresses printed inside an affiliation-kind entry (OXIDIZED_DESIGN_DIVERGENCES #956): every
/// e-mail digested inside one (an `\email` in an `\institute` entry, an address line the Kind step made one,
/// `affiliations::kind_entry_addresses`), wherever its label or call placed it, goes where FIDELITY_CHARTER §4's chain
/// over that entry puts it ([`entry_address_owners`], the frontmatter observer's chain too): the author it spells, else
/// the entry's only carrier, else every carrier of a symbol-led entry, else one per carrier in order, else the block's
/// own creator, once (ruling E, main session 2026-11-02). The entry's carriers are the authors holding it after
/// relocation (an entry of addresses alone stays as their carrier, empty, until `drop_empty_contacts`), else those
/// holding its e-mails. Each placed e-mail follows the entry on its owner (user ruling R8, print order). An entry whose
/// e-mails already stand as the chain puts them is left as it is. Witnesses: 2502.01754, 2602.21641, 2307.02047,
/// 1901.09107, 0902.3918, 1909.00276, astro-ph0512555; guards `perfect_kernel_batch100b`.
fn link_entry_addresses(document: &mut Document, only: Option<&HashSet<String>>) -> Result<()> {
  // (`only`: the entries whose e-mails the Kind step for an address line made after the first pass, OD #966)
  let emails: Vec<Node> = document
    .findnodes(
      &s!("//ltx:creator/ltx:contact[@role='email'][@{ENTRY_KEY}]"),
      None,
    )
    .into_iter()
    .filter(|email| {
      only.is_none_or(|keys| {
        email
          .get_attribute(ENTRY_KEY)
          .is_some_and(|key| keys.contains(&key))
      })
    })
    .collect();
  if emails.is_empty() {
    return Ok(());
  }
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  // (one author, and no author creator without a name beside it that holds names the reader did not part, which are
  // more authors — 1102.4967's "Rizwan Ghaffar, Dimitris Toumpakaris and" beside Jungwon Lee, whose addresses are not
  // his; review of 107c r2's A/B. The block's own creator (OD #159) or an orphan line's is no author: a line's owner
  // never depends on another line — user ruling 2026-11-05, "Sole author": `* Corresponding author, E-mail: qz@a.org`
  // under `Ann Able$^{a}$` is Ann's beside an orphan `$^{b}$ Univ B, City` too, rev107c2 finding 2)
  let sole_author = authors.len() == 1 && !nameless_creator_holds_names(document);
  let default_name = get_frontmatter_name(None, "ltx:contact", "email")?;
  let names: Vec<String> = authors
    .iter()
    .map(|author| {
      document
        .findnode("ltx:personname", Some(author))
        .map(|person| person.get_content())
        .unwrap_or_default()
    })
    .collect();
  // (each name read as an address spells it, as the frontmatter observer's chain reads it: by its base letters,
  // `joze.rozanec@` spells Jože M. Rožanec, 2105.01929 — `resolver::base_letters` —, and its transliteration,
  // `bo.baecker@` Bo Bäcker, html_feedback #7233 — `NameKeys::of`)
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(name))))
    .collect();
  let position = |creator: &Node| authors.iter().position(|a| a == creator);
  // (the marks each author's name shows or requests, and those glued to it as text, read where an address line's mark
  // asks)
  let mut author_marks: Option<(MarkKeys, MarkKeys)> = None;
  // (each entry's holders, with their copies of it; the entries of one printed text are one: the same affiliation set
  // apart for each author, aastex's `\affil` after each name, its address no one carrier's by being printed under each —
  // as the frontmatter observer's reading groups its prints, 2212.12876)
  // (an entry's printed text, its keys, its holders with their copies)
  type EntryGroup = (String, Vec<String>, Vec<(usize, Node)>);
  let mut groups: Vec<EntryGroup> = Vec::new();
  for email in &emails {
    let key = email.get_attribute(ENTRY_KEY).unwrap_or_default();
    if groups
      .iter()
      .any(|(_, entry_keys, _)| entry_keys.contains(&key))
    {
      continue;
    }
    let held: Vec<(usize, Node)> = document
      .findnodes(
        &s!("//ltx:creator/ltx:contact[@{ENTRY_KEY}='{key}'][@role!='email']"),
        None,
      )
      .into_iter()
      .filter_map(|entry| {
        let at = position(&entry.get_parent()?)?;
        Some((at, entry))
      })
      .collect();
    // (an entry of addresses alone is its addresses: two such calls printing the same list, REVTeX's
    // `\altaffiliation[Authors for Correspondence: ]{Email: nikhil@…, bala@…}` after each of two names, are one —
    // 2208.10784)
    let text = held
      .first()
      .map(|(_, entry)| {
        entry
          .get_content()
          .split_whitespace()
          .collect::<Vec<_>>()
          .join(" ")
      })
      .filter(|text| !text.is_empty())
      .unwrap_or_else(|| {
        let mut own: Vec<&Node> = emails
          .iter()
          .filter(|e| e.get_attribute(ENTRY_KEY).as_deref() == Some(key.as_str()))
          .collect();
        own.sort_by_key(|e| {
          e.get_attribute(QUEUE_ORDINAL)
            .and_then(|n| n.parse::<i64>().ok())
            .unwrap_or(i64::MAX)
        });
        let addresses: Vec<String> = own
          .iter()
          .map(|e| e.get_content().trim().to_string())
          .collect();
        s!("\u{1}{}", addresses.join("\u{2}"))
      });
    match groups.iter_mut().find(|(t, ..)| *t == text) {
      Some((_, entry_keys, all)) => {
        entry_keys.push(key);
        all.extend(held);
      },
      None => groups.push((text, vec![key], held)),
    }
  }
  // (the addresses each entry leaves to the block, given once after every entry is placed, and only where no author
  // holds them: an address is never duplicated — 2208.10784's `bala@` on its author by one copy of a shared entry)
  let mut to_block: Vec<(String, Node)> = Vec::new();
  let mut taken_away: Vec<Node> = Vec::new();
  for (entry_text, entry_keys, held) in groups {
    // (in print order: the order their calls were digested in, `QUEUE_ORDINAL`)
    let mut mine: Vec<Node> = emails
      .iter()
      .filter(|e| {
        e.get_attribute(ENTRY_KEY)
          .is_some_and(|k| entry_keys.contains(&k))
      })
      .cloned()
      .collect();
    mine.sort_by_key(|e| {
      e.get_attribute(QUEUE_ORDINAL)
        .and_then(|n| n.parse::<i64>().ok())
        .unwrap_or(i64::MAX)
    });
    let key = entry_keys.join(",");
    let mut carriers: Vec<usize> = held.iter().map(|(at, _)| *at).collect();
    if carriers.is_empty() {
      carriers = mine
        .iter()
        .filter_map(|e| position(&e.get_parent()?))
        .collect();
    }
    carriers.sort_unstable();
    carriers.dedup();
    // (its addresses, one each, in print order; each with the authors holding it now — an e-mail printing several,
    // `\email{a|b@x.org}` inside a line of the entry, written out, each its own, as the frontmatter observer reads it:
    // `resolver::addresses_in`)
    // (an e-mail printing addresses alone, several written out; one printing a template, a list the reader could not
    // part or a group beside words — `{ann,bob}@a.org (corresponding)`, 92c — is no one's address: it stays where its
    // call placed it, as the frontmatter observer's reading leaves it)
    // (written out only where the two readings of it agree — the addresses it prints, `addresses_in`, and the runs its
    // take-out cuts, `address_take_out` —, so no local part one of them reads is lost: the take-out had cut
    // `dzyang,ashishbshah@fb.com` whole while the list held `ashishbshah@` alone, 2112.00969; else it stays whole)
    let readable = |content: &str| {
      let listed = resolver::addresses_in(content);
      let (taken, rest) = resolver::address_take_out(content, &[]);
      // (an e-mail label beside a group keeps it whole, as the frontmatter observer's group write-out does:
      // `Emails: {ann,bob}@a.org`, 92c)
      let words: Vec<&str> = content
        .split(|c: char| c.is_whitespace() || ",;:".contains(c))
        .filter(|w| !w.is_empty())
        .collect();
      // (a label of one word or two, "Electronic Addresses", 1509.07506)
      let labelled = words.iter().any(|w| is_email_label(w))
        || words.windows(2).any(|pair| is_email_label(&pair.join(" ")));
      // (a list written without spaces whose every part spells one author, each its own, is written out, its capitals
      // as printed: `M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk`, astro-ph0612088 — whole, it was copied onto
      // all three; `spelled_unspaced_list`)
      if let Some(spelled) =
        spelled_unspaced_list(content, &|part| spelled_owner(&keys, &s!("{part}@x.org")))
      {
        return Some(spelled);
      }
      if listed.len() > 1 && !labelled {
        (!rest.chars().any(char::is_alphanumeric) && taken == listed).then_some(listed)
      } else if listed.len() > 1 {
        None
      } else {
        let text = content.trim();
        (!lists_local_parts(text) && !is_pattern_address(text)).then(|| vec![text.to_string()])
      }
    };
    // (a contact printing several addresses and more — an ORCID's link, math, a note — is never written out: it stays
    // whole where its call placed it, [`prints_text_only`]; 2308.02269, review rev100b B1)
    mine.retain(|email| {
      readable(&email.get_content())
        .is_some_and(|listed| listed.len() < 2 || prints_text_only(email))
    });
    if mine.is_empty() {
      continue;
    }
    let mut addresses: Vec<(String, Node, Vec<Option<usize>>)> = Vec::new();
    let mut written: Vec<(Node, String)> = Vec::new();
    for email in &mine {
      let content = email.get_content();
      let listed = readable(&content).unwrap_or_default();
      let texts: Vec<String> = if listed.len() > 1 {
        written.extend(
          listed
            .iter()
            .map(|address| (email.clone(), address.clone())),
        );
        listed
      } else {
        vec![content.trim().to_string()]
      };
      let holder = email.get_parent().and_then(|p| position(&p));
      for text in texts {
        match addresses.iter_mut().find(|(t, ..)| *t == text) {
          Some((.., holders)) => holders.push(holder),
          None => addresses.push((text, email.clone(), vec![holder])),
        }
      }
    }
    let symbol = mine.iter().any(|e| e.get_attribute(ENTRY_SYMBOL).is_some());
    let spelled: Vec<Option<usize>> = addresses
      .iter()
      .map(|(text, ..)| {
        email_address_word(text)
          .filter(|address| !is_entry_word_mailbox(address, &entry_text))
          .and_then(|address| spelled_owner(&keys, &address))
      })
      .collect();
    // (an e-mail command the author wrote inside the entry, holding one address, keeps the one author its own call
    // placed it on, unless it spells another: FIDELITY_CHARTER §4 #420, an author's own call holding one address keeps
    // it — jmlr's `\Name{…} \Email{costis@…}` under a misread `\AND`, 2002.00057, 1504.01446, 2509.23923; not an
    // address the Kind step took out of the entry's text, `ENTRY_KINDED`)
    let own_call: Vec<Option<usize>> = addresses
      .iter()
      .zip(&spelled)
      .map(|((text, _, holders), spelled)| {
        let written_by_author = mine.iter().any(|e| {
          e.get_attribute(ENTRY_KINDED).is_none() && e.get_content().trim() == text.as_str()
        });
        match holders.as_slice() {
          [Some(holder)]
            if written_by_author
              && !written.iter().any(|(_, a)| a == text)
              && spelled.is_none_or(|s| s == *holder) =>
          {
            Some(*holder)
          },
          _ => None,
        }
      })
      .collect();
    // (the carriers holding an address of their own, outside this entry)
    let addressed: Vec<usize> = carriers
      .iter()
      .copied()
      .filter(|&c| {
        element_nodes(&authors[c]).iter().any(|n| {
          n.get_name() == "contact"
            && n.get_attribute("role").as_deref() == Some("email")
            && !mine.contains(n)
        })
      })
      .collect();
    // (an unspelled address its line's mark leads, the Kind step's for an address line, `LINE_MARK`: the authors whose
    // names answer the mark, else the block — marks win, user ruling R1 2026-11-01; never the only-carrier or order
    // step: gr-qc0207058's `$**$ email: indrani@…` is I. Chowdhury's, whose name carries `∗∗`, review rev106a B1)
    // (`mark_lists`: the lists no mark places, read as one line's list: `list_placements`)
    let mut mark_lists: Vec<Vec<usize>> = Vec::new();
    let marked: Vec<Option<Option<Vec<usize>>>> = addresses
      .iter()
      .zip(&spelled)
      .map(|((_, template, _), spelled_here)| {
        let mark = template
          .get_attribute(LINE_MARK)
          .filter(|_| spelled_here.is_none())?;
        let (marks, glued) = author_marks.get_or_insert_with(|| {
          (
            author_mark_keys(document, &authors),
            glued_name_marks(document, &authors),
          )
        });
        // (a mark glued to a name as text answers the line opening with it: `Jian Shi$^{a}$*` and `* Corresponding
        // author, E-mail: …`, 1711.00192 — 107c r2)
        let answering = answering_or_glued(marks, glued, &mark);
        // (no name shows or requests a mark: the line's is no key the names give; one the entry's several carriers
        // all answer, and no one else, is the entry's own — `$^{1}$Instituto …, Spain. E-mail: eca@…` under two names
        // carrying `1`, astro-ph0005055 —, which says no more than the entry does: the chain places it)
        if answering.is_empty() && marks.iter().all(Vec::is_empty) {
          return None;
        }
        if answering.len() > 1 && answering == carriers {
          return None;
        }
        // (a list of addresses is one per person, not every carrier's, where a sibling spells an author who does not
        // answer the mark, or the entry names one: spelling, then elimination, else the block. 2102.08716's `*equal
        // contributions. David Macêdo, Pedro Dreyer, Teresa Ludermir, and Cleber Zanchettin are with …
        // {dlm,phdl,cz,tbl}@…` under two `*` names; 2412.13509's `hmfu@`, whose list spells the others. A lone address
        // on a line a shared symbol labels stays every carrier's: user ruling 2026-10-30)
        let members: Vec<usize> = (0..addresses.len())
          .filter(|&at| {
            addresses[at].1.get_attribute(LINE_MARK).as_deref() == Some(mark.as_str())
              && addresses[at].1.get_attribute(LINE_LIST).is_none()
          })
          .collect();
        if answering.len() > 1 && members.len() > 1 {
          let beyond_the_mark = |author: usize| !answering.contains(&author);
          if members
            .iter()
            .any(|&at| spelled[at].is_some_and(beyond_the_mark))
            || (0..names.len()).any(|author| {
              beyond_the_mark(author) && text_prints_name(&entry_text, &names[author])
            })
          {
            if !mark_lists.contains(&members) {
              mark_lists.push(members);
            }
            return None;
          }
          // (the carriers no sibling spells, as the only-carrier step excludes them: `entry_address_owners`' `free`,
          // 1907.01388's `zhutou@`)
          let free: Vec<usize> = answering
            .iter()
            .copied()
            .filter(|author| !spelled.contains(&Some(*author)))
            .collect();
          return Some((!free.is_empty()).then_some(free));
        }
        Some((!answering.is_empty()).then_some(answering))
      })
      .collect();
    // (each address's line labelled by a mark in the sole author's own mark series, which no name answers: its
    // `LINE_MARK`, or the mark its block creator holds it by — `mark_series`)
    let own_series_orphan: Vec<bool> = addresses
      .iter()
      .map(|(_, template, _)| {
        if !sole_author {
          return false;
        }
        let mut line_marks: Vec<String> = template.get_attribute(LINE_MARK).into_iter().collect();
        if let Some(parent) = template.get_parent()
          && position(&parent).is_none()
        {
          line_marks.extend(
            parent
              .get_attribute("_bymark")
              .unwrap_or_default()
              .split(',')
              .filter(|mark| !mark.is_empty())
              .map(str::to_string),
          );
        }
        let (marks, glued) = author_marks.get_or_insert_with(|| {
          (
            author_mark_keys(document, &authors),
            glued_name_marks(document, &authors),
          )
        });
        line_marks.iter().any(|mark| {
          answering_or_glued(marks, glued, mark).is_empty()
            && marks
              .iter()
              .flatten()
              .any(|own| mark_series(own).is_some_and(|series| mark_series(mark) == Some(series)))
        })
      })
      .collect();
    // (a `{…}@domain` list set over several lines, the Kind step's: placed as its one-line form, [`list_placements`])
    let listed = list_placements(
      document,
      &authors,
      &keys,
      &addresses,
      &spelled,
      &mine,
      &mark_lists,
    );
    let owners: Vec<(Option<Vec<usize>>, &'static str)> = entry_address_owners(
      &spelled,
      0..addresses.len(),
      &carriers,
      symbol,
      &addressed,
      authors.len() == 1,
    )
    .into_iter()
    .zip(&own_call)
    .zip(marked)
    .zip(listed)
    .map(|(((owner, own), marked), listed)| match (own, marked) {
      (Some(holder), _) => (Some(vec![*holder]), "owner:call"),
      (None, Some(Some(answering))) => (Some(answering), "owner:mark"),
      (None, Some(None)) => (None, "block:unanswered-mark"),
      (None, None) => listed.unwrap_or(owner),
    })
    // (a paper of one author makes no block creator: the charter's "the group's only author" comes before the block,
    // a line's unanswered mark too — 1711.00192's `{\small * Corresponding author, E-mail: …}` under Jian Shi alone,
    // review rev107c B1 —, unless the mark is in the author's own mark series: `$^{b}$ E-mail: lab@b.org` beside
    // `Ann Able$^{a}$` is the block creator's, as its `$^{b}$ Univ B, City` sibling is (FIDELITY_CHARTER "Sole author",
    // user 2026-11-05; review rev107f finding 4))
    .zip(&own_series_orphan)
    .map(|((to, rule), own_series)| match to {
      None if sole_author && !own_series => (Some(vec![0]), "owner:only-author"),
      to => (to, rule),
    })
    .collect();
    // (already as the chain puts them: left as they are)
    let settled = written.is_empty()
      && addresses
        .iter()
        .zip(&owners)
        .all(|((.., holders), (to, _))| {
          let mut now: Vec<Option<usize>> = holders.clone();
          now.sort_unstable();
          let want: Vec<Option<usize>> = match to {
            Some(to) => {
              let mut to: Vec<Option<usize>> = to.iter().map(|&a| Some(a)).collect();
              to.sort_unstable();
              to
            },
            None => vec![None],
          };
          now == want
        });
    if settled {
      // (each after the entry on its holder, in print order: an inner e-mail an `annotate=` placed is attached before
      // the entry it was digested in)
      // (one whose line printed before every line the entry keeps, before it: `LINE_FIRST`, R8)
      for (at, entry) in &held {
        let mut anchor = entry.clone();
        let mut before: Option<Node> = None;
        for email in &mine {
          if email.get_parent().as_ref() == Some(&authors[*at]) {
            let mut email = email.clone();
            if email.get_attribute(LINE_FIRST).is_some() {
              let placed = match before.clone() {
                Some(mut previous) => previous.add_next_sibling(&mut email).is_ok(),
                None => entry.clone().add_prev_sibling(&mut email).is_ok(),
              };
              if placed {
                before = Some(email);
              }
            } else if anchor.add_next_sibling(&mut email).is_ok() {
              anchor = email;
            }
          }
        }
      }
      continue;
    }
    DebugFeature!(
      "frontmatter",
      "FRONT link entry {key}: carriers {carriers:?}, owners {:?}",
      owners
    );
    let emptied: Vec<Node> = mine.iter().filter_map(Node::get_parent).collect();
    // (the copies made while the entry's e-mails still stand, the templates; those taken away after)
    let mut after: HashMap<usize, Node> = HashMap::default();
    let mut before: HashMap<usize, Node> = HashMap::default();
    for ((text, template, _), (to, _)) in addresses.iter().zip(owners) {
      match to {
        Some(to) => {
          for owner in to {
            let mut author = authors[owner].clone();
            if let Some(held) = element_nodes(&author).into_iter().find(|c| {
              !mine.contains(c)
                && c.get_name() == "contact"
                && c.get_attribute("role").as_deref() == Some("email")
                && c.get_content().trim() == text.as_str()
            }) {
              if only.is_some() {
                carry_printed_label(template, held, default_name.as_deref());
              }
              continue;
            }
            document.append_clone(&mut author, vec![template.clone()])?;
            let Some(mut placed) = element_nodes(&author).last().cloned() else {
              continue;
            };
            if written
              .iter()
              .any(|(node, address)| node == template && *address == *text)
            {
              set_written_out(&placed, text);
            }
            let entry_on_owner = held
              .iter()
              .filter(|(at, _)| *at == owner)
              .map(|(_, entry)| entry.clone())
              .next_back();
            // (one whose line printed before every line the entry keeps, before the entry on a holder of it, or after
            // the address placed so before: `LINE_FIRST`, R8)
            if template.get_attribute(LINE_FIRST).is_some()
              && let Some(entry) = entry_on_owner.clone()
            {
              let placed_ok = match before.get(&owner).cloned() {
                Some(mut previous) => previous.add_next_sibling(&mut placed).is_ok(),
                None => entry.clone().add_prev_sibling(&mut placed).is_ok(),
              };
              if placed_ok {
                before.insert(owner, placed);
              }
              continue;
            }
            // (after the entry, or the address placed after it before)
            let anchor = after.get(&owner).cloned().or(entry_on_owner);
            if let Some(mut anchor) = anchor
              && anchor.add_next_sibling(&mut placed).is_err()
            {
              continue;
            }
            after.insert(owner, placed);
          }
        },
        None => {
          let Some(mut block) = block_creator(document, &authors)? else {
            continue;
          };
          document.append_clone(&mut block, vec![template.clone()])?;
          if let Some(placed) = element_nodes(&block).last().cloned() {
            if written
              .iter()
              .any(|(node, address)| node == template && *address == *text)
            {
              set_written_out(&placed, text);
            }
            to_block.push((text.clone(), placed));
          }
        },
      }
    }
    for email in &mine {
      document.remove_node(email.clone());
    }
    taken_away.extend(emptied);
  }
  // (a block copy an author holds, or a second one on the block, goes)
  let mut on_block: Vec<String> = Vec::new();
  for (text, placed) in to_block {
    let held = authors.iter().find_map(|author| {
      element_nodes(author).into_iter().find(|c| {
        c.get_name() == "contact"
          && c.get_attribute("role").as_deref() == Some("email")
          && c.get_content().trim() == text
      })
    });
    if let Some(held) = held.clone()
      && only.is_some()
    {
      carry_printed_label(&placed, held, default_name.as_deref());
    }
    if held.is_some() || on_block.contains(&text) {
      if let Some(parent) = placed.get_parent() {
        taken_away.push(parent);
      }
      document.remove_node(placed);
    } else {
      on_block.push(text);
    }
  }
  // (a nameless creator the moves emptied is none)
  for creator in taken_away {
    if creator.get_name() == "creator"
      && document
        .findnode("ltx:personname", Some(&creator))
        .is_none()
      && element_nodes(&creator).is_empty()
    {
      document.remove_node(creator);
    }
  }
  Ok(())
}

/// An address's owners, by the Link step (`None`, the block's), and the rule that placed it.
type Placement = (Option<Vec<usize>>, &'static str);

/// The placements of the addresses of each `{…}@domain` list set over several lines among a Link step entry's
/// `addresses` ([`link_entry_addresses`]; the Kind step's [`LINE_LIST`]), `None` for an address of no such list: the
/// author block's reader's placement of the list on one line, over every author (`authors`, keyed `keys`), not the
/// entry's carriers — the author an entry spells (`spelled`), else in author order, else the only author, else by
/// elimination, else the block (`email_owners::list_owners`; FIDELITY_CHARTER §4, review rev107a finding 4: 2305.13191, 1808.00588). `mine`
/// the entry's e-mails, which hold no address of their own outside the list. `mark_lists`, placed the same way: the lists
/// of a line a mark leads that are no carriers' (the addresses' places in `addresses`; 2102.08716, 2412.13509).
fn list_placements(
  document: &mut Document,
  authors: &[Node],
  keys: &[NameKeys],
  addresses: &[(String, Node, Vec<Option<usize>>)],
  spelled: &[Option<usize>],
  mine: &[Node],
  mark_lists: &[Vec<usize>],
) -> Vec<Option<Placement>> {
  let mut listed: Vec<Option<Placement>> = vec![None; addresses.len()];
  let address_word = |text: &str| email_address_word(text).unwrap_or_default();
  let mut lists: Vec<(Option<String>, Vec<usize>)> = Vec::new();
  for (_, template, _) in addresses {
    if let Some(list) = template.get_attribute(LINE_LIST)
      && !lists
        .iter()
        .any(|(listed, _)| listed.as_deref() == Some(list.as_str()))
    {
      let members: Vec<usize> = (0..addresses.len())
        .filter(|&at| addresses[at].1.get_attribute(LINE_LIST).as_deref() == Some(list.as_str()))
        .collect();
      lists.push((Some(list), members));
    }
  }
  lists.extend(mark_lists.iter().map(|members| (None, members.clone())));
  for (list, members) in lists {
    let texts: Vec<String> = members
      .iter()
      .map(|&at| address_word(&addresses[at].0))
      .collect();
    // (the e-mails outside the list: those an author holds, and those no author does — the block's, or this entry's
    // others —, by the address they print)
    let mut held_elsewhere: Vec<(usize, String)> = Vec::new();
    for (at, author) in authors.iter().enumerate() {
      for contact in element_nodes(author) {
        if contact.get_name() == "contact"
          && contact.get_attribute("role").as_deref() == Some("email")
          && (list.is_none() || contact.get_attribute(LINE_LIST) != list)
          && !mine.contains(&contact)
        {
          held_elsewhere.push((at, address_word(&contact.get_content())));
        }
      }
    }
    let unheld_elsewhere: Vec<String> = document
      .findnodes(
        "//ltx:creator[not(ltx:personname)]/ltx:contact[@role='email']",
        None,
      )
      .iter()
      // (a line's list a mark leads may wait on the block's creator, 2102.08716's legend: its own addresses are not
      // elsewhere)
      .filter(|contact| list.is_some() || !mine.contains(contact))
      .map(|contact| address_word(&contact.get_content()))
      .chain(
        (0..addresses.len())
          .filter(|at| !members.contains(at))
          .map(|at| address_word(&addresses[at].0)),
      )
      .collect();
    let mut taken = vec![false; authors.len()];
    for (holder, address) in &held_elsewhere {
      taken[*holder] = true;
      if let Some(owner) = spelled_owner(keys, address) {
        taken[owner] = true;
      }
    }
    let mut spelled_elsewhere = false;
    for address in &unheld_elsewhere {
      if let Some(owner) = spelled_owner(keys, address) {
        taken[owner] = true;
        spelled_elsewhere = true;
      }
    }
    spelled_elsewhere |= held_elsewhere
      .iter()
      .any(|(_, address)| spelled_owner(keys, address).is_some());
    let elsewhere_open = unheld_elsewhere
      .iter()
      .any(|address| spelled_owner(keys, address).is_none() && !is_role_mailbox(address));
    let member_spelled: Vec<Option<usize>> = members.iter().map(|&at| spelled[at]).collect();
    let owners = list_owners(
      keys,
      &texts,
      &member_spelled,
      &taken,
      spelled_elsewhere,
      elsewhere_open,
    );
    for (at, (owner, rule)) in members.into_iter().zip(owners) {
      listed[at] = Some((owner.map(|owner| vec![owner]), rule));
    }
  }
  listed
}

/// The address an e-mail's text prints: its word holding an `@`, its edge punctuation aside.
fn email_address_word(text: &str) -> Option<String> {
  text
    .split_whitespace()
    .find(|word| word.contains('@'))
    .map(|word| {
      word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string()
    })
}

/// The addresses [`kind_address_lines`] takes out of today's entry `entry` as its holders' e-mails, line by line, read
/// as that pass reads them (`spells`: whether a local part spells an author): what the writer would lose making the
/// entry's lines one author's each (`plan::cut_block_lines`, 2310.19837; a template it leaves as printed,
/// 2203.17007's `firstname.secondname@ericsson.com`).
pub(in crate::base_utilities) fn entry_line_addresses(
  entry: &Node,
  spells: &dyn Fn(&str) -> bool,
) -> Vec<String> {
  let lines = entry_lines(entry);
  let printed_lines = lines
    .iter()
    .filter(|(nodes, _)| nodes.iter().any(|n| !n.get_content().trim().is_empty()))
    .count();
  let mut out = Vec::new();
  for (nodes, _) in &lines {
    let mut line = LineText::default();
    if nodes.iter().all(|node| line.read(node))
      && let Some(cut) = address_line_cut(
        &line.text,
        spells,
        printed_lines == 1 && line.opens_with_mark(),
      )
    {
      out.extend(cut.addresses);
    }
  }
  out
}

/// Rust-only (OXIDIZED_DESIGN_DIVERGENCES #966): the Kind step for a line of addresses inside an affiliation-kind entry
/// (an affiliation, alternate affiliation or address, [`CUT_ROLES`]), read at the flush, where every route that makes
/// the entry meets — a class's block (IEEEtran's `\IEEEauthorblockA{… \\ E-mail: a@x, \{b, c\}@y, and d@z}`,
/// 2302.02638; one built of a document's macros, `\IEEEauthorblockA{\hpcaaffiliation\\\hpcaemail}`, 2608.22602),
/// authblk's `\affil{… \authorcr Email: {\tt\{…\}@ufl.edu}}` (2111.01631, 2309.05601), an author block's line
/// (`\normalsize{$^*$These authors contributed equally. \\ Email: a@x, b@y}`, 2207.11193) —, each entry relocated to
/// its holders and the frontmatter observer's writing done (so the writer reads today's frontmatter as before: run
/// ahead of it, the pass made the writer decline 1509.06333's IEEE names re-part and 2501.04244's notes), before the
/// Kind step for a legend. The token-level Kind step (`affiliations::kind_entry_addresses`) reads a line its tokens
/// show; this reads the line as it printed, past a document's macros, `\authorcr`, a font switch, a connective "and", a
/// mark before an address. A line (the entry's children between two `ltx:break`s) whose printed text, a mark aside (a
/// superscript or a formula printing footnote symbols or a number: [`is_mark_text`]), is one run of addresses
/// (`resolver::address_runs`, the reader the frontmatter observer takes them out by) behind a label or none — an e-mail
/// label ("E-mail:", "Emails:", "(email: …)") or the words naming a contact ("Corresponding author:",
/// "Correspondence:", 2609.21085, 2502.19130) — leaves the entry, with the line break before it; a line whose words
/// come before the label (`∗To whom correspondence should be addressed; E-mail: x`, 1111.6932, 1808.02718,
/// astro-ph0306103; `…, USA. Corresponding author: x`, 2305.01135) gives up the label and the run, its words staying
/// ([`address_line_cut`]). Each address becomes an e-mail contact of the entry on each holder (its `ENTRY_KEY`; made by
/// the Kind step, `ENTRY_KINDED`), named by the label as printed (FIDELITY_CHARTER §4: no printed text dropped), else
/// by the class's name of an entry of this line alone, else by the role's own, in the font it printed in; the Link
/// step, run again over these entries alone, places them by the charter's chain ([`link_entry_addresses`]: the author
/// it spells; else, for an address the line's mark leads ([`LINE_MARK`], [`address_marks`]), the authors whose names
/// answer the mark, or the block where none does — marks win, user ruling R1: gr-qc0207058's `$**$ email: indrani@…`
/// is I. Chowdhury's `∗∗`, review rev106a B1; else the entry's only carrier, …, else the block once), a label going onto
/// an address an author holds already ([`carry_printed_label`]), a mark no name answers shown before the first address
/// it leads ([`show_line_marks`]). An entry whose only line opens with a mark and is a contact's words before its
/// addresses (`∗Corresponding author: x`) keeps the words, the legend the Kind step for a legend reads
/// ([`kind_legend_contacts`]). An unlabelled tail of addresses after a `;`, a comma or a full stop and a space (`…,
/// China; lha@…, xuey@…`, 2602.07392; `University of Maryland, jda@…`, 1903.04089; 1701.07541, 1204.4622), a label of
/// contact words and "to" (`Corresponding\ to\ x@y`, 2606.31381), the lines a font wrapper holds ([`entry_lines`]:
/// 2603.02131, 2104.11106) and a list's other printed forms (`[a, b]@x`, `{a, b} @ x`, `{a@p., b@}x.y`, `{a, b}.civ@x`:
/// `resolver::address_runs`; 2405.17925, 1707.04775, 2108.00873, 2605.01350, 1908.06816) are read as well (107c). Left as
/// they print (the frontmatter observer's text reading's): an address glued to the word before it (`smith,j@x.org`; one in a font of its own starts at that font, `China.{\\tt x@y}`, 2203.06585) or
/// beside prose (`see data@a.org for data`), a template or a list the reader could not part (`{name.surname}@x`,
/// [`is_pattern_address`], [`lists_local_parts`]), a run a tag follows (`a@x (Matvey)`, 2512.10975), a line holding a
/// formula, a note or a link that is no address's. Returns the keys of the entries it took lines out of. Guards
/// `perfect_kernel_batch106a`.
fn kind_address_lines(document: &mut Document) -> Result<HashSet<String>> {
  let mut kinded: HashSet<String> = HashSet::default();
  let entries: Vec<Node> = document
    .findnodes(&s!("//ltx:creator/ltx:contact[@{ENTRY_KEY}]"), None)
    .into_iter()
    .filter(|entry| {
      entry
        .get_attribute("role")
        .is_some_and(|role| CUT_ROLES.contains(&role.as_str()))
        && entry.get_content().contains('@')
    })
    .collect();
  if entries.is_empty() {
    return Ok(kinded);
  }
  // (the authors' names read as the Link step reads them, for a list's parts that spell one: `address_runs`)
  let keys: Vec<NameKeys> = document
    .findnodes("//ltx:creator[@role='author']/ltx:personname", None)
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(&name.get_content()))))
    .collect();
  let spells = |local: &str| spelled_owner(&keys, &s!("{local}@x.org")).is_some();
  let default_name = get_frontmatter_name(None, "ltx:contact", "email")?;
  // (the marks the authors' names show or request, as the Link step reads them: a line's mark no name answers stays
  // printed, `LINE_MARK_SHOWN`)
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  let author_marks = author_mark_keys(document, &authors);
  let glued_marks = glued_name_marks(document, &authors);
  DebugFeature!(
    "frontmatter",
    "FRONT kind address line: the authors' marks {author_marks:?}, glued {glued_marks:?}"
  );
  // (the split lists left unread, each warned of once however many holders print it)
  let mut warned: HashSet<String> = HashSet::default();
  for entry in entries {
    let Some(creator) = entry.get_parent() else {
      continue;
    };
    let entry_name = entry.get_attribute("name");
    let role_name = get_frontmatter_name(
      None,
      "ltx:contact",
      &entry.get_attribute("role").unwrap_or_default(),
    )?;
    let lines = entry_lines(&entry);
    let printed_lines = lines
      .iter()
      .filter(|(nodes, _)| nodes.iter().any(|n| !n.get_content().trim().is_empty()))
      .count();
    // (a footnote symbol leading the entry: its unspelled addresses every carrier's, as `led_by_footnote_symbol` reads
    // the entry's tokens)
    let symbol = element_nodes(&entry).first().is_some_and(|first| {
      first.get_name() == "sup"
        && entry
          .get_first_child()
          .is_some_and(|child| child == *first || child.get_content().trim().is_empty())
        && first.get_content().trim().chars().any(events::is_fnsymbol)
    });
    let mut made: Vec<MadeAddress> = Vec::new();
    // (whether a line before this one prints and stays in the entry)
    let mut kept_before = false;
    for (at, (nodes, break_before)) in lines.iter().enumerate() {
      let prints = nodes.iter().any(|n| !n.get_content().trim().is_empty());
      let mut line = LineText::default();
      let read = nodes.iter().all(|node| line.read(node));
      // (a legend the Kind step for a legend can make a note of: the entry's only line, opened by a mark)
      let legend = printed_lines == 1 && line.opens_with_mark();
      let cut = read
        .then(|| address_line_cut(&line.text, &spells, legend))
        .flatten();
      let Some(mut cut) = cut else {
        // (a list of local parts set over several lines, which the line's reading declines — a formula or a note in
        // it, a template, a list the reader cannot part —, stays where it prints, counted: never silently an
        // affiliation's text, html_feedback #7233)
        let printed: String = nodes.iter().map(Node::get_content).collect();
        if split_address_list(nodes) && warned.insert(printed.clone()) {
          Warn!(
            "frontmatter",
            "address-list",
            s!(
              "A list of e-mail addresses set over several lines stays in the {} it prints in, unread: {}",
              entry.get_attribute("role").unwrap_or_default(),
              printed.split_whitespace().collect::<Vec<_>>().join(" ")
            )
          );
        }
        kept_before |= prints;
        continue;
      };
      // (an address starts where its font does, the word glued before it staying with the entry's text: `Hangzhou,
      // China.{\\tt xiangzy@zju.edu.cn}` is no address `China.xiangzy@`, 2203.06585, review rev107g)
      if cut.label.is_none()
        && let Some(start) = line.address_font_start(cut.address_start)
      {
        let glued = line.text[cut.address_start..start].to_string();
        if let Some(address) = cut.addresses.first_mut()
          && let Some(rest) = address.strip_prefix(glued.as_str()).map(str::to_string)
        {
          *address = rest;
          cut.address_start = start;
          cut.start = start;
        }
      }
      // (a whole line before every line the entry keeps prints before the entry's text: R8, its e-mail placed before the
      // entry, `\IEEEauthorblockA{\href{mailto:…}{…}\\ \url{https://orcid.org/…}\\ …}`, 2206.10577 — review rev107c N1)
      let first = cut.start == 0 && !kept_before;
      kept_before |= cut.start != 0;
      DebugFeature!(
        "frontmatter",
        "FRONT kind address line: {:?} of {:?}, label {:?}",
        cut.addresses,
        line.text,
        cut.label
      );
      let font = line.address_font(cut.address_start, &entry);
      // (a list set over several lines — a line break inside its printed brace, or between the brace and its domain:
      // `LineText::read` —, one list however many holders print it, whose addresses the Link step places as the
      // one-line form's, [`LINE_LIST`])
      let list = (!line.breaks.is_empty()).then(|| {
        s!(
          "{}:{at}",
          entry.get_attribute(ENTRY_KEY).unwrap_or_default()
        )
      });
      // (the label it printed; else, an entry of this line alone that its class names — REVTeX's
      // `\altaffiliation[Electronic address: ]{…}` —, the class's name, as the token-level Kind step gives it)
      let name = match &cut.label {
        // (a label ending in its connective, printed with no colon: "Corresponding to ", which `write_role_notes` gives
        // its owner as a correspondence note)
        Some(label) if label.to_lowercase().ends_with(" to") => Some(s!("{label} ")),
        Some(label) => Some(s!("{label}: ")),
        None if cut.start == 0 && printed_lines == 1 && entry_name != role_name => {
          entry_name.clone()
        },
        None => default_name.clone(),
      };
      // (the marks leading each address — the line's opening one, `$**$ email: cx@…, c\_dee@…` (gr-qc0207058), or one
      // before each, `$^1$a@x, $^2$b@y` (1706.04764, 2505.11901) — or following each, its key on the Link step's
      // placement, `LINE_MARK`; one no name answers shown on the first address it leads where the cut takes it out of
      // the entry, review rev106a S1)
      let marks = line.marks();
      let mut shown: Vec<usize> = Vec::new();
      let last = cut.addresses.len().saturating_sub(1);
      for (n, (address, leading)) in cut
        .addresses
        .iter()
        .zip(address_marks(&line.text, &marks, &cut))
        .enumerate()
      {
        let mark = (!leading.is_empty()).then(|| {
          leading
            .iter()
            .map(|&i| marks[i].1.as_str())
            .collect::<Vec<_>>()
            .join(",")
        });
        // (a role tag after the last address is its owner's correspondence note, its mark the address's line mark:
        // `Email: x@y (Corresponding Author)`, 2503.14354; `(* corresponding author)`, 2210.14319 — [`role_tag`])
        let (name, mark) = match &cut.role_tag {
          Some((phrase, tag_mark)) if n == last => (
            Some(s!(
              "{phrase}, {}",
              name
                .clone()
                .or_else(|| default_name.clone())
                .unwrap_or_default()
            )),
            mark.or_else(|| tag_mark.clone()),
          ),
          _ => (name.clone(), mark),
        };
        let show: Vec<&str> = leading
          .iter()
          .filter(|&&i| {
            marks[i].0 >= cut.start
              && answering_or_glued(&author_marks, &glued_marks, &marks[i].1).is_empty()
              && !shown.contains(&i)
          })
          .map(|&i| marks[i].1.as_str())
          .collect();
        shown.extend(leading.iter().copied());
        made.push((
          name.clone().unwrap_or_default(),
          address.clone(),
          font.clone(),
          mark,
          (!show.is_empty()).then(|| show.join("")),
          list.clone(),
          first,
        ));
      }
      if cut.start == 0 {
        // (the whole line, with the break before it — the one after it, the entry's first line —, and each font
        // wrapper it leaves empty)
        let mut parents: Vec<Node> = Vec::new();
        for node in nodes {
          parents.extend(node.get_parent());
          document.remove_node(node.clone());
        }
        let separator = break_before
          .clone()
          .or_else(|| lines.get(at + 1).and_then(|(_, next)| next.clone()));
        if let Some(separator) = separator
          && separator.get_parent().is_some()
        {
          parents.extend(separator.get_parent());
          document.remove_node(separator);
        }
        drop_emptied_wrappers(document, parents);
      } else {
        line.cut_from(document, cut.start);
      }
    }
    if made.is_empty() {
      continue;
    }
    let key = entry.get_attribute(ENTRY_KEY).unwrap_or_default();
    kinded.insert(key.clone());
    let saved = document.get_node().clone();
    document.set_node(&creator);
    // (a first line's e-mails print before the entry only where a line of the entry's text stays after them)
    let entry_after = kept_before;
    for (name, address, font, mark, show, list, first) in made {
      let mut attributes: HashMap<String, String> = HashMap::default();
      attributes.insert("role".to_string(), "email".to_string());
      if !name.is_empty() {
        attributes.insert("name".to_string(), name);
      }
      if let Some(mark) = mark {
        attributes.insert(LINE_MARK.to_string(), mark);
      }
      if let Some(show) = show {
        attributes.insert(LINE_MARK_SHOWN.to_string(), show);
      }
      if let Some(list) = list {
        attributes.insert(LINE_LIST.to_string(), list);
      }
      if first && entry_after {
        attributes.insert(LINE_FIRST.to_string(), "true".to_string());
      }
      attributes.insert(ENTRY_KEY.to_string(), key.clone());
      attributes.insert(ENTRY_KINDED.to_string(), "true".to_string());
      if symbol {
        attributes.insert(ENTRY_SYMBOL.to_string(), "true".to_string());
      }
      if let Some(ordinal) = entry.get_attribute(QUEUE_ORDINAL) {
        attributes.insert(QUEUE_ORDINAL.to_string(), ordinal);
      }
      // (the entry's source, which a copy of the entry's makes its addresses' too: `drop_repeated_contacts`)
      if let Some(source) = entry.get_attribute(CONTACT_SOURCE) {
        attributes.insert(CONTACT_SOURCE.to_string(), source);
      }
      // (in the entry's font, the addresses in theirs)
      if let Some(entry_font) = entry.get_attribute("_font") {
        attributes.insert("_font".to_string(), entry_font);
      }
      let mut contact = document.open_element("ltx:contact", Some(attributes), None)?;
      match font {
        Some(font) => {
          let mut wrapper_attributes: HashMap<String, String> = HashMap::default();
          wrapper_attributes.insert("_font".to_string(), font);
          let mut wrapper = document.open_element("ltx:text", Some(wrapper_attributes), None)?;
          let _ = wrapper.set_content(&address);
          document.close_element("ltx:text")?;
        },
        None => {
          let _ = contact.set_content(&address);
        },
      }
      document.close_element("ltx:contact")?;
    }
    document.set_node(&saved);
  }
  Ok(kinded)
}

/// A `{…}@domain` list set over several lines that the author block read as contacts of their own — its head an
/// affiliation-kind entry leaving a printed brace open (`{e.de.bruyn,mo.schaeffler,…,`), its tail the contacts after it
/// that close it before an `@` (`s.kesselheim,a.bazarova}@fz-juelich.de`, an e-mail contact): the tail joined onto the
/// head's line, a line break between them, where the joined line is a line of addresses alone ([`address_line_cut`]), so
/// the Kind step for an address line reads it as one list ([`entry_lines`]; a `\And` column's lines, html_feedback
/// #7233's layout); else left as they print, counted (`Warning:frontmatter:address-list`).
fn join_split_address_contacts(document: &mut Document) -> Result<()> {
  let keys: Vec<NameKeys> = document
    .findnodes("//ltx:creator[@role='author']/ltx:personname", None)
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(&name.get_content()))))
    .collect();
  let spells = |local: &str| spelled_owner(&keys, &s!("{local}@x.org")).is_some();
  let mut warned: HashSet<String> = HashSet::default();
  for creator in document.findnodes("//ltx:creator", None) {
    let contacts: Vec<Node> = element_nodes(&creator)
      .into_iter()
      .filter(|node| node.get_name() == "contact")
      .collect();
    let mut at = 0;
    while at < contacts.len() {
      let head = &contacts[at];
      let role = head.get_attribute("role").unwrap_or_default();
      // (the head's last line, after its last line break)
      let last_line: String = head
        .get_child_nodes()
        .iter()
        .rev()
        .take_while(|node| {
          node.get_name() != "break" || node.get_type() != Some(NodeType::ElementNode)
        })
        .map(Node::get_content)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
      let mut open = brace_depth(&last_line);
      if !CUT_ROLES.contains(&role.as_str()) || open <= 0 || head.get_content().contains('@') {
        at += 1;
        continue;
      }
      let close = (at + 1..contacts.len())
        .take_while(|&next| {
          contacts[next]
            .get_attribute("role")
            .is_some_and(|role| role == "email" || CUT_ROLES.contains(&role.as_str()))
        })
        .find(|&next| {
          open += brace_depth(&contacts[next].get_content());
          open <= 0
        });
      let close = close.filter(|&close| contacts[close].get_content().contains("}@"));
      // (or the one contact just before the head, where the tail was placed first: a nested `\IEEEauthorblockA` holding
      // the list's last line, `\IEEEauthorblockA{{\{a, b,} \IEEEauthorblockA{c\}@x}}`, is digested — and placed — inside
      // the outer one, before it; 2410.15681: brief 107f)
      let before = (close.is_none() && at > 0).then(|| at - 1).filter(|&tail| {
        let text = contacts[tail].get_content();
        contacts[tail]
          .get_attribute("role")
          .is_some_and(|role| role == "email" || CUT_ROLES.contains(&role.as_str()))
          && text.contains("}@")
          && brace_depth(&last_line) + brace_depth(&text) <= 0
      });
      let tails: Vec<Node> = match (close, before) {
        (Some(close), _) => contacts[at + 1..=close].to_vec(),
        (None, Some(tail)) => vec![contacts[tail].clone()],
        (None, None) => {
          at += 1;
          continue;
        },
      };
      let tail: Vec<String> = tails.iter().map(Node::get_content).collect();
      let joined = format!("{last_line}\n{}", tail.join("\n"));
      if address_line_cut(&joined, &spells, false).is_some_and(|cut| cut.start == 0) {
        let mut head = head.clone();
        for tail in &tails {
          let mut separator = Node::new("break", None, document.get_document())?;
          if let Some(namespace) = head.get_namespace() {
            let _ = separator.set_namespace(&namespace);
          }
          head.add_child(&mut separator)?;
          for mut child in tail.get_child_nodes() {
            child.unlink_node();
            head.add_child(&mut child)?;
          }
          document.remove_node(tail.clone());
        }
      } else if warned.insert(joined.clone()) {
        Warn!(
          "frontmatter",
          "address-list",
          s!(
            "A list of e-mail addresses set over several lines stays in the {role} it prints in, unread: {}",
            joined.split_whitespace().collect::<Vec<_>>().join(" ")
          )
        );
      }
      at = close.map_or(at + 1, |close| close + 1);
    }
  }
  Ok(())
}

/// An address [`kind_address_lines`] reads off an entry's line: its e-mail's name, its text, its font, the marks leading
/// it ([`LINE_MARK`]), the marks it shows — those no name answers, which the line printed and its cut takes out
/// ([`LINE_MARK_SHOWN`]), the list set over several lines it is an address of ([`LINE_LIST`]), and whether its line
/// prints before every line the entry keeps ([`LINE_FIRST`]).
type MadeAddress = (
  String,
  String,
  Option<String>,
  Option<String>,
  Option<String>,
  Option<String>,
  bool,
);

/// An entry's lines for [`kind_address_lines`]: its children between two line breaks (`ltx:break`), each with the break
/// before it (none for the first). Lines a printed brace opens and a later line closes are one, the breaks between them
/// its own (a list of local parts set over two lines, `\texttt{\{ann.able,xq.zz,}\\ \texttt{cy.coe\}@alpha.edu}`:
/// html_feedback #7233, [`LineText::read`]); a brace no later line closes joins nothing. A font wrapper (`ltx:text`)
/// holding line breaks of its own is read through, its children lines like the entry's: a font declaration's reach
/// over the lines after it (`\scriptsize{$^1$Dept …} \\ … \scriptsize{$^\ast$~Correspondence: \texttt{a@x} and
/// \texttt{b@y}}`, 2603.02131; `\author{…\\ $^1$ Know-Center - Graz, Austria \\ \{aremonda, …\} @ know-center.at}`,
/// 2104.11106: 107c).
fn entry_lines(entry: &Node) -> Vec<(Vec<Node>, Option<Node>)> {
  fn read_lines(parent: &Node, lines: &mut Vec<(Vec<Node>, Option<Node>)>) {
    for child in parent.get_child_nodes() {
      let element = child.get_type() == Some(NodeType::ElementNode);
      if element && child.get_name() == "break" {
        lines.push((Vec::new(), Some(child)));
      } else if element
        && child.get_name() == "text"
        && child.get_child_nodes().iter().any(|grandchild| {
          grandchild.get_type() == Some(NodeType::ElementNode) && grandchild.get_name() == "break"
        })
      {
        read_lines(&child, lines);
      } else if let Some((nodes, _)) = lines.last_mut() {
        nodes.push(child);
      }
    }
  }
  let mut lines: Vec<(Vec<Node>, Option<Node>)> = vec![(Vec::new(), None)];
  read_lines(entry, &mut lines);
  let depth = |nodes: &[Node]| -> isize {
    nodes
      .iter()
      .map(|node| brace_depth(&node.get_content()))
      .sum()
  };
  let mut joined: Vec<(Vec<Node>, Option<Node>)> = Vec::with_capacity(lines.len());
  let mut at = 0;
  while at < lines.len() {
    let mut open = depth(&lines[at].0);
    let text = |nodes: &[Node]| nodes.iter().map(Node::get_content).collect::<String>();
    let close = (open > 0)
      .then(|| {
        (at + 1..lines.len()).find(|&next| {
          open += depth(&lines[next].0);
          open <= 0
        })
      })
      .flatten()
      .map(|close| {
        // (and the domain set on the line after the brace closes, `}` / `@x`)
        let goes_on = lines
          .get(close + 1)
          .is_some_and(|(next, _)| domain_goes_on(&text(&lines[close].0), &text(next)));
        close + usize::from(goes_on)
      })
      .or_else(|| {
        lines
          .get(at + 1)
          .filter(|(next, _)| domain_goes_on(&text(&lines[at].0), &text(next)))
          .map(|_| at + 1)
      });
    let (mut nodes, before) = lines[at].clone();
    if let Some(close) = close {
      for (more, between) in &lines[at + 1..=close] {
        nodes.extend(between.clone());
        nodes.extend(more.iter().cloned());
      }
      joined.push((nodes, before));
      at = close + 1;
    } else {
      joined.push((nodes, before));
      at += 1;
    }
  }
  joined
}

/// Whether an entry's line `nodes` is a list of local parts set over several lines: a line break, nested or not, between
/// a printed brace and the `@` after the brace that closes it (`{a,b,` / `c}@x`; `{a,b}` / `@x`).
fn split_address_list(nodes: &[Node]) -> bool {
  // (the line's text, each line break in it a line separator)
  fn printed(node: &Node, text: &mut String) {
    match node.get_type() {
      Some(NodeType::ElementNode) if node.get_name() == "break" => text.push('\u{2028}'),
      Some(NodeType::ElementNode) => {
        for child in node.get_child_nodes() {
          printed(&child, text);
        }
      },
      _ => text.push_str(&node.get_content()),
    }
  }
  let mut text = String::new();
  for node in nodes {
    printed(node, &mut text);
  }
  let Some(open) = text.find('{') else {
    return false;
  };
  let mut depth = 0isize;
  for (at, c) in text[open..].char_indices() {
    match c {
      '{' => depth += 1,
      '}' => {
        depth -= 1;
        if depth == 0 {
          let rest = &text[open + at + 1..];
          let after = rest.trim_start_matches(|c: char| c.is_whitespace());
          return opens_with_domain(after)
            && text[open..open + at + 1 + (rest.len() - after.len())].contains('\u{2028}');
        }
      },
      _ => {},
    }
  }
  false
}

/// Whether a line ending at `before` and the line `after` it are one address list's, its domain set on the next line:
/// the first closes a printed brace list (`{majid,…,svetha.venkatesh}`), the second opens with its `@` and domain
/// (`@deakin.edu.au`, 1902.04228; [`opens_with_domain`]).
fn domain_goes_on(before: &str, after: &str) -> bool {
  let before = before.trim_end();
  before.ends_with('}') && before.contains('{') && opens_with_domain(after.trim_start())
}

/// Whether `text` opens with an `@` and a domain: a word with a dot inside it (`@fz-juelich.de`), not a handle
/// (`lib\{core\}` / `@alpha\_lab on Twitter` is no list of addresses, review rev107a finding 5).
fn opens_with_domain(text: &str) -> bool {
  text.strip_prefix('@').is_some_and(|rest| {
    let domain: String = rest
      .chars()
      .take_while(|c| !c.is_whitespace() && !",;()".contains(*c))
      .collect();
    domain
      .trim_end_matches(|c: char| !c.is_alphanumeric())
      .contains('.')
  })
}

/// The printed braces `text` opens and leaves open (negative: closes more than it opens).
fn brace_depth(text: &str) -> isize {
  text.chars().fold(0, |depth, c| match c {
    '{' => depth + 1,
    '}' => depth - 1,
    _ => depth,
  })
}

/// Whether a character is a mark's symbol for [`kind_address_lines`]: a footnote symbol (`events::is_fnsymbol`) or an
/// asterisk of any form ([`ASTERISK_FORMS`]: all asterisks are one key, user ruling 2026-11-02), or an envelope, the
/// correspondence mark (marvosym's `\Letter`, ✉; 107c r2).
fn is_mark_glyph(c: char) -> bool {
  events::is_fnsymbol(c) || ASTERISK_FORMS.contains(&c) || matches!(c, '\u{2709}' | '\u{1F582}')
}

/// Whether a superscript's text is a mark ([`kind_address_lines`]): footnote symbols, numbers and their commas, or one
/// letter (`$^a$`).
fn is_mark_text(text: &str) -> bool {
  let text = text.trim();
  !text.is_empty()
    && (text
      .chars()
      .all(|c| is_mark_glyph(c) || c.is_ascii_digit() || c == ',' || c.is_whitespace())
      || (text.chars().count() == 1 && text.chars().all(|c| c.is_ascii_lowercase())))
}

/// A line's printed text as [`kind_address_lines`] reads it, each text node's place in it and each mark's (a space), and
/// the line breaks inside a printed brace it holds, each with its place (a newline).
#[derive(Default)]
struct LineText {
  text:   String,
  pieces: Vec<(Node, usize, usize)>,
  breaks: Vec<(Node, usize)>,
}

impl LineText {
  /// Reads `node` into the line: its text; a font wrapper's (`ltx:text`) and an address's link's (an `ltx:ref` printing
  /// an `@`) children; a mark ([`is_mark_text`], a superscript or a formula printing one) as a space; a line break
  /// inside a printed brace the line opened, as a newline (a list of local parts set over two lines, html_feedback
  /// #7233: [`entry_lines`]). `false` for anything else — another formula, a note, another link, another line break —,
  /// which the line is no line of addresses beside.
  fn read(&mut self, node: &Node) -> bool {
    match node.get_type() {
      Some(NodeType::TextNode) => {
        let start = self.text.len();
        self.text.push_str(&node.get_content());
        self.pieces.push((node.clone(), start, self.text.len()));
        true
      },
      Some(NodeType::ElementNode) => match node.get_name().as_str() {
        "text" => node.get_child_nodes().iter().all(|child| self.read(child)),
        "ref" if node.get_content().contains('@') => {
          node.get_child_nodes().iter().all(|child| self.read(child))
        },
        // (an empty superscript, which prints nothing: `\textsuperscript{}\{…\}.civ@mail.mil`, 1908.06816: 107c)
        "sup" if node.get_content().trim().is_empty() && element_nodes(node).is_empty() => true,
        // (a mark set as a superscript, or as a formula not yet parsed: `$^{\ast}$`, 2609.21085)
        "sup" | "Math" if is_mark_text(&node.get_content()) => {
          let start = self.text.len();
          self.text.push(' ');
          self.pieces.push((node.clone(), start, start + 1));
          true
        },
        "break" if brace_depth(&self.text) > 0 => {
          self.breaks.push((node.clone(), self.text.len()));
          self.text.push('\n');
          true
        },
        // (or a line break between a list's closing brace and its domain, which goes on with no space: `{…}` / `@x`,
        // 1902.04228)
        "break"
          if node
            .get_next_sibling()
            .is_some_and(|next| domain_goes_on(&self.text, &next.get_content())) =>
        {
          self.breaks.push((node.clone(), self.text.len()));
          true
        },
        _ => false,
      },
      _ => true,
    }
  }

  /// The line's marks, each with its place in the text and its printed text: each superscript or formula read as a mark,
  /// and the footnote symbols or asterisks a text opening the line prints (`∗∗ email: …`, [`is_mark_glyph`]).
  fn marks(&self) -> Vec<(usize, String)> {
    let mut marks: Vec<(usize, String)> = Vec::new();
    let mut opening = true;
    for (node, start, _) in &self.pieces {
      if node.get_type() == Some(NodeType::ElementNode) {
        marks.push((*start, node.get_content().trim().to_string()));
        opening = false;
        continue;
      }
      let content = node.get_content();
      if opening && !content.trim().is_empty() {
        let lead = content.len() - content.trim_start().len();
        let glyphs: String = content[lead..]
          .chars()
          .take_while(|c| is_mark_glyph(*c))
          .collect();
        if !glyphs.is_empty() {
          marks.push((start + lead, glyphs));
        }
        opening = false;
      }
    }
    marks
  }

  /// Whether a mark opens the line (its first piece that prints: a superscript or a formula read as a mark).
  fn opens_with_mark(&self) -> bool {
    self
      .pieces
      .iter()
      .find(|(node, ..)| {
        node.get_type() != Some(NodeType::TextNode) || !node.get_content().trim().is_empty()
      })
      .is_some_and(|(node, ..)| node.get_type() == Some(NodeType::ElementNode))
  }

  /// The font the line's addresses printed in where it is not the entry's (its `_font`): that of the wrapper below
  /// `entry` round the text holding the first `@` after `from` (`{\tt …}`'s typewriter, 2111.01631).
  fn address_font(&self, from: usize, entry: &Node) -> Option<String> {
    let (node, ..) = self.pieces.iter().find(|(node, start, end)| {
      *end > from
        && node.get_type() == Some(NodeType::TextNode)
        && node.get_content()[from.saturating_sub(*start)..].contains('@')
    })?;
    let wrapper = node.get_parent().filter(|parent| parent != entry)?;
    let font = wrapper.get_attribute("_font")?;
    (entry.get_attribute("_font").as_ref() != Some(&font)).then_some(font)
  }

  /// Where the address from byte `from` on starts in a font of its own: the place of the text node holding its `@`, when
  /// what stands before that node back to `from` is one glued run in another font (`China.` before `{\\tt xiangzy@…}`,
  /// [`kind_address_lines`]).
  fn address_font_start(&self, from: usize) -> Option<usize> {
    let at = self.pieces.iter().position(|(node, start, end)| {
      *end > from
        && node.get_type() == Some(NodeType::TextNode)
        && node.get_content()[from.saturating_sub(*start)..].contains('@')
    })?;
    let (node, start, _) = &self.pieces[at];
    let glued = self.text.get(from..*start).filter(|run| !run.is_empty())?;
    let (before, ..) = self.pieces.get(at.checked_sub(1)?)?;
    let font = |piece: &Node| {
      piece
        .get_parent()
        .and_then(|parent| parent.get_attribute("_font"))
    };
    (!glued.chars().any(char::is_whitespace) && !glued.contains('@') && font(before) != font(node))
      .then_some(*start)
  }

  /// Takes the line's text from byte `cut` on out of the document: each text node after it, the one it falls in cut
  /// short, each mark after it, and each wrapper they leave empty.
  fn cut_from(&self, document: &mut Document, cut: usize) {
    let mut parents: Vec<Node> = Vec::new();
    for (node, at) in &self.breaks {
      if *at >= cut {
        if let Some(parent) = node.get_parent() {
          parents.push(parent);
        }
        document.remove_node(node.clone());
      }
    }
    for (node, start, end) in &self.pieces {
      if *start >= cut {
        if let Some(parent) = node.get_parent() {
          parents.push(parent);
        }
        document.remove_node(node.clone());
      } else if *end > cut {
        let content = node.get_content();
        let mut node = node.clone();
        let _ = node.set_content(&content[..cut - start]);
      }
    }
    drop_emptied_wrappers(document, parents);
  }
}

/// Removes each of `parents` — the wrappers a line's cut took nodes out of ([`kind_address_lines`]) — that the cut left
/// empty (a font wrapper or a link printing nothing, holding no element but such wrappers), and each wrapper above it it
/// leaves so. Each parent is read once (two cut nodes of one wrapper name it twice: `{\small\\ Email: a@x}`), and every
/// removal is decided before any is made, the outermost of each emptied chain removed — so no node is read after it is
/// freed (review rev107c S4).
fn drop_emptied_wrappers(document: &mut Document, parents: Vec<Node>) {
  let mut seen: HashSet<_> = HashSet::default();
  let mut gone: HashSet<_> = HashSet::default();
  let mut emptied: Vec<Node> = Vec::new();
  for parent in parents {
    if !seen.insert(parent.to_hashable()) {
      continue;
    }
    let mut at = Some(parent);
    while let Some(node) = at {
      if gone.contains(&node.to_hashable()) {
        break;
      }
      if node.get_type() != Some(NodeType::ElementNode)
        || !matches!(node.get_name().as_str(), "text" | "ref")
        || !node.get_content().trim().is_empty()
        || !element_nodes(&node)
          .iter()
          .all(|child| gone.contains(&child.to_hashable()))
      {
        break;
      }
      gone.insert(node.to_hashable());
      at = node.get_parent();
      emptied.push(node);
    }
  }
  for node in emptied {
    if node
      .get_parent()
      .is_none_or(|parent| !gone.contains(&parent.to_hashable()))
    {
      document.remove_node(node);
    }
  }
}

/// The address line [`kind_address_lines`] takes out of an entry's line: where its cut starts in the line's text (0 for
/// the whole line), the label it printed, its addresses, and where they start.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::base_utilities) struct AddressLineCut {
  pub start:         usize,
  pub label:         Option<String>,
  pub addresses:     Vec<String>,
  pub address_start: usize,
  /// a role tag after the last address, its correspondence phrase and the mark it prints, if one ([`role_tag`])
  pub role_tag:      Option<(String, Option<String>)>,
}

/// A role tag ending a line after its addresses: a bracket holding a correspondence role phrase and nothing else but a
/// mark before it (`skvishvakarma@… \textbf{(Corresponding Author)}`, 2503.14354; `xychen@…, wangcs@… (* corresponding
/// author)`, 2210.14319) — FIDELITY_CHARTER §4, "Corresponding author" is an annotation on the author: where the tag
/// starts, the phrase and the mark. The tag after the last address is that address's ([`kind_address_lines`]: the
/// phrase its owner's correspondence note, `write_role_notes`; the mark its line mark), the e-mail's text the address
/// alone (brief 107f C, 107c's open item).
fn role_tag(text: &str) -> Option<(usize, String, Option<String>)> {
  let end = text.trim_end();
  let end = end.strip_suffix(['.', ';', ',']).map_or(end, str::trim_end);
  let trimmed = end.strip_suffix(')')?;
  let open = trimmed.rfind('(')?;
  let inner = &trimmed[open + 1..];
  let phrase_at = inner.find(char::is_alphabetic)?;
  let mark: String = inner[..phrase_at]
    .chars()
    .filter(|c| !c.is_whitespace())
    .collect();
  if mark.chars().any(char::is_alphanumeric) {
    return None;
  }
  let phrase = inner[phrase_at..].trim_end();
  if correspondence_phrase(phrase) != Some(phrase.len()) {
    return None;
  }
  // (right after an address: its domain's last letter before the bracket)
  let before = text[..open].trim_end();
  let last_word = before
    .rsplit(char::is_whitespace)
    .next()
    .unwrap_or_default();
  (last_word.contains('@') && before.ends_with(char::is_alphanumeric)).then(|| {
    (
      open,
      phrase.split_whitespace().collect::<Vec<_>>().join(" "),
      (!mark.is_empty()).then_some(mark),
    )
  })
}

/// [`address_line_cut_untagged`] of a line, a role tag after its addresses read apart ([`role_tag`]).
pub(in crate::base_utilities) fn address_line_cut(
  text: &str,
  spells: &dyn Fn(&str) -> bool,
  legend: bool,
) -> Option<AddressLineCut> {
  if let Some((tag_start, phrase, mark)) = role_tag(text)
    && let Some(mut cut) = address_line_cut_untagged(&text[..tag_start], spells, legend)
  {
    cut.role_tag = Some((phrase, mark));
    return Some(cut);
  }
  address_line_cut_untagged(text, spells, legend)
}

/// The cut of a line's text that is a line of addresses ([`kind_address_lines`]): its last run of addresses
/// (`resolver::address_runs`), nothing but separators after it, a template or a list the reader could not part none of
/// them; before it, its label — the e-mail label the run took and the words naming a contact before it
/// ("Corresponding author:", "Contact e-mail:", [`names_a_contact`]) — and before that nothing printed but marks and
/// separators (the whole line), or words a label follows (the label and the run go, the words stay). `legend`: the line
/// is the entry's only one and a mark opens it, so a contact's words with no e-mail word before its addresses stay as
/// the legend the Kind step for a legend makes its carriers' note ([`kind_legend_contacts`]); an unmarked one would
/// stay an affiliation there, so its words name the e-mails instead (77y's block line `Corresponding Author: …`).
fn address_line_cut_untagged(
  text: &str,
  spells: &dyn Fn(&str) -> bool,
  legend: bool,
) -> Option<AddressLineCut> {
  let runs = resolver::address_runs(text, spells);
  let run = runs.last()?;
  if text[run.end..].chars().any(char::is_alphanumeric)
    || run
      .addresses
      .iter()
      .any(|address| is_pattern_address(address) || lists_local_parts(address))
  {
    return None;
  }
  // (where the addresses start: after the run's own label and an opening bracket round it)
  let address_start = run.label.as_ref().map_or(run.start, |label| {
    text[run.start..]
      .find(label.as_str())
      .map_or(run.start, |at| run.start + at + label.len())
  });
  let address_start = address_start
    + text[address_start..]
      .find(|c: char| !c.is_whitespace() && !"([<".contains(c))
      .unwrap_or(0);
  // (the words naming a contact before it, back to the first that names none; a "to" after them, with no colon, the
  // label's own: `$^{*}$ Corresponding\ to\ lingxiao.wang@…`, 2606.31381: 107c)
  let head = &text[..run.start];
  let mut label_start = head.len();
  let trimmed = head.trim_end();
  let connective = trimmed
    .rsplit_once(char::is_whitespace)
    .filter(|(before, to)| {
      to.eq_ignore_ascii_case("to")
        && before
          .trim_end()
          .rsplit(char::is_whitespace)
          .next()
          .is_some_and(names_a_contact)
    })
    .map(|(before, _)| before.trim_end());
  if run.label.is_some() || trimmed.ends_with(':') || connective.is_some() {
    let mut words = connective.unwrap_or_else(|| trimmed.trim_end_matches(':').trim_end());
    loop {
      let word_start = words.rfind(char::is_whitespace).map_or(0, |at| {
        at + words[at..].chars().next().map_or(1, char::len_utf8)
      });
      let word = &words[word_start..];
      if word.is_empty() || !names_a_contact(word) {
        break;
      }
      label_start = word_start;
      words = words[..word_start].trim_end();
    }
  }
  let separator = |c: char| c.is_whitespace() || ",;:-\u{2013}\u{2014}".contains(c);
  // (a correspondence role phrase before them, nothing but marks before it — `* Corresponding author, E-mail: qz@…`,
  // 1711.00192 —: the label's own, so the line is whole; the phrase is its owner's note, [`write_role_notes`] — review
  // rev107c B1, N4)
  let before = text[..label_start].trim_end_matches(separator);
  if let Some(phrase) = before.find(char::is_alphabetic)
    && !before[..phrase].chars().any(char::is_alphanumeric)
    && correspondence_phrase(&before[phrase..]) == Some(before.len() - phrase)
  {
    label_start = phrase;
  }
  let rest = text[..label_start].trim_end_matches(separator);
  let label = text[label_start..address_start]
    .trim()
    .trim_start_matches(|c: char| "([<".contains(c))
    .trim_end_matches(|c: char| separator(c) || "([<".contains(c))
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
  let label = (!label.is_empty()).then_some(label);
  let emailed = label.as_deref().is_some_and(|label| {
    let words: Vec<&str> = label.split_whitespace().collect();
    words.iter().any(|w| is_email_label(w))
      || words.windows(2).any(|pair| is_email_label(&pair.join(" ")))
  });
  let whole = !rest.chars().any(char::is_alphanumeric);
  let addresses = run.addresses.clone();
  if whole && legend && label.is_some() && !emailed {
    // (an entry that is a contact's words before its addresses alone: the words stay, its legend)
    return Some(AddressLineCut {
      start: text[..address_start]
        .trim_end_matches(|c: char| separator(c) || "([<".contains(c))
        .len(),
      label: None,
      addresses,
      address_start,
      role_tag: None,
    });
  }
  if whole {
    return Some(AddressLineCut {
      start: 0,
      label,
      addresses,
      address_start,
      role_tag: None,
    });
  }
  // (words before: only behind a label, and the label at a list's or a sentence's boundary — never inside a bracket the
  // words open and the cut's tail closes: `(Fax: +31-20/44-47999, E-mail: stefan@nat.vu.nl)` stays as printed, its
  // bracket whole (physics0308076, bundle int134a `fm_checks.py` check 7), as on int133a)
  let opens = |open: char, close: char| rest.matches(open).count() > rest.matches(close).count();
  if opens('(', ')') || opens('[', ']') {
    return None;
  }
  let bounded =
    (run.label.is_some() && label_start == head.len()) || rest.ends_with(['.', ';', ',', ')']);
  // (or no label: the line's tail after a list's or a sentence's boundary and a space — `…, China; lha@…, xuey@…`
  // (2602.07392), `…, USA; yalinwu@…` (1701.07541), `University of Maryland, jda@…` (1903.04089), `…Canada. \{hoyer,
  // jrashid\}@ucalgary.ca` (1204.4622) —, or a printed brace's or bracket's list, its own boundary (`Haifa, Israel
  // \{avico@, orengal@alumni.\}technion.ac.il, amiravni83@…`, 2309.06986): addresses alone, never an affiliation's text,
  // 107c. Not a word glued to its address (`smith,j@x.org`, an initial's list, review rev103a S1), a word before the
  // comma that may be a local part of the list (`Canada, mmoza102, wslee@…`, 2003.08808; `India ee13d015,
  // ramkrishna@…`, 1805.00710: no partial list cut), nor contact words alone (`email,ann@x.org`))
  let gap = &text[rest.len()..label_start];
  let words: Vec<&str> = rest
    .split(|c: char| c.is_whitespace() || ",;:.".contains(c))
    .filter(|word| !word.is_empty())
    .collect();
  let listed_before = gap.contains(',')
    && !gap.contains(';')
    && words.last().is_some_and(|word| is_listed_local_part(word));
  let opens_a_list = text[run.start..].starts_with(['{', '[']);
  let tail = label.is_none()
    && (opens_a_list
      || ((rest.ends_with('.') || gap.contains([';', ',']))
        && text[..label_start].ends_with(char::is_whitespace)
        && !listed_before))
    && !words.iter().all(|word| names_a_contact(word));
  // (a stop the cut strands after a list's separator is the addresses' own, and goes with them: `…, USA; . wen.lu@…`
  // once the token-level Kind step took `bruce.hrivnak@valpo.edu` out, 1503.09120 — review rev107c N2; a stop ending
  // the words, `…, Town.`, stays)
  let kept = match rest.strip_suffix('.').map(str::trim_end) {
    Some(before) if tail && before.ends_with([';', ',', ':']) => before.trim_end_matches(separator),
    _ => rest,
  };
  ((label.is_some() && bounded) || tail).then_some(AddressLineCut {
    start: kept.len(),
    label,
    addresses,
    address_start,
    role_tag: None,
  })
}

/// The byte length of the correspondence role phrase opening `label` (its leading spaces in it), if one opens it:
/// "Corresponding author", "Corresponding authors", "Corresponding author(s)", "Corresponding Author's", "Corresponding
/// to" — the annotation on the author FIDELITY_CHARTER §4 reads "Corresponding author" as ([`write_role_notes`]).
fn correspondence_phrase(label: &str) -> Option<usize> {
  let start = label.len() - label.trim_start().len();
  let rest = &label[start..];
  let first_end = rest
    .find(|c: char| !c.is_alphabetic())
    .unwrap_or(rest.len());
  if !rest[..first_end].eq_ignore_ascii_case("corresponding") {
    return None;
  }
  let gap = rest[first_end..].len() - rest[first_end..].trim_start().len();
  let second = &rest[first_end + gap..];
  if gap == 0 {
    return None;
  }
  let second_end = second
    .find(|c: char| !(c.is_alphabetic() || "'\u{2019}()".contains(c)))
    .unwrap_or(second.len());
  let word = second[..second_end].to_lowercase();
  let base = word
    .trim_end_matches("(s)")
    .trim_end_matches("'s")
    .trim_end_matches("\u{2019}s")
    .trim_end_matches("s");
  matches!(base, "author" | "to").then_some(start + first_end + gap + second_end)
}

/// FIDELITY_CHARTER §4: "Corresponding author" is an annotation on the author, not an e-mail's name — an e-mail contact
/// whose printed label opens with a correspondence role phrase ([`correspondence_phrase`]: `Corresponding author:
/// a@x`, `* Corresponding author, E-mail: …` (1711.00192), `$^{*}$ Corresponding\ to\ …` (2606.31381)), by whichever
/// route it was made, gives the phrase to a correspondence note of the creator holding it — the address's owner, the
/// block's own creator where the address went there — before it, as printed (R8), with the mark its line printed
/// ([`LINE_MARK`]); it keeps the label's rest ("E-mail: "), else its role's name. A creator holding a correspondence note
/// already keeps that one, and the e-mail its label as printed (review rev107c N4, the main session's Q6 ruling; rev107c2
/// finding 1).
fn write_role_notes(document: &mut Document) -> Result<()> {
  let contacts = document.findnodes("//ltx:creator/ltx:contact[@role='email'][@name]", None);
  let default_name = get_frontmatter_name(None, "ltx:contact", "email")?;
  for mut contact in contacts {
    let name = contact.get_attribute("name").unwrap_or_default();
    let Some(end) = correspondence_phrase(&name) else {
      continue;
    };
    let Some(creator) = contact.get_parent() else {
      continue;
    };
    let phrase = name[..end].split_whitespace().collect::<Vec<_>>().join(" ");
    // (a creator holding a correspondence note already keeps that one; where the note says another thing — a `\thanks{To
    // whom correspondence should be addressed.}` — the e-mail keeps its label as printed, so the phrase is not lost
    // (review rev107c2 finding 1, `email_label_correspondence_beside_thanks_note`); one this step made of the same
    // phrase holds it already — an author's two e-mails under REVTeX's `\email[Corresponding authors:]{a, b}`,
    // 2504.10237)
    let notes: Vec<String> = element_nodes(&creator)
      .iter()
      .filter(|child| {
        child.get_name() == "note"
          && child
            .get_attribute("class")
            .is_some_and(|class| class.contains("ltx_thanks_correspondence"))
      })
      .map(|note| note.get_content())
      .collect();
    let same_phrase = notes.iter().any(|note| {
      note
        .trim_start_matches(|c: char| !c.is_alphabetic())
        .trim()
        .eq_ignore_ascii_case(&phrase)
    });
    if !notes.is_empty() && !same_phrase {
      continue;
    }
    let remainder = name[end..]
      .trim_start_matches(|c: char| c.is_whitespace() || ",;:-\u{2013}\u{2014}".contains(c));
    match (remainder.chars().any(char::is_alphanumeric), &default_name) {
      (true, _) => document.set_attribute(&mut contact, "name", remainder)?,
      (false, Some(default)) => document.set_attribute(&mut contact, "name", default)?,
      (false, None) => {
        contact.remove_attribute("name").ok();
      },
    }
    if same_phrase {
      continue;
    }
    DebugFeature!(
      "frontmatter",
      "FRONT e-mail label: a correspondence note: {phrase}"
    );
    let mark = contact
      .get_attribute(LINE_MARK)
      .filter(|mark| !mark.trim().is_empty());
    let saved = document.get_node().clone();
    document.set_node(&creator);
    let mut attributes: HashMap<String, String> = HashMap::default();
    attributes.insert("role".to_string(), "thanks".to_string());
    attributes.insert(
      "class".to_string(),
      "ltx_note_frontmatter ltx_thanks_correspondence".to_string(),
    );
    if let Some(mark) = &mark {
      attributes.insert("mark".to_string(), mark.clone());
    }
    let mut note = document.open_element("ltx:note", Some(attributes), None)?;
    document.close_element("ltx:note")?;
    document.set_node(&saved);
    if let Some(mark) = &mark {
      let mut sup = Node::new("sup", None, document.get_document())?;
      if let Some(namespace) = note.get_namespace() {
        let _ = sup.set_namespace(&namespace);
      }
      sup.append_text(mark)?;
      note.add_child(&mut sup)?;
    }
    note.append_text(&phrase)?;
    note.unlink_node();
    contact.add_prev_sibling(&mut note)?;
    // (the note shows the mark the line printed: the address no more)
    contact.remove_attribute(LINE_MARK_SHOWN).ok();
  }
  Ok(())
}

/// The attribute of an e-mail the Kind step for an address line made ([`kind_address_lines`]): the printed marks leading
/// its address on the line (`∗∗`, `1`; several joined by commas), by which the Link step gives it, unspelled, to the
/// authors whose names answer them, else to the block ([`link_entry_addresses`]; user ruling R1, marks win).
const LINE_MARK: &str = "_linemark";

/// The attribute of an e-mail the Kind step for an address line made whose line printed a mark no name answers before
/// it ([`kind_address_lines`]): the mark, shown before the address once it is placed ([`show_line_marks`]).
const LINE_MARK_SHOWN: &str = "_linemarkshown";

/// The attribute (and `\lx@add@email` option) of an e-mail a Kind step for an address line made of a line printed before
/// every line its entry keeps ([`kind_address_lines`], `affiliations::kind_entry_addresses`): placed before the entry on
/// its owner, as it prints (user ruling R8; [`link_entry_addresses`], review rev107c N1).
pub(in crate::base_utilities) const LINE_FIRST: &str = "_linefirst";

/// The attribute of an e-mail the Kind step for an address line made of a `{…}@domain` list set over several lines
/// ([`kind_address_lines`]): the list's key (its entry's and line's), by which the Link step places the list's addresses
/// as the author block's reader places its one-line form — the author each spells; else a list of one address per
/// author that every spelled one agrees with, the author at its place (OXIDIZED_DESIGN_DIVERGENCES #805); else the
/// only author's; else by elimination (batch 77p r4, `owners_by_elimination`); else the block (FIDELITY_CHARTER §4, review rev107a finding 4:
/// [`link_entry_addresses`], `email_owners::list_owners`).
const LINE_LIST: &str = "_linelist";

/// The marks each address of a line's cut ([`AddressLineCut`]) goes with, as indices into the line's `marks`
/// ([`LineText::marks`]): the marks counted are the line's opening one and those the cut takes out (from its `start`).
/// Marks leading the addresses (the first counted mark before the first address): those after the address before it
/// and before it, else the previous address's — a mark opening the line leads all of it (`$**$ email: cx@…, c\_dee@…`,
/// gr-qc0207058), one before each address its own (`$^1$a@x, $^2$b@y`, 1706.04764, 2505.11901). Marks following them:
/// those after the address and before the next. Each address's place is its local part's in `text`, read on from the
/// addresses' start, past each word read.
fn address_marks(text: &str, marks: &[(usize, String)], cut: &AddressLineCut) -> Vec<Vec<usize>> {
  let mut places: Vec<usize> = Vec::new();
  let mut from = cut.address_start.min(text.len());
  for address in &cut.addresses {
    let local = address.split('@').next().unwrap_or_default();
    let at = text[from..].find(local).map_or(from, |at| from + at);
    places.push(at);
    from = at
      + text[at..]
        .find(|c: char| c.is_whitespace() || ",;".contains(c))
        .unwrap_or(text.len() - at)
        .max(local.len().min(text.len() - at));
  }
  let opening = marks
    .first()
    .filter(|(at, _)| !text[..*at].chars().any(char::is_alphanumeric))
    .map(|(at, _)| *at);
  let counted: Vec<usize> = (0..marks.len())
    .filter(|&i| Some(marks[i].0) == opening || marks[i].0 >= cut.start)
    .collect();
  let leading = counted
    .first()
    .zip(places.first())
    .is_some_and(|(&i, &first)| marks[i].0 < first);
  let mut led: Vec<Vec<usize>> = Vec::new();
  for (n, &place) in places.iter().enumerate() {
    let own: Vec<usize> = counted
      .iter()
      .copied()
      .filter(|&i| {
        let at = marks[i].0;
        if leading {
          at < place && (n == 0 || at > places[n - 1])
        } else {
          at > place && places.get(n + 1).is_none_or(|&next| at < next)
        }
      })
      .collect();
    let own = match led.last() {
      Some(previous) if leading && own.is_empty() => previous.clone(),
      _ => own,
    };
    led.push(own);
  }
  led
}

/// The keys of a printed mark as a reader sees it (labels key by what a reader sees, user ruling 2026-11-02): each run
/// of digits, of letters, or of one symbol — `∗∗` one key, `**` the same, every asterisk form one ([`one_asterisk`]) —,
/// commas, brackets, spaces and format characters parting them (`∗a),b)`: `*`, `a`, `b`).
fn printed_mark_keys(text: &str) -> Vec<String> {
  let mut keys: Vec<String> = Vec::new();
  let mut last: Option<char> = None;
  for c in one_asterisk(text).chars() {
    let class = if c.is_ascii_digit() {
      Some('0')
    } else if c.is_alphabetic() {
      Some('a')
    } else if is_printed_symbol(c) || is_mark_glyph(c) {
      Some(c)
    } else {
      None
    };
    match (class, keys.last_mut()) {
      (Some(class), Some(key)) if last == Some(class) => key.push(c),
      (Some(_), _) => keys.push(c.to_string()),
      (None, _) => {},
    }
    last = class;
  }
  keys
}

/// The marks each of `authors` shows or requests, keyed as printed ([`printed_mark_keys`]): its name's superscripts and
/// mark formulas, its notes' marks, and the marks its name requested (`_annotations`' labels, bare or `affiliation:`,
/// `altaffil:`, `institute:`, a symbol command read as it prints: [`unanswered_mark_text`]).
fn author_mark_keys(document: &mut Document, authors: &[Node]) -> Vec<Vec<String>> {
  authors
    .iter()
    .map(|author| {
      let mut keys: Vec<String> = Vec::new();
      for sup in document.findnodes("ltx:personname//ltx:sup", Some(author)) {
        keys.extend(printed_mark_keys(&sup.get_content()));
        // (an author reference mark's number, as its label keys it: `relocate::sup_marks`)
        if let Some(refmarks) = sup.get_attribute("_refmark") {
          keys.extend(refmarks.split(',').map(str::to_string));
        }
      }
      // (a mark the name still holds as a formula, not yet parsed: aastex's `\author{J. E. Beckman$^{1,2}$}`,
      // astro-ph0005055)
      for math in document.findnodes("ltx:personname//ltx:Math", Some(author)) {
        keys.extend(printed_mark_keys(&math.get_content()));
      }
      for note in document.findnodes("ltx:note[@mark]", Some(author)) {
        keys.extend(printed_mark_keys(
          &note.get_attribute("mark").unwrap_or_default(),
        ));
      }
      for label in author
        .get_attribute("_annotations")
        .unwrap_or_default()
        .split(',')
      {
        // (a mark request — a label, authblk's `\author[1]`, or an affiliation's —; not the author's place, `author:N`,
        // nor a binding's key, `fuzzy:…`)
        let key = match label.split_once(':') {
          None => label,
          Some(("affiliation" | "altaffil" | "institute", key)) => key,
          Some(_) => continue,
        };
        let key = label_key(key);
        keys.extend(printed_mark_keys(
          &unanswered_mark_text(&key).unwrap_or(key),
        ));
      }
      keys
    })
    .collect()
}

/// The marks each of `authors` prints glued to its name as text, not set as a superscript — `Jian Shi$^{a}$*`'s `*`, a
/// `\textasteriskcentered` or a ✉ after the name (1711.00192: 107c r2) —, keyed as printed ([`printed_mark_keys`]): the
/// mark glyphs ([`is_mark_glyph`]) ending the name's text, its superscripts and formulas aside. They answer a line that
/// opens with the same mark ([`answering_authors`] over them and [`author_mark_keys`]); they are no request of their
/// own, so a line no name answers is read as before.
fn glued_name_marks(document: &mut Document, authors: &[Node]) -> Vec<Vec<String>> {
  fn name_text(node: &Node, text: &mut String) {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::TextNode) => text.push_str(&child.get_content()),
        Some(NodeType::ElementNode) if child.get_name() == "text" => name_text(&child, text),
        _ => {},
      }
    }
  }
  authors
    .iter()
    .map(|author| {
      let mut text = String::new();
      if let Some(name) = document.findnode("ltx:personname", Some(author)) {
        name_text(&name, &mut text);
      }
      let tail: String = text
        .chars()
        .rev()
        .take_while(|c| is_mark_glyph(*c) || c.is_whitespace() || *c == ',')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
      printed_mark_keys(&tail)
    })
    .collect()
}

/// Each author's mark keys, in author order ([`author_mark_keys`], [`glued_name_marks`]).
type MarkKeys = Vec<Vec<String>>;

/// A mark's series, by its first character: a number, a letter, or a symbol (`*`, `†`, `$\star$`) — the series an
/// orphan line's mark shares with the sole author's own (`$^{b}$` beside her `$^{a}$`, FIDELITY_CHARTER "Sole author",
/// user 2026-11-05: still the block creator's; a `*` beside her `$^{a}$` hers). `None` for an empty mark.
fn mark_series(mark: &str) -> Option<u8> {
  let first = mark.trim().chars().next()?;
  Some(if first.is_ascii_digit() {
    0
  } else if first.is_alphabetic() {
    1
  } else {
    2
  })
}

/// An entry's line ([`entry_lines`]): its nodes, and the line break before it (none for the first).
type EntryLine = (Vec<Node>, Option<Node>);

/// The authors whose names answer a printed `mark` ([`answering_authors`]) by a mark they show or request (`marks`,
/// [`author_mark_keys`]) or one glued to the name as text (`glued`, [`glued_name_marks`]), in order.
fn answering_or_glued(marks: &[Vec<String>], glued: &[Vec<String>], mark: &str) -> Vec<usize> {
  let mut answering = answering_authors(marks, mark);
  answering.extend(answering_authors(glued, mark));
  answering.sort_unstable();
  answering.dedup();
  answering
}

/// The authors (their places in `marks`, [`author_mark_keys`]) whose names answer a printed `mark`: one of its keys.
fn answering_authors(marks: &[Vec<String>], mark: &str) -> Vec<usize> {
  let keys = printed_mark_keys(mark);
  (0..marks.len())
    .filter(|&a| keys.iter().any(|key| marks[a].contains(key)))
    .collect()
}

/// Shows before each e-mail the mark its address line printed and no name answers ([`LINE_MARK_SHOWN`]), once the Link
/// step placed it — on the block, or on the author it spells: authblk's `\affil[1]{Univ X\authorcr $^\dagger$Email:
/// zq@x.org, ab@x.org}` under names carrying no `†` shows `†zq@x.org` on the block (review rev106a S1: no printed mark
/// lost).
fn show_line_marks(document: &mut Document) -> Result<()> {
  for mut contact in document.findnodes(&s!("//ltx:contact[@{LINE_MARK_SHOWN}]"), None) {
    let mark = contact.get_attribute(LINE_MARK_SHOWN).unwrap_or_default();
    contact.remove_attribute(LINE_MARK_SHOWN).ok();
    let mut sup = Node::new("sup", None, document.get_document())?;
    if let Some(namespace) = contact.get_namespace() {
      let _ = sup.set_namespace(&namespace);
    }
    sup.append_text(&mark)?;
    match contact.get_first_child() {
      Some(mut first) => {
        first.add_prev_sibling(&mut sup)?;
      },
      None => {
        contact.add_child(&mut sup)?;
      },
    }
  }
  Ok(())
}

/// The label an address line printed before an address an author holds already ([`kind_address_lines`], OD #966): the
/// held e-mail contact takes it where it carries the role's own name — `Email corresponding author:` before
/// `da_salim@…`, which S. H. Shekh's `\email` holds (2605.16453) —, so no printed word goes with the copy dropped.
fn carry_printed_label(copy: &Node, mut held: Node, default_name: Option<&str>) {
  // (and the mark the line printed that no name answers, shown before it: [`show_line_marks`])
  if let Some(mark) = copy.get_attribute(LINE_MARK_SHOWN)
    && held.get_attribute(LINE_MARK_SHOWN).is_none()
  {
    let _ = held.set_attribute(LINE_MARK_SHOWN, &mark);
  }
  let Some(label) = copy.get_attribute("name") else {
    return;
  };
  let held_name = held.get_attribute("name");
  if Some(label.as_str()) != default_name
    && held_name
      .as_deref()
      .is_none_or(|name| Some(name) == default_name)
  {
    let _ = held.set_attribute("name", &label);
  }
}

/// Whether `node` prints text alone: every element under it a font wrapper (`ltx:text`) or an address's link (an
/// `ltx:ref` printing an `@`, `\url{{a,b}@x}`), so writing one of its addresses out ([`set_written_out`]) drops nothing
/// it prints — not an ORCID's `ltx:ref` (2308.02269's `\orcidID` glued to an address line, review rev100b B1), math,
/// a note.
fn prints_text_only(node: &Node) -> bool {
  element_nodes(node).iter().all(|child| {
    let address_link = child.get_name() == "ref" && child.get_content().contains('@');
    (child.get_name() == "text" || address_link) && prints_text_only(child)
  })
}

/// A copy of an e-mail contact printing several addresses made the contact of one, `address`: written in the font
/// wrapper the whole contact printed in (`<text font="typewriter">`, llncs's `\email`, 64zc), else as the contact's text.
fn set_written_out(contact: &Node, address: &str) {
  let mut node = contact.clone();
  loop {
    let children = node.get_child_nodes();
    let elements: Vec<&Node> = children
      .iter()
      .filter(|c| c.get_type() == Some(NodeType::ElementNode))
      .collect();
    let bare_text = children
      .iter()
      .any(|c| c.get_type() == Some(NodeType::TextNode) && !c.get_content().trim().is_empty());
    match elements.as_slice() {
      [only] if !bare_text => node = (*only).clone(),
      _ => break,
    }
  }
  let _ = node.set_content(address);
}

/// Whether a nameless author creator holds names the reader did not part: a contact of its, an e-mail, link or note
/// aside, whose printed words read as one name or more (`name_count`, its marks left out) — 1102.4967's IEEE
/// `Rizwan Ghaffar, Dimitris Toumpakaris$^\dagger$ and` beside `\IEEEauthorblockN{Jungwon Lee…}`. The block's own
/// creator (OD #159) holding the block's contacts, or an orphan line's (`Univ B, City`), holds none
/// ([`link_entry_addresses`]'s only-author step; user ruling 2026-11-05, "Sole author").
fn nameless_creator_holds_names(document: &mut Document) -> bool {
  document
    .findnodes(
      "//ltx:creator[@role='author'][not(ltx:personname)]/ltx:contact[not(@role='email') and not(@role='url') and not(@role='orcid')]",
      None,
    )
    .iter()
    .any(|contact| {
      let text: String = contact
        .get_content()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ",.-'\u{2019}&".contains(*c))
        .collect();
      // (a list left open, `… and`, ends at its last name)
      let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
      let text = text
        .strip_suffix(" and")
        .or_else(|| text.strip_suffix(" &"))
        .unwrap_or(&text)
        .trim_end_matches(',');
      name_count(text) > 0
    })
}

/// The author block's own creator (OD #159): the first nameless author creator after the named ones, else a new one
/// after the last of them.
pub(super) fn block_creator(document: &mut Document, authors: &[Node]) -> Result<Option<Node>> {
  if let Some(creator) = document
    .findnodes("//ltx:creator[@role='author'][not(ltx:personname)]", None)
    .into_iter()
    .next()
  {
    return Ok(Some(creator));
  }
  let Some(mut last) = authors.last().cloned() else {
    return Ok(None);
  };
  let Some(parent) = last.get_parent() else {
    return Ok(None);
  };
  let saved = document.get_node().clone();
  document.set_node(&parent);
  let mut attributes = HashMap::default();
  attributes.insert("role".to_string(), "author".to_string());
  let mut creator = document.open_element("ltx:creator", Some(attributes), None)?;
  document.close_element("ltx:creator")?;
  document.set_node(&saved);
  if last.add_next_sibling(&mut creator).is_err() {
    return Ok(None);
  }
  Ok(Some(creator))
}

/// FIDELITY_CHARTER §4, "ownership by evidence, never by bare position": an e-mail contact one address alone fills,
/// on an author it does not spell, goes to the one author it spells — an `\email` call placed by its `annotate=last`
/// after a one-call author list (AASTeX 5's `\author{Ann Able and Bob Baker}\email{able@x.org}`: 0911.3506
/// `bourouaine@mps.mpg.de` on Eckart Marsch, 1506.04035, 1203.1317, 0904.0560 — 92 of the 688 aas census papers); a copy
/// the author it spells holds already is dropped. An address that spells no one, or several, stays (`spelled_owner`).
/// Guard `perfect_kernel_batch81::an_email_call_goes_to_the_author_it_spells`.
fn respell_emails(document: &mut Document) -> Result<()> {
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  if authors.len() < 2 {
    return Ok(());
  }
  // (each name read as printed, a transliterated spelling counted as one — whom an address is moved to
  // (`NameKeys::as_printed`: `joerg.mueller@` Jörg Müller's, user ruling 2026-11-05) —; and as an address spells it —
  // base letters too (`NameKeys::of`) — which only keeps an address on the author it spells so: `javier.martinez@ugr.es`
  // spells Javier Martínez-López, its holder, by base letters, so it stays, though as printed it spells Carlos Javier
  // Navarro alone (brief 95c G6, 2410.00275); `muller-group@x.de` on Anna Smith's own `\email` stays, though by base
  // letters it spells Jörg Müller — no target is picked by them (brief 95c r2 B1, FIDELITY_CHARTER §4 #420) —, where
  // `mueller-group@x.de` moves to him, as its ASCII twin beside `Jorg Mueller` does (OXIDIZED_DESIGN_DIVERGENCES #970))
  let names: Vec<String> = authors
    .iter()
    .map(|author| {
      document
        .findnode("ltx:personname", Some(author))
        .map(|person| person.get_content())
        .unwrap_or_default()
    })
    .collect();
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::as_printed(&Tokens::new(ExplodeText!(name))))
    .collect();
  let base_keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(name))))
    .collect();
  let mut moves: Vec<(Node, usize)> = Vec::new();
  for (at, author) in authors.iter().enumerate() {
    for contact in document.findnodes("ltx:contact[@role='email']", Some(author)) {
      // (one a mark's label placed — a marked e-mail line — is no bare position: `*baker@x.org` under `A. Able*`; a fuzzy
      // label, a name, that placed it nowhere left it at the call's position: `\correspondingauthor{…}\email{…}`,
      // 2003.00016, 2003.11098, 2204.06937)
      if contact
        .get_attribute("_label")
        .is_some_and(|label| !label.is_empty() && !label.starts_with("fuzzy:"))
      {
        continue;
      }
      // (its one address, a lead word aside: `\email{Email: pdrew@utexas.edu}`, 2003.00016)
      let text = contact.get_content();
      if text.matches('@').count() != 1 {
        continue;
      }
      let Some(address) = text
        .split_whitespace()
        .find(|word| word.contains('@'))
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
      else {
        continue;
      };
      // (an e-mail its holder's own name spells as well as another's stays: a tie never moves an own call, `yang91@` on
      // Yang Zhang's own call beside Hengyun Yang, 1707.09730; `ricky_ang@` on Lay Kee Ang's, 2604.02636 — FIDELITY_CHARTER
      // §4 #420; a longer spelling does, `sanderson@` on Mark Anderson's call, review rev107f finding 3, `dongsu.kim@` on
      // Jang Soo Kim's, 0902.4311: `own_call_keeps`)
      if let Some(owner) = spelled_owner(&keys, address)
        && owner != at
        && !own_call_keeps(
          &base_keys[at],
          &keys[owner],
          address,
          at + 1 == authors.len(),
        )
      {
        moves.push((contact, owner));
      }
    }
  }
  for (contact, owner) in moves {
    let address = contact.get_content();
    let held = document
      .findnodes("ltx:contact[@role='email']", Some(&authors[owner]))
      .iter()
      .any(|c| c.get_content().trim() == address.trim());
    // (a copy the owner already holds is dropped, not kept: a paper repeating its author block for a supplement lands the
    // second block's `\email` calls on the last creator, 2307.07324, 1801.07635, 1810.03867, 2510.13035, 2001.07542,
    // 2608.31160 — so "printed under both" cannot be told from that here, and 2502.15891's real duplicate reads as the base
    // did: `email_own_call_printed_under_both_stays_on_both_red`, review of bundle int139b)
    if !held {
      let mut target = authors[owner].clone();
      document.append_clone(&mut target, vec![contact.clone()])?;
    }
    document.remove_node(contact);
  }
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
///
/// A coalesce never drops a printed word: when no named author is left to take a nameless creator's annotations (a
/// block of affiliation lines alone, IEEEtran's `\authorblockA{…}` with no `\authorblockN`, 1804.00021; a name
/// written inside `\IEEEmembership{…}`, 2203.14581), the first nameless creator that carries any stays with every one
/// of them, the block's own creator — Perl's `<ltx:personname/>` over the lines (OXIDIZED_DESIGN_DIVERGENCES #159's
/// block owner, #949). Any other printing child of a removed creator is reported (`Warning:unexpected:frontmatter`), never
/// silently lost.
fn coalesce_empty_creators(document: &mut Document) -> Result<()> {
  let creators = document.findnodes("//ltx:creator[@role='author']", None);
  let mut to_remove: Vec<Node> = Vec::new();
  let mut last_real: Option<Node> = None;
  // Annotations (contacts + notes) of leading empties (before any real author), held
  // until the first real one; and the first of those empties that carries any, which keeps them all when no real
  // one follows.
  let mut orphan_annotations: Vec<Node> = Vec::new();
  let mut orphan_holder: Option<Node> = None;
  for creator in creators {
    if creator_personname_empty(document, &creator) {
      let annotations = creator_annotations(&creator);
      if let Some(ref mut prev) = last_real {
        if !annotations.is_empty() {
          document.append_clone(prev, annotations)?;
        }
        to_remove.push(creator);
      } else if orphan_holder.is_none() && !annotations.is_empty() {
        // (kept for now: the holder of the leading empties' annotations, if no real author takes them)
        orphan_holder = Some(creator);
      } else {
        orphan_annotations.extend(annotations);
        to_remove.push(creator);
      }
    } else {
      if let Some(holder) = orphan_holder.take() {
        // (the holder's own annotations first, in the order they stood)
        let held = creator_annotations(&holder);
        orphan_annotations.splice(0..0, held);
        to_remove.push(holder);
      }
      if !orphan_annotations.is_empty() {
        let mut first = creator.clone();
        document.append_clone(&mut first, std::mem::take(&mut orphan_annotations))?;
      }
      last_real = Some(creator);
    }
  }
  // (no named author took the leading empties' annotations: the holder keeps them all, the block's creator)
  if let Some(mut holder) = orphan_holder
    && !orphan_annotations.is_empty()
  {
    document.append_clone(&mut holder, orphan_annotations)?;
  }
  for creator in to_remove {
    // (what else a removed creator prints, no annotation moved with it, is reported: a coalesce never drops a word
    // silently)
    for child in creator.get_child_nodes() {
      if child.get_type() != Some(NodeType::ElementNode) {
        continue;
      }
      let qname = document::get_node_qname(&child);
      if with(qname, |q| {
        !matches!(q, "ltx:personname" | "ltx:contact" | "ltx:note" | "ltx:rdf")
      }) && !child.get_content().trim().is_empty()
      {
        Warn!(
          "unexpected",
          "frontmatter",
          s!(
            "A nameless author's {} was dropped with it: {}",
            with(qname, |q| q.to_string()),
            child.get_content().trim()
          )
        );
      }
    }
    document.remove_node(creator);
  }
  Ok(())
}

/// A creator's annotations, the children a coalesce moves: its `<ltx:contact>`, `<ltx:note>` and `<ltx:rdf>` elements.
fn creator_annotations(creator: &Node) -> Vec<Node> {
  creator
    .get_child_nodes()
    .into_iter()
    .filter(|c| {
      c.get_type() == Some(NodeType::ElementNode)
        && with(document::get_node_qname(c), |q| {
          q == "ltx:contact" || q == "ltx:note" || q == "ltx:rdf"
        })
    })
    .collect()
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
  // (a call made after `\maketitle` placed the frontmatter ([`CONTACT_LATE`]) restating a contact the creator holds
  // from an earlier call — a supplement's restatement — is no contact of the pile: llncs's restated `\institute` put a
  // second `\email{libraboli@…}` beside the first on Bo Li, the last author, and the two read as a grid of one each,
  // handing the first to Lv Tang, who prints none (2009.10916: census138 EMAILSPELL, brief 107f);
  // `drop_repeated_contacts` drops the copy. Late calls of their own still are: 1303.3079's `\email`s in a footnote
  // after `\maketitle`)
  let printed = |x: &Node| {
    (
      x.get_attribute("role"),
      x.get_content()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" "),
    )
  };
  let contacts_of = |c: &Node| -> Vec<Node> {
    let all: Vec<Node> = c
      .get_child_nodes()
      .into_iter()
      .filter(|x| {
        x.get_type() == Some(NodeType::ElementNode)
          && with(document::get_node_qname(x), |q| q == "ltx:contact")
      })
      .collect();
    all
      .iter()
      .filter(|x| {
        !x.has_attribute(CONTACT_LATE)
          || !all
            .iter()
            .any(|y| !y.has_attribute(CONTACT_LATE) && printed(y) == printed(x))
      })
      .cloned()
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
/// (element order within it is canonical, not arrival order). A late abstract then
/// goes behind the cover printed before its marker, as in the eager build.
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
    for marker in document.findnodes(&s!("//{LATE_ABSTRACT_MARK}"), None) {
      document.remove_node(marker);
    }
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
  // A late abstract's marker (`\lx@frontmatter@mark`): the abstract, now placed, goes behind the cover printed before
  // it, as in the eager build (OXIDIZED_DESIGN_DIVERGENCES #903), each in turn; every such marker goes.
  let late_marks = document.findnodes(&s!("//{LATE_ABSTRACT_MARK}"), None);
  for marker in &late_marks {
    place_abstract_after_late_mark(document, marker)?;
  }
  for marker in late_marks {
    document.remove_node(marker);
  }
  Ok(())
}

/// Insert one frontmatter entry `[tag, {attr}, @content]`.
/// Perl: insertFrontMatter_rec($document, $item) for the ARRAY case.
pub(in crate::base_utilities) fn insert_frontmatter_entry(
  document: &mut Document,
  entry: &TagData,
) -> Result<()> {
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
  // (a contact's source entry, which every copy of it keeps — its call's, an annotation's, else its own here:
  // `drop_repeated_contacts`)
  if tag == "ltx:contact" && !attributes.contains_key(CONTACT_SOURCE) {
    let source = lookup_int(CONTACT_SOURCE_COUNTER) + 1;
    assign_value(
      CONTACT_SOURCE_COUNTER,
      Stored::Int(source),
      Some(Scope::Global),
    );
    attributes.insert(CONTACT_SOURCE.to_string(), s!("inserted:{source}"));
  }
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
///
/// Returns the marks it could not show ([`unanswered_mark_text`]: a composite label), for [`count_lost_marks`].
fn show_unanswered_marks(
  document: &mut Document,
  answered: &HashSet<String>,
) -> Result<Vec<String>> {
  let mut unshown: Vec<String> = Vec::new();
  for creator in document.findnodes(".//ltx:creator[@_annotations]", None) {
    // (what the author's entries answer and what its name shows, read where a request no entry answers asks: most
    // authors have none)
    let mut answered_here: Option<Vec<String>> = None;
    let mut shown: Option<(Vec<String>, Vec<String>)> = None;
    let text_marks: Vec<String> = creator
      .get_attribute("_textmarks")
      .unwrap_or_default()
      .split(',')
      .map(str::to_string)
      .collect();
    let picture_marks = creator.get_attribute("_picturemarks").unwrap_or_default();
    let plain_marks = creator.get_attribute("_plainmarks").unwrap_or_default();
    let mut marks: Vec<String> = Vec::new();
    // (a mark written before a symbol the name shows in a superscript of its own, the symbol's place among the labels and
    // the symbol: it prints there, `$^{1*}$`, as its requests were made in the order written, 2610.07087 — after the
    // name's last text, it read `∗1`)
    let mut before_symbol: Vec<(String, usize, String)> = Vec::new();
    let labels = creator.get_attribute("_annotations").unwrap_or_default();
    let labels: Vec<&str> = labels.split(',').collect();
    for (at, label) in labels.iter().copied().enumerate() {
      let Some(mark) = label.strip_prefix("affiliation:") else {
        continue;
      };
      if answered.contains(label) || answered.contains(mark) {
        continue;
      }
      // (a label only a picture prints is no mark of the name: the picture is dropped from it, and its text with it —
      // the TikZ ORCID icon's "iD", `\inst{1\myorcid{…}}`; review rev92b_r3 S1, as rounds 1-2 and Perl showed nothing.
      // Per request: a label the name also requests outside a picture is shown, `Ann Able$^{\circled{b}}$$^{b}$`'s b,
      // in either order — review rev92b_r4 B1, where the picture's hid it)
      if picture_marks.split(',').any(|m| m == label) && !plain_marks.split(',').any(|m| m == label)
      {
        continue;
      }
      // (an author reference mark prints its symbol, keyed by its number: `_refmarkshown`, the request's)
      let refmark = creator
        .get_attribute("_refmarkshown")
        .unwrap_or_default()
        .split(';')
        .find_map(|pair| {
          pair
            .split_once('=')
            .filter(|(key, _)| *key == label)
            .map(|(_, shown)| shown.to_string())
        });
      let Some(mut text) = refmark.or_else(|| unanswered_mark_text(mark)) else {
        unshown.push(mark.to_string());
        continue;
      };
      // (the glyph follows the mark's mode: a `*` requested in text is `*`, in math `∗`)
      if mark == "*" && text_marks.iter().any(|m| m == label) {
        text = "*".to_string();
      }
      // (a symbol mark nothing answers is shown too — 1807.01584, 1805.02699, 2203.01270 — except one that placed one of
      // this author's contacts and was dropped from it, `$^\dagger$baker@y.edu`: answered on this author only,
      // `answered_symbol_key`; one the author's email shows as well stays on the name too, as printed: 1910.00401)
      let answered_here = answered_here.get_or_insert_with(|| {
        document
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
          .collect()
      });
      if symbol_mark_key(mark).is_some_and(|key| answered_here.contains(&key)) {
        continue;
      }
      let (shown, shown_sups) = shown.get_or_insert_with(|| {
        let sups = document
          .findnodes("ltx:personname//ltx:sup", Some(&creator))
          .iter()
          .map(|sup| sup.get_content())
          .collect();
        (shown_marks(document, &creator), sups)
      });
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
      let symbol = labels[at + 1..]
        .iter()
        .enumerate()
        .find_map(|(after, later)| {
          let later = later.strip_prefix("affiliation:")?;
          let later = unanswered_mark_text(later).unwrap_or_else(|| later.to_string());
          (!later.chars().all(char::is_alphanumeric)
            && shown_sups.iter().any(|sup| unify(sup) == unify(&later)))
          .then_some((at + 1 + after, later))
        });
      match (symbol, before_symbol.last_mut()) {
        // (a list before one symbol prints as one, `$^{1,2*}$`)
        (Some((place, _)), Some((list, of, _))) if *of == place => {
          list.push(',');
          list.push_str(&text);
        },
        // (and one after an earlier symbol with the comma it prints after it, `1∗,2∗`)
        (Some((place, symbol)), before) => {
          let text = if before.is_some() {
            s!(",{text}")
          } else {
            text
          };
          before_symbol.push((text, place, symbol));
        },
        (None, _) => marks.push(text),
      }
    }
    if marks.is_empty() && before_symbol.is_empty() {
      continue;
    }
    let Some(mut personname) = document
      .findnodes("ltx:personname", Some(&creator))
      .into_iter()
      .next()
    else {
      continue;
    };
    let name_sups = document.findnodes("ltx:personname//ltx:sup", Some(&creator));
    let symbol_at = |place: usize, glyph: &str| {
      // (the same symbol again is the next of its superscripts, `$^{1*,2*}$`)
      labels[..place]
        .iter()
        .filter(|label| {
          label.strip_prefix("affiliation:").is_some_and(|label| {
            unanswered_mark_text(label)
              .unwrap_or_else(|| label.to_string())
              .replace('\u{2217}', "*")
              == glyph
          })
        })
        .count()
    };
    // (a run after a symbol prints the comma after it, `$^{1*,2,3}$` as `1∗,2,3`, 2609.14365)
    let after_symbol = !before_symbol.is_empty();
    for (text, place, symbol) in before_symbol {
      let glyph = symbol.replace('\u{2217}', "*");
      let mut sup = unanswered_sup(document, &personname, &text)?;
      match name_sups
        .iter()
        .filter(|node| node.get_content().replace('\u{2217}', "*") == glyph)
        .nth(symbol_at(place, &glyph))
      {
        Some(node) => {
          let mut node = node.clone();
          node.add_prev_sibling(&mut sup)?;
        },
        None => personname.add_child(&mut sup)?,
      }
    }
    if !marks.is_empty() {
      let text = marks.join(",");
      let text = if after_symbol { s!(",{text}") } else { text };
      let mut sup = unanswered_sup(document, &personname, &text)?;
      personname.add_child(&mut sup)?;
    }
  }
  Ok(unshown)
}

/// Count, once the observer's writer has written the names, each mark [`show_unanswered_marks`] could not show and nothing
/// shows since: a printed mark lost is never silent (`$^{a-b}$`, `$^{0.8}$`). Only a number or letter with a character no
/// mark is made of beside it: a run of symbols or a number glued to one (`$^{**}$`, `$^{*\dag}$`, `\inst{1*}`) is requested
/// or parted piece by piece, and a label keyed by a macro's spelling, or what prints between `\ref`s (`--`), says nothing
/// of what prints.
fn count_lost_marks(document: &mut Document, unshown: &[String]) {
  let composite = |mark: &&String| {
    !mark.starts_with('\\')
      && mark.chars().any(char::is_alphanumeric)
      && mark
        .chars()
        .any(|c| !c.is_alphanumeric() && !matches!(c, '*' | '#') && !is_printed_symbol(c))
  };
  let unshown: Vec<&String> = unshown.iter().filter(composite).collect();
  if unshown.is_empty() {
    return;
  }
  let shown = document
    .findnodes(".//ltx:creator/ltx:personname//ltx:sup", None)
    .iter()
    .map(|sup| sup.get_content())
    .collect::<String>();
  for mark in unshown {
    if !shown.contains(mark.as_str()) {
      Warn!(
        "frontmatter",
        "unanswered_mark_lost",
        s!(
          "The mark {} an author requests is answered by no entry and cannot be shown on the name",
          lost_mark_as_written(mark)
        )
      );
    }
  }
}

/// A lost mark for the reader of the log: quoted, and an ORCID iD its label reads run on to the number before it apart
/// from that number (`$^{1\orcidlink{0000-0001-8812-0565}}$`, 2205.05709: not the mark "10000-0001-…").
fn lost_mark_as_written(mark: &str) -> String {
  const ORCID_LENGTH: usize = "0000-0000-0000-0000".len();
  let parted = mark.len().checked_sub(ORCID_LENGTH).and_then(|at| {
    let orcid = mark.get(at..)?;
    let shape = orcid.char_indices().all(|(place, c)| match place {
      4 | 9 | 14 => c == '-',
      18 => c.is_ascii_digit() || c == 'X',
      _ => c.is_ascii_digit(),
    });
    shape.then(|| (&mark[..at], orcid))
  });
  match parted {
    Some(("", orcid)) => s!("of the ORCID iD {orcid} alone"),
    Some((number, orcid)) => s!("\"{number}\" (with the ORCID iD {orcid})"),
    None => s!("\"{mark}\""),
  }
}

/// The `ltx:sup` showing `text`, marks no entry answers ([`show_unanswered_marks`]), for `personname`; marked as shown
/// for no entry: the observer's writer takes from it the marks an item it writes answers (review rev94a r2 S3).
fn unanswered_sup(document: &Document, personname: &Node, text: &str) -> Result<Node> {
  let mut sup = Node::new("sup", None, document.get_document())?;
  if let Some(namespace) = personname.get_namespace() {
    let _ = sup.set_namespace(&namespace);
  }
  sup.append_text(text)?;
  sup.set_attribute("_unanswered", "true")?;
  Ok(sup)
}

/// Add `label` (comma separated) to the requests of the frontmatter entry being digested (`_annotations`), and, for a
/// superscript requested in text (`\textsuperscript{1,*}`, not `$^{1,*}$`: `superscript`), to its text-mode marks
/// (`_textmarks`): such a `*` prints as `*`, a math one as `∗` (`show_unanswered_marks`). A mark command (`\inst`,
/// `\IEEEauthorrefmark`) sets its mark in math (llncs.cls `\def\inst#1{\unskip$^{#1}$}`), whatever mode it is read in.
/// The labels only a picture prints (`pictured`, [`frontmatter_label_request`]) are its picture marks
/// (`_picturemarks`), the others its plain marks (`_plainmarks`): a picture mark the entry never requests plainly is
/// never shown unanswered.
pub(in crate::base_utilities) fn add_annotation_labels(
  label: String,
  superscript: bool,
  pictured: Vec<String>,
) {
  let in_text = superscript && !lookup_bool("IN_MATH");
  with_pending_entry_attr(move |attr| {
    let mut append = |key: &str, new: &mut dyn Iterator<Item = &str>| {
      let mut marks = attr.get(key).cloned().unwrap_or_default();
      let before = marks.len();
      for mark in new {
        if !marks.is_empty() {
          marks.push(',');
        }
        marks.push_str(mark);
      }
      if marks.len() != before {
        attr.insert(key.to_string(), marks);
      }
    };
    append("_picturemarks", &mut pictured.iter().map(String::as_str));
    append(
      "_plainmarks",
      &mut label
        .split(',')
        .filter(|key| !key.is_empty() && !pictured.iter().any(|p| p == key)),
    );
    let labels = attr.get("_annotations").cloned().unwrap_or_default();
    // (an annotation being digested — a contact's stub, `role=pending` — whose label is the mark its content requests
    // again is that one label: the relocation reads a stub's labels as one key, so `affiliation:1,affiliation:1` was no
    // creator's, an orphan on the nameless block creator — a rerouted `\affil{$^1$…}` given `label={affiliation:1}`
    // (frontmatter_stores.rs `contact_line_call`) digested under the author markup, pasa.cls:33, 2005.09266)
    let pending = attr.get("role").map(String::as_str) == Some("pending");
    let fresh: Vec<&str> = label
      .split(',')
      .filter(|piece| !(pending && labels.split(',').any(|known| known == *piece)))
      .collect();
    let label = fresh.join(",");
    let newval = if labels.is_empty() {
      label.clone()
    } else if label.is_empty() {
      labels
    } else {
      s!("{labels},{label}")
    };
    DebugFeature!("frontmatter", "FRONT add annotation label {label}");
    attr.insert("_annotations".to_string(), newval);
    if in_text && !label.is_empty() {
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
/// which labels affiliations as a mark, `is_symbol_mark`) as its glyph — a label keyed by its printed text is that glyph
/// already (`\inst{1,\star}`'s `⋆`, 2105.03212; refactor stage 4, OXIDIZED_DESIGN_DIVERGENCES #919), one keyed by its
/// spelling (the fallback) keeps the control sequence; `None` for anything else (a fuzzy or composite label).
pub(in crate::base_utilities) fn unanswered_mark_text(mark: &str) -> Option<String> {
  // (its characters, each combining mark part of the one before it: `\={x}`'s `x̄` is a letter — review rev92b_r2 B1)
  let bases: Vec<char> = mark.chars().filter(|c| !is_combining_mark(*c)).collect();
  let marked = !mark.starts_with(is_combining_mark);
  if marked && !bases.is_empty() && bases.iter().all(|c| c.is_alphanumeric()) {
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
    // (one printed symbol, `queue_digest::is_printed_symbol` — no letter, digit, space, ASCII, format character or
    // combining mark: `\up{\!1,2}`'s zero-width space is none, 2209.14855 —, with the marks on it; a run of several is
    // a whole-run label, its symbols requested apart, `parted_printed_label`)
    _ if marked && matches!(bases.as_slice(), [c] if is_printed_symbol(*c)) => mark,
    _ => return None,
  };
  Some(glyph.to_string())
}

/// What a frontmatter annotation says: its text, or an `ltx:rdf`'s `content`/`resource` (OXIDIZED_DESIGN #479), which
/// has no text.
pub(in crate::base_utilities) fn annotation_value(note: &Node) -> String {
  if note.get_name() == "rdf" {
    note
      .get_attribute("content")
      .or_else(|| note.get_attribute("resource"))
      .unwrap_or_else(|| note.get_content())
  } else {
    note.get_content()
  }
}
