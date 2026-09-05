cmake_minimum_required(VERSION 3.16)
project(PrivateMultisig VERSION 1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets Qml Quick QuickWidgets Concurrent)

# ── FFI shared library (built by `make ffi`) ──────────────────────────────────
add_library(private_multisig_ffi SHARED IMPORTED)
set_target_properties(private_multisig_ffi PROPERTIES
    IMPORTED_LOCATION "${CMAKE_CURRENT_SOURCE_DIR}/lib/libpmsig_ffi.dylib"
)

# ── Plugin (loaded by Basecamp) ────────────────────────────────────────────
add_library(PrivateMultisigPlugin SHARED
    src/PrivateMultisigBackend.cpp
    src/PrivateMultisigPlugin.cpp
)

# Resource name "private_multisig_qml" must match Q_INIT_RESOURCE(private_multisig_qml) in PrivateMultisigPlugin.cpp
qt_add_resources(PrivateMultisigPlugin "private_multisig_qml"
    PREFIX "/"
    FILES qml/Main.qml
)

target_link_libraries(PrivateMultisigPlugin PRIVATE
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Qml Qt6::Quick Qt6::QuickWidgets Qt6::Concurrent
    private_multisig_ffi
)

# Output name must match manifest.json "main" value (lib<name>_plugin.so)
set_target_properties(PrivateMultisigPlugin PROPERTIES OUTPUT_NAME "private_multisig_plugin")

# ── Standalone preview app ─────────────────────────────────────────────────
add_executable(PrivateMultisigApp
    src/main.cpp
    src/PrivateMultisigBackend.cpp
    src/PrivateMultisigPlugin.cpp
)

# Same resource name as the plugin so Q_INIT_RESOURCE resolves in both targets
qt_add_resources(PrivateMultisigApp "private_multisig_qml"
    PREFIX "/"
    FILES qml/Main.qml
)

target_link_libraries(PrivateMultisigApp PRIVATE
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Qml Qt6::Quick Qt6::QuickWidgets Qt6::Concurrent
    private_multisig_ffi
)

set_target_properties(PrivateMultisigApp PROPERTIES
    BUILD_RPATH "${CMAKE_CURRENT_SOURCE_DIR}/lib"
)
