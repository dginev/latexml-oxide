//! glossaries.sty — strict translation of Perl `glossaries.sty.ltxml` (126L).
//!
//! Perl raw-loads real TL glossaries.sty (~8700 lines) via
//! `InputDefinitions(noltxml=>1)`, then layers surgical overrides:
//!   - wrap `\@gls@link` output in `<ltx:glossaryref>`
//!   - hook `\@newglossaryentryposthook` to emit `<ltx:glossarydefinition>`
//!   - redefine `\printglossary` to emit `<ltx:glossary>`
//!
//! This Rust file mirrors the Perl 1:1.

use crate::engine::latex_constructs::{adjust_backmatter_element, note_backmatter_element};
use crate::prelude::*;
use latexml_core::definition::argument::ArgWrap;
use latexml_core::digested::DigestedData;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L18-19.
  InputDefinitions!("glossaries", extension => Some(Cow::Borrowed("sty")), noltxml => true);
  RequirePackage!("xspace");

  // Perl L21: Silence pointless warnings.
  def_macro_noop("\\glsnoidxstripaccents")?;

  //======================================================================
  // Perl L26-37: wrap `\@gls@link` in `<ltx:glossaryref>`.
  Let!("\\lx@orig@glossaries@gls@link", "\\@gls@link");
  DefMacro!("\\@gls@link[]{}{}",
    "\\lx@glossaries@gls@link{\\csname glo@#2@type\\endcsname}{#2}{\\lx@orig@glossaries@gls@link[#1]{#2}{#3}}");
  DefConstructor!("\\lx@glossaries@gls@link{}{}{}",
    "<ltx:glossaryref inlist='#list' key='#2'>#3</ltx:glossaryref>",
    enter_horizontal => true,
    properties => sub[args] {
      // Perl: $list = ToString($_[1]); $list = 'main' unless $_[1];
      let list = args[0].as_ref().map(|t| t.to_string()).unwrap_or_default();
      let list = if list.is_empty() { "main".to_string() } else { list };
      Ok(stored_map!("list" => list))
    });

  //======================================================================
  // Perl L40-42: skip over hyperref wrapping; we handle it.
  DefMacro!("\\glsdohyperlink{}{}",   "#2");
  DefMacro!("\\glsdonohyperlink{}{}", "#2");
  RawTeX!("\\glsdisablehyper");

  // Perl L45: This seems necessary, although it ought to be built in???
  DefMacro!("\\glspostlinkhook", "\\xspace");

  //======================================================================
  // Perl L52-83: hook `\@newglossaryentryposthook` so each entry produces
  // a structured `<ltx:glossarydefinition>` with one `<ltx:glossaryphrase>`
  // per field. The keys mirror Perl exactly; the closing `}` is required
  // because the hook body is interpreted as a single argument group.
  DefMacro!("\\@newglossaryentryposthook",
    "\\lx@glossaries@newentry{\\@glo@type}{\\glslabel}{\
name=\\@glo@name,\
description=\\@glo@desc,\
symbol=\\@glo@symbol,\
symbolplural=\\@glo@symbolplural,\
text=\\@glo@text,\
plural=\\@glo@plural,\
first=\\@glo@first,\
firstplural=\\@glo@firstplural,\
sort=\\@glo@sort,\
counter=\\@glo@counter,\
see=\\@glo@see,\
parent=\\@glo@parent,\
prefix=\\@glo@prefix,\
short=\\@glo@short,\
shortplural=\\@glo@shortpl,\
long=\\@glo@long,\
longplural=\\@glo@longpl\
}");

  // Perl L85-97: DefConstructor that emits the structured definition.
  // Iterate the keyvals in sorted-by-key order and insert one
  // `<ltx:glossaryphrase>` per non-empty value (matches Perl's
  // `if ToString($value)` guard).
  DefConstructor!("\\lx@glossaries@newentry{}{} RequiredKeyVals",
    sub[document, args, _props] {
      let list = args[0].as_ref().map(|d| d.to_string()).unwrap_or_else(|| "main".to_string());
      let key  = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();
      document.open_element("ltx:glossarydefinition",
        Some(string_map!("key" => key.clone(), "inlist" => list)), None)?;
      if let Some(kv_digested) = args[2].as_ref() {
        if let DigestedData::KeyVals(ref kvs) = *kv_digested.data() {
          // Sort by role (Perl: `sort keys %$hash`).
          let mut pairs: Vec<(String, ArgWrap)> = kvs.get_pairs()
            .map(|(k, v)| (k.clone(), v.clone())).collect();
          pairs.sort_by(|a, b| a.0.cmp(&b.0));
          for (role, val) in pairs {
            let val_str = val.to_string();
            if val_str.is_empty() { continue; }
            // Insert <ltx:glossaryphrase key=key role=role>val</ltx:glossaryphrase>
            document.open_element("ltx:glossaryphrase",
              Some(string_map!("key" => key.clone(), "role" => role)), None)?;
            document.absorb_string(&val_str, &NO_PROPERTIES)?;
            document.close_element("ltx:glossaryphrase")?;
          }
        }
      }
      document.close_element("ltx:glossarydefinition")?;
    }
  );

  //======================================================================
  // Perl L101-104: redefine `\printglossary` to dispatch to our constructor.
  DefMacro!("\\printglossary",
    "\\global\\let\\warn@noprintglossary\\relax\
\\@ifnextchar[{\\lx@printglossary}{\\lx@printglossary[type=main]}");
  // Perl L105.
  Let!("\\printnoidxglossary", "\\printglossary");

  // Perl L107-117: emit `<ltx:glossary>` placeholder with computed id,
  // list, and title. The XSLT pipeline later expands it into the actual
  // rendered glossary entries.
  DefConstructor!("\\lx@printglossary OptionalKeyVals",
    "<ltx:glossary xml:id='#id' lists='#list'>\
<ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>\
</ltx:glossary>",
    properties => sub[args] {
      // Perl L113-117 — compute type (default 'main'), title (digest
      // \@glotype@<type>@title), and id (docid + ".glo." + cleaned type).
      let typ = args[0].as_ref().and_then(|d| {
        if let DigestedData::KeyVals(ref kvs) = *d.data() {
          kvs.get_value("type").map(|v| v.to_string())
        } else { None }
      }).unwrap_or_else(|| "main".to_string());
      let title_cs = s!("\\@glotype@{typ}@title");
      let title = stomach::digest(T_CS!(&*title_cs))
        .map(|d| d.to_string()).unwrap_or_default();
      let docid = state::lookup_value("thedocument@ID")
        .and_then(|v| match v { Stored::String(s) => Some(arena::to_string(s)), _ => None })
        .unwrap_or_default();
      let cleaned = typ.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
      let id = if docid.is_empty() {
        format!("glo.{cleaned}")
      } else {
        format!("{docid}.glo.{cleaned}")
      };
      Ok(stored_map!("list" => typ, "id" => id, "title" => title))
    },
    after_digest => sub[whatsit] {
      // Perl L114: noteBackmatterElement(<whatsit>, 'ltx:glossary');
      note_backmatter_element(whatsit, "ltx:glossary");
    },
    before_construct => sub[doc, whatsit] {
      // Perl L117: adjustBackmatterElement($_[0], $_[1]);
      adjust_backmatter_element(doc, whatsit)?;
    }
  );
});
