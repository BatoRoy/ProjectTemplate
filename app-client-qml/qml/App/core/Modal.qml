// Modal is the twin of components/Modal.tsx: a dimmed backdrop, a centred card with
// a titled header and a close affordance.
//
// Built on Controls' Dialog for the modal grab and Esc handling, but with both
// `background` and `contentItem` fully overridden. That matters: a Dialog left with
// its default background picks up the active Controls style, which is how a QML app
// ends up looking like two apps at once. Overriding both means the style never
// shows through and the template is not dependent on which style is installed.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Dialog {
    id: root

    property int contentPadding: Theme.space5
    default property alias body: bodyColumn.data

    modal: true
    // anchors.centerIn on Overlay.overlay is what keeps the dialog centred when the
    // window is resized; centring on the parent leaves it where the window used to be.
    anchors.centerIn: Overlay.overlay
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    // max-w-sm in the web flavor, and never wider than the window it sits in.
    width: Math.min(parent ? parent.width - Theme.space6 * 2 : Theme.px(420), Theme.px(420))

    // …and never taller than it either. The web flavor gets this free: an overlong
    // modal scrolls with the page. A Dialog does not — it simply extends past the
    // window edge, and the bottom of App Options becomes unreachable at the window's
    // own minimum height. Capping the height and scrolling the body is the fix.
    height: Math.min(implicitHeight, parent ? parent.height - Theme.space6 * 2 : Theme.px(720))

    enter: Transition {
        // The web flavor's animate-scale-in.
        NumberAnimation { property: "scale"; from: 0.96; to: 1.0; duration: Theme.animFast; easing.type: Easing.OutCubic }
        NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: Theme.animFast }
    }

    background: Rectangle {
        color: Theme.card
        border.color: Theme.border
        border.width: 1
        radius: Theme.radiusXl
    }

    Overlay.modal: Rectangle {
        color: Qt.rgba(0, 0, 0, 0.6)
    }

    contentItem: ColumnLayout {
        spacing: 0

        // Header
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Theme.px(52)
            color: "transparent"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.space5
                anchors.rightMargin: Theme.space5
                spacing: Theme.space2

                Txt {
                    text: root.title
                    color: Theme.text
                    pixelSize: Theme.fontSm
                    weight: Font.DemiBold
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                }

                Icon {
                    name: "x"
                    size: Theme.px(16)
                    color: closeMouse.containsMouse ? Theme.text : Theme.muted
                    MouseArea {
                        id: closeMouse
                        anchors.fill: parent
                        // A 16px glyph is a 16px hit target, which is a miss more
                        // often than a hit. Negative margins widen the target
                        // without moving the icon.
                        anchors.margins: -Theme.space2
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.close()
                    }
                }
            }

            // QML has no per-side border, so a divider is a 1px child.
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        // Body. Scrolls only when it has to: contentHeight below the viewport leaves
        // the Flickable inert, so a short modal behaves exactly as before.
        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            implicitHeight: bodyColumn.implicitHeight + root.contentPadding * 2
            contentWidth: width
            contentHeight: bodyColumn.implicitHeight + root.contentPadding * 2
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }

            ColumnLayout {
                id: bodyColumn
                x: root.contentPadding
                y: root.contentPadding
                width: parent.width - root.contentPadding * 2
                spacing: Theme.space6
            }
        }
    }
}
