pragma ComponentBehavior: Bound

// Accordion — the twin of components/layout/Accordion.tsx: collapsible sections with a
// rotating chevron.
//
// sections: [{ title, text }]. For arbitrary content use Collapsible directly, one per
// section — a delegate cannot take child items per row.

import QtQuick
import QtQuick.Layouts
import App

ColumnLayout {
    id: root

    property var sections: []
    // Only one section open at a time, like the web flavor's single-open default.
    property bool exclusive: true
    property int openIndex: 0

    spacing: Theme.space1

    Repeater {
        model: root.sections
        delegate: Collapsible {
            required property var modelData
            required property int index

            Layout.fillWidth: true
            title: modelData.title ?? ""
            text: modelData.text ?? ""
            expanded: root.exclusive ? root.openIndex === index : expanded
            onToggled: root.openIndex = (root.openIndex === index && root.exclusive) ? -1 : index
        }
    }
}
