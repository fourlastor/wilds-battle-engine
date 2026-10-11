-- Move 46. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "roar", name = "Roar", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = -6, target = Target.AnyOther,
  flags = {"sound", "bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    if not ctx:force_switch(ctx.target) then ctx:fail() end
  end,
}
