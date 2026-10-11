-- Move 220. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "pain-split", name = "Pain Split", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local hp = math.max(1, math.floor((ctx.user.hp + ctx.target.hp) / 2))
    ctx:set_hp(ctx.target, hp)
    ctx:set_hp(ctx.user, hp)
  end,
}
