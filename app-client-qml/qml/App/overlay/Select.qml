// Select — the twin of the Select exported from components/Dropdown.tsx: a value picker
// showing the active label, with a checkmark against the chosen row.
//
// Built on the same MenuList as Dropdown and ContextMenu rather than on Controls'
// ComboBox, whose popup and indicator carry the active style's own geometry.
//
// options: [{ value, label }]

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Item {
    id: root

    property var options: []
    property var value: undefined
    property string placeholder: qsTr("Select…")
    signal picked(var value)

    readonly property string currentLabel: {
        for (var i = 0; i < options.length; ++i)
            if (options[i].value === value)
                return options[i].label;
        return placeholder;
    }

    implicitHeight: Theme.controlHeight
    implicitWidth: Theme.px(180)

    Rectangle {
        id: trigger
        anchors.fill: parent
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: popup.opened ? Theme.accentRing : triggerMouse.containsMouse ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.space3
            anchors.rightMargin: Theme.space2
            spacing: Theme.space2

            Txt {
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                text: root.currentLabel
                pixelSize: Theme.fontSm
                elide: Text.ElideRight
                color: root.value === undefined ? Theme.muted : Theme.text
            }

            Icon {
                name: "chevron-down"
                size: Theme.px(14)
                color: Theme.muted
                rotation: popup.opened ? 180 : 0
                Behavior on rotation {
                    NumberAnimation {
                        duration: Theme.animFast
                    }
                }
            }
        }

        MouseArea {
            id: triggerMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.opened ? popup.close() : popup.open()
        }
    }

    Popover {
        id: popup
        padding: 0
        width: root.width

        MenuList {
            width: parent.width
            // A checkmark against the current row, which is the whole difference in
            // presentation between a Select and a Dropdown.
            items: root.options.map(o => ({
                        label: o.label,
                        checked: o.value === root.value,
                        value: o.value
                    }))
            onTriggered: (index, item) => {
                root.value = item.value;
                root.picked(item.value);
                popup.close();
            }
        }
    }
}
