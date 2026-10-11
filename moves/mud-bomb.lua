-- Move 426: Mud Bomb. Identity and parameters: pokemon_data/assets/moves/426.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 0.3, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "mud-bomb"
move.name = "Mud Bomb"
move.type = Type.Ground
move.category = Category.Special
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet", "sheer_force"}
return move
