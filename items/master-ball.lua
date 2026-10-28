return {
  id = "master-ball", name = "Master Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, 1, {guaranteed = true}) end,
}
