-- Move 428: Zen Headbutt. Identity and parameters: pokemon_data/assets/moves/428.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.2}
}}

move.id = "zen-headbutt"
move.name = "Zen Headbutt"
move.type = Type.Psychic
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
