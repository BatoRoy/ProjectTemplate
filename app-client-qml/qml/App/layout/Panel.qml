// Panel — one pane inside a PanelGroup. The twin of the Panel in
// components/layout/PanelGroup.tsx.
//
// It carries the sizing policy; PanelGroup does the arithmetic. Declared as its own type
// so a group's children can express minimum and initial sizes declaratively rather than
// the group taking parallel arrays.

import QtQuick
import App

Item {
    id: root

    // Share of the group, relative to its siblings. Equal weights split evenly.
    property real weight: 1
    // Never shrink past this, in scaled pixels. The reason a group cannot collapse a pane
    // to nothing by dragging.
    property int minimumSize: Theme.px(80)

    default property alias content: body.data
    clip: true

    Item {
        id: body
        anchors.fill: parent
    }
}
