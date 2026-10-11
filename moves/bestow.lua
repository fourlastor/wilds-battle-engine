-- Move 516. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "bestow", name = "Bestow", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    if ctx.target.item or not ctx.user.item then ctx:fail(); return end
    local item = ctx:take_item(ctx.user)
    if not item then ctx:fail(); return end
    if not ctx:give_item(ctx.target, item) then
      ctx:give_item(ctx.user, item)
      ctx:fail()
    end
  end,
}
