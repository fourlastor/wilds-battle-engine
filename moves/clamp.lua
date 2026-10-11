-- Move 128. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "clamp", name = "Clamp", type = Type.Water,
  category = Category.Physical, pp = 15, accuracy = 0.85,
  priority = 0, target = Target.AnyOther,
  flags = {"contact"},
  script = function(ctx)
    local hit = ctx:damage(35)
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
