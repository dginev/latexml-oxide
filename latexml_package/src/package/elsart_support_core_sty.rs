//! elsart_support_core.sty — Elsevier journal article support (core)
//! Perl: elsart_support_core.sty.ltxml — 191 lines
//! Shared by elsart.cls and elsarticle.cls
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Frontmatter environment
  DefEnvironment!("{frontmatter}", "#body");

  // Author/affiliation — Perl L32-48
  DefMacro!("\\author[]{}", "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#2}}");
  DefMacro!("\\address[]{}", "\\lx@contact{address}{#2}");
  // \affiliation[label]{key=val,...} — elsarticle uses this for institutions
  // Not in Perl elsart_support_core, but needed for modern elsarticle papers.
  // The TeX elsarticle.cls uses LaTeX3 \keys_set:nn to parse key-value pairs
  // (organization, addressline, city, postcode, state, country) and concatenate
  // the values with comma separators. We parse the key-value body in Rust
  // and produce clean affiliation text.
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affiliation[]{}", sub[(_opt_label, body)] {
    // Parse key-value pairs from the body, respecting brace nesting.
    // Keys: organization, addressline, city, postcode, state, country
    // Values are typically brace-delimited: key={value with, commas}
    let body_str = body.to_string();
    let mut parts: Vec<String> = Vec::new();

    // State machine to parse key=value pairs with brace nesting
    let chars: Vec<char> = body_str.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
      // Skip whitespace and commas between key-value pairs
      while i < len && (chars[i].is_whitespace() || chars[i] == ',') {
        i += 1;
      }
      if i >= len { break; }

      // Read key (until '=' or end)
      let key_start = i;
      while i < len && chars[i] != '=' {
        i += 1;
      }
      if i >= len { break; }
      // Use char-vec slicing (NOT byte slicing on body_str) — affiliation
      // text often contains UTF-8 multi-byte chars (accented names,
      // diacritics) where the char-count `i` is not a byte boundary.
      // Driver: 2407.00104 panic at `body_str[val_start..i]`.
      let _key: String = chars[key_start..i].iter().collect::<String>().trim().to_lowercase();
      i += 1; // skip '='

      // Skip whitespace after '='
      while i < len && chars[i].is_whitespace() {
        i += 1;
      }
      if i >= len { break; }

      // Read value — may be brace-delimited or plain
      let value;
      if i < len && chars[i] == '{' {
        // Brace-delimited value: read until matching '}'
        i += 1; // skip opening '{'
        let val_start = i;
        let mut depth = 1;
        while i < len && depth > 0 {
          if chars[i] == '{' { depth += 1; }
          else if chars[i] == '}' { depth -= 1; }
          if depth > 0 { i += 1; }
        }
        value = chars[val_start..i].iter().collect::<String>().trim().to_string();
        if i < len { i += 1; } // skip closing '}'
      } else {
        // Plain value: read until next comma or end
        let val_start = i;
        while i < len && chars[i] != ',' {
          i += 1;
        }
        value = chars[val_start..i].iter().collect::<String>().trim().to_string();
      }

      // Only include known affiliation keys with non-empty values
      let trimmed = value.trim();
      if !trimmed.is_empty() {
        match _key.as_str() {
          "organization" | "organisation" | "o" | "or"
          | "addressline" | "a" | "ad"
          | "city" | "c" | "ci"
          | "postcode" | "p" | "pc"
          | "state" | "s" | "st"
          | "country" | "cy" => {
            parts.push(trimmed.to_string());
          }
          _ => {
            // Unknown keys: include value as-is (matching elsarticle unknown handler)
            if !trimmed.is_empty() {
              parts.push(trimmed.to_string());
            }
          }
        }
      }
    }

    let affil_text = parts.join(", ");
    let mut result = Vec::new();
    result.push(T_CS!("\\@add@to@frontmatter"));
    result.push(T_BEGIN!());
    for ch in "ltx:creator".chars() {
      result.push(Token { text: arena::pin_char(ch), code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    });
    }
    result.push(T_END!());
    result.push(T_BEGIN!());
    result.push(T_CS!("\\@@@affiliation"));
    result.push(T_BEGIN!());
    for ch in affil_text.chars() {
      if ch == ' ' {
        result.push(T_SPACE!());
      } else if ch.is_ascii_alphabetic() {
        result.push(Token { text: arena::pin_char(ch), code: Catcode::LETTER,
      #[cfg(feature = "token-locators")] loc: 0
    });
      } else {
        result.push(Token { text: arena::pin_char(ch), code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    });
      }
    }
    result.push(T_END!());
    result.push(T_END!());
    result
  });
  DefConstructor!("\\thanks[]{}", "<ltx:note role='thanks'>#2</ltx:note>");
  // Perl L38-39: \person@thanks — inline (restricted_horizontal) variant used for
  // author-embedded thanks marks. Aliased to \thanks in reversion.
  DefConstructor!("\\person@thanks[]{}",
    "^ <ltx:contact role='thanks'>#2</ltx:contact>",
    alias => "\\thanks", mode => "restricted_horizontal");
  // \thanksref / \corref / \corauthref carry footnote labels. Round-34
  // surpass-Perl: emit as superscript so the labels reach the author
  // block (matches IEEE \IEEEauthorrefmark behavior).
  // \thanksref / \corref / \corauthref take label-style args that may contain
  // `_` (and other key-style chars). Read as Semiverbatim so `_` doesn't
  // trigger "Script _ can only appear in math mode" when the label flows
  // into \textsuperscript's body. Witness 2304.14608 (elsarticle):
  // `\author[..]{Xu\corref{corresponding_author}}` triggered the cascade.
  DefMacro!("\\thanksref Semiverbatim", "\\textsuperscript{#1}");
  DefMacro!("\\corauth[] Semiverbatim", "\\lx@contact{correspondent}{#2}");
  DefMacro!("\\corref Semiverbatim", "\\textsuperscript{#1}");
  DefMacro!("\\corauthref Semiverbatim", "\\textsuperscript{#1}");
  // \cortext[id]{text} carries author-typed corresponding-author text.
  // Preserve as ltx:note frontmatter so the prose ("Corresponding
  // author. Email: …") reaches the XML rather than being silently
  // dropped. Content-preserving.
  DefMacro!("\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}");
  // Perl elsart_support_core.sty.ltxml L47: body is `\author{#1}` but in
  // the `OptionalMatch:* {}` signature `#1` is the star flag and `#2` is
  // the content — the author name is silently dropped. Documented as a
  // Perl bug in docs/KNOWN_PERL_ERRORS.md #16 (same root cause as
  // aipproc.cls.ltxml's \tablenote). Rust deliberately uses `#2` so the
  // author content reaches \author correctly.
  DefMacro!("\\collab OptionalMatch:* {}", "\\author{#2}");
  Let!("\\collaboration", "\\collab");
  // Perl L50-51: route through lx@notetext for proper footnote handling
  DefMacro!("\\tnotetext[]{}", "\\lx@notetext[#1]{footnote}{#2}");
  DefMacro!("\\fntext[]{}", "\\lx@notetext[#1]{footnote}{#2}");
  // Perl L52-58: \lx@elsart@noteref splits comma-separated labels
  // into individual \lx@notemark[label]{footnote} calls
  DefMacro!("\\lx@elsart@noteref{}", sub[(labels)] {
    let label_str = labels.to_string();
    let mut result = Vec::new();
    for label in label_str.split(',') {
      let label = label.trim();
      if !label.is_empty() {
        result.push(T_CS!("\\lx@notemark"));
        result.push(T_OTHER!("["));
        for ch in label.chars() {
          result.push(Token { text: arena::pin_char(ch), code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    });
        }
        result.push(T_OTHER!("]"));
        result.push(T_BEGIN!());
        // "footnote" as OTHER tokens
        for ch in "footnote".chars() {
          result.push(Token { text: arena::pin_char(ch), code: Catcode::OTHER,
      #[cfg(feature = "token-locators")] loc: 0
    });
        }
        result.push(T_END!());
      }
    }
    result
  });
  DefMacro!("\\tnoteref{}", "\\lx@elsart@noteref{#1}");
  DefMacro!("\\fnref{}", "\\lx@elsart@noteref{#1}");

  // Title/metadata — Perl L60-106
  // \runauthor / \runtitle are running-header SHORT forms (real elsart.cls
  // L1235 `\def\runauthor#1{\gdef\@runauthor{#1}}` just stores them for
  // `\@oddhead`; never typeset in the body). Perl elsart_support_core.sty.ltxml
  // L60-61 GOBBLES both (`DefMacro('\runauthor{}', Tokens())`) — they are
  // layout-only and redundant with `\author`/`\title` (which preserve the full
  // author/title). The prior Rust over-preservation digested the running-head
  // content, so an author typo like `\runauthor{… T.\Pasurek/Journal…}`
  // (a stray `\` before a name) hit `undefined:\Pasurek`. Gobble to match Perl;
  // no author material is lost (`\author` keeps it). Same class as the
  // `\shortauthors` gobble fix. Witness 1503.06349.
  def_macro_noop("\\runauthor{}")?;
  def_macro_noop("\\runtitle{}")?;
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");
  DefMacro!("\\ead Optional:email Semiverbatim",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{#2}}");
  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#1'>#2</ltx:contact>");
  DefMacro!("\\sep", "\\unskip,\\space");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\communicated{}", "\\@add@frontmatter{ltx:date}[role=communicated]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedicated]{#1}");
  DefMacro!("\\presented{}", "\\@add@frontmatter{ltx:date}[role=presented]{#1}");
  DefMacro!("\\articletype{}", "\\@add@frontmatter{ltx:note}[role=articletype]{#1}");
  DefMacro!("\\issue{}", "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\journal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:date}[role=publication]{#1}");
  def_macro_noop("\\FullCopyrightText")?;
  DefMacro!("\\copyear{}", "\\@add@frontmatter{ltx:date}[role=copyright]{#1}");
  DefMacro!("\\copyrightholder{}", "\\@add@frontmatter{ltx:note}[role=copyrightholder]{#1}");
  Let!("\\copyrightyear", "\\copyear");
  def_macro_noop("\\RUNART")?;
  def_macro_noop("\\RUNDATE")?;
  def_macro_noop("\\RUNJNL")?;
  // Round-34 surpass-Perl: company/article-id are author metadata.
  DefMacro!("\\company{}",
    "\\@add@frontmatter{ltx:note}[role=company]{#1}");
  DefMacro!("\\aid{}",
    "\\@add@frontmatter{ltx:note}[role=article-id]{#1}");
  def_macro_noop("\\ssdi{}{}")?;
  def_macro_noop("\\readRCS Until:$ Until:$")?;
  def_macro_noop("\\RCSdate")?;
  def_macro_noop("\\RCSfile")?;
  def_macro_noop("\\RCSversion")?;
  DefMacro!("\\firstpage{}",
    "\\@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  DefMacro!("\\lastpage{}",
    "\\@add@frontmatter{ltx:note}[role=lastpage]{#1}");
  def_macro_noop("\\preface")?;
  def_macro_noop("\\theHaddress")?;
  def_macro_noop("\\theaddress")?;
  Let!("\\ESpagenumber", "\\arabic");

  // Acknowledgements — Perl L123-125
  DefConstructor!("\\ack", "<ltx:acknowledgements>");
  DefConstructor!("\\endack", "</ltx:acknowledgements>");

  // Acknowledgements tag — Perl L125
  Tag!("ltx:acknowledgements", auto_close => true);

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
    while let Some(tok) = gullet::read_token()? {
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
      gullet::unread_one(tok);
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
  DefMacro!("\\@keyword XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\@PACS XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");
  DefMacro!("\\@MSC{} XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme={#1 MSC}]{#2}");
  DefMacro!("\\@JEL XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=JEL]{#1}");
  DefMacro!("\\@UK XUntil:\\@keyword@cut", "\\@add@frontmatter{ltx:classification}[scheme=UK]{#1}");

  // Document structure — Perl L158-163
  DefMacro!("\\theparagraph", "\\thesubsubsection.\\arabic{paragraph}");
  DefMacro!("\\thesubparagraph", "\\theparagraph.\\arabic{subparagraph}");

  // Per-section equation numbering — Perl L161-163.
  // Emit `\@addtoreset{equation}{section}` + per-section
  // `\theequation` when the `seceqn` class option is active.
  if state::lookup_bool("@seceqn") {
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
  if state::lookup_bool("@seceqn") {
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
