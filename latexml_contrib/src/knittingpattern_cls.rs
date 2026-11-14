//! knittingpattern.cls: the class is loaded raw, and the pattern's copyright line — `\cpyrght{<text>}`, which the
//! class prints only in the page foot (`\fancyfoot[R]{#1}`, knittingpattern.cls:60-68), a page style LaTeXML never
//! typesets — also becomes a frontmatter `note` of role `copyright`, at the command. The foot stays dropped as page
//! furniture; a copyright notice is a semantic note (user ruling 2026-10-01). SHARED: Perl with raw classes drops it
//! alike. Witness knittingpattern/template. Guard `perfect_kernel_batch61::class_copyright_lines_are_frontmatter_notes`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("knittingpattern", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RawTeX!(
    r"\let\lx@knit@cpyrght\cpyrght
\long\def\cpyrght#1{\lx@add@frontmatter{ltx:note}[role=copyright]{#1}\lx@knit@cpyrght{#1}}"
  );
});
