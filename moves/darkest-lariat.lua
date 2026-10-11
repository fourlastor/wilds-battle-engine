-- Move 663: Darkest Lariat. Identity and parameters: pokemon_data/assets/moves/663.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    ctx:damage(85, {ignore_stages = true, ignore_evasion = true})
  end,
}

move.id = "darkest-lariat"
move.name = "Darkest Lariat"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 85
return move
