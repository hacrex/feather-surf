cmake_minimum_required(VERSION 3.19)
project(FeatherSurf LANGUAGES C CXX OBJCXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_OBJCXX_STANDARD 17)

# ---------------------------------------------------------------------------
# CEF integration
#
# Set CEF_DIR to the extracted CEF binary distribution directory.
# Example: cmake -DCEF_DIR=/path/to/cef_binary_120.1.10+g962468f+chromium-120.0.6099.129 ..
# ---------------------------------------------------------------------------

if(NOT DEFINED CEF_DIR)
    message(WARNING "CEF_DIR not set – CEF headers/libs will not be found.")
endif()

# ---------------------------------------------------------------------------
# Rust FFI library
#
# Build the Rust FFI crate first (via cargo), then point RUST_LIB_DIR at
# the output directory containing libfeathersurf_ffi.dylib.
# ---------------------------------------------------------------------------

if(NOT DEFINED RUST_LIB_DIR)
    set(RUST_LIB_DIR "${CMAKE_SOURCE_DIR}/../../rust/target/debug")
endif()

# ---------------------------------------------------------------------------
# Source files
# ---------------------------------------------------------------------------

set(SOURCES
    FeatherSurf/main.m
    FeatherSurf/AppDelegate.mm
    FeatherSurf/BrowserWindowController.mm
)

set(HEADERS
    FeatherSurf/AppDelegate.h
    FeatherSurf/BrowserWindowController.h
)

# ---------------------------------------------------------------------------
# Executable
# ---------------------------------------------------------------------------

add_executable(${PROJECT_NAME} MACOSX_BUNDLE ${SOURCES} ${HEADERS})

target_include_directories(${PROJECT_NAME} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CEF_DIR}
)

# Link against CEF framework
if(DEFINED CEF_DIR)
    find_library(CEF_FRAMEWORK NAMES "Chromium Embedded Framework"
                 PATHS "${CEF_DIR}" NO_DEFAULT_PATH)
    if(CEF_FRAMEWORK)
        target_link_libraries(${PROJECT_NAME} PRIVATE ${CEF_FRAMEWORK})
    endif()

    # CEF wrapper (libcef_dll_wrapper) – static library built with CEF
    if(EXISTS "${CEF_DIR}/build/libcef_dll_wrapper/libcef_dll_wrapper.a")
        target_link_libraries(${PROJECT_NAME} PRIVATE
            "${CEF_DIR}/build/libcef_dll_wrapper/libcef_dll_wrapper.a")
    endif()
endif()

# Link against the Rust FFI library
find_library(RUST_FFI_LIB NAMES feathersurf_ffi
             PATHS "${RUST_LIB_DIR}" NO_DEFAULT_PATH)
if(RUST_FFI_LIB)
    target_link_libraries(${PROJECT_NAME} PRIVATE ${RUST_FFI_LIB})
else()
    # Fallback: link by full path
    target_link_libraries(${PROJECT_NAME} PRIVATE
        "${RUST_LIB_DIR}/libfeathersurf_ffi.dylib")
endif()

# macOS frameworks
target_link_libraries(${PROJECT_NAME} PRIVATE
    "-framework Cocoa"
    "-framework AppKit"
    "-framework CoreFoundation"
)

set_target_properties(${PROJECT_NAME} PROPERTIES
    MACOSX_BUNDLE TRUE
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/Info.plist"
    MACOSX_BUNDLE_GUI_IDENTIFIER "com.feathersurf.browser"
    MACOSX_BUNDLE_BUNDLE_NAME "FeatherSurf"
    MACOSX_BUNDLE_BUNDLE_VERSION "0.1.0"
    MACOSX_BUNDLE_SHORT_VERSION_STRING "0.1.0"
)

# ---------------------------------------------------------------------------
# CEF helper subprocess (required for multi-process architecture)
# ---------------------------------------------------------------------------

add_executable("${PROJECT_NAME} Helper" MACOSX_BUNDLE
    FeatherSurf/main.m
)

target_include_directories("${PROJECT_NAME} Helper" PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CEF_DIR}
)

if(DEFINED CEF_DIR AND CEF_FRAMEWORK)
    target_link_libraries("${PROJECT_NAME} Helper" PRIVATE ${CEF_FRAMEWORK})
endif()

set_target_properties("${PROJECT_NAME} Helper" PROPERTIES
    MACOSX_BUNDLE TRUE
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/Info.plist"
    MACOSX_BUNDLE_GUI_IDENTIFIER "com.feathersurf.browser.helper"
    MACOSX_BUNDLE_BUNDLE_NAME "FeatherSurf Helper"
    MACOSX_BUNDLE_BUNDLE_VERSION "0.1.0"
    MACOSX_BUNDLE_SHORT_VERSION_STRING "0.1.0"
    OUTPUT_NAME "FeatherSurf Helper"
)

# ---------------------------------------------------------------------------
# Install into .app bundle structure
# ---------------------------------------------------------------------------

install(TARGETS ${PROJECT_NAME}
    BUNDLE DESTINATION "${CMAKE_BINARY_DIR}"
)

install(TARGETS "${PROJECT_NAME} Helper"
    BUNDLE DESTINATION "${CMAKE_BINARY_DIR}/${PROJECT_NAME}.app/Contents/Frameworks"
)

# Copy the Rust FFI dylib into the bundle
if(RUST_FFI_LIB OR EXISTS "${RUST_LIB_DIR}/libfeathersurf_ffi.dylib")
    install(FILES "${RUST_LIB_DIR}/libfeathersurf_ffi.dylib"
        DESTINATION "${CMAKE_BINARY_DIR}/${PROJECT_NAME}.app/Contents/Frameworks"
    )
endif()
