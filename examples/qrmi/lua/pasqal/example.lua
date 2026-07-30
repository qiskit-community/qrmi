--  This code is part of Qiskit.
--
-- (C) Copyright IBM, Pasqal 2026
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

if #arg ~= 3 then
    print("Missing arguments\n")
    print("Usage: lua example.lua <backend name> <resource type> <input file>\n")
    os.exit(1)
end

-- Create a resource handle (corresponds to the real qrmi_resource_new)
local resource, err = qrmi.new(arg[1], arg[2])
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

local target, target_err = resource:target()
if not target then
    print("target failed:", target_err)
else
    print("target:", target)
end

local id, id_err = resource:id()
print("id:", id, id_err)
 
local rtype, rtype_err = resource:type()
print("type:", rtype, rtype_err)

local accessible, aerr = resource:is_accessible()
print("is_accessible:", accessible, aerr)

local token, tok_err = resource:acquire()
if not token then
    print("acquire failed:", tok_err)
    os.exit(1)
end
print("acquired, token =", token)

-- Read the task input payload from an external file, using its content as-is.
local payload_file = io.open(arg[3], "r")
if not payload_file then
    print("failed to open "  .. arg[3])
    os.exit(1)
end
local input_json = payload_file:read("*a")
payload_file:close()

print("input =", input_json)
local task_id, start_err = resource:task_start({
    pasqal_cloud = {
        sequence = input_json,
        job_runs = 100,
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
elseif not terminal_statuses[status] then
    print("warning: gave up after " .. max_polls .. " polls, last status = " .. status)
end

local result = resource:task_result(task_id)
print("result:", result)

local logs = resource:task_logs(task_id)
print("logs:", logs)

local ok, rel_err = resource:release()
print("release:", ok, rel_err)

resource:free()
print("resource freed")

-- Error case: unknown resource type
local bad, bad_err = qrmi.new("x", "not_a_real_type")
print("bad new ->", bad, bad_err)

-- Error case: calling a method after free() raises a Lua error
local ok2, err2 = pcall(function() resource:is_accessible() end)
print("call after free -> pcall ok:", ok2, "err:", err2)

-- ---------------------------------------------------------------------------
-- qrmi.config — completely independent from the `resource` object above.
-- ---------------------------------------------------------------------------
local config, config_err = qrmi.load_config("qrmi_config.json")
if not config then
    print("load_config failed:", config_err)
else
    print("config loaded")

    local def, def_err = config:resource_def("ibm_kingston")
    if not def then
        print("resource_def failed:", def_err)
    else
        print("resource_def:")
        print("  name       =", def.name)
        print("  type       =", def.type)
        print("  is_dynamic =", def.is_dynamic)
        print("  environments:")
        for k, v in pairs(def.environments) do
            print("    " .. k .. " = " .. v)
        end
    end

    config:free()
    print("config freed")

    -- error case: missing file
    local bad_config, bad_config_err = qrmi.load_config("/no/such/qrmi_config.json")
    print("load_config (missing file) ->", bad_config, bad_config_err)
end

