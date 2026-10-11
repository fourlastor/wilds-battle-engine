-- Move 612: Power-Up Punch. Identity and parameters: pokemon_data/assets/moves/612.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Attack] = 1}}
}}

move.id = "power-up-punch"
move.name = "Power-Up Punch"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
