// Card is the suite's container: rounded-xl, padded, one hairline border.
// The twin of Card in components/Feedback.tsx, including its optional title row
// and trailing action slot.
//
// clip is on because a rounded Rectangle does not clip its children by default in
// QML, so anything reaching the corner — a table row's hover fill, a progress bar —
// draws over the radius and the card looks square in exactly one corner.

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property string title: ""
    // Anything placed here is right-aligned in the title row, for the "action"
    // prop the React Card takes.
    default property alias content: body.data
    property alias action: actionSlot.data
    property int padding: Theme.space5
    property int spacing: Theme.space3

    color: Theme.card
    border.color: Theme.border
    border.width: 1
    radius: Theme.radiusXl
    clip: true

    implicitWidth: layout.implicitWidth + padding * 2
    implicitHeight: layout.implicitHeight + padding * 2

    ColumnLayout {
        id: layout
        anchors.fill: parent
        anchors.margins: root.padding
        spacing: root.spacing

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space2
            visible: root.title !== "" || actionSlot.children.length > 0

            Txt {
                text: root.title
                color: Theme.text
                pixelSize: Theme.fontSm
                weight: Font.DemiBold
                elide: Text.ElideRight
                // The one shrinkable child in this row — see QML-CLIENT.md. Without
                // minimumWidth 0 a long title pushes the action slot out of the card.
                Layout.fillWidth: true
                Layout.minimumWidth: 0
            }

            Item {
                id: actionSlot
                Layout.preferredWidth: childrenRect.width
                Layout.preferredHeight: childrenRect.height
            }
        }

        ColumnLayout {
            id: body
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: root.spacing
        }
    }
}
