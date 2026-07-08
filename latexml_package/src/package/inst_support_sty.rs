use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: inst_support.sty.ltxml (PR #2767)
  // This bit of code supports the \inst style institution markup
  // used by several document classes and styles [aa, llncs, sv]
  // HOWEVER, the care support is now built into Engine/Base_Utility
  // so this shouldn't be needed.

  // In inst style, the \author (used once for each author)
  // gets some "labels" which are used to connect to \affiliation
  // (also one per affiliation); the affiliation with the matching label
  // is attached to the author.

  // \author[marks]{author}
  // Perl inst_support.sty.ltxml L33 defines `\author{}` — a SINGLE-arg macro
  // that DROPS the optional `[marks]` its own comment documents, and whose
  // `\lx@clear@creators` wipes prior authors on every call. A class using the
  // inst convention of one `\author[label]{name}` PER author (ifacconf:
  // `\author[First]{Eryn Vaid}` ×4) then reads the literal `[` as the name and
  // keeps only the last — Perl and Rust both emit a single `[` personname
  // (shared upstream bug; witness arXiv:2605.00004, whose pdflatex PDF lists all
  // four authors). Surpass-Perl (OXIDIZED_DESIGN #53):
  //   * accept the optional `[marks]` so `[` is never mistaken for the name,
  //   * take the name from #2 and split it on \and/\And/comma as before,
  //   * ACCUMULATE across calls — drop the per-call `\lx@clear@creators`, which
  //     is a no-op on the first call anyway, so single-`\author` classes are
  //     unaffected (and aa/llncs/sv define their own `\author` regardless).
  // The `[marks]` (author↔affiliation label) are dropped, matching Perl's own
  // handling; wiring them to the affiliation annotation is a separate follow-up.
  DefMacro!("\\author[]{}",
    "\\lx@splitting{\\lx@add@author}{\\and\\And,}{#2}");
  DefMacro!("\\institute{}",
    "\\lx@clear@frontmatter{ltx:contact}[role=affiliation]\\lx@splitting{\\lx@add@contact[role=affiliation,labelseq=affiliation]}{\\and\\And}{#1}");
  DefMacro!("\\inst{}", "\\lx@request@frontmatter@annotation[affiliation]{#1}");

  // \and variants — Perl L41-45
  Let!("\\at", "\\and"); // Actually this is different than \and, but...
  Let!("\\iand", "\\and");
  Let!("\\nand", "\\and");
  Let!("\\lastand", "\\and");
  Let!("\\AND", "\\and");
});
