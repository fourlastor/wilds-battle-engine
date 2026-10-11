-- Move 396: Aura Sphere. Identity and parameters: pokemon_data/assets/moves/396.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "aura-sphere"
move.name = "Aura Sphere"
move.type = Type.Fighting
move.category = Category.Special
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet", "pulse"}
return move
