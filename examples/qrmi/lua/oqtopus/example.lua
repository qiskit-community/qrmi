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

if #arg ~= 1 then
    print("Missing arguments\n")
    print("Usage: lua example.lua <device_id>\n")
    os.exit(1)
end

-- Create a resource handle (corresponds to the real qrmi_resource_new)
local resource, err = qrmi.new(arg[1], "oqtopus")
if not resource then
    print("new failed:", err)
    os.exit(1)
end
print("resource created")

local id, id_err = resource:id()
print("id:", id, id_err)
 
local rtype, rtype_err = resource:type()
print("type:", rtype, rtype_err)

local accessible, aerr = resource:is_accessible()
print("is_accessible:", accessible, aerr)

resource:free()
print("resource freed")
