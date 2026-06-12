// Faithful port of Perl t/structure/myclass.cls.ltxml. Named `lxtestclass`
// (not `myclass`) so it never intercepts a real arXiv paper's bundled
// myclass.cls — matching the prior compiled fixture's registration.
DefMacro("\\my@class@stuff", "");
DeclareOption("acommonoption", "\\xdef\\my@class@stuff{\\my@class@stuff, acommonoption}");
DeclareOption("aclassoption", "\\xdef\\my@class@stuff{\\my@class@stuff, aclassoption}");
// Default-option handler (Perl `DeclareOption(undef, sub {...})`): forward any
// unrecognised option to article.cls.
DeclareOption(|| {
  PassOptions("article", "cls", [ToString(Digest(T_CS("\\CurrentOption")))]);
});
ProcessOptions();
LoadClass("article");
DefMacro("\\showclassstuff", "\\par\\noindent Class options: \\my@class@stuff");
