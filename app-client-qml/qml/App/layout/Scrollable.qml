// Scrollable — the twin of components/layout/Scrollable.tsx: a vertically scrolling
// container with the app's scrollbar.
//
// Every page in this template is one of these. Worth using rather than hand-rolling a
// Flickable each time: the contentHeight/clip/boundsBehavior trio is easy to get subtly
// wrong, and a page whose content height is unset simply does not scroll.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Flickable {
    id: root

    default property alias content: column.data
    property int padding: Theme.space6
    property int spacing: Theme.space5
    // Cap the content column, as pages do. 0 means "fill".
    property int maxWidth: 0

    contentWidth: width
    contentHeight: column.implicitHeight + padding * 2
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: ScrollBar {}

    ColumnLayout {
        id: column
        x: Math.max(root.padding, (root.width - width) / 2)
        y: root.padding
        width: root.maxWidth > 0 ? Math.min(root.width - root.padding * 2, root.maxWidth) : root.width - root.padding * 2
        spacing: root.spacing
    }
}
