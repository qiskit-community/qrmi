--  This code is part of Qiskit.
--
-- (C) Copyright IBM 2026
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
    print("Usage: lua example.lua <device_id> <job_spec file> <job_type('sampling','estimation', 'multi_manual' or 'sse')>\n")
    os.exit(1)
end

-- Create a resource handle (corresponds to the real qrmi_resource_new)
local resource, err = qrmi.new(arg[1], "oqtopus")
if not resource then
    print("new failed:", err)
    os.exit(1)
end
print("resource created")

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

local rstatus, rstatus_err = resource:status()
if not rstatus then
    print("status failed:", rstatus_err)
else
    print("status:")
    print("  status            =", rstatus.status)
    print("  status_reason     =", rstatus.status_reason)
    print("  healthy           =", rstatus.healthy)
    print("  busy              =", rstatus.busy)
    print("  pending_job_count =", rstatus.pending_job_count)
    if rstatus.capacity then
        print("  capacity.available_slots =", rstatus.capacity.available_slots)
        print("  capacity.max_slots       =", rstatus.capacity.max_slots)
    else
        print("  capacity          = nil")
    end
end

-- Read the task input payload from an external file, using its content as-is.
local payload_file = io.open(arg[2], "r")
if not payload_file then
    print("failed to open "  .. arg[2])
    os.exit(1)
end
local job_spec = payload_file:read("*a")
payload_file:close()

print("job_spec =", job_spec)
local task_id, start_err = resource:task_start({
    oqtopus = {
        job_spec = job_spec,
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

resource:free()
print("resource freed")
