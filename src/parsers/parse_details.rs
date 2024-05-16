pub mod parse_details {
    use std::fs::File;
    use std::io::Read;

    use scraper::{Html, Selector};

    pub struct GameRule<'a> {
        document: &'a Html,
        has_three_win: Option<bool>,
        is_within_unit: Option<bool>,
        at_least_12_match_played: Option<bool>,
        is_popular: Option<bool>,
        consider_straight_win: Option<bool>,
        win_flag_diff: Option<i32>,
        game_data: &'a GameData,
    }

    pub enum MyError {
        UpTo10MatchIsPlayed,
        DoesNotHaveThreeWin,
        IsNotWithinUnit,
        TeamBetweenIsLessThanFour,
    }

    impl<'a> GameRule<'a> {
        pub fn new(document: &'a Html, game_data: &'a GameData) -> Self {
            GameRule {
                document,
                has_three_win: None,
                is_within_unit: None,
                at_least_12_match_played: None,
                is_popular: None,
                consider_straight_win: None,
                win_flag_diff: None,
                game_data,
            }
        }

        pub fn has_three_win(&mut self) -> Result<&mut Self, MyError> {
            let mut scan: Vec<usize> = Vec::new();
            if let Some(team_play) = &self.game_data.last_five {
                for (idx, mw) in team_play.iter().enumerate() {
                    let W: Vec<char> = mw.chars().filter(|ch| *ch == 'W').collect();
                    if W.len() >= 3 {
                        scan.push(idx)
                    } else {
                        continue;
                    }
                }
            }
            if scan.len() == 1 {
                let team = scan.first().unwrap();
                self.game_data.selected_team = Some(*team + 1);
                self.has_three_win = Some(true);
                Ok(self)
            } else {
                Err(MyError::DoesNotHaveThreeWin)
            }
        }

        pub fn is_within_unit(&mut self) -> Result<&mut Self, MyError> {
            let selector;
            if self.game_data.selected_team.unwrap() % 2 == 0 {
                selector = "odd.highlight".to_string();
            } else {
                selector = "even.highlight".to_string();
            };

            let team_selector: Selector = Selector::parse(&format!("tr.{selector}")).unwrap();

            let tr = self.document.select(&team_selector).next().unwrap();

            let rank_selector: Selector = Selector::parse("td.rank").unwrap();
            if let Some(rank) = tr.select(&rank_selector).next() {
                let rank_val: i8 = rank.inner_html().parse().unwrap();
                if rank_val < 9 {
                    self.is_within_unit = Some(true);
                    return Ok(self);
                }
            }
            self.is_within_unit = Some(false);
            Err(MyError::IsNotWithinUnit)
        }

        pub fn team_difference_is_atleast_four(&mut self) -> Result<&mut Self, MyError> {
            let selector = "highlight";
            let team_selector: Selector = Selector::parse(&format!("tr.{selector}")).unwrap();
            let mut position = vec![];
            for tr in self.document.select(&team_selector) {
                let rank_selector: Selector = Selector::parse("td.rank").unwrap();
                for rank in tr.select(&rank_selector) {
                    let rank_val: i8 = rank.inner_html().parse().unwrap();
                    position.push(rank_val)
                }
            }
            if (position[1] - position[0]) > 3 {
                return Ok(self);
            } else {
                return Err(MyError::TeamBetweenIsLessThanFour);
            }
        }

        pub fn at_least_12_match_played(&mut self) -> Result<&mut Self, MyError> {
            let selector;
            if self.game_data.selected_team.unwrap() % 2 == 0 {
                selector = "odd.highlight".to_string();
            } else {
                selector = "even.highlight".to_string();
            };

            let team_selector: Selector = Selector::parse(&format!("tr.{selector}")).unwrap();

            let tr = self.document.select(&team_selector).next().unwrap();
            //let frag = Html::parse_fragment(&tr);

            let match_played_selector: Selector = Selector::parse("td.number.total.mp").unwrap();
            if let Some(mp) = tr.select(&match_played_selector).next() {
                let match_played: i8 = mp.inner_html().parse().unwrap();
                println!("match played {match_played}");
                if match_played > 12 {
                    self.at_least_12_match_played = Some(true);
                }
            }
        }

        pub fn award_point(mut self) {
            let team_selector: Selector = Selector::parse(&format!("tr.highlight")).unwrap();
            let mut points = Vec::new();
            for tr in self.document.select(&team_selector) {
                let point_selector: Selector = Selector::parse("td.number.points").unwrap();
                if let Some(point) = tr.select(&point_selector).next() {
                    let point_val: i8 = point.inner_html().parse().unwrap();
                    points.push(point_val);
                }
            }
            let point = points.first().unwrap() - points.last().unwrap();
            let pred = match point {
                9..=12 => "ov 0.5",
                13..=16 => "ABorGG",
                17..=20 => "ABor2.5",
                21..=24 => "ov 1.5",
                25..=28 => "Draw/A|B",
                29..=32 => "A|B",
                _ => "A|B|ov 3.5",
            };
            self.game_data.prediction = Some(pred.to_string());
        }
    }

    struct GameData<'a> {
        team_names: Option<Vec<&'a str>>,
        last_five: Option<Vec<String>>,
        selected_team: Option<usize>,
        prediction: Option<String>,
        document: &'a Html,
    }

    pub struct GameDataBuilder<'a> {
        team_names: Option<Vec<&'a str>>,
        last_five: Option<Vec<String>>,
        selected_team: Option<usize>,
        prediction: Option<String>,
        document: &'a Html,
    }

    impl<'a> GameDataBuilder<'a> {
        pub fn new(document: &'a Html) -> Self {
            GameDataBuilder {
                team_names: None,
                last_five: None,
                selected_team: None,
                prediction: None,
                document,
            }
        }

        pub fn set_team_name(mut self) -> Self {
            let team_title = Selector::parse("a.team-title").unwrap();
            let mut team_names: Vec<&str> = vec![];
            for ele in self.document.select(&team_title) {
                if let Some(name) = ele.text().collect::<Vec<_>>().first() {
                    team_names.push(name);
                }
            }
            println!("{team_names:?}");
            self.team_names = Some(team_names);
            return self;
        }

        pub fn set_last_five(mut self) -> Self {
            let last_five = Selector::parse("div.last-five").unwrap();
            let mut last_five_match = vec![];
            for mw in self.document.select(&last_five) {
                let container = Html::parse_fragment(&mw.inner_html());
                let a_selector = Selector::parse("a").unwrap();
                let mut five_match = vec![];
                for mt in container.select(&a_selector) {
                    if let Some(mlw) = mt.text().collect::<Vec<_>>().first() {
                        five_match.push(*mlw);
                    }
                }
                let result = five_match.join(" ");
                last_five_match.push(result);
            }

            println!("{last_five_match:?}");
            self.last_five = Some(last_five_match);
            return self;
        }

        pub fn build(self) -> GameData<'a> {
            let mut game_data = GameData {
                team_names: self.team_names,
                last_five: self.last_five,
                document: &self.document,
                selected_team: self.selected_team,
                prediction: self.prediction,
            };

            game_data
        }
    }

    pub fn game_selector() -> std::io::Result<()> {
        let mut file = File::open("src/y.html")?;
        let mut html = String::new();

        if let Ok(_size) = file.read_to_string(&mut html) {
            log::debug!("succesfully read to string");
        };
        let document = Html::parse_document(&html);
        let game_data = GameDataBuilder::new(&document)
            .set_team_name()
            .set_last_five()
            .build();
        let mut game_rule = GameRule::new(&document, &game_data);
        game_rule.has_three_win().and_then(|gr| gr.is_within_unit());

        println!("yooo {:?}", game_rule.game_data.prediction);
        Ok(())
    }
}
