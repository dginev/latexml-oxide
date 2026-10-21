"""Rebuild synthetic graphics fixtures (optional: reportlab + Pillow).

CC0-1.0. No scientific paper content or external images.
PDF compression is deliberately enabled: content must not be classified by
scanning uncompressed PDF drawing commands. Tests consume the committed PDFs.
"""
from pathlib import Path
from PIL import Image
from reportlab.pdfgen import canvas
root = Path(__file__).parent
Image.new('RGB', (8, 8), (45, 100, 180)).save(root / 'synthetic_tile.png')
for name in ['synthetic_vector', 'mixed_vector_image', 'image_only']:
    pdf = canvas.Canvas(str(root / (name + '.pdf')), pagesize=(180, 120),
                        pageCompression=1, invariant=1)
    if name != 'synthetic_vector':
        pdf.drawImage(str(root / 'synthetic_tile.png'), 10, 10, width=80, height=80)
    if name != 'image_only':
        pdf.drawString(10, 100, 'Vector label')
        pdf.setDash(3, 2)
        pdf.line(10, 5, 170, 110)
    pdf.showPage()
    pdf.save()
# Transparency/soft-mask witness: retain vectors alongside an RGBA image.
alpha = Image.new('RGBA', (8, 8), (45, 100, 180, 128))
alpha.putpixel((0, 0), (45, 100, 180, 0))
alpha.save(root / 'synthetic_alpha_tile.png')
pdf = canvas.Canvas(str(root / 'mixed_alpha_image.pdf'), pagesize=(180, 120),
                    pageCompression=1, invariant=1)
pdf.drawImage(str(root / 'synthetic_alpha_tile.png'), 10, 10, width=80, height=80,
              mask='auto')
pdf.drawString(10, 100, 'Vector label')
pdf.setDash(3, 2)
pdf.line(10, 5, 170, 110)
pdf.showPage()
pdf.save()
