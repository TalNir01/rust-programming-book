use rand::RngExt;
use std::cmp::Ordering;
use std::fmt;
use std::io::{self, Write};

/// Fallback upper bound for the secret number when the `MAX_GUESS`
/// build-time environment variable is not set.
const DEFAULT_MAX_GUESS: u32 = 100;

/// Upper bound (inclusive) of the guessable range, i.e. `1..=MAX_GUESS`.
///
/// Resolved at compile time from the `MAX_GUESS` environment variable via
/// `option_env!`, falling back to [`DEFAULT_MAX_GUESS`] when unset. Building
/// with an invalid or zero value panics at compile time.
const MAX_GUESS: u32 = match option_env!("MAX_GUESS") {
    None => DEFAULT_MAX_GUESS,
    Some(s) => match u32::from_str_radix(s, 10) {
        Ok(n) if n >= 1 => n,
        Ok(_) => panic!("MAX_GUESS must be at least 1"),
        Err(_) => panic!("MAX_GUESS must be a valid u32"),
    },
};

/// Running tally of every guess outcome for a single game.
///
/// Each field counts one category of guess result:
/// - `too_small`: valid, in-range guesses below the secret number.
/// - `too_big`: valid, in-range guesses above the secret number.
/// - `out_of_range`: guesses that parsed as a `u32` but fell outside `1..=MAX_GUESS`.
/// - `invalid`: input that failed to parse as a `u32` at all.
///
/// The winning guess itself is not tallied in any field; it is accounted for
/// separately by [`Stats::valid_guesses`].
#[derive(Default)]
struct Stats {
    too_small: u32,
    too_big: u32,
    out_of_range: u32,
    invalid: u32,
}

impl Stats {
    /// Number of in-range guesses made, including the final winning guess.
    ///
    /// Computed as `too_big + too_small + 1`, using `saturating_add` so the
    /// count clamps at `u32::MAX` instead of overflowing on a pathologically
    /// long game.
    fn valid_guesses(&self) -> u32 {
        self.too_big
            .saturating_add(self.too_small)
            .saturating_add(1)
    }

    /// Total number of guesses made over the whole game.
    ///
    /// Equal to [`Stats::valid_guesses`] plus `out_of_range` plus `invalid`,
    /// i.e. every guess the player entered, valid or not.
    fn total_attempts(&self) -> u32 {
        self.valid_guesses()
            .saturating_add(self.out_of_range)
            .saturating_add(self.invalid)
    }
}

/// Renders the [`Stats`] as a boxed summary table, e.g.:
///
/// ```text
/// ╔══════════════════════════════╗
/// ║       GAME STATISTICS        ║
/// ╠══════════════════════════════╣
/// ║ Total attempts             3 ║
/// ║ Too small                  1 ║
/// ║ Too big                    1 ║
/// ║ Out of range               0 ║
/// ║ Invalid input              0 ║
/// ╚══════════════════════════════╝
/// ```
impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const INNER: usize = 30;
        let bar = "═".repeat(INNER);

        writeln!(f, "╔{bar}╗")?;
        writeln!(f, "║{:^INNER$}║", "GAME STATISTICS")?;
        writeln!(f, "╠{bar}╣")?;
        writeln!(
            f,
            "║ {:<20}{:>8} ║",
            "Total attempts",
            self.total_attempts()
        )?;
        writeln!(f, "║ {:<20}{:>8} ║", "Too small", self.too_small)?;
        writeln!(f, "║ {:<20}{:>8} ║", "Too big", self.too_big)?;
        writeln!(f, "║ {:<20}{:>8} ║", "Out of range", self.out_of_range)?;
        writeln!(f, "║ {:<20}{:>8} ║", "Invalid input", self.invalid)?;
        write!(f, "╚{bar}╝")
    }
}

/// Runs the guessing game.
///
/// Picks a random secret number in `1..=MAX_GUESS`, then repeatedly prompts
/// the player for a guess on stdin, reporting whether it was too small, too
/// big, out of range, or unparseable. The loop exits once the player guesses
/// correctly, after which the accumulated [`Stats`] are printed.
fn main() {
    // Pick the number the player has to guess, uniformly from the valid range.
    let secret_number = rand::rng().random_range(1..=MAX_GUESS);
    println!("Welcome to the guessing game!");
    let mut stats = Stats::default();
    loop {
        // Prompt without a newline, so flush explicitly or the text may sit
        // in the stdout buffer instead of appearing before we block on input.
        print!("Pick a number from 1 to {MAX_GUESS}: ");
        io::stdout().flush().expect("Stdout flush error");

        // read_line appends to the buffer (including the trailing '\n'),
        // so it must be trimmed before parsing.
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        // Parse and range-check in one step via the `1..=MAX_GUESS` pattern.
        // Anything that parses but falls outside that range, or fails to
        // parse at all, is tallied and re-prompted via `continue`.
        let guess_num = match guess.trim().parse::<u32>() {
            Ok(n @ 1..=MAX_GUESS) => n,
            Ok(_) => {
                println!("Number must be between 1 and {MAX_GUESS}.");
                stats.out_of_range += 1;
                continue;
            }
            Err(_) => {
                println!("Please enter a valid number.");
                stats.invalid += 1;
                continue;
            }
        };

        // Compare the guess to the secret number and update stats
        // accordingly; an exact match ends the game.
        match guess_num.cmp(&secret_number) {
            Ordering::Less => {
                stats.too_small += 1;
                println!("Too small!");
            }
            Ordering::Greater => {
                stats.too_big += 1;
                println!("Too big!");
            }
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }

    println!("{stats}");
}
