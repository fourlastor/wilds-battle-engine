-- Move 373. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "embargo", name = "Embargo", type = Type.Dark,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:start_effect(ctx.target, "embargo", {turns=5, rules={{kind=Rule.BlockItems}}}) then ctx:fail() end
  end,
}
