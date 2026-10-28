-- Eaten from the bag, with the same effect as when held.
return {
  id = "sitrus-berry", name = "Sitrus Berry", targets = "party",
  usable = function(target) return target.hp < target.max_hp end,
  script = function(ctx) ctx:receive_berry(ctx.target, "sitrus-berry") end,
}
