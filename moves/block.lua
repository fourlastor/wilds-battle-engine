-- Move 335. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "block", name = "Block", type = Type.Normal,
  category = Category.Status, pp = 5, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    if ctx.target.linked_trap or not ctx:start_effect(ctx.target, "trapped", {rules = {{kind = Rule.Trapped, by = ctx.user}}}) then ctx:fail() end
  end,
}
