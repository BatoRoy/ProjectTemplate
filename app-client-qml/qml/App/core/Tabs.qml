// Tabs — the twin of components/Tabs.tsx. Same pill treatment as SegmentedControl, which
// is deliberate: the web flavor notes they share a style, and two lookalike controls that
// drift apart is worse than one shared look.
//
// Kept separate rather than aliased because the semantics differ: Tabs switch a view and
// take string ids, a SegmentedControl picks a value of any type.

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    // [{ id, label }]
    property var tabs: []
    property string value: ""
    signal picked(string id)

    implicitHeight: Theme.controlHeight
    implicitWidth: row.implicitWidth

    RowLayout {
        id: row
        anchors.fill: parent
        spacing: Theme.px(2)

        Repeater {
            model: root.tabs
            delegate: Rectangle {
                id: tab
                required property var modelData
                readonly property bool active: root.value === modelData.id

                Layout.preferredHeight: root.height
                Layout.preferredWidth: label.implicitWidth + Theme.space4 * 2
                radius: Theme.radius
                color: active ? Theme.accentTintMed : tabMouse.containsMouse ? Theme.alpha(Theme.card, 0.6) : "transparent"
                Behavior on color {
                    ColorAnimation {
                        duration: Theme.animFast
                    }
                }

                Txt {
                    id: label
                    anchors.centerIn: parent
                    text: tab.modelData.label
                    pixelSize: Theme.fontSm
                    weight: tab.active ? Font.Medium : Font.Normal
                    color: tab.active ? Theme.accentText : tabMouse.containsMouse ? Theme.text : Theme.muted
                }

                MouseArea {
                    id: tabMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.value = tab.modelData.id;
                        root.picked(tab.modelData.id);
                    }
                }
            }
        }
    }
}
