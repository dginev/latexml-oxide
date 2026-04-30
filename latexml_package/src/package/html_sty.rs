use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: html.sty.ltxml — 111 lines
  // LaTeX2HTML compatibility package

  RequirePackage!("hyperref");

  // Link/navigation macros — Perl L31-44
  DefMacro!("\\latextohtml",                              "\\LaTeX2\\texttt{HTML}");
  DefMacro!("\\htmladdnormallinkfoot{}{}",                "\\href{#2}{#1}");
  DefMacro!("\\htmladdnormallink{}{}",                    "\\href{#2}{#1}");
  DefMacro!("\\htmladdimg{}",                             "\\hyperimage{#1}");
  DefMacro!("\\externallabels Semiverbatim Semiverbatim", "");
  DefMacro!("\\externalref{}",                            "");
  DefMacro!("\\externalcite",                             "\\nocite");
  DefMacro!("\\htmladdTOClink[]{}{}{}",                   "");
  DefConstructor!("\\htmlrule OptionalMatch:*", "<ltx:rule/>");
  DefConstructor!("\\HTMLrule OptionalMatch:*", "<ltx:rule/>");
  DefConstructor!("\\htmlclear",                "<ltx:br/>");
  DefMacro!("\\bodytext{}", "");
  DefMacro!("\\htmlbody",   "");

  // Hyperref variants — Perl L45-51
  // Perl emits labelref='#label' on ltx:ref and pulls label from arg 4
  // via CleanLabel. Rust stub was `<ltx:ref>#1</ltx:ref>` — the ref
  // was emitted but without a label, so prior the link was inert.
  DefConstructor!("\\hyperrefdef{}{}{} Semiverbatim",
    "<ltx:ref labelref='#label'>#1</ltx:ref>",
    properties => sub[args] {
      let label_arg = args[3].as_ref().map(ToString::to_string).unwrap_or_default();
      Ok(stored_map!("label" => clean_label(&label_arg, None)))
    });
  Let!("\\hyperrefhyper", "\\hyperrefdef");
  Let!("\\hyperrefpagedef", "\\hyperrefdef");
  Let!("\\hyperrefnoref", "\\hyperrefdef");
  Let!("\\hyperrefhtml", "\\hyperrefdef");

  // Perl L53-56: \hypercite[pre]{key1}{key2}[post] Semiverbatim
  // emits an <ltx:cite> with a nested <ltx:bibref>, including optional
  // prefix/suffix phrases. Prior Rust stub was DefMacro!("","") which
  // silently swallowed all content.
  DefConstructor!("\\hypercite[]{}{}[] Semiverbatim",
    "<ltx:cite>#4 <ltx:bibref bibrefs='#5'>?#2(<ltx:bibrefphrase>#2</ltx:bibrefphrase>)</ltx:bibref> #1</ltx:cite>",
    enter_horizontal => true);
  DefMacro!("\\htmlcite{}{}", "\\hypercite{#1}{}{#2}");

  // Image/border — Perl L57-61
  DefMacro!("\\htmlimage{}", "");
  DefMacro!("\\htmlborder{}", "");
  DefEnvironment!("{makeimage}", "#body");
  DefEnvironment!("{tex2html_deferred}", "#body");
  DefMacro!("\\htmladdtonavigation{}", "");

  // rawhtml/htmlonly — Perl L66-88. These envs wrap raw HTML that should
  // bypass TeX tokenization entirely (angle brackets, ampersands, etc.
  // would otherwise trip the tokenizer). Perl's pattern is
  // `DefConstructorI(T_CS('\begin{foo}'), ..., afterDigest => ...)` with
  // a closure that calls `gullet->readRawLine` until `\end{foo}`.
  // Previously the Rust port used a plain DefEnvironment with empty body,
  // which would attempt to digest the raw-HTML body as TeX tokens and
  // fail on `<`/`>`/etc. Switch to the raw-line discard pattern.
  DefConstructor!(T_CS!("\\begin{rawhtml}"), None, "",
    reversion => "",
    after_digest => sub[_whatsit] {
      let endmark = "\\end{rawhtml}";
      let mut nlines = 0;
      gullet::read_raw_line(); // skip first line (after \begin{rawhtml})
      while let Some(line) = gullet::read_raw_line() {
        if line.trim_end() == endmark { break; }
        nlines += 1;
      }
      let _ = nlines;
    });
  DefMacro!("\\endrawhtml", "");
  DefConstructor!(T_CS!("\\begin{htmlonly}"), None, "",
    reversion => "",
    after_digest => sub[_whatsit] {
      let endmark = "\\end{htmlonly}";
      let mut nlines = 0;
      gullet::read_raw_line(); // skip first line (after \begin{htmlonly})
      while let Some(line) = gullet::read_raw_line() {
        if line.trim_end() == endmark { break; }
        nlines += 1;
      }
      let _ = nlines;
    });
  DefMacro!("\\endhtmlonly", "");

  // latexonly — Perl L92-98
  DefEnvironment!("{latexonly}", "#body");
  DefMacro!("\\latexonly@onearg{}", "#1");
  // Plain \latexonly — dispatch on next token. Perl uses ifNext T_BEGIN:
  //   if `{` → \latexonly@onearg{...} ; else → \begin{latexonly}...\end{latexonly}
  DefMacro!("\\latexonly", sub[_args] {
    let tok = gullet::read_token()?;
    if let Some(t) = tok {
      gullet::unread(Tokens!(t));
      if t.get_catcode() == Catcode::BEGIN {
        Ok(Tokens!(T_CS!("\\latexonly@onearg")))
      } else {
        Ok(Tokenize!(r"\begin{latexonly}"))
      }
    } else {
      Ok(Tokens!())
    }
  });

  // Misc — Perl L100-107
  DefMacro!("\\html{}", "");
  DefMacro!("\\latex{}",          "#1");
  DefMacro!("\\latexhtml{}{}",    "#1");
  DefMacro!("\\strikeout{}",      "#1");
  DefMacro!("\\htmlurl Semiverbatim", "\\url{#1}");
  DefMacro!("\\HTMLset{}{}",              "");
  DefMacro!("\\htmlinfo OptionalMatch:*", "");
});
