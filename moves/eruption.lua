-- Move 284: Eruption. Identity and parameters: pokemon_data/assets/moves/284.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.AllOpponents,
    script = function(ctx) ctx:damage(math.max(1, 150 * ctx.user.hp / ctx.user.max_hp)) end}

move.id = "eruption"
move.name = "Eruption"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
move.base_power = 150
return move
