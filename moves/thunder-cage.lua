-- Move 819. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "thunder-cage", name = "Thunder Cage", type = Type.Electric,
  category = Category.Special, pp = 15, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    local hit = ctx:damage(80)
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
