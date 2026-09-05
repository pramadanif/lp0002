// Standalone preview app — loads the QML UI without Basecamp.
// Build with: cmake -B build && cmake --build build
// Run with:   PRIVATE_MULTISIG_PROGRAM_ID=<hex> ./build/private_multisig_app

#include "PrivateMultisigBackend.h"
#include "PrivateMultisigPlugin.h"

#include <QApplication>
#include <QQmlContext>
#include <QQmlEngine>
#include <QQuickWidget>
#include <QUrl>
#include <cstdlib>

int main(int argc, char** argv) {
	QApplication app(argc, argv);
	app.setOrganizationName("logos-co");
	app.setApplicationName("private_multisig");

	PrivateMultisigBackend backend(nullptr);

	QQuickWidget view;
	view.engine()->rootContext()->setContextProperty("backend", &backend);
	view.setResizeMode(QQuickWidget::SizeRootObjectToView);
	view.resize(900, 640);

	const char* qmlPath = std::getenv("QML_PATH");
	if (qmlPath)
		view.setSource(QUrl::fromLocalFile(QString::fromUtf8(qmlPath) + "/Main.qml"));
	else
		view.setSource(QUrl("qrc:/qml/Main.qml"));

	view.setWindowTitle("PrivateMultisig");
	view.show();
	return app.exec();
}
