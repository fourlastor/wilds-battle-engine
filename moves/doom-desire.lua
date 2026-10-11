-- Move 353. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "doom-desire", name = "Doom Desire", type = Type.Steel,
  category = Category.Special, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"future_move"},
  script = function(ctx)
    if not ctx:schedule_attack(ctx.target, 140, 2) then ctx:fail() end
  end,
}
