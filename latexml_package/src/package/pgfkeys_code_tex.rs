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
// The loop keeps the raw chain's SHAPE, token for token: the list sits FLAT in
// the stream as `<items>,\pgfkeys@mainstop`, `\pgfkeys@parse` (native, the
// raw macro's name) reads ONE item up to its top-level `,`, puts the item's
// handler invocation back followed by `\pgfkeys@parse` again — exactly
// `\pgfkeys@@normal#1,` expanding to `<handler of #1> \pgfkeys@parse` (:359).
// The main loop therefore digests each handler body inline, where it can open
// a box or a group that later stream tokens close (zx-calculus's
// `/tikz/on layer/.code={\pgfonlayer{#1}\begingroup\aftergroup…}`,
// tikzlibraryzx-calculus.code.tex:2415-2419, opens an `\hbox` the path's own
// `\endgroup` closes; a nested `digest` of that body would hit the end of its
// mouth inside the box), a handler that scans FORWARD sees the real remaining
// keys behind `\pgfkeys@parse` as it does under raw (robust-externalize's
// placeholder re-scan captured a Rust-side continuation token instead and
// dispatched it as the key `/robExt/\lx@pgfkeys@continue`, 190 errors), and
// a `/.cd`, a key defined mid-list, or a re-entrant `\pgfkeysalso` inside a
// `.code` body see the same state at the same time as under the raw chain.
// The raw file still owns every handler (`/handlers/<h>/.@cmd`), the error
// keys, filtering, families and first-char syntax handlers: a list under
// filtering or a reconfigured case-three dispatch (the two probes below) is
// handed to the raw step body (`\futurelet…\pgfkeys@parse@main`, :326) — at
// every step, since a `.code` body may switch filtering on; first-char syntax
// handlers are dispatched natively.

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

/// `\pgfkeys@spdef` (pgfkeys.code.tex:506-519) as the raw chain runs it in
/// THIS engine: every leading space and one trailing space dropped, with the
/// delimited-argument brace stripping its two helper macros perform along
/// the way. `\pgfkeys@sp@b`'s parameter text starts with a space, and the
/// gullet matches a parameter-text-initial space against the whole run
/// (Perl identical) — real TeX would keep the second space and fail on
/// `/tcb/ title` where a `\newtcolorbox` body's `, #1` meets an indented
/// `[<newline> title=…]` (neoschool.tex:469); Perl is clean there.
fn spdef(mut v: Vec<Token>) -> Vec<Token> {
  while v.first().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
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
/// loop, ahead of the list's continuation. The raw sites load `\pgfkeys@code`
/// through `\pgfkeysgetvalue` (:175-178), which lets it to `\relax` when the
/// handler is absent, so `\pgfkeys@unknown` (:494-503) no-ops while
/// `/handlers/.unknown` is not yet defined (a rawstyles pgf load reaches
/// `\pgfkeys{/pgf/.is family}` before the handlers exist, pgfsys.code.tex:19;
/// scsnowman-sample: 815 undefined-`\pgfkeys@code` errors).
fn run_handler(cs: &Token) {
  if has_meaning(cs) {
    let_i(&T_CS!("\\pgfkeys@code"), cs, None);
  } else {
    let_i(&T_CS!("\\pgfkeys@code"), &T_CS!("\\relax"), None);
  }
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

/// `\pgfkeys@split@path` (:547-570): the key's `/`-segments walk through
/// `\pgfkeys@splitter#1/#2/` with `//` appended; the first segment followed by
/// an EMPTY one is `\pgfkeyscurrentname`, the segments before it (joined, the
/// trailing `/` removed) are `\pgfkeys@pathtoks`, and whatever text follows
/// the empty segment stays in the stream — a key with an empty segment
/// (`/p/q/`, `/p//q`) leaves its surplus behind, typeset before the handler
/// runs, in pdflatex too. The ORIGINAL tokens, with their catcodes: tikz-cd's
/// direction parser compares `\pgfkeyscurrentname`'s first character with
/// the letter `r` by `\ifx` (tikzlibrarycd.code.tex:96-98), which a
/// catcode-12 rebuild would fail. Returns (name, path tokens, leftover).
fn split_path(key_toks: &[Token]) -> Result<(String, Vec<Token>, Vec<Token>)> {
  // Segments at brace depth 0 (`#1/#2/` are delimited arguments), plus the
  // two appended empty ones.
  let mut segs: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0;
  for t in key_toks {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && is_char_of_catcode_other(t, '/') {
      segs.push(Vec::new());
    } else {
      segs.last_mut().unwrap().push(*t);
    }
  }
  // `#1`/`#2` are delimited arguments: a segment that is exactly one brace
  // group loses its braces, and a `#2` is re-supplied to the splitter, so a
  // `/` the braces hid then delimits (`/a/{x/y}/c` walks as `/a/x/y/c`).
  segs[0] = strip_single_group(std::mem::take(&mut segs[0]));
  let mut i = 1;
  while i < segs.len() {
    let seg = &segs[i];
    let braced = seg.len() >= 2
      && seg[0].get_catcode() == Catcode::BEGIN
      && seg[seg.len() - 1].get_catcode() == Catcode::END;
    if braced {
      let stripped = strip_single_group(segs[i].clone());
      if stripped.len() != seg.len() {
        let mut parts: Vec<Vec<Token>> = vec![Vec::new()];
        let mut d = 0;
        for t in &stripped {
          match t.get_catcode() {
            Catcode::BEGIN => d += 1,
            Catcode::END => d -= 1,
            _ => {},
          }
          if d == 0 && is_char_of_catcode_other(t, '/') {
            parts.push(Vec::new());
          } else {
            parts.last_mut().unwrap().push(*t);
          }
        }
        segs.splice(i..=i, parts);
        continue; // the re-supplied segment is examined again
      }
    }
    i += 1;
  }
  segs.push(Vec::new());
  segs.push(Vec::new());
  let k = (0..segs.len() - 1)
    .find(|&i| segs[i + 1].is_empty())
    .unwrap_or(segs.len() - 2);
  let slash = || Token::new("/", Catcode::OTHER);
  let join = |parts: &[Vec<Token>]| -> Vec<Token> {
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
      if i > 0 {
        out.push(slash());
      }
      out.extend_from_slice(p);
    }
    out
  };
  let name = segs[k].clone();
  let path = join(&segs[..k]);
  let leftover = if k + 2 < segs.len() {
    join(&segs[k + 2..])
  } else {
    Vec::new()
  };
  let mut toks = vec![T_CS!("\\pgfkeys@pathtoks"), T_BEGIN!()];
  toks.extend_from_slice(&path);
  toks.push(T_END!());
  digest(Tokens::new(toks))?;
  def_verbatim(T_CS!("\\pgfkeyscurrentname"), Tokens::new(name.clone()))?;
  Ok((key_text(&Tokens::new(name)), path, leftover))
}

// ---------------------------------------------------------------------------
// Slice 2: the definition handlers `.code`, `.style`, `.initial`, `.default`
// and `.cd` (pgfkeys.code.tex:772, :826, :842, :852, :994) natively, storing
// exactly what the raw handlers store — and `\pgfkeysdef` (:648-652) under
// them. These are the corpus's definition load: 72 % of a ZX circuit's
// dispatches, every `\tikzset`/`\tcbset` style library at package load. A
// native handler runs only while the handler key still holds the raw file's
// definition, checked against the `\let` snapshot `\lx@pgfkeys@raw@<h>` taken
// when the file loaded, so a package that redefines `/handlers/.code` gets its
// own. `\pgfkeysedef`, the `args` forms, `.add code`/`.append style` and every
// other handler stay raw.

const NATIVE_HANDLERS: [&str; 5] = [".code", ".style", ".initial", ".default", ".cd"];

fn raw_handler_snapshot(name: &str) -> Token { T_CS!(&s!("\\lx@pgfkeys@raw@{name}")) }

/// `\pgfkeysdef{key}{code}` (:648-652): `\pgfk@<key>/.@cmd` is a `\long`
/// macro with the parameter text `#1\pgfeov` and the code as its body
/// (parameter-packed, as `\def` reads it: `#1` a reference, `##` a `#`);
/// `\pgfk@<key>/.@body` holds the code verbatim for `.add code`/`.show code`.
fn pgfkeysdef(path: &[Token], code: Vec<Token>) -> Result<()> {
  let key = key_text(&Tokens::new(path.to_vec()));
  let cmd = T_CS!(&s!("\\pgfk@{key}/.@cmd"));
  let params = parse_def_parameters(
    &cmd,
    Tokens!(
      Token::new("#", Catcode::PARAM),
      Token::new("1", Catcode::OTHER),
      T_CS!("\\pgfeov")
    ),
  )?;
  def_macro(
    cmd,
    params,
    ExpansionBody::Tokens(Tokens::new(code.clone())),
    Some(ExpandableOptions {
      long: true,
      ..ExpandableOptions::default()
    }),
  )?;
  store_value(T_CS!(&s!("\\pgfk@{key}/.@body")), Tokens::new(code))
}

/// A native definition handler for the current key, if `name` is one and the
/// handler key still holds the raw definition. `path` is
/// `\pgfkeyscurrentpath`'s tokens. Returns whether it ran.
fn native_handler(name: &str, handler: &Token, path: &[Token]) -> Result<bool> {
  if !NATIVE_HANDLERS.contains(&name)
    || !matches!(
      (lookup_meaning(handler), lookup_meaning(&raw_handler_snapshot(name))),
      (Some(a), Some(b)) if a == b
    )
  {
    return Ok(false);
  }
  // The handler's `#1\pgfeov` argument: `\pgfkeyscurrentvalue` expanded once,
  // a single outer group stripped (a delimited argument). The literal braces
  // around `#1` in every handler body (`{#1}`) are the group of the next
  // undelimited argument, so nothing more is stripped.
  let arg = strip_single_group(
    stored_value(&T_CS!("\\pgfkeyscurrentvalue"))?
      .unwrap_or_default()
      .unlist(),
  );
  let key = key_text(&Tokens::new(path.to_vec()));
  match name {
    // :772 `\pgfkeysdef{\pgfkeyscurrentpath}{#1}`
    ".code" => pgfkeysdef(path, arg)?,
    // :826 `\pgfkeys{\pgfkeyscurrentpath/.code=\pgfkeysalso{#1}}` — the nested
    // list's one item, `<path>/.code`, is what the raw handler leaves in
    // `\pgfkeyscurrentkey`/`RAW`/`name`/`value` afterwards.
    ".style" => {
      let mut code = vec![T_CS!("\\pgfkeysalso"), T_BEGIN!()];
      code.extend(arg);
      code.push(T_END!());
      let mut nested_key = path.to_vec();
      nested_key.extend(other_chars("/.code"));
      def_verbatim(
        T_CS!("\\pgfkeyscurrentkeyRAW"),
        Tokens::new(nested_key.clone()),
      )?;
      def_verbatim(T_CS!("\\pgfkeyscurrentkey"), Tokens::new(nested_key))?;
      let_i(
        &T_CS!("\\ifpgfkeysaddeddefaultpath"),
        &T_CS!("\\iffalse"),
        None,
      );
      def_verbatim(
        T_CS!("\\pgfkeyscurrentname"),
        Tokens::new(other_chars(".code").collect()),
      )?;
      def_verbatim(T_CS!("\\pgfkeyscurrentvalue"), Tokens::new(code.clone()))?;
      pgfkeysdef(path, code)?;
    },
    // :842 `\pgfkeyssetvalue{\pgfkeyscurrentpath}{#1}`
    ".initial" => store_value(T_CS!(&s!("\\pgfk@{key}")), Tokens::new(arg))?,
    // :852 `\pgfkeyssetvalue{\pgfkeyscurrentpath/.@def}{#1}`
    ".default" => store_value(T_CS!(&s!("\\pgfk@{key}/.@def")), Tokens::new(arg))?,
    // :994 `\edef\pgfkeysdefaultpath{\pgfkeyscurrentpath/}`
    ".cd" => {
      let mut newpath = path.to_vec();
      newpath.push(Token::new("/", Catcode::OTHER));
      def_verbatim(T_CS!("\\pgfkeysdefaultpath"), Tokens::new(newpath))?;
    },
    _ => return Ok(false),
  }
  Ok(true)
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
/// Returns whether the key was dispatched (an empty key is skipped, :372).
fn dispatch_item(item: &[Token]) -> Result<bool> {
  let (raw_key, value) = unpack(item);
  let key_toks = Expand!(Tokens::new(spdef(raw_key)));
  let key_str = key_text(&key_toks);
  if key_str.is_empty() {
    return Ok(false);
  }
  // The inner conditional's `\fi` (:387), behind the handler tokens.
  unread(Tokens!(T_CS!("\\fi")));
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
    return Ok(true);
  }
  // Case one: a command key.
  let cmd = T_CS!(&s!("\\pgfk@{full_key}/.@cmd"));
  if has_meaning(&cmd) {
    if lookup_meaning(&cmd) == lookup_meaning(&T_CS!("\\relax")) {
      trace(&full_key, "case one, relax -> unknown");
      run_unknown()?;
      return Ok(true);
    }
    trace(&full_key, "case one");
    run_handler(&cmd);
    return Ok(true);
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
    return Ok(true);
  }
  // Case three: a handler named by the last path component.
  let (name, path, leftover) = split_path(&full_key_toks)?;
  let handler = T_CS!(&s!("\\pgfk@/handlers/{name}/.@cmd"));
  if has_meaning(&handler) {
    if native_handler(&name, &handler, &path)? {
      trace(&full_key, &s!("case three, native {name}"));
    } else {
      trace(&full_key, &s!("case three, handler {name}"));
      run_handler(&handler);
    }
  } else {
    trace(&full_key, &s!("unknown (name {name})"));
    run_unknown()?;
  }
  // The splitter's surplus text sits in front of the handler tokens (:550).
  if !leftover.is_empty() {
    unread(Tokens::new(leftover));
  }
  Ok(true)
}

/// `\pgfkeys@syntax@handlers` (:340-357): with first-char syntax on
/// (zx-calculus's `?…`/`!…` items) every leading space of the item is skipped
/// first (`\pgf@keys@utilifnextchar`, an `\@ifnextchar`, :340), then the
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

/// The next item of the list in the stream: the tokens up to the first
/// top-level `,` (`\pgfkeys@@normal#1,`, unexpanded, brace groups hiding the
/// delimiter), or `None` when `\pgfkeys@mainstop` is next (the list is done,
/// the sentinel consumed — `\pgfkeys@parse@main`, :328-333).
fn read_item() -> Result<Option<Vec<Token>>> {
  let mut item = Vec::new();
  let mut depth = 0;
  while let Some(t) = read_token()? {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 {
      if is_char_of_catcode_other(&t, ',') {
        return Ok(Some(item));
      }
      if item.is_empty() && is_cs_named(&t, "\\pgfkeys@mainstop") {
        return Ok(None);
      }
    }
    item.push(t);
  }
  Ok(Some(item))
}

/// The characters of `s` as OTHER-catcode tokens (a `\csname` name).
fn other_chars(s: &str) -> impl Iterator<Item = Token> + '_ {
  s.chars().map(|c| Token::new(c.to_string(), Catcode::OTHER))
}

/// `<list>,\pgfkeys@mainstop` into the stream (:323, :590, :607).
fn unread_list(list: Tokens) {
  let mut toks = list.unlist();
  toks.push(Token::new(",", Catcode::OTHER));
  toks.push(T_CS!("\\pgfkeys@mainstop"));
  unread(Tokens::new(toks));
}

/// One step of `\pgfkeys@parse` (:326-333): the next item read from the
/// stream and dispatched, its handler tokens followed by `\pgfkeys@parse`
/// again for the rest of the list.
fn parse_step() -> Result<()> {
  if raw_loop_needed() {
    // The raw step's own body (:326); the list is already in the stream.
    unread(Tokens!(
      T_CS!("\\futurelet"),
      T_CS!("\\pgfkeys@possiblerelax"),
      T_CS!("\\pgfkeys@parse@main")
    ));
    return Ok(());
  }
  let Some(item) = read_item()? else {
    return Ok(());
  };
  unread(Tokens!(T_CS!("\\pgfkeys@parse")));
  let (item, handler) = syntax_handlers(item)?;
  if let Some(handler) = handler {
    let_i(&T_CS!("\\pgfkeys@the@handler"), &handler, None);
    let mut toks = vec![T_CS!("\\pgfkeys@the@handler"), T_BEGIN!()];
    toks.extend(strip_single_group(item));
    toks.push(T_END!());
    unread(Tokens::new(toks));
    return Ok(());
  }
  // `\pgfkeys@unpack` runs the case dispatch inside TWO open conditionals,
  // `\ifx\pgfkeyscurrentkey\pgfkeys@empty…\else … \fi` (:372/:388) and, in
  // its else branch, `\ifx\pgfkeyscurrentvalue\pgfkeysvaluerequired…\else …
  // \fi` (:382/:387), so the tokens that trail a handler body are `\fi\fi`
  // (one `\fi` after an empty key's skip), with `\pgfkeys@parse` beyond: a
  // handler that over-grabs a token takes an inert `\fi`. robust-externalize's
  // `\robExtArgumentList` m-grab (sty:4230) does, and detokenizes it; a grabbed
  // continuation became the key `/robExt/\lx@pgfkeys@set{@}parse` and a
  // recursion Fatal.
  unread(Tokens!(T_CS!("\\fi")));
  let dispatched = dispatch_item(&item)?;
  unread(Tokens!(T_CS!("\\iftrue")));
  if dispatched {
    unread(Tokens!(T_CS!("\\iftrue")));
  }
  Ok(())
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
    // Slice 2's guard: the raw definition of each natively handled handler,
    // as loaded, so a later redefinition is honored.
    for h in NATIVE_HANDLERS {
      let_i(&raw_handler_snapshot(h), &T_CS!(&s!("\\pgfk@/handlers/{h}/.@cmd")), None);
    }
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
    // The three entry points are MACROS, as in the raw file (:320, :589,
    // :607): their list is read as a macro argument — a group-closing `}` in
    // its place is refused and left (`{{\tikzset}\marg{options}}`,
    // sa-tikz-doc.tex:336, where `\tikzset` is `\pgfqkeys{/tikz}`), which a
    // primitive's `{}` parameter would consume — and handed, braced, to the
    // internal primitive that runs the list.
    // :318-323 `\pgfkeys`: path reset to `/` for the list, restored after.
    DefMacro!("\\pgfkeys{}", sub[(list)] {
      Ok(Tokens!(T_CS!("\\lx@pgfkeys@set"), T_BEGIN!(), list, T_END!()))
    }, locked => true);
    DefPrimitive!("\\lx@pgfkeys@set{}", sub[(list)] {
      if raw_loop_needed() {
        return Ok(raw_pgfkeys(vec![T_CS!("\\expandafter"), T_CS!("\\pgfkeys@@set"), T_CS!("\\expandafter"), T_BEGIN!(), T_CS!("\\pgfkeysdefaultpath"), T_END!()], list));
      }
      let saved = Expand!(Tokens!(T_CS!("\\pgfkeysdefaultpath")));
      let_i(&T_CS!("\\pgfkeysdefaultpath"), &T_CS!("\\pgfkeys@root"), None);
      unread_path_restore(saved);
      unread_list(list);
      parse_step()?;
    }, locked => true);
    // :607 `\pgfkeysalso`: the current path kept.
    DefMacro!("\\pgfkeysalso{}", sub[(list)] {
      Ok(Tokens!(T_CS!("\\lx@pgfkeys@also"), T_BEGIN!(), list, T_END!()))
    }, locked => true);
    DefPrimitive!("\\lx@pgfkeys@also{}", sub[(list)] {
      unread_list(list);
      parse_step()?;
    }, locked => true);
    // :589-590 `\pgfqkeys{path}{list}`: path `#1/` for the list, restored after.
    DefMacro!("\\pgfqkeys{}{}", sub[(path, list)] {
      Ok(Tokens!(T_CS!("\\lx@pgfkeys@qset"), T_BEGIN!(), path, T_END!(), T_BEGIN!(), list, T_END!()))
    }, locked => true);
    DefPrimitive!("\\lx@pgfkeys@qset{}{}", sub[(path, list)] {
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
      unread_list(list);
      parse_step()?;
    }, locked => true);
    // :326 the step itself, under the raw macro's own name so the stream a
    // handler may scan is the raw stream; the raw `\pgfkeysalsofrom` and
    // `\pgfkeysalsofiltered` (:610-620) reach it too.
    DefPrimitive!("\\pgfkeys@parse", sub[()] {
      parse_step()?;
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
