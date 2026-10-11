-- Move 579. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "flower-shield", name = "Flower Shield", type = Type.Fairy,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {"bypass_substitute"},
  script = function(ctx)
    local changed = false
    for _, pokemon in ipairs(ctx.everyone) do
      if pokemon:has_type(Type.Grass) and ctx:change_stat(pokemon, Stat.Defense, 1) ~= 0 then changed = true end
    end
    if not changed then ctx:fail() end
  end,
}
