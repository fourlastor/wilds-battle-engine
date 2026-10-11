-- Move 724: Searing Sunraze Smash. Identity and parameters: pokemon_data/assets/moves/724.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.
-- INCOMPLETE: The engine does not execute abilities; explicit ability-bypass semantics require an ability engine.

local move = {effects = {
  {kind = Effect.Damage, power = 200}
}}

move.id = "searing-sunraze-smash"
move.name = "Searing Sunraze Smash"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
