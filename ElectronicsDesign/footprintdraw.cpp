#include "footprintdraw.h"

FootprintDraw::FootprintDraw(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAcceptedMouseButtons(Qt::AllButtons);
}

void FootprintDraw::mouseDoubleClickEvent(QMouseEvent *event)
{

}

void FootprintDraw::mouseMoveEvent(QMouseEvent *event)
{

}

void FootprintDraw::mousePressEvent(QMouseEvent *event)
{

}

void FootprintDraw::mouseReleaseEvent(QMouseEvent *event)
{

}

void FootprintDraw::mouseUngrabEvent()
{

}

void FootprintDraw::paint(QPainter *painter)
{
    painter->save();
    QRect vp = painter->viewport();
    QRect w = painter->window();
    painter->drawRect(w.x(), w.y(), w.width()-1, w.height()-1);
    painter->restore();
}

void FootprintDraw::qml_register()
{
    qmlRegisterType<FootprintDraw>("uglyoldbob", 1, 0, "FootprintDraw");
}
