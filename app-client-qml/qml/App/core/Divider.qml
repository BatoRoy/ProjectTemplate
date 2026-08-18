// Divider — one of the structural primitives from components/layout/Primitives.tsx.
//
// QML has no per-side border, so this is what every visual separation in the template
// uses. A 1px Rectangle, not a scaled one: hairlines are the one thing that should NOT
// grow with UI scale, or they stop reading as hairlines.

import QtQuick
import App

Rectangle {
    property bool vertical: false
    implicitHeight: vertical ? Theme.px(16) : 1
    implicitWidth: vertical ? 1 : Theme.px(16)
    color: Theme.border
}
