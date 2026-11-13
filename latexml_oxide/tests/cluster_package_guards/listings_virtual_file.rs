//! Batch 56cv: `\lstinputlisting` reads a file that exists only in the
//! session's virtual file store — tcolorbox's `tcblisting` writes its body
//! to `\jobname.listing` with `\tcbverbatimwrite` and reads it back
//! (tcblistingscore.code.tex:275-282, tcblistings.code.tex:42-53). The
//! reader went to disk only, so every `listing only` box rendered an empty
//! listing at 0 errors (sim-os-menus' terminal windows; ~33 manuals
//! round-trip a listing file). Perl reports "Can't read listings file".

/// The whole `<listing>` — its base64 `data` is the body, and the lines
/// are typeset.
#[test]
fn tcblisting_listing_only_reads_the_virtual_listing_file() {
  if !latexml::util::test::kpse_has("tcolorbox.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{tcolorbox}\\tcbuselibrary{listings,skins}\n\\begin{document}\n\\begin{tcblisting}{listing only,listing engine=listings}\ntest@DESKTOP:~$ ping -c 2 ctan.org\nPING ctan.org 56 bytes of data.\n\\end{tcblisting}\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "listing",
    &[],
    r##"<listing class="ltx_lst_language_TeX_LaTeX ltx_lstlisting" data="dGVzdEBERVNLVE9QOn4kIHBpbmcgLWMgMiBjdGFuLm9yZwpQSU5HIGN0YW4ub3JnIDU2IGJ5dGVzIG9mIGRhdGEu" dataencoding="base64" datamimetype="text/plain" dataname="t.listing"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">test@DESKTOP</text><text color="#000000" font="typewriter" fontsize="90%">:~$</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">ping</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text color="#000000" font="typewriter" fontsize="90%">-</text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">c</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text color="#000000" font="typewriter" fontsize="90%">2</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">ctan</text><text color="#000000" font="typewriter" fontsize="90%">.</text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">org</text></listingline><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">PING</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">ctan</text><text color="#000000" font="typewriter" fontsize="90%">.</text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">org</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text color="#000000" font="typewriter" fontsize="90%">56</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">bytes</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">of</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">data</text><text color="#000000" font="typewriter" fontsize="90%">.</text></listingline></listing>"##,
  );
}
