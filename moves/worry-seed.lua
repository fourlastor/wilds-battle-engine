-- Move 388. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "worry-seed", name = "Worry Seed", type = Type.Grass,
  category = Category.Status, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    local target = ctx.target
    if target.ability_flags.cantsuppress or target.ability_id == "truant"
        or target.ability_id == "insomnia" then ctx:fail(); return end
    if not ctx:try_hit(target) then return end
    if not ctx:set_ability(target, "insomnia") then ctx:fail(); return end
    ctx:cure_status(target, Status.Asleep)
  end,
}
