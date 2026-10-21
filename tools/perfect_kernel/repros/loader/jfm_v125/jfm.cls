% A minimal stand-in for jfm.cls v1.25 (Cambridge University Press's Journal of Fluid Mechanics class, the copy
% 2605.03843 ships — not in TeX Live; its notice allows use only for CUP publications), written from scratch for
% jfm_old_copy_has_no_lineno_option.tex: the options the class declares (its :40-58: `oneside`, `twoside`, `prodtf` —
% no `lineno` among them), on article. No code of the class is copied.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jfm}[2001/08/08 v1.25 Journal of Fluid Mechanics document class]
\newif\ifCUPmtlplainloaded
\DeclareOption{oneside}{}
\DeclareOption{twoside}{\@twosidetrue\@mparswitchtrue}
\DeclareOption{prodtf}{\CUPmtlplainloadedtrue}
\ProcessOptions
\LoadClass{article}
