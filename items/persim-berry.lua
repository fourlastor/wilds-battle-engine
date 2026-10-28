-- Eaten from the bag, with the same effect as when held.
return {
  id = "persim-berry", name = "Persim Berry", targets = "party",
  usable = function(target) return target.confused end,
  script = function(ctx) ctx:receive_berry(ctx.target, "persim-berry") end,
}
