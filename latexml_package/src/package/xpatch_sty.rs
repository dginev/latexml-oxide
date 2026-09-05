//! xpatch.sty — etoolbox patching that also reaches robust/`\protect`ed commands.
//!
//! **Why a native binding.** xpatch is an expl3 package: every public command is
//! a `\NewDocumentCommand` dispatching to `\xpatch_main:NN`, which stringifies the
//! target with `\cs_replacement_spec:N` and then re-reads the body delimited by a
//! sentinel token list, `\c__xpatch_bizarre_tl` = `**)-(**/**]-[**`. Raw-loading
//! it makes that delimited scan run to end-of-file, so **everything after the
//! first `\xpatchcmd` is silently discarded** — no error, no output.
//!
//! Witness **2605.25157**: `\xpatchcmd{\@tocline}…` in the preamble; the paper's
//! own `\begin{thebibliography}` with 33 `\bibitem`s never digested, and the
//! document was truncated mid-proof at line 1292 of 1749 with **zero** errors
//! reported. Perl truncates identically, but at least raises
//! `Error:expected:Until:**)-(**/**]-[**` twice — our silence was strictly worse.
//! 10 papers in the 2605+2606 bibliography-absence residual load xpatch.
//!
//! **Why the mapping is this simple.** xpatch exists to pierce one indirection:
//! a command declared `\DeclareRobustCommand`/`\newrobustcmd` keeps its body in
//! `\<name><space>`, which etoolbox's `\patchcmd` cannot see. **That indirection
//! does not exist here** — `latex_base.rs`'s `\DeclareRobustCommand` marks the
//! macro itself `robust => true` (as does etoolbox's `\newrobustcmd`), so there
//! is no inner macro to redirect to and each `\x…` command is exactly its
//! etoolbox counterpart applied to the same derived control-sequence name.
//! `\patchcmd` already reports a native-definition target with an `Info`, so the
//! failure mode is a diagnostic rather than a runaway.
//!
//! Perl has no xpatch binding, so this is surpass-Perl; ground truth is the arXiv
//! PDF. Audit `docs/parity/BIB_ABSENCE_AUDIT_2026-07-29.md`.
use crate::prelude::*;

LoadDefinitions!({
  // xpatch.sty:42 `\RequirePackage{xparse,etoolbox}`: xparse.sty restores
  // ltcmd's legacy `g`/`G`/`l`/`u` argument types (latex.ltx:2287-2308 reject
  // them otherwise) — prtec.cls:316 `\NewDocumentCommand\entry{m g}` under
  // newtxtext→xpatch (RUST-ONLY). Guard:
  // `perfect_kernel_batch56::xpatch_loads_xparse_for_legacy_arg_types`.
  RequirePackage!("xparse");
  RequirePackage!("etoolbox");

  // RawTeX! (not TeX!) because every derived name has `@` as a letter
  // (`abx@macro@…`, `blx@bbx@…`); TeX! tokenizes at compile time with `@` as
  // OTHER and would split them apart.
  //
  // The `[1]`/`[2][*]` argument counts mirror xpatch's own `m` and `O{*} m`
  // signatures (xpatch.sty L125-192); the trailing
  // `{<search>}{<replace>}{<success>}{<failure>}` are left in the stream for
  // the etoolbox command to read, exactly as `\xpatch_main:N{N,c}` does.
  RawTeX!(
    r"
% \x{patch,preto,appto,show}cmd — op applied straight to the given command.
% One argument, so both `\xpatchcmd\foo` and `\xpatchcmd{\foo}` work, matching
% the single-token `#2` of \xpatch_main:NN.
\newcommand{\xpatchcmd}[1]{\patchcmd{#1}}
\newcommand{\xpretocmd}[1]{\pretocmd{#1}}
\newcommand{\xapptocmd}[1]{\apptocmd{#1}}
\newcommand{\xshowcmd}[1]{\show#1}

% biblatex bibmacros: abx@macro@<name>
\newcommand{\xpatchbibmacro}[1]{\expandafter\patchcmd\csname abx@macro@#1\endcsname}
\newcommand{\xpretobibmacro}[1]{\expandafter\pretocmd\csname abx@macro@#1\endcsname}
\newcommand{\xapptobibmacro}[1]{\expandafter\apptocmd\csname abx@macro@#1\endcsname}
\newcommand{\xshowbibmacro}[1]{\expandafter\show\csname abx@macro@#1\endcsname}

% biblatex field formats: abx@ffd@<type>@<name>, default type `*`
\newcommand{\xpatchfieldformat}[2][*]{\expandafter\patchcmd\csname abx@ffd@#1@#2\endcsname}
\newcommand{\xpretofieldformat}[2][*]{\expandafter\pretocmd\csname abx@ffd@#1@#2\endcsname}
\newcommand{\xapptofieldformat}[2][*]{\expandafter\apptocmd\csname abx@ffd@#1@#2\endcsname}
\newcommand{\xshowfieldformat}[2][*]{\expandafter\show\csname abx@ffd@#1@#2\endcsname}

% name formats: abx@nfd@<type>@<name>
\newcommand{\xpatchnameformat}[2][*]{\expandafter\patchcmd\csname abx@nfd@#1@#2\endcsname}
\newcommand{\xpretonameformat}[2][*]{\expandafter\pretocmd\csname abx@nfd@#1@#2\endcsname}
\newcommand{\xapptonameformat}[2][*]{\expandafter\apptocmd\csname abx@nfd@#1@#2\endcsname}
% NOTE: upstream xpatch.sty L153 reads `abx@ffd@` here, not `abx@nfd@` — an
% upstream typo, reproduced faithfully. Harmless: \show only prints a meaning.
\newcommand{\xshownameformat}[2][*]{\expandafter\show\csname abx@ffd@#1@#2\endcsname}

% list formats: abx@lfd@<type>@<name>
\newcommand{\xpatchlistformat}[2][*]{\expandafter\patchcmd\csname abx@lfd@#1@#2\endcsname}
\newcommand{\xpretolistformat}[2][*]{\expandafter\pretocmd\csname abx@lfd@#1@#2\endcsname}
\newcommand{\xapptolistformat}[2][*]{\expandafter\apptocmd\csname abx@lfd@#1@#2\endcsname}
\newcommand{\xshowlistformat}[2][*]{\expandafter\show\csname abx@lfd@#1@#2\endcsname}

% index field formats: abx@fid@<type>@<name>
\newcommand{\xpatchindexfieldformat}[2][*]{\expandafter\patchcmd\csname abx@fid@#1@#2\endcsname}
\newcommand{\xpretoindexfieldformat}[2][*]{\expandafter\pretocmd\csname abx@fid@#1@#2\endcsname}
\newcommand{\xapptoindexfieldformat}[2][*]{\expandafter\apptocmd\csname abx@fid@#1@#2\endcsname}
\newcommand{\xshowindexfieldformat}[2][*]{\expandafter\show\csname abx@fid@#1@#2\endcsname}

% index name formats: abx@nid@<type>@<name>
\newcommand{\xpatchindexnameformat}[2][*]{\expandafter\patchcmd\csname abx@nid@#1@#2\endcsname}
\newcommand{\xpretoindexnameformat}[2][*]{\expandafter\pretocmd\csname abx@nid@#1@#2\endcsname}
\newcommand{\xapptoindexnameformat}[2][*]{\expandafter\apptocmd\csname abx@nid@#1@#2\endcsname}
\newcommand{\xshowindexnameformat}[2][*]{\expandafter\show\csname abx@nid@#1@#2\endcsname}

% index list formats: abx@lid@<type>@<name>. `\xappindextolistformat` is
% upstream's own name for the append variant (xpatch.sty L182) — the odd word
% order is theirs, kept so documents that call it keep working.
\newcommand{\xpatchindexlistformat}[2][*]{\expandafter\patchcmd\csname abx@lid@#1@#2\endcsname}
\newcommand{\xpretoindexlistformat}[2][*]{\expandafter\pretocmd\csname abx@lid@#1@#2\endcsname}
\newcommand{\xappindextolistformat}[2][*]{\expandafter\apptocmd\csname abx@lid@#1@#2\endcsname}
\newcommand{\xshowindexlistformat}[2][*]{\expandafter\show\csname abx@lid@#1@#2\endcsname}

% biblatex bibliography drivers: blx@bbx@<name>
\newcommand{\xpatchbibdriver}[1]{\expandafter\patchcmd\csname blx@bbx@#1\endcsname}
\newcommand{\xpretobibdriver}[1]{\expandafter\pretocmd\csname blx@bbx@#1\endcsname}
\newcommand{\xapptobibdriver}[1]{\expandafter\apptocmd\csname blx@bbx@#1\endcsname}
\newcommand{\xshowbibdriver}[1]{\expandafter\show\csname blx@bbx@#1\endcsname}
"
  );
});
