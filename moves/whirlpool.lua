-- Move 250. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "whirlpool", name = "Whirlpool", type = Type.Water,
  category = Category.Special, pp = 15, accuracy = 0.85,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    local underwater = ctx.target.hidden == Hidden.Underwater
    local hit = ctx:damage(underwater and 70 or 35, {hits_hidden = {Hidden.Underwater}})
    if hit.hit and not hit.substitute then ctx:bind(ctx.target, {min_turns = 4, max_turns = 5}) end
  end,
}
