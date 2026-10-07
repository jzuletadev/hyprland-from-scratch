// ~/hyprland-from-scratch/system/sddm/themes/arch-neutral/Main.qml
//
// Phase 24, made minimal in Phase 31. SDDM login theme matching the lock screen
// (dotfiles/hypr/hyprlock.conf): wallpaper blurred and dimmed, a modest clock
// and date. The sign-in form (user, password, session, reboot / power off)
// stays hidden until a click or a key press, which also counts as the first
// character of the password; Esc or 30 s without input hide it again.
// Qt 6 (QtVersion=6 in metadata.desktop), so SDDM runs it with sddm-greeter-qt6.
//
// SDDM hands every theme these objects: `sddm` (login(), reboot(), powerOff(),
// signal loginFailed), `userModel` (lastUser), `sessionModel` (lastIndex),
// `config` (theme.conf) and `keyboard` (capsLock).
//
// Preview without logging out:
//   sddm-greeter-qt6 --test-mode --theme ~/hyprland-from-scratch/system/sddm/themes/arch-neutral

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Effects

Rectangle {
    id: root
    width: 1920
    height: 1080
    color: "#141414"
    focus: true

    // Arch Neutral palette (docs/build-log.md Phase 15)
    readonly property color fg: "#e0e0e0"
    readonly property color dim: "#a8a8a8"
    readonly property color muted: "#6e6e6e"
    readonly property color surface: "#1f1f1f"
    readonly property color borderColor: "#333333"
    readonly property color accent: "#1793d1"
    readonly property color warning: "#e0a84e"
    readonly property color critical: "#e05f65"
    readonly property string fontFamily: "Adwaita Sans"   // Phase 26: same as the desktop

    property bool failed: false
    property bool revealed: false

    function reveal() {
        root.revealed = true
        password.forceActiveFocus()
        idle.restart()
    }
    function conceal() {
        root.revealed = false
        root.failed = false
        password.text = ""
        root.forceActiveFocus()
    }
    function login() {
        root.failed = false
        sddm.login(user.text, password.text, session.currentIndex)
    }

    Connections {
        target: sddm
        function onLoginFailed() {
            root.failed = true
            password.text = ""
            password.forceActiveFocus()
            idle.restart()
        }
    }

    // any key while the form is hidden reveals it; a printable key is kept as
    // the first character of the password
    Keys.onPressed: (event) => {
        if (root.revealed)
            return
        root.reveal()
        if (event.text.length > 0 && event.key !== Qt.Key_Escape
                && event.key !== Qt.Key_Return && event.key !== Qt.Key_Enter)
            password.insert(password.cursorPosition, event.text)
        event.accepted = true
    }

    // 30 s without input: back to just the clock
    Timer {
        id: idle
        interval: 30000
        onTriggered: root.conceal()
    }

    // ---- background: wallpaper, blurred and darkened like hyprlock's ----
    Image {
        id: wallpaper
        anchors.fill: parent
        source: config.background
        fillMode: Image.PreserveAspectCrop
        visible: false   // drawn through the effect below
    }
    MultiEffect {
        anchors.fill: parent
        source: wallpaper
        blurEnabled: true
        blur: 1.0
        blurMax: 48
        brightness: -0.4
    }

    // a click anywhere reveals the form
    MouseArea {
        anchors.fill: parent
        onClicked: root.reveal()
    }

    // ---- clock and date ----
    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -110
        spacing: 6

        Text {
            id: clock
            anchors.horizontalCenter: parent.horizontalCenter
            color: root.fg
            font.family: root.fontFamily
            font.pixelSize: 84
            font.weight: Font.Medium
            font.features: { "tnum": 1 }   // tabular digits: the clock doesn't shift
        }
        Text {
            id: date
            anchors.horizontalCenter: parent.horizontalCenter
            color: root.dim
            font.family: root.fontFamily
            font.pixelSize: 18
        }
    }

    // hint while the form is hidden
    Text {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: 20
        text: "Press any key or click to sign in"
        color: root.muted
        font.family: root.fontFamily
        font.pixelSize: 13
        opacity: root.revealed ? 0 : 1
        Behavior on opacity { NumberAnimation { duration: 250 } }
    }

    // ---- sign-in form (hidden until revealed) ----
    Column {
        id: form
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: 40
        spacing: 10
        opacity: root.revealed ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: 250 } }

        TextField {
            id: user
            anchors.horizontalCenter: parent.horizontalCenter
            width: 260
            text: userModel.lastUser
            placeholderText: "user"
            placeholderTextColor: root.dim
            color: root.fg
            font.family: root.fontFamily
            font.pixelSize: 15
            horizontalAlignment: TextInput.AlignHCenter
            background: Item {}
            KeyNavigation.tab: password
            onTextEdited: idle.restart()
            onAccepted: password.forceActiveFocus()
        }

        TextField {
            id: password
            anchors.horizontalCenter: parent.horizontalCenter
            width: 260
            height: 42
            echoMode: TextInput.Password
            placeholderText: "Password"
            placeholderTextColor: root.muted
            color: root.fg
            font.family: root.fontFamily
            font.pixelSize: 14
            horizontalAlignment: TextInput.AlignHCenter
            background: Rectangle {
                radius: height / 2
                color: "#b3141414"
                border.width: 1
                // red after a failed login, amber with Caps Lock, Arch blue otherwise
                border.color: root.failed ? root.critical
                            : keyboard.capsLock ? root.warning : root.accent
            }
            Keys.onEscapePressed: root.conceal()
            onTextEdited: { root.failed = false; idle.restart() }
            onAccepted: root.login()
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            color: root.failed ? root.critical : root.warning
            font.family: root.fontFamily
            font.pixelSize: 12
            text: root.failed ? "Wrong password" : keyboard.capsLock ? "Caps Lock is on" : " "
        }
    }

    // ---- session picker and power buttons (with the form) ----
    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 32
        spacing: 20
        opacity: form.opacity
        visible: form.visible

        ComboBox {
            id: session
            width: 240
            height: 34
            model: sessionModel
            textRole: "name"
            currentIndex: sessionModel.lastIndex
            font.family: root.fontFamily
            font.pixelSize: 12

            contentItem: Text {
                leftPadding: 12
                rightPadding: 26
                text: session.displayText
                color: root.dim
                font: session.font
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }
            indicator: Text {
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                text: "\u25be"   // small down triangle
                color: root.dim
                font.pixelSize: 10
            }
            background: Rectangle {
                radius: height / 2
                color: "#991f1f1f"
                border.color: session.hovered || session.popup.visible ? root.accent : root.borderColor
            }
            delegate: ItemDelegate {
                id: option
                required property int index
                required property string name
                width: session.width
                highlighted: session.highlightedIndex === index
                contentItem: Text {
                    text: option.name
                    color: option.highlighted ? root.accent : root.fg
                    font: session.font
                    elide: Text.ElideRight
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    radius: 6
                    color: option.highlighted ? "#2b2b2b" : "transparent"
                }
            }
            popup.background: Rectangle {
                radius: 10
                color: root.surface
                border.color: root.borderColor
            }
        }

        Repeater {
            model: [
                { icon: "\uf01e", label: "Reboot",    action: function() { sddm.reboot() },   enabled: sddm.canReboot },
                { icon: "\uf011", label: "Power off", action: function() { sddm.powerOff() }, enabled: sddm.canPowerOff }
            ]
            delegate: Text {
                id: powerButton
                required property var modelData
                visible: modelData.enabled
                height: 34
                verticalAlignment: Text.AlignVCenter
                text: modelData.icon + "  " + modelData.label
                color: area.containsMouse ? root.accent : root.dim
                font.family: root.fontFamily
                font.pixelSize: 12
                MouseArea {
                    id: area
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: powerButton.modelData.action()
                }
            }
        }
    }

    // ---- clock updates ----
    function tick() {
        const now = new Date()
        clock.text = Qt.formatTime(now, "HH:mm")
        date.text = Qt.formatDate(now, "dddd, dd MMMM")
    }
    Timer {
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: root.tick()
    }

    Component.onCompleted: root.forceActiveFocus()
}
