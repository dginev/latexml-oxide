//! Batch-25 engine contract: the alignment brace ledger follows tex.web's
//! protocol — braces count when SCANNED (get_next §342/§357), pushback of
//! previously-scanned tokens retracts (back_input §325), and expansion
//! output enters WITHOUT adjustment (begin_token_list; `read_balanced`
//! likewise no longer localizes the ledger — scan_toks §473-482 doesn't).
//! Perl instead retracts EVERY pushback and localizes readBalanced, which
//! drifts the ledger on expl3 brace-tricks (`\if_true: { \else: } \fi:`
//! halves traveling through different reads) and makes a later cell-top
//! `&` go stray — Perl LaTeXML shares this failure (verified 2026-09-01
//! on this exact repro). Root of the l3doc `{function}` stray-`&` family
//! (17+ bundles: every l3doc manual with a `{syntax}` block).

fn convert(tex: &str) -> String { super::convert(tex, false).0 }

#[test]
fn tl_greplace_then_protected_amp_in_cell() {
  // The l3doc shape shrunk to its core: an l3tl replace (whose delimited
  // scanning pushes net-unbalanced fragments) followed by a protected
  // macro that emits the cell separator. SURPASS-PERL: Perl errors
  // "Stray alignment &" here.
  let stderr = convert(
    "\\documentclass{article}\n\\ExplSyntaxOn\n\\tl_new:N \\g_my_tl\n\
       \\cs_new_protected:Npn \\my_amp: { & }\n\
       \\cs_new_protected:Npn \\my_row:\n  {\n    \\tl_gset:Nn \\g_my_tl { a~b }\n\
       \\tl_greplace_all:Nnn \\g_my_tl { ~ } { x }\n    name \\my_amp: e \\\\\n  }\n\
       \\ExplSyntaxOff\n\\begin{document}\n\\begin{tabular}{lr}\n\
       \\ExplSyntaxOn \\my_row: \\ExplSyntaxOff\n\\end{tabular}\n\\end{document}\n",
  );
  assert!(
    !stderr.contains("Stray alignment"),
    "expl3 brace-trick drift resurfaced (ledger protocol regression):\n{stderr}"
  );
  assert!(
    !stderr.contains("Error:"),
    "greplace+protected-& cell should convert clean:\n{stderr}"
  );
}

#[test]
fn ifstar_reemitted_brace_keeps_borders() {
  // The compensating half of the protocol: a closure-backed expandable
  // (`\@ifstar`-family) re-emits a token it READ; that token's brace
  // count must be retracted (tex.web §368 back_input around the one-step
  // expansion) or every `\foo{...}` behind an \@ifstar guard drifts the
  // ledger +1 and a later `&` misses the column-end check silently
  // (borders lost via handle_template never firing — cells.xml/graphrot
  // golden regressions caught this).
  let stderr = convert(
    "\\documentclass{article}\n\\newsavebox{\\foo}\n\
       \\def\\testrot#1{\\savebox{\\foo}{\\parbox{1in}{whales}}\\framebox{---\\usebox{\\foo}---}}\n\
       \\begin{document}\n\\begin{tabular}{|c|c|}\n\\hline\n\
       \\testrot{0} & \\testrot{1}\\\\\na & b \\\\\n\\hline\n\\end{tabular}\n\\end{document}\n",
  );
  assert!(
    !stderr.contains("Error:"),
    "savebox/framebox row should convert clean:\n{stderr}"
  );
}
