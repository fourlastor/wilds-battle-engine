-- Eaten from the bag, with the same effect as when held.
return {
  id = "chesto-berry", name = "Chesto Berry", targets = "party",
  usable = function(target) return target.status == Status.Asleep end,
  script = function(ctx) ctx:receive_berry(ctx.target, "chesto-berry") end,
}
