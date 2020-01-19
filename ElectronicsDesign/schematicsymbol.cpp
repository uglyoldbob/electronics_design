#include "schematicsymbol.h"

SchematicSymbol::SchematicSymbol()
{
    //the first three texts are mandatory
    text.append("");
    text.append("");
    text.append("");
}

/*
Lines
    2 point indexes
    width
    color
Arcs
    1 point index for center
    radius
    start angle
    end angle
    direction
    color
Ovals
    1 point index for center
    line width
    width
    height
    angle
    fill type
    color
Polygons
    number points n
    n point indexes
    line width
    fill type
    color
Polylines
    number points n
    n point indexes
    line width
    color
Rectangles
    4 point indexes
    line width
    color
Rounded Rectangles
    4 point indexes
    line width
    color
*/

void SchematicSymbol::draw(QPainter *painter)
{

}
