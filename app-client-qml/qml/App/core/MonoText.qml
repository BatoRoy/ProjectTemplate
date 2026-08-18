// MonoText is the twin of the web flavor's `.mono-text` class: JetBrains Mono,
// small, and **always selectable** regardless of the Text selection setting.
//
// That exception is deliberate and matches index.css. The things rendered in mono
// are versions, hex colours, URLs, ports and paths — values whose whole purpose is
// to be copied somewhere else. Making them depend on a preference would mean the
// user has to go and find a setting before they can paste a server URL into a
// terminal.

import QtQuick
import App

TextEdit {
    color: Theme.subtext
    font.family: Theme.fontMono
    font.pixelSize: Theme.fontXs
    readOnly: true
    selectByMouse: true
    selectionColor: Theme.accentTintHi
    selectedTextColor: color
    verticalAlignment: Text.AlignVCenter
    renderType: Text.NativeRendering
    activeFocusOnPress: false
    cursorVisible: false
}
