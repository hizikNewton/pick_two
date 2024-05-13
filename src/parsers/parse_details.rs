pub mod parse_details {
    use std::fs::File;
    use std::io::Read;

    use scraper::{selectable::Selectable, Html, Selector};

    pub struct GameRule<'a> {
        document: &'a Html,
        has_three_win: Option<bool>,
        is_within_unit: Option<bool>,
        is_popular: Option<bool>,
        consider_straight_win: Option<bool>,
        game_difference: Option<i32>,
        win_flag_diff: Option<i32>,
    }
    impl<'a> GameRule<'a> {
        pub fn new(document: &'a Html) -> Self {
            GameRule {
                document,
                has_three_win: None,
                is_within_unit: None,
                is_popular: None,
                consider_straight_win: None,
                game_difference: None,
                win_flag_diff: None,
            }
        }

        pub fn has_three_win(self) {
            // let container_right_selector = Selector::parse("div.last-five").unwrap();

            let last_five = Selector::parse("div.last-five").unwrap();

            for mw in self.document.select(&last_five) {
                let container = Html::parse_fragment(&mw.inner_html());
                let a_selector = Selector::parse("a").unwrap();

                for mt in container.select(&a_selector) {
                    println!("{:?}", mt.text());
                }
            }
        }
    }

    pub fn game_selector() -> std::io::Result<()> {
        let mut file = File::open("src/y.html")?;
        let mut html = String::new();

        if let Ok(_size) = file.read_to_string(&mut html) {
            log::debug!("succesfully read to string");
        };
        let document = Html::parse_document(&html);
        let game_rule = GameRule::new(&document);
        game_rule.has_three_win();
        Ok(())
    }
}
