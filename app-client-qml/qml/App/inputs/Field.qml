// Field — the twin of components/inputs/Field.tsx: the label / hint / error scaffold every
// other input wraps itself in, so the vertical rhythm and the error treatment are defined
// once.
//
// Used as a container:
//
//     Field { label: "Name"; hint: "As it appears on the invoice"
//         TextField { Layout.fillWidth: true }
//     }

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property string label: ""
    property string hint: ""
    property string error: ""
    property bool required: false

    spacing: Theme.space1

    RowLayout {
        Layout.fillWidth: true
        visible: root.label !== ""
        spacing: Theme.px(3)

        Txt {
            text: root.label
            pixelSize: Theme.fontXs
            weight: Font.Medium
            color: Theme.subtext
        }
        Txt {
            visible: root.required
            text: "*"
            pixelSize: Theme.fontXs
            color: Theme.red
        }
        Item {
            Layout.fillWidth: true
        }
    }

    // The control itself goes here, between the label and the message.
    ColumnLayout {
        id: control
        Layout.fillWidth: true
        spacing: Theme.space1
    }

    // Error replaces the hint rather than stacking with it: two lines of guidance where
    // one of them is now wrong is worse than one line that is right.
    Txt {
        Layout.fillWidth: true
        visible: root.error !== "" || root.hint !== ""
        text: root.error !== "" ? root.error : root.hint
        pixelSize: Theme.fontXs
        color: root.error !== "" ? Theme.readableOn(Theme.red, Theme.card) : Theme.muted
        wrapMode: Text.WordWrap
    }

    // Declared last so `default` collects children into the control slot, not the root.
    default property alias content: control.data
}
