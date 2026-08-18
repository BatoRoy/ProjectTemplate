pragma ComponentBehavior: Bound

// Breadcrumbs — the twin of components/layout/Breadcrumbs.tsx.
//
// items: [{ label, id? }]. The last crumb is the current location and is not a link.

import QtQuick
import QtQuick.Layouts
import App

Flow {
    id: root

    property var items: []
    signal picked(int index, var item)

    spacing: Theme.space1

    Repeater {
        model: root.items
        delegate: Row {
            id: crumb
            required property var modelData
            required property int index
            readonly property bool last: index === root.items.length - 1
            spacing: Theme.space1

            Txt {
                anchors.verticalCenter: parent.verticalCenter
                text: crumb.modelData.label ?? crumb.modelData
                pixelSize: Theme.fontXs
                color: crumb.last ? Theme.text : crumbMouse.containsMouse ? Theme.accentText : Theme.muted
                weight: crumb.last ? Font.Medium : Font.Normal

                MouseArea {
                    id: crumbMouse
                    anchors.fill: parent
                    enabled: !crumb.last
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.picked(crumb.index, crumb.modelData)
                }
            }

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: !crumb.last
                name: "chevron-right"
                size: Theme.px(12)
                color: Theme.muted
            }
        }
    }
}
