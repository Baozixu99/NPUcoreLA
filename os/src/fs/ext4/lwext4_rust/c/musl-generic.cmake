if(NOT DEFINED ENV{ARCH})
    set(ARCH "x86_64")
else()
    set(ARCH $ENV{ARCH})
endif()

# Name of the target
set(CMAKE_SYSTEM_NAME "Linux")
set(CMAKE_SYSTEM_PROCESSOR ${ARCH})

# Toolchain settings
set(TOOLCHAIN_PREFIX ${ARCH}-linux-musl)

set(CMAKE_C_COMPILER    ${TOOLCHAIN_PREFIX}-cc)
set(CMAKE_CXX_COMPILER  ${TOOLCHAIN_PREFIX}-c++)
set(AS                  ${TOOLCHAIN_PREFIX}-as)
set(AR                  ${TOOLCHAIN_PREFIX}-ar)
set(OBJCOPY             ${TOOLCHAIN_PREFIX}-objcopy)
set(OBJDUMP             ${TOOLCHAIN_PREFIX}-objdump)
set(SIZE                ${TOOLCHAIN_PREFIX}-size)

set(LD_FLAGS "-nolibc -nostdlib -static --gc-sections -nostartfiles")

set(CMAKE_C_FLAGS   "-std=gnu99 -fdata-sections -ffunction-sections" CACHE INTERNAL "c compiler flags")
set(CMAKE_CXX_FLAGS "-fdata-sections -ffunction-sections" CACHE INTERNAL "cxx compiler flags")
set(CMAKE_ASM_FLAGS "" CACHE INTERNAL "asm compiler flags")

# set(CMAKE_PASS_LIB_FLAGS " -I/home/os/rust/arceos/ulib/axlibc/include ")
# set(CMAKE_C_FLAGS "-nostdinc -fno-builtin -ffreestanding ${CMAKE_PASS_LIB_FLAGS} ${CMAKE_C_FLAGS}")
set(CMAKE_C_FLAGS "-fPIC -fno-builtin -ffreestanding ${CMAKE_C_FLAGS}")
set(CMAKE_CXX_FLAGS "-fPIC -nostdinc -fno-builtin -ffreestanding ${CMAKE_CXX_FLAGS}")

if (APPLE)
    set(CMAKE_EXE_LINKER_FLAGS "-dead_strip" CACHE INTERNAL "exe link flags")
else (APPLE)
    set(CMAKE_EXE_LINKER_FLAGS "-Wl,--gc-sections" CACHE INTERNAL "exe link flags")
endif (APPLE)

SET(CMAKE_C_FLAGS_DEBUG "-O0 -g -ggdb3" CACHE INTERNAL "c debug compiler flags")
SET(CMAKE_CXX_FLAGS_DEBUG "-O0 -g -ggdb3" CACHE INTERNAL "cxx debug compiler flags")
SET(CMAKE_ASM_FLAGS_DEBUG "-g -ggdb3" CACHE INTERNAL "asm debug compiler flags")

SET(CMAKE_C_FLAGS_RELEASE "-O2 -g -ggdb3" CACHE INTERNAL "c release compiler flags")
SET(CMAKE_CXX_FLAGS_RELEASE "-O2 -g -ggdb3" CACHE INTERNAL "cxx release compiler flags")
SET(CMAKE_ASM_FLAGS_RELEASE "" CACHE INTERNAL "asm release compiler flags")
