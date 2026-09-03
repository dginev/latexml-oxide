use crate::prelude::*;

// quantumview.cls raw-loads (a self-contained derivative of quantumarticle).
// Its :661 `\renewcommand{\author}[2][]{…\internal@elseauthor…}` cannot take
// effect — the kernel `\author` is locked in both engines to keep the
// frontmatter capture (Perl latex_constructs.pool:1079 identical) — so
// :673-680 `\internal@elseauthor`, which initialises the author-group lists
// (`\csdef{@authorgroup}{}`, `\listxadd`), never runs, and the raw
// `\maketitle` loop `\forlistloop{…}{\@authorgroup}` (quantumarticle.cls:1169)
// meets an undefined list (quantumview 8 errors; Perl 33; pdflatex clean).
// The binding initialises the lists empty so the raw layout loop is inert
// while the locked `\author` still captures the creators. Guard:
// `perfect_kernel_batch54::quantumview_author_group_lists_are_initialised`.
#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("quantumview", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RawTeX!(r"\makeatletter
\@ifundefined{@authorgroup}{\def\@authorgroup{}}{}
\@ifundefined{@authors}{\def\@authors{}}{}
\makeatother");
});
