return {
  id = "antidote", name = "Antidote", targets = "party",
  usable = function(target) return target.status == Status.Poisoned or target.status == Status.BadlyPoisoned end,
  script = function(ctx)
    local cured = false
    if ctx:cure_status(ctx.target, Status.Poisoned) then cured = true end
    if ctx:cure_status(ctx.target, Status.BadlyPoisoned) then cured = true end
    if cured then ctx:message("{0} was cured of its poisoning.", ctx.target) end
  end,
}
