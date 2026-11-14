"""Derive the fonts this client embeds from the shared ones in native/assets/fonts.

GPUI's Linux text system (cosmic-text + swash in gpui_wgpu) never applies font
variations when rasterising, so the variable Inter would render every weight at
the default wght=400. We therefore instance the shared variable font at the
three weights the Hybrid spec uses (400/500/600). The outlines are untouched.
The shared fonts carry no hinting instructions. Chromium renders them with
FreeType's light autohinter (fontconfig hintstyle "slight"), which snaps
baselines, x-height and cap height to the pixel grid vertically. GPUI's
rasteriser (swash) only runs TrueType bytecode, so we bake the same autohinter
into the fonts with ttfautohint (it *is* FreeType's autohinter, emitted as
bytecode; it only hints the y direction). All faces are renamed ("Bato Inter", "Bato JetBrains Mono") so a system-wide
Inter/JetBrains Mono can never be matched instead of the bundled one.

    python -m venv /tmp/ft && /tmp/ft/bin/pip install fonttools ttfautohint-py
    /tmp/ft/bin/python scripts/make-fonts.py
"""
import io
import pathlib
import ttfautohint
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

here = pathlib.Path(__file__).resolve().parent.parent
shared = here.parent / "assets" / "fonts"
out = here / "assets" / "fonts"
out.mkdir(parents=True, exist_ok=True)


def rename(font, family, style, ps):
    name = font["name"]
    for rec in list(name.names):
        if rec.nameID in (1, 2, 3, 4, 6, 16, 17, 21, 22, 25):
            name.removeNames(nameID=rec.nameID)
    for nid, val in ((1, family), (2, style), (3, ps), (4, f"{family} {style}"), (6, ps), (16, family), (17, style)):
        name.setName(val, nid, 3, 1, 0x409)
        name.setName(val, nid, 1, 0, 0)


def save_hinted(font, path):
    buf = io.BytesIO()
    font.save(buf)
    path.write_bytes(ttfautohint.ttfautohint(in_buffer=buf.getvalue(), hint_composites=True, increase_x_height=0))


for wght, style in ((400, "Regular"), (500, "Medium"), (600, "SemiBold")):
    f = TTFont(shared / "InterVariable-latin.ttf")
    inst = instancer.instantiateVariableFont(f, {"wght": wght})
    inst["OS/2"].usWeightClass = wght
    rename(inst, "Bato Inter", style, f"BatoInter-{style}")
    save_hinted(inst, out / f"BatoInter-{wght}.ttf")

for src, wght, style in (("JetBrainsMono-Regular-latin.ttf", 400, "Regular"), ("JetBrainsMono-Medium-latin.ttf", 500, "Medium")):
    f = TTFont(shared / src)
    f["OS/2"].usWeightClass = wght
    rename(f, "Bato JetBrains Mono", style, f"BatoJetBrainsMono-{style}")
    save_hinted(f, out / f"BatoJetBrainsMono-{wght}.ttf")

# GPUI cannot change the font size inside one wrapped paragraph, but Home's
# Notifications paragraph is 13px with an inline 12px mono word. This face is
# JetBrains Mono with unitsPerEm × 13/12, so at 13px it draws exactly as 12px.
f = TTFont(shared / "JetBrainsMono-Regular-latin.ttf")
f["head"].unitsPerEm = round(f["head"].unitsPerEm * 13 / 12)
f["OS/2"].usWeightClass = 400
rename(f, "Bato JetBrains Mono 12in13", "Regular", "BatoJetBrainsMono12in13-Regular")
save_hinted(f, out / "BatoJetBrainsMono12in13-400.ttf")

for lic in ("OFL-Inter.txt", "OFL-JetBrainsMono.txt"):
    (out / lic).write_bytes((shared / lic).read_bytes())
print("fonts written to", out)
