pragma ComponentBehavior: Bound

// ShowcasePage is the living gallery — the twin of components/ShowcasePage.tsx and the
// parity harness for the whole kit. Every component the template ships appears here, so
// a visual diff against the Electron flavor's Examples page is the acceptance test for
// "the two look the same".
//
// It doubles as a smoke test: `make check-qml` loads this page, so a component that
// cannot be instantiated, or that overflows its card between 560 and 1200px, fails the
// build rather than waiting to be noticed.
//
// Meant to be deleted along with its entry in Sidebar.nav when you start a real app.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Flickable {
    id: root

    contentWidth: width
    contentHeight: column.implicitHeight + Theme.space6 * 2
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: ScrollBar {}

    property string section: "inputs"

    ColumnLayout {
        id: column
        x: Math.max(Theme.space6, (root.width - width) / 2)
        y: Theme.space6
        width: Theme.wide ? root.width - Theme.space6 * 2 : Math.min(root.width - Theme.space6 * 2, Theme.maxW3xl)
        spacing: Theme.space6

        Txt {
            text: qsTr("Examples")
            color: Theme.text
            pixelSize: Theme.fontXl
            weight: Font.DemiBold
        }

        SegmentedControl {
            Layout.fillWidth: true
            value: root.section
            options: [
                {
                    value: "inputs",
                    label: qsTr("Inputs")
                },
                {
                    value: "feedback",
                    label: qsTr("Feedback")
                },
                {
                    value: "layout",
                    label: qsTr("Layout")
                },
                {
                    value: "data",
                    label: qsTr("Data")
                },
                {
                    value: "overlays",
                    label: qsTr("Overlays")
                },
                {
                    value: "icons",
                    label: qsTr("Icons")
                }
            ]
            onPicked: v => root.section = v
        }

        // ── Inputs ───────────────────────────────────────────────────────────
        Card {
            Layout.fillWidth: true
            visible: root.section === "inputs"
            title: qsTr("Text")

            TextField {
                Layout.fillWidth: true
                label: qsTr("Name")
                hint: qsTr("As it appears on the invoice")
                placeholder: qsTr("Ada Lovelace")
                clearable: true
                leftIcon: "search"
            }

            TextField {
                Layout.fillWidth: true
                label: qsTr("Password")
                password: true
                placeholder: "••••••••"
            }

            TextField {
                Layout.fillWidth: true
                label: qsTr("Endpoint")
                error: qsTr("Must start with http://")
                text: "ftp://example"
                mono: true
            }

            TextArea {
                Layout.fillWidth: true
                label: qsTr("Notes")
                placeholder: qsTr("Anything worth remembering…")
                autosize: true
                maxLength: 280
                showCount: true
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "inputs"
            title: qsTr("Values")

            NumberInput {
                Layout.fillWidth: true
                label: qsTr("Retries")
                value: 3
                min: 0
                max: 10
            }

            Field {
                Layout.fillWidth: true
                label: qsTr("Threshold")
                Slider {
                    Layout.fillWidth: true
                    value: 40
                    marks: true
                    step: 10
                }
            }

            TagsInput {
                Layout.fillWidth: true
                label: qsTr("Tags")
                value: ["qml", "go"]
            }

            Field {
                Layout.fillWidth: true
                label: qsTr("Environment")
                Select {
                    Layout.fillWidth: true
                    value: "prod"
                    options: [
                        {
                            value: "dev",
                            label: qsTr("Development")
                        },
                        {
                            value: "staging",
                            label: qsTr("Staging")
                        },
                        {
                            value: "prod",
                            label: qsTr("Production")
                        }
                    ]
                }
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "inputs"
            title: qsTr("Toggles")

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space4
                Switch {
                    checked: true
                    label: qsTr("Enabled")
                }
                Checkbox {
                    checked: true
                    label: qsTr("Checked")
                }
                Checkbox {
                    indeterminate: true
                    label: qsTr("Partial")
                }
                Checkbox {
                    label: qsTr("Unchecked")
                }
            }

            RadioGroup {
                Layout.fillWidth: true
                value: "b"
                options: [
                    {
                        value: "a",
                        label: qsTr("First option")
                    },
                    {
                        value: "b",
                        label: qsTr("Second option")
                    }
                ]
            }

            Tabs {
                value: "one"
                tabs: [
                    {
                        id: "one",
                        label: qsTr("One")
                    },
                    {
                        id: "two",
                        label: qsTr("Two")
                    },
                    {
                        id: "three",
                        label: qsTr("Three")
                    }
                ]
            }
        }

        // ── Feedback ─────────────────────────────────────────────────────────
        Card {
            Layout.fillWidth: true
            visible: root.section === "feedback"
            title: qsTr("Status")

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space2
                Badge {
                    tone: "success"
                    text: qsTr("Success")
                }
                Badge {
                    tone: "error"
                    text: qsTr("Error")
                }
                Badge {
                    tone: "warning"
                    text: qsTr("Warning")
                }
                Badge {
                    tone: "info"
                    text: qsTr("Info")
                }
                Badge {
                    tone: "neutral"
                    text: qsTr("Neutral")
                }
            }

            Alert {
                Layout.fillWidth: true
                tone: "info"
                title: qsTr("Heads up")
                text: qsTr("Alerts carry their tone at 10% with a 30% border, so the text keeps its normal contrast.")
            }
            Alert {
                Layout.fillWidth: true
                tone: "error"
                title: qsTr("Something failed")
                text: qsTr("And this one can be dismissed.")
                closable: true
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space4
                Spinner {}
                CircularProgress {
                    value: 62
                    showLabel: true
                }
                ColumnLayout {
                    Layout.fillWidth: true
                    // Without a zero minimum this column refuses to shrink below its
                    // children's implicit width and the row overflows the card.
                    Layout.minimumWidth: 0
                    spacing: Theme.space2
                    Progress {
                        Layout.fillWidth: true
                        value: 62
                    }
                    Progress {
                        Layout.fillWidth: true
                        indeterminate: true
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: Theme.space2
                Skeleton {
                    Layout.fillWidth: true
                }
                Skeleton {
                    Layout.preferredWidth: parent.width * 0.6
                }
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "feedback"
            title: qsTr("Empty")

            EmptyState {
                Layout.fillWidth: true
                icon: "layout-grid"
                title: qsTr("Nothing here yet")
                subtitle: qsTr("EmptyState takes a glyph, a title and one line explaining what would fill the space.")
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "inputs"
            title: qsTr("Pickers")

            SearchInput {
                Layout.fillWidth: true
                suggestions: ["batoai", "batomusic", "batogit", "batoscribe", "bato-auth"]
                placeholder: qsTr("Search services…")
            }

            Field {
                Layout.fillWidth: true
                label: qsTr("Range")
                RangeSlider {
                    Layout.fillWidth: true
                    from: 20
                    to: 70
                }
            }

            // Flow, not RowLayout: a date picker and three time segments side by side are
            // wider than the content column at the window's minimum size, and neither has
            // a sensible way to shrink further.
            Flow {
                Layout.fillWidth: true
                spacing: Theme.space3
                DatePicker {}
                TimePicker {}
            }

            FileDropzone {
                Layout.fillWidth: true
            }

            ColorPicker {
                Layout.fillWidth: true
                value: Theme.accentHex
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "inputs"
            title: qsTr("Structured")

            Field {
                Layout.fillWidth: true
                label: qsTr("Service")
                Combobox {
                    Layout.fillWidth: true
                    value: "batoai"
                    options: [
                        {
                            "value": "batoai",
                            "label": "batoai"
                        },
                        {
                            "value": "batomusic",
                            "label": "batomusic"
                        },
                        {
                            "value": "batogit",
                            "label": "batogit"
                        }
                    ]
                }
            }

            Field {
                Layout.fillWidth: true
                label: qsTr("Tags")
                MultiSelect {
                    Layout.fillWidth: true
                    values: ["go"]
                    options: [
                        {
                            "value": "go",
                            "label": "Go"
                        },
                        {
                            "value": "qml",
                            "label": "QML"
                        },
                        {
                            "value": "ts",
                            "label": "TypeScript"
                        }
                    ]
                }
            }

            MaskedInput {
                Layout.fillWidth: true
                label: qsTr("Reference")
                mask: "AA-####-AA"
            }

            CurrencyInput {
                Layout.fillWidth: true
                label: qsTr("Amount")
                value: 1249.5
            }

            Field {
                Layout.fillWidth: true
                label: qsTr("Verification code")
                OtpInput {}
            }

            Field {
                Layout.fillWidth: true
                label: qsTr("Volume")
                VolumeControl {
                    Layout.fillWidth: true
                }
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "inputs"
            title: qsTr("Code")

            CodeEditor {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(150)
                language: "go"
                text: "// A monospace editor with line numbers.\n// Highlighting is regex-based, not a grammar.\nfunc main() {\n\tport := 42117\n\tfmt.Println(\"listening on\", port)\n}\n"
            }
        }

        // ── Layout ───────────────────────────────────────────────────────────
        Card {
            Layout.fillWidth: true
            visible: root.section === "layout"
            title: qsTr("Structure")

            Breadcrumbs {
                Layout.fillWidth: true
                items: [
                    {
                        "label": qsTr("Home")
                    },
                    {
                        "label": qsTr("Projects")
                    },
                    {
                        "label": qsTr("Current")
                    }
                ]
            }

            Stepper {
                Layout.fillWidth: true
                current: 1
                steps: [
                    {
                        "label": qsTr("Configure")
                    },
                    {
                        "label": qsTr("Review")
                    },
                    {
                        "label": qsTr("Deploy")
                    }
                ]
            }

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space2
                Avatar {
                    name: "Ada Lovelace"
                }
                Avatar {
                    name: "Grace Hopper"
                }
                Avatar {
                    name: "Alan Turing"
                }
                Avatar {
                    name: "Katherine Johnson"
                }
            }

            Divider {
                Layout.fillWidth: true
            }

            Accordion {
                Layout.fillWidth: true
                sections: [
                    {
                        "title": qsTr("What is this?"),
                        "text": qsTr("An Accordion opens one section at a time and animates its height, so the page does not jump.")
                    },
                    {
                        "title": qsTr("And this?"),
                        "text": qsTr("Each row is a Collapsible, usable on its own.")
                    }
                ]
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "layout"
            title: qsTr("Panels")

            Txt {
                Layout.fillWidth: true
                text: qsTr("Drag the divider. The split is a fraction, so resizing the window keeps the proportion.")
                color: Theme.subtext
                pixelSize: Theme.fontXs
                wrapMode: Text.WordWrap
            }

            ResizablePanels {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(120)
                first: Rectangle {
                    color: Theme.alpha(Theme.border, 0.4)
                    radius: Theme.radiusSm
                    Txt {
                        anchors.centerIn: parent
                        text: qsTr("First")
                        color: Theme.muted
                        pixelSize: Theme.fontXs
                    }
                }
                second: Rectangle {
                    color: Theme.alpha(Theme.border, 0.4)
                    radius: Theme.radiusSm
                    Txt {
                        anchors.centerIn: parent
                        text: qsTr("Second")
                        color: Theme.muted
                        pixelSize: Theme.fontXs
                    }
                }
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "layout"
            title: qsTr("Tabs and grids")

            EditorTabs {
                Layout.fillWidth: true
                current: "main"
                tabs: [
                    {
                        "id": "main",
                        "label": "main.go"
                    },
                    {
                        "id": "theme",
                        "label": "Theme.qml",
                        "dirty": true
                    },
                    {
                        "id": "readme",
                        "label": "README.md"
                    }
                ]
            }

            AutoGrid {
                Layout.fillWidth: true
                minWidth: Theme.px(120)

                Repeater {
                    model: 6
                    delegate: Rectangle {
                        required property int index
                        Layout.fillWidth: true
                        implicitHeight: Theme.px(40)
                        radius: Theme.radiusSm
                        color: Theme.alpha(Theme.border, 0.4)
                        Txt {
                            anchors.centerIn: parent
                            text: qsTr("Cell %1").arg(parent.index + 1)
                            pixelSize: Theme.fontXs
                            color: Theme.muted
                        }
                    }
                }
            }
        }

        // ── Data ─────────────────────────────────────────────────────────────
        Card {
            Layout.fillWidth: true
            visible: root.section === "data"
            title: qsTr("Table")

            DataTable {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(200)
                sortKey: "name"
                columns: [
                    {
                        "key": "name",
                        "label": qsTr("Name"),
                        "width": 2
                    },
                    {
                        "key": "port",
                        "label": qsTr("Port"),
                        "mono": true
                    },
                    {
                        "key": "status",
                        "label": qsTr("Status"),
                        "align": "right"
                    }
                ]
                rows: [
                    {
                        "name": "bato-auth",
                        "port": 42111,
                        "status": "running"
                    },
                    {
                        "name": "batoai-server",
                        "port": 42116,
                        "status": "running"
                    },
                    {
                        "name": "batomusic-server",
                        "port": 42101,
                        "status": "stopped"
                    }
                ]
            }

            Pagination {
                Layout.fillWidth: true
                page: 4
                pages: 20
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "data"
            title: qsTr("Charts")

            Txt {
                Layout.fillWidth: true
                text: qsTr("Drawn with QtQuick.Shapes, not QtCharts — so a chart costs no extra runtime dependency.")
                color: Theme.subtext
                pixelSize: Theme.fontXs
                wrapMode: Text.WordWrap
            }

            LineChart {
                Layout.fillWidth: true
                area: true
                points: [4, 9, 6, 12, 8, 15, 11, 18, 14, 21]
            }

            BarChart {
                Layout.fillWidth: true
                points: [
                    {
                        "value": 12,
                        "label": "Mon"
                    },
                    {
                        "value": 19,
                        "label": "Tue"
                    },
                    {
                        "value": 7,
                        "label": "Wed"
                    },
                    {
                        "value": 22,
                        "label": "Thu"
                    },
                    {
                        "value": 15,
                        "label": "Fri"
                    }
                ]
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space3
                Txt {
                    text: qsTr("Inline trend")
                    pixelSize: Theme.fontXs
                    color: Theme.muted
                }
                Sparkline {
                    points: [3, 5, 4, 8, 6, 9, 7, 11]
                }
                Item {
                    Layout.fillWidth: true
                }
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "data"
            title: qsTr("Sequences")

            Timeline {
                Layout.fillWidth: true
                items: [
                    {
                        "title": qsTr("Published"),
                        "time": "14:02",
                        "text": qsTr("v0.3.0 pushed to the registry."),
                        "tone": "success"
                    },
                    {
                        "title": qsTr("Build failed"),
                        "time": "13:47",
                        "text": qsTr("qmllint reported an unqualified access."),
                        "tone": "error"
                    },
                    {
                        "title": qsTr("Started"),
                        "time": "13:40"
                    }
                ]
            }

            Divider {
                Layout.fillWidth: true
            }

            Txt {
                Layout.fillWidth: true
                text: qsTr("Drag a row by its grip to reorder.")
                color: Theme.subtext
                pixelSize: Theme.fontXs
            }

            SortableList {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(140)
                items: [qsTr("First item"), qsTr("Second item"), qsTr("Third item")]
            }
        }

        Card {
            Layout.fillWidth: true
            visible: root.section === "data"
            title: qsTr("Board")

            KanbanBoard {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(180)
                columns: [
                    {
                        "id": "todo",
                        "title": qsTr("To do"),
                        "cards": [
                            {
                                "id": "a",
                                "title": qsTr("Port the date family")
                            }
                        ]
                    },
                    {
                        "id": "doing",
                        "title": qsTr("Doing"),
                        "cards": [
                            {
                                "id": "b",
                                "title": qsTr("Widget kit")
                            }
                        ]
                    },
                    {
                        "id": "done",
                        "title": qsTr("Done"),
                        "cards": [
                            {
                                "id": "c",
                                "title": qsTr("Publish pipeline")
                            }
                        ]
                    }
                ]
            }
        }

        // ── Overlays ─────────────────────────────────────────────────────────
        Card {
            Layout.fillWidth: true
            visible: root.section === "overlays"
            title: qsTr("Overlays")

            Txt {
                Layout.fillWidth: true
                text: qsTr("Right-click this card for a context menu. Toasts stack from the bottom right.")
                color: Theme.subtext
                pixelSize: Theme.fontXs
                wrapMode: Text.WordWrap
            }

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space2

                Button {
                    text: qsTr("Toast")
                    onClicked: toasts.success(qsTr("That worked."))
                }
                Button {
                    variant: "danger"
                    text: qsTr("Confirm")
                    onClicked: confirm.open()
                }
                Button {
                    variant: "ghost"
                    text: qsTr("Drawer")
                    onClicked: drawer.open()
                }
                Dropdown {
                    label: qsTr("Actions")
                    icon: "more-horizontal"
                    items: [
                        {
                            label: qsTr("Rename"),
                            icon: "pencil"
                        },
                        {
                            label: qsTr("Duplicate"),
                            icon: "plus"
                        },
                        {
                            separator: true
                        },
                        {
                            label: qsTr("Delete"),
                            icon: "x",
                            danger: true
                        }
                    ]
                }
            }

            MouseArea {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.px(40)
                acceptedButtons: Qt.RightButton
                onClicked: e => menu.popupAt(e.x, e.y)

                Txt {
                    anchors.centerIn: parent
                    text: qsTr("Right-click here")
                    color: Theme.muted
                    pixelSize: Theme.fontXs
                }
            }
        }

        // ── Icons ────────────────────────────────────────────────────────────
        Card {
            Layout.fillWidth: true
            visible: root.section === "icons"
            title: qsTr("Icons")

            Txt {
                Layout.fillWidth: true
                text: qsTr("Every vendored lucide glyph, so a missing or misnamed SVG is visible here rather than on some page you forgot to open.")
                color: Theme.subtext
                pixelSize: Theme.fontXs
                wrapMode: Text.WordWrap
            }

            Flow {
                Layout.fillWidth: true
                spacing: Theme.space3

                Repeater {
                    model: ["activity", "alert-circle", "alert-triangle", "bar-chart-3", "bell", "box", "calendar", "calendar-clock", "check", "check-circle", "chevron-down", "chevron-left", "chevron-right", "chevron-up", "chevrons-up-down", "cloud-moon", "download", "eye", "eye-off", "file-code", "grip-vertical", "home", "info", "layout-grid", "list-checks", "minus", "moon", "more-horizontal", "panel-left-close", "panel-left-open", "pencil", "plug", "plus", "refresh-cw", "save", "search", "settings", "star", "sun-medium", "upload", "volume-1", "volume-2", "volume-x", "x", "x-circle"]
                    delegate: Icon {
                        required property var modelData
                        name: modelData
                        size: Theme.px(18)
                        color: Theme.subtext
                    }
                }
            }
        }
    }

    ConfirmDialog {
        id: confirm
        title: qsTr("Delete this?")
        message: qsTr("This cannot be undone.")
        confirmText: qsTr("Delete")
        danger: true
        onConfirmed: toasts.error(qsTr("Deleted."))
    }

    Drawer {
        id: drawer
        Txt {
            text: qsTr("A drawer slides in from the edge.")
            color: Theme.text
            pixelSize: Theme.fontSm
        }
    }

    ContextMenu {
        id: menu
        items: [
            {
                label: qsTr("Cut"),
                icon: "x"
            },
            {
                label: qsTr("Copy"),
                icon: "save"
            },
            {
                separator: true
            },
            {
                label: qsTr("Delete"),
                icon: "x",
                danger: true
            }
        ]
    }

    Toast {
        id: toasts
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.space5
    }
}
