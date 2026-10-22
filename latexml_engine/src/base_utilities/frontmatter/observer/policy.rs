//! The observer's policy table (Layer B of the design note's §2; OXIDIZED_DESIGN_DIVERGENCES #837): the association
//! conventions a binding family applies on top of the class-independent resolver (`resolver.rs`, Layer C), keyed on
//! (binding, hook) as the ledger records them at the call — the binding owning the `\author` meaning
//! (`ledger::author_binding`) and the `\lx@add@authors*` hook it called (notes decision 2: no policy keyval in the
//! binding files). Each entry reproduces a convention today's code already applies for that family, and cites the
//! OXIDIZED_DESIGN_DIVERGENCES entry or the witness it reproduces; a family with no entry takes [`KERNEL`], the
//! kernel's (Perl's `@authorsplits` reading, Base_Utility.pool.ltxml:676-700).

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

/// One family's association conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
  /// the entry's name (reported with every decision it made)
  pub name:             &'static str,
  /// the binding (`revtex4_support.sty`) and the hook (`\lx@add@authors@append`) it applies to: `*` any
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
  /// what the entry reproduces
  pub cite:             &'static str,
}

/// The kernel's conventions: every family without an entry.
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
  cite:             "Perl Base_Utility.pool.ltxml:676-700 (@authorsplits, positionOf); FIDELITY_CHARTER §4",
};

/// The table, first match wins (a binding's file name as the BINDINGS registry gives it, `<name>.<extension>`).
pub const POLICIES: &[Policy] = &[
  // revtex: `\affiliation` after a run of `\author`s is every author of the run without one (revtex4-2.cls's group
  // rule; Perl revtex4_support.sty.ltxml `annotate=new`) — declared by the binding at the call, `annotate=new`, which
  // the resolver's `api:annotate:new` honours. (Its title marks, revtex4-2.cls:3040-3043 `\frontmatter@thefootnote`,
  // are `\@fnsymbol` by default — :5976-5978 `\@booleanfalse\altaffilletter@sw` after :3043's default — and `\@alph`
  // under `altaffilletter`; the binding declares no title hook, so the kernel's fallback is the default's.)
  Policy {
    name: "revtex",
    binding: "revtex4_support.sty",
    cite: "revtex4_support_sty.rs:79-105 `\\lx@add@affiliation[annotate=new]` (Perl revtex4_support.sty.ltxml); \
           0911.0568",
    ..KERNEL
  },
  // authblk: an unlabelled `\author` shows `\AB@thenote` and an unlabelled `\affil` is labelled by it (ruling 1,
  // review revfms2a2's answer to question 6b): when the marks show, the affiliation is the authors' showing its
  // number, a labelled author's too (2605.25017); in block mode (no marks shown) the run of authors since the last
  // `\affil`, as the binding declares it at the call (`annotate=run:<n>`)
  Policy {
    name: "authblk",
    binding: "authblk.sty",
    thenote: true,
    state_counter: Some("Maxaffil"),
    cite: "authblk.sty:114-118, 148-167 (`\\AB@thenote`, `\\newaffil`, `Maxaffil`); Perl authblk.sty.ltxml:57 \
           `annotate=new`; authblk_sty.rs:221 (2605.01689, 2605.25017)",
    ..KERNEL
  },
  // llncs: `\institute{A \and B}` numbered by its `\and`s (`labelseq=affiliation`), `\inst` the request; an `\inst`
  // after a comma is the name's before it; an unmarked `\\` line under names is their affiliation
  Policy {
    name: "llncs",
    binding: "llncs.cls",
    inst_after_comma: true,
    and_inline: true,
    cite: "llncs.cls:870-885, 894-896; OD #791 (2405.16615, 2512.08621), #792 (2604.11038), #807 (2401.16957)",
    ..KERNEL
  },
  Policy {
    name: "aa",
    binding: "aa.cls",
    inst_after_comma: true,
    cite: "aa.cls:759-760 `\\def\\inst#1{\\unskip\\hbox{$^{#1}$}}`; OD #807 (1604.07826, 2309.00573)",
    ..KERNEL
  },
  Policy {
    name: "inst_support",
    binding: "inst_support.sty",
    inst_after_comma: true,
    cite: "Perl inst_support.sty.ltxml:33-35; OD #807 (2401.16957)",
    ..KERNEL
  },
  Policy {
    name: "svjour",
    binding: "sv_support.sty",
    inst_after_comma: true,
    cite: "svjour3.cls `\\inst` (`\\unskip`); OD #807",
    ..KERNEL
  },
  // amsart: `\and` prints "and", in an `\address` too (`\address{Einstein Institute … \and Rutgers}`, 1712.02568)
  Policy {
    name: "ams",
    binding: "ams_support.sty",
    and_word: true,
    cite: "amsart.cls:599 `\\def\\and`; Perl ams_support.sty.ltxml:94 `\\lx@author@conj`; 1712.02568",
    ..KERNEL
  },
  // the JHEP family: `\emailAdd` addresses printed as one shared line, placed by spelling, else by author order
  Policy {
    name: "jheppub",
    binding: "jheppub.sty",
    email_owner: EmailOwner::SpelledElseOrder,
    cite: "jheppub.sty:126-131; OD #805 (2407.18115, 2406.03543), #809 (1901.08773)",
    ..KERNEL
  },
  Policy {
    name: "jcappub",
    binding: "jcappub.sty",
    email_owner: EmailOwner::SpelledElseOrder,
    cite: "jcappub.sty (jheppub's `\\emailAdd`); OD #805 (1804.07528, 2407.07862)",
    ..KERNEL
  },
  Policy {
    name: "jinstpub",
    binding: "jinstpub.sty",
    email_owner: EmailOwner::SpelledElseOrder,
    cite: "OD #801, #805 (jinstpub's placement: steps 1 and 4)",
    ..KERNEL
  },
  // iopart: `\ead` by spelling, else author order, else the address mark, else the group's only author (each
  // `\author` a group)
  Policy {
    name: "iopart",
    binding: "iopart_support.sty",
    email_owner: EmailOwner::SpelledElseOrder,
    call_is_group: true,
    cite: "iopart.cls:225-231, 363-364; OD #806 (2311.12551), #808 (2407.14305, 2311.07087, 2311.17496), #809 \
           (2504.15015)",
    ..KERNEL
  },
  // report and book: their title page (`titlepage`, their default) prints the notes in the body's arabic numbers;
  // without one (`notitlepage`) the kernel declares the title hook (`declare_fnsymbol_title_block`), which wins
  Policy {
    name: "report",
    binding: "report.cls",
    title_marks: TitleMarks::Body,
    cite: "report.cls:211 (OD #851, `declare_fnsymbol_title_block`)",
    ..KERNEL
  },
  Policy {
    name: "book",
    binding: "book.cls",
    title_marks: TitleMarks::Body,
    cite: "book.cls:217 (OD #851, `declare_fnsymbol_title_block`)",
    ..KERNEL
  },
  Policy {
    name: "jpconf",
    binding: "jpconf.cls",
    email_owner: EmailOwner::SpelledElseOrder,
    call_is_group: true,
    cite: "jpconf.cls:104-110 (iopart's `\\address`/`\\ead`); OD #841 (0705.0202, 1011.0523)",
    ..KERNEL
  },
];

/// Whether a binding has an entry of its own (the class fallback's test, `ledger::policy_binding`).
pub fn has_entry(binding: &str) -> bool { POLICIES.iter().any(|p| p.binding == binding) }

/// The policy for a block recorded under `binding` (the `\author` owner) and `hook`: the table's first entry whose
/// binding and hook match (`*` any), else [`KERNEL`].
pub fn policy_for(binding: &str, hook: &str) -> &'static Policy {
  POLICIES
    .iter()
    .find(|p| (p.binding == "*" || p.binding == binding) && (p.hook == "*" || p.hook == hook))
    .unwrap_or(&KERNEL)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_entry_cites_what_it_reproduces() {
    for policy in POLICIES {
      assert!(!policy.cite.is_empty(), "{}", policy.name);
      assert!(
        policy.cite.contains("OD #") || policy.cite.contains(".ltxml"),
        "{}",
        policy.name
      );
    }
  }

  #[test]
  fn an_unknown_family_takes_the_kernel() {
    assert_eq!(
      policy_for("revtex4_support.sty", "\\lx@add@authors@append").name,
      "revtex"
    );
    assert_eq!(
      policy_for("llncs.cls", "\\lx@add@author@split").name,
      "llncs"
    );
    assert_eq!(
      policy_for("kernel", "\\lx@add@authors@adaptive").name,
      "kernel"
    );
    assert_eq!(
      policy_for("acmart.cls", "\\lx@add@authors@append").name,
      "kernel"
    );
  }
}
