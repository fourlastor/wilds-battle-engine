-- Move 378. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "wring-out", name = "Wring Out", type = Type.Normal,
  category = Category.Special, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local hp_ratio = math.floor(ctx.target.hp * 4096 / ctx.target.max_hp)
    local power = math.max(1, math.floor(math.floor((120 * 100 * hp_ratio + 2047) / 4096) / 100))
    ctx:damage(power)
  end,
}
