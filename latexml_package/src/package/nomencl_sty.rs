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
//! prints every written line). Surpass over Perl
//! (OXIDIZED_DESIGN_DIVERGENCES #234). Guards:
//! `cluster_package_guards::nomencl_inline::{nomenclature_entries_become_glossary_definitions,
//! printnomenclature_lists_every_entry}`.
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
  // digests as math and the description as text.
  DefMacro!("\\nomenclature", "\\@ifnextchar[{\\lx@nomencl@entry}{\\lx@nomencl@entry[\\nomprefix]}");
  RawTeX!(r"\makeatletter
\def\lx@nomencl@entry[#1]#2#3{\if@nomentbl\expandafter\lx@nomencl@entrytbl\else\expandafter\lx@nomencl@entryplain\fi{#1}{#2}{#3}}
\def\lx@nomencl@entrytbl#1#2#3#4#5{\lx@nomencl@definition{#1}{#2}{{#3\nomeqref{\theequation}}}{#4}{#5}}
\def\lx@nomencl@entryplain#1#2#3{\lx@nomencl@definition{#1}{#2}{{#3\nomeqref{\theequation}}}{}{}}
\makeatother");
  DefConstructor!("\\lx@nomencl@definition{}{}{}{}{}",
    "<ltx:glossarydefinition key='#key' inlist='nomenclature'>\
     <ltx:glossaryphrase key='#key' role='sort'>#1#2</ltx:glossaryphrase>\
     <ltx:glossaryphrase key='#key' role='name'>#2</ltx:glossaryphrase>\
     <ltx:glossaryphrase key='#key' role='description'>#3</ltx:glossaryphrase>\
     ?#4(<ltx:glossaryphrase key='#key' role='unit'>#4</ltx:glossaryphrase>)()\
     ?#5(<ltx:glossaryphrase key='#key' role='note'>#5</ltx:glossaryphrase>)()\
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
      let title = digest(T_CS!("\\nomname")).map(|d| d.to_string()).unwrap_or_default();
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
