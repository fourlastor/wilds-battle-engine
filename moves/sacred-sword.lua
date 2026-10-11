-- Move 533: Sacred Sword. Identity and parameters: pokemon_data/assets/moves/533.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    ctx:damage(90, {ignore_stages = true, ignore_evasion = true})
  end,
}

move.id = "sacred-sword"
move.name = "Sacred Sword"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
move.base_power = 90
return move
