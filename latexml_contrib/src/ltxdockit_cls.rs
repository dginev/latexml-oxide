//! ltxdockit.cls — Philipp Lehman's "LaTeX documentation kit" class
//! (TL `texmf-dist/tex/latex/ltxdockit/ltxdockit.cls`, v1.2d 2011/03/25),
//! the class behind the biblatex-ecosystem manuals (biblatex, etoolbox,
//! abraces, … — 12+ TL doc bundles in the perfect-kernel corpus).
//!
//! Why a binding exists (user directive 2026-08-31): the raw class does
//! `\renewrobustcmd*{\titlepage}[1]{\setkeys{ltd@ttp}{#1}}`, but `\titlepage`
//! is a LOCKED kernel CS, so under raw interpretation the redefinition is
//! refused ("Ignoring redefinition of \titlepage") and the whole keyval
//! title machinery breaks (`undefined \ltd@title@title` ×6 per document;
//! Perl 0.8.8 fails identically, plus a fatal on some members). Locks stay
//! (user decision, same day) — the resolution for lock-conflicting classes
//! is a `_cls.rs` binding that sets the desired definitions explicitly from
//! the Rust layer, where binding loads run UNLOCKED.
//!
//! The mapping is SEMANTIC, not a replay of the print title page: the
//! `\titlepage` keys feed standard frontmatter (`\title`/`\author`/`\date`,
//! subtitle as `ltx:subtitle`, url/email/revision as classed notes) and
//! `\printtitlepage` becomes `\maketitle` — so the manuals get real
//! `<ltx:title>`/`<ltx:creator>` markup instead of a centered group.
//!
//! Base class: the real ltxdockit.def passes `11pt,a4paper,DIV9,…` to
//! scrartcl and loads it; we route to our scrartcl binding the same way.
//! The `ltxdockit` PACKAGE (ltxdockit.sty — lstnewenvironment-based example
//! envs, doc markup commands) has no binding and raw-loads exactly as
//! before; only the lock-conflicting CLASS layer is bound here.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("scrartcl");

  // ltxdockit.cls dependency block (L26-33). fontenc[T1]/hypcap options are
  // presentation-only; the packages themselves carry the definitions the
  // manuals use (`\href`, `\Hurl`, etoolbox's test/hook suite, multicols).
  RequirePackage!("etoolbox");
  RequirePackage!("multicol");
  RequirePackage!("keyval");
  RequirePackage!("textcomp");
  RequirePackage!("ltxdockit"); // the .sty half — raw-loads (no binding)
  RequirePackage!("hyperref");
  RequirePackage!("hypcap");

  // L38-39: \email{addr} → mailto link (the raw version goes through
  // \hyper@normalise, an href-internal we don't expose).
  DefMacro!("\\email{}", "\\href{mailto:#1}{#1}");
  // L35-36: \fnurl[pre]{url} — footnote with a linked URL.
  DefMacro!("\\fnurl[]{}", "\\footnote{#1\\url{#2}}");

  // L57-67: the keyval title page, set explicitly at the Rust layer (the
  // reason this binding exists — see module docs). Each key lands as real
  // frontmatter; \printtitlepage (L69-91) then just fires \maketitle.
  RawTeX!(
    r"\define@key{ltd@ttp}{title}{\title{#1}}
      \define@key{ltd@ttp}{subtitle}{\@add@frontmatter{ltx:subtitle}{#1}}
      \define@key{ltd@ttp}{url}{\@add@frontmatter{ltx:note}[role=url]{\url{#1}}}
      \define@key{ltd@ttp}{author}{\author{#1}}
      \define@key{ltd@ttp}{email}{\@add@frontmatter{ltx:note}[role=email]{\email{#1}}}
      \define@key{ltd@ttp}{revision}{\@add@frontmatter{ltx:note}[role=revision]{Version #1}}
      \define@key{ltd@ttp}{date}{\date{#1}}"
  );
  DefMacro!("\\titlepage{}", "\\setkeys{ltd@ttp}{#1}");
  DefMacro!("\\printtitlepage", "\\maketitle");
  def_macro_noop("\\titlefont")?;

  // L95-131: RCS tag extraction (`\rcsid{$Id: … $}` → \rcsfile /
  // \rcsrevision / \rcsdate / …). Faithful raw port — delimiter-matched
  // \defs, no lock conflicts — minus the `\write\@auxout` indirection: the
  // aux round-trip only exists so the values survive to the NEXT LaTeX run;
  // we define them directly in this one.
  RawTeX!(
    r"\providecommand*{\rcsfile}{[rcsfile]}
      \providecommand*{\rcsrevision}{[revision]}
      \providecommand*{\rcsdate}{[date]}
      \providecommand*{\rcstime}{[time]}
      \providecommand*{\rcsstate}{[state]}
      \providecommand*{\rcsauthor}{[author]}
      \providecommand*{\rcslocker}{[unlocked]}
      \providecommand*{\rcstoday}{\today}
      \providecommand*{\rcsid}[1]{\ifblank{#1}{}{\ltd@rcsid@i#1}}
      \def\ltd@rcsid@i$Id#1${\ifblank{#1}{}{\ltd@rcsid@ii#1&}}
      \def\ltd@rcsid@ii#1#2&{\ifblank{#1}{}{\ltd@rcsid@iii#2&}}
      \def\ltd@rcsid@iii#1 #2 #3 #4&{\gdef\rcsfile{#2}\gdef\rcsrevision{#3}\ltd@rcsid@iv#4&}
      \def\ltd@rcsid@iv#1/#2/#3 #4:#5:#6 #7&{\gdef\rcsdate{#1/#2/#3}\gdef\rcstime{#4:#5:#6}\ltd@rcsid@v#7&}
      \def\ltd@rcsid@v#1 #2 #3&{\gdef\rcsauthor{#1}\gdef\rcsstate{#2}\ifblank{#3}{}{\ltd@rcsid@vi#3&}}
      \def\ltd@rcsid@vi#1 &{\gdef\rcslocker{#1}}"
  );

  // ltxdockit.def L17-37: \AtBeginToc/\AtEndToc/\AtBeginLot/\AtEndLot hook
  // registries. Defined for real (documents may append to them); the def's
  // own use — wrapping the TOC in a 2-column multicols — is print layout and
  // deliberately not replayed.
  RawTeX!(
    r"\newcommand*{\@begintochook}{}\newcommand*{\@endtochook}{}
      \newcommand*{\@beginlothook}{}\newcommand*{\@endlothook}{}
      \newcommand*{\AtBeginToc}{\g@addto@macro\@begintochook}
      \newcommand*{\AtEndToc}{\g@addto@macro\@endtochook}
      \newcommand*{\AtBeginLot}{\g@addto@macro\@beginlothook}
      \newcommand*{\AtEndLot}{\g@addto@macro\@endlothook}"
  );

  // L135 (end of class): the DFSG config file — it defines the name/logo
  // macro suite the manuals use in running text (\etex, \latex, \lppl,
  // \biber, …) and appends to the Toc hooks, so it must load AFTER they
  // exist (real class order: ltxdockit.def first, cfg last). Raw-loads from
  // the texmf tree exactly as the real class does.
  RawTeX!(r"\InputIfFileExists{ltxdockit.cfg}{}{}");
});
