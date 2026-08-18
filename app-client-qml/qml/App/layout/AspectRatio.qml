// AspectRatio — one of the structural primitives from components/layout/Primitives.tsx.
//
// Most of that file's exports (Stack, HStack, VStack, Grid, Container, Center, Spacer)
// have direct QML equivalents already — ColumnLayout, RowLayout, GridLayout, anchors.fill,
// anchors.centerIn, and `Item { Layout.fillWidth: true }` — so wrapping them would add
// names without adding capability. This one and AutoGrid are the two QML genuinely lacks.
// PARITY.md records the mapping.

import QtQuick
import App

Item {
    id: root

    property real ratio: 16 / 9
    default property alias content: inner.data

    implicitHeight: width / ratio

    Item {
        id: inner
        anchors.fill: parent
    }
}
