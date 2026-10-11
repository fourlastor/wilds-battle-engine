-- Move 611. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  base_power = 20,
  id = "infestation", name = "Infestation", type = Type.Bug,
  category = Category.Special, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local hit = ctx:damage(20)
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
