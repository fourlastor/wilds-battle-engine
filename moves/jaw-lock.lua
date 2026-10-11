-- Move 746: Jaw Lock. Identity and parameters: pokemon_data/assets/moves/746.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      if not ctx:damage(80).hit or ctx.target.fainted or ctx.user.fainted then return end
      if not ctx.target.linked_trap then ctx:start_effect(ctx.target, "trapped", {rules = {{kind = Rule.Trapped, by = ctx.user}}}) end
      if not ctx.user.linked_trap then ctx:start_effect(ctx.user, "trapped", {rules = {{kind = Rule.Trapped, by = ctx.target}}}) end
    end}

move.id = "jaw-lock"
move.name = "Jaw Lock"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite"}
return move
