pub mod parse_details {
    use std::fs::File;
    use std::io::Read;

    use scraper::{Html, Selector};

    pub struct GameRule<'a> {
        document: &'a Html,
        has_three_win: Option<bool>,
        is_within_unit: Option<bool>,
        is_popular: Option<bool>,
        consider_straight_win: Option<bool>,
        game_difference: Option<i32>,
        win_flag_diff: Option<i32>,
        game_data: GameData<'a>,
    }

    impl<'a> GameRule<'a> {
        pub fn new(document: &'a Html) -> Self {
            let gdb = GameDataBuilder::new(document)
                .set_team_name()
                .set_last_five()
                .build();
            GameRule {
                document,
                has_three_win: None,
                is_within_unit: None,
                is_popular: None,
                consider_straight_win: None,
                game_difference: None,
                win_flag_diff: None,
                game_data: gdb,
            }
        }

        pub fn has_three_win(&mut self) {
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
            } else {
                self.has_three_win = Some(false);
            }

            println!("{:?}", scan);
        }

        pub fn is_within_unit(self) {
            let selector;
            if self.game_data.selected_team.unwrap() % 2 == 0 {
                selector = "odd.highlight".to_string();
            } else {
                selector = "even.highlight".to_string();
            };
            let team_selector: Selector = Selector::parse(&format!("tr.{selector}")).unwrap();
            let frag = self.document.select(&team_selector);
            println!("fraaag {frag:?}");
        }
    }

    struct GameData<'a> {
        team_names: Option<Vec<&'a str>>,
        last_five: Option<Vec<String>>,
        selected_team: Option<usize>,
        document: &'a Html,
    }

    pub struct GameDataBuilder<'a> {
        team_names: Option<Vec<&'a str>>,
        last_five: Option<Vec<String>>,
        selected_team: Option<usize>,
        document: &'a Html,
    }

    impl<'a> GameDataBuilder<'a> {
        pub fn new(document: &'a Html) -> Self {
            GameDataBuilder {
                team_names: None,
                last_five: None,
                selected_team: None,
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
        let mut game_rule = GameRule::new(&document);
        game_rule.has_three_win();

        println!("yooo {:?}", game_rule.is_within_unit());
        Ok(())
    }
}
