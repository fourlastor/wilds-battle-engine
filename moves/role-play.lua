-- Move 272. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "role-play", name = "Role Play", type = Type.Psychic,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    local user, target = ctx.user, ctx.target
    if not target.ability_id or target.ability_id == user.ability_id
        or target.ability_flags.failroleplay or user.ability_flags.cantsuppress
        then ctx:fail(); return end
    if not ctx:try_hit(target, {ignore_protect=true}) then return end
    if not ctx:copy_ability(user, target) then ctx:fail() end
  end,
}
