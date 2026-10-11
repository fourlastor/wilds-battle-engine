-- Move 550: Bolt Strike. Identity and parameters: pokemon_data/assets/moves/550.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 130},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.2}
}}

move.id = "bolt-strike"
move.name = "Bolt Strike"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
