//! enotez.sty — raw load; the notes become LaTeXML endnotes and `\printendnotes` lists them.
//!
//! enotez gives the list its notes only through the `.aux` round trip: `\endnote` writes
//! `\enotez@note{id}{mark}{split}{…}{text}` to `\@auxout` (enotez.sty:323-327) and only the next
//! run's `\enotez@note` (:330-340) fills the property lists `\printendnotes` loops over (:435-471).
//! LaTeXML converts in one pass with no `.aux`, so the list printed empty and every mark's
//! `\hyperlink{enz.N}` (:197-212) dangled (skeldoc manual: 15 of 345 words of its notes kept, 15
//! schema errors; mla-example). Perl has no binding and loses them alike.
//!
//! The package is loaded raw — classes build on its templates and internals (mla.cls:285-290,
//! skeldoc.sty:831 and :860-867, wiley-article.cls:695-700) — and its note internals are replaced:
//! * the note is recorded in the store `\enotez@note` fills, directly (no `.aux`; its text is not
//!   kept, since the list is the notes themselves);
//! * `\endnote`, `\endnotemark`, `\endnotetext` are `\lx@note`/`\lx@notemark`/`\lx@notetext` of
//!   type `endnote`, as endnotes.sty's binding; an optional mark is the note's mark without a step
//!   (enotez.sty:236-238); the inline tag follows `mark-cs`/`mark-format`;
//! * each note is in the list `ent` and in the list of its partition `ent<k>` — the notes made
//!   after the k-th plain `\printendnotes` (enotez_en.tex:167-170), or a `split` unit;
//! * `split=section|chapter` advances at the first note of a new unit, found by comparing what the
//!   unit's headings changed: the counter and printed number (an appendix's section A after section
//!   1 is a new unit) and the starred-heading counter LaTeXML steps (`\c@UNsection`/`\c@UNchapter`).
//!   enotez prepends to `\section`/`\chapter` (:884-905), starred forms included, which LaTeXML
//!   keeps locked. Differences from pdflatex: under `split=section` a `\chapter` heading starts a
//!   split (pdflatex: only a `\section` after it); an `\endnotetext` whose `\endnotemark` came before
//!   a heading is listed in the split of the last note before it, titled by the text's unit, so a
//!   later note under that heading opens another split with the same title (pdflatex: one split, the
//!   text numbered 0 under `reset`); a `\c@chapter` created
//!   mid-document (a `\chapter*` alias in a class without chapters) splits once;
//! * `\printendnotes` prints, through the package's own heading, list name and split titles, one
//!   `ltx:TOC` per partition (`\printendnotes*`: all notes).
//!
//! Presentational keys (`list-style`, the list template's `format`/`number`/`notes-sep`, `backref`)
//! have no effect: the TOC replaces the typeset list. Dropped with the replaced internals: the
//! undocumented `\enotezdisable` (:240), an optional mark's `\@currentlabel` (:237; a `\label` after
//! it names the note by the kernel's tag), and the non-split list's `\prop_gremove` (:634; the
//! partitions keep the lists apart). The marks' and the list's `enz.N` hyperlinks are not made, so
//! none dangles; a class's `\enotez_write_mark:nn` (skeldoc.sty:831) is not reached. Guards
//! `perfect_kernel_batch58::{enotez_notes_are_listed, enotez_split_lists_each_section,
//! enotez_split_by_chapter, enotez_marks_texts_and_repeated_lists, enotez_split_by_section_in_a_book}`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("enotez", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\endnotetyperefname", None, "endnote");
  DefConstructor!(
    "\\lx@enotez@TOC{}",
    "<ltx:TOC lists='#1' scope='global' show='refnum > note'/>"
  );
  RawTeX!(
    r"\def\lx@enotez@list#1{\edef\lx@enotez@lists{#1}\expandafter\lx@enotez@TOC\expandafter{\lx@enotez@lists}}
\ExplSyntaxOn
\cs_set_eq:NN \lx@enotez@store \enotez@note
\cs_set_protected:Npn \enotez_save_note:nnnnnnn #1#2#3#4#5#6#7
  { \lx@enotez@store {#1} {#2} {#3} {#4} {#5} {#6} { } }
\cs_set_eq:NN \lx@enotez@record:nn \enotez_endnote_text:nn
\tl_new:N \g__lx_enotez_unit_tl
\cs_new_protected:Npn \lx@enotez@split@sync
  {
    \bool_if:NT \l__enotez_split_bool
      {
        \str_if_eq:VnTF \l__enotez_split_tl {section}
          {
            \tl_set:Nx \l__enotez_tmpc_tl
              {
                \cs_if_exist:NT \c@chapter { \int_use:N \c@chapter }
                \cs_if_exist:NT \c@UNchapter { : \int_use:N \c@UNchapter }
                / \int_compare:nNnF \c@section = 0 { \int_use:N \c@section . \thesection }
                \cs_if_exist:NT \c@UNsection { / \int_use:N \c@UNsection }
              }
          }
          {
            \tl_set:Nx \l__enotez_tmpc_tl
              {
                \cs_if_exist:NT \c@chapter { \int_use:N \c@chapter . \thechapter }
                \cs_if_exist:NT \c@UNchapter { / \int_use:N \c@UNchapter }
              }
          }
        \tl_if_eq:NNF \l__enotez_tmpc_tl \g__lx_enotez_unit_tl
          {
            \tl_gset_eq:NN \g__lx_enotez_unit_tl \l__enotez_tmpc_tl
            \int_gincr:N \g__enotez_list_printed_int
            \bool_if:NT \l__enotez_reset_bool { \setcounter {endnote} {0} }
          }
      }
  }
\cs_set:Npn \ext@endnote { ent ~ ent \int_use:N \g__enotez_list_printed_int }
\cs_set:Npn \fnum@endnote { \enotezwritemark { \enmarkstyle \theendnote } }
\cs_set_protected:Npn \enotez_endnote:nn #1#2
  {
    \lx@enotez@split@sync
    \int_gincr:N \g__enotez_endnote_id_int
    \quark_if_no_value:nTF {#1}
      { \lx@note {endnote} {#2} }
      { \lx@note {endnote} [#1] {#2} }
    \lx@enotez@record:nn {#1} { }
  }
\cs_set_protected:Npn \enotez_endnote_mark:n #1
  {
    \lx@enotez@split@sync
    \int_gincr:N \g__enotez_endnote_id_int
    \quark_if_no_value:nTF {#1}
      { \lx@notemark {endnote} }
      { \lx@notemark {endnote} [#1] }
  }
\cs_set_protected:Npn \enotez_endnote_text:nn #1#2
  {
    \quark_if_no_value:nTF {#1}
      { \lx@notetext {endnote} {#2} }
      { \lx@notetext {endnote} [#1] {#2} }
    \lx@enotez@record:nn {#1} { }
  }
\cs_set_protected:Npn \enotez_build_print_list:nnnn #1#2#3#4
  {
    \bool_if:nTF {#1}
      { \lx@enotez@list {ent} }
      {
        \bool_if:NTF \l__enotez_split_bool
          {
            \int_zero:N \l__enotez_tmpa_int
            \int_do_while:nn { \l__enotez_tmpa_int <= \g__enotez_list_printed_int }
              {
                \seq_clear:N \l__enotez_tmpa_seq
                \prop_map_inline:Nn \g__enotez_endnote_split_prop
                  {
                    \tl_if_eq:xxT { \int_use:N \l__enotez_tmpa_int } {##2}
                      { \seq_put_right:Nn \l__enotez_tmpa_seq {##1} }
                  }
                \seq_if_empty:NF \l__enotez_tmpa_seq
                  {
                    \enotez_get_split_title:x { \seq_item:Nn \l__enotez_tmpa_seq {1} }
                    \lx@enotez@list { ent \int_use:N \l__enotez_tmpa_int }
                  }
                \int_incr:N \l__enotez_tmpa_int
              }
          }
          { \lx@enotez@list { ent \int_use:N \g__enotez_list_printed_int } }
      }
  }
\ExplSyntaxOff"
  );
});
