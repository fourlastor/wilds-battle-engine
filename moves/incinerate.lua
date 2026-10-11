-- Move 510: Incinerate. Identity and parameters: pokemon_data/assets/moves/510.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Removing berries/gems works, but pre-hit item activation timing requires an item engine.

local move = {
  script = function(ctx)
    ctx:damage(60)
    for _, pokemon in ipairs(ctx.targets) do
      if ctx:reached(pokemon) and pokemon.item and (pokemon.item:find("berry", 1, true) or pokemon.item:find("gem", 1, true)) then
        ctx:take_item(pokemon)
      end
    end
  end,
}

move.id = "incinerate"
move.name = "Incinerate"
move.type = Type.Fire
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"incomplete"}
return move
