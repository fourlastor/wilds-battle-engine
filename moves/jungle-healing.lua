-- Move 816. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "jungle-healing", name = "Jungle Healing", type = Type.Grass,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.UserAndAllies,
  flags = {"bypass_substitute", "heal"},
  script = function(ctx)
    local helped = false
    for _, pokemon in ipairs(ctx.targets) do
      if ctx:try_hit(pokemon, {ignore_protect = true}) then
        if ctx:heal(pokemon, math.max(1, math.floor(pokemon.max_hp / 4))) > 0 then helped = true end
        if ctx:cure_status(pokemon) then helped = true end
      end
    end
    if not helped then ctx:fail() end
  end,
}
