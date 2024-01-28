---
--- Generated by EmmyLua(https://github.com/EmmyLua)
--- Created by matheuswhite.
--- DateTime: 24/01/24 09:45
---

require "scope"

function serial_rx(msg)
    msg_str = bytes2str(msg)

    if msg_str ~= "AT\r\n" then
        return
    end

    scope.println("Sending msg \"OK\" via serial tx...")
    scope.serial_tx(str2bytes("OK\r\n"))
    scope.println("Message sent!")
end

function user_command(arg_list)
    scope.println("Hello! This is your echo:")
    scope.println(arg_list[1])
end
