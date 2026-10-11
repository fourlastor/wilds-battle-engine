-- Move 178: Cotton Spore. Identity and parameters: pokemon_data/assets/moves/178.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    for _, pokemon in ipairs(ctx.targets) do
      if not pokemon:has_type(Type.Grass) and ctx:reached(pokemon) then ctx:change_stat(pokemon, Stat.Speed, -2) end
    end
  end,
}

move.id = "cotton-spore"
move.name = "Cotton Spore"
move.type = Type.Grass
move.category = Category.Status
move.pp = 40
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"reflectable", "powder"}
return move
