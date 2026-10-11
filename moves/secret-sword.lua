-- Move 548: Secret Sword. Identity and parameters: pokemon_data/assets/moves/548.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  ctx:damage(85, {defense_stat = Stat.Defense})
end}

move.id = "secret-sword"
move.name = "Secret Sword"
move.type = Type.Fighting
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"slicing"}
return move
