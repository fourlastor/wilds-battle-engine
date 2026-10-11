-- Move 791: identity and parameters from pokemon_data/assets/moves/791.json.
return {
  id = "life-dew", name = "Life Dew",
  type = Type.Water, category = Category.Status,
  pp = 10, accuracy = Accuracy.Always, priority = 0, target = Target.UserAndAllies,
  flags = {"heal", "bypass_substitute"},
  script = function(ctx)
    local healed = false
    for _, pokemon in ipairs(ctx.targets) do
      if ctx:try_hit(pokemon, {ignore_protect = true}) then
        if ctx:heal(pokemon, math.max(1, math.floor(pokemon.max_hp / 4))) > 0 then healed = true end
      end
    end
    if not healed then return ctx:fail() end
  end,
}
