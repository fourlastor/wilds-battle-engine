-- Move 721: Plasma Fists. Identity and parameters: pokemon_data/assets/moves/721.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      if ctx:damage(100).hit then
        ctx:start_effect(ctx.field, "ion_deluge", {turns = 1, restart = true,
          rules = {{kind = Rule.MoveType, from = Type.Normal, to = Type.Electric}}})
        ctx:message("A deluge of ions showers the battlefield!")
      end
    end}

move.id = "plasma-fists"
move.name = "Plasma Fists"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
move.base_power = 100
return move
