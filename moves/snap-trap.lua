-- Move 779. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  base_power = 35,
  id = "snap-trap", name = "Snap Trap", type = Type.Grass,
  category = Category.Physical, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local hit = ctx:damage(35)
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
