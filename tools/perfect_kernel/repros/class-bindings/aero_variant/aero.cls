% A minimal stand-in for the edited aero.cls 2509.08607 and 2511.17720 ship (latexstudio.net's template for
% Astrodynamics, v1.0 2022/01/04 — not in TeX Live; it states no licence), written from scratch for the repro beside
% it: the same `\ADsetup` keys and title-page variables as the template (the binding, latexml_contrib/src/aero_cls.rs,
% leaves alone a variable still holding its placeholder), but a `\maketitle` (:128-136) that prints the kernel's
% `\@title` and `\@author`. No code of the class is copied; the placeholders are shortened.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{aero}[2022/01/04 repro stand-in]
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{article}}
\ProcessOptions\relax
\LoadClass[11pt]{article}
\RequirePackage{marvosym}
% :92 the corresponding author's mark, an envelope
\newcommand\cor{{\rm(\Letter)}}
\ExplSyntaxOn
% :97-118 each title-page field: its variable, its placeholder, its key
\cs_new_protected:Npn \__aero_field:nn #1#2
  {
    \tl_new:c { l_ADkeys_#1_tl }
    \tl_set:cn { l_ADkeys_#1_tl } {#2}
    \keys_define:nn { ADkeys } { #1 .tl_set:c = { l_ADkeys_#1_tl } }
  }
\__aero_field:nn { name } { }
\__aero_field:nn { url } { }
\__aero_field:nn { title } { Paper~title }
\__aero_field:nn { author } { First~A.~Author }
\__aero_field:nn { address } { Affiliation }
\__aero_field:nn { email } { Corresponding~author's~e-mail }
\__aero_field:nn { abstract } { Abstract~content }
\__aero_field:nn { keywords } { keywords~content }
\NewDocumentCommand \ADsetup { +m } { \keys_set:nn { ADkeys } {#1} }
\RenewDocumentCommand \maketitle { }
  { \par\noindent { \LARGE \bfseries \@title } \par\medskip \noindent \@author \par\medskip }
\ExplSyntaxOff
\endinput
