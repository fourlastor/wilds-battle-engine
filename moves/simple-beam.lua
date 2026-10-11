-- Move 493. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "simple-beam", name = "Simple Beam", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    local target = ctx.target
    if target.ability_flags.cantsuppress or target.ability_id == "truant"
        or target.ability_id == "simple" then ctx:fail(); return end
    if not ctx:try_hit(target) then return end
    if not ctx:set_ability(target, "simple") then ctx:fail() end
  end,
}
