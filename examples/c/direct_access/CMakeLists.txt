# This code is part of Qiskit.
#
# (C) Copyright IBM 2025
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_STANDARD 11)
SET(CMAKE_OSX_DEPLOYMENT_TARGET 15.2)

project("qrmi-c" LANGUAGES C)

if ("${CMAKE_BUILD_TYPE}" STREQUAL "Debug")
  set(LIB_PATH debug)
else()
  set(LIB_PATH release)
endif()

set(PROJECT_ROOT ${CMAKE_CURRENT_SOURCE_DIR}/../../..)

function(add_example TARGET_NAME C_FILE)
  # source files for the client app
  set(SRC_FILE
          ${CMAKE_CURRENT_SOURCE_DIR}/${C_FILE}
          ${CMAKE_CURRENT_SOURCE_DIR}/src/common.c
  )

  add_executable(${TARGET_NAME} ${SRC_FILE})

  # add libraries
  target_link_libraries(${TARGET_NAME}
          PUBLIC
          "-L${PROJECT_ROOT}/target/${LIB_PATH} -Wl,-rpath ${PROJECT_ROOT}/target/${LIB_PATH}" qrmi
  )

  if (APPLE)
    target_link_libraries(${TARGET_NAME} PUBLIC "-framework Security")
    target_link_libraries(${TARGET_NAME} PUBLIC "-framework CoreFoundation")
    target_link_libraries(${TARGET_NAME} PUBLIC "-framework SystemConfiguration")
  elseif(UNIX)
    target_link_libraries(${TARGET_NAME} PUBLIC "-lssl")
    target_link_libraries(${TARGET_NAME} PUBLIC "-lcrypto")
    target_link_libraries(${TARGET_NAME} PUBLIC "-ldl")
    target_link_libraries(${TARGET_NAME} PUBLIC "-lpthread")
    target_link_libraries(${TARGET_NAME} PUBLIC "-lm")
  endif()

  # path to headerfile
  target_include_directories(${TARGET_NAME}
          PRIVATE
          ${CMAKE_CURRENT_SOURCE_DIR}/../../..
  )

  install (TARGETS ${TARGET_NAME} DESTINATION .)
endfunction()

add_example(direct_access src/direct_access.c)
