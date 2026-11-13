% A reduced model of sig-alternate-05-2015.cls:1639-1641 (1610.03579, 1710.01977): `\doi` stores, `\printdoi` is `\url`.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{raw_class_doi_store_is_a_string}
\LoadClass{article}
\RequirePackage{url}
\def\doi#1{\def\@doi{#1}}
\def\printdoi#1{\url{#1}}
\def\@doi{}
