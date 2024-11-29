use anyhow::{Context, Result};
use core::str;
use std::process::{exit, Command};

fn main() {
    let Ok(output_string) = Command::new("xrandr")
        .arg("-q")
        .output()
        .context("During launching xrandr an error occured")
        .map_err(|e| {
            eprintln!("{}", e);
        })
    else {
        exit(1);
    };
    let Ok(utf8_out) = &str::from_utf8(&output_string.stdout) else {
        eprintln!("Non utf8 characters encountered when parsing xrandr output.");
        exit(1);
    };
    let Ok(monitors) = Monitors::from_cli_text(utf8_out)
        .map_err(|e| eprintln!("Parseing the output of xrandr failed due to {}", e))
    else {
        exit(1);
    };
    if monitors.monitors.is_empty() {
        eprintln!("No active monitors found.");
        exit(1);
    }
    let mut biggest_monitor = &monitors.monitors[0];
    for monitor in &monitors.monitors {
        if monitor.width > biggest_monitor.width {
            biggest_monitor = monitor;
        }
    }
    let name = biggest_monitor.name.clone();
    let commands = monitors
        .monitors
        .into_iter()
        .flat_map(|m| m.set_strings(m.name == name))
        .collect::<Vec<String>>();
    Command::new("xrandr")
        .args(commands)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

#[derive(Debug)]
struct Monitors {
    monitors: Vec<Monitor>,
}

impl Monitors {
    fn from_cli_text(xrandr_outputs: &str) -> Result<Monitors> {
        let chunks = parse_xrandr_monitors(xrandr_outputs);
        let alive_monitors = chunks
            .into_iter()
            .skip(1)
            .filter(|vec| !vec[0].contains("disconnected"))
            .map(Monitor::parse_max_from_chunk)
            .collect::<Result<Vec<Monitor>>>()
            .context("Failure during parsing out monitor details")?;
        Ok(Monitors {
            monitors: alive_monitors,
        })
    }
}

#[derive(Debug)]
struct Monitor {
    height: usize,
    width: usize,
    name: String,
    refresh: String,
}
impl Monitor {
    fn set_strings(&self, on: bool) -> Vec<String> {
        if on {
            return vec![
                "--output".into(),
                self.name.clone(),
                "--mode".into(),
                format!("{}x{}", self.width, self.height),
            ];
        };
        vec!["--output".into(), self.name.clone(), "--off".into()]
    }
    fn parse_max_from_chunk(chunk: impl AsRef<[String]>) -> Result<Monitor> {
        let chunk = chunk.as_ref();
        let (name, _) = chunk[0]
            .split_once(' ')
            .context(format!("Splitting line for name failed: {:?}", &chunk))?;
        let max_res = chunk[1].trim();
        let (max_res, refreshrate) = max_res.split_once(' ').context(format!(
            "Can't find max_refreshrate and resolution from: {}",
            max_res
        ))?;
        let (primary_refreshrate, _) = refreshrate
            .trim()
            .split_once(' ')
            .context(format!(
                "Couldn't parse refreshrate from string {}",
                refreshrate
            ))
            .unwrap_or_else(|_| (refreshrate.trim(), ""));
        let (width, height) = max_res
            .split_once('x')
            .context(format!("Expect reslotion to be widthxheight: {}", max_res))?;
        let width: usize = width
            .parse()
            .context("Height and width should be well bounded integers.")?;
        let height = height
            .parse()
            .context("Height and width should be well bounded integers.")?;
        Ok(Monitor {
            name: String::from(name),
            width,
            height,
            refresh: String::from(primary_refreshrate),
        })
    }
    // Used to asses state of displays.
    fn get_current_from_list(listactivemonitors: &str) -> Result<Vec<Monitor>> {
        for line in listactivemonitors.lines().skip(1) {
            todo!();
        }
        Ok(vec![])
    }
}

fn parse_xrandr_monitors(xrandr_outputs: &str) -> Vec<Vec<String>> {
    let mut chunks = Vec::new();
    let mut lines: Vec<String> = xrandr_outputs.trim().lines().map(String::from).collect();
    let mut peak;
    while lines.len() > 1 {
        peak = 1;
        let mut peak_line = &lines[peak];
        while !peak_line.contains("connected") {
            match lines.get(peak) {
                Some(p) => {
                    peak_line = p;
                }
                None => break,
            };
            peak += 1
        }
        // Double allocation of the print could probably just steal this out of the buffer.
        chunks.push(lines.drain(..peak).collect());
    }
    chunks
}

#[cfg(test)]
mod test {

    use super::*;

    const OUTPUT: &str = "
Screen 0: minimum 320 x 200, current 2560 x 1440, maximum 16384 x 16384
eDP-1 connected primary (normal left inverted right x axis y axis)
   1920x1200     60.10 +  60.10    40.06
   1920x1080     60.10
   1600x1200     60.10
   1680x1050     60.10
   1400x1050     60.10
   1600x900      60.10
   1280x1024     60.10
   1400x900      60.10
   1280x960      60.10
   1440x810      60.10
   1368x768      60.10
   1280x800      60.10
   1280x720      60.10
   1024x768      60.10
   960x720       60.10
   928x696       60.10
   896x672       60.10
   1024x576      60.10
   960x600       60.10
   960x540       60.10
   800x600       60.10
   840x525       60.10
   864x486       60.10
   700x525       60.10
   800x450       60.10
   640x512       60.10
   700x450       60.10
   640x480       60.10
   720x405       60.09
   684x384       60.10
   640x360       60.09
   512x384       60.10
   512x288       60.09
   480x270       60.09
   400x300       60.10
   432x243       60.09
   320x240       60.10
   360x202       60.09
   320x180       60.09
DP-1 disconnected (normal left inverted right x axis y axis)
HDMI-1 disconnected (normal left inverted right x axis y axis)
DP-2 disconnected (normal left inverted right x axis y axis)
HDMI-2 disconnected (normal left inverted right x axis y axis)
DP-3 disconnected (normal left inverted right x axis y axis)
HDMI-3 disconnected (normal left inverted right x axis y axis)
HDMI-4 disconnected (normal left inverted right x axis y axis)
DP-1-0 disconnected (normal left inverted right x axis y axis)
DP-1-1 disconnected (normal left inverted right x axis y axis)
DP-1-2 disconnected (normal left inverted right x axis y axis)
DP-1-3 disconnected (normal left inverted right x axis y axis)
HDMI-1-0 connected 2560x1440+0+0 (normal left inverted right x axis y axis) 597mm x 336mm
   2560x1440     59.95*+
   2048x1080     60.00
   1920x1200     59.88
   1920x1080     60.00    59.94    50.00
   1680x1050     59.95
   1600x1200     60.00
   1280x1024     75.02    60.02
   1280x800      59.81
   1280x720      59.94    50.00
   1152x864      75.00
   1024x768      75.03    60.00
   800x600       75.00    60.32
   720x576       50.00
   720x480       59.94
   640x480       75.00    59.94    59.93";

    #[test]
    fn test_parse() {
        let chunks = parse_xrandr_monitors(OUTPUT);
        let chunk_str = chunks
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("\n");
        assert_eq!(chunk_str.trim(), OUTPUT.trim());
    }

    #[test]
    fn test_monitor_parse() {
        let monitors = parse_xrandr_monitors(OUTPUT.trim_end())
            .into_iter()
            .filter(|chunks| !chunks[0].contains("disconnected") && chunks[0].contains("connected"))
            .map(Monitor::parse_max_from_chunk)
            .collect::<Result<Vec<Monitor>>>()
            .map_err(|e| {
                eprintln!("{}", e);
                e
            });

        assert_eq!(monitors.unwrap().len(), 2)
    }
}
