// FileDropzone — the twin of components/inputs/FileDropzone.tsx: a drop target that also
// opens a file dialog on click.
//
// QtQuick.Dialogs' FileDialog is the native picker, so unlike the Electron flavor this
// needs no IPC round trip to the host — one of the few places the QML client is simpler
// rather than merely equivalent.

import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property string title: qsTr("Drop files here")
    property string hint: qsTr("or click to choose")
    property bool multiple: true
    property var nameFilters: ["All files (*)"]
    property var files: []
    signal chosen(var files)

    implicitHeight: Theme.px(110)
    radius: Theme.radiusXl
    color: drop.containsDrag ? Theme.accentTint : "transparent"
    border.width: 1
    border.color: drop.containsDrag ? Theme.accentRing : Theme.border
    Behavior on color {
        ColorAnimation {
            duration: Theme.animFast
        }
    }
    Behavior on border.color {
        ColorAnimation {
            duration: Theme.animFast
        }
    }

    // A dashed border would be closer to the web flavor, but QML has no dash pattern on
    // a Rectangle; the accent tint on hover carries the same "this is a target" meaning.

    ColumnLayout {
        anchors.centerIn: parent
        spacing: Theme.space1

        Icon {
            Layout.alignment: Qt.AlignHCenter
            name: "upload"
            size: Theme.px(22)
            color: drop.containsDrag ? Theme.accentTextOnBg : Theme.muted
        }
        Txt {
            Layout.alignment: Qt.AlignHCenter
            text: root.files.length > 0 ? qsTr("%1 file(s) selected").arg(root.files.length) : root.title
            pixelSize: Theme.fontSm
            weight: Font.Medium
            color: Theme.text
        }
        Txt {
            Layout.alignment: Qt.AlignHCenter
            text: root.hint
            pixelSize: Theme.fontXs
            color: Theme.muted
        }
    }

    DropArea {
        id: drop
        anchors.fill: parent
        onDropped: drag => {
            if (drag.hasUrls) {
                root.files = drag.urls.map(u => String(u));
                root.chosen(root.files);
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: dialog.open()
    }

    FileDialog {
        id: dialog
        fileMode: root.multiple ? FileDialog.OpenFiles : FileDialog.OpenFile
        nameFilters: root.nameFilters
        onAccepted: {
            root.files = selectedFiles.map(u => String(u));
            root.chosen(root.files);
        }
    }
}
