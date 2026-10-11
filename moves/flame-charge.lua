-- Move 488: Flame Charge. Identity and parameters: pokemon_data/assets/moves/488.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Speed] = 1}}
}}

move.id = "flame-charge"
move.name = "Flame Charge"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
