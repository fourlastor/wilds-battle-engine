-- Move 199. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "lock-on", name = "Lock-On", type = Type.Normal,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    if ctx.user.marks.lock_on then return ctx:fail() end
    if not ctx:try_hit(ctx.target) then return end
    ctx:start_effect(ctx.user, "lock_on", {turns = 2, rules = {{kind = Rule.AlwaysHit, against = ctx.target}}})
  end,
}
