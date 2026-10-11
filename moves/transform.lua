-- Move 144. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "transform", name = "Transform", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect=true}) then return end
    if not ctx:transform(ctx.user, ctx.target) then ctx:fail() end
  end,
}
