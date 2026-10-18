% A reduced svmult.cls copy whose class is a shipped intermediate class (mybase.cls), which loads article: the class slot
% sees the classes svmult's binding loaded (book) from inside mybase.cls too (72a review r7 NIT 8).
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{svmult}[1999/07/09 v1.0 reduced]
\LoadClass[twoside]{mybase}
\newcommand\svextra{SV}
