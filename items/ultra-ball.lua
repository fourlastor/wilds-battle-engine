return {
  id = "ultra-ball", name = "Ultra Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, 2) end,
}
