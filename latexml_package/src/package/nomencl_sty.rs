//! nomencl.sty — a nomenclature list.
//!
//! The raw package is loaded (as Perl's `nomencl.sty.ltxml` does), then its
//! two ends are rebound onto this engine's glossary model. Raw, the package
//! only WRITES: `\nomenclature[prefix]{sym}{desc}` is a `\protected@write`
//! of a `\nomenclatureentry` line to `\jobname.nlo` (nomencl.sty:227-245),
//! and `\printnomenclature` `\@input@`s `\jobname.nls` (:277-282), the output
//! of an external makeindex run over that file. No makeindex runs here (nor
//! in Perl — the list is lost in both engines, silently: 0 errors and
//! 17-37 % recall on the five shipped samples). The entries become
//! `<ltx:glossarydefinition inlist="nomenclature">` with the phrases the
//! written line carries — `sort` (prefix + symbol, the makeindex key),
//! `name`, `description` (with `\nomeqref{\theequation}` appended, as the
//! written line does: ", see equation (N)" when the entry said `\nomrefeq`),
//! and under `nomentbl` `unit` and `note` — and `\printnomenclature` an
//! empty `<ltx:glossary lists="nomenclature" role="nomenclature">` that the
//! MakeGlossary post stage fills and sorts, the same in-memory route
//! glossaries takes instead of makeindex; `role="nomenclature"` tells it to
//! list EVERY definition (a nomenclature has no `\gls` references — makeindex
//! prints every written line). Under `stdsubgroups` each entry also carries
//! its makeindex group heading (`group`): `lethead_flag` writes
//! `\nomgroup{<letter>}` before each initial's entries (nomencl.ist:36-38),
//! and nomencl sets it as a label (nomencl.sty:247-272: "Latin Letters",
//! "Greek Letters", "Subscripts"), so `\nomgroup` itself is digested for the
//! prefix's first letter, its `\item[…]` (or nomentbl's `\item&\multicolumn`)
//! reduced to the label; MakeGlossary sets the heading before the group's
//! first entry — only while `\nomgroup` is the package's own (nomencl's, or
//! nomentbl.sty's): a class's redefinition may set sectioning headings
//! (unbtex.cls:594-606 `\section*`, `\pretextualchapter`), which cannot sit
//! in a glossary phrase. Surpass over Perl
//! (OXIDIZED_DESIGN_DIVERGENCES #234). Guards:
//! `cluster_package_guards::nomencl_inline::{nomenclature_entries_become_glossary_definitions,
//! printnomenclature_lists_every_entry, stdsubgroups_lists_group_headings}`.
use crate::{
  engine::latex_constructs::{adjust_backmatter_element, note_backmatter_element},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: nomencl.sty.ltxml — the raw package.
  InputDefinitions!("nomencl", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\nomenclature` = `\protect\@nomenclature` (nomencl.sty:216), and
  // `\makenomenclature` (:205-213) re-points `\@nomenclature` at the writing
  // branch; the user-level macro is rebound whole, past both. The optional
  // prefix defaults to `\nomprefix` (:39-40, option `prefix`/`noprefix`),
  // and `nomentbl` (:41-42) makes the entry `{sym}{desc}{unit}{note}`
  // (:228-235). The arguments are read WITHOUT the raw macro's
  // `\begingroup\@sanitize` (that serves the written string), so `$a$`
  // digests as math and the description as text. The unit column is
  // `\unit{#4}`, as the written line carries it (:232 `&\unit{#4}&`;
  // `nomentbl` loads siunitx, :182-184): a unit macro is a unit only inside
  // it. Bare, siunitx's `\meter` is `\relax` (siunitx.sty.ltxml:1352) and the
  // entry's unit vanished — pdflatex prints "m", "K", "W K−1" (nomencl
  // sample03.tex:11-30, `{\meter}`, `{\kelvin}`, `{\watt\per\kelvin}`).
  // A blank unit (`{}`, sample03.tex:29) or note prints nothing and keeps no phrase;
  // the test is ltcmd's `\IfBlankF` (blank = empty or spaces), not
  // `\ifx\relax#1`, which is true of `\meter`.
  // Guard: `package_leads_56::nomentbl_unit_column_is_a_siunitx_unit`.
  // The entry is typeset only where `\printnomenclature` prints the list (user ruling 2026-10-18,
  // OXIDIZED_DESIGN_DIVERGENCES #738): TeX writes it to the `.nlo` (nomencl.sty:208-245) and typesets it only from the
  // `.nls`, so it is kept raw (`\lx@defer@print`, sect11.rs) with the equation number it was written with
  // (`\theequation`, expanded by the write), and digested when the list is printed — before the entries too, as in
  // 2506.02168, whose `\printnomenclature` stands ahead of them (a printed entry's errors are TeX's: its
  // `\nomenclature{\|\cdot\|_p}` symbol sets `\|` and `_` outside math). Guard:
  // `perfect_kernel_batch73a::nomenclature_entry_is_typeset_only_when_printed`.
  // nomencl.sty:216-221 reads the `[prefix]` — a makeindex sort key — under `\@sanitize`: `\nomenclature[R_alpha]{…}`'s
  // `_` is a character, where a live one erred in the sort phrase (12 of run 336's papers). Only the prefix: the symbol
  // and description, digested here rather than written out, keep their catcodes.
  DefMacro!("\\nomenclature",
    "\\begingroup\\@sanitize\\@ifnextchar[{\\lx@nomencl@prefixed}{\\endgroup\\lx@nomencl@entry[\\nomprefix]}");
  RawTeX!(r"\makeatletter
\def\lx@nomencl@prefixed[#1]{\endgroup\lx@nomencl@entry[#1]}
\def\lx@nomencl@entry[#1]#2#3{\if@nomentbl\expandafter\lx@nomencl@entrytbl\else\expandafter\lx@nomencl@entryplain\fi{#1}{#2}{#3}}
\def\lx@nomencl@entrytbl#1#2#3#4#5{\protected@edef\lx@nomencl@eq{\theequation}\expandafter\lx@nomencl@entrytbl@\expandafter{\lx@nomencl@eq}{#1}{#2}{#3}{#4}{#5}}
\def\lx@nomencl@entrytbl@#1#2#3#4#5#6{\lx@defer@print{nomenclature}{\lx@nomencl@ifaccepted{#2}{#3}{#4#5#6}{\lx@nomencl@definition{#2}{#3}{{#4\nomeqref{#1}}}{\lx@nomencl@unit{#5}}{\IfBlankF{#6}{#6}}{\lx@nomencl@group{#2}}}{}}}
\def\lx@nomencl@unit#1{\IfBlankF{#1}{\unit{#1}}}
\def\lx@nomencl@entryplain#1#2#3{\protected@edef\lx@nomencl@eq{\theequation}\expandafter\lx@nomencl@entryplain@\expandafter{\lx@nomencl@eq}{#1}{#2}{#3}}
\def\lx@nomencl@entryplain@#1#2#3#4{\lx@defer@print{nomenclature}{\lx@nomencl@ifaccepted{#2}{#3}{#4}{\lx@nomencl@definition{#2}{#3}{{#4\nomeqref{#1}}}{}{}{\lx@nomencl@group{#2}}}{}}}
\def\lx@nomencl@group#1{\begingroup\edef\lx@nomencl@pfx{#1}\ifx\lx@nomencl@pfx\@empty\else
  \ifx\nomgroup\lx@nomencl@ownnomgroup\expandafter\lx@nomencl@group@\lx@nomencl@pfx\@nil\fi\fi\endgroup}
\let\lx@nomencl@ownnomgroup\nomgroup
\def\lx@nomencl@group@#1#2\@nil{\if@nomentbl\def\item&\multicolumn##1##2##3{##3}\else\def\item[##1]{##1}\fi
  \uppercase{\nomgroup{#1}}}
\makeatother");
  // makeindex rejects a written line with a second `|` (encap) or `@` (actual), braced or not, and nomencl writes one of
  // each itself (nomencl.sty:227-245: `<prefix><symbol>@[{<symbol>}]\begingroup <description>…|nompageref`, in table mode
  // the unit and the note too, :228-232 `&\unit{#4}&\begingroup#5\endgroup&`; nomencl.ist
  // keeps makeindex's `|` and `@`, quote `%`): an entry whose prefix, symbol or description holds one is never in the
  // `.nls`, never printed ("Extra `|'", the `.ilg`). 2506.02168's `\nomenclature{\|\cdot\|_p}{…}`: pdflatex and makeindex
  // print a list without it, where the binding's list set `\|` and `_` as text (an Error). Such an entry is left out with
  // a Warning when the list is printed, as makeindex leaves it out. Guards: `perfect_kernel_batch73a::{
  // nomenclature_entry_is_typeset_only_when_printed, nomentbl_unit_with_a_bar_is_rejected}`.
  DefMacro!("\\lx@nomencl@ifaccepted{}{}{}", sub[(prefix, symbol, description)] {
    let mut extra = None;
    for part in [&prefix, &symbol, &description] {
      for token in part.unlist_ref() {
        if let Some(special) = token.with_str(|text| text.chars().find(|c| matches!(c, '|' | '@'))) {
          extra.get_or_insert(special);
        }
      }
    }
    Ok(match extra {
      None => Tokens!(T_CS!("\\@firstoftwo")),
      Some(special) => {
        Warn!("malformed", "nomenclature",
          s!("makeindex rejects the nomenclature entry {} (Extra `{special}'), so the list prints without it", symbol.untex()));
        Tokens!(T_CS!("\\@secondoftwo"))
      },
    })
  });
  // The sort prefix (#1) is only written to the `.nlo` for makeindex (nomencl.sty `\nomenclature[#1]`), never typeset,
  // so it is read as a string: `\nomenclature[Enc_Aut]{…}` raised "Script _ can only appear in math mode" where pdflatex
  // is silent (2105.05016).
  DefConstructor!("\\lx@nomencl@definition Semiverbatim{}{}{}{}{}",
    "<ltx:glossarydefinition key='#key' inlist='nomenclature'>\
     <ltx:glossaryphrase key='#key' role='sort'>#1#2</ltx:glossaryphrase>\
     <ltx:glossaryphrase key='#key' role='name'>#2</ltx:glossaryphrase>\
     <ltx:glossaryphrase key='#key' role='description'>#3</ltx:glossaryphrase>\
     ?#4(<ltx:glossaryphrase key='#key' role='unit'>#4</ltx:glossaryphrase>)()\
     ?#5(<ltx:glossaryphrase key='#key' role='note'>#5</ltx:glossaryphrase>)()\
     ?#6(<ltx:glossaryphrase key='#key' role='group'>#6</ltx:glossaryphrase>)()\
     </ltx:glossarydefinition>",
    properties => sub[_args] {
      let n = lookup_int("nomencl@entries") + 1;
      assign_value("nomencl@entries", Stored::Int(n), Some(Scope::Global));
      Ok(stored_map!("key" => s!("nomencl.{n}")))
    });
  // `\printnomenclature[width]` (:277-282): the list container, titled
  // `\nomname` (:53 and the babel captions :55-175), filled at post.
  DefMacro!("\\printnomenclature", "\\@ifnextchar[{\\lx@printnomenclature}{\\lx@printnomenclature[]}");
  DefConstructor!("\\lx@printnomenclature[]",
    "<ltx:glossary xml:id='#id' lists='nomenclature' role='nomenclature'>\
     <ltx:title>#title</ltx:title></ltx:glossary>",
    properties => sub[_args] {
      // The list is printed: its deferred entries are typeset (OXIDIZED_DESIGN_DIVERGENCES #738).
      resolve_deferred_prints(Some("nomenclature"))?;
      let title = digest_in_group(T_CS!("\\nomname")).map(|d| d.to_string()).unwrap_or_default();
      let docid = lookup_value("thedocument@ID")
        .and_then(|v| match v { Stored::String(s) => Some(to_string(s)), _ => None })
        .unwrap_or_default();
      let id = if docid.is_empty() { "glo.nomenclature".to_string() } else { format!("{docid}.glo.nomenclature") };
      Ok(stored_map!("id" => id, "title" => title))
    },
    after_digest => sub[whatsit] {
      note_backmatter_element(whatsit, "ltx:glossary");
    },
    before_construct => sub[doc, whatsit] {
      adjust_backmatter_element(doc, whatsit)?;
    });
});
