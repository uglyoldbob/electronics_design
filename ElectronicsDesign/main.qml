import QtQuick 2.0
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12
import uglyoldbob 1.0

ApplicationWindow {
    visible: true
    id: window
    title: ((currentDesign.blab !== 0) ? "blabbing " : "non-blabbing ") + "sandwhich"
    Design {
        id: currentDesign
    }

    Button {
            text: "Ok"
            onClicked: {
                currentDesign.tweak_blabbing()
            }
        }

}
