cmake_minimum_required(VERSION 3.20)
project(feathersurf-linux VERSION 0.0.1 LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Find GTK4
find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK4 REQUIRED gtk4)

# CEF configuration (optional, for web rendering)
option(USE_CEF "Use CEF for web rendering" OFF)

if(USE_CEF)
    if(NOT DEFINED CEF_ROOT)
        message(FATAL_ERROR "CEF_ROOT must be set when USE_CEF is ON")
    endif()
    include("${CEF_ROOT}/cmake/cef_variables.cmake")
endif()

add_executable(feathersurf
    src/main.c
    src/browser_window.c
    src/cef_handler.c
)

target_include_directories(feathersurf PRIVATE
    src
    ${GTK4_INCLUDE_DIRS}
)

target_link_directories(feathersurf PRIVATE
    ${GTK4_LIBRARY_DIRS}
)

target_link_libraries(feathersurf PRIVATE
    ${GTK4_LIBRARIES}
    m
)

target_compile_options(feathersurf PRIVATE
    ${GTK4_CFLAGS_OTHER}
    -Wall -Wextra -Wpedantic
)

if(USE_CEF)
    target_link_libraries(feathersurf PRIVATE
        libcef_dll_wrapper
        ${CEF_LIBRARIES}
    )
    target_compile_definitions(feathersurf PRIVATE USE_CEF=1)
endif()

set_target_properties(feathersurf PROPERTIES
    OUTPUT_NAME "feathersurf"
    INSTALL_RPATH "$ORIGIN"
)

# Install
install(TARGETS feathersurf DESTINATION bin)
