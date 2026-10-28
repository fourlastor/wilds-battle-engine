-- Eaten from the bag, with the same effect as when held.
return {
  id = "aspear-berry", name = "Aspear Berry", targets = "party",
  usable = function(target) return target.status == Status.Frozen end,
  script = function(ctx) ctx:receive_berry(ctx.target, "aspear-berry") end,
}
