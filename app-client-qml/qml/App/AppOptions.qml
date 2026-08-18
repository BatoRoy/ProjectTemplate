// AppOptions is the settings modal, ported section for section from
// components/AppOptionsModal.tsx: Server, Theme, Accent, Scale, Content width,
// Text selection.
//
// BatoAI's version dropped Content width ("responsive-web only") and Text selection
// ("QML text isn't selectable"). Both are here instead, because both are solvable:
// Content width is one max-width on the page container, and Text selection is what
// Txt and MonoText exist to implement. Dropping options is how two flavors of the
// same template stop being the same template.
//
// The custom-colour picker is the one place this diverges in *mechanism*: the web
// flavor opens the browser's native <input type="color">. There is no such control
// in QtQuick.Dialogs, so this offers a hex field instead — same capability, and it
// also lets you paste a brand colour, which the swatch picker never could.

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import App

Modal {
    id: root
    title: qsTr("App Options")

    // Server address — keep for daemon-style backends (a fixed PORTS.md port,
    // possibly on another machine); remove for apps with only a bundled
    // session-bound server.
    ServerUrlCard {
        Layout.fillWidth: true
    }

    // ── Theme ────────────────────────────────────────────────────────────────
    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space3

        SectionLabel {
            text: qsTr("Theme")
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space3

            Repeater {
                model: Theme.presets
                ThemePreview {
                    required property var modelData
                    presetId: modelData.id
                    label: modelData.label
                    active: Theme.preset === modelData.id
                    onPicked: Theme.preset = modelData.id
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                }
            }
        }
    }

    // ── Accent ───────────────────────────────────────────────────────────────
    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space3

        SectionLabel {
            text: qsTr("Accent")
        }

        // A plain Row, not a Flow: Flow positions children against widths that
        // settle a frame later, so a row of swatches overflows instead of wrapping.
        // Nine fixed-width circles fit the modal at every scale, so there is
        // nothing to wrap. See QML-CLIENT.md.
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(10)

            // The app's own accent, marked with a star — always first.
            Swatch {
                hex: Brand.accentHex
                selected: Theme.accentHex.toLowerCase() === Brand.accentHex.toLowerCase()
                glyph: selected ? "check" : "star"
                onPicked: Theme.accentHex = Brand.accentHex
            }

            Repeater {
                // Deduped against the brand default, so the same colour never
                // appears twice in the row.
                model: Theme.accents.filter(a => a.hex.toLowerCase() !== Brand.accentHex.toLowerCase())
                Swatch {
                    required property var modelData
                    hex: modelData.hex
                    selected: Theme.accentHex.toLowerCase() === modelData.hex.toLowerCase()
                    glyph: selected ? "check" : ""
                    onPicked: Theme.accentHex = modelData.hex
                }
            }

            Item {
                Layout.fillWidth: true
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space2

            Txt {
                text: qsTr("Custom")
                color: Theme.muted
                pixelSize: Theme.fontXs
            }

            Input {
                id: hexField
                mono: true
                text: Theme.accentHex
                placeholder: "#8b5cf6"
                Layout.preferredWidth: Theme.px(100)
                // Accept only once it is a complete colour, so the UI does not
                // repaint through "#8b", "#8b5", "#8b5c" as you type.
                invalid: text.length > 0 && !root.isHex(text)
                onAccepted: if (root.isHex(text))
                    Theme.accentHex = text.toLowerCase()
            }

            Button {
                variant: "ghost"
                text: qsTr("Apply")
                enabled: hexField.text !== Theme.accentHex && root.isHex(hexField.text)
                onClicked: Theme.accentHex = hexField.text.toLowerCase()
            }

            Item {
                Layout.fillWidth: true
            }
        }
    }

    // ── Scale ────────────────────────────────────────────────────────────────
    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space3

        SectionLabel {
            text: qsTr("Scale")
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(6)

            Repeater {
                model: Theme.scales
                OptionChip {
                    required property var modelData
                    label: modelData + "%"
                    active: Theme.scalePercent === modelData
                    onClicked: Theme.scalePercent = modelData
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                }
            }
        }

        Txt {
            Layout.fillWidth: true
            text: qsTr("Scales all UI elements. Takes effect immediately.")
            color: Theme.muted
            pixelSize: Theme.fontXs
            wrapMode: Text.WordWrap
        }
    }

    // ── Content width ────────────────────────────────────────────────────────
    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space3

        SectionLabel {
            text: qsTr("Content width")
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(6)

            OptionChip {
                label: qsTr("Comfortable")
                active: !Theme.wide
                onClicked: Theme.wide = false
                Layout.fillWidth: true
                Layout.minimumWidth: 0
            }
            OptionChip {
                label: qsTr("Full width")
                active: Theme.wide
                onClicked: Theme.wide = true
                Layout.fillWidth: true
                Layout.minimumWidth: 0
            }
        }

        Txt {
            Layout.fillWidth: true
            text: qsTr("Full width lets pages use the whole window.")
            color: Theme.muted
            pixelSize: Theme.fontXs
            wrapMode: Text.WordWrap
        }
    }

    // ── Text selection ───────────────────────────────────────────────────────
    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space3

        SectionLabel {
            text: qsTr("Text selection")
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.px(6)

            OptionChip {
                label: qsTr("Off")
                active: !Theme.textSelect
                onClicked: Theme.textSelect = false
                Layout.fillWidth: true
                Layout.minimumWidth: 0
            }
            OptionChip {
                label: qsTr("On")
                active: Theme.textSelect
                onClicked: Theme.textSelect = true
                Layout.fillWidth: true
                Layout.minimumWidth: 0
            }
        }

        Txt {
            Layout.fillWidth: true
            text: qsTr("Lets you select and copy UI text. Off gives a more native desktop feel. Values in monospace are always selectable.")
            color: Theme.muted
            pixelSize: Theme.fontXs
            wrapMode: Text.WordWrap
        }
    }

    function isHex(s) {
        return /^#[0-9a-fA-F]{6}$/.test(s);
    }
}
