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
    from PySide6.QtCore import QUrl, QtMsgType, qInstallMessageHandler
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

    for width in [int(w) for w in args.widths.split(",")]:
        window.setWidth(width)
        window.setHeight(700)
        # Two passes: layouts settle asynchronously, so measuring immediately after a
        # resize reads the previous frame's geometry. This is the same class of problem
        # that makes Flow unreliable.
        for _ in range(3):
            app.processEvents()

        content = window.contentItem()
        for problem in overflowing(content, f"[{width}px]"):
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
