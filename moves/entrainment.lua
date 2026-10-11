-- Move 494. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "entrainment", name = "Entrainment", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    local user, target = ctx.user, ctx.target
    if user == target or not user.ability_id or target.ability_id == user.ability_id
        or target.ability_flags.cantsuppress or target.ability_id == "truant"
        or user.ability_flags.noentrain then ctx:fail(); return end
    if not ctx:try_hit(target) then return end
    if not ctx:copy_ability(target, user) then ctx:fail() end
  end,
}
