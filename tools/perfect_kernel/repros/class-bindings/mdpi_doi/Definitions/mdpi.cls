% A minimal stand-in for Definitions/mdpi.cls (MDPI's class, 2208.12066's copy of 22 December 2022 — not in TeX Live; no
% licence stated for the class), written from scratch for the repro beside it: the lines the mdpi binding reads raw
% after itself (OXIDIZED_DESIGN_DIVERGENCES #731) — the ifthen it loads (:37), the journal table it inputs (:74) and the
% default DOI (:523-530: the article number alone for molbank and chemproc, else the journal abbreviation, volume, two-
% digit issue and five-digit article number) —, with what the class does there. No code of the class is copied.
\ProvidesClass{Definitions/mdpi}[22/12/2022 MDPI paper class]
\RequirePackage{ifthen}
\input{Definitions/journalnames}
\ifthenelse{\equal{\@journal}{molbank}\OR\equal{\@journal}{chemproc}}%
  {\doinum{10.3390/\@articlenumber}}%
  {\doinum{10.3390/\@doiabbr\@pubvolume\twodigits\@issuenum\fivedigits\@articlenumber}}
