-- Move 602: Magnetic Flux. Identity and parameters: pokemon_data/assets/moves/602.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    local changed = false
    for _, pokemon in ipairs(ctx.targets) do
      if pokemon.ability == "plus" or pokemon.ability == "minus" then
        for _, stat in ipairs({Stat.Defense, Stat.SpDefense}) do changed = ctx:change_stat(pokemon, stat, 1) ~= 0 or changed end
      end
    end
    if not changed then return ctx:fail() end
  end,
}

move.id = "magnetic-flux"
move.name = "Magnetic Flux"
move.type = Type.Electric
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.UserAndAllies
move.flags = {"snatch", "bypass_substitute"}
return move
