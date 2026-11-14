\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{replayunbal}[2026/10/12 a class author setter that closes a group of the document and errs (65e repro)]
\LoadClass{article}
\newcounter{authcount}
\renewcommand\author[1]{\global\advance\c@authcount by 1\relax\egroup\@latex@error{Unknown author #1}\@ehc}
