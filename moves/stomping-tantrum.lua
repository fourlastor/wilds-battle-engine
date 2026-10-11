-- Move 707: Stomping Tantrum. Identity and parameters: pokemon_data/assets/moves/707.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx) ctx:damage(ctx.user.last_move_failed and 150 or 75) end}

move.id = "stomping-tantrum"
move.name = "Stomping Tantrum"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 75
return move
