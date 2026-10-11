-- Move 714: Moongeist Beam. Identity and parameters: pokemon_data/assets/moves/714.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100}
}}

move.id = "moongeist-beam"
move.name = "Moongeist Beam"
move.type = Type.Ghost
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"ignore_ability"}
return move
