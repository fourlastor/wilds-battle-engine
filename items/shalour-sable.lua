return {
  id = "shalour-sable", name = "Shalour Sable", targets = "party",
  usable = function(target) return target.status ~= nil or target.confused end,
  script = function(ctx)
    local cured = ctx:cure_status(ctx.target)
    if ctx:cure_confusion(ctx.target) then cured = true end
    if cured then ctx:message("{0} became healthy.", ctx.target) end
  end,
}
