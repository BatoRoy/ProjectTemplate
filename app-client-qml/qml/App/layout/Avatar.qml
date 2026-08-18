// Avatar — the twin of components/layout/Avatar.tsx: an image, or initials on a tinted
// circle when there is none.
//
// The tint is derived from the name so the same person is the same colour every time,
// rather than random per mount. Ink comes from onColor, because the fill is solid.

import QtQuick
import App

Rectangle {
    id: root

    property string name: ""
    property url source: ""
    property int size: Theme.px(32)

    implicitWidth: size
    implicitHeight: size
    radius: width / 2
    color: Qt.hsla(hue, 0.45, Theme.isLight ? 0.62 : 0.42, 1.0)
    clip: true

    // A stable hue per name: sum the codepoints and wrap. Not cryptographic, just
    // deterministic — the requirement is only that it does not change between runs.
    readonly property real hue: {
        var h = 0;
        for (var i = 0; i < name.length; ++i)
            h = (h + name.charCodeAt(i) * 17) % 360;
        return h / 360;
    }

    readonly property string initials: {
        var parts = name.trim().split(/\s+/).filter(p => p.length > 0);
        if (parts.length === 0)
            return "?";
        if (parts.length === 1)
            return parts[0].substring(0, 2).toUpperCase();
        return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
    }

    Txt {
        anchors.centerIn: parent
        visible: root.source == ""
        text: root.initials
        pixelSize: Math.round(root.size * 0.38)
        weight: Font.Medium
        color: Theme.onColor(root.color)
    }

    Image {
        anchors.fill: parent
        visible: root.source != ""
        source: root.source
        fillMode: Image.PreserveAspectCrop
        sourceSize.width: root.size
        sourceSize.height: root.size
        smooth: true
    }
}
