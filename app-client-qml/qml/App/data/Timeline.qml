pragma ComponentBehavior: Bound

// Timeline — the twin of components/data/Timeline.tsx: events on a shared vertical spine.
//
// items: [{ title, time?, text?, tone? }]

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property var items: []
    spacing: 0

    Repeater {
        model: root.items
        delegate: RowLayout {
            id: entry
            required property var modelData
            required property int index
            readonly property bool last: index === root.items.length - 1

            Layout.fillWidth: true
            spacing: Theme.space3

            // Spine: the dot plus the line to the next entry, as one column so the line
            // is always centred under its own dot.
            Item {
                Layout.preferredWidth: Theme.px(12)
                Layout.fillHeight: true
                Layout.minimumHeight: Theme.px(40)

                Rectangle {
                    id: dot
                    anchors.horizontalCenter: parent.horizontalCenter
                    y: Theme.px(4)
                    width: Theme.px(9)
                    height: width
                    radius: width / 2
                    color: {
                        switch (entry.modelData.tone) {
                        case "success":
                            return Theme.green;
                        case "error":
                            return Theme.red;
                        case "warning":
                            return Theme.yellow;
                        default:
                            return Theme.accent;
                        }
                    }
                }

                Rectangle {
                    visible: !entry.last
                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.top: dot.bottom
                    anchors.topMargin: Theme.px(2)
                    anchors.bottom: parent.bottom
                    width: 1
                    color: Theme.border
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                Layout.bottomMargin: entry.last ? 0 : Theme.space4
                spacing: Theme.px(2)

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.space2

                    Txt {
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                        text: entry.modelData.title ?? ""
                        pixelSize: Theme.fontSm
                        weight: Font.Medium
                        color: Theme.text
                        elide: Text.ElideRight
                    }

                    MonoText {
                        visible: entry.modelData.time !== undefined
                        text: entry.modelData.time ?? ""
                        color: Theme.muted
                    }
                }

                Txt {
                    Layout.fillWidth: true
                    visible: entry.modelData.text !== undefined
                    text: entry.modelData.text ?? ""
                    pixelSize: Theme.fontXs
                    color: Theme.subtext
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
