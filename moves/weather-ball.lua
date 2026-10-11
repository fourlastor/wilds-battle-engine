-- Move 311: Weather Ball. Identity and parameters: pokemon_data/assets/moves/311.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      local kinds = {[Weather.Sun] = Type.Fire, [Weather.Rain] = Type.Water, [Weather.Hail] = Type.Ice, [Weather.Sandstorm] = Type.Rock}
      if ctx.hit_index == 1 then ctx.move_state.kind = ctx.weather and kinds[ctx.weather] end
      local kind = ctx.move_state.kind
      ctx:damage(kind and 100 or 50, {type = kind})
    end}

move.id = "weather-ball"
move.name = "Weather Ball"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
move.base_power = 50
return move
