%% A class wrapper declaring itself smfart that defers its load of smfart to its own end.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{smfart}
\AtEndOfClass{\LoadClass{smfart}\renewcommand{\smaller}[1][1]{SMALLER}}
\endinput
