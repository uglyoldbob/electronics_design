#include "footprintdraw.h"

FootprintDraw::FootprintDraw(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{

}

void FootprintDraw::paint(QPainter *painter)
{

}

void FootprintDraw::qml_register()
{
    qmlRegisterType<FootprintDraw>("uglyoldbob", 1, 0, "FootprintDraw");
}
