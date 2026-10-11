-- Move 248. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  base_power = 120,
  id = "future-sight", name = "Future Sight", type = Type.Psychic,
  category = Category.Special, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"future_move"},
  script = function(ctx)
    if not ctx:schedule_attack(ctx.target, 120, 2) then ctx:fail() end
  end,
}
