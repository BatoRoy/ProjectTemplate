#!/usr/bin/env python3
"""Load the QML UI and report what static analysis cannot see.

qmllint is static: it catches unqualified access, shadowed properties and unknown
types. The bugs a QML app actually ships are runtime sizing problems — a row whose
children do not shrink, a Flow laying out against stale widths — and they only appear
when something is instantiated at a particular size.

So this does two things qmllint cannot:

  1. Instantiates the real components and fails on any QML warning. Note that a
     ReferenceError in a binding is *silent* by default and does not affect the exit
     code, which is why QT_ASSUME_STDERR_HAS_CONSOLE=1 is set below and why "it ran
     without complaining" is not evidence of anything without it.

  2. Resizes the window across the range a user can actually drag it to and asserts
     that no element has escaped its parent. That is the mechanical version of the
     hand-written notes in BatoAI's components ("progressive hiding in a RowLayout is
     predictable; Flow is not") — which were learned by shipping the bug.

Runs offscreen, so it needs no display and belongs in CI.

    python3 tools/check-qml.py [--widths 300,600,900]

Requires PySide6 (pip install PySide6). It is a development dependency only — the app
itself never imports Python.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

# Must be set before Qt initialises, or QML diagnostics never reach us.
os.environ.setdefault("QT_ASSUME_STDERR_HAS_CONSOLE", "1")
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

try:
    from PySide6.QtCore import QObject, QUrl, QtMsgType, qInstallMessageHandler
    from PySide6.QtGui import QGuiApplication
    from PySide6.QtQml import QQmlApplicationEngine
    # Importing QtQuick is not decorative: without it shiboken has no QQuickWindow type
    # registered, so rootObjects()[0] comes back as a plain QWindow and contentItem()
    # does not exist on it.
    from PySide6.QtQuick import QQuickItem, QQuickWindow  # noqa: F401
except ImportError:
    print("✗ PySide6 is not installed — pip install PySide6", file=sys.stderr)
    print("  (development-only dependency; the app itself never uses Python)", file=sys.stderr)
    sys.exit(2)

# Dialogs to open and measure. Named explicitly rather than discovered, so deleting one
# from Main.qml fails this check instead of quietly reducing its coverage.
DIALOGS = {"AppOptions", "AboutDialog"}

ROOT = Path(__file__).resolve().parent.parent
QML_ROOT = ROOT / "app-client-qml" / "qml"
MAIN = QML_ROOT / "App" / "Main.qml"

messages: list[str] = []


def handler(mode, context, message):
    # Qt emits an "unknown" category for a few internal notices; only warnings and
    # worse are treated as failures.
    if mode in (QtMsgType.QtWarningMsg, QtMsgType.QtCriticalMsg, QtMsgType.QtFatalMsg):
        messages.append(message)
    print(message, file=sys.stderr)


def type_name(item) -> str:
    """A readable name for a QML-defined component.

    type(item).__name__ is QQuickItem for everything declared in QML. The metaobject
    carries the real name, mangled as e.g. "Card_QMLTYPE_42".
    """
    cls = item.metaObject().className()
    for marker in ("_QMLTYPE_", "_QML_"):
        if marker in cls:
            cls = cls.split(marker)[0]
    return cls.removeprefix("QQuick") or "Item"


def scrollable(item, needed: float) -> bool:
    """Is there a Flickable in here that can actually reach `needed` pixels of content?"""
    if item.metaObject().indexOfProperty("contentHeight") >= 0:
        if (item.property("contentHeight") or 0.0) > item.height() - 1.0:
            return True
    return any(scrollable(c, needed) for c in item.childItems())


def overflowing(item, path="root", out=None, depth=0):
    """Collect children whose geometry escapes their parent's box.

    Only visible items with a real size are considered: an item that has not been laid
    out yet is 0x0 and would otherwise report as fitting.
    """
    if out is None:
        out = []

    # A Flickable's content is *supposed* to be bigger than the viewport — that is what
    # scrolling is. Descend into it (its children still have to fit each other) but do
    # not measure it against the viewport.
    #
    # Detected by the presence of contentHeight rather than by type name: every page in
    # this template is a *subclass* of Flickable, so its metaobject reports "HomePage",
    # not "Flickable", and a name comparison silently flags every scrolling page.
    parent_is_flickable = item.metaObject().indexOfProperty("contentHeight") >= 0

    for child in item.childItems():
        if not child.isVisible() or child.width() <= 0 or child.height() <= 0:
            continue
        name = type_name(child)
        here = f"{path} > {name}"

        # A MouseArea that is bigger than the thing it sits on is the *point*: a 16px
        # glyph is a 16px hit target, which is a miss more often than a hit, so the
        # template widens them with negative margins. Input handlers draw nothing, so
        # they cannot overflow anything visually.
        if name == "MouseArea":
            continue

        # An explicit opt-out for decorations that are deliberately larger than their
        # parent — the Swatch selection ring, which grows outward so selecting a colour
        # does not resize it. Marked at the declaration rather than pattern-matched
        # here, so the exemption lives next to the reason for it.
        if child.objectName() == "overflow-ok":
            continue

        if not parent_is_flickable:
            # A 1px allowance: Qt lays out in floats and rounds to device pixels, so an
            # exact comparison reports phantom overflows on fractional scales.
            if child.x() + child.width() > item.width() + 1.0:
                out.append(f"{here}: right edge {child.x() + child.width():.0f} "
                           f"exceeds parent width {item.width():.0f}")
            if child.y() + child.height() > item.height() + 1.0:
                out.append(f"{here}: bottom edge {child.y() + child.height():.0f} "
                           f"exceeds parent height {item.height():.0f}")

        overflowing(child, here, out, depth + 1)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--widths", default="560,700,900,1200",
                    help="window widths to check, comma separated. The low end should match Main.qml's minimumWidth — narrower is a state the WM cannot produce.")
    args = ap.parse_args()

    if not MAIN.exists():
        print(f"✗ {MAIN} not found", file=sys.stderr)
        return 2

    qInstallMessageHandler(handler)
    app = QGuiApplication(sys.argv)

    engine = QQmlApplicationEngine()
    # The same -I the host passes: the App module has to be on the import path or every
    # `import App` fails.
    engine.addImportPath(str(QML_ROOT))
    engine.load(QUrl.fromLocalFile(str(MAIN)))

    if not engine.rootObjects():
        print("✗ the UI failed to load", file=sys.stderr)
        return 1

    window = engine.rootObjects()[0]
    failures: list[str] = []

    # Dialogs are checked too, and they are the ones that need it most: they are the
    # tallest thing in the app and they are never on screen during normal use, so a
    # dialog that runs off the bottom of a small window is invisible until a user with a
    # small window opens it. App Options did exactly that before the Modal body was made
    # to scroll.
    #
    # Searched as QObject, not QQuickItem: a Controls Dialog is a Popup, which is NOT an
    # Item — it merely owns one. Filtering on QQuickItem finds nothing and the whole
    # check passes vacuously, which is exactly what it did before this comment existed.
    dialogs = [c for c in window.findChildren(QObject)
               if type_name(c) in DIALOGS]
    if len(dialogs) != len(DIALOGS):
        print(f"✗ expected {sorted(DIALOGS)}, found {sorted(type_name(d) for d in dialogs)}",
              file=sys.stderr)
        return 1
    for d in dialogs:
        d.setProperty("visible", True)

    for width in [int(w) for w in args.widths.split(",")]:
        window.setWidth(width)
        # The window's own declared minimum, so a dialog is measured against the
        # smallest box a user can actually put it in.
        window.setHeight(420 if width == min(int(w) for w in args.widths.split(",")) else 700)
        # Two passes: layouts settle asynchronously, so measuring immediately after a
        # resize reads the previous frame's geometry. This is the same class of problem
        # that makes Flow unreliable.
        for _ in range(3):
            app.processEvents()

        content = window.contentItem()
        for problem in overflowing(content, f"[{width}px]"):
            failures.append(problem)

        # Dialogs are checked for *reachability*, not for fitting.
        #
        # "Is the dialog taller than the window" can never fail: Qt clamps a popup to the
        # window it belongs to, so an overlong dialog silently reports the window's own
        # height while its content is cut off. Measured: App Options wants 809px, and
        # reports exactly 420 in a 420px window whether or not its content can be
        # scrolled to.
        #
        # The property that actually matters is therefore: if the natural content is
        # taller than the box it was given, something must be able to scroll it.
        for d in dialogs:
            name = type_name(d)
            dw, dh = d.property("width"), d.property("height")
            natural = d.property("implicitHeight") or 0.0

            if dw > window.width() + 1.0:
                failures.append(
                    f"[{width}px] {name}: width {dw:.0f} exceeds the window's "
                    f"{window.width():.0f}")

            content = d.property("contentItem")
            if content is None:
                continue

            if natural > dh + 1.0 and not scrollable(content, dh):
                failures.append(
                    f"[{width}x{window.height():.0f}] {name}: content is {natural:.0f}px "
                    f"tall in a {dh:.0f}px dialog with nothing to scroll it — the bottom "
                    f"is unreachable")

            for problem in overflowing(content, f"[{width}px] {name}"):
                failures.append(problem)

    print()
    if messages:
        print(f"✗ {len(messages)} QML warning(s)", file=sys.stderr)
    if failures:
        print(f"✗ {len(failures)} layout overflow(s):", file=sys.stderr)
        for f in failures:
            print(f"    {f}", file=sys.stderr)

    if messages or failures:
        return 1

    print(f"✓ no QML warnings; nothing overflows at {args.widths}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
