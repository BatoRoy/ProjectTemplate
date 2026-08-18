// NavRow is one sidebar row. The twin of the inner `Row` component in
// components/Sidebar.tsx, including the tooltip that only appears when collapsed.
//
// The active state is an accent *tint* with normal-weight text — never a solid
// accent fill. This is the suite's rule and the specific thing BatoAI got wrong
// before its rebuild: a solid accent pill with white text measured 1.82:1.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property string icon: ""
    property string label: ""
    property bool active: false
    property bool collapsed: false
    signal clicked()

    Layout.fillWidth: true
    implicitHeight: Theme.rowHeight
    radius: Theme.radius

    color: active ? Theme.accentTint
         : mouse.containsMouse ? Theme.alpha(Theme.card, 0.6)
         : "transparent"
    Behavior on color { ColorAnimation { duration: Theme.animFast } }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: root.collapsed ? 0 : Theme.space3
        anchors.rightMargin: root.collapsed ? 0 : Theme.space3
        spacing: Theme.px(10)

        // When collapsed the glyph is centred, which two flexible spacers achieve
        // without a second layout branch.
        Item { Layout.fillWidth: root.collapsed }

        Icon {
            name: root.icon
            size: Theme.iconSize
            color: root.active ? Theme.accentTextOnBg
                 : mouse.containsMouse ? Theme.subtext
                 : Theme.muted
        }

        Txt {
            visible: !root.collapsed
            text: root.label
            pixelSize: Theme.fontSm
            weight: root.active ? Font.Medium : Font.Normal
            color: root.active ? Theme.text
                 : mouse.containsMouse ? Theme.text
                 : Theme.muted
            elide: Text.ElideRight
            Layout.fillWidth: !root.collapsed
            Layout.minimumWidth: 0
        }

        Item { Layout.fillWidth: root.collapsed }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }

    // Collapsed rows have no label, so the tooltip is the only way to know what a
    // glyph does. Controls' attached ToolTip is used rather than a custom component
    // because it already handles the delay, the follow and the screen-edge flip.
    ToolTip.visible: root.collapsed && mouse.containsMouse
    ToolTip.text: root.label
    ToolTip.delay: 400
}
