cmake_minimum_required(VERSION 3.5)
project(mhttpd VERSION 1.0.0 LANGUAGES CXX)

set(THREADS_PREFER_PTHREAD_FLAG ON)
find_package(Threads REQUIRED)

file(GLOB mhttpd_SRC
        "${PROJECT_SOURCE_DIR}/include/*.h"
        "${PROJECT_SOURCE_DIR}/src/*.cc"
)
configure_file(
        "${PROJECT_SOURCE_DIR}/include/config.h.in"
        "${PROJECT_SOURCE_DIR}/include/config.h"
		)
add_executable(mhttpd ${mhttpd_SRC})
target_include_directories(
		mhttpd
		PUBLIC
		$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
		$<INSTALL_INTERFACE:include>
		)
target_compile_features(mhttpd PRIVATE cxx_std_17)
target_link_libraries(mhttpd Threads::Threads)
