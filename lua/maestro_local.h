/*  This code is part of Qiskit.
 *
 * (C) Copyright IBM 2026
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
/* Maestro payload adapter extracted from lua_qrmi.c. */

#ifndef QRMI_LUA_MAESTRO_LOCAL_H
#define QRMI_LUA_MAESTRO_LOCAL_H

#include "qrmi.h"
#include <lua.h>

/* Private adapter for a maestro_local payload table. The strings stay on the
 * Lua stack until submission completes; the table and its fields are then
 * removed. The caller handles the return code and owns any returned task ID. */
QrmiReturnCode qrmi_lua_submit_maestro_payload(lua_State *L,
                                               QrmiQuantumResource *resource,
                                               int variant_idx, char **task_id);

#endif
