//! pgfkeys.code.tex — the raw key engine plus a native dispatch: the leaf
//! accessors (slice 0) and the `\pgfkeys{}` parse-and-dispatch loop (slice 1)
//! of `docs/performance/PERFORMANCE.md`'s native pgfkeys program.
//!
//! The whole raw file loads first, so every handler (`/handlers/*/.@cmd`),
//! the error keys, filtering and families are the real TeX; the natives below
//! replace the leaf accessors every key dispatch runs (`\pgfkeys@ifcsname`
//! 23,650 calls on one ZX circuit, `\pgfkeysgetvalue` and `\pgfkeysifdefined`
//! as many) and the loop itself, on the SAME `\pgfk@<key>` control-sequence
//! storage the raw code uses (pgfkeys.code.tex:101/155/173/213) — so `\tikzset`,
//! `\tcbset`, pgfplots and forest, which poke that storage directly, see one
//! store. Perl's own native engine (`pgfkeys.code.tex.ltxml:40-544`,
//! `pgfkeyCS` + `DefMacroI`) is the anchor; it is disabled there as incomplete,
//! which this hybrid avoids by never replacing a handler. `\pgfkeyssetvalue`
//! stores the value tokens verbatim (`\edef` of `\the\pgfkeys@temptoks`, :101 —
//! no expansion, `#` stays a parameter character), so the store is a
//! parameter-free macro whose body is not parameter-packed.
//!
//! `LATEXML_PGFKEYS_NATIVE=0` keeps the raw accessors and loop — the
//! differential harness (`cluster_package_guards::pgfkeys_native_accessors`)
//! converts every fixture both ways and requires byte-identical core XML;
//! `LATEXML_PGFKEYS_TRACE=1` prints one line per dispatched key.
use std::cell::RefCell;

use crate::prelude::*;

/// A key's text: a space token that came from an end of line carries `\n`
/// as its text, and a key name is a `\csname` — one spelling (Perl
/// `pgfkeyCS`, which normalizes the same way).
fn key_text(t: &Tokens) -> String { t.to_string().replace('\n', " ") }

/// The `\pgfk@<key>` control sequence of a key (Perl `pgfkeyCS`).
fn pgfkey_cs(key: &Tokens) -> Result<Token> {
  let key = key_text(&Expand!(key.clone()));
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

// ---------------------------------------------------------------------------
// Slice 1: the `\pgfkeys{}` / `\pgfkeysalso{}` / `\pgfqkeys{}{}` parse and
// dispatch loop (pgfkeys.code.tex:318-440, :494-520, :589-607), natively.
// The loop keeps the raw chain's SHAPE: one item is resolved per step, its
// handler invocation is put back into the token stream, and the rest of the
// list re-enters through `\lx@pgfkeys@continue{<slot>}` placed BEHIND those
// tokens
// — `\pgfkeys@@normal#1,` expanding to `<handler of #1> \pgfkeys@parse` (:328).
// The main loop therefore digests each handler body inline, where it can open
// a box or a group that later stream tokens close (zx-calculus's
// `/tikz/on layer/.code={\pgfonlayer{#1}\begingroup\aftergroup…}`,
// tikzlibraryzx-calculus.code.tex:2415-2419, opens an `\hbox` the path's own
// `\endgroup` closes; a nested `digest` of that body would hit the end of its
// mouth inside the box), and a `/.cd`, a key defined mid-list, or a
// re-entrant `\pgfkeysalso` inside a `.code` body see the same state at the
// same time as under the raw chain. The raw file still owns every handler
// (`/handlers/<h>/.@cmd`), the error keys, filtering, families and first-char
// syntax handlers: a list under filtering or a reconfigured case-three
// dispatch (the two probes below) is handed to the raw `\pgfkeys@parse`
// untouched — re-checked at every step, since a `.code` body may switch
// filtering on; first-char syntax handlers are dispatched natively.

/// Is `t` the character `c` with catcode OTHER — the only spelling of the
/// `,` and `=` delimiters the raw `\pgfkeys@@normal#1,` / `\pgfkeys@unpack`
/// parameter texts match (an active or letter-catcode `,` is not one).
fn is_char_of_catcode_other(t: &Token, c: char) -> bool {
  t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.len() == c.len_utf8() && s.starts_with(c))
}

fn is_cs_named(t: &Token, name: &str) -> bool {
  t.get_catcode() == Catcode::CS && t.with_str(|s| s == name)
}

/// TeX's delimited-parameter rule: an argument that is exactly one brace
/// group loses its outer braces.
fn strip_single_group(mut v: Vec<Token>) -> Vec<Token> {
  if v.len() >= 2
    && v[0].get_catcode() == Catcode::BEGIN
    && v[v.len() - 1].get_catcode() == Catcode::END
  {
    let mut depth = 0;
    for (i, t) in v.iter().enumerate() {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 && i != v.len() - 1 {
            return v; // the first group closes before the end
          }
        },
        _ => {},
      }
    }
    v.pop();
    v.remove(0);
  }
  v
}

/// Split at every top-level occurrence of the OTHER-catcode character `c`
/// (`\pgfkeys@@normal#1,` over the whole list): brace groups hide it.
fn split_top_level(v: &[Token], c: char) -> Vec<Vec<Token>> {
  let mut out = Vec::new();
  let mut cur = Vec::new();
  let mut depth = 0;
  for t in v {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && is_char_of_catcode_other(t, c) {
      out.push(std::mem::take(&mut cur));
    } else {
      cur.push(*t);
    }
  }
  out.push(cur);
  out
}

/// `\pgfkeys@unpack#1=#2=#3\pgfkeys@stop` on `item=\pgfkeysnovalue=`: the key
/// is the text before the first top-level `=`, the value the text between
/// the first and second (a value's own top-level `=` ends it, as in TeX);
/// each is a delimited argument (single-group stripping).
fn unpack(item: &[Token]) -> (Vec<Token>, Option<Vec<Token>>) {
  let parts = split_top_level(item, '=');
  let key = strip_single_group(parts[0].clone());
  let value = if parts.len() > 1 {
    Some(strip_single_group(parts[1].clone()))
  } else {
    None
  };
  (key, value)
}

/// `\pgfkeys@spdef` (pgfkeys.code.tex:506-519): one leading and one trailing
/// space dropped, with the delimited-argument brace stripping its two
/// helper macros perform along the way.
fn spdef(mut v: Vec<Token>) -> Vec<Token> {
  if v.first().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
    v.remove(0);
  }
  if v.last().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
    v.pop();
    // `\pgfkeys@sp@b`'s argument was exactly the value: stripped once there…
    v = strip_single_group(v);
  }
  // …and `\pgfkeys@sp@c#1\pgfkeys@stop` strips once more.
  strip_single_group(v)
}

fn def_verbatim(cs: Token, body: Tokens) -> Result<()> {
  def_macro(
    cs,
    None,
    ExpansionBody::Tokens(body),
    Some(ExpandableOptions {
      nopack_parameters: true,
      ..ExpandableOptions::default()
    }),
  )
}

fn newif_true(cs: &str) -> bool {
  matches!((lookup_meaning(&T_CS!(cs)), lookup_meaning(&T_CS!("\\iftrue"))), (Some(a), Some(b)) if a == b)
}

/// The two shapes the raw loop keeps: filtering active, or the case-three
/// dispatch reconfigured (`/handler config=only existing|full or existing`).
fn raw_loop_needed() -> bool {
  newif_true("\\ifpgfkeysfilteringisactive")
    || !matches!(
      (lookup_meaning(&T_CS!("\\pgfkeys@case@three")), lookup_meaning(&T_CS!("\\pgfkeys@case@three@handleall"))),
      (Some(a), Some(b)) if a == b
    )
}

/// `\expandafter\pgfkeys@code\pgfkeyscurrentvalue\pgfeov` with `\pgfkeys@code`
/// let to the handler under `cs` — put back into the stream for the main
/// loop, ahead of the list's continuation.
fn run_handler(cs: &Token) {
  let_i(&T_CS!("\\pgfkeys@code"), cs, None);
  unread(Tokens!(
    T_CS!("\\expandafter"),
    T_CS!("\\pgfkeys@code"),
    T_CS!("\\pgfkeyscurrentvalue"),
    T_CS!("\\pgfeov")
  ));
}

/// `\pgfkeys@unknown` (:494-504).
fn run_unknown() -> Result<()> {
  let path = key_text(&Expand!(Tokens!(T_CS!("\\pgfkeyscurrentpath"))));
  let local = T_CS!(&s!("\\pgfk@{path}/.unknown/.@cmd"));
  if has_meaning(&local) {
    run_handler(&local);
  } else {
    run_handler(&T_CS!("\\pgfk@/handlers/.unknown/.@cmd"));
  }
  Ok(())
}

/// `\pgfkeys@split@path` (:547-570): `\pgfkeys@pathtoks` = the key's tokens up
/// to the last `/`, `\pgfkeyscurrentname` = the tokens after it. The ORIGINAL
/// tokens, with their catcodes: tikz-cd's direction parser compares
/// `\pgfkeyscurrentname`'s first character with the letter `r` by `\ifx`
/// (tikzlibrarycd.code.tex:96-98), which a catcode-12 rebuild would fail.
fn split_path(key_toks: &[Token]) -> Result<String> {
  let cut = key_toks
    .iter()
    .rposition(|t| is_char_of_catcode_other(t, '/'));
  let (path, name): (&[Token], &[Token]) = match cut {
    Some(i) => (&key_toks[..i], &key_toks[i + 1..]),
    None => (&[], key_toks),
  };
  let mut toks = vec![T_CS!("\\pgfkeys@pathtoks"), T_BEGIN!()];
  toks.extend_from_slice(path);
  toks.push(T_END!());
  digest(Tokens::new(toks))?;
  def_verbatim(T_CS!("\\pgfkeyscurrentname"), Tokens::new(name.to_vec()))?;
  Ok(key_text(&Tokens::new(name.to_vec())))
}

fn is_single_cs(v: &Tokens, name: &str) -> bool {
  let t = v.unlist_ref();
  t.len() == 1 && is_cs_named(&t[0], name)
}

/// `LATEXML_PGFKEYS_TRACE=1`: one stderr line per dispatched key (key, the
/// case taken) — the bisection aid for a native/raw divergence.
static TRACE: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_PGFKEYS_TRACE").is_some());

fn trace(key: &str, what: &str) {
  if *TRACE {
    eprintln!("PGFKEYS {what}: {key}");
  }
}

/// One key of the list (`\pgfkeys@unpack` :364-388 onward).
fn dispatch_item(item: &[Token]) -> Result<()> {
  let (raw_key, value) = unpack(item);
  let key_toks = Expand!(Tokens::new(spdef(raw_key)));
  let key_str = key_text(&key_toks);
  if key_str.is_empty() {
    return Ok(());
  }
  // \pgfkeys@add@path@as@needed — the key's own tokens are kept (catcodes
  // and all) for `\pgfkeyscurrentkey`/`RAW` and the path split; the string
  // only names control sequences.
  def_verbatim(T_CS!("\\pgfkeyscurrentkeyRAW"), key_toks.clone())?;
  let (full_key, full_key_toks) = if key_str.starts_with('/') {
    let_i(
      &T_CS!("\\ifpgfkeysaddeddefaultpath"),
      &T_CS!("\\iffalse"),
      None,
    );
    (key_str, key_toks.unlist())
  } else {
    let_i(
      &T_CS!("\\ifpgfkeysaddeddefaultpath"),
      &T_CS!("\\iftrue"),
      None,
    );
    let path_toks = Expand!(Tokens!(T_CS!("\\pgfkeysdefaultpath")));
    let path = key_text(&path_toks);
    let mut toks = path_toks.unlist();
    toks.extend(key_toks.unlist());
    (s!("{path}{key_str}"), toks)
  };
  def_verbatim(
    T_CS!("\\pgfkeyscurrentkey"),
    Tokens::new(full_key_toks.clone()),
  )?;
  let cs = T_CS!(&s!("\\pgfk@{full_key}"));
  // The value: verbatim, or the key's `.@def` when none was given.
  let novalue = match &value {
    None => true,
    Some(v) => v.len() == 1 && is_cs_named(&v[0], "\\pgfkeysnovalue"),
  };
  let mut value_is_required = false;
  if novalue {
    let def_cs = T_CS!(&s!("\\pgfk@{full_key}/.@def"));
    if has_meaning(&def_cs) {
      let_i(&T_CS!("\\pgfkeyscurrentvalue"), &def_cs, None);
      value_is_required =
        stored_value(&def_cs)?.is_some_and(|v| is_single_cs(&v, "\\pgfkeysvaluerequired"));
    } else {
      def_verbatim(
        T_CS!("\\pgfkeyscurrentvalue"),
        Tokens!(T_CS!("\\pgfkeysnovalue")),
      )?;
    }
  } else {
    def_verbatim(
      T_CS!("\\pgfkeyscurrentvalue"),
      Tokens::new(spdef(value.unwrap_or_default())),
    )?;
  }
  if value_is_required {
    // :380-382: the `/errors/value required` key with `{key}{}`.
    unread(mouth::tokenize_internal(
      r"\def\pgf@marshal{\pgfkeysvalueof{/errors/value required/.@cmd}}\expandafter\pgf@marshal\expandafter{\pgfkeyscurrentkey}{}\pgfeov",
    ));
    return Ok(());
  }
  // Case one: a command key.
  let cmd = T_CS!(&s!("\\pgfk@{full_key}/.@cmd"));
  if has_meaning(&cmd) {
    if lookup_meaning(&cmd) == lookup_meaning(&T_CS!("\\relax")) {
      trace(&full_key, "case one, relax -> unknown");
      return run_unknown();
    }
    trace(&full_key, "case one");
    run_handler(&cmd);
    return Ok(());
  }
  // Case two: a stored value.
  if has_meaning(&cs) {
    trace(&full_key, "case two");
    let now_novalue = stored_value(&T_CS!("\\pgfkeyscurrentvalue"))?
      .is_some_and(|v| is_single_cs(&v, "\\pgfkeysnovalue"));
    if now_novalue {
      // `\csname\pgfkeyscurrentkey\endcsname` (:398): the stored value expands
      // in the stream.
      unread(Tokens!(cs));
    } else {
      let_i(&cs, &T_CS!("\\pgfkeyscurrentvalue"), None);
    }
    return Ok(());
  }
  // Case three: a handler named by the last path component.
  let name = split_path(&full_key_toks)?;
  let handler = T_CS!(&s!("\\pgfk@/handlers/{name}/.@cmd"));
  if has_meaning(&handler) {
    trace(&full_key, &s!("case three, handler {name}"));
    run_handler(&handler);
    return Ok(());
  }
  trace(&full_key, &s!("unknown (name {name})"));
  run_unknown()
}

/// `\pgfkeys@syntax@handlers` (:340-357): with first-char syntax on
/// (zx-calculus's `?…`/`!…` items) EVERY leading space of the item is skipped
/// first (`\pgf@keys@utilifnextchar`, an `\@ifnextchar`, :340 — where the
/// plain path's `\pgfkeys@spdef` drops exactly one, so a `\wrap{ a=1}` whose
/// body writes `, #1` reaches the key ` a` under pdflatex too), then the
/// `\meaning` of the first token selects `/handlers/first char syntax/<meaning>`;
/// when that key holds a handler, the whole item goes to it
/// (`\pgfkeys@use@handler`) instead of the key=value path. Returns the
/// space-skipped item and the handler, if any.
fn syntax_handlers(item: Vec<Token>) -> Result<(Vec<Token>, Option<Token>)> {
  if !newif_true("\\ifpgfkeys@syntax@handlers") {
    return Ok((item, None));
  }
  let skip = item
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  let item = item[skip..].to_vec();
  let Some(first) = item.first() else {
    return Ok((item, None));
  };
  let meaning = key_text(&Expand!(Tokens!(T_CS!("\\meaning"), *first)));
  let cs = T_CS!(&s!("\\pgfk@/handlers/first char syntax/{meaning}"));
  let handler = if has_meaning(&cs) { Some(cs) } else { None };
  Ok((item, handler))
}

/// The first top-level `,`-delimited item of `list[from..]` and where the
/// rest starts (`\pgfkeys@@normal#1,`): `None` when no further comma
/// follows, so this item is the last.
fn split_first(list: &[Token], from: usize) -> (Vec<Token>, Option<usize>) {
  let mut depth = 0;
  for (i, t) in list.iter().enumerate().skip(from) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && is_char_of_catcode_other(t, ',') {
      return (list[from..i].to_vec(), Some(i + 1));
    }
  }
  (list[from..].to_vec(), None)
}

/// A list whose dispatch is in progress: its tokens, held once, and the
/// offset of its next item.
type PendingList = (Vec<Token>, usize);

/// The in-progress lists, one slot per `\pgfkeys`-style call, plus the free
/// slot ids.
#[derive(Default)]
struct PendingLists {
  slots: Vec<Option<PendingList>>,
  free:  Vec<usize>,
}

thread_local! {
  /// The lists whose dispatch is in progress. The stream carries only
  /// `\lx@pgfkeys@continue{<slot>}` between items — the raw chain's
  /// `\pgfkeys@parse` continuation, without its re-scan of the remaining
  /// list at every step (a braced `{rest}` argument re-read per item cost
  /// tcolorbox's long `\tcbset` lists +7 % instructions). A slot is freed
  /// when its list is exhausted or handed to the raw loop; one whose
  /// continuation token a broken `.code` body swallows as an argument (the
  /// raw chain's `\pgfkeys@parse` would be swallowed the same way) stays
  /// allocated — a leak of one list, never a wrong resumption, since ids
  /// are never reused while live.
  static PENDING: RefCell<PendingLists> = RefCell::new(PendingLists::default());
}

fn pending_alloc(list: Vec<Token>, offset: usize) -> usize {
  PENDING.with_borrow_mut(|p| {
    if p.free.len() == p.slots.len() {
      // Nothing live: a fresh table, so a slot leaked by a swallowed
      // continuation (above) is reclaimed at the next top-level call rather
      // than kept for the process, across conversions.
      p.slots.clear();
      p.free.clear();
    }
    if let Some(id) = p.free.pop() {
      p.slots[id] = Some((list, offset));
      id
    } else {
      p.slots.push(Some((list, offset)));
      p.slots.len() - 1
    }
  })
}

fn pending_take(id: usize) -> Option<PendingList> {
  PENDING.with_borrow_mut(|p| {
    let entry = p.slots.get_mut(id).and_then(Option::take);
    if entry.is_some() {
      p.free.push(id);
    }
    entry
  })
}

/// The characters of `s` as OTHER-catcode tokens (a `\csname` name).
fn other_chars(s: &str) -> impl Iterator<Item = Token> + '_ {
  s.chars().map(|c| Token::new(c.to_string(), Catcode::OTHER))
}

/// `\lx@pgfkeys@continue{<slot>}` behind the current item's handler tokens.
fn unread_continuation(id: usize) {
  let mut toks = vec![T_CS!("\\lx@pgfkeys@continue"), T_BEGIN!()];
  toks.extend(other_chars(&id.to_string()));
  toks.push(T_END!());
  unread(Tokens::new(toks));
}

/// One step of `\pgfkeys@parse … ,\pgfkeys@mainstop` (:328-333): the item at
/// `from` dispatched, the rest of the list re-entering behind its handler
/// tokens through `\lx@pgfkeys@continue{<slot>}`.
fn parse_step(list: Vec<Token>, from: usize) -> Result<()> {
  if raw_loop_needed() {
    let mut toks = vec![T_CS!("\\pgfkeys@parse")];
    toks.extend_from_slice(&list[from..]);
    toks.push(Token::new(",", Catcode::OTHER));
    toks.push(T_CS!("\\pgfkeys@mainstop"));
    unread(Tokens::new(toks));
    return Ok(());
  }
  let (item, rest) = split_first(&list, from);
  if let Some(next) = rest {
    unread_continuation(pending_alloc(list, next));
  }
  let (item, handler) = syntax_handlers(item)?;
  if let Some(handler) = handler {
    let_i(&T_CS!("\\pgfkeys@the@handler"), &handler, None);
    let mut toks = vec![T_CS!("\\pgfkeys@the@handler"), T_BEGIN!()];
    toks.extend(strip_single_group(item));
    toks.push(T_END!());
    unread(Tokens::new(toks));
    return Ok(());
  }
  dispatch_item(&item)
}

/// The `\def\pgfkeysdefaultpath{#1}` a `\pgfkeys`/`\pgfqkeys` list leaves
/// behind its `\pgfkeys@mainstop` (:323, :590), as a stream token.
fn unread_path_restore(saved: Tokens) {
  let mut toks = vec![T_CS!("\\lx@pgfkeys@restorepath"), T_BEGIN!()];
  toks.extend(saved.unlist());
  toks.push(T_END!());
  unread(Tokens::new(toks));
}

/// The raw loop, untouched, for the shapes the native path does not own.
fn raw_pgfkeys(prefix: Vec<Token>, list: Tokens) -> Vec<Digested> {
  let mut toks = prefix;
  toks.push(T_BEGIN!());
  toks.extend(list.unlist());
  toks.push(T_END!());
  unread(Tokens::new(toks));
  Vec::new()
}

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("pgfkeys.code", extension => Some(Cow::Borrowed("tex")), noltxml => true, reloadable => true);
  if std::env::var("LATEXML_PGFKEYS_NATIVE").as_deref() != Ok("0") {
    // :29 — the branch selector every case-dispatch step runs.
    DefMacro!("\\pgfkeys@ifcsname{}", sub[(name)] {
      let name = key_text(&Expand!(name));
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
      let key_str = key_text(&Expand!(key));
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
    // ----- slice 1: the dispatch loop -----
    // :318-323 `\pgfkeys`: path reset to `/` for the list, restored after.
    DefPrimitive!("\\pgfkeys{}", sub[(list)] {
      if raw_loop_needed() {
        return Ok(raw_pgfkeys(vec![T_CS!("\\expandafter"), T_CS!("\\pgfkeys@@set"), T_CS!("\\expandafter"), T_BEGIN!(), T_CS!("\\pgfkeysdefaultpath"), T_END!()], list));
      }
      let saved = Expand!(Tokens!(T_CS!("\\pgfkeysdefaultpath")));
      let_i(&T_CS!("\\pgfkeysdefaultpath"), &T_CS!("\\pgfkeys@root"), None);
      unread_path_restore(saved);
      parse_step(list.unlist(), 0)?;
    }, locked => true);
    // :607 `\pgfkeysalso`: the current path kept.
    DefPrimitive!("\\pgfkeysalso{}", sub[(list)] {
      parse_step(list.unlist(), 0)?;
    }, locked => true);
    // :589-590 `\pgfqkeys{path}{list}`: path `#1/` for the list, restored after.
    DefPrimitive!("\\pgfqkeys{}{}", sub[(path, list)] {
      if raw_loop_needed() {
        let mut prefix = vec![T_CS!("\\expandafter"), T_CS!("\\pgfkeys@@qset"), T_CS!("\\expandafter"), T_BEGIN!(), T_CS!("\\pgfkeysdefaultpath"), T_END!(), T_BEGIN!()];
        prefix.extend(path.unlist());
        prefix.push(T_END!());
        return Ok(raw_pgfkeys(prefix, list));
      }
      let saved = Expand!(Tokens!(T_CS!("\\pgfkeysdefaultpath")));
      let mut newpath = path.unlist();
      newpath.push(Token::new("/", Catcode::OTHER));
      def_verbatim(T_CS!("\\pgfkeysdefaultpath"), Tokens::new(newpath))?;
      unread_path_restore(saved);
      parse_step(list.unlist(), 0)?;
    }, locked => true);
    // The loop's continuation and the path restore — internal, locked.
    DefPrimitive!("\\lx@pgfkeys@continue{}", sub[(slot)] {
      // The slot id is one this file wrote; anything else is a swallowed
      // and re-emitted continuation, which has no list to resume.
      if let Some((list, from)) = slot.to_string().trim().parse().ok().and_then(pending_take) {
        parse_step(list, from)?;
      }
    }, locked => true);
    DefPrimitive!("\\lx@pgfkeys@restorepath{}", sub[(saved)] {
      def_verbatim(T_CS!("\\pgfkeysdefaultpath"), saved)?;
    }, locked => true);
    // :193-194 — `\csname pgfk@<key>\endcsname` when the value's control
    // sequence exists, else the raw file's pre-defined `\pgfkeys@relax`: the
    // key itself is never `\csname`-defined (zx-calculus reads
    // `\pgfkeysvalueof{/zx/zx@isInFrac}` before the key exists and later
    // probes it with `\pgfkeysifdefined`). The `\csname` is kept, not
    // collapsed to the token: the raw macro reaches `\pgfk@<key>` in TWO
    // expansion steps, and circuitikz's `\unexpandedvalueof`
    // (circuitikz-1.7.2-body.tex:987-998) counts them —
    // `\expandafter\expandafter\expandafter\pgf@circ@valueof@chk\pgfkeysvalueof{#1}`
    // must hand the checker the cs token, not the first token of its body
    // (a one-step native leaked a flipflop's pin values into `\ifx…\fi`:
    // circuitikzmanual.tex:7749, a stray `\fi`).
    // Only the full expansion and the two-step depth are the raw macro's:
    // its one-step form is `\csname\pgfkeys@ifcsname{…}…\endcsname`, ours
    // the resolved name.
    DefMacro!("\\pgfkeysvalueof{}", sub[(key)] {
      let key = Expand!(key);
      let mut toks = vec![T_CS!("\\csname")];
      if has_meaning(&T_CS!(&s!("\\pgfk@{}", key_text(&key)))) {
        toks.extend(other_chars("pgfk@"));
        toks.extend(key.unlist());
      } else {
        toks.extend(other_chars("pgfkeys@relax"));
      }
      toks.push(T_CS!("\\endcsname"));
      Ok(Tokens::new(toks))
    }, locked => true);
  }
});
