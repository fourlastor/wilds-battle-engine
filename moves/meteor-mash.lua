-- Move 309: Meteor Mash. Identity and parameters: pokemon_data/assets/moves/309.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Stats, chance = 0.2, self = true, stages = {[Stat.Attack] = 1}}
}}

move.id = "meteor-mash"
move.name = "Meteor Mash"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch", "sheer_force"}
return move
