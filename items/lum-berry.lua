-- Eaten from the bag, with the same effect as when held.
return {
  id = "lum-berry", name = "Lum Berry", targets = "party",
  usable = function(target) return target.status ~= nil or target.confused end,
  script = function(ctx) ctx:receive_berry(ctx.target, "lum-berry") end,
}
