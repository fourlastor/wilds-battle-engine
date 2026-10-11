-- Move 725: Menacing Moonraze Maelstrom. Identity and parameters: pokemon_data/assets/moves/725.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.
-- INCOMPLETE: The engine does not execute abilities; explicit ability-bypass semantics require an ability engine.

local move = {effects = {
  {kind = Effect.Damage, power = 200}
}}

move.id = "menacing-moonraze-maelstrom"
move.name = "Menacing Moonraze Maelstrom"
move.type = Type.Ghost
move.category = Category.Special
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
