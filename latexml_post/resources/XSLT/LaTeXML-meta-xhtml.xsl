<?xml version="1.0" encoding="utf-8"?>
<!--
/=====================================================================\
|  LaTeXML-meta-xhtml.xsl                                             |
|  Converting various meta-level elements to xhtml                    |
|=====================================================================|
| Part of LaTeXML:                                                    |
|  Public domain software, produced as part of work done by the       |
|  United States Government & not subject to copyright in the US.     |
|=====================================================================|
| Bruce Miller <bruce.miller@nist.gov>                        #_#     |
| http://dlmf.nist.gov/LaTeXML/                              (o o)    |
\=========================================================ooo==U==ooo=/
-->
<xsl:stylesheet
    version     = "1.0"
    xmlns:xsl   = "http://www.w3.org/1999/XSL/Transform"
    xmlns:ltx   = "http://dlmf.nist.gov/LaTeXML"
    xmlns:f     = "http://dlmf.nist.gov/LaTeXML/functions"
    xmlns:m     = "http://www.w3.org/1998/Math/MathML"
    extension-element-prefixes="f"
    exclude-result-prefixes = "ltx f m">

  <!-- ======================================================================
       Typically invisible meta elements
       ltx:note, ltx:indexmark, ltx:rdf, ltx:ERROR
       ====================================================================== -->

  <!-- Only a few generated elements need $context switches.
       See the CONTEXT discussion in LaTeXML-common -->

  <!-- normally hidden, but should be exposable various ways.
       The role will likely distinguish various modes of footnote, endnote,
       and other annotation -->
  <xsl:preserve-space elements="ltx:note"/>
  <!-- An accessible description (acmart \Description) is never rendered: it is
       hidden by ltx_nodisplay and referenced by aria-describedby, so assistive
       technology reads its text content directly. The generic note template
       below would wrap it in the footnote scaffolding — a leading dagger mark,
       a second mark inside ltx_note_content, and a "<role>: " type prefix —
       all of which lands in the computed accessible description ("†† : …").
       Emit a plain span instead.

       The class is written out directly rather than via add_attributes, which
       derives an ltx_[local-name] class from the element: these carry ltx:note
       only because that is what the document builder places inside a float's
       caption, and labelling them ltx_note would assert footnote semantics they
       do not have. -->
  <xsl:template match="ltx:note[contains(@class,'ltx_acm_description')]" priority="2">
    <xsl:param name="context"/>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:call-template name="add_id"/>
      <xsl:attribute name="class"><xsl:value-of select="@class"/></xsl:attribute>
      <xsl:apply-templates>
        <xsl:with-param name="context" select="'inline'"/>
      </xsl:apply-templates>
    </xsl:element>
  </xsl:template>

  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #990): an author's note with no mark (no @mark, no tag)
       prints no glyph: its mark is unknown here (an unbound class's \thanks, a document's own \maketitle) or none is
       printed (a picture, an ORCID icon, 2107.13277; 2404.13552's page-overlay notice; a \titlenote, 1003.2281; \pacs
       codes, 0803.0204), and the frontmatter never invents one: note-mark's dagger below
       (Perl's) put another author's glyph on 1807.01940's name (DG 2026-11-16: a wrong or invented glyph is worse
       than none). So no ltx_note_mark and no hidden ltx_note_content (shown on hovering the mark): its content inline,
       with no "note: " type label, after its author's name (class ltx_note_unmarked). A mark-only note with no
       glyph (role footnotemark, its text never placed) shows nothing; its id stays. -->
  <xsl:template match="ltx:creator/ltx:note[not(normalize-space(@mark)) and not(ltx:tags/ltx:tag[not(@role)])]" priority="1">
    <xsl:param name="context"/>
    <xsl:choose>
      <xsl:when test="@role='footnotemark' and not(node()[not(self::ltx:tags)][normalize-space(.) or self::*])">
        <xsl:if test="@xml:id">
          <xsl:element name="span" namespace="{$html_ns}">
            <xsl:call-template name="add_id"/>
          </xsl:element>
        </xsl:if>
      </xsl:when>
      <xsl:otherwise>
        <xsl:call-template name="unmarked-creator-note">
          <xsl:with-param name="context" select="$context"/>
        </xsl:call-template>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <xsl:template name="unmarked-creator-note">
    <xsl:param name="context"/>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:variable name="innercontext" select="'inline'"/><!-- override -->
      <xsl:call-template name="add_id"/>
      <xsl:call-template name="add_attributes">
        <xsl:with-param name="extra_classes" select="'ltx_note_unmarked'"/>
      </xsl:call-template>
      <xsl:apply-templates select="." mode="begin">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:apply-templates>
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:apply-templates select="." mode="end">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
    </xsl:element>
  </xsl:template>

  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #1052): a note shared by several authors is copied onto each
       carrier, each copy with its @mark (Perl's model: relocateAnnotations, Base_Utility.pool.ltxml:895-901). The page
       prints it once (DG 2026-11-16, rulings 1-2): the first copy in full, with class ltx_note_shared; every later copy
       of the same block, role, mark and text is a mark linked to it (2606.00446, 2407.07614, 1706.03762). Only an
       author's notes render, so only theirs are keyed (an editor's copy first would link to no page element); a note
       with no mark or no text is never a repeat (two image-only notes are two notes). -->
  <xsl:key name="f:shared-creator-note"
           match="ltx:creator[@role='author']/ltx:note[normalize-space(@mark)][normalize-space(.)][not(@role='footnotemark')]"
           use="concat(generate-id(../..),'&#x9;',@role,'&#x9;',@mark,'&#x9;',normalize-space(.))"/>
  <!-- The same notes by block and mark alone, and the mark-only notes (role="footnotemark": a mark printed on a name
       that no note of its own answers, an affiliation letter, an unpaired \footnotemark; 0801.1438) by block and
       mark. A mark-only note links to the note its mark names in the block (one note, however many copies), which it
       then makes shared, but only a symbol mark or Perl's tagged leftover: a letter (Latin or Greek) or digit is as
       often an affiliation label, and a wrong link is worse than none (revREND2). -->
  <xsl:key name="f:creator-note-by-mark"
           match="ltx:creator[@role='author']/ltx:note[normalize-space(@mark)][normalize-space(.)][not(@role='footnotemark')]"
           use="concat(generate-id(../..),'&#x9;',normalize-space(@mark))"/>
  <xsl:key name="f:creator-mark-only"
           match="ltx:creator[@role='author']/ltx:note[@role='footnotemark'][normalize-space(@mark)]
                  [ltx:tags or translate(normalize-space(@mark),'0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεζηθικλμνξοπρςστυφχψωΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ','')
                               = normalize-space(@mark)]"
           use="concat(generate-id(../..),'&#x9;',normalize-space(@mark))"/>

  <xsl:template
      match="ltx:creator[@role='author']/ltx:note[normalize-space(@mark)][normalize-space(.)][not(@role='footnotemark')]">
    <xsl:param name="context"/>
    <xsl:variable name="copies"
                  select="key('f:shared-creator-note',
                              concat(generate-id(../..),'&#x9;',@role,'&#x9;',@mark,'&#x9;',normalize-space(.)))"/>
    <xsl:variable name="by_mark" select="concat(generate-id(../..),'&#x9;',normalize-space(@mark))"/>
    <xsl:variable name="marked_elsewhere"
                  select="key('f:creator-mark-only',$by_mark)[count(.. | current()/..) = 2]
                          and count(key('f:creator-note-by-mark',$by_mark)) = count($copies)"/>
    <xsl:choose>
      <xsl:when test="count($copies) = 1 and not($marked_elsewhere)">
        <xsl:call-template name="note">
          <xsl:with-param name="context" select="$context"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:when test="generate-id() = generate-id($copies[1])">
        <xsl:call-template name="note">
          <xsl:with-param name="context" select="$context"/>
          <xsl:with-param name="extra_classes" select="'ltx_note_shared'"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise>
        <xsl:element name="span" namespace="{$html_ns}">
          <xsl:call-template name="add_id"/>
          <xsl:attribute name="class">ltx_note ltx_role_footnotemark ltx_note_shared</xsl:attribute>
          <xsl:call-template name="creator-note-mark">
            <xsl:with-param name="target" select="$copies[1]"/>
            <xsl:with-param name="glyph" select="string(@mark)"/>
          </xsl:call-template>
        </xsl:element>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- A creator's mark-only note is its mark alone, with no empty "footnotemark: " pop-up: linked, and shared, when it
       is an author's linkable mark (f:creator-mark-only) and its mark names one author's note in the block; else the
       plain glyph. Its glyph is note-mark's (Perl's tag, else @mark). -->
  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #1059): no dangling mark (DG 2026-11-16, 2006.10886). A
       mark-only note that links to no note on the page shows its glyph only where the same glyph (each piece of a
       printed run, `1,2`) prints elsewhere in the document (f:printed-glyph), as it was likely meant as connected; or
       where it names a title's or date's note, whose mark the page does not print (class ltx_mark_titlenote, set by
       its given number: `\footnotemark[1]` under `\title{T\thanks{…}}`); or where its name carries a note with no
       known mark, which may print that glyph (1503.07953's `\altaffiltext{*}{Corresponding author}`). Open: a legend
       whose glyph the XML drops (an author-line contact's consumed label, 1302.0490; an e-mail or correspondence contact
       with its symbol cut) leaves the mark hidden though the PDF prints it twice.
       Else its glyph, printed once over its name alone, connects nothing and shows nothing (its id stays; the XML
       keeps the note). -->
  <xsl:template match="ltx:creator/ltx:note[@role='footnotemark'][normalize-space(@mark)]">
    <xsl:variable name="by_mark" select="concat(generate-id(../..),'&#x9;',normalize-space(@mark))"/>
    <xsl:variable name="notes"
                  select="key('f:creator-note-by-mark',$by_mark)[count(key('f:creator-mark-only',$by_mark) | current())
                                                                 = count(key('f:creator-mark-only',$by_mark))]"/>
    <xsl:variable name="target"
                  select="$notes[1][count($notes) = count(key('f:shared-creator-note',
                            concat(generate-id(../..),'&#x9;',@role,'&#x9;',@mark,'&#x9;',normalize-space(.))))]"/>
    <xsl:variable name="glyph">
      <xsl:choose>
        <xsl:when test="ltx:tags/ltx:tag[not(@role)]"><xsl:value-of select="ltx:tags/ltx:tag[not(@role)]"/></xsl:when>
        <xsl:otherwise><xsl:value-of select="@mark"/></xsl:otherwise>
      </xsl:choose>
    </xsl:variable>
    <xsl:variable name="pieces">
      <xsl:call-template name="f:glyph-pieces">
        <xsl:with-param name="glyph" select="normalize-space($glyph)"/>
      </xsl:call-template>
    </xsl:variable>
    <xsl:variable name="elsewhere">
      <xsl:call-template name="f:glyph-elsewhere">
        <xsl:with-param name="pieces" select="string($pieces)"/>
        <xsl:with-param name="me" select="generate-id()"/>
      </xsl:call-template>
    </xsl:variable>
    <xsl:choose>
      <xsl:when test="not($target) and not(string($elsewhere))
                      and not(contains(concat(' ',@class,' '),' ltx_mark_titlenote '))
                      and not(../ltx:note[not(@role='footnotemark')][not(normalize-space(@mark))]
                                         [not(ltx:tags/ltx:tag[not(@role)])][normalize-space(.)])">
        <xsl:if test="@xml:id">
          <xsl:element name="span" namespace="{$html_ns}">
            <xsl:call-template name="add_id"/>
          </xsl:element>
        </xsl:if>
      </xsl:when>
      <xsl:otherwise>
        <xsl:element name="span" namespace="{$html_ns}">
          <xsl:call-template name="add_id"/>
          <xsl:call-template name="add_attributes">
            <xsl:with-param name="extra_classes"><xsl:if test="$target">ltx_note_shared</xsl:if></xsl:with-param>
          </xsl:call-template>
          <xsl:call-template name="creator-note-mark">
            <xsl:with-param name="target" select="$target"/>
            <xsl:with-param name="glyph" select="$glyph"/>
          </xsl:call-template>
        </xsl:element>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #1059): a note several authors carry is one note, the xml:id
       of its copy first in document order the idref of each other copy (the engine's identity, DG 2026-11-16, #893
       Option A; never its text).
       An UNMARKED one (no @mark, no tag: its class prints no mark, or its glyph is unknown) prints once, at its first
       carrier, under a mark the page invents so the reader can tell whom it is about: [a], [b], … in document order,
       class ltx_mark_generated, a bracketed letter the document prints nowhere (DG 2026-11-16: never \@fnsymbol, never a bare
       letter or digit); each other copy is that mark alone, linked to it. A one-carrier unmarked note stays under its
       name, with no mark. -->
  <xsl:key name="f:note-copies" match="ltx:creator/ltx:note[@idref]" use="@idref"/>
  <xsl:template match="ltx:creator/ltx:note[not(normalize-space(@mark))][not(ltx:tags/ltx:tag[not(@role)])]
                                           [@idref or key('f:note-copies',@xml:id)]" priority="2">
    <xsl:param name="context"/>
    <xsl:variable name="first" select="current()[not(@idref)] | //ltx:creator/ltx:note[@xml:id = current()/@idref]"/>
    <xsl:variable name="glyph">
      <xsl:call-template name="f:generated-mark">
        <xsl:with-param name="n"
            select="count($first/preceding::ltx:note[parent::ltx:creator][not(normalize-space(@mark))]
                          [not(ltx:tags/ltx:tag[not(@role)])][not(@idref)][key('f:note-copies',@xml:id)]) + 1"/>
      </xsl:call-template>
    </xsl:variable>
    <xsl:choose>
      <xsl:when test="not(@idref)">
        <xsl:call-template name="note">
          <xsl:with-param name="context" select="$context"/>
          <xsl:with-param name="extra_classes" select="'ltx_note_shared'"/>
          <xsl:with-param name="glyph" select="string($glyph)"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise>
        <xsl:element name="span" namespace="{$html_ns}">
          <xsl:call-template name="add_id"/>
          <xsl:attribute name="class">ltx_note ltx_role_footnotemark ltx_note_shared</xsl:attribute>
          <xsl:call-template name="creator-note-mark">
            <xsl:with-param name="target" select="$first"/>
            <xsl:with-param name="glyph" select="string($glyph)"/>
            <xsl:with-param name="generated" select="true()"/>
          </xsl:call-template>
        </xsl:element>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- The pieces of a mark's printed glyph, each followed by a comma: a run of letters or digits one piece, each other
       character one (`1*` is 1 and *, `a,b` a and b), the separators of a run or range (`,` `-` `–`) and spaces none. -->
  <xsl:template name="f:glyph-pieces">
    <xsl:param name="glyph"/>
    <xsl:param name="run" select="''"/>
    <xsl:variable name="alnum"
                  select="'0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεζηθικλμνξοπρςστυφχψω'"/>
    <xsl:variable name="c" select="substring($glyph,1,1)"/>
    <xsl:choose>
      <xsl:when test="$glyph = ''"><xsl:if test="$run != ''"><xsl:value-of select="concat($run,',')"/></xsl:if></xsl:when>
      <xsl:when test="translate($c,$alnum,'') = ''">
        <xsl:call-template name="f:glyph-pieces">
          <xsl:with-param name="glyph" select="substring($glyph,2)"/>
          <xsl:with-param name="run" select="concat($run,$c)"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise>
        <xsl:if test="$run != ''"><xsl:value-of select="concat($run,',')"/></xsl:if>
        <xsl:if test="not(contains(', -&#x2013;',$c))"><xsl:value-of select="concat($c,',')"/></xsl:if>
        <xsl:call-template name="f:glyph-pieces">
          <xsl:with-param name="glyph" select="substring($glyph,2)"/>
        </xsl:call-template>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- Each glyph the document prints, asterisks folded (* = ∗), by the element printing it: a note's mark; a note's or
       creator's printed tag (not a section's number); a superscript; a formula of the glyph alone inside a note,
       contact or publication note (2012.13912's `\authornote{… \\ $\dagger$ Xiaojiang Peng is …}`, 2508.05506's
       `\blfootnote{$\dagger$ Prior to joining …}`); the symbol, or the run of letters or digits, that a note, contact or
       publication note opens with, its tags left out (a legend's `$*$Xiang Wang is the corresponding author`,
       2010.10783). -->
  <xsl:key name="f:printed-glyph" match="ltx:note[normalize-space(@mark)]"
           use="translate(normalize-space(@mark),'&#x2217;','*')"/>
  <xsl:key name="f:printed-glyph" match="ltx:tag[not(@role)][ancestor::ltx:note or ancestor::ltx:creator] | ltx:sup"
           use="translate(normalize-space(.),'&#x2217;','*')"/>
  <!-- (a bracketed one by its glyph out of its brackets too, as a name's mark run keeps its pieces: the legend
       `$^{(2)}$ Parasites and Microbes` prints the 2 of `Tonkin-Hill$^{(2),(3)}$`, 2302.11378) -->
  <xsl:key name="f:printed-glyph"
           match="ltx:note[contains(@mark,'(') or contains(@mark,'[')]/@mark
                  | ltx:tag[not(@role)][ancestor::ltx:note or ancestor::ltx:creator][contains(.,'(') or contains(.,'[')]
                  | ltx:sup[contains(.,'(') or contains(.,'[')]"
           use="translate(normalize-space(.),'&#x2217;()[]','*')"/>
  <!-- (a formula's one token, in its XMath or its MathML, whichever the post-processing kept) -->
  <xsl:key name="f:printed-glyph"
           match="ltx:XMTok[ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote]
                                              [count(.//ltx:XMTok[normalize-space()]) = 1]]
                  | *[self::m:mo or self::m:mi or self::m:mn]
                     [ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote]
                                        [count(.//m:mo[normalize-space()] | .//m:mi[normalize-space()]
                                               | .//m:mn[normalize-space()]) = 1]]"
           use="translate(normalize-space(.),'&#x2217;','*')"/>
  <xsl:key name="f:printed-glyph"
           match="*[self::ltx:note or self::ltx:contact or self::ltx:pubnote]
                   [translate(substring(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),1,1),
                              '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεζηθικλμνξοπρςστυφχψω','')]"
           use="translate(substring(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),1,1),
                          '&#x2217;','*')"/>
  <xsl:key name="f:printed-glyph"
           match="*[self::ltx:note or self::ltx:contact or self::ltx:pubnote]
                   [normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1])]
                   [not(translate(substring(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),1,1),
                                  '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεζηθικλμνξοπρςστυφχψω',''))]"
           use="substring-before(concat(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),'|'),
                  substring(concat(translate(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),
                                             '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεζηθικλμνξοπρςστυφχψω',''),
                                   '|'),1,1))"/>

  <!-- "yes" where a piece of a mark's glyph ($pieces, `f:glyph-pieces`) prints elsewhere in the document
       (f:printed-glyph), or in another name's mark run (`Dhami$^{1,3}$` prints the 1 of `Zečević$^{1}$`, 2110.12052):
       anywhere but in the mark-only note $me itself. -->
  <xsl:template name="f:glyph-elsewhere">
    <xsl:param name="pieces"/>
    <xsl:param name="me"/>
    <xsl:variable name="piece" select="translate(normalize-space(substring-before($pieces,',')),'&#x2217;','*')"/>
    <xsl:choose>
      <xsl:when test="not(contains($pieces,','))"/>
      <xsl:when test="$piece != ''
                      and (key('f:printed-glyph',$piece)[not(ancestor-or-self::ltx:note[generate-id() = $me])]
                           or //ltx:creator/ltx:note[@role='footnotemark'][generate-id() != $me]
                                [contains(concat(',',translate(normalize-space(@mark),'-&#x2013;&#x2217; ',',,*'),','),
                                          concat(',',$piece,','))])">yes</xsl:when>
      <xsl:otherwise>
        <xsl:call-template name="f:glyph-elsewhere">
          <xsl:with-param name="pieces" select="substring-after($pieces,',')"/>
          <xsl:with-param name="me" select="$me"/>
        </xsl:call-template>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- The $n-th bracketed letter the document prints nowhere (f:printed-glyph: no note's mark or tag, no superscript):
       [a], [b], … -->
  <xsl:template name="f:generated-mark">
    <xsl:param name="n"/>
    <xsl:param name="at" select="1"/>
    <xsl:variable name="letters" select="'abcdefghijklmnopqrstuvwxyz'"/>
    <xsl:variable name="candidate"
                  select="concat('[',substring($letters,(($at - 1) mod 26) + 1,1),
                                 substring(concat($letters,$letters),1,floor(($at - 1) div 26)),']')"/>
    <xsl:choose>
      <xsl:when test="key('f:printed-glyph',$candidate) and $at &lt; 260">
        <xsl:call-template name="f:generated-mark">
          <xsl:with-param name="n" select="$n"/>
          <xsl:with-param name="at" select="$at + 1"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:when test="$n &gt; 1">
        <xsl:call-template name="f:generated-mark">
          <xsl:with-param name="n" select="$n - 1"/>
          <xsl:with-param name="at" select="$at + 1"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise><xsl:value-of select="$candidate"/></xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- An author's mark: the glyph, linked to the $target note's body when it has one on the page. -->
  <xsl:template name="creator-note-mark">
    <xsl:param name="target"/>
    <xsl:param name="glyph"/>
    <xsl:param name="generated" select="false()"/>
    <xsl:element name="sup" namespace="{$html_ns}">
      <xsl:attribute name="class">ltx_note_mark<xsl:if test="$generated"> ltx_mark_generated</xsl:if></xsl:attribute>
      <xsl:choose>
        <xsl:when test="$target/@fragid">
          <xsl:element name="a" namespace="{$html_ns}">
            <xsl:attribute name="href">#<xsl:value-of select="$target/@fragid"/></xsl:attribute>
            <xsl:value-of select="$glyph"/>
          </xsl:element>
        </xsl:when>
        <xsl:otherwise><xsl:value-of select="$glyph"/></xsl:otherwise>
      </xsl:choose>
    </xsl:element>
  </xsl:template>

  <xsl:template match="ltx:note" name="note">
    <xsl:param name="context"/>
    <xsl:param name="extra_classes" select="''"/>
    <xsl:param name="glyph" select="''"/>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:variable name="innercontext" select="'inline'"/><!-- override -->
      <xsl:call-template name="add_id"/>
      <xsl:call-template name="add_attributes">
        <xsl:with-param name="extra_classes" select="$extra_classes"/>
      </xsl:call-template>
      <xsl:apply-templates select="." mode="begin">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:call-template name="note-mark">
        <xsl:with-param name="context" select="$innercontext"/>
        <xsl:with-param name="glyph" select="$glyph"/>
      </xsl:call-template>
      <xsl:element name="span" namespace="{$html_ns}">
        <xsl:attribute name="class">ltx_note_outer</xsl:attribute>
        <xsl:element name="span" namespace="{$html_ns}">
          <xsl:attribute name="class">ltx_note_content</xsl:attribute>
          <xsl:call-template name="note-mark">
            <xsl:with-param name="context" select="$innercontext"/>
            <xsl:with-param name="glyph" select="$glyph"/>
          </xsl:call-template>
          <xsl:if test="not(@role = 'footnote')">
            <xsl:element name="span" namespace="{$html_ns}">
              <xsl:attribute name="class">ltx_note_type</xsl:attribute>
              <xsl:value-of select="@role"/>
              <xsl:text>: </xsl:text>
            </xsl:element>
          </xsl:if>
          <xsl:apply-templates>
            <xsl:with-param name="context" select="$innercontext"/>
          </xsl:apply-templates>
          <xsl:apply-templates select="." mode="end">
            <xsl:with-param name="context" select="$innercontext"/>
          </xsl:apply-templates>
        </xsl:element>
      </xsl:element>
    </xsl:element>
  </xsl:template>

  <xsl:preserve-space elements="ltx:note-mark"/>
  <xsl:template name="note-mark">
    <xsl:param name="glyph" select="''"/>
    <xsl:element name="sup" namespace="{$html_ns}">
      <xsl:attribute name="class">ltx_note_mark<xsl:if test="$glyph"> ltx_mark_generated</xsl:if></xsl:attribute>
      <xsl:choose>
        <xsl:when test="$glyph"><xsl:value-of select="$glyph"/></xsl:when>
        <xsl:when test="ltx:tags/ltx:tag[not(@role)]"><xsl:value-of select="ltx:tags/ltx:tag[not(@role)]"/></xsl:when>
        <xsl:when test="@mark"><xsl:value-of select="@mark"/></xsl:when>
        <xsl:otherwise>&#x2020;</xsl:otherwise>
      </xsl:choose>
    </xsl:element>
  </xsl:template>

  <!-- disappears -->
  <xsl:template match="ltx:declare"/>

  <!-- Actually, this ought to be annoyingly visible -->
  <xsl:preserve-space elements="ltx:ERROR"/>
  <xsl:template match="ltx:ERROR">
    <xsl:param name="context"/>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:variable name="innercontext" select="'inline'"/><!-- override -->
      <xsl:call-template name="add_id"/>
      <xsl:call-template name="add_attributes"/>
      <xsl:apply-templates select="." mode="begin">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:apply-templates>
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:apply-templates select="." mode="end">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
    </xsl:element>
  </xsl:template>

  <!-- The indexmark disappears -->
  <xsl:template match="ltx:indexmark"/>

  <!-- but the phrases it contains may be used in back-ref situations -->
  <xsl:preserve-space elements="ltx:indexphrase"/>
  <xsl:template match="ltx:indexphrase">
    <xsl:param name="context"/>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:variable name="innercontext" select="'inline'"/><!-- override -->
      <xsl:call-template name="add_id"/>
      <xsl:call-template name="add_attributes"/>
      <xsl:apply-templates select="." mode="begin">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:apply-templates>
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
      <xsl:apply-templates select="." mode="end">
        <xsl:with-param name="context" select="$innercontext"/>
      </xsl:apply-templates>
    </xsl:element>
  </xsl:template>

  <!-- Typically will end up with css display:none -->
  <xsl:preserve-space elements="ltx:rdf"/>
  <xsl:template match="ltx:rdf">
    <xsl:param name="context"/>
    <xsl:element name="{f:blockelement($context,'div')}" namespace="{$html_ns}">
      <xsl:call-template name="add_id"/>
      <xsl:call-template name="add_attributes"/>
      <xsl:apply-templates select="." mode="begin">
        <xsl:with-param name="context" select="$context"/>
      </xsl:apply-templates>
      <xsl:apply-templates>
        <xsl:with-param name="context" select="$context"/>
      </xsl:apply-templates>
      <xsl:apply-templates select="." mode="end">
        <xsl:with-param name="context" select="$context"/>
      </xsl:apply-templates>
    </xsl:element>
    <xsl:text>&#x0A;</xsl:text>
  </xsl:template>

</xsl:stylesheet>
