-- Move 679: Lunge. Identity and parameters: pokemon_data/assets/moves/679.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1}}
}}

move.id = "lunge"
move.name = "Lunge"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
