-- Move 258. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "hail", name = "Hail", type = Type.Ice,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {},
  script = function(ctx)
    if not ctx:set_weather(Weather.Hail, 5) then ctx:fail() end
  end,
}
