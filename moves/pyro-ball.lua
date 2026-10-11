-- Move 780: Pyro Ball. Identity and parameters: pokemon_data/assets/moves/780.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "pyro-ball"
move.name = "Pyro Ball"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
