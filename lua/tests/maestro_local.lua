-- SPDX-License-Identifier: Apache-2.0
-- Run by the scripted socket tests in python/tests/unit/quantum_resource.
local loader = assert(package.loadlib(os.getenv("QRMI_TEST_LUA_MODULE"), "luaopen_qrmi"))
local qrmi = loader()
local resource = assert(qrmi.new("py_maestro", "maestro-local"))
assert(resource:acquire() == "8")
local payload = { input = "OPENQASM 2.0; qreg q[1];", qubits = 1 }
if arg[1] == "estimate" then
    payload.job_type = "estimate"
    payload.observables = "Z"
    payload.simulator_type = 2
    payload.simulation_method = 3
    payload.config = '{"shots":8}'
end
local task, err = resource:task_start({ maestro_local = payload })
assert(task == "1", err)
local rejected, reason = resource:task_start({
    qiskit_primitive = { input = "{}", program_id = "sampler" }
})
assert(rejected == nil and reason:find("payload type is not supported", 1, true))
-- Legacy 32-bit fields are range-checked, never wrapped (4294967297 is not 1).
for field, value in pairs({ qubits = 4294967297, simulator_type = -1, simulation_method = 4294967296 }) do
    local saved = payload[field]
    payload[field] = value
    local ok, message = pcall(resource.task_start, resource, { maestro_local = payload })
    assert(not ok and message:find(field .. " must be an integer in 0..4294967295", 1, true), message)
    payload[field] = saved
end
payload.input = string.char(255)
rejected, reason = resource:task_start({ maestro_local = payload })
assert(rejected == nil and reason:find("UTF-8", 1, true))
assert(resource:detach() == "8")
assert(resource:detach() == nil)
resource:free()
local other = assert(qrmi.new("py_maestro", "maestro-local"))
assert(other:release("8"))
other:free()
