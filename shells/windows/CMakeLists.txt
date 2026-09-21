cmake_minimum_required(VERSION 3.20)
project(feathersurf-windows VERSION 0.0.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# CEF configuration - set CEF_ROOT to your CEF binary distribution
if(NOT DEFINED CEF_ROOT)
    message(FATAL_ERROR "CEF_ROOT must be set to the CEF binary distribution path")
endif()

set(CEF_ROOT "${CEF_ROOT}" CACHE PATH "CEF binary distribution root")

# Include CEF
include("${CEF_ROOT}/cmake/cef_variables.cmake")
include("${CEF_ROOT}/cmake/cef_utils.cmake")

add_executable(feathersurf WIN32
    src/main.cpp
    src/browser_window.cpp
    src/cef_handler.cpp
)

target_include_directories(feathersurf PRIVATE
    src
    ${CEF_ROOT}
)

# Link CEF
target_link_libraries(feathersurf PRIVATE
    libcef_dll_wrapper
    ${CEF_LIBRARIES}
)

# Add CEF-specific compile definitions
target_compile_definitions(feathersurf PRIVATE
    ${CEF_DEFINITIONS}
)

# Windows-specific settings
target_link_libraries(feathersurf PRIVATE
    comctl32
    shlwapi
)

set_target_properties(feathersurf PROPERTIES
    VS_DPI_AWARE "ON"
    OUTPUT_NAME "FeatherSurf"
)
