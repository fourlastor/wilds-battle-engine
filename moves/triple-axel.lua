-- Move 813: Triple Axel. Identity and parameters: pokemon_data/assets/moves/813.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
for hit = 1, 3 do
  if not ctx:damage(20 * hit, {accuracy = 0.9}).hit or ctx.target.fainted then break end
end
end}

move.id = "triple-axel"
move.name = "Triple Axel"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
