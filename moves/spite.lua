-- Move 180. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "spite", name = "Spite", type = Type.Ghost,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local last = ctx.target.last_move
    if not last or ctx:reduce_pp(ctx.target, last, 4) == 0 then ctx:fail() end
  end,
}
