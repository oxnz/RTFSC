cmake_minimum_required(VERSION 3.22)
project(rtfsc LANGUAGES C CXX)


set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_C_STANDARD 23)

add_compile_options(-Wall -Wextra -Werror)

add_subdirectory(par)
add_subdirectory(net/dns)
# add_subdirectory(net/http/mhttpd)
