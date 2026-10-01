-- Move data transcribed from pokewilds-next/battle/moves. Keep the C# values,
-- including current quirks, until a behavior change is made explicitly.
local function end_thrash(ctx)
  if ctx.total_turns and ctx.turn == ctx.total_turns then ctx:confuse_self() end
  ctx:break_sequence()
end

return {
  { id="ancient_power", name="Ancient Power", type=Type.Rock, category=Category.Special, pp=5,
    effects={{kind=Effect.Damage, power=40}, {kind=Effect.Stats, chance=0.1, self=true,
      stages={[Stat.Attack]=1, [Stat.Defense]=1, [Stat.SpAttack]=1, [Stat.SpDefense]=1, [Stat.Speed]=1}}} },
  { id="bind", name="Bind", type=Type.Normal, category=Category.Physical, pp=20,
    effects={{kind=Effect.Damage, power=15}, {kind=Effect.Bind}} },
  { id="calm_mind", name="Calm Mind", type=Type.Psychic, category=Category.Status, pp=20,
    target=Target.User, accuracy=Accuracy.Always, effects={{kind=Effect.Stats, stages={[Stat.SpAttack]=1, [Stat.SpDefense]=1}}} },
  { id="charge", name="Charge", type=Type.Electric, category=Category.Status, pp=20,
    target=Target.User, accuracy=Accuracy.Always,
    script=function(ctx)
      ctx:change_self_stat(ctx.Stat.SpDefense, 1)
      ctx:boost_next_move(ctx.Type.Electric, 2)
    end },
  { id="confuse_ray", name="Confuse Ray", type=Type.Ghost, category=Category.Status, pp=10,
    effects={{kind=Effect.Confuse}} },
  { id="dive", name="Dive", type=Type.Water, category=Category.Physical, pp=10,
    effects={{kind=Effect.TwoTurn, semi_invulnerable=true, charge_message="{0} hid underwater!", power=80}} },
  { id="double_edge", name="Double-Edge", type=Type.Normal, category=Category.Physical, pp=15,
    effects={{kind=Effect.Damage, power=120, recoil=1/3}} },
  { id="double_kick", name="Double Kick", type=Type.Fighting, category=Category.Physical, pp=30,
    effects={{kind=Effect.MultiHit, power=30, min_hits=2, max_hits=2}} },
  { id="double_team", name="Double Team", type=Type.Normal, category=Category.Status, pp=15,
    target=Target.User, accuracy=Accuracy.Always, effects={{kind=Effect.Stats, stages={[Stat.Evasion]=1}}} },
  { id="dragon_rage", name="Dragon Rage", type=Type.Dragon, category=Category.Special, pp=10,
    effects={{kind=Effect.FixedDamage, amount=40}} },
  { id="draining_kiss", name="Draining Kiss", type=Type.Fairy, category=Category.Physical, pp=15,
    effects={{kind=Effect.Damage, power=40, drain=0.75}} },
  { id="fissure", name="Fissure", type=Type.Ground, category=Category.Physical, pp=5,
    accuracy=Accuracy.Ohko, effects={{kind=Effect.Ohko}} },
  { id="headbutt", name="Headbutt", type=Type.Normal, category=Category.Physical, pp=15,
    effects={{kind=Effect.Damage, power=70}, {kind=Effect.Flinch, chance=0.3}} },
  { id="night_shade", name="Night Shade", type=Type.Ghost, category=Category.Special, pp=15,
    effects={{kind=Effect.LevelDamage}} },
  { id="petal_dance", name="Petal Dance", type=Type.Grass, category=Category.Special, pp=10,
    effects={{kind=Effect.Consecutive, power=10, min_turns=2, max_turns=3, confuse_after=true}} },
  { id="pin_missile", name="Pin Missile", type=Type.Bug, category=Category.Physical, pp=20, accuracy=0.95,
    effects={{kind=Effect.MultiHit, power=25, min_hits=2, max_hits=4}} },
  { id="poison_sting", name="Poison Sting", type=Type.Poison, category=Category.Physical, pp=35,
    effects={{kind=Effect.Damage, power=15}, {kind=Effect.Status, status=Status.Poisoned, chance=0.3}} },
  { id="protect", name="Protect", type=Type.Normal, category=Category.Status, pp=10,
    target=Target.User, priority=4, effects={{kind=Effect.Protect}} },
  { id="recover", name="Recover", type=Type.Normal, category=Category.Status, pp=5,
    target=Target.User, accuracy=Accuracy.Always, fail_on_full_hp=true, effects={{kind=Effect.Heal, fraction=0.5}} },
  { id="rage", name="Rage", type=Type.Normal, category=Category.Physical, pp=20,
    on_hit=function(ctx)
      ctx:change_self_stat(ctx.Stat.Attack, 1)
    end,
    script=function(ctx)
      ctx:watch_hits_until_next_action()
      ctx:damage(20)
    end },
  { id="rest", name="Rest", type=Type.Psychic, category=Category.Status, pp=5,
    target=Target.User, accuracy=Accuracy.Always, fail_on_full_hp=true,
    effects={{kind=Effect.Status, status=Status.RestSleep, replace=true}, {kind=Effect.Heal, fraction=1, hide_message=true}} },
  { id="rollout", name="Rollout", type=Type.Rock, category=Category.Physical, pp=20, accuracy=0.9,
    effects={{kind=Effect.Consecutive, power=30, min_turns=5, max_turns=5, double_power=true}} },
  { id="sandstorm", name="Sandstorm", type=Type.Rock, category=Category.Status, pp=10,
    target=Target.Field, effects={{kind=Effect.Weather, weather=Weather.Sandstorm, turns=5}} },
  { id="shadow_claw", name="Shadow Claw", type=Type.Ghost, category=Category.Physical, pp=15,
    effects={{kind=Effect.Damage, power=70, high_crit=true}} },
  { id="solar_beam", name="Solar Beam", type=Type.Grass, category=Category.Special, pp=10,
    manual_announce=true,
    script=function(ctx)
      if ctx.turn == 1 and ctx.weather ~= Weather.Sun then
        ctx:message(ctx.user.name .. " took in sunlight!")
        ctx:force_move(2, ctx.TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      ctx:damage(120)
    end },
  { id="splash", name="Splash", type=Type.Normal, category=Category.Physical, pp=20,
    effects={{kind=Effect.Splash}} },
  { id="storm_throw", name="Storm Throw", type=Type.Fighting, category=Category.Physical, pp=10,
    effects={{kind=Effect.Damage, power=60, always_crit=true}} },
  { id="sunny_day", name="Sunny Day", type=Type.Fire, category=Category.Status, pp=5,
    target=Target.Field, effects={{kind=Effect.Weather, weather=Weather.Sun, turns=5}} },
  { id="swagger", name="Swagger", type=Type.Normal, category=Category.Status, pp=15, accuracy=0.85,
    effects={{kind=Effect.Stats, stages={[Stat.Attack]=2}}} },
  { id="tackle", name="Tackle", type=Type.Normal, category=Category.Physical, pp=35, accuracy=0.95,
    effects={{kind=Effect.Damage, power=40}} },
  { id="thunder_shock", name="Thunder Shock", type=Type.Electric, category=Category.Special, pp=30,
    effects={{kind=Effect.Damage, power=40}, {kind=Effect.Status, status=Status.Paralyzed, chance=0.1}} },
  { id="toxic", name="Toxic", type=Type.Poison, category=Category.Status, pp=10, accuracy=0.9,
    effects={{kind=Effect.Status, status=Status.BadlyPoisoned}} },
  -- Scripted moves call the Rust battle API and resume with each operation's result.
  { id="dream_eater", name="Dream Eater", type=Type.Psychic, category=Category.Special, pp=15,
    script=function(ctx)
      if ctx.target.status ~= ctx.Status.Asleep then return ctx:fail() end
      ctx:damage(100, {drain=0.5})
    end },
  { id="false_swipe", name="False Swipe", type=Type.Normal, category=Category.Physical, pp=40,
    script=function(ctx)
      ctx:damage(40, {min_target_hp=1})
    end },
  { id="triple_kick", name="Triple Kick", type=Type.Fighting, category=Category.Physical, pp=10,
    script=function(ctx)
      for hit=1,3 do
        local result = ctx:damage(10*hit, {accuracy=0.9})
        if not result.hit then return end
      end
    end },
  { id="explosion", name="Explosion", type=Type.Normal, category=Category.Physical, pp=5, target=Target.AllOthers,
    script=function(ctx)
      ctx:faint_user()
      ctx:damage(250)
    end },
  { id="hyper_beam", name="Hyper Beam", type=Type.Normal, category=Category.Special, pp=5, accuracy=0.9,
    script=function(ctx)
      local result = ctx:damage(150)
      if result.hit then ctx:recharge() end
    end },
  { id="thrash", name="Thrash", type=Type.Normal, category=Category.Physical, pp=10,
    accuracy=1.0, target=Target.RandomOpponent, on_interrupt=end_thrash,
    script=function(ctx)
      if not ctx.total_turns then
        ctx:force_move(ctx:random_int(2, 3), ctx.TargetPolicy.RandomOpponent)
      end
      local result = ctx:damage(120)
      if not result.hit then return end_thrash(ctx) end
      if ctx.total_turns and ctx.turn == ctx.total_turns then ctx:confuse_self() end
    end },
}
