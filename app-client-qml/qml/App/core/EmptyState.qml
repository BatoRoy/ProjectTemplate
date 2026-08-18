// EmptyState is the twin of EmptyState in components/Feedback.tsx: a muted glyph,
// a title, and one line explaining what would fill the space.

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property string icon: "box"
    property string title: ""
    property string subtitle: ""

    spacing: Theme.space3

    Icon {
        Layout.alignment: Qt.AlignHCenter
        name: root.icon
        size: Theme.px(34)
        color: Theme.muted
        opacity: 0.6
    }

    Txt {
        Layout.alignment: Qt.AlignHCenter
        text: root.title
        color: Theme.text
        pixelSize: Theme.fontSm
        weight: Font.Medium
    }

    Txt {
        Layout.alignment: Qt.AlignHCenter
        // Capped rather than filling the parent: an explanation stretched across a
        // wide window is one very long line and reads worse than two short ones.
        Layout.maximumWidth: Theme.px(420)
        visible: root.subtitle !== ""
        text: root.subtitle
        color: Theme.muted
        pixelSize: Theme.fontXs
        wrapMode: Text.WordWrap
        horizontalAlignment: Text.AlignHCenter
    }
}
