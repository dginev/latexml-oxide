use crate::prelude::*;
LoadDefinitions!({
  // Perl: loads these packages
  RequirePackage!("refcount");
  RequirePackage!("gettitlestring");
  RequirePackage!("ltxcmds");

  // We want to display the actual "name" of the labeled structure (e.g. \section),
  //   which is accessible via show="title"
  //
  // TODO: The star forms prevent nested double links.
  // Perl L28 passes `enterHorizontal => 1` so \nameref in vertical mode
  // transitions into horizontal like \ref does. Rust was missing it, so
  // a bare `\nameref{…}` at the start of a cell or a potential vmode
  // position could skip the mode flip.
  DefConstructor!("\\nameref OptionalMatch:* Semiverbatim",
  "<ltx:ref ?#1(class='ltx_refmacro_nameref ltx_nolink')(class='ltx_refmacro_nameref')\
    show='title' labelref='#label' _force_font='true'/>",
  enter_horizontal => true,
  properties => sub[args] {
    let label_arg = args[1].as_ref().map(ToString::to_string).unwrap_or_default();
    Ok(stored_map!(
      "label" => clean_label(&label_arg, None)))
  });

  DefMacro!("\\Nameref", "\\nameref"); //\def\Nameref#1{‘\nameref{#1}’ on page~\pageref{#1}}
  DefMacro!("\\Sectionformat{}{}", "#1");
  DefMacro!("\\Ref", "\\ref"); // can be improved if "varioref.sty" is loaded?
  //The original nameref docs say: "Overload an AMS LaTEX command, which uses \newlabel. Sigh!"
  DefMacro!("\\slabel", "\\label");
  // We can improve if we had \vpageref
  DefMacro!("\\vnameref", "\\nameref");
  // nameref.sty:189-192, defined unconditionally: the title-capture hook. The
  // binding (Perl nameref.sty.ltxml too) reimplements `\nameref` and never
  // defined it, but memoir.cls:7020-7026 sets `\NR@nopatch@sectioning`,
  // requires nameref, and routes its own `\M@gettitle` (heads, `\PoemTitle`,
  // memoir.cls:3079/3147/3754/5376) through `\NR@gettitle` — srbook-mem ×3,
  // serbian-apostrophe ×2 had it as their sole error. Guard:
  // `perfect_kernel_batch54::nameref_gettitle_records_the_title`.
  DefMacro!(
    "\\NR@gettitle{}",
    "\\GetTitleString{#1}\\let\\@currentlabelname\\GetTitleStringResult"
  );
  RawTeX!(r"\providecommand*{\@currentlabelname}{}");
  // nameref.sty:297-314: `\NR@setref{<label>}<selector>{<label>}` hands the label's `\r@` entry to `\@setref`, whose
  // selector picks a field: `\@firstoffive` the number (`\T@ref`), `\@secondoffive` the page, `\@thirdoffive` the
  // title, `\NR@MakeUppercaseFirstOfFive` the capitalised number (`\T@Ref`). No label has an `\r@` entry while the
  // document is read (references resolve afterwards), so each selector is the reference command that prints its
  // field, and any other consumer the number. 1811.01873 builds its `\iref` on it (`\NR@setref{#1}\U@@ref{#1}`): it
  // read `\NR@setref` undefined, then `\U@@ref`'s five arguments from the text, a cascade of math errors.
  // `\@safe@activestrue` is provided as nameref.sty:300 does, for raw code that calls it.
  RawTeX!(
    r"\providecommand*\@safe@activestrue{}
\long\def\@thirdoffive#1#2#3#4#5{#3}\long\def\@fourthoffive#1#2#3#4#5{#4}\long\def\@fifthoffive#1#2#3#4#5{#5}
\def\NR@MakeUppercaseFirstOfFive#1#2#3#4#5{\MakeUppercase#1}
\def\NR@setref#1#2#3{\ifx#2\@secondoffive\def\lx@nameref@field{\pageref}%
\else\ifx#2\@thirdoffive\def\lx@nameref@field{\nameref}%
\else\ifx#2\NR@MakeUppercaseFirstOfFive\def\lx@nameref@field{\Ref}%
\else\def\lx@nameref@field{\ref}\fi\fi\fi\lx@nameref@field{#1}}"
  );
  // nameref.sty:352-360 redeclares `\ref`, `\pageref` and `\Ref` in the begindocument hook, so a preamble redefinition
  // never reaches the document: a self-recursive `\renewcommand{\ref}[1]{\hyperref[#1]{…\ref{#1}…}}` (2503.08060,
  // 1908.01329) or a class's `\def\ref#1{\hbox{\rref{#1}}}` looping through the document's `\rref` (1811.01873)
  // recursed endlessly here and in Perl. The chunk restores the kernel's `\ref` (this binding's `\T@ref`; with the
  // constructor its robust wrapper calls, which a `\DeclareRobustCommand\ref` replaces) under the `nameref` label,
  // so later packages' chunks and the document's own `\AtBeginDocument` code run after it, as in LaTeX.
  // KNOWN_PERL_ERRORS #519; guards `perfect_kernel_batch63::{preamble_ref_redefinition_reset_at_begin_document,
  // begin_document_ref_redefinition_survives_nameref}`.
  Let!("\\lx@nameref@Ref", "\\Ref");
  RawTeX!(
    r"\def\lx@nameref@reset{\let\ref\lx@kernel@ref\expandafter\let\csname ref \endcsname\lx@kernel@ref@
\let\pageref\lx@kernel@ref\let\Ref\lx@nameref@Ref}"
  );
  if lookup_definition(&T_CS!("\\AddToHook"))?.is_some() {
    RawTeX!(r"\AddToHook{begindocument}[nameref]{\lx@nameref@reset}");
  } else {
    at_begin_document(Tokens!(T_CS!("\\lx@nameref@reset")))?;
  }
});
