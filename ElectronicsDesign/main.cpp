#include <QApplication>
#include <QQmlApplicationEngine>

#include "design.h"

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);

    design::qml_register();

    QQmlApplicationEngine engine;
    engine.load(QUrl("qrc:/main.qml"));




    return a.exec();
}
