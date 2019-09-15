#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>

#include "design.h"

namespace Ui {
class MainWindow;
}

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    void on_pushButton_clicked();
    void on_pushButton_2_clicked();

    void on_pushButton_3_clicked();

    void on_pushButton_4_clicked();

signals:
    void hasDesign(bool n);

private:
    Ui::MainWindow *ui;

    design *d1;
};

#endif // MAINWINDOW_H