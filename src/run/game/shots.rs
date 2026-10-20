//! 開發用截圖：`just shots` 把每個畫面各存一張 PNG，不用手動玩到那個畫面就能檢查排版。
//! 只有原生版有這個功能。

use super::{Game, Scene};
use crate::battle::Side;
use crate::battle::view::BattleScreen;
use crate::run::Run;
use crate::run::map::NodeKind;
use crate::run::placement::PlacementScreen;
use crate::run::rest_view::RestScreen;
use crate::screen;
use crate::sprite::growth::Boost;
use macroquad::prelude::*;

/// 截圖用的隊伍：開局精靈再加幾隻
const PARTY: [&str; 5] = [
    "pebble_turtle",
    "dew_sprout",
    "shadow_cat",
    "frost_moth",
    "moon_whale",
];

/// 把所有畫面存到 `dir`
pub async fn save_all(dir: &str) {
    std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("無法建立 {dir}：{e}"));
    for (name, mut game) in scenes() {
        let path = format!("{dir}/{name}.png");
        // 多畫幾幀，讓字型快取等準備好
        for frame in 0..3 {
            screen::begin();
            game.frame(0.0);
            if frame == 2 {
                screen::save_png(&path);
            }
            screen::end();
            next_frame().await;
        }
        println!("{path}");
    }
}

fn game(run: Run, scene: Scene) -> Game {
    Game {
        run,
        scene,
        seed: 7,
        fade: 0.0,
    }
}

/// 有幾隻精靈、一些金幣，已經走了 `steps` 個節點的冒險
fn sample_run(steps: usize) -> Run {
    let mut run = Run::new(7);
    run.balls = PARTY.len() as u32 + 2;
    for id in PARTY {
        run.catch(id);
    }
    run.gold = 40;
    for _ in 0..steps {
        let next = run.choices()[0];
        run.enter(next);
    }
    run
}

fn battle(run: &mut Run, kind: NodeKind) -> BattleScreen {
    let enemies = run.encounter(kind);
    let formation = &run.formations[1];
    let placement: Vec<_> = run
        .party
        .iter()
        .take(formation.slots.len())
        .map(Some)
        .collect();
    let allies = formation.place(&placement, Side::Ally).unwrap();
    BattleScreen::new(run.start_battle(allies, enemies))
}

fn scenes() -> Vec<(&'static str, Game)> {
    let mut shots = vec![
        ("01-map-start", game(Run::new(7), Scene::Map)),
        ("02-map-midway", game(sample_run(3), Scene::Map)),
    ];

    let mut run = sample_run(1);
    let enemies = run.encounter(NodeKind::Elite);
    let mut placement = PlacementScreen::new(&run, enemies);
    placement.select(1);
    shots.push(("03-placement", game(run, Scene::Placement(placement))));

    let mut run = sample_run(1);
    let mut screen = battle(&mut run, NodeKind::Boss);
    for _ in 0..3 {
        screen.update(1.0);
    }
    // 停在這一回合動畫的中間
    screen.update(0.3);
    shots.push(("04-battle", game(run, Scene::Battle(screen, None))));

    let mut run = sample_run(1);
    let mut screen = battle(&mut run, NodeKind::Wild);
    screen.skip();
    shots.push(("05-battle-end", game(run, Scene::Battle(screen, None))));

    let mut run = sample_run(1);
    let offers = run.catch_offers();
    shots.push(("06-catch", game(run, Scene::Catch(offers))));
    shots.push(("07-shop", game(sample_run(1), Scene::Shop)));

    let rest = RestScreen {
        selected: Some(1),
        ..Default::default()
    };
    shots.push(("08-rest", game(sample_run(1), Scene::Rest(rest))));
    let rest = RestScreen {
        done: Some((1, Boost::Attack, 3)),
        ..Default::default()
    };
    shots.push(("09-rest-done", game(sample_run(1), Scene::Rest(rest))));

    shots.push(("10-event", game(sample_run(1), Scene::Notice("事件"))));
    shots.push(("11-game-over", game(sample_run(4), Scene::GameOver)));
    shots.push(("12-cleared", game(sample_run(7), Scene::Cleared)));
    shots
}
