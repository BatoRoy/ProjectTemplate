pragma ComponentBehavior: Bound

// KanbanBoard — the twin of components/data/KanbanBoard.tsx: columns of cards, dragged
// between columns.
//
// columns: [{ id, title, cards: [{ id, title }] }]
//
// The web flavor uses dnd-kit; here a card is a draggable Item and each column is a
// DropArea, which is the whole mechanism.

import QtQuick
import QtQuick.Layouts
import App

RowLayout {
    id: root

    property var columns: []
    signal moved(string cardId, string toColumn)

    spacing: Theme.space3

    Repeater {
        model: root.columns
        delegate: Rectangle {
            id: col
            required property var modelData

            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumWidth: 0
            radius: Theme.radiusXl
            color: drop.containsDrag ? Theme.accentTint : Theme.alpha(Theme.border, 0.25)
            border.width: 1
            border.color: drop.containsDrag ? Theme.accentRing : Theme.border
            Behavior on color {
                ColorAnimation {
                    duration: Theme.animFast
                }
            }

            DropArea {
                id: drop
                anchors.fill: parent
                onDropped: drag => root.moved(drag.source.cardId, col.modelData.id)
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Theme.space2
                spacing: Theme.space2

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.space2

                    Txt {
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                        text: col.modelData.title ?? ""
                        pixelSize: Theme.fontXs
                        weight: Font.DemiBold
                        color: Theme.subtext
                        capitalization: Font.AllUppercase
                        letterSpacing: 0.6
                        elide: Text.ElideRight
                    }
                    Badge {
                        tone: "neutral"
                        text: String((col.modelData.cards ?? []).length)
                    }
                }

                Repeater {
                    model: col.modelData.cards ?? []
                    delegate: Rectangle {
                        id: card
                        required property var modelData
                        property string cardId: modelData.id ?? ""

                        Layout.fillWidth: true
                        implicitHeight: cardLabel.implicitHeight + Theme.space3 * 2
                        radius: Theme.radius
                        color: Theme.card
                        border.width: 1
                        border.color: cardDrag.drag.active ? Theme.accentRing : Theme.border
                        opacity: cardDrag.drag.active ? 0.8 : 1.0

                        Drag.active: cardDrag.drag.active
                        Drag.source: card
                        Drag.hotSpot.x: width / 2
                        Drag.hotSpot.y: height / 2

                        Txt {
                            id: cardLabel
                            anchors.fill: parent
                            anchors.margins: Theme.space3
                            text: card.modelData.title ?? ""
                            pixelSize: Theme.fontSm
                            color: Theme.text
                            wrapMode: Text.WordWrap
                        }

                        MouseArea {
                            id: cardDrag
                            anchors.fill: parent
                            cursorShape: Qt.OpenHandCursor
                            drag.target: card
                            // Snap home; the model decides where the card really goes, so
                            // leaving it where it was dropped would show a stale position
                            // until the caller updates.
                            onReleased: card.Drag.drop()
                        }
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }
    }
}
