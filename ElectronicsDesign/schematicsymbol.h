#ifndef SCHEMATICSYMBOL_H
#define SCHEMATICSYMBOL_H

#include <QPainter>
#include <QPointF>
#include <QVector>

class SchematicSymbol
{
public:
    SchematicSymbol();
    void draw(QPainter *painter);
private:
    QVector<QPointF> points;
    QVector<QString> text;

    float x;
    float y;
    float angle;
};

#endif // SCHEMATICSYMBOL_H
