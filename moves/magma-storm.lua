-- Move 463. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  base_power = 100,
  id = "magma-storm", name = "Magma Storm", type = Type.Fire,
  category = Category.Special, pp = 5, accuracy = 0.75,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    local hit = ctx:damage(100)
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
