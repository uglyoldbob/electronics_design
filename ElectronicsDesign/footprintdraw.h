#ifndef FOOTPRINTDRAW_H
#define FOOTPRINTDRAW_H

#include <QtQuick/QQuickPaintedItem>

class FootprintDraw : public QQuickPaintedItem
{
Q_OBJECT
public:
    FootprintDraw(QQuickItem *parent = nullptr);
    void paint(QPainter *painter);
    static void qml_register();
};

#endif // FOOTPRINTDRAW_H
