-- Eaten from the bag, with the same effect as when held.
return {
  id = "cheri-berry", name = "Cheri Berry", targets = "party",
  usable = function(target) return target.status == Status.Paralyzed end,
  script = function(ctx) ctx:receive_berry(ctx.target, "cheri-berry") end,
}
