//! pgfkeys.code.tex — the raw key engine plus native accessors (slice 0 of the
//! native pgfkeys dispatch, `docs/performance/PERFORMANCE.md`).
//!
//! The whole raw file loads first, so every handler (`/handlers/*/.@cmd`),
//! the error keys, filtering, families and the parse loop are the real TeX;
//! the natives below then replace only the leaf accessors every key dispatch
//! runs (`\pgfkeys@ifcsname` 23,650 calls on one ZX circuit, `\pgfkeysgetvalue`
//! and `\pgfkeysifdefined` as many), on the SAME `\pgfk@<key>` control-sequence
//! storage the raw code uses (pgfkeys.code.tex:101/155/173/213) — so `\tikzset`,
//! `\tcbset`, pgfplots and forest, which poke that storage directly, see one
//! store. Perl's own native engine (`pgfkeys.code.tex.ltxml:40-544`,
//! `pgfkeyCS` + `DefMacroI`) is the anchor; it is disabled there as incomplete,
//! which this hybrid avoids by never replacing a handler. `\pgfkeyssetvalue`
//! stores the value tokens verbatim (`\edef` of `\the\pgfkeys@temptoks`, :101 —
//! no expansion, `#` stays a parameter character), so the store is a
//! parameter-free macro whose body is not parameter-packed.
//!
//! `LATEXML_PGFKEYS_NATIVE=0` keeps the raw accessors — the differential
//! harness (`cluster_package_guards::pgfkeys_native_accessors`) converts every
//! fixture both ways and requires byte-identical core XML.
use crate::prelude::*;

/// The `\pgfk@<key>` control sequence of a key (Perl `pgfkeyCS`).
fn pgfkey_cs(key: &Tokens) -> Result<Token> {
  let key = Expand!(key.clone()).to_string().replace('\n', " ");
  Ok(T_CS!(&s!("\\pgfk@{key}")))
}

/// `\ifcsname …\endcsname`: true once ANY meaning exists — a definition, a
/// `\csname`-made `\relax`, or a `\let` to a character (pgfplots.code.tex:5332
/// `\pgfkeyslet{/tikz/#1discont}=\pgfutil@empty` lets the key to the `=`).
fn csname_defined(name: &str) -> bool { has_meaning(&T_CS!(&s!("\\{name}"))) }

fn has_meaning(cs: &Token) -> bool { lookup_meaning(cs).is_some() }

fn first_or_second(defined: bool) -> Tokens {
  Tokens!(T_CS!(if defined {
    "\\pgfkeys@firstoftwo"
  } else {
    "\\pgfkeys@secondoftwo"
  }))
}

/// The value tokens stored under a key (a macro body, or the token a
/// `\pgfkeyslet` bound it to), or none.
fn stored_value(cs: &Token) -> Result<Option<Tokens>> {
  Ok(match lookup_meaning(cs) {
    Some(Stored::Token(t)) => Some(Tokens!(t)),
    Some(_) => match lookup_definition(cs)? {
      Some(defn) => match defn.get_expansion() {
        Some(ExpansionBody::Tokens(t)) => Some(t.clone()),
        _ => None,
      },
      None => None,
    },
    None => None,
  })
}

/// Store `value` verbatim under `cs` (pgfkeys.code.tex:101 `\edef … {\the\pgfkeys@temptoks}`).
fn store_value(cs: Token, value: Tokens) -> Result<()> {
  def_macro(
    cs,
    None,
    ExpansionBody::Tokens(value),
    Some(ExpandableOptions {
      nopack_parameters: true,
      ..ExpandableOptions::default()
    }),
  )
}

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("pgfkeys.code", extension => Some(Cow::Borrowed("tex")), noltxml => true, reloadable => true);
  if std::env::var("LATEXML_PGFKEYS_NATIVE").as_deref() != Ok("0") {
    // :29 — the branch selector every case-dispatch step runs.
    DefMacro!("\\pgfkeys@ifcsname{}", sub[(name)] {
      let name = Expand!(name).to_string();
      Ok(first_or_second(csname_defined(&name)))
    }, locked => true);
    // :213
    DefMacro!("\\pgfkeysifdefined{}", sub[(key)] {
      let cs = pgfkey_cs(&key)?;
      Ok(first_or_second(has_meaning(&cs)))
    }, locked => true);
    // :226 — true for command keys too (`/.@cmd`), not for handled keys.
    DefMacro!("\\pgfkeysifassignable{}{}{}", sub[(key, yes, no)] {
      let cs = pgfkey_cs(&key)?;
      let key_str = Expand!(key).to_string();
      let cmd = T_CS!(&s!("\\pgfk@{key_str}/.@cmd"));
      Ok(if has_meaning(&cs) || has_meaning(&cmd) { yes } else { no })
    }, locked => true);
    // :173 — `\let#2` the stored value, or `\relax`.
    DefPrimitive!("\\pgfkeysgetvalue{} DefToken", sub[(key, cmd)] {
      let cs = pgfkey_cs(&key)?;
      if has_meaning(&cs) {
        let_i(&cmd, &cs, None);
      } else {
        let_i(&cmd, &T_CS!("\\relax"), None);
      }
    }, locked => true);
    // :155
    DefPrimitive!("\\pgfkeyslet{} DefToken", sub[(key, value)] {
      let cs = pgfkey_cs(&key)?;
      let_i(&cs, &value, None);
    }, locked => true);
    // :101
    DefPrimitive!("\\pgfkeyssetvalue{}{}", sub[(key, value)] {
      let cs = pgfkey_cs(&key)?;
      store_value(cs, value)?;
    }, locked => true);
    // :125-135 — before + value + after; the `\xdef` there is only the
    // transport out of the scratch group, the assignment is `\pgfkeyslet` =
    // a LOCAL `\let` (keys are local to the current group, :94).
    DefPrimitive!("\\pgfkeysaddvalue{}{}{}", sub[(key, before, after)] {
      let cs = pgfkey_cs(&key)?;
      let mut body: Vec<Token> = before.unlist();
      if let Some(existing) = stored_value(&cs)? {
        body.extend(existing.unlist());
      }
      body.extend(after.unlist());
      store_value(cs, Tokens::new(body))?;
    }, locked => true);
    // :193-194 — the value's own control sequence when it exists, else the
    // raw file's pre-defined `\pgfkeys@relax`: the key itself is never
    // `\csname`-defined (zx-calculus reads `\pgfkeysvalueof{/zx/zx@isInFrac}`
    // before the key exists and later probes it with `\pgfkeysifdefined`).
    DefMacro!("\\pgfkeysvalueof{}", sub[(key)] {
      let cs = pgfkey_cs(&key)?;
      Ok(Tokens!(if has_meaning(&cs) { cs } else { T_CS!("\\pgfkeys@relax") }))
    }, locked => true);
  }
});
