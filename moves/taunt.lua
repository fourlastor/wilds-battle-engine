-- Move 269. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "taunt", name = "Taunt", type = Type.Dark,
  category = Category.Status, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local ability = ctx.target.ability and ctx.target.ability:gsub("[-_]", "")
    if ability == "oblivious" or ctx.target.effects.taunt then ctx:fail(); return end
    if not ctx:start_effect(ctx.target, "taunt", {turns = ctx.target.acted and 4 or 3, rules = {
      {kind = Rule.BlockCategory, category = Category.Status, except = {"me-first"}, message = "{0} cannot use status moves after the taunt!"}
    }}) then ctx:fail() end
  end,
}
