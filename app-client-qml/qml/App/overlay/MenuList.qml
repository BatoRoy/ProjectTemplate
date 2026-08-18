pragma ComponentBehavior: Bound

// MenuList — the twin of components/Menu.tsx: the purely presentational menu body that
// ContextMenu and Dropdown both render. It positions nothing and owns no popup; it is
// only the list, so the two consumers cannot drift apart visually.
//
// items: [{ label, icon?, danger?, separator?, enabled?, onTriggered? }]

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property var items: []
    signal triggered(int index, var item)

    color: Theme.card
    border.color: Theme.border
    border.width: 1
    radius: Theme.radius
    implicitWidth: Math.max(Theme.px(160), column.implicitWidth + Theme.space2 * 2)
    implicitHeight: column.implicitHeight + Theme.space1 * 2
    clip: true

    ColumnLayout {
        id: column
        anchors.fill: parent
        anchors.margins: Theme.space1
        spacing: 0

        Repeater {
            model: root.items
            delegate: Item {
                id: entry
                required property var modelData
                required property int index

                readonly property bool isSeparator: modelData.separator === true
                readonly property bool isEnabled: modelData.enabled !== false

                Layout.fillWidth: true
                implicitHeight: isSeparator ? Theme.space2 : Theme.rowHeight

                Rectangle {
                    visible: entry.isSeparator
                    anchors.centerIn: parent
                    width: parent.width - Theme.space2 * 2
                    height: 1
                    color: Theme.border
                }

                Rectangle {
                    visible: !entry.isSeparator
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space1
                    anchors.rightMargin: Theme.space1
                    radius: Theme.radiusSm
                    color: (entryMouse.containsMouse && entry.isEnabled) ? Theme.alpha(Theme.border, 0.5) : "transparent"

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Theme.space2
                        anchors.rightMargin: Theme.space2
                        spacing: Theme.space2

                        Icon {
                            visible: entry.modelData.icon !== undefined
                            name: entry.modelData.icon ?? "check"
                            size: Theme.px(14)
                            color: entry.modelData.danger === true ? Theme.red : Theme.muted
                        }

                        Txt {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 0
                            text: entry.modelData.label ?? ""
                            pixelSize: Theme.fontSm
                            elide: Text.ElideRight
                            color: !entry.isEnabled ? Theme.muted : entry.modelData.danger === true ? Theme.readableOn(Theme.red, Theme.card) : Theme.text
                        }

                        Icon {
                            visible: entry.modelData.checked === true
                            name: "check"
                            size: Theme.px(13)
                            color: Theme.accentText
                        }
                    }
                }

                MouseArea {
                    id: entryMouse
                    anchors.fill: parent
                    enabled: !entry.isSeparator && entry.isEnabled
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (entry.modelData.onTriggered)
                            entry.modelData.onTriggered();
                        root.triggered(entry.index, entry.modelData);
                    }
                }
            }
        }
    }
}
