\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{replayerr}[2026/10/12 a class author setter that stops on an error (65e repro)]
\LoadClass{article}
\newcounter{authcount}
\renewcommand\author[1]{\global\advance\c@authcount by 1\relax\@latex@error{Unknown author #1}\@ehc}
