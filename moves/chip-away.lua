-- Move 498: Chip Away. Identity and parameters: pokemon_data/assets/moves/498.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    ctx:damage(70, {ignore_stages = true, ignore_evasion = true})
  end,
}

move.id = "chip-away"
move.name = "Chip Away"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 70
return move
