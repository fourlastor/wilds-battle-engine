-- Move 261: Will-O-Wisp. Identity and parameters: pokemon_data/assets/moves/261.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Status, status = Status.Burned, chance = 1}
}}

move.id = "will-o-wisp"
move.name = "Will-O-Wisp"
move.type = Type.Fire
move.category = Category.Status
move.pp = 15
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
