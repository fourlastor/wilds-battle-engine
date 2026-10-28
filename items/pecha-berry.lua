-- Eaten from the bag, with the same effect as when held.
return {
  id = "pecha-berry", name = "Pecha Berry", targets = "party",
  usable = function(target) return target.status == Status.Poisoned or target.status == Status.BadlyPoisoned end,
  script = function(ctx) ctx:receive_berry(ctx.target, "pecha-berry") end,
}
