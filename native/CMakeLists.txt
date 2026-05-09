cmake_minimum_required(VERSION 3.20)
project(cide_native VERSION 0.1.0 LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Unified output directories: all binaries go to build/bin or build/lib
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Platform detection
if(WIN32)
    set(CIDE_PLATFORM "windows")
elseif(ANDROID)
    set(CIDE_PLATFORM "android")
    # Android 16 requires 16 KB page size alignment
    set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -Wl,-z,max-page-size=16384")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    set(CIDE_PLATFORM "linux")
elseif(APPLE)
    set(CIDE_PLATFORM "ios")
endif()

# Source files (explicitly listed to ensure rebuild on add/remove)
set(CIDE_SOURCES
    src/capi/cide_capi.cpp
    src/compiler/BytecodeGen.cpp
    src/compiler/Lexer.cpp
    src/compiler/Parser.cpp
    src/compiler/TypeChecker.cpp
    src/diagnostics/AlgorithmMatcher.cpp
    src/vm/CideVM.cpp
    src/vm/HostFunctions.cpp
)

# Header directories
set(CIDE_INCLUDE_DIRS
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/src"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/diagnostics"
)

# wasm3 removed - using custom CideVM instead

# Main shared library
add_library(cide_native SHARED ${CIDE_SOURCES})

target_include_directories(cide_native PUBLIC ${CIDE_INCLUDE_DIRS})

# Force UTF-8 for all source files and string literals
if(MSVC)
    target_compile_options(cide_native PRIVATE /utf-8)
elseif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    target_compile_options(cide_native PRIVATE -Wall -Wextra -fexec-charset=UTF-8 -finput-charset=UTF-8)
endif()



# Export C API
if(WIN32)
    target_compile_definitions(cide_native PRIVATE CIDE_EXPORTS)
endif()

set_target_properties(cide_native PROPERTIES
    C_VISIBILITY_PRESET hidden
    CXX_VISIBILITY_PRESET hidden
    VISIBILITY_INLINES_HIDDEN ON
)

# Tests
option(CIDE_BUILD_TESTS "Build tests" ON)
if(CIDE_BUILD_TESTS)
    enable_testing()
    set(CIDE_TESTS
        phase2_regression_test:Phase2RegressionTest.cpp
        phase3_batch1_test:Phase3Batch1Test.cpp
        phase3_batch2_test:Phase3Batch2Test.cpp
        phase3_batch3_test:Phase3Batch3Test.cpp
        phase3_batch4_test:Phase3Batch4Test.cpp
        phase3_step_test:Phase3StepTest.cpp
        test_new_features:test_new_features.cpp
        stage2_diagnostic_test:Stage2DiagnosticTest.cpp
        algorithm_match_test:AlgorithmMatchTest.cpp
        struct_init_test:StructInitTest.cpp
    )
    foreach(test_mapping ${CIDE_TESTS})
        string(REPLACE ":" ";" parts ${test_mapping})
        list(GET parts 0 test_name)
        list(GET parts 1 test_file)
        add_executable(${test_name} tests/${test_file})
        target_link_libraries(${test_name} PRIVATE cide_native)
        target_include_directories(${test_name} PRIVATE ${CIDE_INCLUDE_DIRS})
        if(MSVC)
            target_compile_options(${test_name} PRIVATE /utf-8)
        elseif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
            target_compile_options(${test_name} PRIVATE -Wall -Wextra -fexec-charset=UTF-8 -finput-charset=UTF-8)
        endif()
        add_test(NAME ${test_name} COMMAND ${test_name})
    endforeach()

endif()

# Install
install(TARGETS cide_native
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/include/cide_capi.h" DESTINATION include)
