return {
  id = "max-revive", name = "Max Revive", targets = "fainted",
  script = function(ctx)
    if ctx:revive(ctx.target, {fraction = 1}) then ctx:message("{0} was revived!", ctx.target) end
  end,
}
