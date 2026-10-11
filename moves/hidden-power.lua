-- Move 237: Hidden Power. Identity and parameters: pokemon_data/assets/moves/237.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local ivs = ctx.user.ivs or {hp=31, attack=31, defense=31, speed=31, sp_attack=31, sp_defense=31}
local bits = 0
for bit, iv in ipairs({ivs.hp, ivs.attack, ivs.defense, ivs.speed, ivs.sp_attack, ivs.sp_defense}) do
  bits = bits + (iv % 2) * 2 ^ (bit - 1)
end
ctx:damage(60, {type = ctx:type_number(bits * 15 // 63)})
end}

move.id = "hidden-power"
move.name = "Hidden Power"
move.type = Type.Normal
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 60
return move
