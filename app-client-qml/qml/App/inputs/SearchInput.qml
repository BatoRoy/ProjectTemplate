pragma ComponentBehavior: Bound

// SearchInput — the twin of components/inputs/SearchInput.tsx: a search field with a
// suggestions dropdown, debounced.
//
// Debounced because the React version is: a suggestion callback that runs per keystroke
// is a request per keystroke once anything real is wired to it.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property string text: ""
    property var suggestions: []
    property string placeholder: qsTr("Search…")
    property int debounce: 200
    property int maxResults: 8
    signal searched(string query)
    signal picked(string value)

    implicitHeight: field.implicitHeight
    implicitWidth: Theme.px(240)

    readonly property var matches: {
        var q = text.trim().toLowerCase();
        if (q === "")
            return [];
        return suggestions.filter(s => String(s).toLowerCase().indexOf(q) !== -1).slice(0, maxResults);
    }

    TextField {
        id: field
        anchors.fill: parent
        leftIcon: "search"
        clearable: true
        placeholder: root.placeholder
        text: root.text
        onEdited: {
            root.text = text;
            debounceTimer.restart();
        }
    }

    Timer {
        id: debounceTimer
        interval: root.debounce
        onTriggered: root.searched(root.text)
    }

    // A plain positioned panel rather than a Popup: suggestions belong to the field and
    // should scroll away with it, not float above the whole window.
    Rectangle {
        id: popup
        anchors.top: parent.bottom
        anchors.topMargin: Theme.space1
        width: parent.width
        height: Math.min(list.contentHeight, Theme.px(200)) + 2
        // Bound, not toggled imperatively: assigning popup.visible in onEdited would
        // break this binding on the first keystroke and the panel would never hide.
        visible: field.activeFocus && root.matches.length > 0
        color: Theme.card
        border.color: Theme.border
        border.width: 1
        radius: Theme.radius
        clip: true
        z: 10

        ListView {
            id: list
            anchors.fill: parent
            anchors.margins: 1
            model: root.matches
            clip: true

            delegate: Rectangle {
                id: item
                required property var modelData
                width: list.width
                height: Theme.rowHeight
                color: itemMouse.containsMouse ? Theme.alpha(Theme.border, 0.5) : "transparent"

                Txt {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.space3
                    text: item.modelData
                    pixelSize: Theme.fontSm
                    color: Theme.text
                }

                MouseArea {
                    id: itemMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.text = item.modelData;
                        field.text = item.modelData;
                        root.picked(item.modelData);
                    }
                }
            }
        }
    }
}
