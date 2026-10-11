-- Move 614: Thousand Arrows. Identity and parameters: pokemon_data/assets/moves/614.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    for _, pokemon in ipairs(ctx.targets) do
      -- The first hit on an airborne Flying type is neutral against both of its types.
      local matchups = {}
      if pokemon:has_type(Type.Flying) and not pokemon.grounded then
        for _, kind in ipairs(pokemon.types) do matchups[kind] = 1 end
      end
      local hit = ctx:damage(90, {target = pokemon, effective = matchups, hits_hidden = {Hidden.Air}})
      if hit.hit and not hit.substitute then
        ctx:end_effect(pokemon, "magnet_rise")
        ctx:end_effect(pokemon, "telekinesis")
        ctx:unhide(pokemon)
        ctx:start_effect(pokemon, "smacked_down", {rules = {{kind = Rule.Grounded}}})
      end
    end
  end,
}

move.id = "thousand-arrows"
move.name = "Thousand Arrows"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"hits_airborne"}
move.hits_hidden = {Hidden.Air}
return move
