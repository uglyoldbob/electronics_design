#ifndef FOOTPRINTDRAW_H
#define FOOTPRINTDRAW_H

#include <QPainter>
#include <QtQuick/QQuickPaintedItem>

class FootprintDraw : public QQuickPaintedItem
{
Q_OBJECT
public:
    FootprintDraw(QQuickItem *parent = nullptr);
    virtual void paint(QPainter *painter) override;
    static void qml_register();

    virtual void mouseDoubleClickEvent(QMouseEvent *event) override;
    virtual void mouseMoveEvent(QMouseEvent *event) override;
    virtual void mousePressEvent(QMouseEvent *event) override;
    virtual void mouseReleaseEvent(QMouseEvent *event) override;
    virtual void mouseUngrabEvent() override;
};

#endif // FOOTPRINTDRAW_H
