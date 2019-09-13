#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QFileDialog>

MainWindow::MainWindow(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::MainWindow)
{
    d1 = nullptr; //initialize to null pointer
    ui->setupUi(this);
}

MainWindow::~MainWindow()
{
    if (d1 != nullptr)
    {
        delete d1;
        d1 = nullptr;
    }
    delete ui;
}

void MainWindow::on_pushButton_clicked()
{
    if (d1 == nullptr)
    {
        d1 = new design();
        //TODO: find a better default folder
        QString fileName = QFileDialog::getOpenFileName(this,
            tr("Open Image"), "./", tr("Image Files (*.png *.jpg *.bmp)"));
    }
}
