return {
  id = "burn-heal", name = "Burn Heal", targets = "party",
  usable = function(target) return target.status == Status.Burned end,
  script = function(ctx)
    local cured = false
    if ctx:cure_status(ctx.target, Status.Burned) then cured = true end
    if cured then ctx:message("{0}'s burn was healed.", ctx.target) end
  end,
}
