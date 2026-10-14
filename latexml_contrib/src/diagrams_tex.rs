//! Paul Taylor's `diagrams` package (diagrams.tex v3.97, 2024-11-20; www.PaulTaylor.EU/diagrams,
//! CTAN macros/generic/diagrams/taylor). Not in TeX Live (its licence forbids alteration), but in
//! arXiv's TeX tree, so ~470 arXiv papers build with it (`\diagramstyle` 467, `\newarrow` 441,
//! `\rTo` 362 papers undefined in run 349). Perl's ar5iv-bindings `diagrams.tex.ltxml` discards
//! every `{diagram}` as `<ltx:ERROR>{diagram}</ltx:ERROR>`; this binding keeps the diagram, after
//! the model of amscd's `{CD}` (amscd_sty.rs): a math array of objects and labelled arrows
//! (OXIDIZED_DESIGN_DIVERGENCES #589). Witnesses: math0001049, math-ph0001006, math0003019,
//! math-ph0007001, math0010298, math0012073, math0102115, math0104065, math0104175, math0109209,
//! math0310311, math0404431, cs0204036; plain TeX `\input diagrams`: math0105166; the `amstex`
//! option: math0104206, math0403203.
//!
//! The macro surface is the manual's (manual.tex v3.97, "Typing the diagram", "Labels",
//! "Parallel arrows", "Defining arrow styles", "Diagonal lines", "Options"), checked against the
//! use in the 100 witness papers:
//! - `\begin{diagram}[options] … \end{diagram}` (plain `\diagram … \enddiagram`): cells `&`, rows
//!   `\\` (the package's `\cr`, with no `[dim]`). In maths it is a matrix of the formula; outside
//!   maths it is displayed ("it is better to remove such enclosings", manual FAQ "Display
//!   options"), but in a table cell or an LR box, as with `[inline]`, it stays in the text line;
//!   `[LaTeXeqno]` numbers it with LaTeX's `equation` counter ("You can put a \label inside").
//!   The options are read only when `[` follows at once (manual "Options": after
//!   `\begin{diagram}` and a new line a first cell may begin with a bracket).
//! - An arrow `\<dir><Name>` for the directions r l u d ru rd lu ld, followed by `[options]`,
//!   `(x,y)` (the cells a diagonal spans), positional `{f}{g}` (very old versions) and the labels
//!   `^` above, `_` below, `<` left, `>` right, `~` middle, each a single token or a braced group.
//!   "For reverse compatibility, above=left and below=right for vertical arrows"; for negative
//!   gradient diagonals (rd, lu) above=right and below=left. Outside a diagram the source reads
//!   no span and no `<`/`>` (`\switch@arg`, diagrams.tex:301-306). Horizontal arrows "may also be
//!   used in text, `$A \rTo^f B$`"; one outside maths is the package's error, as is a vertical
//!   arrow outside a diagram (in maths or not); a diagonal one outside a diagram raises nothing.
//! - `\newarrow{Name}{tail}{filler}{middle}{filler}{head}` defines the eight arrows of `Name`
//!   (diagrams.tex:449, the definitions made at the caller's group level with `\aftergroup`; a tip
//!   or middle equal to its filler is empty; ignored within maths), and the arrows the source
//!   declares in its section (24) are predefined, with its synonyms (`\rInto`, `\hLine`, `\NW`, …).
//! - `\pile{\rTo^f \\ \rTo_g}`: parallel horizontal arrows, a one-column array.
//! - Layout only, no output: `\diagramstyle[…]` (but its `amstex` loads amscd's `{CD}`),
//!   `\newarrowhead/tail/middle/filler`, `\newdiagramgrid`, `\HonV`, `\VonH`, `\HmeetV`. The
//!   pullback corners `\SEpbk` etc. are a visible mark not emitted (open, OD #589;
//!   math0203179, math0306018).
use latexml_package::prelude::*;

/// The eight arrow directions of `\newarrow` (manual "Defining arrow styles").
const DIRECTIONS: [&str; 8] = ["r", "l", "u", "d", "ru", "rd", "lu", "ld"];

/// The direction an arrow drawn backwards (a tail that is an arrowhead, `<----`) points.
fn reverse_direction(dir: &str) -> &'static str {
  match dir {
    "r" => "l",
    "l" => "r",
    "u" => "d",
    "d" => "u",
    "ru" => "ld",
    "ld" => "ru",
    "rd" => "lu",
    _ => "rd",
  }
}

fn direction_index(dir: &str) -> usize { DIRECTIONS.iter().position(|d| *d == dir).unwrap_or(0) }

/// The glyphs of one arrow kind in the order of [`DIRECTIONS`].
fn kind_glyphs(kind: &str) -> [&'static str; 8] {
  const TO: [&str; 8] = ["→", "←", "↑", "↓", "↗", "↘", "↖", "↙"];
  match kind {
    "line" => ["—", "—", "|", "|", "╱", "╲", "╲", "╱"],
    "eq" => ["=", "=", "‖", "‖", "=", "=", "=", "="],
    "dots" => ["⋯", "⋯", "⋮", "⋮", "⋰", "⋱", "⋱", "⋰"],
    "dashes" => ["╌", "╌", "╎", "╎", "╱", "╲", "╲", "╱"],
    "triple" => ["≡", "≡", "⦀", "⦀", "≡", "≡", "≡", "≡"],
    "dashto" => ["⇢", "⇠", "⇡", "⇣", TO[4], TO[5], TO[6], TO[7]],
    "mapsto" => ["↦", "↤", "↥", "↧", TO[4], TO[5], TO[6], TO[7]],
    "hook" => ["↪", "↩", TO[2], TO[3], TO[4], TO[5], TO[6], TO[7]],
    "tail" => ["↣", "↢", TO[2], TO[3], TO[4], TO[5], TO[6], TO[7]],
    "onto" => ["↠", "↞", "↟", "↡", TO[4], TO[5], TO[6], TO[7]],
    "tailonto" => ["⤖", "⬻", "↟", "↡", TO[4], TO[5], TO[6], TO[7]],
    "mapsonto" => ["⤅", "⬶", "↟", "↡", TO[4], TO[5], TO[6], TO[7]],
    "implies" => ["⇒", "⇐", "⇑", "⇓", "⇗", "⇘", "⇖", "⇙"],
    "tripleto" => ["⇛", "⇚", "⤊", "⤋", TO[4], TO[5], TO[6], TO[7]],
    "both" => ["↔", "↔", "↕", "↕", "⤢", "⤡", "⤡", "⤢"],
    "iff" => ["⇔", "⇔", "⇕", "⇕", "⤢", "⤡", "⤡", "⤢"],
    "harpoon" => ["⇀", "↽", "↾", "⇂", TO[4], TO[5], TO[6], TO[7]],
    "open" => ["⇾", "⇽", TO[2], TO[3], TO[4], TO[5], TO[6], TO[7]],
    _ => TO,
  }
}

/// The glyph `\<dir><Name>` is drawn with, from the five components `\newarrow` was given
/// (diagrams.tex section (23) names them: heads `>` `vee` `LaTeX` `->` `>>` `->>` `doublevee`
/// `=>` `harpoon` `triangle` …, tails `|` `mapsto` `C` `hook` `boldhook` `>` `<` `<=` …, fillers
/// `-` `=` `.` `dash` `3`). A component the package does not know (a document's own
/// `\newarrowhead`) draws as the plain arrow of its shaft; an arrow of no components
/// (`\newarrow{Nul}{}{}{}{}{}`, math0310311) draws nothing.
fn arrow_glyph(dir: &str, tail: &str, filler1: &str, filler2: &str, head: &str) -> &'static str {
  if [tail, filler1, filler2, head]
    .iter()
    .all(|part| part.is_empty())
  {
    return "";
  }
  let filler = if filler1.is_empty() { filler2 } else { filler1 };
  let shaft = match filler {
    "=" => "double",
    "." => "dotted",
    "dash" => "dashed",
    "3" => "triple",
    _ => "solid",
  };
  let tail_kind = match tail {
    "" => "",
    "|" | "mapsto" => "bar",
    "C" | "hook" | "hooka" | "hookb" | "boldhook" | "boldhooka" | "boldhookb" | "sqhook" => "hook",
    ">" | "vee" | "littlevee" | "boldlittlevee" | "curlyvee" | "LaTeX" | "triangle" => "tail",
    "<" | "<<" | "<=" | "<-" | "<<-" => "head",
    _ => "",
  };
  let head_kind = match head {
    "" => "",
    ">>" | "->>" | "doublevee" | "doubleLaTeX" => "two",
    "=>" => "double",
    "harpoon" => "harpoon",
    "triangle" => "open",
    _ => "simple",
  };
  let (kind, dir) = match (tail_kind, head_kind) {
    ("head", "") => {
      let kind = if shaft == "double" || tail == "<=" {
        "implies"
      } else if tail.starts_with("<<") {
        "onto"
      } else {
        "to"
      };
      (kind, reverse_direction(dir))
    },
    ("head", _) => (
      if shaft == "double" || tail == "<=" || head_kind == "double" {
        "iff"
      } else {
        "both"
      },
      dir,
    ),
    (_, "") => (
      match shaft {
        "double" => "eq",
        "dotted" => "dots",
        "dashed" => "dashes",
        "triple" => "triple",
        _ => "line",
      },
      dir,
    ),
    (_, "double") => ("implies", dir),
    (..) if shaft == "double" => ("implies", dir),
    (..) if shaft == "triple" => ("tripleto", dir),
    ("tail", "two") => ("tailonto", dir),
    ("bar", "two") => ("mapsonto", dir),
    (_, "two") => ("onto", dir),
    (_, "harpoon") => ("harpoon", dir),
    (_, "open") => ("open", dir),
    ("bar", _) => ("mapsto", dir),
    ("hook", _) => ("hook", dir),
    ("tail", _) => ("tail", dir),
    _ if shaft == "dotted" || shaft == "dashed" => ("dashto", dir),
    _ => ("to", dir),
  };
  kind_glyphs(kind)[direction_index(dir)]
}

/// `\newarrow{Name}{tail}{filler}{middle}{filler}{head}` as definitions, one per direction:
/// `\def\<dir><Name>{\lx@dg@arrow\<dir><Name>{<dir>}{<glyph>}}` (diagrams.tex:449-470).
fn newarrow_definitions(
  name: &str,
  tail: &str,
  filler1: &str,
  filler2: &str,
  head: &str,
) -> Tokens {
  // diagrams.tex:450-454: a tail equal to the first filler, a head equal to the second, is empty.
  let tail = if tail == filler1 { "" } else { tail };
  let head = if head == filler2 { "" } else { head };
  let mut tokens = Vec::new();
  for dir in DIRECTIONS {
    let cs = T_CS!(&*s!("\\{dir}{name}"));
    let glyph = arrow_glyph(dir, tail, filler1, filler2, head);
    tokens.extend([
      T_CS!("\\def"),
      cs,
      T_BEGIN!(),
      T_CS!("\\lx@dg@arrow"),
      cs,
      T_BEGIN!(),
    ]);
    tokens.extend(Explode!(dir));
    tokens.extend([T_END!(), T_BEGIN!()]);
    tokens.extend(glyph.chars().map(|c| CharToken!(c)));
    tokens.extend([T_END!(), T_END!()]);
  }
  Tokens::new(tokens)
}

/// Is `token` the label marker `^` / `_`? The manual ("Labels", footnote): "The sub- and
/// superscript characters are recognised by their `\catcode`s, so `\sp` and `\sb` will work".
fn script_catcode(token: &Token) -> Option<Catcode> {
  let code = match lookup_meaning(token) {
    Some(Stored::Token(meaning)) => meaning.get_catcode(),
    _ => token.get_catcode(),
  };
  matches!(code, Catcode::SUPER | Catcode::SUB).then_some(code)
}

/// Read a delimited group `open … close` after its opener was read; it is kept when `accept` says
/// the text is the package's syntax, else the tokens go back to the input (`\rTo(\calf.\pi)^c`,
/// `$f\colon X\rTo [0,1]$` are an object after a text arrow, not a span or options).
fn read_bracketed(open: Token, close: &str, accept: fn(&str) -> bool) -> Result<bool> {
  let mut read = vec![open];
  let mut depth = 0usize;
  loop {
    let Some(token) = read_token()? else { break };
    read.push(token);
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => break,
      Catcode::END => depth -= 1,
      _ if depth == 0
        && token.get_catcode() == Catcode::OTHER
        && token.with_str(|s| s == close) =>
      {
        let text = Tokens::new(read[1..read.len() - 1].to_vec()).to_string();
        if accept(&text) {
          return Ok(true);
        }
        break;
      },
      _ => {},
    }
    if read.len() > 200 {
      break;
    }
  }
  unread(Tokens::new(read));
  Ok(false)
}

/// `(x,y)`: two numbers (manual "Diagonal lines"; `\rdTo(5.225,1.9)` in math0104175-era papers).
fn is_span(text: &str) -> bool {
  let parts: Vec<&str> = text.split(',').map(str::trim).collect();
  parts.len() == 2
    && parts.iter().all(|p| {
      !p.is_empty()
        && p
          .chars()
          .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
    })
}

/// `[options]`: a key list, which begins with a letter (`[abut]`, `[leftshortfall=.425cm]`).
fn is_options(text: &str) -> bool {
  text
    .trim_start()
    .chars()
    .next()
    .is_some_and(|c| c.is_ascii_alphabetic())
}

/// The labels of an arrow: above / below (left / right of a vertical) and middle.
#[derive(Default)]
struct Labels {
  first:  Option<Tokens>,
  second: Option<Tokens>,
  mid:    Option<Tokens>,
}

/// Read what follows an arrow command (manual "Labels", "Options", "Diagonal lines"). Inside a
/// diagram that is the source's `\CD@PB` (diagrams.tex:290-299, made `\switch@arg` at :729):
/// `^`, `_`, `<`, `>`, `~`, `(`, `[`, `.`, a space or a `{`. Outside one it is `\switch@arg`
/// (:301-306), which reads `^`, `_`, `[`, `.`, `~`, a space or a `{` and leaves a `(`, `<` or
/// `>` to the formula (`$X\rTo (0,1)$` prints "X → (0, 1)"). A `.` is the dotted-shaft switch
/// (`\CD@JJ`, :583), read and not drawn. A `{` is a positional label (`\positional@`, :578): the
/// upper one while that is unread, then the lower one; after a lower label it ends the arrow.
fn read_arrow_labels(dir: &str, in_diagram: bool) -> Result<Labels> {
  let mut labels = Labels::default();
  let mut any_label = false;
  // For a negative-gradient diagonal `<` is below and `>` above (manual "Labels").
  let negative = matches!(dir, "rd" | "lu");
  loop {
    let mut spaces = Vec::new();
    let token = loop {
      match read_token()? {
        Some(t) if t.get_catcode() == Catcode::SPACE => spaces.push(t),
        other => break other,
      }
    };
    let Some(token) = token else {
      unread(Tokens::new(spaces));
      break;
    };
    let other = |text: &str| token.get_catcode() == Catcode::OTHER && token.with_str(|s| s == text);
    if other(".") {
      continue;
    }
    let slot: Option<u8> = match script_catcode(&token) {
      Some(Catcode::SUPER) => Some(1),
      Some(_) => Some(2),
      None if in_diagram && other("<") => Some(if negative { 2 } else { 1 }),
      None if in_diagram && other(">") => Some(if negative { 1 } else { 2 }),
      None if token.get_catcode() == Catcode::ACTIVE && token.with_str(|s| s == "~") => Some(3),
      None => None,
    };
    if let Some(slot) = slot {
      let place = match slot {
        1 => &mut labels.first,
        2 => &mut labels.second,
        _ => &mut labels.mid,
      };
      if place.is_some() {
        unread_one(token);
        unread(Tokens::new(spaces));
        break;
      }
      *place = Some(read_arg(ExpansionLevel::Off)?);
      any_label = true;
      continue;
    }
    if !any_label && token.get_catcode() == Catcode::OTHER {
      let opener = token.with_str(|s| match s {
        "(" if in_diagram => Some((")", is_span as fn(&str) -> bool)),
        "[" => Some(("]", is_options as fn(&str) -> bool)),
        _ => None,
      });
      if let Some((close, accept)) = opener {
        if read_bracketed(token, close, accept)? {
          continue;
        }
        unread(Tokens::new(spaces));
        break;
      }
    }
    // Very old versions' positional labels, `\rTo{f}{g}`, `\rTo^f{g}` (manual "Labels").
    if labels.second.is_none() && token.get_catcode() == Catcode::BEGIN {
      let group = read_balanced(ExpansionLevel::Off, false, false)?;
      let place = if labels.first.is_none() {
        &mut labels.first
      } else {
        &mut labels.second
      };
      *place = Some(group);
      any_label = true;
      continue;
    }
    unread_one(token);
    unread(Tokens::new(spaces));
    break;
  }
  Ok(labels)
}

fn braced(tokens: Option<Tokens>) -> Vec<Token> {
  let mut out = vec![T_BEGIN!()];
  if let Some(tokens) = tokens {
    out.extend(tokens.unlist());
  }
  out.push(T_END!());
  out
}

#[rustfmt::skip]
LoadDefinitions!({
  // diagrams.tex section (25): labels are `\textstyle` in a diagram and `\scriptstyle` in text;
  // objects are `\displaystyle` (documents `\renewcommand{\labelstyle}`, math0012073).
  RawTeX!(r"\newif\ifincommdiag
\def\labelstyle{\ifincommdiag\textstyle\else\scriptstyle\fi}
\let\objectstyle\displaystyle");

  // `\lx@dg@arrow <cs>{<dir>}{<glyph>}`: an arrow command. Unexpandable, as the source's arrows
  // begin with `\relax` (`\CD@nB`, `\CD@pB`): the mode is tested when the arrow is digested, not
  // when an alignment's look-ahead for `\omit` expands it at a cell's start, before the cell's
  // math begins (`&\rTo` in `align*`, math0404431). A horizontal arrow belongs in a formula
  // (manual "Typing the diagram": "You still need the dollars"); typed in text it is the
  // package's error (diagrams.tex `\CD@nB` → `\CD@E`, "Horizontal arrows may be used outside
  // diagrams, but must still be in maths"), and, where TeX's inserted `$` runs on to the
  // paragraph's end, the arrow alone is set as a formula (`$`: plain TeX has no `\ensuremath`,
  // math0105166). A vertical arrow outside a diagram, in maths or text, is the package's other
  // error (`\CD@pB` sets `\CD@ZD` to `\CD@LK`, diagrams.tex:488, "vertical map ignored outside
  // diagram", :497-498), with no `$` test; TeX drops the arrow, we keep the author's arrow, flagged
  // by the Error, as for a horizontal one. A diagonal goes through `\Vector@Map`, which raises
  // nothing: in text it is set as a formula without an Error.
  DefPrimitive!("\\lx@dg@arrow Token {}{}", sub[(cs, dir, glyph)] {
    let in_math = lookup_bool("IN_MATH");
    match dir.to_string().as_str() {
      "u" | "d" => {
        if !if_condition(&T_CS!("\\ifincommdiag"))?.unwrap_or(false) {
          Error!("unexpected", cs,
            "Commutative Diagram: vertical map ignored outside diagram");
        }
      },
      "r" | "l" if !in_math => {
        Error!("unexpected", cs,
          "Commutative Diagram: \"$\" inserted before horizontal map");
      },
      _ => {},
    }
    let mut tokens = Vec::new();
    if !in_math {
      tokens.push(T_MATH!());
    }
    tokens.extend([T_CS!("\\lx@dg@read"), cs, T_BEGIN!()]);
    tokens.extend(dir.unlist());
    tokens.extend([T_END!(), T_BEGIN!()]);
    tokens.extend(glyph.unlist());
    tokens.extend([T_END!(), T_BEGIN!()]);
    if !in_math {
      tokens.push(T_MATH!());
    }
    tokens.push(T_END!());
    unread(Tokens::new(tokens));
    Ok(Vec::new())
  });
  // `\lx@dg@read <cs>{<dir>}{<glyph>}{<close>}`, in maths: the arrow reads its options, span and
  // labels, then `<close>` (the `$` of a text arrow).
  DefMacro!("\\lx@dg@read Token {}{}{}", sub[(cs, dir, glyph, close)] {
    let dir = dir.to_string();
    let in_diagram = if_condition(&T_CS!("\\ifincommdiag"))?.unwrap_or(false);
    let labels = read_arrow_labels(&dir, in_diagram)?;
    let constructor = if matches!(dir.as_str(), "u" | "d") { "\\lx@dg@side" } else { "\\lx@dg@stack" };
    let mut tokens = vec![T_CS!(constructor), T_BEGIN!()];
    tokens.push(cs);
    tokens.extend([T_END!(), T_BEGIN!(), T_CS!("\\lx@dg@glyph"), T_BEGIN!()]);
    tokens.extend(Explode!(dir));
    tokens.extend([T_END!(), T_BEGIN!()]);
    tokens.extend(glyph.unlist());
    tokens.extend([T_END!(), T_END!()]);
    tokens.extend(braced(labels.first));
    tokens.extend(braced(labels.second));
    tokens.extend(braced(labels.mid));
    tokens.extend(close.unlist());
    Ok(Tokens::new(tokens))
  });

  // The arrow itself, a stretchy operator as amscd's `\lx@amscd@rightarrow`.
  DefPrimitive!("\\lx@dg@glyph{}{}", sub[(dir, glyph)] {
    let dir = dir.to_string();
    let glyph = glyph.to_string();
    let props = match dir.as_str() {
      "r" | "l" => stored_map!(
        "role" => "ARROW",
        "stretchy" => "true",
        "class" => "ltx_horizontally_stretchy",
        "width" => Dimension!("30pt")
      ),
      "u" | "d" => stored_map!("role" => "ARROW", "stretchy" => "true"),
      _ => stored_map!("role" => "ARROW"),
    };
    if glyph.is_empty() {
      return Ok(Vec::new());
    }
    let reversion = Tokens::new(glyph.chars().map(|c| CharToken!(c)).collect());
    Ok(vec![Digested::from(Tbox::new(pin(&glyph), None, None, reversion, props))])
  });

  // A horizontal or diagonal arrow with its labels above and below, as amscd's
  // `\lx@amscd@stack`; a middle label (`~`) takes the place above when that is free.
  DefConstructor!("\\lx@dg@stack Undigested {} ScriptStyle ScriptStyle ScriptStyle",
    sub[document, args, props] {
      let scriptpos = props.get("scriptpos").map(|v| v.to_string()).filter(|s| !s.is_empty());
      let present = |i: usize| args.get(i).and_then(|a| a.as_ref())
        .filter(|a| !a.is_empty().unwrap_or(true));
      let op = args.get(1).and_then(|a| a.as_ref());
      let mut over: Vec<&Digested> = present(2).into_iter().collect();
      let mut under: Vec<&Digested> = present(3).into_iter().collect();
      if let Some(mid) = present(4) {
        if over.is_empty() || !under.is_empty() { over.push(mid) } else { under.push(mid) }
      }
      let script_attrs = |role: &str| {
        let mut attrs: HashMap<String, String> = map!("role" => role.to_string());
        if let Some(pos) = &scriptpos { attrs.insert("scriptpos".to_string(), pos.clone()); }
        attrs
      };
      if !under.is_empty() {
        document.open_element("ltx:XMApp", Some(map!("role" => "ARROW".to_string())), None)?;
        document.insert_element("ltx:XMTok", Vec::new(), Some(script_attrs("SUBSCRIPTOP")))?;
      }
      if !over.is_empty() {
        // The scripted arrow is still an arrow (a relation between the objects either side, `$A
        // \rTo^f B$`); amscd leaves its superscript-only stack without a role.
        let role = under.is_empty().then(|| map!("role" => "ARROW".to_string()));
        document.open_element("ltx:XMApp", role, None)?;
        document.insert_element("ltx:XMTok", Vec::new(), Some(script_attrs("SUPERSCRIPTOP")))?;
      }
      if let Some(op) = op { document.insert_element("ltx:XMArg", vec![op], None)?; }
      if !over.is_empty() {
        document.insert_element("ltx:XMArg", over, None)?;
        document.close_element("ltx:XMApp")?;
      }
      if !under.is_empty() {
        document.insert_element("ltx:XMArg", under, None)?;
        document.close_element("ltx:XMApp")?;
      }
    },
    properties => sub[_args] {
      Ok(stored_map!("scriptpos" => format!("mid{}", get_script_level())))
    },
    reversion => sub[_whatsit, args] { revert_arrow(args, false) }
  );

  // A vertical arrow with its labels left and right, as amscd's `\lx@amscd@adjacent` (the
  // labels have no width, so they do not move the arrow off the column's centre); a middle label
  // takes the right when that is free.
  DefConstructor!("\\lx@dg@side Undigested {} ScriptStyle ScriptStyle ScriptStyle",
    sub[document, args, _props] {
      let present = |i: usize| args.get(i).and_then(|a| a.as_ref())
        .filter(|a| !a.is_empty().unwrap_or(true));
      let op = args.get(1).and_then(|a| a.as_ref());
      let mut left: Vec<&Digested> = present(2).into_iter().collect();
      let mut right: Vec<&Digested> = present(3).into_iter().collect();
      if let Some(mid) = present(4) {
        if right.is_empty() || !left.is_empty() { right.push(mid) } else { left.push(mid) }
      }
      document.open_element("ltx:XMWrap", Some(map!("role" => "ARROW".to_string())), None)?;
      if !left.is_empty() {
        document.insert_element("ltx:XMArg", left, Some(map!("width" => "0.0pt".to_string())))?;
      }
      if let Some(op) = op { document.insert_element("ltx:XMArg", vec![op], None)?; }
      if !right.is_empty() {
        document.insert_element("ltx:XMArg", right, Some(map!("width" => "0.0pt".to_string())))?;
      }
      document.close_element("ltx:XMWrap")?;
    },
    reversion => sub[_whatsit, args] { revert_arrow(args, true) }
  );

  // The source's synonyms, `\let\rInto\rIntoA` (diagrams.tex section (24)), as arrows of their own
  // name, so a formula's source keeps the command its author typed.
  DefMacro!("\\lx@dg@alias Token Token", sub[(new, old)] {
    let body = lookup_definition(&old)?.and_then(|d| match d.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone().unlist()),
      _ => None,
    });
    match body {
      Some(mut body) if body.len() > 1 && body[0] == T_CS!("\\lx@dg@arrow") => {
        body[1] = new;
        let mut tokens = vec![T_CS!("\\def"), new, T_BEGIN!()];
        tokens.extend(body);
        tokens.push(T_END!());
        Ok(Tokens::new(tokens))
      },
      _ => Ok(Tokens!(T_CS!("\\let"), new, old)),
    }
  });

  // `\newarrow{Name}{tail}{filler}{middle}{filler}{head}` (diagrams.tex:449): "ignored within
  // maths".
  DefMacro!("\\newarrow{}{}{}{}{}{}", sub[(name, tail, filler1, _middle, filler2, head)] {
    if lookup_bool("IN_MATH") {
      Warn!("unexpected", "\\newarrow", s!("diagrams: {} ignored within maths", name.to_string()));
      return Ok(Tokens!());
    }
    let part = |t: &Tokens| t.to_string().trim().to_string();
    Ok(newarrow_definitions(&part(&name), &part(&tail), &part(&filler1), &part(&filler2), &part(&head)))
  });
  // The arrow components (diagrams.tex:408-410) are drawing instructions.
  DefMacro!("\\newarrowhead{}{}{}{}{}", "");
  DefMacro!("\\newarrowtail{}{}{}{}{}", "");
  DefMacro!("\\newarrowmiddle{}{}{}{}{}", "");
  DefMacro!("\\newarrowfiller{}{}{}{}{}", "");
  // Grid and style options set the drawing (manual "Options", diagrams.tex:615 `\diagramsstyle`).
  DefMacro!("\\newdiagramgrid{}{}{}", "");
  // `\diagramstyle[opts]`, or `\diagramstyle{opts}` (diagrams.tex `\CD@CF\CD@AE\CD@AE`, :282-285: a
  // `[` read as the options, else the next argument; `\diagramstyle{PostScript=dvips,…}`,
  // math0403203). The option `amstex` enables AmS-TeX's commutative-diagram commands (manual
  // "Options", `\cds@amstex` defines `\CD`): the amscd binding's `{CD}` (math0104206).
  DefMacro!("\\diagramstyle", {
    let next = read_token()?;
    let is_bracket = next
      .as_ref()
      .is_some_and(|t| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "["));
    if let Some(next) = next {
      unread_one(next);
    }
    let options = if is_bracket { read_optional(None)? } else { Some(read_arg(ExpansionLevel::Off)?) };
    let amstex = options.is_some_and(|o| o.to_string().split(',').any(|o| o.trim() == "amstex"));
    Ok(if amstex { Tokens!(T_CS!("\\lx@dg@amstex")) } else { Tokens!() })
  });
  DefPrimitive!("\\lx@dg@amstex", {
    RequirePackage!("amscd");
  });
  Let!("\\diagramsstyle", "\\diagramstyle");
  // Lines meeting and crossing (manual "The Cube"): drawing only. The pullback corners (manual
  // "Diagonal lines", diagrams.tex:1314-1331 `\make@pbk`) are a visible ⌟ ⌞ ⌝ ⌜ at the cell's
  // corner, typed mid-expression (math0203179 `X\times_Z\SEpbk Y`, math0306018), so no inline
  // token: not emitted, open (OXIDIZED_DESIGN_DIVERGENCES #589).
  DefMacro!("\\HonV[]", "");
  DefMacro!("\\VonH[]", "");
  DefMacro!("\\HmeetV[]", "");
  DefMacro!("\\SEpbk", "");
  DefMacro!("\\SWpbk", "");
  DefMacro!("\\NEpbk", "");
  DefMacro!("\\NWpbk", "");

  // `\pile{\rTo^f \\ \rTo_g}`: "Effectively, a one-column diagram is created" (manual "Parallel
  // arrows"); in diagrams and in text.
  DefMacro!("\\pile{}", "\\lx@gen@plain@matrix{name=pile}{#1}");

  // `\begin{diagram}[options]`: the options only when `[` follows at once (manual "Options").
  DefMacro!("\\diagram", {
    let mut options = Tokens!();
    if let Some(next) = read_token()? {
      if next.get_catcode() == Catcode::OTHER && next.with_str(|s| s == "[") {
        unread_one(next);
        if let Some(opts) = read_optional(None)? {
          options = opts;
        }
      } else {
        unread_one(next);
      }
    }
    let mut tokens = vec![T_CS!("\\lx@dg@open"), T_BEGIN!()];
    tokens.extend(options.unlist());
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });
  // The mode is tested when the diagram is digested, not when an alignment's look-ahead for
  // `\omit` expands `\diagram` at a cell's start, before the cell's math begins (as the arrows).
  DefPrimitive!("\\lx@dg@open{}", sub[(options)] {
    let options = options.to_string();
    let has = |name: &str| options.split(',').any(|o| o.trim() == name);
    let source = if lookup_bool("IN_MATH") {
      r"\def\lx@dg@close{\lx@dg@end}\lx@dg@begin"
    } else if has("inline") || lookup_string("MODE") == "restricted_horizontal" {
      // In a box or a table cell (restricted horizontal mode, where `$$` is an empty formula)
      // there is no display to make: the diagram is a formula of the line (math0010298).
      r"\def\lx@dg@close{\lx@dg@end$}$\lx@dg@begin"
    } else if has("LaTeXeqno") {
      r"\def\lx@dg@close{\lx@dg@end\end{equation}}\begin{equation}\lx@dg@begin"
    } else {
      // `$$`, not LaTeX's `\[`: the package is plain TeX's too (`\input diagrams`, math0105166).
      r"\def\lx@dg@close{\lx@dg@end$$}$$\lx@dg@begin"
    };
    unread(TokenizeInternal!(source));
    Ok(Vec::new())
  });
  DefMacro!("\\enddiagram", "\\lx@dg@close");
  DefMacro!("\\lx@dg@close", "");
  // In a diagram `\\` is `\cr` (diagrams.tex:729 `\let\\\CD@cC`, :739 `\def\CD@cC{\cr}`): it reads
  // no `[dim]`, so a next row may begin with a bracket (`\\ [x,yK]`, math0109209; `\\ [0]`,
  // math0403203). The row end of a `\\` without its optional argument (tex_tables.rs
  // `alignment_newline_tokens`).
  DefMacro!("\\lx@dg@cr", "\\lx@hidden@cr{\\lx@alignment@newline@marker}");
  DefMacro!("\\lx@dg@begin",
    "\\lx@gen@matrix@bindings{name=diagram,datameaning=commutative-diagram}\
     \\incommdiagtrue\\let\\\\\\lx@dg@cr\
     \\lx@ams@matrix@{name=diagram,datameaning=commutative-diagram}\\lx@begin@alignment");
  DefMacro!("\\lx@dg@end", "\\lx@end@alignment\\lx@end@gen@matrix");

  // The arrows the source declares (diagrams.tex section (24), the default `\else` branch).
  RawTeX!(r"\newarrow{To}---->
\newarrow{Arr}---->
\newarrow{Dotsto}....>
\newarrow{Dotsarr}....>
\newarrow{Dashto}{}{dash}{}{dash}>
\newarrow{Dasharr}{}{dash}{}{dash}>
\newarrow{Mapsto}{mapsto}--->
\newarrow{Mapsarr}{mapsto}--->
\newarrow{IntoA}{hooka}--->
\newarrow{IntoB}{hookb}--->
\newarrow{Embed}>--->
\newarrow{Emarr}>--->
\newarrow{Onto}----{>>}
\newarrow{Dotsonarr}....{>>}
\newarrow{Dotsonto}....{>>}
\newarrow{Implies}===={=>}
\newarrow{Project}----{triangle}
\newarrow{Pto}----{harpoon}
\newarrow{Relto}{harpoon}---{harpoon}
\newarrow{Eq}=====
\newarrow{Line}-----
\newarrow{Dots}.....
\newarrow{Dashes}{}{dash}{}{dash}{}
\newarrow{SquareInto}{sqhook}--->
\lx@dg@alias\uFrom\uTo\lx@dg@alias\lFrom\lTo\lx@dg@alias\uDotsfrom\uDotsto\lx@dg@alias\lDotsfrom\lDotsto
\lx@dg@alias\uDashfrom\uDashto\lx@dg@alias\lDashfrom\lDashto\lx@dg@alias\uImpliedby\uImplies\lx@dg@alias\lImpliedby\lImplies
\lx@dg@alias\uMapsfrom\uMapsto\lx@dg@alias\lMapsfrom\lMapsto\lx@dg@alias\lOnfrom\lOnto\lx@dg@alias\uOnfrom\uOnto
\lx@dg@alias\lPfrom\lPto\lx@dg@alias\uPfrom\uPto
\lx@dg@alias\uInfromA\uIntoA\lx@dg@alias\uInfromB\uIntoB\lx@dg@alias\lInfromA\lIntoA\lx@dg@alias\lInfromB\lIntoB
\lx@dg@alias\rInto\rIntoA\lx@dg@alias\lInto\lIntoA\lx@dg@alias\dInto\dIntoB\lx@dg@alias\uInto\uIntoA
\lx@dg@alias\ruInto\ruIntoA\lx@dg@alias\luInto\luIntoA\lx@dg@alias\rdInto\rdIntoA\lx@dg@alias\ldInto\ldIntoA
\lx@dg@alias\hEq\rEq\lx@dg@alias\vEq\uEq\lx@dg@alias\hLine\rLine\lx@dg@alias\vLine\uLine
\lx@dg@alias\hDots\rDots\lx@dg@alias\vDots\uDots\lx@dg@alias\hDashes\rDashes\lx@dg@alias\vDashes\uDashes
\lx@dg@alias\NW\luTo\lx@dg@alias\NE\ruTo\lx@dg@alias\SW\ldTo\lx@dg@alias\SE\rdTo
\def\nNW{\luTo(2,3)}\def\nNE{\ruTo(2,3)}\def\sSW{\ldTo(2,3)}\def\sSE{\rdTo(2,3)}
\def\wNW{\luTo(3,2)}\def\eNE{\ruTo(3,2)}\def\wSW{\ldTo(3,2)}\def\eSE{\rdTo(3,2)}
\def\NNW{\luTo(2,4)}\def\NNE{\ruTo(2,4)}\def\SSW{\ldTo(2,4)}\def\SSE{\rdTo(2,4)}
\def\WNW{\luTo(4,2)}\def\ENE{\ruTo(4,2)}\def\WSW{\ldTo(4,2)}\def\ESE{\rdTo(4,2)}");
});

/// The source of an arrow: the command, then its labels in the package's syntax (`^`/`_` on a
/// horizontal or diagonal, `<`/`>` on a vertical, `~` in the middle).
fn revert_arrow(args: &[Option<Digested>], vertical: bool) -> Result<Tokens> {
  let mut tokens = match args.first() {
    Some(Some(cs)) => cs.revert()?.unlist(),
    _ => Vec::new(),
  };
  let markers = if vertical {
    [T_OTHER!("<"), T_OTHER!(">"), T_ACTIVE!('~')]
  } else {
    [T_SUPER!(), T_SUB!(), T_ACTIVE!('~')]
  };
  for (i, marker) in markers.into_iter().enumerate() {
    if let Some(Some(label)) = args.get(i + 2)
      && !label.is_empty().unwrap_or(true)
    {
      tokens.push(marker);
      tokens.push(T_BEGIN!());
      tokens.extend(label.revert()?.unlist());
      tokens.push(T_END!());
    }
  }
  Ok(Tokens::new(tokens))
}
