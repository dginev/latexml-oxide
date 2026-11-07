//! The `\lx@add@authors` machinery: `add_authors_calls` and its sectioned form, which turn one `\author` block into
//! `\lx@add@author` calls (lifted from the `LoadDefinitions!` body).

use super::*;

/// The `\lx@add@authors` machinery (author-line parsing: `\and`/`and`
/// splitting, `$^{1,}$` affiliation marks, tabular/minipage lines, ijcai
/// `\affiliations`) as a function, so both the replacing kernel `\author`
/// and the appending raw-class mode share it.
pub(in crate::base_utilities) fn add_authors_calls(stuff: Tokens, replace: bool) -> Result<Tokens> {
  add_authors_calls_sectioned(stuff, replace, true, None, false)
}
/// [`add_authors_calls`]; `sectioned` false for the names an IJCAI split already took out, which
/// go round no more (Perl's `\lx@add@authors` has no marker branch, ijcai.sty.ltxml:40), and for a per-author
/// class's `\author`, which Perl adds whole (its names re-entering as `\lx@ijcai@names` would replace the authors
/// before it). `keyvals` are handed to every `\lx@add@author` the block makes (a per-author class's
/// `\author[labels]{…}`: revtex `annotations=`); `cautious` for a per-author class binding, whose names lines split
/// only where they read as names ([`split_author_line_cautious`]).
pub(in crate::base_utilities) fn add_authors_calls_sectioned(
  stuff: Tokens,
  replace: bool,
  sectioned: bool,
  keyvals: Option<Tokens>,
  cautious: bool,
) -> Result<Tokens> {
  // (a `\let` alias of a mark command is the command: `\let\up\textsuperscript`, 2209.14855; [`unalias_mark_commands`])
  let stuff = unalias_mark_commands(expand_separator_macros(expand_sole_author_macro(stuff)));
  // (a note written inside a mark is taken out after it, the mark's labels read without it: `\textsuperscript{\rm 1,\rm
  // 2\thanks{…}}`, 2512.05464; [`notes_out_of_marks`])
  let stuff = notes_out_of_marks(stuff);
  // (an empty mark prints nothing and is no mark: `\textsuperscript{}Univ. Grenoble Alpes…` leaves the block unmarked,
  // its line every name's; [`drop_empty_marks`])
  let stuff = drop_empty_marks(stuff);
  // (a mark opening a line that prints no label, `\\ $^{\;}$Ontario N2L 3G1, Canada`, is none: the line continues the
  // entry above it, 2303.08793; [`blank_line_marks_dropped`])
  let stuff = blank_line_marks_dropped(stuff);
  // (a line of notes only after a `\\` joins the line above, its notes the names': `… and Cat Cole$^{1}$\\
  // \thanks{$^{1}$…}`, 2609.17106, 2411.05212, 2312.10807; [`notes_line_joins_names`], OXIDIZED_DESIGN_DIVERGENCES
  // #957) — its notes no name requests the document's, added after the block's calls
  let (stuff, document_notes) = notes_line_joins_names(stuff);
  // (the notes an address line carries, the document's, added after those, in print order: [`address_line_notes`],
  // 2605.23553)
  let mut address_notes: Vec<Token> = Vec::new();
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
  // affiliations / emails, each email placed as a class's address list is (OD #908). This runs
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
    out.extend(document_notes);
    return Ok(Tokens::new(out));
  }
  let mut calls: Vec<Token> = Vec::new();
  // A per-author class binding's `\author` (aastex, amsart, revtex, an IEEE block, a labelled authblk `\author`),
  // which Perl reads as one author, splits its names only where they read as names (63e); a raw class's own
  // `\author` (`\lx@add@authors@adaptive`, 2309.03769's dan2e.sty) splits as the article `\author` does.
  // A collaboration or team the line names beside its names is read out of them, for the contact it makes on the names
  // it covers (`collaboration_split`, [`Collaborations`]; FIDELITY_CHARTER §4, user ruling 2026-11-01): `… and the GAPS
  // Team` (1708.04166), `… J.~Zu$^{68,55}$ (BESIII Collaboration)` (2212.06489).
  let split_names = |line: Tokens| {
    if let Some(pieces) = collaboration_split(&line, cautious) {
      pieces
    } else if cautious {
      split_author_line_cautious(line)
    } else {
      split_author_line(line)
    }
  };
  // (the rows of more names a names line goes on with, split as an unmarked names line is)
  let split_more =
    |line: Tokens| collaboration_split(&line, false).unwrap_or_else(|| split_author_line(line));
  // LaTeX-faithful `\author` REPLACES (the last call wins); a raw class that
  // redefined `\author` to call it once per author appends instead (56fl).
  if replace {
    replace_queued_authors();
  }
  // Consume any `\\[len]` / `\\*[len]` row-break optionals up front so the line
  // splits below see a bare `\\` (KNOWN_PERL_ERRORS #75, witness 2605.23553). This
  // also runs ahead of the tabular/minipage fallback: that fallback is a
  // last-resort escape hatch for author markup we cannot structure, and its
  // `<tabular>`-in-`<personname>` output is a presentational artifact we do not
  // want in frontmatter anyway, so there is nothing to preserve by skipping it. A positive length between lines is
  // kept as the gap it prints, `\\ \\` ([`strip_linebreak_options_keeping_gaps`]: gr-qc9906006) — not in that
  // fallback, whose content keeps its line breaks as written (1901.06146's `\\*[10pt]`, one break; the 85a r3 review's
  // finding 4).
  let lines_prepared = |stuff: Tokens| {
    let stuff = note_environments_as_commands(stuff);
    let stuff = math_active_scripts(stuff);
    let stuff = tabular_juggles_as_line_breaks(stuff);
    let stuff = unbrace_acm_author_columns(stuff);
    let stuff = unwrap_alignment_environment(stuff);
    let stuff = trailing_alignment_environment_as_lines(stuff);
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
    let stuff = glued_line_marks_as_marks(stuff, false);
    let stuff = rewrite_symbol_superscripts(stuff);
    let stuff = rewrite_ordinal_superscripts(stuff);
    rewrite_in_word_superscripts(stuff)
  };
  let written = stuff;
  let stuff = lines_prepared(strip_linebreak_options_keeping_gaps(written.clone()));
  // If too much formatting, fall back to unstructured author content
  let stuff_string = stuff.to_string();
  // Perl Base_Utility.pool.ltxml:693 tests `{tabular}`, `{minipage}` and `\halign`; a `\\` inside any alignment ends
  // its row, not an author line, so the whole family keeps the author content whole: a `tabular*`/`tabularx` author
  // block was split at its row ends, leaving the alignment open across the pieces (2609.34061, 34965, 39374, 39909;
  // KNOWN_PERL_ERRORS #502).
  let kept_whole = [
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
  .any(|form| stuff_string.contains(form));
  // (a wrapper around the block's lines is each line's, the lines read as written at the top level —
  // [`wrapper_over_lines_as_lines`], 2311.03613, 2306.04870, 1202.2048, 2306.04988 —; not in a block kept whole)
  let stuff = if kept_whole {
    stuff
  } else {
    wrapper_over_lines_as_lines(stuff)
  };
  if kept_whole {
    let stuff = lines_prepared(strip_linebreak_options(written));
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
    // (the entries of unmarked institution lines under the names, each with how many names its row above holds)
    let mut shared_institutions: Vec<(usize, usize)> = Vec::new();
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
    // (and a collaboration's name a line break parts after "and" one line again, [`joined_collaboration`]: 1210.5341)
    let group_lines = |group: Tokens| -> Vec<(Vec<Token>, Tokens)> {
      let mut lines: Vec<(Vec<Token>, Tokens)> = Vec::new();
      for (delimiter, line) in split_tokens_delimited(group, author_affil_splits())
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
      {
        if let Some((_, last)) = lines.last_mut()
          && let Some(joined) = joined_collaboration(last, &line)
        {
          *last = joined;
          continue;
        }
        lines.push((delimiter, line));
      }
      lines
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
      let lines = group_lines(group);
      for (at, (delimiter, line)) in lines.iter().cloned().enumerate() {
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
            // …and so is one carrying notes, its notes the document's ([`address_line_notes`]: 2605.23553's
            // `\texttt{…}\thanks{This manuscript has been submitted …}`, read whole an affiliation of the last name)
            let line = match address_line_notes(&line, |rest| {
              Ok(line_is_email_list(rest) || labelled_email_line(rest)?.is_some())
            })? {
              Some((rest, notes)) => {
                address_notes.extend(notes);
                rest
              },
              None => line,
            };
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
              && (last_names
                .as_ref()
                .is_some_and(|names| names_list_continues(names, &line))
                // (a line of names horizontal space alone parts is more names, however styled: `Ann Able$^{1}$ \quad
                // Cat Cole \\ \textbf{Bob Baker \quad Dan Dee}`, 2507.19054's row — welded, Cat Cole held them,
                // [`space_parted_name_line`])
                || space_parted_name_line(&line))
            {
              // An unfinished name list goes on past the break, as in an unmarked block ([`names_continue`]):
              // `Gus Gray\IEEEauthorrefmark{1}, …, Ida Ivy,~\IEEEmembership{Fellow,~IEEE,}\\ Jon Jay, … and~Kim
              // Key` (2408.02464) is five authors, not Ida Ivy's name welded to the next line's.
              last_names = Some(line.clone());
              for author in split_more(line) {
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
              // (a line naming a collaboration is no part of the entry above it: the collaboration of the names above
              // it, a contact on them — `\\ (XYZ Collaboration)\\` under the names, 2410.11607; `\\ on behalf of the
              // LHCb collaboration`, 1905.11825; `\\[1em] -- QCDSF-UKQCD Collaboration --`, hep-ph0502212 — not
              // welded into the last name, Base_Utility.pool.ltxml:701; [`Collaborations`]) — unless it names one unframed
              // and its address goes on under it, an employer's ([`collaboration_line_before`], 2501.13770)
              if collaboration_line_before(&line, lines.get(at + 1).map(|(_, next)| next)) {
                entries.push((AuthorLineKind::Collaboration, line));
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
              // (the marks the block's authors request, each one: a parenthesized lead labels by them,
              // `parenthesized_mark_line`; a mark written in parentheses, `$^{(a)}$`, requests `a`, the rev77t review's
              // A2)
              let requested: Vec<String> = entries
                .iter()
                .filter(|(kind, _)| *kind == AuthorLineKind::Author)
                .flat_map(|(_, author)| author_mark_operands(author.unlist_ref()))
                .flat_map(|operand| {
                  // (each mark of a list stripped of the parentheses at its ends, whether they wrap the list or
                  // each mark: `$^{(a,b)}$`, `$^{(a),(b)}$`, the rev77t2 re-review's finding 5)
                  Tokens::new(operand)
                    .to_string()
                    .split(',')
                    .map(|mark| mark.trim().trim_matches(['(', ')']).trim().to_string())
                    .collect::<Vec<_>>()
                })
                .collect();
              let labelled = symbol_labelled_affiliation(&line, &carried)
                .or_else(|| parenthesized_mark_line(&line, &requested));
              // (a link printing an institution too: placed as a link line is, an affiliation, [`url_contact_call`])
              if !finishes_an_address && is_link_line(&line) {
                entries.push((AuthorLineKind::Link, line));
                continue;
              }
              if !finishes_an_address
                && (labelled.is_some()
                  || (entries[continued].0 == AuthorLineKind::Author && is_legend_line(&line))
                  // (an unmarked line reading as an institution under a line of marked names is no part of the
                  // last name: the block's, as such a line under marked affiliations is — user ruling R12
                  // 2026-10-20 — where the marks label no line, `Ann Able$^{\dag}$ \and Bob Baker\\ Univ A` and
                  // `Ann Able$^{1}$, Bob Baker$^{2}$\\ Univ A` gave "Bob BakerUniv A"; a text `\dag` mark is an
                  // affiliation mark, `is_symbol_mark`, so `\dag` blocks are marked ones)
                  || (entries[continued].0 == AuthorLineKind::Author
                    && unmarked_institution_line(&line))
                  || under_email
                  || entries[continued].0 == AuthorLineKind::Link
                  || entries[continued].0 == AuthorLineKind::Collaboration)
              {
                if labelled.is_none()
                  && entries[continued].0 == AuthorLineKind::Author
                  && unmarked_institution_line(&line)
                {
                  // (the names row directly above it: the run of names its line ends, not every name of the `\and`
                  // column — `Ann Able$^{\dag}$\\ University A\\ Bob Baker$^{\ddag}$\\ University B`, the rev77t
                  // review's r1)
                  let row = entries[group_start..=continued]
                    .iter()
                    .rev()
                    .take_while(|(kind, _)| *kind == AuthorLineKind::Author)
                    .count();
                  shared_institutions.push((entries.len(), row));
                }
                entries.push((AuthorLineKind::Affiliation, labelled.unwrap_or(line)));
                continue;
              }
              // (a names row under an unmarked institution line is more authors, the line no part of it: `…\\
              // University of Somewhere\\ Cat Cole, Dan Dee\\ Other Institute`, the rev77t review's r7, where the row
              // went into the shared line and two authors were lost — only over an institution line of its own: a
              // place or a note under the institution, `Hong Kong SAR`, `Equal Contribution`, stays the institution's,
              // the rev77t2 re-review's s1-s4)
              if !finishes_an_address
                && shared_institutions
                  .last()
                  .is_some_and(|(shared, _)| *shared == continued)
                && name_count(&visible_name_text(line.unlist_ref())) >= 1
                && block_lines
                  .get(position)
                  .is_some_and(unmarked_institution_line)
              {
                last_names = Some(line.clone());
                for author in split_more(line) {
                  entries.push((AuthorLineKind::Author, author));
                }
                continue;
              }
              // (a line of names that horizontal space alone parts from the name before it, on that name's printed
              // line, is more names, as pdflatex sets them apart: `Matej Ze\v{c}evi\'c\textsuperscript{\rm $\dagger$}
              // \quad Moritz Willig \quad Jonas Seng \quad Florian P.\ Busch\textsuperscript{+}`, 2212.12575;
              // `\textbf{Ruiyao Xu}$^{\star}$ \quad \textbf{Mihir Parmar}$^{\diamond}$ \quad …`, 2604.17501;
              // `\textbf{Senthooran Rajamanoharan}\quad\quad \textbf{Neel Nanda}`, 2411.14257 — welded, one creator
              // held them; OXIDIZED_DESIGN_DIVERGENCES #897)
              if !finishes_an_address
                && continued + 1 == entries.len()
                && entries[continued].0 == AuthorLineKind::Author
                && space_parts_names(&delimiter, &entries[continued].1, &line)
              {
                last_names = Some(line.clone());
                for author in split_more(line) {
                  entries.push((AuthorLineKind::Author, author));
                }
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
                for author in split_more(line) {
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
                  clean_frontmatter_labels(&Tokens::new(operand.clone()).to_string(), "affiliation")
                    .iter()
                    .any(|label| !requested.contains(label))
                })
            };
            // (a link line among them, as when it read as an affiliation)
            let affiliations_begun = entries
              .iter()
              .any(|(kind, _)| matches!(kind, AuthorLineKind::Affiliation | AuthorLineKind::Link));
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
    // The collaborations read out of the names ([`collaboration_split`]) are contacts, no authors; so is a line of the
    // call that names one where an author stands, a heading over the names (`\center BESIII Collaboration\\~\\
    // M.~Ablikim$^{1}$, …`, 2212.06489), while the call holds a name besides ([`Collaborations`]).
    for (kind, line) in entries.iter_mut() {
      if *kind == AuthorLineKind::Author
        && let Some((text, _)) = unwrapped_collaboration(line)
      {
        *kind = AuthorLineKind::Collaboration;
        *line = text;
      }
    }
    let headings: Vec<usize> = entries
      .iter()
      .enumerate()
      .filter(|(_, (kind, line))| *kind == AuthorLineKind::Author && collaboration_line(line))
      .map(|(at, _)| at)
      .collect();
    if entries
      .iter()
      .filter(|(kind, _)| *kind == AuthorLineKind::Author)
      .count()
      > headings.len()
    {
      for at in headings {
        entries[at].0 = AuthorLineKind::Collaboration;
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
      .iter()
      .cloned()
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
          LinkOwner::Block if link_prints_an_institution(&entries[index].1) => 0..made(kinds.len()),
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
    // (the call's one name, making its creator itself: its addresses are its own, by a label of the creator it makes,
    // placed in the order the lines print — the `author:N` label they would go by is the empty creator's around it,
    // 1508.03930)
    let sole_own_creator = own_creator.len() == 1 && own_creator[0];
    let mut placed: Vec<Option<String>> = Vec::new();
    let mut sole_label = if sole_own_creator {
      Some(placed_label(&mut placed, 0)?)
    } else {
      None
    };
    let mut placed_calls: Vec<Vec<Token>> = vec![Vec::new(); made(kinds.len())];
    for (_, authors) in &link_authors {
      for author in authors.clone() {
        if let (_, Some(call)) = placed_label(&mut placed, author)? {
          placed_calls[author].extend(call);
        }
      }
    }
    let mut author_at = 0;
    let mut collaborations = Collaborations::default();
    for (index, (kind, mut line, email)) in entries.into_iter().enumerate() {
      if group_starts.contains(&index) {
        collaborations.close_group();
      }
      match kind {
        AuthorLineKind::Collaboration => calls.extend(collaborations.read(line)?),
        AuthorLineKind::Author => {
          correspondence_mark = None;
          // (a name making its creator itself, PoS's `\speaker{Lukas Nellen}$^a$`, takes the marks after it in that
          // creator's content, which they label: the speaker's affiliation is his, no nameless creator's — 1508.03930,
          // [`marks_into_own_creator`])
          let line = if own_creator.get(author_at).copied().unwrap_or(false) {
            let extra = sole_label
              .as_mut()
              .and_then(|(_, call)| call.take())
              .unwrap_or_default();
            marks_into_own_creator(line, extra)?
          } else {
            line
          };
          // (the headings over it, its collaborations, after its words and before the notes it carries, as they print
          // above it, [`heading_place`]; after the name when the name makes its creator itself, whose content they
          // then are)
          let headed = collaborations.author()?;
          let own = own_creator.get(author_at).copied().unwrap_or(false);
          let line = if own || headed.is_empty() {
            line
          } else {
            let v = line.unlist_ref();
            let at = heading_place(v);
            let mut headed_line = v[..at].to_vec();
            headed_line.extend(headed.iter().copied());
            headed_line.extend_from_slice(&v[at..]);
            Tokens::new(headed_line)
          };
          let mut content = Invocation!(T_CS!("\\lx@author@withsup"), vec![Some(line)]).unlist();
          if own {
            content.extend(headed);
          }
          content.extend(std::mem::take(&mut placed_calls[author_at]));
          author_at += 1;
          if let Some(email) = email {
            content.extend(Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(email)]).unlist());
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
          // (an unmarked institution line under the names: each author's of the names row directly above it, as
          // the row prints it under them — of its `\and` column at most, the whole block's when it has no `\and`,
          // R12)
          let shared = shared_institutions
            .iter()
            .find(|(shared, _)| *shared == index)
            .map(|(_, row)| *row)
            .filter(|authors| *authors > 1)
            .map(|authors| {
              mouth::tokenize_internal(TeXString::assembled(s!("annotate={authors}")))
            });
          calls.extend(
            Invocation!(T_CS!("\\lx@add@affiliation"), vec![shared, Some(withsup)]).unlist(),
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
          // (a call's one name making its creator itself has its addresses, as an unmarked block gives a name's lines
          // to it: the `author:N` label they would go by is the empty creator's around it, 1508.03930)
          if let Some((placement, _)) = &sole_label
            && let Some((label, addresses)) = labelled_emails(&line)?
          {
            calls.extend(email_calls(
              label.as_ref(),
              addresses,
              Some(&s!("label={placement}")),
            )?);
          } else if let Some((label, addresses)) = labelled_emails(&line)? {
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
        AuthorLineKind::Email if sole_label.is_some() => {
          let placement = sole_label
            .as_ref()
            .map(|(label, _)| label.as_str())
            .unwrap_or_default();
          let keyvals = mouth::tokenize_internal(TeXString::assembled(s!("label={placement}")));
          calls
            .extend(Invocation!(T_CS!("\\lx@add@email"), vec![Some(keyvals), Some(line)]).unlist());
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
              let owners = owners_by_elimination(&author_keys, &addresses, &taken, elsewhere_open);
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
    calls.extend(collaborations.finish()?);
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
    // (a line of names horizontal space alone parts is the names' row, not a line under them:
    // [`space_parted_rows_joined`])
    let stuff = space_parted_rows_joined(stuff);
    // Each block's groups, with whether only horizontal space (`\quad`, `\qquad`, the `\hspace`/`\hfill` and spacing
    // runs `normalize_hspace_separators` makes `\quad`) parts it from the block before.
    let mut blocks: Vec<(bool, Vec<Group>)> = Vec::new();
    // (the block's link lines no names stand over: the whole block's URL contacts, after its names)
    let mut block_links: Vec<Tokens> = Vec::new();
    // (the lines under such links, the block's affiliations)
    let mut block_lines: Vec<Tokens> = Vec::new();
    let mut spaced = true;
    // (the names line of the last group pushed, as written: the `\hspace`-before-`\\` branch below reads the lines
    // after it with it)
    let mut last_names_line: Option<Tokens> = None;
    // (the groups read as a collaboration's heading, waiting for the names they are over; and whether a group named
    // anyone)
    let mut headings: Vec<Tokens> = Vec::new();
    let mut named_any = false;
    // (the addresses and the nameless pieces no name came before, for the first names after them: [`address_only_line`],
    // [`names_no_one`])
    let mut leading_addresses: Vec<Tokens> = Vec::new();
    let mut leading_nameless: Vec<Token> = Vec::new();
    let pieces_of_block = split_tokens_delimited(stuff, author_group_splits());
    // (by piece, whether horizontal space parts it from the next, for a line of names after a row's addresses, below)
    let spaced_after: Vec<bool> = pieces_of_block
      .iter()
      .skip(1)
      .map(|(delimiter, _)| {
        delimiter
          .first()
          .is_some_and(|d| *d == T_CS!("\\quad") || *d == T_CS!("\\qquad"))
      })
      .chain(std::iter::once(false))
      .collect();
    for ((delimiter, block), row_goes_on) in pieces_of_block.into_iter().zip(spaced_after) {
      spaced &= delimiter
        .first()
        .is_some_and(|d| *d == T_CS!("\\quad") || *d == T_CS!("\\qquad"));
      let block = trailing_alignment_environment_as_lines(unwrap_alignment_environment(block));
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
      // (whether the block opens with a line that prints nothing, as the loop below reads one: a leading spacer line
      // too, `\and ~\\ University of X`, the rev98b review's pb1)
      let opens_with_break = pieces.len() > 1 && pieces.first().is_some_and(spacer_line);
      // (and by line, whether a line that prints nothing stood before it, a gap between the lines above and the next
      // author's: `\\ \vspace{0.1cm}\\`, `\\ \\`, [`name_groups`])
      // (a block a line break opens: whether a gap stands before its first line too, two empty pieces, `Ann Able
      // \hspace{.2cm} \\[5pt] Univ A`)
      let mut gaps: Vec<bool> = Vec::new();
      let mut lines: Vec<Tokens> = Vec::new();
      let mut gap = false;
      let mut leading_empty = 0usize;
      for piece in pieces {
        // (a line that prints nothing, in any position — a tie, a control space, an empty group: `~\\ Ann Able \\ …`,
        // 0901.1754, hep-lat0105010, math0306336; `{}\\[0mm] Meijiao Guan\\ …`, 0812.2273; `{}~\\~\\ \large{\sc Jun
        // Nishimura}…`, hep-lat9207028; `\ \\ Ann Able …`, solv-int9710025 — is the block's spacing, no line,
        // [`spacer_line`])
        if spacer_line(&piece) {
          gap = !lines.is_empty();
          leading_empty += usize::from(lines.is_empty());
          continue;
        }
        for line in link_run_lines(piece) {
          gaps.push(std::mem::take(&mut gap));
          lines.push(line);
        }
      }
      // A piece that prints no name is no person (as a marked line of addresses is none, rev97a S1, OD #944): a piece
      // with nothing to read, an `\orcidlink` or a graphic after the name (`Ann Able\quad\orcidlink{…}`), is the name's;
      // one of addresses alone is lines under the names before it, the row's list when it goes on with a line of them,
      // `Xinyu Wu \quad Jiansong Deng\\ \small \texttt{xinyu97@…}\quad \texttt{dengjs@…}` (2609.02424, where the second
      // address was a creator), `Ann Able\quad ann@x.org`, `\quad(ann@x.org)`, `\quad\href{mailto:…}{…}` — a line of
      // addresses never names the block, nor does one under a link (`{\LARGE \href{https://sherpa.ai/}{Sherpa.ai}}\\
      // research@sherpa.ai`, 2609.25082: below). What no name came before waits for the first that does.
      if !lines.is_empty() && lines.iter().all(names_no_one) {
        let piece: Vec<Token> = lines.into_iter().flat_map(Tokens::unlist).collect();
        match blocks
          .last_mut()
          .and_then(|(_, groups)| groups.last_mut())
          .and_then(|(names, _)| names.last_mut())
        {
          Some(name) => {
            let mut held = std::mem::take(name).unlist();
            held.extend(piece);
            *name = Tokens::new(held);
          },
          None => leading_nameless.extend(piece),
        }
        spaced = true;
        continue;
      }
      let addresses_at = lines
        .iter()
        .position(address_only_line)
        .filter(|at| lines[..*at].iter().all(prints_a_url));
      if let Some(at) = addresses_at {
        let end = at
          + lines[at..]
            .iter()
            .take_while(|line| address_only_line(line))
            .count();
        let addresses: Vec<Tokens> = lines.drain(at..end).collect();
        gaps.drain(at..end.min(gaps.len()));
        let above = blocks
          .last_mut()
          .and_then(|(_, groups)| groups.last_mut())
          .map(|(_, affils)| affils);
        match above {
          Some(affils) => {
            for (at, line) in addresses.into_iter().enumerate() {
              // (a row's addresses a space parts are the one line they print on: joined as they print, at the `\quad` no
              // address text reaches, each keeping its font, [`address_tokens`]; the comma or semicolon ending a piece no
              // part of its address: `e-mail: a@x, \quad b@y`, 2505.22665, whose second address read `˜˜ b@y`)
              match affils.last_mut() {
                Some(last) if at == 0 && spaced && address_only_line(last) => {
                  let mut joined = without_row_spacing(std::mem::take(last).unlist());
                  joined.push(T_CS!("\\quad"));
                  joined.extend(line.unlist());
                  *last = Tokens::new(joined);
                },
                // (`Alessio Brini\footnote{…} \quad \emph{alessio.brini@duke.edu}, \and …`, 2404.04962)
                _ => affils.push(Tokens::new(without_row_spacing(line.unlist()))),
              }
            }
            // (and what comes after them is under the row as well — `… \\ a@x \quad b@y \\ Univ X`; a line of notes, a
            // link, an ORCID, 2606.09024, 2609.05531, 2610.34467; `{\it DeepMind Paris, France}`, 2201.12909; `Princeton
            // Language and Intelligence`, 2606.11499 — unless it opens the next row of names, more of them beside it:
            // `… \\ a@x \quad b@y \\ Cat Cole \quad Dan Dee`)
            if !(row_goes_on
              && lines
                .first()
                .is_some_and(|line| name_count(&visible_name_text(line.unlist_ref())) > 0))
            {
              affils.append(&mut lines);
            }
          },
          None => leading_addresses.extend(addresses),
        }
        // (nothing else on the piece: no names, the row goes on; a line under them starts the next author)
        if lines.is_empty() {
          spaced = true;
          continue;
        }
        if at == 0 {
          spaced = false;
        }
      }
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
      // …and so is a block a line break opens after names only horizontal space parts it from, which have no lines of
      // their own, when its first line is an institution: the space before the line break parts no names, the
      // institution is the names' line (`Lin Liu \hspace{.2cm} \\ Institute of Natural Sciences, School of
      // Mathematical Sciences, MOE-LSC \\ …`, 1908.10448: three creators named by its parts, 85a; the JASA template's
      // `Name\thanks{…}\hspace{.2cm}\\`)
      let names_without_lines = spaced
        && blocks
          .last()
          .and_then(|(_, groups)| groups.last())
          .is_some_and(|(_, affils)| affils.is_empty());
      let labelled_lines = (opens_with_break || after_lines) && lines.first().is_some_and(labelled);
      // (under lines no mark labels, the names line and these lines are one group's lines, which [`name_groups`]
      // parts as it parts any: its first group's lines are the names', its later groups the next authors' — an "and"
      // line between them dropped, a gap, the names' styling, a note or an institution under a person's name opening
      // one: `Assaf P. Oron\hspace{.2cm}\\ Children's Core …\\ and \\ Nancy Flournoy \\ Department of Statistics, …`,
      // 1701.05964; 2004.03971's Han Lin Shang; 2503.11250's `\\[5pt]` gaps; the 85a r2 review's finding 1)
      let mut later_groups: Vec<(Tokens, Vec<Tokens>)> = Vec::new();
      if !labelled_lines
        && opens_with_break
        && names_without_lines
        && lines.first().is_some_and(reads_as_institution)
        && let Some(names_line) = last_names_line.clone()
        && let Some((_, groups)) = blocks.last_mut()
        && let Some((_, affils)) = groups.last_mut()
      {
        let mut grouped_lines = vec![names_line.clone()];
        grouped_lines.append(&mut lines);
        let mut grouped_gaps = vec![false];
        grouped_gaps.append(&mut gaps);
        grouped_gaps[1] = leading_empty > 1;
        let mut grouped = name_groups(grouped_lines.clone(), &grouped_gaps).into_iter();
        match grouped.next() {
          // (the names line read as the names it is)
          Some((first, mut first_lines)) if first == names_line => {
            affils.append(&mut first_lines);
            later_groups.extend(grouped);
          },
          // (else every line is theirs, as a block of names with no lines of their own reads)
          _ => affils.extend(grouped_lines.into_iter().skip(1)),
        }
        if later_groups.is_empty() {
          spaced = true;
          continue;
        }
        spaced = false;
      } else if labelled_lines
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
        let names_gaps = gaps.split_off(names_at);
        gaps.clear();
        affils.append(&mut lines);
        if names.is_empty() {
          spaced = true;
          continue;
        }
        // (a line break, no space, parts the names from the lines above)
        lines = names;
        gaps = names_gaps;
        spaced = false;
      } else if opens_with_break
        && !blocks.is_empty()
        && lines.first().is_some_and(reads_as_institution)
      {
        // (a block a line break opens after the names before it, its first line an institution, has no names: its
        // lines are those names', the nameless creator's lines kept on them by `coalesce_empty_creators`, as Perl
        // reads it — not an author named by the institution: `\author{Ann Able \and ~\\ University of X}`, `… \and
        // \\ University of X`, the rev98b review's pb1, pb6; [`spacer_line`] reads the opening line)
        lines.insert(0, Tokens::default());
        gaps.insert(0, false);
      }
      // A link line printing a URL is no name ([`prints_a_url`]; a linked name is one, 2608.24427): one opening the
      // block goes with the lines under its names
      // (`\url{…} \\ Ann Able \\ Univ A`, the r5 re-review's u17), a block of link lines alone is the whole block's
      // (`Ann Able \\ Univ A \and \url{…}`, u16), not a creator named by its URL.
      // (…but with no other name before it and addresses under it, a link printing a bare address is the block's name:
      // `{\LARGE \href{https://sherpa.ai/}{Sherpa.ai}}\\ research@sherpa.ai`, 2609.25082)
      let leading_links = if addresses_at.is_some_and(|at| at > 0)
        && blocks.iter().all(|(_, groups)| groups.is_empty())
        && lines.first().is_some_and(link_prints_a_bare_address)
      {
        0
      } else {
        (0..lines.len())
          .take_while(|at| prints_a_url(&lines[*at]))
          .count()
      };
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
      gaps.drain(..leading_links.min(gaps.len()));
      let groups = if later_groups.is_empty() {
        name_groups(lines, &gaps)
      } else {
        later_groups
      };
      for (names_line, mut affils) in groups {
        last_names_line = Some(names_line.clone());
        affils.splice(0..0, std::mem::take(&mut links));
        affils.splice(0..0, std::mem::take(&mut leading_addresses));
        let mut names = split_names(if leading_nameless.is_empty() {
          names_line
        } else {
          let mut held = std::mem::take(&mut leading_nameless);
          held.extend(names_line.unlist());
          Tokens::new(held)
        });
        // (a group that is only a collaboration's name, before names of the call's — `\small H.E.S.S. Collaboration
        // \and H.~Abdalla \inst{…} \and …`, 1912.05868 — is a heading over them, [`Collaborations`])
        if affils.is_empty() && names.len() == 1 && collaboration_line(&names[0]) {
          headings.append(&mut names);
          continue;
        }
        // (the collaborations among the names, and a line under them naming one, are the group's lines, each a
        // contact on the names before it, [`Collaborations`]: `… and the GAPS Team`; `(The Nearby Supernova Factory)`
        // under the names, 1511.01102 — every name's affiliation since 82r)
        let mut named = Vec::with_capacity(names.len());
        let mut pieces = Vec::new();
        for name in names {
          match unwrapped_collaboration(&name) {
            Some((text, _)) => pieces.push((text, named.len())),
            None => named.push(name),
          }
        }
        names = named;
        // (each with the names of the group after it)
        let mut collaboration_lines: Vec<Tokens> = pieces
          .into_iter()
          .map(|(text, before)| collaboration_in_lines(text, names.len() - before))
          .collect();
        if !names.is_empty() {
          // (a collaboration's name a line break parts after "and" one line again, [`joined_collaboration`]:
          // 1210.5341)
          let mut joined: Vec<Tokens> = Vec::with_capacity(affils.len());
          for line in affils {
            if let Some(last) = joined.last_mut()
              && let Some(line) = joined_collaboration(last, &line)
            {
              *last = line;
              continue;
            }
            joined.push(line);
          }
          affils = joined;
          // (not one naming it unframed whose address goes on under it, an employer's: [`collaboration_line_before`],
          // 2501.13770)
          let read: Vec<bool> = (0..affils.len())
            .map(|at| collaboration_line_before(&affils[at], affils.get(at + 1)))
            .collect();
          for (line, read) in affils.iter_mut().zip(read) {
            if read {
              *line = collaboration_in_lines(line.clone(), 0);
            }
          }
        }
        // (over a group of names; else each the author it was, where it stood: `\author{The ATLAS Collaboration \and
        // CERN}` keeps both as written)
        if names.iter().any(|name| {
          name_count(&visible_name_text(name.unlist_ref())) >= 1 || marked_after_name(name)
        }) {
          named_any = true;
          collaboration_lines.splice(
            0..0,
            headings
              .drain(..)
              .map(|heading| collaboration_in_lines(heading, names.len())),
          );
        } else {
          pairs.extend(
            headings
              .drain(..)
              .map(|heading| (vec![heading], Vec::new())),
          );
        }
        affils.splice(0..0, collaboration_lines);
        if names.is_empty() {
          names.push(Tokens::default());
        }
        pairs.push((names, affils));
      }
      blocks.push((spaced, pairs));
      spaced = true;
    }
    // (addresses or an icon with no name in the call at all: a nameless author's, as printed, none of them a name —
    // `\author{research@x.ai}`, `\author{\includegraphics{logo}}`)
    if !leading_addresses.is_empty() || !leading_nameless.is_empty() {
      blocks.push((false, vec![(
        vec![Tokens::new(std::mem::take(&mut leading_nameless))],
        std::mem::take(&mut leading_addresses),
      )]));
    }
    // (a heading no names came after: a trailing `\and` group, every name's before it that carries no collaboration
    // yet, as with marks — `\author{Ann Able \\ Univ X \and Bob Baker \\ Univ Y \and The XYZ Collaboration}`,
    // [`Collaborations::finish`]; with no name in the call, the author the call names)
    let trailing_headings = if named_any {
      headings
    } else {
      if !headings.is_empty() {
        blocks.push((
          false,
          headings
            .into_iter()
            .map(|heading| (vec![heading], Vec::new()))
            .collect(),
        ));
      }
      Vec::new()
    };
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
    // (where a heading's collaboration goes in each name's body: after its words, before the notes it carries,
    // [`heading_place`]; right after the name's own tokens when the name makes its creator itself, whose content it
    // then is)
    let mut heading_places: Vec<usize> = Vec::with_capacity(name_keys.len());
    for (names, affils) in groups {
      let first = bodies.len();
      for (name, email) in names {
        let mut body = name.unlist();
        heading_places.push(if makes_its_creator(&body)? {
          body.len()
        } else {
          heading_place(&body)
        });
        if let Some(email) = email {
          body.extend(Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(email)]).unlist());
        }
        bodies.push(body);
        after_bodies.push(Vec::new());
      }
      spans.push((first, bodies.len() - 1, affils));
    }
    // (the names an address on any e-mail line of the block spells, whom elimination gives no other address)
    // (an address line carrying notes is one, its notes the document's — as Perl adds the article `\author`'s lines,
    // nested, where a `\thanks` is a pubnote; a per-author class binding gives its line's notes to the name, below:
    // [`address_line_notes`], `Ann Able, Bob Baker \\ Univ X \\ \texttt{a@x, b@y}\thanks{…}`, 2605.23553's sibling;
    // 2304.05485, 2608.24641)
    if !cautious {
      for (_, _, affils) in &mut spans {
        for line in affils.iter_mut() {
          if let Some((rest, notes)) =
            address_line_notes(line, |rest| Ok(line_emails(rest)?.is_some()))?
          {
            *line = rest;
            address_notes.extend(notes);
          }
        }
      }
    }
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
    // (each group's first name, and the collaborations of the groups' lines, each with the name it is read at and,
    // after a name, its place in that name's body — at its line's place among the contacts there; a heading, `None`,
    // before the name, [`Collaborations`])
    let group_firsts: Vec<usize> = spans.iter().map(|(first, ..)| *first).collect();
    let mut collaboration_events: Vec<(usize, Option<usize>, Tokens)> = Vec::new();
    let groups_with_lines: Vec<bool> = spans.iter().map(|(.., lines)| !lines.is_empty()).collect();
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
      // (a group of one name under lines of its own, right after other lines or none: an `\and` block, printed in its own
      // column, whose addresses are its name's, unless another name spells them — elimination stays inside it; `Michael
      // Ray Johnson\\ … \\ \texttt{john3678@…} \quad \texttt{johnson.michael@…} \and Linda Naimi …`, 2609.05531, where
      // `john3678@` went to the block. Not one after names with no lines of their own, whose addresses its lines may be:
      // `Ann Able \and Bob Baker\\ \{aable,bbaker\}@x.edu`, 2409.18978; nor a piece horizontal space parts, in the row of
      // those above: `Cornell University \hspace{8pt} University of Pennsylvania \\ … \\ \{nac86,…\}@cornell.edu`, 2404.19007,
      // 2505.23367, 2609.33182, an institution read as a name)
      let owns_its_lines = last == first
        && !kept_apart
        && row_starts.get(group).copied().unwrap_or(true)
        && (group == 0 || groups_with_lines.get(group - 1).copied().unwrap_or(false));
      let owners_of = |addresses: &[&Tokens],
                       taken: &mut Vec<bool>,
                       unplaced: &mut Vec<(usize, usize)>,
                       line_at: usize|
       -> Vec<Option<usize>> {
        let texts: Vec<String> = addresses
          .iter()
          .map(|address| address_text(address))
          .collect();
        let whole_block =
          addresses.len() > last - shared_first + 1 && name_keys.len() > 1 && !owns_its_lines;
        let elsewhere_open = unplaced
          .iter()
          .enumerate()
          .any(|(at, (of, count))| at != line_at && (whole_block || *of == group) && *count > 0);
        let owners: Vec<Option<usize>> = if whole_block {
          owners_by_elimination(&name_keys, &texts, taken, elsewhere_open)
        } else if owns_its_lines && addresses.len() > 1 {
          texts
            .iter()
            .map(|text| spelled_owner(&name_keys, text))
            .collect()
        } else if last > shared_first {
          // (under a group kept apart its whole row's names, as its other lines are: `… Songnam Hong$^{\dagger}$\\ … \\
          // \texttt{snhong@…}\\ {\small $^{*}$Equal contribution. …}`, 2609.03842, where `snhong@` went to the block)
          owners_by_elimination(
            &name_keys[shared_first..=last],
            &texts,
            &taken[shared_first..=last],
            elsewhere_open,
          )
          .into_iter()
          .map(|owner| owner.map(|at| shared_first + at))
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
        // (a role mailbox in a list under one name of several is no person's, the block's: `\{eve, info\}@b.edu`, R10)
        let list_of_several = owns_its_lines && addresses.len() > 1 && name_keys.len() > 1;
        owners
          .into_iter()
          .zip(&texts)
          .map(|(owner, text)| {
            owner.or((alone && !(list_of_several && is_role_mailbox(text))).then_some(last))
          })
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
        if let Some((text, after)) = unwrapped_collaboration(line) {
          // (the name after it: a heading before the group's first; else after the name before it, in its body)
          let next = last + 1 - after.unwrap_or(0).min(last + 1 - first);
          if next == first {
            collaboration_events.push((first, None, text));
          } else {
            collaboration_events.push((next - 1, Some(bodies[next - 1].len()), text));
          }
          continue;
        }
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
              Some((stripped, carriers)) => line_emails(&stripped)?.map(|found| (found, carriers)),
              None => None,
            }
          } else {
            None
          };
          let (label, placements): (Option<Tokens>, Vec<(EmailAddress, Vec<usize>)>) = match marked
          {
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
              after_bodies[last].extend(email_calls(label.as_ref(), vec![address], Some(&block))?);
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
          let above = std::mem::take(&mut continuing.carriers);
          after_bodies[last].extend(row_line_calls(
            &line,
            &row_symbols,
            row.clone(),
            &mut placed,
            &mut bodies,
            &above,
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
    // (each collaboration's contact made where it prints among the name's contacts, LaTeXML-structure-xhtml.xsl:795
    // rendering them in document order: `Ann Able \\ for the XYZ Collaboration \\ Univ X` the collaboration, then the
    // institution — 0910.2149, hep-ex0011032, rev97cd D1)
    let mut collaborations = Collaborations::default();
    for (at, (body, after)) in bodies.into_iter().zip(after_bodies).enumerate() {
      if group_firsts.contains(&at) {
        collaborations.close_group();
      }
      for (_, _, text) in collaboration_events
        .iter()
        .filter(|(host, place, _)| *host == at && place.is_none())
      {
        calls.extend(collaborations.read(text.clone())?);
      }
      let mut inserts: Vec<(usize, Vec<Token>)> =
        vec![(heading_places[at], collaborations.author()?)];
      for (_, place, text) in collaboration_events
        .iter()
        .filter(|(host, place, _)| *host == at && place.is_some())
      {
        inserts.push((
          place.unwrap_or_default(),
          collaborations.read(text.clone())?,
        ));
      }
      let mut content = Vec::with_capacity(body.len());
      let mut from = 0;
      for (place, calls) in inserts {
        let place = place.clamp(from, body.len());
        content.extend_from_slice(&body[from..place]);
        content.extend(calls);
        from = place;
      }
      content.extend_from_slice(&body[from..]);
      calls.extend(
        Invocation!(T_CS!("\\lx@add@author"), vec![
          keyvals.clone(),
          Some(Tokens::new(content))
        ])
        .unlist(),
      );
      calls.extend(after);
    }
    // (a trailing heading group, its own `\and` group: every name's before it that carries none)
    if !trailing_headings.is_empty() {
      collaborations.close_group();
      for heading in trailing_headings {
        calls.extend(collaborations.read(heading)?);
      }
    }
    calls.extend(collaborations.finish()?);
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
  calls.extend(document_notes);
  calls.extend(address_notes);
  Ok(Tokens::new(calls))
}

/// Whether `piece`, a `\\`-line of an author block, is only the block's spacing, wherever it stands — the one
/// predicate the block's lines and whether a line break opens the block are read by: empty or spacing alone
/// ([`spacing_end`]), or printing nothing ([`prints_nothing`]) and opening or closing no environment (an environment's
/// `\begin`/`\end`, which the block's lines go on inside).
fn spacer_line(piece: &Tokens) -> bool {
  spacing_end(piece.unlist_ref()) == 0
    || (prints_nothing(piece)
      && !piece
        .unlist_ref()
        .iter()
        .any(|t| *t == T_CS!("\\begin") || *t == T_CS!("\\end")))
}
