// Txt is the app's text primitive. Every component draws text through this rather
// than through Text directly, so the "Text selection" setting has one lever instead
// of ninety.
//
// Named Txt, not Label, on purpose: QtQuick.Controls already exports a Label, and a
// file that happens to import Controls *after* App would silently get that one
// instead — with different property names (font.pixelSize rather than pixelSize).
// A unique name removes the import-order dependency entirely. It has no React
// counterpart to match; the web flavor styles bare spans with Tailwind classes.
//
// The web flavor gets selection for free — HTML text is selectable and its
// setting turns it *off* for a native desktop feel. QML is the other way round:
// Text is inert, and only TextEdit can be selected. So the polarity is inverted
// (see Theme.textSelect) and this component swaps implementations.
//
// The deviation, recorded in QML-CLIENT.md: an **elided** label is never selectable.
// TextEdit has no elide property at all, so a component that needs to truncate
// gets the Text implementation regardless of the setting. That covers table cells
// and sidebar rows; the paragraphs and values a user actually wants to copy are
// not elided, and mono values are selectable always (see MonoText).

pragma ComponentBehavior: Bound

import QtQuick
import App

Item {
    id: root

    property string text: ""
    property color color: Theme.text
    property int pixelSize: Theme.fontSm
    property int weight: Font.Normal
    property string family: Theme.fontSans
    property int elide: Text.ElideNone
    property int wrapMode: Text.NoWrap
    property int horizontalAlignment: Text.AlignLeft
    property int verticalAlignment: Text.AlignVCenter
    property real lineHeight: 1.0
    property real letterSpacing: 0
    // Font.AllUppercase rather than uppercasing the string, so the property reads
    // back as it was written and translations are not mangled.
    property int capitalization: Font.MixedCase

    // Selection is only possible on the TextEdit path, and only when nothing has
    // to be truncated.
    readonly property bool selectable: Theme.textSelect && elide === Text.ElideNone

    // Implicit width comes from TextMetrics, not from the loaded item.
    //
    // The obvious `implicitWidth: impl.implicitWidth` is wrong in one specific and very
    // damaging case: an elided Text that is currently sized to a zero-width parent reports
    // an implicit width of 0. Since the Loader fills this item, and this item's width is
    // often decided by a layout that is itself asking for the implicit width, an elided
    // label would report 0 — telling every enclosing layout it needs no space at all.
    // Measured: a Txt with elide set reported 0 where the same text without elide reported
    // 33. It is why SegmentedControl claimed 38px and clipped its own labels.
    //
    // TextMetrics measures the *unelided* text independently of any layout pass, which is
    // exactly what an implicit width should be.
    // Rounded up, with a pixel to spare. TextMetrics.width is fractionally *smaller* than
    // the width the Text actually needs (measured: 32 vs 32.34 for the same string and
    // font), so binding a width to it truncates by a sub-pixel — and an eliding Text given
    // a width a fraction under its natural one elides. That is how headings with plenty of
    // room around them came out as "PO…".
    implicitWidth: Math.ceil(metrics.width) + 1
    implicitHeight: Math.max(metrics.height, impl.implicitHeight)

    TextMetrics {
        id: metrics
        text: root.text
        font.family: root.family
        font.pixelSize: root.pixelSize
        font.weight: root.weight
        font.letterSpacing: root.letterSpacing
        font.capitalization: root.capitalization
    }

    Loader {
        id: impl
        anchors.fill: parent
        sourceComponent: root.selectable ? selectableText : plainText
    }

    Component {
        id: plainText
        Text {
            text: root.text
            color: root.color
            font.family: root.family
            font.pixelSize: root.pixelSize
            font.weight: root.weight
            font.letterSpacing: root.letterSpacing
            font.capitalization: root.capitalization
            elide: root.elide
            wrapMode: root.wrapMode
            horizontalAlignment: root.horizontalAlignment
            verticalAlignment: root.verticalAlignment
            lineHeight: root.lineHeight
            renderType: Text.NativeRendering
        }
    }

    Component {
        id: selectableText
        TextEdit {
            text: root.text
            color: root.color
            font.family: root.family
            font.pixelSize: root.pixelSize
            font.weight: root.weight
            font.letterSpacing: root.letterSpacing
            font.capitalization: root.capitalization
            wrapMode: root.wrapMode
            horizontalAlignment: root.horizontalAlignment
            verticalAlignment: root.verticalAlignment
            readOnly: true
            selectByMouse: true
            selectionColor: Theme.accentTintHi
            selectedTextColor: root.color
            renderType: Text.NativeRendering
            // A read-only TextEdit still takes focus on click and shows a caret,
            // which reads as an editable field. Neither is wanted for a label.
            activeFocusOnPress: false
            cursorVisible: false
        }
    }
}
