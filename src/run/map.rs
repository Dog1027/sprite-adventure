//! 分支地圖：一層由多個階段組成，每個階段有幾個節點，最後是頭目。規則見 docs/DESIGN.md「一局的流程」。

use crate::rng::Rng;

/// 一層有幾個階段（含頭目）；暫定，DESIGN.md 尚未決定
pub const STAGES: usize = 8;
/// 每個階段最少、最多幾個節點；暫定
const MIN_WIDTH: usize = 2;
const MAX_WIDTH: usize = 4;

/// 地圖上的節點種類
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Wild,
    Elite,
    Shop,
    Rest,
    Event,
    Boss,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub stage: usize,
    /// 在這個階段中的位置（0 起算）
    pub lane: usize,
    /// 下一個階段可以走到的節點
    pub next: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Map {
    pub nodes: Vec<Node>,
    /// 每個階段的節點編號
    pub stages: Vec<Vec<usize>>,
}

impl Map {
    pub fn generate(rng: &mut Rng) -> Self {
        let mut nodes = Vec::new();
        let mut stages = Vec::new();
        for stage in 0..STAGES {
            let width = if stage == STAGES - 1 {
                1
            } else {
                MIN_WIDTH + rng.below(MAX_WIDTH - MIN_WIDTH + 1)
            };
            let ids: Vec<usize> = (0..width)
                .map(|lane| {
                    nodes.push(Node {
                        kind: pick_kind(stage, rng),
                        stage,
                        lane,
                        next: Vec::new(),
                    });
                    nodes.len() - 1
                })
                .collect();
            stages.push(ids);
        }

        // 連線：每個節點連到下一階段位置相近的 1～2 個節點，且下一階段每個節點都有人連進來
        for stage in 0..STAGES - 1 {
            let (here, there) = (&stages[stage], &stages[stage + 1]);
            let near = |lane| scale_lane(lane, here.len(), there.len());
            for (lane, &id) in here.iter().enumerate() {
                let mut targets = vec![there[near(lane)]];
                let extra = near(lane) + 1;
                if extra < there.len() && rng.below(2) == 0 {
                    targets.push(there[extra]);
                }
                nodes[id].next = targets;
            }
            for (lane, &id) in there.iter().enumerate() {
                if !here.iter().any(|&h| nodes[h].next.contains(&id)) {
                    // 從位置最接近的節點連進來
                    let from = here[scale_lane(lane, there.len(), here.len())];
                    nodes[from].next.push(id);
                    nodes[from].next.sort();
                }
            }
        }

        Self { nodes, stages }
    }
}

/// 把一個階段中的位置換算到另一個節點數不同的階段（四捨五入）
fn scale_lane(lane: usize, from_len: usize, to_len: usize) -> usize {
    if from_len <= 1 {
        return (to_len - 1) / 2;
    }
    (lane * (to_len - 1) * 2 + (from_len - 1)) / ((from_len - 1) * 2)
}

/// 節點種類的比例；暫定
fn pick_kind(stage: usize, rng: &mut Rng) -> NodeKind {
    match stage {
        0 => NodeKind::Wild,
        s if s == STAGES - 1 => NodeKind::Boss,
        s if s == STAGES - 2 => NodeKind::Rest,
        s => {
            let roll = rng.below(100);
            match roll {
                0..50 => NodeKind::Wild,
                50..65 if s >= 2 => NodeKind::Elite,
                50..65 => NodeKind::Wild,
                65..80 => NodeKind::Event,
                80..92 => NodeKind::Shop,
                _ => NodeKind::Rest,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maps() -> impl Iterator<Item = Map> {
        (1..200).map(|seed| Map::generate(&mut Rng::new(seed)))
    }

    #[test]
    fn starts_with_wild_and_ends_with_boss() {
        for m in maps() {
            assert_eq!(m.stages.len(), STAGES);
            assert!(
                m.stages[0]
                    .iter()
                    .all(|&i| m.nodes[i].kind == NodeKind::Wild)
            );
            let last = &m.stages[STAGES - 1];
            assert_eq!(last.len(), 1);
            assert_eq!(m.nodes[last[0]].kind, NodeKind::Boss);
        }
    }

    #[test]
    fn every_node_is_reachable_and_leads_on() {
        for m in maps() {
            for stage in 0..STAGES - 1 {
                for &id in &m.stages[stage] {
                    let n = &m.nodes[id];
                    assert!(!n.next.is_empty(), "節點 {id} 走不下去");
                    assert!(n.next.iter().all(|&t| m.nodes[t].stage == stage + 1));
                }
                for &id in &m.stages[stage + 1] {
                    let incoming = m.stages[stage]
                        .iter()
                        .any(|&h| m.nodes[h].next.contains(&id));
                    assert!(incoming, "節點 {id} 走不到");
                }
            }
        }
    }

    #[test]
    fn no_elites_in_first_two_stages() {
        for m in maps() {
            for stage in 0..2 {
                assert!(
                    m.stages[stage]
                        .iter()
                        .all(|&i| m.nodes[i].kind != NodeKind::Elite)
                );
            }
        }
    }

    #[test]
    fn same_seed_same_map() {
        let a = Map::generate(&mut Rng::new(7));
        let b = Map::generate(&mut Rng::new(7));
        let kinds = |m: &Map| {
            m.nodes
                .iter()
                .map(|n| (n.kind, n.next.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(kinds(&a), kinds(&b));
    }
}
