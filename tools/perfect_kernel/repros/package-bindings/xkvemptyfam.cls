\ProvidesClass{xkvemptyfam}
\RequirePackage{xkeyval}
\define@key[sap]{}{english}[]{\def\SAPlang{english}}
\define@key[sap]{}{binding}{\def\SAPbinding{#1}}
\DeclareOptionX[sap]<>{oneside}{\def\SAPside{one}}
\DeclareOptionX*{\ClassError{xkvemptyfam}{Option '\CurrentOption' is not valid}{}}
\ProcessOptionsX[sap]<>
\LoadClass{article}
\key@ifundefined[sap]{}{english}{\def\SAPkey{missing}}{\def\SAPkey{found}}
