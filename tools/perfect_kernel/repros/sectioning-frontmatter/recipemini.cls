% A recipe class in miniature (RecipeBook.cls:207-209, :284-286, :300-302): each recipe's \title typesets
% in place, its \author goes to the page footer, and \contentsname switches the font unbraced.
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{recipemini}[2026/10/11 a recipe class in miniature]
\LoadClass{article}
\RequirePackage{fancyhdr}
\pagestyle{fancy}
\renewcommand{\contentsname}{\bfseries\Large Recipes}
\RenewDocumentCommand{\title}{O{\Large} m}{\begingroup #1\bfseries #2\endgroup}
\RenewDocumentCommand{\author}{O{Author: } m}{\lfoot{#1#2}}
