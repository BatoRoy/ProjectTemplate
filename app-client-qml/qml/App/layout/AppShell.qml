// AppShell — the twin of components/layout/AppShell.tsx: a generic page scaffold with an
// optional header, left rail, footer and a scrolling content region.
//
// Main.qml does not use this — it lays out its own shell, because it also owns the error
// banner and the page stack. This exists for the same reason the React one does: a second
// window, a settings surface or a detached panel wants the same frame without copying it.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property Item header: null
    property Item sidebar: null
    property Item footer: null
    default property alias content: contentSlot.data

    onHeaderChanged: if (header)
        header.parent = headerSlot
    onSidebarChanged: if (sidebar)
        sidebar.parent = sidebarSlot
    onFooterChanged: if (footer)
        footer.parent = footerSlot

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Item {
            id: sidebarSlot
            visible: root.sidebar !== null
            Layout.fillHeight: true
            Layout.preferredWidth: root.sidebar ? root.sidebar.implicitWidth : 0
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumWidth: 0
            spacing: 0

            Item {
                id: headerSlot
                visible: root.header !== null
                Layout.fillWidth: true
                Layout.preferredHeight: root.header ? root.header.implicitHeight : 0

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: 1
                    color: Theme.border
                }
            }

            Item {
                id: contentSlot
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
            }

            Item {
                id: footerSlot
                visible: root.footer !== null
                Layout.fillWidth: true
                Layout.preferredHeight: root.footer ? root.footer.implicitHeight : 0

                Rectangle {
                    anchors.top: parent.top
                    width: parent.width
                    height: 1
                    color: Theme.border
                }
            }
        }
    }
}
