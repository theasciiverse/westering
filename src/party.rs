//! The wagon party and what it carries.

/// Everything the store sells, in the order the store lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Oxen,
    Food,
    Clothing,
    Shot,
    Wheel,
    Axle,
    Tongue,
}

pub const ITEMS: [Item; 7] = [Item::Oxen, Item::Food, Item::Clothing, Item::Shot, Item::Wheel, Item::Axle, Item::Tongue];

pub struct Good {
    pub name: &'static str,
    /// what one purchase buys, as the store says it
    pub unit: &'static str,
    /// how much of the item one purchase is (oxen, pounds, sets, rounds, pieces)
    pub per: u32,
    /// price of one purchase in Independence, in cents
    pub price: u32,
    /// pounds of load per item
    pub weight: f64,
    pub advice: &'static str,
}

impl Item {
    pub fn good(self) -> &'static Good {
        &GOODS[self as usize]
    }
}

static GOODS: [Good; 7] = [
    Good {
        name: "Oxen",
        unit: "yoke of two",
        per: 2,
        price: 5000,
        weight: 0.0,
        advice: "Slow, but they live on prairie grass and stand up to the mountains. Three yokes at least; four if you can spare it.",
    },
    Good {
        name: "Provisions",
        unit: "50 lb",
        per: 50,
        price: 400,
        weight: 1.0,
        advice: "Flour, bacon, beans, coffee, sugar and dried apples. Figure on two or three pounds a day for each of you.",
    },
    Good {
        name: "Clothing",
        unit: "set",
        per: 1,
        price: 1000,
        weight: 5.0,
        advice: "Wool shirts, a coat, stout boots. The nights in the mountains are cold even in August. Two sets each.",
    },
    Good {
        name: "Powder and shot",
        unit: "box of 20 rounds",
        per: 20,
        price: 200,
        weight: 1.0,
        advice: "For the buffalo on the Platte and the deer in the hills. Fresh meat saves your flour.",
    },
    Good {
        name: "Spare wheel",
        unit: "wheel",
        per: 1,
        price: 1000,
        weight: 50.0,
        advice: "There is no wheelwright between here and Fort Laramie, and few after.",
    },
    Good {
        name: "Spare axle",
        unit: "axle",
        per: 1,
        price: 1000,
        weight: 40.0,
        advice: "Hickory. An axle snaps on the rocks more often than you would think.",
    },
    Good {
        name: "Spare tongue",
        unit: "tongue",
        per: 1,
        price: 1000,
        weight: 40.0,
        advice: "The pole the oxen pull by. Break it in a ravine and you will wish you had another.",
    },
];

/// How much a wagon can carry, in pounds, before the oxen suffer for it.
pub const CAPACITY: f64 = 2000.0;

#[derive(Clone, Debug, Default)]
pub struct Supplies {
    amount: [u32; 7],
}

impl Supplies {
    pub fn get(&self, item: Item) -> u32 {
        self.amount[item as usize]
    }

    pub fn set(&mut self, item: Item, n: u32) {
        self.amount[item as usize] = n;
    }

    pub fn add(&mut self, item: Item, n: u32) {
        self.amount[item as usize] += n;
    }

    /// Take up to n, returning how much there was to take.
    pub fn take(&mut self, item: Item, n: u32) -> u32 {
        let have = self.get(item);
        let took = have.min(n);
        self.set(item, have - took);
        took
    }

    pub fn load(&self) -> f64 {
        ITEMS.iter().map(|&i| self.get(i) as f64 * i.good().weight).sum()
    }
}

#[derive(Clone, Debug)]
pub struct Person {
    pub name: String,
    /// 0 is dead, 100 hale
    pub health: i32,
    pub illness: Option<Illness>,
    /// what killed them, for the diary and the end
    pub cause: Option<String>,
    pub epitaph: Option<String>,
}

/// A sickness or an injury: health lost each day until it runs its course.
#[derive(Clone, Debug)]
pub struct Illness {
    pub name: &'static str,
    pub drain: i32,
    pub days: u32,
}

impl Person {
    pub fn new(name: &str) -> Person {
        Person { name: name.into(), health: 100, illness: None, cause: None, epitaph: None }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }
}

pub const LEADER_NAMES: [&str; 8] = ["Josiah", "Hannah", "Elias", "Rebecca", "Nathaniel", "Lucinda", "Amos", "Tabitha"];
pub const COMPANION_NAMES: [&str; 16] = [
    "Abigail", "Thomas", "Mary", "Samuel", "Eliza", "Jacob", "Martha", "Levi", "Sarah", "Caleb", "Ruth", "Henry", "Phoebe", "Silas", "Clara", "Moses",
];

/// The months a party may leave Independence, with what each means.
pub const MONTHS: [(&str, &str); 5] = [
    ("March", "Early. The grass is not up yet and the oxen go hungry, but the mountains are far behind you before the snow."),
    ("April", "The grass is coming. Most who know the trail say leave now."),
    ("May", "The grass is good and so is the company: the trail is crowded, and sickness travels with crowds."),
    ("June", "Late. You will have to hurry to cross the Blue Mountains before the first snow."),
    ("July", "Very late. Only the lucky and the fast reach Oregon before winter."),
];

pub const PURSE: u32 = 80000;

pub fn money(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}
