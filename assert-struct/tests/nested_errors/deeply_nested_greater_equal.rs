use assert_struct::assert_struct;

#[derive(Debug)]
struct Score {
    points: u32,
    bonus: u32,
}

#[derive(Debug)]
struct Player {
    id: u64,
    score: Score,
}

#[derive(Debug)]
struct Game {
    session: String,
    player: Player,
}

pub fn test_case() {
    let game = Game {
        session: "xyz789".to_string(),
        player: Player {
            id: 12345,
            score: Score {
                points: 80,
                bonus: 5,
            },
        },
    };

    assert_struct!(game, Game {
        session: "xyz789",
        player: Player {
            id: 12345,
            score: Score {
                points: >= 100,  // Line 38 - should report this line (80 is not >= 100)
                bonus: 5,
            },
        },
    });
}