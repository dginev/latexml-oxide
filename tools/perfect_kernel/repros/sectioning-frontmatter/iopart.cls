%% 
%% This is file `iopart.cls'
%% 
%% This file is distributed in the hope that it will be useful, 
%% but WITHOUT ANY WARRANTY; without even the implied warranty of 
%% MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
%% 
%% Licensed under the LPPL: http://www.latex-project.org/lppl.txt
%% Current Maintainer: IOP Publishing Ltd
%% 
%% MODIFIED COPY (latexml-oxide, tools/perfect_kernel/repros/sectioning-frontmatter/): a shortened copy of
%% iopart.cls as arXiv 2311.12551 ships it, for the frontmatter repros to compile with pdflatex from the tree.
%% Kept as the class has them: the options, the page and title commands, \author, \address, \ead, \mailto, the
%% abstract, the lists and the indented environment. Removed: the journal abbreviation macros, sectioning, floats,
%% tables of contents, bibliography and equation layout. Its size file iopart12.clo, which states no licence, is not
%% copied: the block marked "stand-in" below is written from scratch for this copy (it stands for iopart12.clo:24-61,
%% :73-74, :95-100, no line of it copied).
%% \CharacterTable
%%  {Upper-case    \A\B\C\D\E\F\G\H\I\J\K\L\M\N\O\P\Q\R\S\T\U\V\W\X\Y\Z
%%   Lower-case    \a\b\c\d\e\f\g\h\i\j\k\l\m\n\o\p\q\r\s\t\u\v\w\x\y\z
%%   Digits        \0\1\2\3\4\5\6\7\8\9
%%   Exclamation   \!     Double quote  \"     Hash (number) \#
%%   Dollar        \$     Percent       \%     Ampersand     \&
%%   Acute accent  \'     Left paren    \(     Right paren   \)
%%   Asterisk      \*     Plus          \+     Comma         \,
%%   Minus         \-     Point         \.     Solidus       \/
%%   Colon         \:     Semicolon     \;     Less than     \<
%%   Equals        \=     Greater than  \>     Question mark \?
%%   Commercial at \@     Left bracket  \[     Backslash     \\
%%   Right bracket \]     Circumflex    \^     Underscore    \_
%%   Grave accent  \`     Left brace    \{     Vertical bar  \|
%%   Right brace   \}     Tilde         \~}
%
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{iopart}[1996/06/10 v0.0 IOP Journals LaTeX article class (shortened copy, latexml-oxide)]
\newcommand\@ptsize{0}
\newif\if@restonecol
\newif\if@titlepage
\newif\ifiopams
\@titlepagefalse
\DeclareOption{a4paper}
   {\setlength\paperheight {297mm}%
    \setlength\paperwidth  {210mm}}
\DeclareOption{letterpaper}
   {\setlength\paperheight {11in}%
    \setlength\paperwidth  {8.5in}}
\DeclareOption{landscape}
   {\setlength\@tempdima   {\paperheight}%
    \setlength\paperheight {\paperwidth}%
    \setlength\paperwidth  {\@tempdima}}
\DeclareOption{10pt}{\renewcommand\@ptsize{0}}
\DeclareOption{11pt}{\renewcommand\@ptsize{2}} % No 11pt version
\DeclareOption{12pt}{\renewcommand\@ptsize{2}}
\DeclareOption{draft}{\setlength\overfullrule{5pt}}
\DeclareOption{final}{\setlength\overfullrule{0pt}}
\DeclareOption{titlepage}{\@titlepagetrue}
\DeclareOption{notitlepage}{\@titlepagefalse}
\ExecuteOptions{letterpaper,final}
\ProcessOptions
\DeclareMathAlphabet{\bi}{OML}{cmm}{b}{it}
\DeclareMathAlphabet{\bcal}{OMS}{cmsy}{b}{n}
% stand-in for \input{iopart1\@ptsize.clo}, written from scratch for this copy (latexml-oxide; no line of
% iopart12.clo is copied): a 12pt text and the sizes the title commands select (standing for iopart12.clo:24-61),
% the text block (:73-74) and the first list level (:95-100)
\renewcommand\normalsize{\fontsize{12}{16}\selectfont}
\newcommand\small{\fontsize{11}{14}\selectfont}
\newcommand\footnotesize{\fontsize{10}{12}\selectfont}
\newcommand\scriptsize{\fontsize{8}{10}\selectfont}
\newcommand\tiny{\fontsize{6}{7}\selectfont}
\newcommand\large{\fontsize{14}{18}\selectfont}
\newcommand\Large{\fontsize{17}{22}\selectfont}
\newcommand\LARGE{\fontsize{20}{25}\selectfont}
\newcommand\huge{\fontsize{25}{30}\selectfont}
\newcommand\Huge{\huge}
\textwidth=37pc \textheight=56pc
\def\@listi{\topsep=6pt \itemsep=3pt \parsep=0pt \leftmargin=\leftmargini}
\normalsize
% end of the stand-in
\setlength\lineskip{1\p@}
\setlength\normallineskip{1\p@}
\renewcommand\baselinestretch{}
\setlength\parskip{0\p@ \@plus \p@}
\@lowpenalty   51
\@medpenalty  151
\@highpenalty 301
\setlength\parindent{2em}
\setcounter{topnumber}{8}
\renewcommand\topfraction{1}
\setcounter{bottomnumber}{3}
\renewcommand\bottomfraction{.99}
\setcounter{totalnumber}{8}
\renewcommand\textfraction{0.01}
\renewcommand\floatpagefraction{.8}
\setcounter{dbltopnumber}{6}
\renewcommand\dbltopfraction{1}
\renewcommand\dblfloatpagefraction{.8}
%
\pretolerance=5000
\tolerance=8000
%
% Headings for all pages apart from first
%
\def\ps@headings{\let\@oddfoot\@empty
      \let\@evenfoot\@empty
      \def\@evenhead{\thepage\hfil\itshape\rightmark}%
      \def\@oddhead{{\itshape\leftmark}\hfil\thepage}%
      \let\@mkboth\markboth
      \let\sectionmark\@gobble
      \let\subsectionmark\@gobble}
%
% Headings for first page
%
\def\ps@myheadings{\let\@oddfoot\@empty\let\@evenfoot\@empty
    \let\@oddhead\@empty\let\@evenhead\@empty
    \let\@mkboth\@gobbletwo
    \let\sectionmark\@gobble
    \let\subsectionmark\@gobble}
%
% \maketitle just ends page
%
\newcommand\maketitle{\newpage}
%
% Article titles
%
% Usage: \title[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\renewcommand{\title}{\@ifnextchar[{\@stitle}{\@ftitle}}
\def\@stitle[#1]#2{\markboth{#1}{#1}%
    \thispagestyle{myheadings}%
    \vspace*{3pc}{\exhyphenpenalty=10000\hyphenpenalty=10000 
    \Large\raggedright\noindent
    \bf#2\par}}
\def\@ftitle#1{\markboth{#1}{#1}%
    \thispagestyle{myheadings}%
    \vspace*{3pc}{\exhyphenpenalty=10000\hyphenpenalty=10000 
    \Large\raggedright\noindent
    \bf#1\par}}
%
% Can use \paper instead of \title
%
\let\paper=\title
%
% Generic title command for articles other than papers
%
% Usage: \article[Short title]{Article Type}{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\newcommand{\article}{\@ifnextchar[{\@sarticle}{\@farticle}}
\def\@sarticle[#1]#2#3{\markboth{#1}{#1}%
    \thispagestyle{myheadings}%
     \vspace*{.5pc}%
    {\parindent=\mathindent \bf #2\par}%
     \vspace*{1.5pc}%
    {\exhyphenpenalty=10000\hyphenpenalty=10000
     \Large\raggedright\noindent
     \bf#3\par}}%
\def\@farticle#1#2{\markboth{#2}{#2}%
    \thispagestyle{myheadings}%
     \vspace*{.5pc}%
    {\parindent=\mathindent \bf #1\par}%
     \vspace*{1.5pc}%
    {\exhyphenpenalty=10000\hyphenpenalty=10000
     \Large\raggedright\noindent
     \bf#2\par}}%
%
% Letters to the Editor
%
% Usage \letter{Full title}
% No short title is required for Letters
%
\def\letter#1{\article[Letter to the Editor]{Letter to the Editor}{#1}}
%
% Fast Track Communications (added by sxb 9 March 2011)
%
% Usage \ftc{Full title} - there's no short title
\def\ftc#1{\article[Fast Track Communication]{Fast Track Communication}{#1}}
%
%
% Review articles
%
% Usage: \review[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\def\review{\@ifnextchar[{\@sreview}{\@freview}}
\def\@sreview[#1]#2{\@sarticle[#1]{Review Article}{#2}}
\def\@freview#1{\@farticle{Review Article}{#1}}
%
% Topical Review
%
% Usage: \topical[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\def\topical{\@ifnextchar[{\@stopical}{\@ftopical}}
\def\@stopical[#1]#2{\@sarticle[#1]{Topical Review}{#2}}
\def\@ftopical#1{\@farticle{Topical Review}{#1}}
%
% Comments
%
% Usage: \comment[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\def\comment{\@ifnextchar[{\@scomment}{\@fcomment}}
\def\@scomment[#1]#2{\@sarticle[#1]{Comment}{#2}}
\def\@fcomment#1{\@farticle{Comment}{#1}}
%
% Rapid Communications
%
% Usage: \rapid[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\def\rapid{\@ifnextchar[{\@srapid}{\@frapid}}
\def\@srapid[#1]#2{\@sarticle[#1]{Rapid Communication}{#2}}
\def\@frapid#1{\@farticle{Rapid Communication}{#1}}
%
% Notes
%
% Usage: \note[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\def\note{\@ifnextchar[{\@snote}{\@fnote}}
\def\@snote[#1]#2{\@sarticle[#1]{Note}{#2}}
\def\@fnote#1{\@farticle{Note}{#1}}
%
% Preliminary Communications
%
% Usage: \prelim[Short title]{Full title}
% [Short title] is optional; use where title is too long
% or contains footnotes, 50 characters maximum 
%
\def\prelim{\@ifnextchar[{\@sprelim}{\@fprelim}}
\def\@sprelim[#1]#2{\@sarticle[#1]{Preliminary Communication}{#2}}
\def\@fprelim#1{\@farticle{Preliminary Communication}{#1}}
%
% List of authors
%
% Usage \author[Short form]{List of all authors}
% The short form excludes footnote symbols linking authors to addresses 
% and is used for running heads in printed version (but not on preprints)
%
\renewcommand{\author}{\@ifnextchar[{\@sauthor}{\@fauthor}}
\def\@sauthor[#1]#2{\markright{#1}    % for production only
   \vspace*{1.5pc}%
   \begin{indented}%
   \item[]\normalsize\bf\raggedright#2
   \end{indented}%
   \smallskip}
\def\@fauthor#1{%\markright{#1}         for production only
   \vspace*{1.5pc}%
   \begin{indented}%
   \item[]\normalsize\bf\raggedright#1
   \end{indented}%
   \smallskip}
%
% Affiliation (authors address)
%
% Usage: \address{Address of first author}
%        \address{Address of second author}
% Use once for each address, use symbols \dag \ddag \S \P $\|$
% to connect authors with addresses
%
\newcommand{\address}[1]{\begin{indented}
   \item[]\rm\raggedright #1
   \end{indented}}
%
% American Mathematical Society Classification Numbers
% Usage: \ams{57.XX, 58.XX}
%
\def\ams#1{\vspace{10pt}
     \begin{indented}
     \item[]\rm AMS classification scheme numbers: #1\par
     \end{indented}}
%
% A single Physics & Astronomy Classification Number
% Usage \pacno{31.10}
%
\def\pacno#1{\vspace{10pt}
     \begin{indented}
     \item[]\rm PACS number: #1\par
     \end{indented}}
%
% Physics & Astronomy Classification Numbers (more than one)
% Usage \pacs{31.10, 31.20T}
%
\def\pacs#1{\vspace{10pt}
     \begin{indented}
     \item[]\rm PACS numbers: #1\par
     \end{indented}}
%
% Submission details. If \jl command used journals name printed
% otherwise Institute of Physics Publishing 
%
\def\submitted{\vspace{28pt plus 10pt minus 18pt}
     \noindent{\small\rm Submitted to: {\it \journal}\par}}
%
\def\submitto#1{\vspace{28pt plus 10pt minus 18pt}
     \noindent{\small\rm Submitted to: {\it #1}\par}}
%
% For articles (other than Letters) not divided into sections
% Usage \nosections Start of text
%
\def\nosections{\vspace{30\p@ plus12\p@ minus12\p@}
    \noindent\ignorespaces}
%
% Acknowledgments (no heading if letter)
% Usage \ack for Acknowledgments, \ackn for Acknowledgement
%
\def\ack{\ifletter\bigskip\noindent\ignorespaces\else
    \section*{Acknowledgments}\fi}
\def\ackn{\ifletter\bigskip\noindent\ignorespaces\else
    \section*{Acknowledgment}\fi}
%
% Footnotes: symbols selected in order \dag (1), \ddag (2), \S (3), 
% $\|$ (4), $\P$ (5), $^+$ (6), $^*$ (7), \sharp (8), \dagger\dagger (9)
% unless optional argument of [<num>] use to specify required symbol, 
% 1=\dag, 2=\ddag, etc
% Usage: \footnote{Text of footnote}
%        \footnote[3]{Text of footnote}
%
\def\footnoterule{}%
\setcounter{footnote}{1}
\long\def\@makefntext#1{\parindent 1em\noindent 
 \makebox[1em][l]{\footnotesize\rm$\m@th{\fnsymbol{footnote}}$}%
 \footnotesize\rm #1}
\def\@makefnmark{\hbox{${\fnsymbol{footnote}}\m@th$}}
\def\@thefnmark{\fnsymbol{footnote}}
\def\footnote{\@ifnextchar[{\@xfootnote}{\stepcounter{\@mpfn}%
       \begingroup\let\protect\noexpand
       \xdef\@thefnmark{\thempfn}\endgroup
     \@footnotemark\@footnotetext}}
\def\@xfootnote[#1]{\setcounter{footnote}{#1}%
   \addtocounter{footnote}{-1}\footnote}
\def\@fnsymbol#1{\ifcase#1\or \dagger\or \ddagger\or \S\or
   \|\or \P\or ^{+}\or ^{\tsty *}\or \sharp
   \or \dagger\dagger \else\@ctrerr\fi\relax}
%
% IOP Journals
%
\newcounter{jnl}
\newcommand{\jl}[1]{\setcounter{jnl}{#1}}
\def\journal{\ifnum\thejnl=0 Institute of Physics Publishing\fi
        \ifnum\thejnl=1 J. Phys.\ A: Math.\ Gen.\ \fi
        \ifnum\thejnl=2 J. Phys.\ B: At.\ Mol.\ Opt.\ Phys.\ \fi
        \ifnum\thejnl=3 J. Phys.:\ Condens. Matter\ \fi
        \ifnum\thejnl=4 J. Phys.\ G: Nucl.\ Part.\ Phys.\ \fi
        \ifnum\thejnl=5 Inverse Problems\ \fi
        \ifnum\thejnl=6 Class. Quantum Grav.\ \fi
        \ifnum\thejnl=7 Network: Comput.\ Neural Syst.\ \fi
        \ifnum\thejnl=8 Nonlinearity\ \fi
        \ifnum\thejnl=9 J. Opt. B: Quantum Semiclass. Opt.\ \fi
        \ifnum\thejnl=10 Waves Random Media\ \fi
        \ifnum\thejnl=11 J. Opt. A: Pure Appl. Opt.\ \fi
        \ifnum\thejnl=12 Phys. Med. Biol.\ \fi
        \ifnum\thejnl=13 Modelling Simul.\ Mater.\ Sci.\ Eng.\ \fi
        \ifnum\thejnl=14 Plasma Phys. Control. Fusion\ \fi
        \ifnum\thejnl=15 Physiol. Meas.\ \fi
        \ifnum\thejnl=16 Combust. Theory Modelling\ \fi
        \ifnum\thejnl=17 High Perform.\ Polym.\ \fi
        \ifnum\thejnl=18 Public Understand. Sci.\ \fi
        \ifnum\thejnl=19 Rep.\ Prog.\ Phys.\ \fi
        \ifnum\thejnl=20 J.\ Phys.\ D: Appl.\ Phys.\ \fi
        \ifnum\thejnl=21 Supercond.\ Sci.\ Technol.\ \fi
        \ifnum\thejnl=22 Semicond.\ Sci.\ Technol.\ \fi
        \ifnum\thejnl=23 Nanotechnology\ \fi
        \ifnum\thejnl=24 Measur.\ Sci.\ Technol.\ \fi
        \ifnum\thejnl=25 Plasma.\ Sources\ Sci.\ Technol.\ \fi
        \ifnum\thejnl=26 Smart\ Mater.\ Struct.\ \fi
        \ifnum\thejnl=27 J.\ Micromech.\ Microeng.\ \fi
        \ifnum\thejnl=28 Distrib.\ Syst.\ Engng\ \fi
        \ifnum\thejnl=29 Bioimaging\ \fi
        \ifnum\thejnl=30 J.\ Radiol. Prot.\ \fi
        \ifnum\thejnl=31 Europ. J. Phys.\ \fi
        \ifnum\thejnl=32 J. Opt. A: Pure Appl. Opt.\ \fi
        \ifnum\thejnl=33 New. J. Phys.\ \fi}
%
% E-mail addresses (to provide links from headers)
%
\def\eads#1{\vspace*{5pt}\address{E-mail: #1}}
\def\ead#1{\vspace*{5pt}\address{E-mail: \mailto{#1}}}
\def\mailto#1{{\tt #1}}
%
% Switches
%
\newif\ifletter
%
\setcounter{secnumdepth}{3}
\newcounter {section}
\newcounter {subsection}[section]
\newcounter {subsubsection}[subsection]
\newcounter {paragraph}[subsubsection]
\newcounter {subparagraph}[paragraph]
\renewcommand\thesection       {\arabic{section}}
\renewcommand\thesubsection    {\thesection.\arabic{subsection}}
\renewcommand\thesubsubsection {\thesubsection.\arabic{subsubsection}}
\renewcommand\theparagraph     {\thesubsubsection.\arabic{paragraph}}
\renewcommand\thesubparagraph  {\theparagraph.\arabic{subparagraph}}
\def\@chapapp{Section}

\newcommand\section{\@startsection {section}{1}{\z@}%
                   {-3.5ex \@plus -1ex \@minus -.2ex}%
                   {2.3ex \@plus.2ex}%
                   {\reset@font\normalsize\bfseries\raggedright}}
\newcommand\subsection{\@startsection{subsection}{2}{\z@}%
                   {-3.25ex\@plus -1ex \@minus -.2ex}%
                   {1.5ex \@plus .2ex}%
                   {\reset@font\normalsize\itshape\raggedright}}
\newcommand\subsubsection{\@startsection{subsubsection}{3}{\z@}%
                                     {-3.25ex\@plus -1ex \@minus -.2ex}%
                                     {-1em \@plus .2em}%
                                     {\reset@font\normalsize\itshape}}
\newcommand\paragraph{\@startsection{paragraph}{4}{\z@}%
                                    {3.25ex \@plus1ex \@minus.2ex}%
                                    {-1em}%
                                    {\reset@font\normalsize\itshape}}
\newcommand\subparagraph{\@startsection{subparagraph}{5}{\parindent}%
                                       {3.25ex \@plus1ex \@minus .2ex}%
                                       {-1em}%
                                      {\reset@font\normalsize\itshape}}
\def\@sect#1#2#3#4#5#6[#7]#8{\ifnum #2>\c@secnumdepth
     \let\@svsec\@empty\else
     \refstepcounter{#1}\edef\@svsec{\csname the#1\endcsname. }\fi
     \@tempskipa #5\relax
      \ifdim \@tempskipa>\z@
        \begingroup #6\relax
          \noindent{\hskip #3\relax\@svsec}{\interlinepenalty \@M #8\par}%
        \endgroup
       \csname #1mark\endcsname{#7}\addcontentsline
         {toc}{#1}{\ifnum #2>\c@secnumdepth \else
                      \protect\numberline{\csname the#1\endcsname}\fi
                    #7}\else
        \def\@svsechd{#6\hskip #3\relax  %% \relax added 2 May 90
                   \@svsec #8\csname #1mark\endcsname
                      {#7}\addcontentsline
                           {toc}{#1}{\ifnum #2>\c@secnumdepth \else
                             \protect\numberline{\csname the#1\endcsname}\fi
                       #7}}\fi
     \@xsect{#5}}
%
\def\@ssect#1#2#3#4#5{\@tempskipa #3\relax
   \ifdim \@tempskipa>\z@
     \begingroup #4\noindent{\hskip #1}{\interlinepenalty \@M #5\par}\endgroup
   \else \def\@svsechd{#4\hskip #1\relax #5}\fi
    \@xsect{#3}}

\setlength\leftmargini{2em}
\setlength\leftmarginii{2em}
\setlength\leftmarginiii{1.8em}
\setlength\leftmarginiv{1.6em}
\setlength\leftmarginv{1em}
\setlength\leftmarginvi{1em}
\setlength\leftmargin{\leftmargini}
\setlength\labelsep{0.5em}
\setlength\labelwidth{\leftmargini}
\addtolength\labelwidth{-\labelsep}
\@beginparpenalty -\@lowpenalty
\@endparpenalty   -\@lowpenalty
\@itempenalty     -\@lowpenalty
\renewcommand\theenumi{\roman{enumi}}
\renewcommand\theenumii{\alph{enumii}}
\renewcommand\theenumiii{\arabic{enumiii}}
\renewcommand\theenumiv{\Alph{enumiv}}
\newcommand\labelenumi{(\theenumi)}
\newcommand\labelenumii{(\theenumii)}
\newcommand\labelenumiii{\theenumiii.}
\newcommand\labelenumiv{(\theenumiv)}
\renewcommand\p@enumii{(\theenumi)}
\renewcommand\p@enumiii{(\theenumi.\theenumii)}
\renewcommand\p@enumiv{(\theenumi.\theenumii.\theenumiii)}
\newcommand\labelitemi{$\m@th\bullet$}
\newcommand\labelitemii{\normalfont\bfseries --}
\newcommand\labelitemiii{$\m@th\ast$}
\newcommand\labelitemiv{$\m@th\cdot$}
\newenvironment{description}
               {\list{}{\labelwidth\z@ \itemindent-\leftmargin
                        \let\makelabel\descriptionlabel}}
               {\endlist}
\newcommand\descriptionlabel[1]{\hspace\labelsep
                                \normalfont\bfseries #1}
\newenvironment{abstract}{%
      \vspace{16pt plus3pt minus3pt}
      \begin{indented}
      \item[]{\bfseries \abstractname.}\quad\rm\ignorespaces} 
      {\end{indented}\if@titlepage\newpage\else\vspace{18\p@ plus18\p@}\fi}
\newenvironment{verse}
               {\let\\=\@centercr
                \list{}{\itemsep      \z@
                        \itemindent   -1.5em%
                        \listparindent\itemindent
                        \rightmargin  \leftmargin
                        \advance\leftmargin 1.5em}%
                \item[]}
               {\endlist}
\newenvironment{quotation}
               {\list{}{\listparindent 1.5em%
                        \itemindent    \listparindent
                        \rightmargin   \leftmargin
                        \parsep        \z@ \@plus\p@}%
                \item[]}
               {\endlist}
\newenvironment{quote}
               {\list{}{\rightmargin\leftmargin}%
                \item[]}
               {\endlist}
\DeclareOldFontCommand{\rm}{\normalfont\rmfamily}{\mathrm}
\DeclareOldFontCommand{\sf}{\normalfont\sffamily}{\mathsf}
\DeclareOldFontCommand{\tt}{\normalfont\ttfamily}{\mathtt}
\DeclareOldFontCommand{\bf}{\normalfont\bfseries}{\mathbf}
\DeclareOldFontCommand{\it}{\normalfont\itshape}{\mathit}
\DeclareOldFontCommand{\sl}{\normalfont\slshape}{\@nomath\sl}
\DeclareOldFontCommand{\sc}{\normalfont\scshape}{\@nomath\sc}
\newcommand\refname{References}
\newcommand\indexname{Index}
\newcommand\figurename{Figure}
\newcommand\tablename{Table}
\newcommand\partname{Part}
\newcommand\appendixname{Appendix}
\newcommand\abstractname{Abstract}
\newcommand\today{\number\day\space\ifcase\month\or
  January\or February\or March\or April\or May\or June\or
  July\or August\or September\or October\or November\or December\fi
  \space\number\year}
\setlength\columnsep{10\p@}
\setlength\columnseprule{0\p@}
\newlength{\indentedwidth}
\newdimen\mathindent
\indentedwidth=\mathindent
\def\indented{\list{}{\itemsep=0\p@\labelsep=0\p@\itemindent=0\p@
   \labelwidth=0\p@\leftmargin=\mathindent\topsep=0\p@\partopsep=0\p@
   \parsep=0\p@\listparindent=15\p@}\footnotesize\rm}

\let\endindented=\endlist 
\pagenumbering{arabic}      % Arabic page numbers
\raggedbottom
\onecolumn
\endinput
%% 
%% End of file `iopart.cls'.

