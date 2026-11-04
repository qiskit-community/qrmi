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

#include "maestro_local.h"
#include <lauxlib.h>

QrmiReturnCode qrmi_lua_submit_maestro_payload(lua_State *L,
                                               QrmiQuantumResource *resource,
                                               int variant_idx,
                                               char **task_id) {
    lua_getfield(L, variant_idx, "input");
    const char *input = luaL_checkstring(L, -1);
    lua_getfield(L, variant_idx, "job_type");
    const char *job_type = luaL_optstring(L, -1, "execute");
    lua_getfield(L, variant_idx, "qubits");
    lua_Integer qubits = luaL_optinteger(L, -1, 0);
    lua_getfield(L, variant_idx, "simulator_type");
    lua_Integer simulator_type = luaL_optinteger(L, -1, 0);
    lua_getfield(L, variant_idx, "simulation_method");
    lua_Integer simulation_method = luaL_optinteger(L, -1, 0);
    lua_getfield(L, variant_idx, "observables");
    const char *observables = luaL_optstring(L, -1, "");
    lua_getfield(L, variant_idx, "config");
    const char *config = luaL_optstring(L, -1, "{}");

    QrmiPayload payload;
    payload.tag = QRMI_PAYLOAD_MAESTRO_LOCAL;
    payload.MAESTRO_LOCAL.input = (char *)input;
    payload.MAESTRO_LOCAL.job_type = (char *)job_type;
    payload.MAESTRO_LOCAL.qubits = (uint32_t)qubits;
    payload.MAESTRO_LOCAL.simulator_type = (uint32_t)simulator_type;
    payload.MAESTRO_LOCAL.simulation_method = (uint32_t)simulation_method;
    payload.MAESTRO_LOCAL.observables = (char *)observables;
    payload.MAESTRO_LOCAL.config = (char *)config;

    QrmiReturnCode rc = qrmi_resource_task_start(resource, &payload, task_id);
    lua_settop(L, variant_idx - 1);
    return rc;
}
