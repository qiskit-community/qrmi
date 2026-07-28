package.cpath = package.cpath .. ";./?.so"
local qrmi = require("qrmi")

-- Create a resource handle (corresponds to the real qrmi_resource_new)
local resource, err = qrmi.new("ibm_kingston", "qiskit-runtime-service")
if not resource then
    print("new failed:", err)
    os.exit(1)
end
print("resource created")

local accessible, aerr = resource:is_accessible()
print("is_accessible:", accessible, aerr)

local token, tok_err = resource:acquire()
if not token then
    print("acquire failed:", tok_err)
    os.exit(1)
end
print("acquired, token =", token)

-- Read the task input payload from an external file, using its content as-is.
local payload_file = io.open("task_payload.txt", "r")
if not payload_file then
    print("failed to open task_payload.txt")
    os.exit(1)
end
local input_json = payload_file:read("*a")
payload_file:close()

local task_id, start_err = resource:task_start({
    qiskit_primitive = {
        program_id = "estimator",
        input = input_json,
    }
})
if not task_id then
    print("task_start failed:", start_err)
    os.exit(1)
end
print("task started, id =", task_id)


-- Poll until the task reaches a terminal status (completed/failed/cancelled).
-- max_polls is just a safety net against an infinite loop if something is
-- stuck (e.g. a stale/unreachable resource); it is not a normal exit path.
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

