local M = {}

function M.selected_uri_list()
	local paths = {}
	for _, u in pairs(cx.active.selected) do
		paths[#paths + 1] = "file://" .. ya.percent_encode(tostring(u.path))
	end
	if #paths == 0 and cx.active.current.hovered then
		paths[1] = "file://" .. ya.percent_encode(tostring(cx.active.current.hovered.path))
	end
	return paths
end

function M.copy_uri_list()
	local list = M.selected_uri_list()
	if #list == 0 then
		return false
	end

	rt.tty:queue("WriteClipboard", { mime = "text/uri-list", data = table.concat(list, "\r\n") })
	rt.tty:flush()
	return true
end

return M
