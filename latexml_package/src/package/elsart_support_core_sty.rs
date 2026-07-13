//! elsart_support_core.sty — Elsevier journal article support (core)
//! Perl: elsart_support_core.sty.ltxml — 191 lines
//! Shared by elsart.cls and elsarticle.cls
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Frontmatter environment (Perl PR #2767)
  DefPrimitive!(T_CS!("\\begin{frontmatter}"), None, None);
  DefPrimitive!(T_CS!("\\end{frontmatter}"), None, None);

  // \author[mark]{name}
  // One \author per author! possibly with a mark to connect to affiliation
  // It either should be followed by \affiliation,
  // or both should get matching marks
  DefMacro!("\\author OptionalSemiverbatim {}",
    "\\lx@add@creator[role=author,annotations={#1}]{#2}");

  DefMacro!("\\address OptionalSemiverbatim {}",
    "\\lx@add@contact[label={#1},role=address]{#2}");
  // \affiliation[label]{text-OR-keyvals} !!  (Perl PR #2767)
  DefMacro!("\\affiliation OptionalSemiverbatim {}",
    "\\lx@add@contact[label={#1},role=affiliation]{\\lx@els@parse@affiliation{#2}{#2}}");

  // Detect if affiliation is just text, or keyvals; if latter, format into text.
  DefMacro!("\\lx@els@parse@affiliation {} RequiredKeyVals", sub[(raw, data)] {
    let pairs: Vec<(String, ArgWrap)> = data.get_pairs().cloned().collect();
    // 1 key, novalue: No Keyvals at all!
    if pairs.len() <= 1
      && pairs.first().is_none_or(|(_, v)| matches!(v, ArgWrap::None))
    {
      Ok(raw)
    } else {
      let mut affil: Vec<Token> = Vec::new();
      for (key, value) in pairs {
        if matches!(key.as_str(),
          "o" | "or" | "organization"
          | "a" | "ad" | "addressline"
          | "c" | "ci" | "city"
          | "p" | "pc" | "postcode"
          | "s" | "st" | "state"
          | "country")
        {
          if !affil.is_empty() {
            affil.push(T_OTHER!(","));
            affil.push(T_SPACE!());
          }
          match value {
            ArgWrap::Tokens(tks) => affil.extend(tks.unlist()),
            ArgWrap::None => {},
            other => affil.extend(other.revert()?.unlist()),
          }
        }
      }
      Ok(Tokens::new(affil))
    }
  });
  // Redefine to account for the label, which we ignore for now!
  DefConstructor!("\\thanks[]{}", "<ltx:note role='thanks'>#2</ltx:note>");

  // Is this significantly different?
  // Perl elsart_support_core.sty.ltxml: body is `\author{#1}` but in
  // the `OptionalMatch:* {}` signature `#1` is the star flag and `#2` is
  // the content — the author name is silently dropped. Documented as a
  // Perl bug in docs/parity/KNOWN_PERL_ERRORS.md #16 (same root cause as
  // aipproc.cls.ltxml's \tablenote). Rust deliberately uses `#2` so the
  // author content reaches \author correctly.
  DefMacro!("\\collab OptionalMatch:* {}", "\\author{#2}");
  Let!("\\collaboration", "\\collab");

  // These pairs add various contact information to authors.  (Perl PR #2767)
  // The \<XXX>ref forms are used within the \author text to anchor the connection
  // the \<XXX>text forms supply the text (& and role) for the contact that will be attached.
  DefMacro!("\\thanksref{}",  "\\lx@request@frontmatter@annotation[thanks]{#1}");
  DefMacro!("\\corref{}",     "\\lx@request@frontmatter@annotation[cor]{#1}");
  DefMacro!("\\corauthref{}", "\\lx@request@frontmatter@annotation[corauth]{#1}");
  DefMacro!("\\fnref{}",      "\\lx@request@frontmatter@annotation[fn]{#1}");
  DefMacro!("\\tnoteref{}",   "\\lx@request@frontmatter@annotation[tnote]{#1}"); // Possibly for title, not author?
  DefMacro!("\\thanks OptionalSemiverbatim {}",
    "\\lx@add@contact[label={thanks:#1},role=thanks]{#2}");
  DefMacro!("\\cortext OptionalSemiverbatim {}",
    "\\lx@add@contact[label={cor:#1},role=correspondent]{#2}");
  DefMacro!("\\corauth OptionalSemiverbatim {}",
    "\\lx@add@contact[label={corauth:#1},role=correspondent]{#2}");
  DefMacro!("\\fntext OptionalSemiverbatim {}",
    "\\lx@add@contact[label={fn:#1},role=note]{#2}");
  DefMacro!("\\tnotetext OptionalSemiverbatim {}",
    "\\lx@add@contact[label={tnote:#1},role=note]{#2}"); // title note?

  // Title/metadata (Perl PR #2767)
  // \runauthor / \runtitle are running-header SHORT forms (real elsart.cls
  // L1235 just stores them for `\@oddhead`; never typeset in the body). Perl
  // gobbles both — layout-only and redundant with `\author`/`\title`. Prior
  // over-preservation digested the running-head content, so an author typo
  // (stray `\` before a name) hit undefined-CS. Witness 1503.06349.
  def_macro_noop("\\runauthor{}")?;
  def_macro_noop("\\runtitle{}")?;
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  // \ead[label]{email} provides email address for the preceding \author (no marks used)
  DefMacro!("\\ead Optional:email Semiverbatim",
    "\\lx@add@contact[role=#1]{#2}");
  DefMacro!("\\sep", "\\unskip,\\space");
  DefMacro!("\\received{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\revised{}", "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\communicated{}", "\\lx@add@date[role=communicated]{#1}");
  DefMacro!("\\dedicated{}", "\\lx@add@pubnote[role=dedication]{#1}");
  DefMacro!("\\presented{}", "\\lx@add@date[role=presented]{#1}");
  DefMacro!("\\articletype{}", "\\lx@add@pubnote[role=type]{#1}");
  DefMacro!("\\issue{}", "\\lx@add@pubnote[role=issue]{#1}");
  DefMacro!("\\journal{}", "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\lx@add@pubnote[role=volume]{#1}");
  DefMacro!("\\pubyear{}", "\\lx@add@date[role=publication]{#1}");
  def_macro_noop("\\FullCopyrightText")?;
  DefMacro!("\\copyear{}", "\\lx@add@copyrightyear{#1}");
  DefMacro!("\\copyrightholder{}", "\\lx@add@copyrightholder{#1}");
  Let!("\\copyrightyear", "\\copyear");
  def_macro_noop("\\RUNART")?;
  def_macro_noop("\\RUNDATE")?;
  def_macro_noop("\\RUNJNL")?;
  // Round-34 surpass-Perl: company/article-id are author metadata.
  DefMacro!("\\company{}",
    "\\lx@add@frontmatter{ltx:note}[role=company]{#1}");
  DefMacro!("\\aid{}",
    "\\lx@add@frontmatter{ltx:note}[role=article-id]{#1}");
  def_macro_noop("\\ssdi{}{}")?;
  def_macro_noop("\\readRCS Until:$ Until:$")?;
  def_macro_noop("\\RCSdate")?;
  def_macro_noop("\\RCSfile")?;
  def_macro_noop("\\RCSversion")?;
  DefMacro!("\\firstpage{}",
    "\\lx@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  DefMacro!("\\lastpage{}",
    "\\lx@add@frontmatter{ltx:note}[role=lastpage]{#1}");
  def_macro_noop("\\preface")?;
  def_macro_noop("\\theHaddress")?;
  def_macro_noop("\\theaddress")?;
  Let!("\\ESpagenumber", "\\arabic");

  // Acknowledgements — Perl L123-125
  DefConstructor!("\\ack", "<ltx:acknowledgements>");
  DefConstructor!("\\endack", "</ltx:acknowledgements>");

  // Acknowledgements tag — Perl L125
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).

  // Keywords — Perl L130-153
  // keyword environment and macros with XUntil pattern
  // Perl L135-152: \begin{keyword}/\end{keyword} use DefMacroI with begingroup/endgroup,
  // NOT DefEnvironment!, to properly scope the XUntil delimiter reading.
  DefMacro!(T_CS!("\\begin{keyword}"), None, "\\begingroup\\@keyword");
  DefMacro!(T_CS!("\\end{keyword}"), None, "\\@keyword@cut\\endgroup");
  // Perl elsart_support_core.sty.ltxml L138-141: `\keyword{...}` (1-arg
  // form) reads a balanced group and wraps with `\@keyword <arg>
  // \@keyword@cut` so XUntil terminates at the inserted token.
  //
  // History — DO NOT switch this back to `read_balanced(_, _, false)`:
  // commit 09bc60c2 ("\keyword absorbs trailing unbalanced }") tried to
  // mirror Perl's `$gullet->readBalanced` with no args (require_open=
  // false → level starts at 1) to absorb the legacy unbalanced-`}`
  // idiom from arXiv:1710.03688 / hep-ph0702114, where the abstract
  // ends:
  //     \keyword{Keyword1; ...} \vskip ...\noindent{...e3}}
  // (a trailing literal `}` past the keyword arg). For that input
  // alone, `require_open=false` correctly consumes `{...} }` and stops.
  //
  // BUT for the common balanced form — `\keyword{Higgs; Boson}` with
  // no trailing extra `}` (the test fixture
  // `tests/babel/elsart_keyword_brace_form.tex` and the vast majority
  // of real-world usage) — `require_open=false` reads past the
  // matching `}`, walks through `\end{abstract}` (raw `\end` is a CS
  // and CC_END count is unchanged across that expansion), then
  // `\end{frontmatter}`, … all the way to EOF. The reader's pushback
  // grows unboundedly on the way; in CI we observed a single
  // `memory allocation of 21743271936 bytes failed` (21 GB) → SIGABRT
  // before the test could even report. The cost of one paper's
  // 1-error reduction is the entire test suite OOM-aborting at
  // `81_babel::elsart_keyword_brace_form_test`.
  //
  // RESOLVED 2026-05-30 via the design note's first suggestion: keep the
  // strict balanced read of the keyword argument, then peek past spaces and
  // gobble an OPTIONAL trailing `}` (catcode END). This reproduces Perl's
  // lenient `readBalanced` *result* (it absorbs the legacy unbalanced `}`)
  // for the trailing-`}` idiom — abstract bodies that end
  //     \keyword{Kw1; Kw2; …}   <comments/spaces>   }   \end{abstract}
  // (the stray `}` is the orphaned close of a commented-out brace group;
  // witness 1601.01227, also 1710.03688 / hep-ph0702114) — WITHOUT the
  // speculative read-to-EOF that OOM'd the balanced fixture: for the common
  // `\keyword{Higgs; Boson}\end{abstract}` form the token after the strict
  // read is `\end` (a CS, not `}`), so nothing is gobbled. The peek is
  // bounded to one token, never crossing `\end`.
  // Skip leading spaces and blank-line `\par`s, then gobble a single trailing
  // `}` (catcode END) if one is there — Perl's `readBalanced` reads straight
  // through the `\par` from a blank line before the stray brace (witness
  // 1705.01354 separates `\keyword{…}` and the orphaned `}` by a blank line).
  // The skipped whitespace/`\par` tokens are buffered and FULLY RESTORED when
  // no trailing `}` follows, so the common `\keyword{…}\end{abstract}` form
  // (and `\keyword{…}<blank>\end{abstract}`) is left byte-for-byte untouched.
  // Bounded: stops at the first non-space, non-`\par` token; never crosses
  // `\end`.
  // Perl's `readBalanced` (L140) reads tokens up to the next UNMATCHED `}`
  // (tracking brace depth), absorbing that stray `}` as its terminator —
  // regardless of what non-brace material (spaces, `\par`, `\vskip <dimen>`,
  // `\noindent`, …) sits between `\keyword{…}` and the orphaned brace. The
  // earlier version only skipped spaces/`\par`s, so it stopped at the first
  // real token (e.g. `\vskip`) and left the stray `}` to hit the abstract's
  // mode-switch frame (`ltx:para`-free `}` → "close a group that switched to
  // mode internal_vertical"; witness 1604.00855:
  // `\keyword{…} \vskip 0.5\baselineskip}`).
  //
  // Walk forward tracking brace depth: an unmatched `}` (depth 0) is the
  // stray terminator — DROP it, re-inject everything else (so the `\vskip`
  // etc. still render, content-preserving). Bound the scan at `\end` (depth
  // 0) — the env terminator — and a hard token cap, so the common
  // `\keyword{Higgs; Boson}\end{abstract}` form (no stray `}`) reads one
  // token (`\end`), restores it, and is left untouched (this is also what
  // averts the read-to-EOF OOM a naive `readBalanced` hit). Witnesses:
  // 1604.00855, 1601.01227, 1705.01354, 1710.03688, hep-ph0702114.
  DefPrimitive!("\\lx@elsart@gobble@optbrace", {
    let mut skipped: Vec<Token> = Vec::new();
    let mut depth: i32 = 0;
    let mut count: usize = 0;
    while let Some(tok) = read_token()? {
      count += 1;
      if count > 4096 {
        skipped.push(tok); // safety cap: restore everything, gobble nothing
        break;
      }
      let cc = tok.get_catcode();
      if cc == Catcode::BEGIN {
        depth += 1;
        skipped.push(tok);
      } else if cc == Catcode::END {
        if depth == 0 {
          break; // unmatched stray `}` — drop it (absorbed, like readBalanced)
        }
        depth -= 1;
        skipped.push(tok);
      } else if depth == 0 && tok == T_CS!("\\end") {
        skipped.push(tok); // env end reached before any stray `}` — restore all
        break;
      } else {
        skipped.push(tok);
      }
    }
    // Re-inject everything we buffered, in order. The only token ever removed
    // is the stray `}` (which is never pushed onto `skipped`).
    for tok in skipped.into_iter().rev() {
      unread_one(tok);
    }
  });
  DefMacro!("\\keyword{}", "\\@keyword #1 \\@keyword@cut\\lx@elsart@gobble@optbrace");
  DefMacro!("\\endkeyword", "\\@keyword@cut");
  DefMacro!("\\PACS", "\\@keyword@cut\\@PACS");
  DefMacro!("\\MSC[]", "\\@keyword@cut\\@MSC{#1}");
  DefMacro!("\\JEL", "\\@keyword@cut\\@JEL");
  DefMacro!("\\UK", "\\@keyword@cut\\@UK");

  // Perl L148-152: @keyword reads until @keyword@cut delimiter using XUntil.
  // XUntil expands tokens while reading, so \end{keyword} → \@keyword@cut is found.
  DefConstructor!("\\@keyword@cut", "");
  DefMacro!("\\@keyword XUntil:\\@keyword@cut", "\\lx@add@keywords{#1}");
  DefMacro!("\\@PACS XUntil:\\@keyword@cut", "\\lx@add@classification[scheme=PACS,name={PACS:~}]{#1}");
  DefMacro!("\\@MSC{} XUntil:\\@keyword@cut", "\\lx@add@classification[scheme={#1 MSC},name={MSC:~}]{#2}");
  DefMacro!("\\@JEL XUntil:\\@keyword@cut", "\\lx@add@classification[scheme=JEL,name={JEL:~}]{#1}");
  DefMacro!("\\@UK XUntil:\\@keyword@cut", "\\lx@add@classification[scheme=UK,name={UK:~}]{#1}");

  // Document structure — Perl L158-163
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");

  // Per-section equation numbering — Perl L161-163.
  // Emit `\@addtoreset{equation}{section}` + per-section
  // `\theequation` when the `seceqn` class option is active.
  if lookup_bool("@seceqn") {
    RawTeX!(r"\@addtoreset{equation}{section}");
    DefMacro!("\\theequation", "\\thesection.\\arabic{equation}");
  }

  // Theorems — Perl elsart_support_core.sty.ltxml L168-175.
  // Perl conditional on `@seceqn` flag (set by elsart.cls's `seceqn`
  // class option):
  //   if @seceqn:  `\newtheorem{thm}{Theorem}[section] \@addtoreset{thm}{section}`
  //   else      :  `\newtheorem{thm}{Theorem}`
  // Then aliases `\newdefinition` and `\newproof` to `\newtheorem`
  // (elsdoc §7).
  // The base `\newtheorem{thm}` declaration was missing in the prior
  // Rust port — every `\begin{thm}` in elsart papers reported
  // `{thm} undefined`. Driver paper: math0611842 (3 errors → 0).
  if lookup_bool("@seceqn") {
    RawTeX!(r"\newtheorem{thm}{Theorem}[section]\@addtoreset{thm}{section}");
  } else {
    RawTeX!(r"\newtheorem{thm}{Theorem}");
  }
  Let!("\\newdefinition", "\\newtheorem");
  Let!("\\newproof", "\\newtheorem");

  // Registers — Perl L180-183
  DefRegister!("\\eqnarraycolsep" => Dimension!("1pt"));
  DefRegister!("\\eqnbaselineskip" => Glue!("14pt"));
  DefRegister!("\\eqnlineskip" => Glue!("2pt"));
  DefRegister!("\\eqntopsep" => Glue!("12pt"));

  // Figures — Perl L186-191
  def_macro_noop("\\printfigures{}")?;
  def_macro_noop("\\printtables{}")?;
  def_macro_noop("\\MARK{}")?;
  def_macro_noop("\\mpfootnotemark")?;

  // Perl elsart_support_core.sty.ltxml L189:
  //   DefMacro('\note{}', "<ltx:note>#1</ltx:note>");    # ?
  // This is a *DefMacro* (token expansion), NOT a DefConstructor — so the
  // body tokenises to LITERAL TEXT (`<`, `>` are catcode-OTHER): `\note{X}`
  // expands to the characters `<ltx:note>` + X + `</ltx:note>`, NOT a real
  // <ltx:note> element. Perl's own `# ?` flags it as questionable, but it is
  // the ground truth and crucially is ERROR-FREE: a block argument such as
  // `\note{\begin{remark}…\end{remark}}` (a `\newtheorem`-based environment)
  // renders the remark as a normal <ltx:theorem> bracketed by stray literal
  // text, with no content-model violation. Porting it as a DefConstructor
  // (real <ltx:note>) instead made `ltx:theorem isn't allowed in <ltx:note>`
  // — papers that wrap a theorem-like env in `\note{…}` (common with a
  // user `\newcommand\note[1]{…}` that LaTeX/Perl *ignore* because elsart
  // already defined `\note`) then failed. Match Perl: use DefMacro.
  // Witness 2006.06087 (elsarticle, `\note{\begin{remark}…}`): 1 error → 0.
  DefMacro!("\\note{}", "<ltx:note>#1</ltx:note>");

  // Float environment
  DefEnvironment!("{esmark}",  "#body");
  def_macro_noop("\\figmark{}{}")?;
  def_macro_noop("\\tabmark{}{}")?;

  // \qed (proof end-of-proof marker). Previously only in elsart_support
  // (NOT loaded by elsarticle.cls — only by elsart.cls), so plain
  // elsarticle papers using \qed got `Error:undefined:\qed`. Move the
  // def here so all elsart_*-loading classes (elsarticle + elsart)
  // have it. Witness 2306.02411 (elsarticle Pfaffian paper).
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed");
});
