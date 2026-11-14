%% A renamed copy of the Journal of Fluid Mechanics class (JFM-FLM_Au.cls, `\ProvidesClass{jfm}`), cut down to its
%% per-section equation numbers (JFM-FLM_Au.cls:1317-1318)
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{jfm}[2023/01/01 v1.28 Journal of Fluid Mechanics]
\LoadClass{article}
\renewcommand\theequation{\thesection.\@arabic\c@equation}
\@addtoreset{equation}{section}
\endinput
