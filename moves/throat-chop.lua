-- Move 675. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "throat-chop", name = "Throat Chop", type = Type.Dark,
  category = Category.Physical, pp = 15, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"contact", "sheer_force"},
  script = function(ctx)
    local hit = ctx:damage(80)
    if hit.hit and not hit.substitute and not ctx.target.fainted then
      ctx:start_effect(ctx.target, "throat_chop", {turns = 2, rules = {{kind = Rule.BlockMoves, flags = {"sound"}, message = "{name} cannot use sound moves!"}}})
    end
  end,
}
