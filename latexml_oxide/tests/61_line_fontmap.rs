//! Picture-mode `line10` chars must have nonzero width, or LaTeX-2.09
//! `\@sline`-style drawing loops never terminate.
//!
//! Root cause of the canvas_3 OOM cluster (math0102053, math0102089,
//! math0212126, math0504436, math0506088, math0604321): plain-TeX papers
//! inline LaTeX 2.09 picture mode, whose `\@whiledim` loop advances by the
//! width of an `\hbox{\@linefnt\@getlinechar(x,y)}`. Real TeX gets nonzero
//! widths (2.5–10pt) from `line10.tfm`; without a fontmap, `FontDecode`
//! dropped the char → empty box → 0pt → unbounded box accumulation (~1.9M
//! boxes / 4.5GB RSS). Perl LaTeXML ships no `line` fontmap and OOMs the same
//! way; `line_fontmap.rs`/`lcircle_fontmap.rs` fix this at the root through
//! the architecture's own mechanism (a surpass-Perl reliability fix with no
//! control-flow divergence). Modern latex.ltx guards this exact hazard
//! (`\ifdim\wd\@linechar=\z@ \setbox\@linechar\hbox{.}\@badlinearg\fi`) but
//! these old documents inline the unguarded 2.09 macros.
//!
//! Dump-independent (plain-TeX input; the fontmap binding is compiled in).
use latexml::util::test::convert_fixture;

#[test]
fn line_font_chars_have_nonzero_width_and_loops_terminate() {
  let r = convert_fixture("tests/cluster_regressions/line_font_picture.tex");

  let out = r
    .result
    .unwrap_or_else(|| {
      panic!(
        "conversion produced no result (status_code={}) — the \\@whiledim line \
         loop likely ran away again",
        r.status_code
      )
    })
    .to_string();
  assert!(
    out.contains("LINEWIDTH-OK") && !out.contains("LINEWIDTH-ZERO"),
    "a line10 \\char box measured 0pt wide — the `line` fontmap regressed \
     (zero width re-opens the \\@sline infinite-loop OOM cluster)"
  );
  assert!(
    out.contains("LOOP-DONE"),
    "the \\whiledim drawing loop did not complete"
  );
  assert!(
    !r.log.contains("PushbackLimit")
      && !r.log.contains("runaway")
      && !r.log.contains("Infinite digestion loop"),
    "a runaway guard fired — the drawing loop is no longer terminating"
  );
}
