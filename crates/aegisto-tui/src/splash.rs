//! Startup splash screen: animated ASCII banner in the Aegisto blue palette.
//!
//! Renders the AEGISTO banner inside the terminal's *alternate screen* with a
//! moving color wave derived from the brand color `#2171B5` (`rgb(33, 113, 181)`),
//! then restores the normal screen before the CLI takes over.
//!
//! - Press `q` or `Esc` to skip the splash and continue.
//! - Press `Ctrl+C` to abort the run (the terminal is always restored).

use std::fmt::Write as _;
use std::io::{IsTerminal, Write, stdin, stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyCode, KeyModifiers, poll, read};
use crossterm::execute;
use crossterm::style::{Color, SetForegroundColor};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};

/// How long the splash stays on screen before the CLI starts.
const SPLASH_DURATION: Duration = Duration::from_secs(4);

/// Frame interval of the wave animation (~30 fps).
const FRAME_DURATION: Duration = Duration::from_millis(33);

/// Wave speed in radians per second.
const WAVE_SPEED: f32 = 2.6;

/// Wave density along the x axis (per character column).
const WAVE_FREQ_X: f32 = 0.12;

/// Wave density along the y axis (per row).
const WAVE_FREQ_Y: f32 = 0.09;

/// Palette shades derived from the brand color `#2171B5`.
mod palette {
    /// Darkest shade — wave troughs.
    pub const DARK: (u8, u8, u8) = (8, 48, 107); // #08306B
    /// Brand base color.
    pub const BASE: (u8, u8, u8) = (33, 113, 181); // #2171B5
    /// Brightest shade — wave crests.
    pub const LIGHT: (u8, u8, u8) = (107, 174, 214); // #6BAED6
}

/// The AEGISTO banner (figlet "standard" font, exactly as provided).
const BANNER: &str = r#"/$$$$$$                      /$$             /$$
 /$$__  $$                    |__/            | $$
| $$  \ $$  /$$$$$$   /$$$$$$  /$$  /$$$$$$$ /$$$$$$    /$$$$$$
| $$$$$$$$ /$$__  $$ /$$__  $$| $$ /$$_____/|_  $$_/   /$$__  $$
| $$__  $$| $$$$$$$$| $$  \ $$| $$|  $$$$$$   | $$    | $$  \ $$
| $$  | $$| $$_____/| $$  | $$| $$ \____  $$  | $$ /$$| $$  | $$
| $$  | $$|  $$$$$$$|  $$$$$$$| $$ /$$$$$$$/  |  $$$$/|  $$$$$$/
|__/  |__/ \_______/ \____  $$|__/|_______/    \___/   \______/
                        /$$  \ $$
                       |  $$$$$$/
                        \______/"#;

/// Subtitle rendered underneath the banner.
const TAGLINE: &str = "AEGISTO — autonomous binary analysis framework";

/// Key hint rendered at the bottom of the screen.
const HINT: &str = "press q to skip";

/// What ended the splash animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplashOutcome {
    /// Ran for the full [`SPLASH_DURATION`].
    Completed,
    /// User pressed `q`/`Esc` and the run should continue.
    Skipped,
    /// User pressed `Ctrl+C` and the process should exit.
    Aborted,
}

/// Run the animated splash screen.
///
/// This is a no-op when stdout or stdin is not attached to a terminal, so
/// piped and redirected output is never polluted with escape sequences and the
/// event reader never consumes redirected input.
pub fn run() -> Result<SplashOutcome> {
    if !stdout().is_terminal() || !stdin().is_terminal() {
        return Ok(SplashOutcome::Completed);
    }
    splash_inner()
}

fn splash_inner() -> Result<SplashOutcome> {
    let _guard = ScreenGuard::enter()?;

    let lines: Vec<&str> = BANNER.lines().collect();
    let banner_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let banner_height = lines.len();

    // Center the banner horizontally and place it in the upper third.
    let (cols, rows) = size().unwrap_or((80, 24));
    let start_x = (cols as usize).saturating_sub(banner_width) / 2;
    let start_y = (rows as usize).saturating_sub(banner_height) / 3;
    let tag_x = (cols as usize).saturating_sub(TAGLINE.chars().count()) / 2;
    let tag_y = start_y + banner_height + 1;

    let mut stdout = stdout();
    let began = Instant::now();
    let mut last_frame = Instant::now();

    loop {
        let elapsed = began.elapsed();
        if elapsed >= SPLASH_DURATION {
            return Ok(SplashOutcome::Completed);
        }

        // Pace frames at FRAME_DURATION even while key events stream in, so a
        // held key cannot turn the render loop into a busy loop.
        let since_last = last_frame.elapsed();
        if since_last < FRAME_DURATION {
            let wait = FRAME_DURATION.saturating_sub(since_last);
            if poll(wait)?
                && let Some(outcome) = handle_key(read()?)?
            {
                return Ok(outcome);
            }
            continue;
        }

        last_frame = Instant::now();
        draw_frame(
            &mut stdout,
            &lines,
            start_x,
            start_y,
            tag_x,
            tag_y,
            rows,
            elapsed,
        )?;
    }
}

/// Render one animated frame into the alternate screen.
#[allow(clippy::too_many_arguments)]
fn draw_frame(
    stdout: &mut impl Write,
    lines: &[&str],
    start_x: usize,
    start_y: usize,
    tag_x: usize,
    tag_y: usize,
    rows: u16,
    elapsed: Duration,
) -> Result<()> {
    // Build the whole frame as one string, then flush it in a single write.
    let mut buf = String::with_capacity(16 * 1024);
    write!(buf, "{}{}", Clear(ClearType::All), MoveTo(0, 0))?;

    for (row, line) in lines.iter().enumerate() {
        write!(
            buf,
            "{}",
            MoveTo(start_x as u16, start_y as u16 + row as u16)
        )?;
        for (col, ch) in line.chars().enumerate() {
            if ch == ' ' {
                buf.push(' ');
                continue;
            }
            let brightness = wave_brightness(col as f32, row as f32, elapsed.as_secs_f32());
            let (r, g, b) = shade(brightness);
            write!(buf, "{}{}", SetForegroundColor(Color::Rgb { r, g, b }), ch)?;
        }
    }

    // Subtitle and key hint in the palette's lightest shade.
    let (tr, tg, tb) = palette::LIGHT;
    write!(
        buf,
        "{}{}{}",
        MoveTo(tag_x as u16, tag_y as u16),
        SetForegroundColor(Color::Rgb {
            r: tr,
            g: tg,
            b: tb
        }),
        TAGLINE
    )?;
    write!(
        buf,
        "{}{}{}",
        MoveTo(0, rows.saturating_sub(1)),
        SetForegroundColor(Color::Rgb {
            r: tr,
            g: tg,
            b: tb
        }),
        HINT
    )?;

    stdout.write_all(buf.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

/// Map a key event to a splash outcome, if the user asked to skip or abort.
fn handle_key(event: Event) -> Result<Option<SplashOutcome>> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(SplashOutcome::Skipped)),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(Some(SplashOutcome::Aborted));
            }
            _ => {}
        }
    }
    Ok(None)
}

/// Brightness in `[0, 1]` for a character at `(x, y)` at time `t` (seconds).
fn wave_brightness(x: f32, y: f32, t: f32) -> f32 {
    let phase = x * WAVE_FREQ_X + y * WAVE_FREQ_Y + t * WAVE_SPEED;
    (phase.sin() * 0.5 + 0.5).clamp(0.0, 1.0)
}

/// Map a brightness to a palette color: `DARK -> BASE -> LIGHT`.
fn shade(brightness: f32) -> (u8, u8, u8) {
    let (a, b, t) = if brightness < 0.5 {
        (palette::DARK, palette::BASE, brightness * 2.0)
    } else {
        (palette::BASE, palette::LIGHT, (brightness - 0.5) * 2.0)
    };
    lerp(a, b, t.clamp(0.0, 1.0))
}

/// Linear interpolation between two RGB colors.
fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

/// Restores the terminal when dropped, so every exit path (including errors)
/// leaves the user's shell in a usable state.
struct ScreenGuard;

impl ScreenGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        if let Err(e) = execute!(stdout(), EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(e.into());
        }
        Ok(Self)
    }
}

impl Drop for ScreenGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_is_bounded() {
        for x in 0..80 {
            for y in 0..12 {
                for t in [0.0_f32, 1.0, 2.5] {
                    let b = wave_brightness(x as f32, y as f32, t);
                    assert!((0.0..=1.0).contains(&b), "brightness {b} out of range");
                }
            }
        }
    }

    #[test]
    fn shade_hits_palette_endpoints() {
        assert_eq!(shade(0.0), palette::DARK);
        assert_eq!(shade(0.5), palette::BASE);
        assert_eq!(shade(1.0), palette::LIGHT);
    }
}
