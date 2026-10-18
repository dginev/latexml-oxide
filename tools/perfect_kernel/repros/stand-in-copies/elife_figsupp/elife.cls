\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{elife}[2022/06/06, v1.11]
\LoadClass{extarticle}
\RequirePackage{etoolbox}
\RequirePackage{graphicx}
\newcounter{figsupp}
\setcounter{figsupp}{0}
\newcounter{fsdata}
\newcounter{fssrccode}
\RequirePackage{newfile}
\newoutputstream{suppinfo}
\openoutputfile{\jobname.suppinfo}{suppinfo}
\addtostream{suppinfo}{\protect\setcounter{figure}{0}}
\DeclareRobustCommand{\figsuppdata}[1]{%
  \protect\refstepcounter{fsdata}%
  \par\small\textbf{\figurename~\thefigure---figure supplement \arabic{figsupp}---source data~\arabic{fsdata}. } #1\par
}
\DeclareRobustCommand{\figsuppsrccode}[1]{%
  \protect\refstepcounter{fssrccode}%
  \par\small\textbf{\figurename~\thefigure---figure supplement \arabic{figsupp}---source code~\arabic{fssrccode}. } #1\par
}
\AtBeginEnvironment{figure}{%
  \setcounter{figsupp}{0}
  \addtostream{suppinfo}{
    \protect\setcounter{figsupp}{0}
    \protect\refstepcounter{figure}
  }
}
%% Updated 2017/06/30 to allow optional argument
\DeclareRobustCommand{\figsupp}[3][]{%
  \refstepcounter{figsupp}%
  \addtostream{suppinfo}{%
  \protect\refstepcounter{figsupp}
  \protect\setcounter{fsdata}{0}
  \protect\setcounter{fssrccode}{0}
  }%
  {%
  \ifstrequal{#1}{none}{}{%
    \small\textbf{\figurename~\thefigure---figure supplement \arabic{figsupp}.} 
    \ifstrempty{#1}{%
      \renewcommand\figsuppdata[1]{}%
      \renewcommand\figsuppsrccode[1]{}%
      #2}{%
      \renewcommand\figsuppdata[1]{}%
      \renewcommand\figsuppsrccode[1]{}%
      #1}}}\raggedright\par
  \addtostream{suppinfo}{%
    \noindent\protect\begin{minipage}{\linewidth}
    \protect #3\noexpand\par
    \textbf{\figurename~\thefigure---figure supplement \arabic{figsupp}.} #2\noexpand\par
    \vskip8pt
    \protect\end{minipage}
    \vskip16pt
  }
}
\AtEndDocument{%
  \closeoutputstream{suppinfo}
  \input{\jobname.suppinfo}
}
