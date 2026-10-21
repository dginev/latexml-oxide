% A minimal stand-in for aero.cls (latexstudio.net's template for the journal Astrodynamics, v1.0 2022/01/04, which
% 2610.12827 ships — not in TeX Live; it states no licence), written from scratch for the repros beside it: it defines
% only the interface they and the binding (latexml_contrib/src/aero_cls.rs) use — `\ADsetup`'s keys and the title-page
% variables `\l_ADkeys_<key>_tl` they set (the binding reads them and leaves alone one still holding its placeholder),
% the corresponding mark `\cor` and a `\maketitle` printing them — with what the class does there; line numbers are
% those of 2610.12827's copy. No code of the class is copied; the placeholders are shortened.
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
% :128-148 the title page: the title, the authors, the addresses (`\sep` a new line), the e-mail after an envelope, the
% abstract and the keywords (`\sep` a centred dot)
\RenewDocumentCommand \maketitle { }
  {
    \par\noindent { \LARGE \bfseries \l_ADkeys_title_tl } \par\medskip
    \noindent \l_ADkeys_author_tl \par\medskip
    { \itshape \cs_set:Npn \sep { \par } \noindent \l_ADkeys_address_tl \par } \medskip
    \noindent \Letter \  \l_ADkeys_email_tl \par\medskip
    \noindent \textbf { Abstract } \par \noindent \l_ADkeys_abstract_tl \par\medskip
    \noindent \textbf { Keywords } \par
    { \cs_set:Npn \sep { ~\textperiodcentered\ } \noindent \l_ADkeys_keywords_tl } \par
  }
\ExplSyntaxOff
\endinput
