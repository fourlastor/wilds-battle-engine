using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using Godot;
using PokeWilds.Battle;
using PokeWilds.Battle.BattleEvents;
using PokeWilds.Battle.MoveBases;
using PokeWilds.Battle.Moves;
using PokeWilds.Data;

public partial class ParityHarness : Node
{
    public override void _Ready()
    {
        RunCase("dragon_rage", new MDragonRage(), 7);
        RunCase("dragon_rage_ko", new MDragonRage(), 7, foeHp: 35);
        RunCase("tackle_1", new MTackle(), 1);
        RunCase("tackle_7", new MTackle(), 7);
        RunCase("tackle_99", new MTackle(), 99);
        RunCase("tackle_miss", new MTackle(), FindTackleSeed(miss: true));
        RunCase("tackle_crit", new MTackle(), FindTackleSeed(miss: false));
        RunCase("toxic", new MToxic(), 7);
        RunCase("recover", new MRecover(), 7, allyHp: 50);
        GetTree().Quit();
    }

    private static ulong FindTackleSeed(bool miss)
    {
        for (ulong seed = 1; seed < 10000; seed++)
        {
            GD.Seed(seed);
            GD.RandRange(0, 0);
            GD.Randi();
            GD.Randi();
            GD.Randf();
            var accuracy = GD.Randf();
            if (miss && accuracy > 0.95f)
                return seed;
            if (!miss && accuracy <= 0.95f && GD.Randf() < 0.01f)
                return seed;
        }
        throw new Exception("No seed found for Tackle fixture");
    }

    private static void RunCase(string id, Move allyMove, ulong seed, ushort allyHp = 100, ushort foeHp = 100)
    {
        // Mirror the GD calls in BattleLogic, hit checks, and the selected
        // effect. The next draw after the battle checks the draw count.
        GD.Seed(seed);
        GD.RandRange(0, 0); // foe move choice
        GD.Randi(); // ally order tie breaker
        GD.Randi(); // foe order tie breaker
        GD.Randf(); // ally consecutive hit check
        var chances = new List<float>();
        var allyAccuracy = 1f;
        if (allyMove is not MRecover)
        {
            allyAccuracy = GD.Randf();
            chances.Add(allyAccuracy);
        }
        float? damageRoll = null;
        if (allyMove is MTackle && allyAccuracy <= 0.95f)
        {
            chances.Add(GD.Randf()); // critical hit
            damageRoll = (float)GD.RandRange(0.85, 1.0);
        }
        else if (allyMove is MToxic)
        {
            chances.Add(GD.Randf()); // status effect chance
        }
        if (id != "dragon_rage_ko")
        {
            GD.Randf(); // foe consecutive hit check
            chances.Add(GD.Randf()); // foe accuracy
            chances.Add(GD.Randf()); // Splash effect chance
        }
        var nextDraw = GD.Randi();

        GD.Seed(seed);
        var foeMove = new MSplash();
        var participants = new BattleParticipants(
            [new BattlePokemon(1, "Ally", [allyMove], allyHp, 100, [PType.Normal], Speed: 200)],
            [new BattlePokemon(2, "Foe", [foeMove], foeHp, 100, [PType.Normal], Speed: 100)]
        );
        var battle = new BattleLogic(participants);
        battle.ExecuteTurn([
            new MoveSelection(
                new PParticipantID(PBattleSide.Allies, 0),
                allyMove,
                [new PParticipantID(allyMove is MRecover ? PBattleSide.Allies : PBattleSide.Foes, 0)]
            ),
        ]);

        var events = new List<string>();
        Event? next;
        while ((next = battle.PrepareNextEvent()) is not null)
        {
            switch (next)
            {
                case MessageEvent message:
                    events.Add(string.Format(message.Original, message.Args));
                    break;
                case ReceiveDamageEvent damage:
                    events.Add($"damage:{damage.ParticipantID.Side}:{damage.Amount}");
                    break;
                case HealEvent heal:
                    events.Add($"heal:{heal.ParticipantID.Side}:{heal.Amount}");
                    break;
                case ApplyStatusEvent status:
                    events.Add($"status:{status.ParticipantID.Side}:{status.Status}");
                    break;
                case WinBattleEvent win:
                    events.Add($"win:{win.Side}");
                    break;
            }
            battle.ConsumeNextEffect();
        }

        if (GD.Randi() != nextDraw)
            throw new Exception($"{id}: Godot RNG draw count changed");

        var foe = battle.BattleState.Participants.Foes[0];
        Print(id, "EVENTS", string.Join("|", events));
        Print(id, "HP", battle.BattleState.Participants.Allies[0].HP + "," + foe.HP);
        Print(id, "STATUS", foe.StatusCondition + "," + foe.StackedStatusTurns);
        Print(id, "CHANCES", string.Join(",", chances.Select(x => x.ToString("R", CultureInfo.InvariantCulture))));
        Print(id, "ROLL", damageRoll?.ToString("R", CultureInfo.InvariantCulture) ?? "none");
    }

    private static void Print(string id, string key, string value) =>
        GD.Print($"PARITY_CSHARP_{id}_{key}={value}");
}
