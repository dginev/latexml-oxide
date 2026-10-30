//! Binding for nature-pre.cls and nature_mod.cls, the copies papers ship of TL's nature.cls (the Nature pre-print
//! template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // The class's own list environments, verbatim (TL nature.cls:67-81 and :110-123, byte-identical in the
  // nature-pre.cls and nature_mod.cls copies papers ship: 2608.12563's nature-pre.cls:67-81 and :110-123,
  // 1702.00936's and 2402.00473's nature_mod.cls:74-88 and :117-130), so the list typesets the class's labels:
  // `{affiliations}`, put after `\maketitle`, numbers each institution `\upshape$^{\arabic{enumi}}$` (an `\item[*]`
  // its own mark) in a `\slshape` body (2608.12563, 1702.00936, 1408.2522, 2412.11966); `{addendum}` labels each item
  // through `\addendumlabel`, the bold `\item[Competing Interests]` (2402.00473, 2412.11966), the list's
  // "Acknowledgements" for a bare `\item` (1408.2522, 1702.00936; not bold: the kernel's `\list` sets its default
  // label without `\makelabel`, as Perl's latex_constructs.pool.ltxml:1642-1643 does). Emptied to nothing (the
  // former stubs), the items were plain paragraphs without their labels, and the affiliations read as body prose to
  // the cover rule (`cover_order.rs` `is_body_prose`), so a later abstract stayed above them.
  RawTeX!(
    r"\newenvironment{affiliations}{%
    \setcounter{enumi}{1}%
    \setlength{\parindent}{0in}%
    \slshape\sloppy%
    \begin{list}{\upshape$^{\arabic{enumi}}$}{%
        \usecounter{enumi}%
        \setlength{\leftmargin}{0in}%
        \setlength{\topsep}{0in}%
        \setlength{\labelsep}{0in}%
        \setlength{\labelwidth}{0in}%
        \setlength{\listparindent}{0in}%
        \setlength{\itemsep}{0ex}%
        \setlength{\parsep}{0in}%
        }
    }{\end{list}\par\vspace{12pt}}
\newenvironment{addendum}{%
    \setlength{\parindent}{0in}%
    \small%
    \begin{list}{Acknowledgements}{%
        \setlength{\leftmargin}{0in}%
        \setlength{\listparindent}{0in}%
        \setlength{\labelsep}{0em}%
        \setlength{\labelwidth}{0in}%
        \setlength{\itemsep}{12pt}%
        \let\makelabel\addendumlabel}
    }
    {\end{list}\normalsize}
\newcommand*{\addendumlabel}[1]{\textbf{#1}\hspace{1em}}"
  );
  // Preserve author content.
  DefMacro!(
    "\\correspondingauthor[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  DefMacro!(
    "\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}"
  );
});
