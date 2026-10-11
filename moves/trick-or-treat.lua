-- Move 567. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "trick-or-treat", name = "Trick-or-Treat", type = Type.Ghost,
  category = Category.Status, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if ctx.target:has_type(Type.Ghost) then ctx:fail(); return end
    ctx:add_type(ctx.target, Type.Ghost)
  end,
}
