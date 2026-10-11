-- Move 394: Flare Blitz. Identity and parameters: pokemon_data/assets/moves/394.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120, recoil = 0.33},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "flare-blitz"
move.name = "Flare Blitz"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
move.usable_while_frozen = true
return move
