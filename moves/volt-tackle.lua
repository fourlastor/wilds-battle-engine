-- Move 344: Volt Tackle. Identity and parameters: pokemon_data/assets/moves/344.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120, recoil = 0.33},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.1}
}}

move.id = "volt-tackle"
move.name = "Volt Tackle"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
