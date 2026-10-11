-- Move 557: V-create. Identity and parameters: pokemon_data/assets/moves/557.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 180},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Defense] = -1, [Stat.SpDefense] = -1, [Stat.Speed] = -1}}
}}

move.id = "v-create"
move.name = "V-create"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
