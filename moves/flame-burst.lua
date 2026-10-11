-- Move 481: Flame Burst. Identity and parameters: pokemon_data/assets/moves/481.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      if not ctx:damage(70).hit then return end
      for _, pokemon in ipairs(ctx.target.team.pokemon) do
        if ctx:adjacent(pokemon, ctx.target) then ctx:hurt(pokemon, {fraction = 1/16}) end
      end
    end}

move.id = "flame-burst"
move.name = "Flame Burst"
move.type = Type.Fire
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 70
return move
