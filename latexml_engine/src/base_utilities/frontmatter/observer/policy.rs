//! The observer's conventions (Layer B of the design note's §2; OXIDIZED_DESIGN_DIVERGENCES #837): the association
//! conventions a binding family applies on top of the class-independent resolver (`resolver/`, Layer C). Each binding
//! declares its own at load ([`declare_frontmatter_conventions`], user ruling 2026-10-28 "in each binding", superseding
//! the design note's decision 2, "no policy keyval in the binding files"): the declaration is kept in the state under
//! the binding's file name, keyed on (binding, hook) as the ledger records them at the call — the binding owning the
//! `\author` meaning (`ledger::author_binding`, or the class's binding with a declaration, `ledger::policy_binding`)
//! and the `\lx@add@authors*` hook it called. Each declaration reproduces a convention the binding's family already
//! had and cites the OXIDIZED_DESIGN_DIVERGENCES entry or the witness it reproduces. The kernel's own blocks (`\author`
//! still LaTeXML's, under `article` or any class whose bindings declare none) take [`KERNEL`] (Perl's `@authorsplits`
//! reading, Base_Utility.pool.ltxml:676-700), written (OD #925); a binding owning `\author` that declares none takes the
//! same reading unwritten, `UNDECLARED`, until its family passes its own gate.

use std::rc::Rc;

use super::*;

/// How an e-mail no mark of its own places is owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailOwner {
  /// FIDELITY_CHARTER §4 (2026-10-21): the author it spells; else the group's only author; else the one author marked
  /// as the corresponding author; else the block.
  Chain,
  /// The class's address list printed apart from the names (`owner=sequence`): the author it spells; else, in a list
  /// with one address per author that every spelled address agrees with, the author at its place; else the chain
  /// (OXIDIZED_DESIGN_DIVERGENCES #805, #806, #808, #809).
  SpelledElseOrder,
}

/// What a bare `\footnotemark` in the block prints when no binding declared the title's marks (the kernel hook
/// `\lx@title@thefootnote`, `latex_constructs::TITLE_THEFOOTNOTE`, which wins when defined).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleMarks {
  /// The kernel `\maketitle`'s `\@fnsymbol\c@footnote` (article.cls:203-205).
  Fnsymbol,
  /// The class's own `\thefootnote`, as its title page prints the notes: report and book under `titlepage`, their
  /// default (report.cls:211, book.cls:217 change `\thefootnote` only in the `\if@titlepage` false branch), arabic.
  Body,
}

/// One family's association conventions, as its binding declares them ([`declare_frontmatter_conventions`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
  /// the declaration's name (reported with every decision it made)
  pub name:             &'static str,
  /// the declaring binding's file name, as the BINDINGS registry gives it (`<name>.<extension>`), and the hook
  /// (`\lx@add@authors@append`) it applies to: `*` any
  pub binding:          &'static str,
  pub hook:             &'static str,
  /// an `\inst` mark (the convention's own, the rehearsal's `\lx@fm@obs@mark`) after a comma is the name's before the
  /// comma, glued or not (llncs.cls:894-896 and aa.cls:759-760 `\def\inst#1{\unskip$^{#1}$}`; OD #807, 2401.16957);
  /// any other mark after a comma is the name's before only when glued to the comma
  pub inst_after_comma: bool,
  /// `\and` is inline text joining the names of one row, not a column (llncs.cls:1013-1018 `\def\and{\unskip,…}`
  /// in `\@maketitle`; OD #792: `Ann Able \and Bob Baker\\ University of Somewhere` is both names')
  pub and_inline:       bool,
  /// a `labelseq=<role>` entry is labelled by its place among the entries of that role (llncs's and aa's
  /// `\institute{A \and B}`, numbered 1, 2 as the class prints them: OD #791, #159)
  pub label_sequence:   bool,
  pub email_owner:      EmailOwner,
  /// each call is an author group of its own (an e-mail nothing else places is the group's only author's: iopart's
  /// `\author`, OD #808)
  pub call_is_group:    bool,
  pub title_marks:      TitleMarks,
  /// authblk's notes (authblk.sty:114-118, 148-151): an unlabelled `\author` shows `\AB@thenote`, the `affil` counter
  /// stepped at the first `\author` after an `\affil`, and an unlabelled `\affil` is labelled by it; the marks show
  /// when a label is given or the counter passes `\c@Maxaffil` (`state_counter`), else the affiliations print under
  /// the run of authors (block mode)
  pub thenote:          bool,
  /// a counter the policy reads at each block's call (the ledger's `counter`)
  pub state_counter:    Option<&'static str>,
  /// `\and` inside an annotation prints the word "and" (amsart.cls:599 `\def\and{\unskip{ }\@@and{ }\ignorespaces}`):
  /// the entry's text holds it
  pub and_word:         bool,
  /// the title's notes print no mark (IEEEtran.cls:4756 `\def\thefootnote{}` in `\maketitle`, "the \thanks{} mark type
  /// is empty"): a note stands beside a name only by its place, so its inner contacts are a name's only by spelling
  /// (main-session decision 2 of 2026-10-22, user ruling 2026-10-20 #4)
  pub unmarked_notes:   bool,
  /// the class's `\email` prints its address where it is called (llncs.cls:897 `\def\email#1{{\tt#1}}`, aa.cls:824
  /// `e-mail: {\aa@emailfont #1}`, svjour3.cls:303, svmult.cls:2304, svjour.cls:256): one inside a document's note
  /// (`\offprints{O. Kochukhov, \email{…}}`, a `\thanks`) prints in the note, after the name the note gives, which
  /// designates its owner — while the `\email` meaning is one of the class's bindings' (`ledger::email_meaning`, read at
  /// the call; `Resolver::answer_document_notes`). Not amsart's (amsart.cls:538 `\def\email` stores the address for
  /// the author's address block) nor REVTeX's (the author's footnote): there the address is the call's current
  /// author's, as the binding routes it
  pub email_in_place:   bool,
  /// the observer writes this family's frontmatter (FM S2b, `writer/`): the family passed its per-family gate — its own
  /// seeded classified-precision sample ≥ 95 % right and its unclassified rate < 0.5 % (main-session decision at int63,
  /// OXIDIZED_DESIGN_DIVERGENCES #837 "S2a.2 r5 at landing"); a paper is written only when every place its output and
  /// the observer's reading differ is one of the writer's operations (`writer::plan`)
  pub written:          bool,
  /// what the declaration reproduces
  pub cite:             &'static str,
}

/// The kernel's conventions: the blocks of the kernel's `\author` under a class whose bindings declare none (`article`
/// and every class without a frontmatter binding of its own), written (FM stage 7, OD #925: the writer applies the
/// reading where its operations explain every difference, else today's output stays); and the template a declaration
/// completes (`..KERNEL`, each declaration stating its own `written`).
pub const KERNEL: Policy = Policy {
  name:             "kernel",
  binding:          "*",
  hook:             "*",
  inst_after_comma: false,
  and_inline:       false,
  label_sequence:   true,
  email_owner:      EmailOwner::Chain,
  call_is_group:    false,
  title_marks:      TitleMarks::Fnsymbol,
  thenote:          false,
  state_counter:    None,
  and_word:         false,
  unmarked_notes:   false,
  email_in_place:   false,
  written:          true,
  cite:             "Perl Base_Utility.pool.ltxml:676-700 (@authorsplits, positionOf); FIDELITY_CHARTER §4; OD #925",
};

/// The conventions of a binding owning the `\author` meaning that declares none of its own (JHEP.cls, acmart.cls,
/// mn2e_support.sty, …), or of a meaning no binding owns (the document's own, a file read raw, the fallback class's):
/// the kernel's reading, not written — the observer writes a family only once its own gate passed (OD #837 S2b).
pub const UNDECLARED: Policy = Policy {
  name: "undeclared",
  written: false,
  cite: "Perl Base_Utility.pool.ltxml:676-700 (@authorsplits, positionOf); OD #925",
  ..KERNEL
};

/// The owner name the ledger gives the kernel's `\author` (`ledger::meaning_owner`, `ledger::policy_binding`).
pub const KERNEL_BINDING: &str = "kernel";

/// The conventions for a block of `binding` when it declared none: the kernel's for the kernel's own blocks, else
/// `UNDECLARED`.
fn undeclared(binding: &str) -> &'static Policy {
  if binding == KERNEL_BINDING {
    &KERNEL
  } else {
    &UNDECLARED
  }
}

/// The state key a binding's declarations are kept under.
fn declarations_key(binding: &str) -> String { s!("frontmatter_conventions:{binding}") }

/// The declarations `binding` made, in declaration order, if any.
fn declared(binding: &str) -> Option<Rc<Vec<&'static Policy>>> {
  with_value(&declarations_key(binding), |v| match v {
    Some(Stored::Opaque(payload)) => Rc::clone(payload).downcast::<Vec<&'static Policy>>().ok(),
    _ => None,
  })
}

/// Declare a binding's frontmatter conventions (the binding's load, before anything else it defines): kept in the
/// state, globally, under `conventions.binding` for the hook `conventions.hook` (`*` any); a binding may declare one
/// per hook (the first matching, in declaration order, applies), and a later declaration for the same hook replaces the
/// earlier in its place (a binding loaded again declares the same). What `policy_for` reads.
pub fn declare_frontmatter_conventions(conventions: &'static Policy) {
  debug_assert!(
    !conventions.binding.is_empty() && conventions.binding != "*",
    "{}: conventions are declared for the declaring binding's file",
    conventions.name
  );
  debug_assert!(
    conventions.cite.contains("OD #") || conventions.cite.contains(".ltxml"),
    "{}: a declaration cites the entry or the Perl binding it reproduces",
    conventions.name
  );
  let mut all: Vec<&'static Policy> =
    declared(conventions.binding).map_or_else(Vec::new, |d| d.to_vec());
  match all.iter_mut().find(|p| p.hook == conventions.hook) {
    Some(earlier) => *earlier = conventions,
    None => all.push(conventions),
  }
  assign_value(
    &declarations_key(conventions.binding),
    Stored::Opaque(Rc::new(all)),
    Some(Scope::Global),
  );
}

/// Whether a binding declared conventions of its own (the class fallback's test, `ledger::policy_binding`).
pub fn has_entry(binding: &str) -> bool { declared(binding).is_some() }

/// Whether a binding declared that its class's `\email` prints in place ([`Policy::email_in_place`], any of its hooks).
pub fn declares_email_in_place(binding: &str) -> bool {
  declared(binding).is_some_and(|d| d.iter().any(|p| p.email_in_place))
}

/// The conventions among `declared` for a block recorded under `binding` (the `\author` owner) and `hook`: the first
/// declaration whose binding and hook match (`*` any), else the kernel's for the kernel's own blocks, `UNDECLARED`
/// for any other — `policy_for`'s rule, for a caller holding
/// the declarations itself (`ab_audit fmshadow --re-resolve`).
pub fn conventions_among<'a>(
  declared: impl IntoIterator<Item = &'a &'static Policy>,
  binding: &str,
  hook: &str,
) -> &'static Policy {
  declared
    .into_iter()
    .copied()
    .find(|p| p.binding == binding && (p.hook == "*" || p.hook == hook))
    .unwrap_or_else(|| undeclared(binding))
}

/// The conventions for a block recorded under `binding` (the `\author` owner) and `hook`: the first that binding
/// declared whose hook matches (`*` any), else the kernel's for the kernel's own blocks, `UNDECLARED` for any other.
pub fn policy_for(binding: &str, hook: &str) -> &'static Policy {
  declared(binding).map_or_else(
    || undeclared(binding),
    |d| conventions_among(d.iter(), binding, hook),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A synthetic binding's declarations: one for every hook, one for a hook of its own (declared first, so it wins
  /// for that hook, as the first matching declaration does).
  const ANY_HOOK: Policy = Policy {
    name: "synthetic",
    binding: "synthetic_binding",
    and_inline: true,
    cite: "OD #837 (a synthetic binding: the declaration's lookup)",
    ..KERNEL
  };
  const APPEND_HOOK: Policy = Policy {
    name: "synthetic_append",
    hook: "\\lx@add@authors@append",
    and_word: true,
    ..ANY_HOOK
  };

  #[test]
  fn a_declaration_is_read_and_an_undeclared_family_takes_the_kernel_unwritten() {
    assert_eq!(
      policy_for("synthetic_binding", "\\lx@add@author@split").name,
      "undeclared"
    );
    assert!(!has_entry("synthetic_binding"));
    declare_frontmatter_conventions(&APPEND_HOOK);
    declare_frontmatter_conventions(&ANY_HOOK);
    assert!(has_entry("synthetic_binding"));
    assert_eq!(
      policy_for("synthetic_binding", "\\lx@add@authors@append").name,
      "synthetic_append"
    );
    assert_eq!(
      policy_for("synthetic_binding", "\\lx@add@author@split").name,
      "synthetic"
    );
    assert_eq!(
      policy_for("kernel", "\\lx@add@authors@adaptive").name,
      "kernel"
    );
    assert_eq!(
      policy_for("undeclared_binding", "\\lx@add@authors@append").name,
      "undeclared"
    );
    assert!(policy_for("kernel", "\\lx@add@authors@adaptive").written);
    assert!(!policy_for("undeclared_binding", "\\lx@add@authors@append").written);
    // (a declaration again for a hook replaces the earlier in its place: a binding loaded again)
    declare_frontmatter_conventions(&APPEND_HOOK);
    assert_eq!(
      policy_for("synthetic_binding", "\\lx@add@authors@append").name,
      "synthetic_append"
    );
  }
}
