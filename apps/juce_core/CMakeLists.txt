cmake_minimum_required(VERSION 3.22)
project(RysynJuceCore VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find JUCE (installed via system or submodule)
find_package(JUCE CONFIG QUIET)

if(NOT JUCE_FOUND)
    # Fetch JUCE if not found
    include(FetchContent)
    FetchContent_Declare(
        JUCE
        GIT_REPOSITORY https://github.com/juce-framework/JUCE.git
        GIT_TAG 8.0.4
    )
    FetchContent_MakeAvailable(JUCE)
endif()

# Rust FFI library path
set(RYSYN_FFI_LIB "${CMAKE_SOURCE_DIR}/../../target/release/librysyn_ffi_bridge.a")
set(RYSYN_FFI_INCLUDE "${CMAKE_SOURCE_DIR}/include")

# Create the audio engine library
add_library(rysyn_juce_core STATIC
    src/AudioCore.cpp
    src/AudioCore.h
    src/PluginHost.cpp
    src/PluginHost.h
    src/Transport.cpp
    src/Transport.h
    src/TrackProcessor.cpp
    src/TrackProcessor.h
)

target_include_directories(rysyn_juce_core PUBLIC
    ${CMAKE_SOURCE_DIR}/include
    ${CMAKE_SOURCE_DIR}/src
)

target_link_libraries(rysyn_juce_core
    PUBLIC
        juce::juce_audio_basics
        juce::juce_audio_devices
        juce::juce_audio_formats
        juce::juce_audio_processors
        juce::juce_audio_utils
        juce::juce_core
        juce::juce_data_structures
        juce::juce_events
        juce::juce_graphics
        juce::juce_gui_basics
)

# Link to Rust FFI if available
if(EXISTS ${RYSYN_FFI_LIB})
    target_link_libraries(rysyn_juce_core PRIVATE ${RYSYN_FFI_LIB})
    target_compile_definitions(rysyn_juce_core PRIVATE RYSYN_FFI_AVAILABLE=1)
endif()

# JUCE recommended definitions
target_compile_definitions(rysyn_juce_core
    PUBLIC
        JUCE_WEB_BROWSER=0
        JUCE_USE_CURL=0
        JUCE_VST3_CAN_REPLACE_VST2=0
        JUCE_PLUGINHOST_VST3=1
        JUCE_PLUGINHOST_AU=1
        JUCE_PLUGINHOST_LADSPA=1
)

# Standalone executable for testing
add_executable(rysyn_audio_test
    src/main.cpp
)

target_link_libraries(rysyn_audio_test PRIVATE rysyn_juce_core)
