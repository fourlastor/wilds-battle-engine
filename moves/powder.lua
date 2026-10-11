-- Move 600. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "powder", name = "Powder", type = Type.Bug,
  category = Category.Status, pp = 20, accuracy = 1.0,
  priority = 1, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable", "powder"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    if not ctx:start_effect(ctx.target, "powder", {turns = 1, rules = {
      {kind = Rule.ExplodeOnType, type = Type.Fire, fraction = 1/4, message = "The powder around {name} exploded!"}
    }}) then ctx:fail() end
  end,
}
