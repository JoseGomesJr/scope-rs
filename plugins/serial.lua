---
--- Generated by EmmyLua(https://github.com/EmmyLua)
--- Created by matheuswhite.
--- DateTime: 06/04/24 20:54
---
require "scope"

function serial_rx(msg)
end

function user_command(arg_list)
    if arg_list[1] == 'connect' or arg_list[1] == 'reconnect' then
        scope.connect(arg_list[2], arg_list[3])
    elseif arg_list[1] == 'disconnect' then
        scope.disconnect()
    elseif #arg_list == 0 or arg_list == nil then
        scope.eprintln('Invalid command. Enter a "connect" or "disconnect" command')
    else
        scope.eprintln('Invalid command "' .. arg_list[1] .. '".')
    end
end
