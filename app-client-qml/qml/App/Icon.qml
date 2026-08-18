// Icon renders one vendored lucide glyph in an arbitrary colour.
//
// The twin of lucide-react on the Electron side, and the reason the two flavors
// show the *same* glyphs rather than two different icon sets. The SVGs live in
// qml/App/icons/ and are produced by tools/vendor-icons.mjs.
//
// How the tint works, and why it is not `colorization`: MultiEffect's colorization
// blends toward a hue while preserving the source's luminance, so a white glyph
// colorized to a mid grey comes out pale rather than that grey — close enough to
// look right and wrong enough to fail a contrast check. Instead the rendered SVG
// is used as an alpha *mask* over a solid rectangle of the requested colour, so
// the output is exactly Theme.muted, exactly Theme.accentText, and so on.
//
// sourceSize is bound to the scaled pixel size rather than left to default,
// because an SVG rasterised at its natural size and then scaled by the item is
// blurry. Rasterising at the size it will occupy keeps it crisp at every UI scale.

import QtQuick
import QtQuick.Effects

Item {
    id: root

    // Kebab-case lucide name, e.g. "panel-left-close". Matching a file in icons/.
    required property string name
    property int size: Theme.iconSize
    property color color: Theme.text

    implicitWidth: size
    implicitHeight: size

    // A missing icon is a mistake worth seeing rather than an invisible gap: the
    // glyph simply does not appear, and Image logs its own load failure — which is
    // visible because the host sets QT_ASSUME_STDERR_HAS_CONSOLE=1.
    Image {
        id: glyph
        anchors.fill: parent
        source: root.name ? Qt.resolvedUrl("icons/" + root.name + ".svg") : ""
        sourceSize.width: root.size
        sourceSize.height: root.size
        fillMode: Image.PreserveAspectFit
        smooth: true
        // Rendered only as the mask input, never drawn directly.
        visible: false
    }

    Rectangle {
        id: fill
        anchors.fill: parent
        color: root.color
        visible: false
        layer.enabled: true
    }

    MultiEffect {
        anchors.fill: parent
        source: fill
        maskEnabled: true
        maskSource: glyph
    }
}
