-- Move 734: Buzzy Buzz. Identity and parameters: pokemon_data/assets/moves/734.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 1}
}}

move.id = "buzzy-buzz"
move.name = "Buzzy Buzz"
move.type = Type.Electric
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
