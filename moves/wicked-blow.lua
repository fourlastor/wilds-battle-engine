-- Move 817: Wicked Blow. Identity and parameters: pokemon_data/assets/moves/817.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75, always_crit = true}
}}

move.id = "wicked-blow"
move.name = "Wicked Blow"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
