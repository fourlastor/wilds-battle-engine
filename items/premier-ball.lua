return {
  id = "premier-ball", name = "Premier Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, 1) end,
}
