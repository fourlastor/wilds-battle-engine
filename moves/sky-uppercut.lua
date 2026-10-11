-- Move 327: Sky Uppercut. Identity and parameters: pokemon_data/assets/moves/327.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.9, hits_hidden = {Hidden.Air},
    flags = {"contact"}, effects = {{kind = Effect.Damage, power = 85}}}

move.id = "sky-uppercut"
move.name = "Sky Uppercut"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
