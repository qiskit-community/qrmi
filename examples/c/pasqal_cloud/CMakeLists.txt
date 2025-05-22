cmake_minimum_required(VERSION 3.10)
set(CMAKE_CXX_STANDARD 17)
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

add_example(pasqal_cloud src/pasqal_cloud.c)
