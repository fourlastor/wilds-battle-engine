-- Move 703: Genesis Supernova. Identity and parameters: pokemon_data/assets/moves/703.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Terrain is recorded, but grounded-only damage/status/healing rules and Psychic Terrain priority blocking require engine support.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {
  script = function(ctx)
    if ctx:damage(185).hit then
      ctx:start_effect(ctx.field, "psychic_terrain", {turns = 5, group = "terrain", restart = true})
    end
  end,
}

move.id = "genesis-supernova"
move.name = "Genesis Supernova"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
