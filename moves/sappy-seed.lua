-- Move 738. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  base_power = 100,
  id = "sappy-seed", name = "Sappy Seed", type = Type.Grass,
  category = Category.Physical, pp = 10, accuracy = 0.9,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local hit = ctx:damage(100)
    if hit.hit and not hit.substitute and not ctx.target.fainted and not ctx.target:has_type(Type.Grass) then
      ctx:start_effect(ctx.target, "leech_seed", {rules = {{kind = Rule.DrainEachTurn, fraction = 1/8, to = ctx.user}}})
    end
  end,
}
