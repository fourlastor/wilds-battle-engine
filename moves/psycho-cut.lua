-- Move 427: Psycho Cut. Identity and parameters: pokemon_data/assets/moves/427.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70, high_crit = true}
}}

move.id = "psycho-cut"
move.name = "Psycho Cut"
move.type = Type.Psychic
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"slicing"}
return move
