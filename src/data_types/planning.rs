use std::rc::Rc;

use crate::ParsedRecipe;

pub struct Day {
    pub breakfast: Option<Rc<ParsedRecipe>>,
    pub lunch: Option<Rc<ParsedRecipe>>,
    pub diner: Option<Rc<ParsedRecipe>>,
}

pub struct Week {
    pub sunday: Day,
    pub monday: Day,
    pub tuesday: Day,
    pub wednesday: Day,
    pub thursday: Day,
    pub friday: Day,
    pub saturday: Day,
}

pub struct CookHistory {
    pub past_weeks: Vec<Week>,
    pub next_weeks: Vec<Week>,
}
