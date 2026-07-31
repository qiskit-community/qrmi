#
# This code is part of Qiskit.
#
# (C) Copyright IBM 2026
#
# This program and the accompanying materials are made available under the
# terms of the GNU General Public License version 3, as published by the
# Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
#

cmake_minimum_required(VERSION 3.16)
project(lua_qrmi C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# -----------------------------------------------------------------------------
# Find Lua itself (standard/PUC-Rio Lua 5.1-5.4)
# -----------------------------------------------------------------------------
find_package(PkgConfig REQUIRED)

# The pkg-config name varies by distro (lua5.4 / lua5.3 / lua), so try each in turn
set(_LUA_PKG_CANDIDATES lua5.4 lua5.3 lua5.1 lua)
foreach(_pkg ${_LUA_PKG_CANDIDATES})
    pkg_check_modules(LUA QUIET ${_pkg})
    if(LUA_FOUND)
        message(STATUS "Lua found via pkg-config: ${_pkg} (version ${LUA_VERSION})")
        break()
    endif()
endforeach()

if(NOT LUA_FOUND)
    message(FATAL_ERROR
        "Could not find Lua development headers. "
        "Try: apt install liblua5.4-dev, or "
        "pass -DLUA_INCLUDE_DIRS=... -DLUA_LIBRARIES=... manually.")
endif()

# -----------------------------------------------------------------------------
# Find QRMI (the real libqrmi.so)
#
# Expected layout (header and library in the same directory):
#   <QRMI_ROOT>/qrmi.h
#   <QRMI_ROOT>/libqrmi.so
#
# Usage:
#   cmake -DQRMI_ROOT=/path/to/qrmi/install ..
# or individually:
#   cmake -DQRMI_INCLUDE_DIR=/path/to/dir -DQRMI_LIBRARY=/path/to/libqrmi.so ..
# -----------------------------------------------------------------------------
find_path(QRMI_INCLUDE_DIR
    NAMES qrmi.h
    HINTS ${QRMI_ROOT} ${CMAKE_CURRENT_SOURCE_DIR}
    DOC "Directory containing qrmi.h")

find_library(QRMI_LIBRARY
    NAMES qrmi
    HINTS ${QRMI_ROOT}
    DOC "libqrmi.so (or libqrmi.a)")

if(NOT QRMI_INCLUDE_DIR OR NOT QRMI_LIBRARY)
    message(FATAL_ERROR
        "Could not find libqrmi.so / qrmi.h.\n"
        "  cmake -DQRMI_ROOT=/path/to/qrmi ..\n"
        "or\n"
        "  cmake -DQRMI_INCLUDE_DIR=... -DQRMI_LIBRARY=... ..")
endif()

message(STATUS "QRMI include dir: ${QRMI_INCLUDE_DIR}")
message(STATUS "QRMI library:     ${QRMI_LIBRARY}")

# -----------------------------------------------------------------------------
# The Lua module itself (the .so loaded by require("qrmi"))
# -----------------------------------------------------------------------------
add_library(qrmi_module MODULE lua_qrmi.c)

set_target_properties(qrmi_module PROPERTIES
    OUTPUT_NAME "qrmi"   # produces qrmi.so (no "lib" prefix)
    PREFIX ""
    C_VISIBILITY_PRESET default)

target_include_directories(qrmi_module PRIVATE
    ${LUA_INCLUDE_DIRS}
    ${QRMI_INCLUDE_DIR})

target_link_directories(qrmi_module PRIVATE ${LUA_LIBRARY_DIRS})
target_link_libraries(qrmi_module PRIVATE ${LUA_LIBRARIES} ${QRMI_LIBRARY})

# Assumes libqrmi.so lives alongside this module, but to let it run without
# setting LD_LIBRARY_PATH during development, bake the linked library's
# directory into the rpath.
get_filename_component(_qrmi_lib_dir ${QRMI_LIBRARY} DIRECTORY)
set_target_properties(qrmi_module PROPERTIES
    BUILD_RPATH "${_qrmi_lib_dir}"
    INSTALL_RPATH "${_qrmi_lib_dir}")

# -----------------------------------------------------------------------------
# Install (optional)
# -----------------------------------------------------------------------------
install(TARGETS qrmi_module
    LIBRARY DESTINATION lib/lua/${LUA_VERSION})
