-- Move 681: Power Trip. Identity and parameters: pokemon_data/assets/moves/681.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx) ctx:damage(20 + 20 * ctx.user:raised_stages()) end}

move.id = "power-trip"
move.name = "Power Trip"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 20
return move
