-- Eaten from the bag, with the same effect as when held.
return {
  id = "rawst-berry", name = "Rawst Berry", targets = "party",
  usable = function(target) return target.status == Status.Burned end,
  script = function(ctx) ctx:receive_berry(ctx.target, "rawst-berry") end,
}
