-- Move 35. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  base_power = 15,
  id = "wrap", name = "Wrap", type = Type.Normal,
  category = Category.Physical, pp = 20, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local hit = ctx:damage(15)
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
