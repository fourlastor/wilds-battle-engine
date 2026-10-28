return {
  id = "paralyze-heal", name = "Paralyze Heal", targets = "party",
  usable = function(target) return target.status == Status.Paralyzed end,
  script = function(ctx)
    local cured = false
    if ctx:cure_status(ctx.target, Status.Paralyzed) then cured = true end
    if cured then ctx:message("{0} was cured of paralysis.", ctx.target) end
  end,
}
