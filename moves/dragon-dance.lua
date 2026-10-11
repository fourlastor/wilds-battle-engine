-- Move 349: Dragon Dance. Identity and parameters: pokemon_data/assets/moves/349.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1, [Stat.Speed] = 1}}
}}

move.id = "dragon-dance"
move.name = "Dragon Dance"
move.type = Type.Dragon
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "dance"}
return move
