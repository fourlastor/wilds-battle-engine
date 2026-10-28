return {
  id = "full-restore", name = "Full Restore", targets = "party",
  usable = function(target) return target.hp < target.max_hp or target.status ~= nil or target.confused end,
  script = function(ctx)
    local healed = ctx:heal(ctx.target, {fraction = 1})
    local cured = ctx:cure_status(ctx.target)
    if ctx:cure_confusion(ctx.target) then cured = true end
    if cured then ctx:message("{0} became healthy.", ctx.target) end
    if healed > 0 then ctx:message("{0}'s HP was restored.", ctx.target) end
  end,
}
