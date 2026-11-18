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
    xmlns:exsl  = "http://exslt.org/common"
    extension-element-prefixes="f exsl"
    exclude-result-prefixes = "ltx f m exsl">

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
        <xsl:call-template name="unmarked-note">
          <xsl:with-param name="context" select="$context"/>
        </xsl:call-template>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #1063): #990's rule beyond the creators, for the notes the PDF
       prints with no mark in the title block or as a footnote. The roles of a note of the document's own level that
       do: a note's text (icml's notice, 2602.04557; a markless \footnotetext, 2511.18287), an address, an e-mail, a
       correspondence or funding line (Interspeech's \address and \email, 2405.13379, 2603.01467), elsarticle's
       highlights and graphical abstract, printed before the title (2510.16146, 2303.00352). Each shows as a line of its
       own, once per role and text (sn-jnl's \equalcont under each name, 2412.01641; a note with no text, a picture, is
       never a repeat). Every other role, a running head or the publication's metadata (jmlr's heading, 1710.08165; a
       volume; a short title), the PDF prints in no title block: Perl's hidden note. Keyed by role and text. -->
  <xsl:key name="f:document-note-shown"
           match="/ltx:document/ltx:note[not(normalize-space(@mark)) and not(ltx:tags/ltx:tag[not(@role)])]
                  [@role='footnote' or @role='footnotetext' or @role='thanks' or @role='affiliationnotice'
                   or @role='address' or @role='affiliation' or @role='email' or @role='corresponding'
                   or @role='correspondence' or @role='authorinfo' or @role='equal-contributors' or @role='funding'
                   or @role='highlights' or @role='graphicalabstract' or @role='url']"
           use="concat(@role,'&#x9;',normalize-space(.))"/>
  <xsl:template match="ltx:document/ltx:note[key('f:document-note-shown',concat(@role,'&#x9;',normalize-space(.)))]
                       [not(normalize-space(@mark)) and not(ltx:tags/ltx:tag[not(@role)])]" priority="0.5">
    <xsl:param name="context"/>
    <xsl:choose>
      <xsl:when test="not(normalize-space(.))
                      or generate-id() = generate-id(key('f:document-note-shown',concat(@role,'&#x9;',normalize-space(.)))[1])">
        <xsl:call-template name="unmarked-note">
          <xsl:with-param name="context" select="$context"/>
          <xsl:with-param name="element" select="f:blockelement($context,'div')"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:when test="@xml:id">
        <xsl:element name="span" namespace="{$html_ns}">
          <xsl:call-template name="add_id"/>
        </xsl:element>
      </xsl:when>
    </xsl:choose>
  </xsl:template>

  <!-- SURPASS (OXIDIZED_DESIGN_DIVERGENCES #1063): a note with no mark inside a frontmatter field, the date, the abstract,
       the keywords, the classification, a publication note, a creator's entry, shows its text, inline in a paragraph (a
       markless \footnotetext in the abstract, 2506.06941). Not in a title: ltx:title is also a section's, a theorem's or a
       float's, whose notes keep Perl's dagger (body notes, numbered by the counter). -->
  <xsl:template match="ltx:note[not(normalize-space(@mark)) and not(ltx:tags/ltx:tag[not(@role)])]
                               [ancestor::ltx:date or ancestor::ltx:abstract or ancestor::ltx:keywords
                                or ancestor::ltx:classification or ancestor::ltx:pubnote or ancestor::ltx:creator]"
                priority="0.5">
    <xsl:param name="context"/>
    <xsl:call-template name="unmarked-note">
      <xsl:with-param name="context" select="$context"/>
      <xsl:with-param name="element" select="f:blockelement($context,'div')"/>
    </xsl:call-template>
  </xsl:template>

  <xsl:template name="unmarked-note">
    <xsl:param name="context"/>
    <xsl:param name="element" select="'span'"/>
    <xsl:element name="{$element}" namespace="{$html_ns}">
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
           match="ltx:creator/ltx:note[../@role='author'][normalize-space(@mark)][normalize-space(.)][not(@role='footnotemark')]"
           use="concat(generate-id(../..),'&#x9;',@role,'&#x9;',@mark,'&#x9;',normalize-space(.))"/>
  <!-- The same notes by block and mark alone, and the linkable mark-only notes (a symbol, or Perl's tagged leftover),
       ∗ folded to *: a mark-only note links to the one note its mark names (OD #1052, R22; 1705.05427, 1903.10478,
       1610.06031, 0801.1438, 2406.11149). -->
  <xsl:key name="f:creator-note-by-mark"
           match="ltx:creator/ltx:note[../@role='author'][normalize-space(@mark)][normalize-space(.)][not(@role='footnotemark')]"
           use="concat(generate-id(../..),'&#x9;',translate(normalize-space(@mark),'&#x2217;','*'))"/>
  <xsl:key name="f:creator-mark-only"
           match="ltx:creator/ltx:note[../@role='author'][@role='footnotemark'][normalize-space(@mark)]
                  [ltx:tags or translate(normalize-space(@mark),'0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZαβγδεζηθικλμνξοπρςστυφχψωΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ','')
                               = normalize-space(@mark)]"
           use="concat(generate-id(../..),'&#x9;',translate(normalize-space(@mark),'&#x2217;','*'))"/>
  <!-- An author's note by block and the character its text opens with: a legend of the glyph (2607.28158). -->
  <xsl:key name="f:creator-note-opening"
           match="ltx:creator/ltx:note[../@role='author'][not(@role='footnotemark')]"
           use="concat(generate-id(../..),'&#x9;',
                  translate(substring(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),1,1),
                            '&#x2217;','*'))"/>
  <!-- A glyph the title page prints outside the author blocks (no section, float or table holds it; ∗ folded to *): a
       note's mark (`$\ast$` too), the symbol a note, a publication note, the date, a title, a paragraph or a table cell
       opens with (revCONNFIXb s4, b3); a mark of it links
       nowhere (OD #1052; revCONNFIX p1, p3-p5, q4, q5; a body footnote or a table's `*` line not, 2008.06630, 2406.07029).
       (The patterns walk only the root, its children and the subtrees of the children no excluded unit is: libxslt
       builds every key of a document at the first key() call, and a full-document pattern pass costs 4-8 % of a
       conversion, 0906.1883, 1607.07280. The node sets are those of `X[not(ancestor::…)]`.) -->
  <xsl:key name="f:glyph-elsewhere"
           match="/ltx:note[normalize-space(@mark)][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/ltx:note[normalize-space(@mark)][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:note[normalize-space(@mark)][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]"
           use="translate(normalize-space(@mark),'&#x2217;$^{} ','*')"/>
  <xsl:key name="f:glyph-elsewhere"
           match="/ltx:note[translate(@mark,'$^{} ','') = '\ast'][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/ltx:note[translate(@mark,'$^{} ','') = '\ast'][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:note[translate(@mark,'$^{} ','') = '\ast'][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]"
           use="'*'"/>
  <xsl:key name="f:glyph-elsewhere"
           match="/*[self::ltx:note or self::ltx:pubnote or self::ltx:date or self::ltx:title or self::ltx:subtitle or self::ltx:p
                     or self::ltx:td][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[self::ltx:note or self::ltx:pubnote or self::ltx:date or self::ltx:title or self::ltx:subtitle or self::ltx:p
                     or self::ltx:td][not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:note[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:pubnote[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:date[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:title[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:subtitle[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:p[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]
                  |/*/*[not(self::ltx:creator or self::ltx:section or self::ltx:chapter or self::ltx:part or self::ltx:appendix
                       or self::ltx:bibliography or self::ltx:index or self::ltx:float or self::ltx:table or self::ltx:figure)]
                    //ltx:td[not(ancestor::ltx:creator or ancestor::ltx:section or ancestor::ltx:chapter or ancestor::ltx:part or ancestor::ltx:appendix
                       or ancestor::ltx:bibliography or ancestor::ltx:index or ancestor::ltx:float or ancestor::ltx:table or ancestor::ltx:figure)]"
           use="translate(substring(normalize-space((.//text()[normalize-space()][not(ancestor::ltx:tags)])[1]),1,1),
                          '&#x2217;','*')"/>

  <xsl:template
      match="ltx:creator[@role='author']/ltx:note[normalize-space(@mark)][normalize-space(.)][not(@role='footnotemark')]">
    <xsl:param name="context"/>
    <xsl:variable name="copies"
                  select="key('f:shared-creator-note',
                              concat(generate-id(../..),'&#x9;',@role,'&#x9;',@mark,'&#x9;',normalize-space(.)))"/>
    <xsl:variable name="by_mark"
                  select="concat(generate-id(../..),'&#x9;',translate(normalize-space(@mark),'&#x2217;','*'))"/>
    <xsl:variable name="marked_elsewhere"
                  select="key('f:creator-mark-only',$by_mark)[count(.. | current()/..) = 2]
                            [not(contains(concat(' ',@class,' '),' ltx_note_label '))
                             or normalize-space(@mark) = normalize-space(current()/@mark)]
                          and count(key('f:creator-note-by-mark',$by_mark)) = count($copies)
                          and not(key('f:creator-mark-only',$by_mark)[normalize-space(@mark) != normalize-space(current()/@mark)]
                                  /parent::*[count(. | $copies/..) = count($copies/..)])
                          and not(key('f:creator-note-opening',$by_mark)[count(. | $copies) != count($copies)])
                          and not(key('f:glyph-elsewhere',substring-after($by_mark,'&#x9;')))"/>
    <xsl:choose>
      <xsl:when test="count($copies) = 1 and not($marked_elsewhere)">
        <!-- (anchored where a mark-only note of the block carries its glyph: that mark's copy may name this note, even
             in the same creator — `\thanks{…}\footnotemark[1]` — so the copy's position-anchor always has its anchor;
             an unused name is inert. REVCSSHL, OD #1075) -->
        <xsl:call-template name="note">
          <xsl:with-param name="context" select="$context"/>
          <xsl:with-param name="anchor" select="boolean(key('f:creator-mark-only',$by_mark))"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:when test="generate-id() = generate-id($copies[1])">
        <xsl:call-template name="note">
          <xsl:with-param name="context" select="$context"/>
          <xsl:with-param name="extra_classes" select="'ltx_note_shared'"/>
          <xsl:with-param name="anchor" select="true()"/>
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
          <xsl:call-template name="f:note-copy">
            <xsl:with-param name="target" select="$copies[1]"/>
          </xsl:call-template>
        </xsl:element>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- A creator's mark-only note is its mark alone, with no empty "footnotemark: " pop-up: linked, and shared, when it
       is an author's linkable mark (f:creator-mark-only) and its mark names one author's note in the block; else the
       plain glyph. Its glyph is note-mark's (Perl's tag, else @mark). -->
  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #1059): a mark-only note that links to no note on the page
       still shows its glyph (DG 2026-11-16, "hide only when certain"; 2006.10886's once-printed mark): the XML cannot
       prove the glyph prints once, its legend often where the frontmatter does not see it (a body line, the \date, a
       tabular cell, plain-text \dag or *), kept with its glyph lost (2311.11099, 2402.05011, 1302.0490) or inside
       another name's run (2110.12052). -->
  <xsl:template match="ltx:creator/ltx:note[@role='footnotemark'][normalize-space(@mark)]">
    <xsl:variable name="by_mark"
                  select="concat(generate-id(../..),'&#x9;',translate(normalize-space(@mark),'&#x2217;','*'))"/>
    <xsl:variable name="notes"
                  select="key('f:creator-note-by-mark',$by_mark)[count(key('f:creator-mark-only',$by_mark) | current())
                                                                 = count(key('f:creator-mark-only',$by_mark))]
                            [not(contains(concat(' ',current()/@class,' '),' ltx_note_label '))
                             or normalize-space(@mark) = normalize-space(current()/@mark)]"/>
    <xsl:variable name="target"
                  select="$notes[1][count($notes) = count(key('f:shared-creator-note',
                            concat(generate-id(../..),'&#x9;',@role,'&#x9;',@mark,'&#x9;',normalize-space(.))))]
                          [not(key('f:creator-mark-only',$by_mark)[normalize-space(@mark) != normalize-space($notes[1]/@mark)]
                                    /parent::*[count(. | $notes/..) = count($notes/..)])]
                          [not(key('f:creator-note-opening',$by_mark)[count(. | $notes) != count($notes)])]
                          [not(key('f:glyph-elsewhere',substring-after($by_mark,'&#x9;')))]"/>
    <xsl:variable name="glyph">
      <xsl:choose>
        <xsl:when test="ltx:tags/ltx:tag[not(@role)]"><xsl:value-of select="ltx:tags/ltx:tag[not(@role)]"/></xsl:when>
        <xsl:otherwise><xsl:value-of select="@mark"/></xsl:otherwise>
      </xsl:choose>
    </xsl:variable>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:call-template name="add_id"/>
      <xsl:call-template name="add_attributes">
        <xsl:with-param name="extra_classes"><xsl:if test="$target">ltx_note_shared</xsl:if></xsl:with-param>
      </xsl:call-template>
      <xsl:call-template name="creator-note-mark">
        <xsl:with-param name="target" select="$target"/>
        <xsl:with-param name="glyph" select="$glyph"/>
      </xsl:call-template>
      <xsl:if test="$target">
        <xsl:call-template name="f:note-copy">
          <xsl:with-param name="target" select="$target"/>
        </xsl:call-template>
      </xsl:if>
    </xsl:element>
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
          <xsl:with-param name="anchor" select="true()"/>
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
          <xsl:call-template name="f:note-copy">
            <xsl:with-param name="target" select="$first"/>
            <xsl:with-param name="glyph" select="string($glyph)"/>
          </xsl:call-template>
        </xsl:element>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <!-- Each glyph the document prints, asterisks folded (* = ∗), by the element printing it: a note's mark; a note's or
       creator's printed tag (not a section's number); a superscript; a formula of the glyph alone inside a note,
       contact or publication note (2012.13912's `\authornote{… \\ $\dagger$ Xiaojiang Peng is …}`, 2508.05506's
       `\blfootnote{$\dagger$ Prior to joining …}`). A generated mark is a glyph none of these prints (f:generated-mark).
       Its one lookup asks for a bracketed letter, [a], [b], …, so the glyphs that cannot be one are not indexed: a
       bracketed mark out of its brackets (the 2 of `Tonkin-Hill$^{(2),(3)}$` that the legend `$^{(2)}$ Parasites and
       Microbes` prints, 2302.11378), and the one symbol or the run of letters or digits a note, contact or publication
       note opens with (a legend's `$*$Xiang Wang is the corresponding author`, 2010.10783). (libxslt builds every key
       of a document at the first key() call: each pattern starts at an element name, a note's mark stands for its
       note and a formula is reached through its note, so no pass tests every node; 0906.1883, 1607.07280.) -->
  <xsl:key name="f:printed-glyph"
           match="ltx:note/@mark[normalize-space()]
                  |ltx:note//*[self::ltx:tag and not(@role)
                    or self::ltx:XMTok and ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote][count(.//ltx:XMTok[normalize-space()]) = 1]
                    or (self::m:mo or self::m:mi or self::m:mn)
                       and ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote]
                                        [count(.//m:mo[normalize-space()] | .//m:mi[normalize-space()] | .//m:mn[normalize-space()]) = 1]]
                  |ltx:contact//*[self::ltx:XMTok and ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote][count(.//ltx:XMTok[normalize-space()]) = 1]
                    or (self::m:mo or self::m:mi or self::m:mn)
                       and ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote]
                                        [count(.//m:mo[normalize-space()] | .//m:mi[normalize-space()] | .//m:mn[normalize-space()]) = 1]]
                  |ltx:pubnote//*[self::ltx:XMTok and ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote][count(.//ltx:XMTok[normalize-space()]) = 1]
                    or (self::m:mo or self::m:mi or self::m:mn)
                       and ancestor::ltx:Math[ancestor::ltx:note or ancestor::ltx:contact or ancestor::ltx:pubnote]
                                        [count(.//m:mo[normalize-space()] | .//m:mi[normalize-space()] | .//m:mn[normalize-space()]) = 1]]
                  |ltx:creator//ltx:tag[not(@role)]
                  |ltx:sup"
           use="translate(normalize-space(.),'&#x2217;','*')"/>

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
    <xsl:param name="anchor" select="false()"/>
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
      <xsl:call-template name="note-outer">
        <xsl:with-param name="context" select="$innercontext"/>
        <xsl:with-param name="glyph" select="$glyph"/>
        <xsl:with-param name="anchor" select="$anchor"/>
      </xsl:call-template>
    </xsl:element>
  </xsl:template>

  <!-- A note's pop-up: its mark, its type and its content. A shared note's ($anchor) content box is the anchor its
       copies are laid over on wide screens (OD #1075); not its outer box, whose anchor-name builds the margin stack. -->
  <xsl:template name="note-outer">
    <xsl:param name="context"/>
    <xsl:param name="glyph" select="''"/>
    <xsl:param name="anchor" select="false()"/>
    <xsl:element name="span" namespace="{$html_ns}">
      <xsl:attribute name="class">ltx_note_outer</xsl:attribute>
      <xsl:element name="span" namespace="{$html_ns}">
        <xsl:attribute name="class">ltx_note_content</xsl:attribute>
        <xsl:if test="$anchor and @fragid">
          <xsl:attribute name="style">anchor-name:<xsl:call-template name="f:note-anchor"/></xsl:attribute>
        </xsl:if>
        <xsl:call-template name="note-mark">
          <xsl:with-param name="context" select="$context"/>
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
          <xsl:with-param name="context" select="$context"/>
        </xsl:apply-templates>
        <xsl:apply-templates select="." mode="end">
          <xsl:with-param name="context" select="$context"/>
        </xsl:apply-templates>
      </xsl:element>
    </xsl:element>
  </xsl:template>

  <!-- SURPASS (latexml-oxide OXIDIZED_DESIGN_DIVERGENCES #1052, DG 2026-11-17 "Copy in HTML only"): a shared note's
       other mark carries the $target note's pop-up too, hidden (class ltx_note_copy, aria-hidden), so its mark pops it
       up where the first is out of sight (narrow screens: 2407.07614, 2606.00446, 1706.03762, 2506.06941). The XML
       keeps one note; the copy holds no id, and its links are out of the tab order. On wide screens it is laid over its
       note in the margin, positioned at the note's anchor, so its mark highlights the note as the first mark does
       (OD #1075, DG 2026-11-17; 1706.03762). -->
  <xsl:template name="f:note-copy">
    <xsl:param name="target"/>
    <xsl:param name="glyph" select="''"/>
    <xsl:variable name="anchor">
      <xsl:for-each select="$target[1][@fragid]"><xsl:call-template name="f:note-anchor"/></xsl:for-each>
    </xsl:variable>
    <xsl:variable name="outer">
      <xsl:for-each select="$target[1]">
        <xsl:call-template name="note-outer">
          <xsl:with-param name="context" select="'inline'"/>
          <xsl:with-param name="glyph" select="$glyph"/>
        </xsl:call-template>
      </xsl:for-each>
    </xsl:variable>
    <xsl:apply-templates select="exsl:node-set($outer)/*" mode="f:note-copy">
      <xsl:with-param name="anchor" select="string($anchor)"/>
    </xsl:apply-templates>
  </xsl:template>
  <xsl:template match="*" mode="f:note-copy">
    <xsl:param name="anchor" select="''"/>
    <xsl:copy>
      <xsl:copy-of select="@*[not(local-name() = 'id' or local-name() = 'tabindex')]"/>
      <xsl:if test="not(parent::*)">
        <xsl:attribute name="class">ltx_note_outer ltx_note_copy</xsl:attribute>
        <xsl:attribute name="aria-hidden">true</xsl:attribute>
        <xsl:if test="$anchor">
          <xsl:attribute name="style">position-anchor:<xsl:value-of select="$anchor"/></xsl:attribute>
        </xsl:if>
      </xsl:if>
      <xsl:if test="@tabindex or (local-name() = 'a' and @href) or (local-name() = 'area' and @href)
                    or self::*[local-name() = 'button' or local-name() = 'input' or local-name() = 'select'
                               or local-name() = 'textarea' or local-name() = 'summary' or local-name() = 'iframe'
                               or local-name() = 'object' or local-name() = 'embed']
                    or ((local-name() = 'audio' or local-name() = 'video') and @controls)">
        <xsl:attribute name="tabindex">-1</xsl:attribute>
      </xsl:if>
      <xsl:apply-templates mode="f:note-copy"/>
    </xsl:copy>
  </xsl:template>
  <xsl:template match="text() | comment() | processing-instruction()" mode="f:note-copy">
    <xsl:copy/>
  </xsl:template>
  <!-- The current note's anchor name, a CSS dashed ident: its @fragid is an NCName, whose one character a CSS name may
       not hold, '.', is escaped. -->
  <xsl:template name="f:note-anchor">
    <xsl:text>--ltx-note-</xsl:text>
    <xsl:call-template name="f:css-escape-dots">
      <xsl:with-param name="s" select="string(@fragid)"/>
    </xsl:call-template>
  </xsl:template>
  <xsl:template name="f:css-escape-dots">
    <xsl:param name="s"/>
    <xsl:choose>
      <xsl:when test="contains($s,'.')">
        <xsl:value-of select="substring-before($s,'.')"/><xsl:text>\.</xsl:text>
        <xsl:call-template name="f:css-escape-dots">
          <xsl:with-param name="s" select="substring-after($s,'.')"/>
        </xsl:call-template>
      </xsl:when>
      <xsl:otherwise><xsl:value-of select="$s"/></xsl:otherwise>
    </xsl:choose>
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
