%% A renamed REVTeX 4 copy (qm2008_abs.cls, snp.cls) cut down to ltxgrid's direct write to the document hook
%% (qm2008_abs.cls:3233 `\prepdef\@begindocumenthook{\open@column@one\@ne\set@colht…}`)
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{revtex4}[2001/08/02 v4.0 (http://publish.aps.org/revtex4/ for documentation)]
\LoadClass{article}
\providecommand\qmhooked{UNHOOKED}
\g@addto@macro\@begindocumenthook{\gdef\qmhooked{HOOKED}}
\endinput
