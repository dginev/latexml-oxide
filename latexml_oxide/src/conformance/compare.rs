//! Comparing the real macro's view with its binding's: the mismatch classes of
//! KERNEL_CAPABILITIES K13.

use super::view::{Arg, View};

/// How much a mismatch matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
  /// The binding reads a different document or makes a different mode transition.
  High,
  /// The same shape, a different reading of it.
  Medium,
  /// A typed scan where TeX reads an argument (K14's input).
  Low,
}

/// One difference between the real macro and its binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mismatch {
  /// The macro peeks for `[…]`; the binding does not.
  OptMissing,
  /// The binding peeks for `[…]`; the macro does not.
  OptExtra,
  /// The macro peeks for `*`; the binding does not.
  StarMissing,
  /// The binding peeks for `*`; the macro does not.
  StarExtra,
  /// The macro peeks for this token (`(` of the picture forms); the binding does not.
  PeekMissing(String),
  /// The binding peeks for this token; the macro does not.
  PeekExtra(String),
  /// The two read different numbers of arguments.
  Arity { raw: usize, binding: usize },
  /// The same position is read differently (delimited, literal, …).
  Delim { raw: String, binding: String },
  /// Both have the optional argument, with different defaults.
  OptDefault { raw: String, binding: String },
  /// The binding scans a quantity where the macro reads an argument.
  ScanKind(String),
  /// The macro starts a paragraph (`\leavevmode`); the binding does not.
  EnterHorizontalMissing,
  /// The macro ends the paragraph (`\par`); the binding does not.
  LeaveHorizontalMissing,
  /// The binding starts a paragraph; the macro does not.
  EnterHorizontalExtra,
  /// The binding ends the paragraph; the macro does not.
  LeaveHorizontalExtra,
  /// The binding declares both entering and leaving horizontal mode, which no definition does.
  EnterAndLeave,
  /// The macro makes a mode transition that the binding's walk stopped short of seeing either
  /// way (a wrapper macro ahead of its constructor): inspect by hand.
  PrologueUnknown,
  /// A side could not be read to the end, so only its prologue is compared.
  Incomplete { raw: bool, binding: bool },
}

impl Mismatch {
  /// The report's name for the class.
  pub fn class(&self) -> &'static str {
    match self {
      Mismatch::OptMissing => "OPT_MISSING",
      Mismatch::OptExtra => "OPT_EXTRA",
      Mismatch::StarMissing => "STAR_MISSING",
      Mismatch::StarExtra => "STAR_EXTRA",
      Mismatch::PeekMissing(_) => "PEEK_MISSING",
      Mismatch::PeekExtra(_) => "PEEK_EXTRA",
      Mismatch::Arity { .. } => "ARITY",
      Mismatch::Delim { .. } => "DELIM",
      Mismatch::OptDefault { .. } => "OPT_DEFAULT",
      Mismatch::ScanKind(_) => "SCAN_KIND",
      Mismatch::EnterHorizontalMissing => "PROLOGUE_ENTERH_MISSING",
      Mismatch::LeaveHorizontalMissing => "PROLOGUE_LEAVEH_MISSING",
      Mismatch::EnterHorizontalExtra => "PROLOGUE_ENTERH_EXTRA",
      Mismatch::LeaveHorizontalExtra => "PROLOGUE_LEAVEH_EXTRA",
      Mismatch::EnterAndLeave => "PROLOGUE_ENTER_AND_LEAVE",
      Mismatch::PrologueUnknown => "PROLOGUE_UNKNOWN",
      Mismatch::Incomplete { .. } => "INCOMPLETE",
    }
  }

  pub fn severity(&self) -> Severity {
    match self {
      Mismatch::OptMissing
      | Mismatch::OptExtra
      | Mismatch::StarMissing
      | Mismatch::StarExtra
      | Mismatch::Arity { .. }
      | Mismatch::Delim { .. }
      | Mismatch::EnterHorizontalMissing
      | Mismatch::LeaveHorizontalMissing => Severity::High,
      Mismatch::OptDefault { .. }
      | Mismatch::PeekMissing(_)
      | Mismatch::PeekExtra(_)
      | Mismatch::EnterAndLeave
      | Mismatch::Incomplete { .. } => Severity::Medium,
      Mismatch::ScanKind(_)
      | Mismatch::EnterHorizontalExtra
      | Mismatch::LeaveHorizontalExtra
      | Mismatch::PrologueUnknown => Severity::Low,
    }
  }
}

fn describe(arg: &Arg) -> String {
  match arg {
    Arg::Mandatory => "{}".to_string(),
    Arg::Optional(_) => "[]".to_string(),
    Arg::Star => "*".to_string(),
    Arg::Peek(token) => format!("peek {token}"),
    Arg::Delimited(until) => format!("until {until}"),
    Arg::Literal(text) => format!("literal {text}"),
    Arg::Scan(kind) => kind.clone(),
  }
}

/// The mismatches between the real macro's view and its binding's.
pub fn compare(raw: &View, binding: &View) -> Vec<Mismatch> {
  let mut found = Vec::new();
  if binding.prologue.declared_both {
    found.push(Mismatch::EnterAndLeave);
  }
  if !raw.complete || !binding.complete {
    found.push(Mismatch::Incomplete {
      raw:     !raw.complete,
      binding: !binding.complete,
    });
  }
  // A transition one side makes is evidence against the other only where the other's prologue is
  // final: read to its material or to a declared mode record.
  let (r, b) = (&raw.prologue, &binding.prologue);
  let raw_only =
    (r.enter_horizontal && !b.enter_horizontal) || (r.leave_horizontal && !b.leave_horizontal);
  if binding.prologue_final {
    if r.enter_horizontal && !b.enter_horizontal {
      found.push(Mismatch::EnterHorizontalMissing);
    }
    if r.leave_horizontal && !b.leave_horizontal {
      found.push(Mismatch::LeaveHorizontalMissing);
    }
  } else if raw_only {
    found.push(Mismatch::PrologueUnknown);
  }
  if raw.prologue_final {
    if b.enter_horizontal && !r.enter_horizontal {
      found.push(Mismatch::EnterHorizontalExtra);
    }
    if b.leave_horizontal && !r.leave_horizontal {
      found.push(Mismatch::LeaveHorizontalExtra);
    }
  }
  let (raw_args, binding_args) = (&raw.args, &binding.args);
  let (mut i, mut j) = (0, 0);
  while i < raw_args.len() && j < binding_args.len() {
    match (&raw_args[i], &binding_args[j]) {
      (Arg::Optional(a), Arg::Optional(b)) => {
        if let (Some(a), Some(b)) = (a, b)
          && a != b
        {
          found.push(Mismatch::OptDefault {
            raw:     a.clone(),
            binding: b.clone(),
          });
        }
        i += 1;
        j += 1;
      },
      // A brace-delimited prefix (`\def\textcolor#1#{…}`, xcolor) is TeX's idiom for an optional
      // `[…]` before the first group: it reads the bracket, or nothing.
      (Arg::Delimited(until), Arg::Optional(_)) if until == "{" => {
        i += 1;
        j += 1;
      },
      (Arg::Optional(_), _) => {
        found.push(Mismatch::OptMissing);
        i += 1;
      },
      (_, Arg::Optional(_)) => {
        found.push(Mismatch::OptExtra);
        j += 1;
      },
      (Arg::Star, Arg::Star) => {
        i += 1;
        j += 1;
      },
      (Arg::Star, _) => {
        found.push(Mismatch::StarMissing);
        i += 1;
      },
      (_, Arg::Star) => {
        found.push(Mismatch::StarExtra);
        j += 1;
      },
      (Arg::Peek(a), Arg::Peek(b)) if a == b => {
        i += 1;
        j += 1;
      },
      (Arg::Peek(a), _) => {
        found.push(Mismatch::PeekMissing(a.clone()));
        i += 1;
      },
      (_, Arg::Peek(b)) => {
        found.push(Mismatch::PeekExtra(b.clone()));
        j += 1;
      },
      (Arg::Mandatory, Arg::Scan(kind)) => {
        found.push(Mismatch::ScanKind(kind.clone()));
        i += 1;
        j += 1;
      },
      (a, b) => {
        if a != b {
          found.push(Mismatch::Delim {
            raw:     describe(a),
            binding: describe(b),
          });
        }
        i += 1;
        j += 1;
      },
    }
  }
  // What is left over: optionals and stars by name, the rest as a difference in arity — when
  // both sides were read to the end.
  for arg in &raw_args[i..] {
    match arg {
      Arg::Optional(_) => found.push(Mismatch::OptMissing),
      Arg::Star => found.push(Mismatch::StarMissing),
      Arg::Peek(token) => found.push(Mismatch::PeekMissing(token.clone())),
      _ => {},
    }
  }
  for arg in &binding_args[j..] {
    match arg {
      Arg::Optional(_) => found.push(Mismatch::OptExtra),
      Arg::Star => found.push(Mismatch::StarExtra),
      Arg::Peek(token) => found.push(Mismatch::PeekExtra(token.clone())),
      _ => {},
    }
  }
  let counted = |args: &[Arg]| {
    args
      .iter()
      .filter(|a| !matches!(a, Arg::Optional(_) | Arg::Star | Arg::Peek(_)))
      .filter(|a| !matches!(a, Arg::Delimited(until) if until == "{"))
      .count()
  };
  let (raw_count, binding_count) = (counted(raw_args), counted(binding_args));
  if raw.complete && binding.complete && raw_count != binding_count {
    found.push(Mismatch::Arity {
      raw:     raw_count,
      binding: binding_count,
    });
  }
  found
}
