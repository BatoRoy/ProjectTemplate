// AutoGrid — the twin of the AutoGrid in components/layout/Responsive.tsx: as many
// columns of at least `minWidth` as the available width allows.
//
// The web flavor gets this from CSS `repeat(auto-fill, minmax(…))`. QML's GridLayout
// wants a column count, so the count is computed — which is the whole component.

import QtQuick
import QtQuick.Layouts
import App

GridLayout {
    id: root

    property int minWidth: Theme.px(200)

    columns: Math.max(1, Math.floor((width + columnSpacing) / (minWidth + columnSpacing)))
    columnSpacing: Theme.space3
    rowSpacing: Theme.space3
}
