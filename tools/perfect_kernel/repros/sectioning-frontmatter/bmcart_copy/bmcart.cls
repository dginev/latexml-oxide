% A minimal stand-in, written from scratch, for BioMed Central's bmcart.cls (VTeX; 1711.01177's copy), which is not in
% TeX Live. It keeps the frontmatter machinery the repro reads: `{frontmatter}` lets `\title`, `\author`, `\address`
% and `{fmbox}` to the class's printers (:914-929, :966-984); `\title@fmt` typesets the title (:1032-1053),
% `\author@fmt` the name (:1196-1255; its `addressref` key, :1176-1180, the address numbers the real class prints
% from its labels), `\address@fmt` stores
% the address by its `id` (:1278-1296), printed under "Author details" by `\printaddresses`, which `\thebibliography`
% calls (:1357-1365, :1708-1712); `{artnotes}`'s `\note[id=…]{…}` stores an article note by its `id` (:1371-1395),
% which `\printartnotes` prints at the end of the `{fmbox}` (:999-1002, :1406-1409, :1440-1447), the authors naming it
% in their `noteref` key (:1139-1143).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{bmcart}[2026/10/21 BioMed Central article (stand-in)]
\LoadClass{article}
\RequirePackage{keyval}
\newcounter{author}
\newcounter{address}
\newbox\fm@box
\def\frontmatter{\let\title\bmc@title\let\author\bmc@author\let\address\bmc@address
  \let\fmbox\bmc@fmbox\let\endfmbox\bmc@endfmbox\let\artnotes\bmc@artnotes\let\endartnotes\endbmc@artnotes
  \parindent\z@}
\def\endfrontmatter{}
\def\bmc@fmbox{\setbox\fm@box=\vbox\bgroup}
\def\bmc@endfmbox{\par\printartnotes\egroup\noindent\box\fm@box\par}
\def\bmc@title{\@ifnextchar[{\title@fmt}{\title@fmt[]}}
\def\title@fmt[#1]#2{{\LARGE\sffamily #2\par}}
\define@key{author}{addressref}{\def\author@refs{#1}}
\define@key{author}{email}{}
\define@key{author}{noteref}{}
\def\bmc@author{\@ifnextchar[{\author@fmt}{\author@fmt[]}}
\def\author@fmt[#1]#2{\stepcounter{author}\let\author@refs\@empty\setkeys{author}{#1}%
  {\def\inits##1{}\def\fnm##1{##1}\def\snm##1{##1}\ifnum\c@author>1 , \fi #2}\ignorespaces}
\define@key{bmcartnote}{id}{\def\bmcartnote@id{#1}%
  \@ifundefined{bmcartnote@id@list}{\xdef\bmcartnote@id@list{#1}}{\xdef\bmcartnote@id@list{\bmcartnote@id@list,#1}}}
\newenvironment{bmc@artnotes}{\def\note{\@ifnextchar[{\@bmcartnote}{\@bmcartnote[]}}}{}
\def\@bmcartnote[#1]#2{{\setkeys{bmcartnote}{#1}%
  \expandafter\protected@xdef\csname bmcartnote@\bmcartnote@id\endcsname{#2}}}
\def\printartnotes{\@ifundefined{bmcartnote@id@list}{}%
  {\@for\bmcnote@id:=\bmcartnote@id@list\do{\textsuperscript{\bmcnote@id}\@nameuse{bmcartnote@\bmcnote@id}\par}}}
\define@key{address}{id}{\def\address@id{#1}}
\def\bmc@address{\@ifnextchar[{\address@fmt}{\address@fmt[]}}
\def\address@fmt[#1]#2{\stepcounter{address}\edef\address@id{aff\the\c@address}\setkeys{address}{#1}%
  \expandafter\gdef\csname address@text@\the\c@address\endcsname{#2}}
\def\printaddresses{\par\textbf{Author details}\par
  {\def\orgname##1{##1}\def\city##1{##1}\def\cny##1{##1}%
   \@tempcnta\z@\loop\ifnum\@tempcnta<\c@address\advance\@tempcnta\@ne
   \the\@tempcnta\ \csname address@text@\the\@tempcnta\endcsname.\par\repeat}%
  \global\let\printaddresses\relax}
\newenvironment{backmatter}{}{}
\let\saved@thebibliography\thebibliography
\def\thebibliography{\printaddresses\saved@thebibliography}
