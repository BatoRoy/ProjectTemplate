// Toast + ToastQueue — the twin of components/Toast.tsx and the useToast hook together.
//
// The React version splits state (a hook) from presentation (a container). QML has no
// hooks, so this is one component that owns its own stack and exposes the same verbs:
//
//     Toast { id: toasts }
//     toasts.show("Saved", "success")
//
// Declared once in Main.qml, like ToastContainer is mounted once in App.tsx.

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    // Anchored by the caller; sized to its content so it never intercepts clicks
    // outside the toasts themselves.
    implicitWidth: column.implicitWidth
    implicitHeight: column.implicitHeight

    property int duration: 4000
    property ListModel model: ListModel {}
    property int nextId: 0

    function show(text, tone) {
        model.append({
            toastId: root.nextId++,
            text: text,
            tone: tone || "neutral"
        });
    }
    function success(text) {
        show(text, "success");
    }
    function error(text) {
        show(text, "error");
    }
    function dismiss(index) {
        if (index >= 0 && index < model.count)
            model.remove(index);
    }

    ColumnLayout {
        id: column
        spacing: Theme.space2

        Repeater {
            model: root.model
            delegate: Rectangle {
                id: toast
                required property int index
                required property string text
                required property string tone

                readonly property color toneColor: {
                    switch (tone) {
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

                Layout.alignment: Qt.AlignRight
                implicitWidth: Math.min(toastRow.implicitWidth + Theme.space4 * 2, Theme.px(380))
                implicitHeight: toastRow.implicitHeight + Theme.space3 * 2
                radius: Theme.radius
                color: Theme.card
                border.width: 1
                border.color: Theme.border

                // The tone is a leading bar rather than a fill, so the text stays on the
                // card colour and keeps its normal contrast.
                Rectangle {
                    width: Theme.px(3)
                    height: parent.height
                    color: toast.toneColor
                    radius: width / 2
                }

                RowLayout {
                    id: toastRow
                    anchors.fill: parent
                    anchors.margins: Theme.space3
                    anchors.leftMargin: Theme.space4
                    spacing: Theme.space2

                    Txt {
                        Layout.fillWidth: true
                        Layout.minimumWidth: 0
                        text: toast.text
                        pixelSize: Theme.fontSm
                        color: Theme.text
                        wrapMode: Text.WordWrap
                    }

                    Icon {
                        name: "x"
                        size: Theme.px(12)
                        color: dismissMouse.containsMouse ? Theme.text : Theme.muted
                        MouseArea {
                            id: dismissMouse
                            anchors.fill: parent
                            anchors.margins: -Theme.space2
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.dismiss(toast.index)
                        }
                    }
                }

                // Auto-dismiss. Paused on hover, because a toast that vanishes while you
                // are reading it is worse than one that lingers.
                Timer {
                    interval: root.duration
                    running: !hoverArea.containsMouse
                    onTriggered: root.dismiss(toast.index)
                }
                MouseArea {
                    id: hoverArea
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                }

                opacity: 0
                Component.onCompleted: opacity = 1
                Behavior on opacity {
                    NumberAnimation {
                        duration: Theme.animFast
                    }
                }
            }
        }
    }
}
