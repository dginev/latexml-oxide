//! Deferred label references: `\lx@label@ref{<key>}`, the label a binding knows only once later input is read (cas's
//! `\stmRef`, which LaTeX reads from the `.aux` of the run before), filled by the binding into one table
//! ([`set_label_ref`]) and read where the frontmatter is flushed. The flush's readers of the queued keyvals read the
//! label it stands for ([`resolve_label_refs`]), as the digestion's expansion of the macro does. Witness 2303.00352
//! (cas's `\author[aff1]` before its `\affiliation[aff1]`).

use super::*;

/// The macro a binding writes for a label recorded later ([`set_label_ref`]); it expands to that label, else to
/// nothing (`\lx@label@ref{<key>}`, defined in `defs/frontmatter_api.rs`).
pub const LABEL_REF: &str = "\\lx@label@ref";

fn label_ref_key(key: &str) -> String { s!("lx_label_ref:{}", key.trim()) }

/// Record `text` as the label `key` stands for, globally; a later record replaces it (the binding's current label,
/// `\@currentlabel`). A label a binding fills from an `.aux` round trip follows that binding's own redefinition rule
/// as LaTeX reads the `.aux` (in a group, latex.ltx:9486-9490: a global definition stays, a local one goes): the
/// first record wins under a `\newlabel`-like `\cs_new`-if-free (cas's `\NewLabel`, cas-common.sty:191-195,
/// [`set_label_ref_once`]).
pub fn set_label_ref(key: &str, text: &str) {
  assign_value(
    &label_ref_key(key),
    Stored::String(pin(text)),
    Some(Scope::Global),
  );
}

/// Record `text` as the label `key` stands for unless one is recorded already: the first record wins, as a label
/// LaTeX defines globally only when it is free and reads from the `.aux` in a group (cas's `\NewLabel`,
/// cas-common.sty:191-195; [`set_label_ref`]).
pub fn set_label_ref_once(key: &str, text: &str) {
  if label_ref_text(key).is_none() {
    set_label_ref(key, text);
  }
}

/// The label `key` stands for, once recorded ([`set_label_ref`]).
pub fn label_ref_text(key: &str) -> Option<String> {
  lookup_value(&label_ref_key(key)).map(|_| lookup_string(&label_ref_key(key)))
}

/// `\lx@label@ref{key}`: the label `key` stands for, read where it is expanded.
pub fn label_ref_tokens(key: &str) -> Vec<Token> {
  let mut out = vec![T_CS!(LABEL_REF), T_BEGIN!()];
  out.extend(mouth::tokenize_internal(TeXString::assembled(key.to_string())).unlist());
  out.push(T_END!());
  out
}

/// `text` (a queued entry's keyvals, `annotations={\lx@label@ref {a},b}`) with every `\lx@label@ref{key}` read as the
/// label it stands for, nothing for one never recorded, as the macro expands: the text the flush's readers of the
/// queue read, before the entries are digested.
pub fn resolve_label_refs(text: &str) -> Cow<'_, str> {
  if !text.contains(LABEL_REF) {
    return Cow::Borrowed(text);
  }
  let mut out = String::with_capacity(text.len());
  let mut rest = text;
  while let Some(at) = rest.find(LABEL_REF) {
    out.push_str(&rest[..at]);
    let after = &rest[at + LABEL_REF.len()..];
    let argument = after.trim_start();
    // (a longer control word, or no braced argument: no reference)
    let word_goes_on = after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '@');
    let end = (!word_goes_on && argument.starts_with('{'))
      .then(|| {
        let mut depth = 0i32;
        argument.char_indices().find_map(|(i, c)| {
          match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {},
          }
          (depth == 0).then_some(i)
        })
      })
      .flatten();
    match end {
      Some(end) => {
        out.push_str(&label_ref_text(&argument[1..end]).unwrap_or_default());
        rest = &argument[end + 1..];
      },
      None => {
        out.push_str(LABEL_REF);
        rest = after;
      },
    }
  }
  out.push_str(rest);
  Cow::Owned(out)
}

/// The queued entries' keyvals (`commands`, a flush's queue) with their label references read
/// ([`resolve_label_refs`]), for the flush's readers of them (`place.rs`'s `author_labels` and legends,
/// [`sequenced_contact_labels`]); the entries' tokens, which are digested, keep them.
pub(in crate::base_utilities) fn resolve_queued_label_refs(commands: &mut [RawFrontmatter]) {
  for entry in commands {
    for value in entry.1.values_mut() {
      if let Cow::Owned(resolved) = resolve_label_refs(value) {
        *value = resolved;
      }
    }
  }
}
