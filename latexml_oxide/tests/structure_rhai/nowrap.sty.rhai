// Regression fixture for issue #508: a DefEnvironment whose constructor wraps
// #body in an <ltx:block> must keep the WHOLE body inside the block (matching
// Perl LaTeXML), not over-close the block on a paragraph break. The .tex
// exercises the band: a blank-line \par (was the bug — it split the block), a
// bare newline (single <p>), and \\ (a <break/> inside one <p>).
DefEnvironment("{nowrap}", "<ltx:block class='nowrap'>#body</ltx:block>");
