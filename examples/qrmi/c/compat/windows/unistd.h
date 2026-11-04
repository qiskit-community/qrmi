/*
 * Minimal POSIX shim so the QRMI C examples build with MSVC on Windows.
 *
 * This file is only on the include path (and force-included via /FI) for
 * MSVC builds; see the `if (MSVC)` block in each example's CMakeLists.txt.
 * It provides just what the examples use: sleep(), setenv(), strtok_r().
 * <windows.h> is deliberately not included to avoid its macros clashing
 * with names in qrmi.h.
 */
#ifndef QRMI_EXAMPLES_COMPAT_UNISTD_H
#define QRMI_EXAMPLES_COMPAT_UNISTD_H

#ifdef _WIN32

#include <stdlib.h>
#include <string.h>

__declspec(dllimport) void __stdcall Sleep(unsigned long dwMilliseconds);

static __inline unsigned int sleep(unsigned int seconds) {
    Sleep((unsigned long)seconds * 1000UL);
    return 0;
}

static __inline int setenv(const char *name, const char *value, int overwrite) {
    if (!overwrite && getenv(name) != NULL) {
        return 0;
    }
    return _putenv_s(name, value) == 0 ? 0 : -1;
}

#define strtok_r strtok_s

#endif /* _WIN32 */

#endif /* QRMI_EXAMPLES_COMPAT_UNISTD_H */
