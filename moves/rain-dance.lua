-- Move 240. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "rain-dance", name = "Rain Dance", type = Type.Water,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {},
  script = function(ctx)
    if not ctx:set_weather(Weather.Rain, 5) then ctx:fail() end
  end,
}
