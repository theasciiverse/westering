//! The road from Independence to the Willamette Valley as it ran in 1848,
//! landmark by landmark. Mileages are rounded, from emigrant guides; the
//! whole comes to a little under two thousand miles, as the guides said.

use crate::trail::Region;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Start,
    /// something to see and write home about
    Sight,
    /// a trading post with a store, prices times the factor
    Fort(f64),
    /// a river to cross: its usual depth in feet, width in yards, and a
    /// ferry's price in cents where one ran
    River { depth: f64, width: u32, ferry: Option<u32> },
    /// a fork in the road; the choices are in `Landmark::next`
    Fork,
    End,
}

/// Where the road goes after a landmark.
#[derive(Clone, Copy, Debug)]
pub enum Next {
    /// the next landmark's index and the miles to it
    On(usize, f64),
    /// two roads: a label, the landmark it leads to and the miles
    Fork([(&'static str, usize, f64); 2]),
    Done,
}

pub struct Landmark {
    pub name: &'static str,
    pub kind: Kind,
    /// the country on the way to it
    pub region: Region,
    pub note: &'static str,
    pub next: Next,
}

use Kind::*;
use Region::*;

pub const START: usize = 0;

pub static ROUTE: [Landmark; 25] = [
    Landmark {
        name: "Independence, Missouri",
        kind: Start,
        region: Prairie,
        note: "The jumping-off place. Wagons, oxen and greenhorns crowd the square.",
        next: Next::On(1, 102.0),
    },
    Landmark {
        name: "Kansas River",
        kind: River { depth: 3.5, width: 220, ferry: Some(400) },
        region: Prairie,
        note: "The first real river. The Papin brothers run a ferry for those who will pay; the rest float their wagons over.",
        next: Next::On(2, 83.0),
    },
    Landmark {
        name: "Big Blue River",
        kind: River { depth: 2.5, width: 70, ferry: None },
        region: Prairie,
        note: "Clear water and good grass. A spring falls into an alcove of rock nearby, and emigrants have carved their names beside it.",
        next: Next::On(3, 120.0),
    },
    Landmark {
        name: "Fort Kearny",
        kind: Sight,
        region: Prairie,
        note: "Soldiers are building a fort here this summer on the Platte, to guard the road west. Sod walls, a few tents, nothing to buy yet.",
        next: Next::On(4, 145.0),
    },
    Landmark {
        name: "South Platte crossing",
        kind: River { depth: 2.0, width: 600, ferry: None },
        region: HighPlains,
        note: "Half a mile wide and a few feet deep, with quicksand under it. Keep the wagon moving or it sinks.",
        next: Next::On(5, 45.0),
    },
    Landmark {
        name: "Ash Hollow",
        kind: Sight,
        region: HighPlains,
        note: "The road drops down Windlass Hill so steeply that wagons go down on ropes. Below: ash trees, a sweet spring, the North Platte.",
        next: Next::On(6, 80.0),
    },
    Landmark {
        name: "Chimney Rock",
        kind: Sight,
        region: HighPlains,
        note: "A spire of clay and sandstone, seen for days before you reach it. Hardly a diary on the road fails to mention it.",
        next: Next::On(7, 21.0),
    },
    Landmark {
        name: "Scotts Bluff",
        kind: Sight,
        region: HighPlains,
        note: "Bluffs like the walls of a ruined city, named for a fur trader who is said to have died alone near here.",
        next: Next::On(8, 54.0),
    },
    Landmark {
        name: "Fort Laramie",
        kind: Fort(1.5),
        region: HighPlains,
        note: "Adobe walls where the Laramie River meets the North Platte. Lakota families camp nearby to trade. Prices are high, and rise further west.",
        next: Next::On(9, 140.0),
    },
    Landmark {
        name: "North Platte crossing",
        kind: River { depth: 4.5, width: 150, ferry: Some(300) },
        region: HighPlains,
        note: "Mormon pioneers set up a ferry here last year and charge for passage. The river is fast and cold with snowmelt.",
        next: Next::On(10, 50.0),
    },
    Landmark {
        name: "Independence Rock",
        kind: Sight,
        region: Mountains,
        note: "A great grey turtle of granite by the Sweetwater. Reach it by the Fourth of July, they say, and you will beat the snows. Thousands have written their names on it.",
        next: Next::On(11, 100.0),
    },
    Landmark {
        name: "South Pass",
        kind: Sight,
        region: Mountains,
        note: "The Continental Divide, so broad and gentle you could miss it. From here the waters run west, to the Pacific.",
        next: Next::On(12, 20.0),
    },
    Landmark {
        name: "Parting of the Ways",
        kind: Fork,
        region: Mountains,
        note: "The Sublette Cutoff strikes due west across fifty miles without water or grass and saves most of a week. The main road bends south to Fort Bridger.",
        next: Next::Fork([("Take the Sublette Cutoff", 14, 60.0), ("Keep to the road for Fort Bridger", 13, 100.0)]),
    },
    Landmark {
        name: "Fort Bridger",
        kind: Fort(1.8),
        region: Mountains,
        note: "Jim Bridger's log post on Black's Fork, with a blacksmith's forge. Supplies are dear.",
        next: Next::On(15, 175.0),
    },
    Landmark {
        name: "Green River",
        kind: River { depth: 5.0, width: 120, ferry: Some(500) },
        region: Mountains,
        note: "You made the dry crossing. Mountain men run a ferry over the Green for a steep price.",
        next: Next::On(15, 145.0),
    },
    Landmark {
        name: "Soda Springs",
        kind: Sight,
        region: Mountains,
        note: "Springs that bubble and fizz like soda water. One, the Steamboat Spring, puffs and chugs like an engine.",
        next: Next::On(16, 55.0),
    },
    Landmark {
        name: "Fort Hall",
        kind: Fort(2.0),
        region: Mountains,
        note: "The Hudson's Bay Company's post on the Snake. The traders say wagons cannot reach Oregon and you should go to California. Wagons have done it every year since 1843.",
        next: Next::On(17, 135.0),
    },
    Landmark {
        name: "Three Island Crossing",
        kind: River { depth: 5.5, width: 300, ferry: None },
        region: SnakeDesert,
        note: "The Snake runs swift and deep, but the islands break the crossing in three, and the far bank has grass and water.",
        next: Next::On(18, 125.0),
    },
    Landmark {
        name: "Fort Boise",
        kind: Fort(2.2),
        region: SnakeDesert,
        note: "A small Hudson's Bay post of adobe near the mouth of the Boise. The last store before the Blue Mountains.",
        next: Next::On(19, 120.0),
    },
    Landmark {
        name: "Grande Ronde Valley",
        kind: Sight,
        region: BlueMountains,
        note: "A green valley ringed by the Blue Mountains. Rest the oxen here; the climb ahead is the hardest of the journey.",
        next: Next::On(20, 50.0),
    },
    Landmark {
        name: "Umatilla River",
        kind: Sight,
        region: BlueMountains,
        note: "The road leaves the mountains. The Whitman mission to the north has stood empty since the killings of last November, and the Cayuse war goes on.",
        next: Next::On(21, 100.0),
    },
    Landmark {
        name: "The Dalles",
        kind: Fork,
        region: Columbia,
        note: "The wagon road ends at the Columbia. Barlow's toll road climbs round the shoulder of Mount Hood; or build a raft and take your chances on the river.",
        next: Next::Fork([("Take the Barlow Road ($5 toll)", 22, 100.0), ("Raft down the Columbia", 23, 85.0)]),
    },
    Landmark {
        name: "Laurel Hill",
        kind: Sight,
        region: Columbia,
        note: "The worst slope on the Barlow Road. Wagons go down with trees chained behind them as brakes.",
        next: Next::On(24, 30.0),
    },
    Landmark {
        name: "Fort Vancouver",
        kind: Fort(1.4),
        region: Columbia,
        note: "You came through the Cascades rapids alive. Dr. McLoughlin's people have helped many a ragged party here.",
        next: Next::On(24, 25.0),
    },
    Landmark {
        name: "Oregon City",
        kind: End,
        region: Columbia,
        note: "The falls of the Willamette, a sawmill, a land office, and the valley spread out green before you.",
        next: Next::Done,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every road from Independence reaches the end, in under two thousand miles.
    #[test]
    fn every_road_reaches_oregon_city() {
        fn walk(i: usize, miles: f64, totals: &mut Vec<f64>) {
            match ROUTE[i].next {
                Next::On(n, m) => walk(n, miles + m, totals),
                Next::Fork(f) => f.iter().for_each(|&(_, n, m)| walk(n, miles + m, totals)),
                Next::Done => {
                    assert_eq!(ROUTE[i].kind, Kind::End);
                    totals.push(miles);
                }
            }
        }
        let mut totals = Vec::new();
        walk(START, 0.0, &mut totals);
        assert_eq!(totals.len(), 4);
        for t in totals {
            assert!((1750.0..2000.0).contains(&t), "{t}");
        }
    }
}
