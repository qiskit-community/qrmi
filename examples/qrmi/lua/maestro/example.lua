--  This code is part of Qiskit.
--
-- (C) Copyright IBM, Qoro 2026
--
-- This code is licensed under the Apache License, Version 2.0. You may
-- obtain a copy of this license in the LICENSE.txt file in the root directory
-- of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
--
-- Any modifications or derivative works of this code must retain this
-- copyright notice, and modified files need to carry a notice indicating
-- that they have been altered from the originals.
--
package.cpath = package.cpath .. ";./?.so"
local qrmi = require("qrmi")

if #arg < 2 then
    print("Missing arguments\n")
    print("Usage: lua example.lua <backend name> <QASM input file> [job_type('execute' or 'estimate'), default 'execute'] [qubits, default 5] [observables (pauli strings separated by ';'), default '']\n")
    os.exit(1)
end

local backend_name = arg[1]
local input_file = arg[2]
local job_type = arg[3] or "execute"
local qubits = tonumber(arg[4] or "5")
local observables = arg[5] or ""

-- Create a resource handle (corresponds to the real qrmi_resource_new)
local resource, err = qrmi.new(backend_name, "maestro-local")
if not resource then
    print("new failed:", err)
    os.exit(1)
end
print("resource created")

local meta, meta_err = resource:metadata()
if not meta then
    print("metadata failed:", meta_err)
else
    print("metadata:")
    for k, v in pairs(meta) do
        print("  " .. k .. " = " .. tostring(v))
    end
end

local id, id_err = resource:id()
print("id:", id, id_err)

local rtype, rtype_err = resource:type()
print("type:", rtype, rtype_err)

local accessible, aerr = resource:is_accessible()
print("is_accessible:", accessible, aerr)

-- Note: the acquired session is held in memory by the resource handle for
-- the lifetime of this process, so subsequent task_start()/release() calls
-- on the same `resource` don't need the token to be re-supplied. If a
-- session needs to be reused from a *different* process, export it as
-- `<backend_name>_QRMI_JOB_ACQUISITION_TOKEN`.
local token, tok_err = resource:acquire()
if not token then
    print("acquire failed:", tok_err)
    os.exit(1)
end
print("acquired, token =", token)

local target, target_err = resource:target()
if not target then
    print("target failed:", target_err)
else
    print("target:", target)
end

-- Read the QASM input payload from an external file, using its content as-is.
local payload_file = io.open(input_file, "r")
if not payload_file then
    print("failed to open "  .. input_file)
    os.exit(1)
end
local qasm_input = payload_file:read("*a")
payload_file:close()

local task_id, start_err = resource:task_start({
    maestro_local = {
        input = qasm_input,
        job_type = job_type,
        qubits = qubits,
        simulator_type = 0,
        simulation_method = 0,
        observables = observables,
        config = "{}",
    }
})
if not task_id then
    print("task_start failed:", start_err)
    os.exit(1)
end
print("task started, id =", task_id)

-- Poll until the task reaches a terminal status (completed/failed/cancelled).
local terminal_statuses = { completed = true, failed = true, cancelled = true }
local status, status_err = resource:task_status(task_id)
print("status = " .. tostring(status))

while status and not terminal_statuses[status] do
    os.execute("sleep 1")
    status, status_err = resource:task_status(task_id)
    print("status = " .. tostring(status))
end

if not status then
    print("task_status failed:", status_err)
elseif status == "completed" then
    local result = resource:task_result(task_id)
    print("result:", result)
else
    local logs, logs_err = resource:task_logs(task_id)
    if logs then
        print("logs:", logs)
    else
        print("logs unavailable:", logs_err)
    end
end

resource:task_stop(task_id)

local ok, rel_err = resource:release()
print("release:", ok, rel_err)

resource:free()
print("resource freed")
