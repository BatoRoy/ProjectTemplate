pragma ComponentBehavior: Bound

// EditorTabs — the twin of components/layout/EditorTabs.tsx: the VS Code style strip, with
// a dirty dot and a close affordance per tab.
//
// Scrolls horizontally rather than shrinking tabs to nothing, which is what an editor
// strip does: a tab too narrow to read its filename is worse than one off-screen.
//
// tabs: [{ id, label, dirty? }]

import QtQuick
import QtQuick.Controls
import App

Rectangle {
    id: root

    property var tabs: []
    property string current: ""
    signal picked(string id)
    signal closed(string id)

    implicitHeight: Theme.px(34)
    color: Theme.bg

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: 1
        color: Theme.border
    }

    ListView {
        anchors.fill: parent
        orientation: ListView.Horizontal
        model: root.tabs
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.horizontal: ScrollBar {
            policy: ScrollBar.AsNeeded
        }

        delegate: Rectangle {
            id: tab
            required property var modelData
            readonly property bool active: root.current === modelData.id

            width: label.implicitWidth + Theme.px(52)
            height: root.height
            color: active ? Theme.card : tabMouse.containsMouse ? Theme.alpha(Theme.card, 0.5) : "transparent"

            // The active tab is marked with an accent rule on top, not a fill: a filled
            // tab in a strip of eight is a wall of accent.
            Rectangle {
                anchors.top: parent.top
                width: parent.width
                height: 2
                color: tab.active ? Theme.accent : "transparent"
            }

            Rectangle {
                anchors.right: parent.right
                width: 1
                height: parent.height
                color: Theme.border
            }

            Txt {
                id: label
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.space3
                text: tab.modelData.label ?? ""
                pixelSize: Theme.fontXs
                color: tab.active ? Theme.text : Theme.muted
            }

            // Dirty dot where the close button goes, swapping on hover — the editor
            // convention, and it keeps the tab width stable.
            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: Theme.space3
                visible: tab.modelData.dirty === true && !closeMouse.containsMouse
                width: Theme.px(7)
                height: width
                radius: width / 2
                color: Theme.subtext
            }

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: Theme.space3 - Theme.px(2)
                visible: !(tab.modelData.dirty === true) || closeMouse.containsMouse
                name: "x"
                size: Theme.px(12)
                color: closeMouse.containsMouse ? Theme.text : Theme.muted

                MouseArea {
                    id: closeMouse
                    anchors.fill: parent
                    anchors.margins: -Theme.px(3)
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.closed(tab.modelData.id)
                }
            }

            MouseArea {
                id: tabMouse
                anchors.fill: parent
                hoverEnabled: true
                // Below the close target, so clicking the x does not also select the tab.
                z: -1
                onClicked: root.picked(tab.modelData.id)
            }
        }
    }
}
