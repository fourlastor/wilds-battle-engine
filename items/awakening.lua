return {
  id = "awakening", name = "Awakening", targets = "party",
  usable = function(target) return target.status == Status.Asleep end,
  script = function(ctx)
    local cured = false
    if ctx:cure_status(ctx.target, Status.Asleep) then cured = true end
    if cured then ctx:message("{0} woke up!", ctx.target) end
  end,
}
