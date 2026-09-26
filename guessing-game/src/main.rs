use rand::RngExt;
use std::cmp::Ordering;
use std::fmt;
use std::io::{self, Write};
const DEFAULT_MAX_GUESS: u32 = 100;
const MAX_GUESS: u32 = match option_env!("MAX_GUESS") {
    None => DEFAULT_MAX_GUESS,
    Some(s) => match u32::from_str_radix(s, 10) {
        Ok(n) if n >= 1 => n,
        Ok(_) => panic!("MAX_GUESS must be at least 1"),
        Err(_) => panic!("MAX_GUESS must be a valid u32"),
    },
};

/// Stat struct collect game statistics.
///
/// # Fields
///
/// - `too_small` (`u32`) - Amount of guesses that were `too-small`
/// - `too_big` (`u32`) - Amount of guesses that were `too-big`
/// - `out_of_range` (`u32`) - Amount of guesses that weren't at the valid range (from 1 to MAX_GUESS)
/// - `invalid` (`u32`) - Amount of guess that weren't an positive integer...
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let s = Stat {
///     too_small: value,
///     too_big: value,
///     out_of_range: value,
///     invalid: value,
/// };
/// ```
#[derive(Default)]
struct Stats {
    too_small: u32,
    too_big: u32,
    out_of_range: u32,
    invalid: u32,
}

impl Stats {
    /// Total amount of valid guess until success
    /// Number of "overshot" plus "undershot" plus 1 (for the exact match)
    /// To prevent any case of overflow of u32 I have used `saturating_add`
    /// # Arguments
    ///
    /// - `&self` (`undefined`) - Describe this parameter.
    ///
    /// # Returns
    ///
    /// - `u32` - Describe the return value.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = valid_guesses();
    /// ```
    fn valid_guesses(&self) -> u32 {
        self.too_big
            .saturating_add(self.too_small)
            .saturating_add(1)
    }

    /// All valid guess (that lead to victory) plus the invalid and the ones out of range
    ///
    /// # Arguments
    ///
    /// - `&self` (`undefined`) - Describe this parameter.
    ///
    /// # Returns
    ///
    /// - `u32` - Describe the return value.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::...;
    ///
    /// let _ = total_attempts();
    /// ```
    fn total_attempts(&self) -> u32 {
        self.valid_guesses()
            .saturating_add(self.out_of_range)
            .saturating_add(self.invalid)
    }
}

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

/// Guessing game entrypoint.
fn main() {
    let secret_number = rand::rng().random_range(1..=MAX_GUESS);
    println!("Welcome to the guessing game!");
    let mut stats = Stats::default();
    loop {
        print!("Pick a number from 1 to {MAX_GUESS}: ");
        io::stdout().flush().expect("Stdout flush error");

        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

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
