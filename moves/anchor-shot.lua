-- Move 677: Anchor Shot. Identity and parameters: pokemon_data/assets/moves/677.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx:damage(80).hit and not ctx.target.fainted and not ctx.target.linked_trap and not ctx.user.fainted then
      ctx:start_effect(ctx.target, "trapped", {rules = {{kind = Rule.Trapped, by = ctx.user}}})
    end
  end,
}

move.id = "anchor-shot"
move.name = "Anchor Shot"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
