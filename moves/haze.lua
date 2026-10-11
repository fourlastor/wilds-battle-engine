-- Move 114. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "haze", name = "Haze", type = Type.Ice,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.Field,
  flags = {"bypass_substitute", "pressure_all"},
  script = function(ctx)
    for _, pokemon in ipairs(ctx.everyone) do ctx:reset_stats(pokemon) end
  end,
}
