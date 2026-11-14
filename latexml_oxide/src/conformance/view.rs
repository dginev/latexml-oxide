//! What one side of a comparison reads and does: the arguments the document supplies, in order,
//! and the mode transitions made before any material.

use latexml_core::{
  common::arena,
  definition::DeclaredMode,
  parameter::{Parameter, Parameters},
};

/// One argument, as the document writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
  /// An undelimited argument: a braced group or one token (TeX's `#1`, a binding's `{}`).
  Mandatory,
  /// `[…]`, peeked for; with the default its absent branch supplies, when the walk sees one.
  Optional(Option<String>),
  /// A peek for `*`.
  Star,
  /// A peek for another token (`(` of the picture forms), whose absent branch the walk took.
  Peek(String),
  /// An argument delimited by these tokens (`\def\x#1.`, a binding's `Until:`).
  Delimited(String),
  /// Tokens the definition requires literally (`\def\x[#1]`, a binding's `Match:`).
  Literal(String),
  /// A quantity the binding scans (`Dimension`, `Number`, …) where TeX reads an argument.
  Scan(String),
}

/// The mode transitions made before the first material.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Prologue {
  /// `\leavevmode` (or a binding's `enter_horizontal`): starts a paragraph.
  pub enter_horizontal: bool,
  /// `\par` (or a binding's `leave_horizontal`): ends the paragraph.
  pub leave_horizontal: bool,
  /// `\@bsphack`: keeps the space factor across the command (latex.ltx:9276).
  pub bsphack:          bool,
  /// `\ifmmode` before any material: the command behaves by mode.
  pub mode_test:        bool,
  /// The mode a binding begins ([`DeclaredMode::mode`]).
  pub mode:             Option<String>,
  /// One mode record declares both entering and leaving horizontal mode, which no definition does.
  pub declared_both:    bool,
}

impl Prologue {
  pub(crate) fn declare(&mut self, declared: &DeclaredMode) {
    self.enter_horizontal |= declared.enter_horizontal;
    self.leave_horizontal |= declared.leave_horizontal;
    self.declared_both |= declared.enter_horizontal && declared.leave_horizontal;
    if let Some(mode) = declared.mode {
      self.mode = Some(arena::to_string(mode));
    }
  }
}

/// One side of the comparison for a control sequence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct View {
  pub args:           Vec<Arg>,
  pub prologue:       Prologue,
  /// The arguments are all read: the walk reached the material, the end of a body, or a
  /// definition that declares its arguments. False when it stopped at something it cannot read (a
  /// macro coded in Rust, an undefined control sequence, a body whose last reading call sits in a
  /// conditional, an `\ifmmode` test, its depth limit).
  pub complete:       bool,
  /// Every mode transition before the material is in [`Self::prologue`]: the walk reached the end
  /// of a body, a mode record that declares a transition, or a definition with no code of its own.
  /// False at material (TeX decides what it does to the mode) and at a call the body goes on after.
  pub prologue_final: bool,
  /// The definitions it passed through: `\makebox`, `\makebox `, `\@makebox`, ….
  pub chain:          Vec<String>,
  /// Branches it did not follow and why it stopped short, for the report.
  pub notes:          Vec<String>,
}

/// The quantity scans among the binding parameter types: TeX reads an argument where these read a
/// number, dimension or glue (theme 7; KERNEL_CAPABILITIES K14).
const SCANS: &[&str] = &[
  "Number",
  "Dimension",
  "Glue",
  "MuGlue",
  "MuDimension",
  "Float",
  "SetlengthDimension",
  "TempboxaDimension",
];

fn text_of(parameter: &Parameter) -> String {
  parameter
    .extra
    .first()
    .map(ToString::to_string)
    .unwrap_or_default()
}

/// What one parameter reads, by its type.
enum Kind {
  /// Reads nothing the document writes as an argument (`SkipSpaces`, `SkipMatch:=`,
  /// `RequireBrace`).
  Skip,
  /// A peeked optional argument opening with `opener`; `default` when its type carries one
  /// (`[Default:x]`).
  Optional {
    opener:  char,
    default: Option<String>,
  },
  /// A peek for one token (`OptionalMatch:*`).
  OptionalMatch(String),
  /// Literal tokens (`Match:`).
  Match(String),
  /// An argument delimited by these tokens (`Until:`, `XUntil:`, `UntilBrace`).
  Until(String),
  /// A quantity scan.
  Scan(String),
  /// An undelimited argument.
  Mandatory,
}

fn kind_of(parameter: &Parameter, name: &str, text: &str) -> Kind {
  if name.starts_with("Skip") || name == "RequireBrace" {
    Kind::Skip
  } else if name.starts_with("OptionalMatch") {
    Kind::OptionalMatch(text.to_string())
  } else if name == "Optional" {
    Kind::Optional {
      opener:  '[',
      default: parameter.extra.first().map(ToString::to_string),
    }
  } else if name.starts_with("Optional") || parameter.optional {
    let opener = if name.contains("Angled") {
      '<'
    } else if name.contains("Coord") {
      '('
    } else {
      '['
    };
    Kind::Optional { opener, default: None }
  } else if name == "Match" {
    Kind::Match(text.to_string())
  } else if name == "UntilBrace" {
    Kind::Until("{".to_string())
  } else if name.ends_with("Until") || name.starts_with("Until") || name.starts_with("XUntil") {
    Kind::Until(text.to_string())
  } else if SCANS.contains(&name) {
    Kind::Scan(name.to_string())
  } else {
    Kind::Mandatory
  }
}

/// How a parameter list met the arguments a calling body gave it.
pub(crate) struct Read {
  /// A pending optional (see [`args_of`]) was read.
  pub consumed_pending: bool,
  /// For each parameter number `#k` (index `k - 1`), the given item that fills it.
  pub filled:           Vec<Option<usize>>,
  /// Given items the parameters left over: the body goes on after the call.
  pub unconsumed:       usize,
}

/// An argument a calling body gives: its text, and whether it was a bracketed `[…]` one or a braced
/// group (a single plain token otherwise).
#[derive(Clone, Copy)]
pub(crate) struct Given<'a> {
  pub text:    &'a str,
  pub bracket: bool,
  pub group:   bool,
}

/// The document-supplied arguments of `params`, after those the calling body gives it
/// (`supplied`, in order; a call `\@rsbox{#1}` gives one braced item). An optional parameter takes
/// a given bracketed item; meeting a braced one instead, it is absent — the document never sees it.
/// A delimited parameter takes the given items up to its delimiter; with no delimiter among them,
/// it goes on reading the document. `optional_pending`: a peek for `[` has already counted the next
/// bracketed argument, which the first document-read `[` optional then reads.
pub(crate) fn args_of(
  params: &Parameters,
  supplied: &[Given],
  optional_pending: bool,
  args: &mut Vec<Arg>,
) -> Read {
  let params = params.get_parameters();
  let mut pending = optional_pending;
  let mut given = supplied.iter().copied().enumerate().peekable();
  let mut filled: Vec<Option<usize>> = Vec::new();
  // TeX's parameter text `[#1][#2]#3`: a literal `[`, `#1` delimited by `][`, `#2` by `]`. A
  // bracketed argument opens with the `[` that ends the literal or delimiter before it and closes
  // at the `]` that begins a delimiter; delimiters inside it (`[#1/#2]`) split it into parameters
  // of one argument.
  let mut open_bracket = false;
  // The given item a bracketed argument took, for its inner parameters.
  let mut bracket_item: Option<Option<usize>> = None;
  for parameter in params {
    let name = arena::to_string(parameter.name);
    let text = text_of(parameter);
    let kind = kind_of(parameter, &name, &text);
    if let Kind::Skip = kind {
      continue;
    }
    if name == "Match" && text.ends_with('[') {
      if text.len() > 1 {
        args.push(Arg::Literal(text[..text.len() - 1].to_string()));
      }
      open_bracket = true;
      continue;
    }
    // TeX numbers every parameter that yields a value, `#1`…`#9`; a literal (`Match`) yields none.
    let number = (!parameter.novalue).then(|| {
      filled.push(None);
      filled.len() - 1
    });
    let fill = |slot: Option<usize>, item: usize, filled: &mut Vec<Option<usize>>| {
      if let Some(slot) = slot {
        filled[slot] = Some(item);
      }
    };
    if open_bracket && name == "Until" && !text.starts_with(']') {
      // A parameter inside the bracketed argument: it belongs to that argument's item.
      let taken = *bracket_item.get_or_insert_with(|| match given.peek() {
        Some((index, Given { bracket: true, .. })) => {
          let index = *index;
          given.next();
          Some(index)
        },
        _ => None,
      });
      if let Some(item) = taken {
        fill(number, item, &mut filled);
      }
      continue;
    }
    let bracket = open_bracket && name == "Until" && text.starts_with(']');
    // `#1[`: an argument delimited by the `[` that opens the next, bracketed one (`\@rsbox#1[#2]`).
    let opens = name == "Until" && text.ends_with('[') && (bracket || text.len() == 1);
    open_bracket = opens;
    if bracket {
      // Text between the `]` and the next `[`: a literal the document writes after the argument.
      let tail = text[1..].trim_end_matches('[').to_string();
      match bracket_item.take() {
        // The bracketed argument's last parameter: its item, if an inner parameter took one.
        Some(Some(item)) => fill(number, item, &mut filled),
        Some(None) if pending => pending = false,
        Some(None) => {
          args.push(Arg::Literal("[".to_string()));
          args.push(Arg::Delimited("]".to_string()));
        },
        None => match given.peek() {
          Some((index, Given { bracket: true, .. })) => {
            fill(number, *index, &mut filled);
            given.next();
          },
          _ if pending => pending = false,
          _ => {
            args.push(Arg::Literal("[".to_string()));
            args.push(Arg::Delimited("]".to_string()));
          },
        },
      }
      if !tail.is_empty() {
        args.push(Arg::Literal(tail));
      }
      continue;
    }
    match kind {
      Kind::Skip => {},
      Kind::Optional { opener, default } => match given.peek() {
        Some((index, Given { bracket: true, .. })) if opener == '[' => {
          fill(number, *index, &mut filled);
          given.next();
        },
        // The body gives a braced item where the optional one could be: absent.
        Some(_) => {},
        None if opener == '[' && pending => pending = false,
        None if opener == '[' => args.push(Arg::Optional(default)),
        None => args.push(Arg::Peek(opener.to_string())),
      },
      Kind::OptionalMatch(token) => match given.peek() {
        Some((index, item)) if !item.group && !item.bracket && item.text == token => {
          fill(number, *index, &mut filled);
          given.next();
        },
        Some(_) => {},
        None if token == "*" => args.push(Arg::Star),
        None => args.push(Arg::Peek(token)),
      },
      Kind::Until(delimiter) => {
        if given.peek().is_none() {
          args.push(if opens {
            Arg::Mandatory
          } else {
            Arg::Delimited(delimiter)
          });
          continue;
        }
        // The given items up to the delimiter; without it among them, the argument reads on
        // into the document.
        let mut first = None;
        let mut closed = false;
        for (index, item) in given.by_ref() {
          // Only a plain token closes it: a braced `{.}` or bracketed `[.]` is part of it.
          if !item.group && !item.bracket && item.text == delimiter && !opens {
            closed = true;
            break;
          }
          first.get_or_insert(index);
          if opens {
            closed = true;
            break;
          }
        }
        if let Some(item) = first {
          fill(number, item, &mut filled);
        }
        if !closed {
          args.push(Arg::Delimited(delimiter));
        }
      },
      Kind::Match(literal) => {
        if given.next().is_none() {
          args.push(Arg::Literal(literal));
        }
      },
      Kind::Scan(scan) => match given.next() {
        Some((index, _)) => fill(number, index, &mut filled),
        None => args.push(Arg::Scan(scan)),
      },
      Kind::Mandatory => match given.next() {
        Some((index, _)) => fill(number, index, &mut filled),
        None => args.push(Arg::Mandatory),
      },
    }
  }
  Read {
    consumed_pending: optional_pending && !pending,
    filled,
    unconsumed: given.count(),
  }
}
