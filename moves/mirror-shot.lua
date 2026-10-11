-- Move 429: Mirror Shot. Identity and parameters: pokemon_data/assets/moves/429.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 0.3, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "mirror-shot"
move.name = "Mirror Shot"
move.type = Type.Steel
move.category = Category.Special
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
