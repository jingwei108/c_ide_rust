# The Flutter tooling requires that developers have a version of Visual Studio
# installed that includes CMake 3.14 or later. You should not increase this
# version, as doing so will cause the plugin to fail to compile for some
# customers of the plugin.
cmake_minimum_required(VERSION 3.14)

# Project-level configuration.
set(PROJECT_NAME "cide_native")
project(${PROJECT_NAME} LANGUAGES CXX)

# 计算项目根目录（从软链接路径向上回溯到 c_ide_rust）
get_filename_component(PLUGIN_DIR "${CMAKE_CURRENT_SOURCE_DIR}" DIRECTORY)
get_filename_component(SYMLINKS_DIR "${PLUGIN_DIR}" DIRECTORY)
get_filename_component(EPHEMERAL_DIR "${SYMLINKS_DIR}" DIRECTORY)
get_filename_component(FLUTTER_DIR "${EPHEMERAL_DIR}" DIRECTORY)
get_filename_component(WINDOWS_DIR "${FLUTTER_DIR}" DIRECTORY)
get_filename_component(CIDEFLutter_DIR "${WINDOWS_DIR}" DIRECTORY)
get_filename_component(PROJECT_ROOT "${CIDEFLutter_DIR}" DIRECTORY)

# 查找预构建的 DLL（支持手动 cargo build 产物）
set(CIDE_NATIVE_CANDIDATE_DLLS
    "${PROJECT_ROOT}/native/target/debug/cide_native.dll"
    "${PROJECT_ROOT}/native/target/release/cide_native.dll"
)

set(CIDE_NATIVE_PREBUILT_DLL "")
foreach(dll_path IN LISTS CIDE_NATIVE_CANDIDATE_DLLS)
    if(EXISTS "${dll_path}")
        set(CIDE_NATIVE_PREBUILT_DLL "${dll_path}")
        break()
    endif()
endforeach()

if(CIDE_NATIVE_PREBUILT_DLL)
    message(STATUS "Using prebuilt cide_native.dll: ${CIDE_NATIVE_PREBUILT_DLL}")
    set(cide_native_bundled_libraries
        "${CIDE_NATIVE_PREBUILT_DLL}"
        PARENT_SCOPE
    )
else()
    message(STATUS "Prebuilt DLL not found, falling back to cargokit build")
    include("../cargokit/cmake/cargokit.cmake")
    apply_cargokit(${PROJECT_NAME} ../../../native cide_native "")
    set(cide_native_bundled_libraries
        "${${PROJECT_NAME}_cargokit_lib}"
        PARENT_SCOPE
    )
endif()
