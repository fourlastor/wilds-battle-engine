return {
  id = "ice-heal", name = "Ice Heal", targets = "party",
  usable = function(target) return target.status == Status.Frozen end,
  script = function(ctx)
    local cured = false
    if ctx:cure_status(ctx.target, Status.Frozen) then cured = true end
    if cured then ctx:message("{0} thawed out!", ctx.target) end
  end,
}
