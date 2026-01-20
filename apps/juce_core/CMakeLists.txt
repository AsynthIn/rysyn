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

# Find required audio codec libraries
find_package(PkgConfig REQUIRED)
pkg_check_modules(FLAC REQUIRED flac)
pkg_check_modules(VORBIS REQUIRED vorbisenc vorbisfile vorbis ogg)

# Rust FFI library path
set(RYSYN_FFI_LIB "${CMAKE_SOURCE_DIR}/../../target/release/librysyn_ffi_bridge.a")
set(RYSYN_FFI_INCLUDE "${CMAKE_SOURCE_DIR}/include")

# Create the audio engine library using juce_add_console_app to get JuceHeader.h
juce_add_console_app(rysyn_juce_core
    PRODUCT_NAME "Rysyn Audio Core"
    VERSION "0.1.0"
)

# Add sources
target_sources(rysyn_juce_core PRIVATE
    src/AudioCore.cpp
    src/AudioCore.h
    src/AudioLoader.cpp
    src/AudioLoader.h
    src/PluginHost.cpp
    src/PluginHost.h
    src/Transport.cpp
    src/Transport.h
    src/TrackProcessor.cpp
    src/TrackProcessor.h
    src/main.cpp
)

# Generate JuceHeader.h
juce_generate_juce_header(rysyn_juce_core)

target_include_directories(rysyn_juce_core PUBLIC
    ${CMAKE_SOURCE_DIR}/include
    ${CMAKE_SOURCE_DIR}/src
)

target_link_libraries(rysyn_juce_core
    PRIVATE
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
        juce::juce_recommended_config_flags
        juce::juce_recommended_warning_flags
        ${FLAC_LIBRARIES}
        ${VORBIS_LIBRARIES}
)

# JUCE recommended definitions
target_compile_definitions(rysyn_juce_core
    PRIVATE
        JUCE_WEB_BROWSER=0
        JUCE_USE_CURL=0
        JUCE_VST3_CAN_REPLACE_VST2=0
        JUCE_PLUGINHOST_VST3=1
        JUCE_PLUGINHOST_AU=1
        JUCE_PLUGINHOST_LADSPA=1
)

# Link to Rust FFI if available (must come after target definitions)
if(EXISTS ${RYSYN_FFI_LIB})
    target_link_libraries(rysyn_juce_core PRIVATE ${RYSYN_FFI_LIB} dl pthread)
    target_compile_definitions(rysyn_juce_core PRIVATE RYSYN_FFI_AVAILABLE=1)
    message(STATUS "Linking with Rust FFI library: ${RYSYN_FFI_LIB}")
else()
    message(WARNING "Rust FFI library not found at: ${RYSYN_FFI_LIB}")
endif()
