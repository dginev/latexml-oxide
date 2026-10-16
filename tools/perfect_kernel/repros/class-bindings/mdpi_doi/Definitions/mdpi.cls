% Minimal stand-in for Definitions/mdpi.cls (2208.12066's copy, 22 December 2022): the lines the mdpi binding reads,
% :74 the journal table it inputs and :523-530 the default DOI. pdflatex validated with the real copy.
\ProvidesClass{Definitions/mdpi}[22/12/2022 MDPI paper class]
\input{Definitions/journalnames}
\ifthenelse{\equal{\@journal}{molbank}
	\OR \equal{\@journal}{chemproc}
	}{
	\doinum{10.3390/\@articlenumber}
	}{
	\doinum{10.3390/\@doiabbr\@pubvolume\twodigits\@issuenum\fivedigits\@articlenumber}
}
