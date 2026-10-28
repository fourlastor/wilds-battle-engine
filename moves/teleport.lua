-- Move 100. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "teleport", name = "Teleport", type = Type.Psychic,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = -6, target = Target.User,
  flags = {},
  script = function(ctx)
    -- Gen 8: the user leaves a wild battle, and switches out of a trainer battle.
    if ctx.wild_battle then
      if not ctx:flee() then ctx:fail() end
    elseif not ctx:switch_out() then
      ctx:fail()
    end
  end,
}
