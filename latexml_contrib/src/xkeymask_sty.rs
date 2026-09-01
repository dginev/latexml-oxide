use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // xkeymask.sty (Ramkumar Ramachandra, 2022-2023) — an xkeyval extension
  // that "masks" keys so `\setkeys` ignores them. No Perl LaTeXML binding
  // exists; the raw package works under real TeX but relies on `\XKV@resa`
  // surviving across `\KV@@sp@def` calls — keyval.tex's `\KV@@sp@def`
  // clobbers `\XKV@resa` via `\futurelet` (keyval.tex L41), so xkeymask's
  // accumulator ends up as the self-referential `macro:->\XKV@resa ,…`
  // (verified with pdflatex `\meaning`). Real TeX shrugs it off because the
  // walk only ever expands it one level at a time; our (and Perl LaTeXML's)
  // recursion guard reports `Error:recursion:\XKV@resa` at every masked
  // `\setkeys`. The binding provides the same user surface on clobber-safe
  // `\XKM@…` scratch names instead — observable behavior matches pdflatex
  // (mask honored, unmasked keys applied, no diagnostics).
  //
  // Witness: doc/latex/xkeymask/xkeymask.tex (perfect-kernel sweep 12).
  RequirePackage!("xkeyval");

  RawTeX!(r"\ProvidesPackage{xkeymask}[v1.0 An extension of xkeyval with a mask]");

  // Real xkeymask takes its `prefix` option through kvoptions
  // (`\DeclareStringOption{prefix}` + `\ProcessKeyvalOptions*`); the mask
  // machinery is only activated when the option is present. Parse the
  // key=value option with a catch-all handler (geometry_sty.rs pattern).
  RawTeX!(
    r"\def\XKM@prefix{}
\def\XKM@grab#1=#2\@nil{\def\XKM@key{#1}\XKM@grabv#2\@nil}
\def\XKM@grabv#1=#2\@nil{\def\XKM@val{#1}}
\DeclareOption*{%
  \expandafter\XKM@grab\CurrentOption==\@nil
  \def\XKM@tempc{prefix}%
  \ifx\XKM@key\XKM@tempc\global\let\XKM@prefix\XKM@val\fi
}
\ProcessOptions\relax"
  );

  // The mask machinery — xkeymask.sty L36-113, on `\XKM@…` scratch names.
  // `\appendmask`/`\ifinmask` keep the real package's xkeyval front-end
  // (`\XKV@testopta{\XKV@testoptc …}`), including `\XKM@ifinmask`'s
  // per-family×key branch execution. `\XKM@setkeys` filters masked keys out
  // of the list and hands the survivors to `\XKV@s@tkeys`, exactly like the
  // original; only the accumulator names differ (see module comment).
  RawTeX!(
    r"\ifx\XKM@prefix\@empty\else
  \def\XKM@mask{}
  \long\def\XKM@appendmask[#1]{%
    \XKV@for@o\XKV@fams\XKV@tfam{%
      \xdef\XKM@hdr{\XKV@prefix\XKV@tfam @}%
      \XKV@for@n{#1}\XKM@tempa{%
        \expandafter\KV@@sp@def\expandafter\XKM@tempa\expandafter{\XKM@tempa}%
        \ifx\XKM@mask\@empty\xdef\XKM@mask{\XKM@hdr\XKM@tempa}%
        \else\xdef\XKM@mask{\XKM@mask,\XKM@hdr\XKM@tempa}\fi
      }%
    }%
  }
  \def\appendmask{\XKV@testopta{\XKV@testoptc\XKM@appendmask}}
  \long\def\XKM@ifinmask[#1]#2#3{%
    \XKV@checksanitizea{#1}\XKM@tempb
    \XKV@for@o\XKV@fams\XKV@tfam{%
      \XKV@for@o\XKM@tempb\XKM@tempa{%
        \xdef\XKM@hdr{\XKV@prefix\XKV@tfam @}%
        \xdef\XKM@tempd{\XKV@prefix\XKV@tfam @\XKM@tempa}%
        \@expandtwoargs\in@\XKM@tempd\XKM@mask
        \ifin@#2\else#3\fi
      }%
    }%
  }
  \def\ifinmask{\XKV@testopta{\XKV@testoptc\XKM@ifinmask}}
  \def\clearmask{\global\let\XKM@mask\@empty}
  \long\def\XKM@setkeys[#1]#2{%
    \XKV@checksanitizea{#2}\XKM@resb
    \let\XKM@filtered\@empty
    \XKV@for@o\XKV@fams\XKV@tfam{%
      \XKV@for@o\XKM@resb\XKM@tempb{%
        \expandafter\XKV@g@tkeyname\XKM@tempb=\@nil\XKM@tempc
        \expandafter\KV@@sp@def\expandafter\XKM@tempc\expandafter{\XKM@tempc}%
        \xdef\XKM@tempd{\XKV@prefix\XKV@tfam @\XKM@tempc}%
        \@expandtwoargs\in@\XKM@tempd\XKM@mask
        \ifin@\else\XKV@addtolist@o\XKM@filtered\XKM@tempb\fi
      }%
    }%
    \ifnum\XKV@depth=\z@\let\XKV@rm\@empty\fi
    \expandafter\XKV@s@tkeys\expandafter{\XKM@filtered}{#1}%
    \let\CurrentOption\@empty
  }
  \long\def\XKM@setkeys@dispatch{%
    \xdef\XKM@tempa{\XKM@prefix @}%
    \ifx\XKV@prefix\XKM@tempa
      \expandafter\XKM@setkeys
    \else
      \expandafter\XKV@setkeys
    \fi
  }
  \def\setkeys{\XKV@testopta{\XKV@testoptc\XKM@setkeys@dispatch}}
\fi"
  );
});
