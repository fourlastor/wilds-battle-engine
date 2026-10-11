-- Move 662: Spirit Shackle. Identity and parameters: pokemon_data/assets/moves/662.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      if ctx:damage(80).hit and not ctx.target.fainted and not ctx.target.linked_trap and not ctx.user.fainted then
        ctx:start_effect(ctx.target, "trapped", {rules = {{kind = Rule.Trapped, by = ctx.user}}})
      end
    end}

move.id = "spirit-shackle"
move.name = "Spirit Shackle"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
