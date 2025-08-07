use chrono::{FixedOffset, Offset, Utc};
use itertools::Itertools;
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Styled},
    widgets::StatefulWidget,
};

use crate::{
    candle::{Candle, CandleType},
    candlestick_chart_state::CandleStikcChartInfo,
    symbols::*,
    x_axis::{Interval, XAxis},
    y_axis::{Numeric, YAxis},
    CandleStickChartState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartFitMode {
    /// Fixed scale - chart maintains consistent scale regardless of window size
    Fixed,
    /// Fit to available space - chart automatically stretches or compresses to fit
    Fit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleStickChart {
    /// Candle interval
    interval: Interval,
    /// Candle data
    candles: Vec<Candle>,
    /// y axis scale/precision
    numeric: Numeric,
    /// Widget style
    style: Style,
    /// Candle style,
    bearish_color: Color,
    bullish_color: Color,
    /// Wick colors
    bearish_wick_color: Color,
    bullish_wick_color: Color,
    /// display timezone
    display_timezone: FixedOffset,
    /// show/hide y axis
    show_y_axis: bool,
    /// show/hide x axis
    show_x_axis: bool,
    /// Chart fitting mode
    fit_mode: ChartFitMode,
}

impl CandleStickChart {
    pub fn new(interval: Interval) -> Self {
        Self {
            interval,
            candles: Vec::default(),
            numeric: Numeric::default(),
            style: Style::default(),
            bearish_color: Color::Rgb(234, 74, 90),
            bullish_color: Color::Rgb(52, 208, 88),
            bearish_wick_color: Color::Rgb(234, 74, 90),  // Same as body by default
            bullish_wick_color: Color::Rgb(52, 208, 88),  // Same as body by default
            display_timezone: Utc.fix(),
            show_y_axis: true,
            show_x_axis: true,
            fit_mode: ChartFitMode::Fixed,  // Default to fixed mode
        }
    }

    pub fn candles(mut self, candles: Vec<Candle>) -> Self {
        self.candles = candles;
        self
    }

    pub fn y_axis_numeric(mut self, numeric: Numeric) -> Self {
        self.numeric = numeric;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn bearish_color(mut self, color: Color) -> Self {
        self.bearish_color = color;
        self
    }

    pub fn bullish_color(mut self, color: Color) -> Self {
        self.bullish_color = color;
        self
    }

    pub fn bearish_wick_color(mut self, color: Color) -> Self {
        self.bearish_wick_color = color;
        self
    }

    pub fn bullish_wick_color(mut self, color: Color) -> Self {
        self.bullish_wick_color = color;
        self
    }

    pub fn display_timezone(mut self, offset: FixedOffset) -> Self {
        self.display_timezone = offset;
        self
    }

    pub fn show_y_axis(mut self, show: bool) -> Self {
        self.show_y_axis = show;
        self
    }

    pub fn show_x_axis(mut self, show: bool) -> Self {
        self.show_x_axis = show;
        self
    }

    pub fn fit_mode(mut self, mode: ChartFitMode) -> Self {
        self.fit_mode = mode;
        self
    }
}

impl Styled for CandleStickChart {
    type Item = CandleStickChart;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style.into())
    }
}

impl StatefulWidget for CandleStickChart {
    type State = CandleStickChartState;

    /// render like:
    /// |---|-----------------------|
    /// | y |                       |
    /// |   |                       |
    /// | a |                       |
    /// | x |                       |
    /// | i |                       |
    /// | s |       chart data      |
    /// |   |                       |
    /// | a |                       |
    /// | r |                       |
    /// | e |                       |
    /// | a |                       |
    /// |---|-----------------------|
    ///     |      x axis area      |
    ///     |-----------------------|
    ///
    ///
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if self.candles.is_empty() {
            return;
        }

        let global_min = self.candles.iter().map(|c| c.low).min().unwrap();
        let global_max = self.candles.iter().map(|c| c.high).max().unwrap();

        let y_axis_width: u16 = if self.show_y_axis {
            YAxis::estimated_width(self.numeric.clone(), global_min, global_max)
        } else {
            0
        };
        let x_axis_height: u16 = if self.show_x_axis { 3 } else { 0 };
        
        if area.width <= y_axis_width || area.height <= x_axis_height {
            return;
        }

        let chart_width = area.width - y_axis_width;
        let chart_width_usize = chart_width as usize;

        // with first/last dummies
        let first_timestamp = self.candles.first().unwrap().timestamp;
        let last_timestamp = self.candles.last().unwrap().timestamp;

        let mut candles = Vec::new();
        for i in (1..=(chart_width as i64 - 1)).rev() {
            candles.push(
                Candle::new(
                    first_timestamp - i * self.interval as i64 * 1000,
                    0.,
                    0.,
                    0.,
                    0.,
                )
                .unwrap(),
            );
        }
        candles.extend(self.candles.clone());
        for i in 1..=(chart_width as i64 - 1) {
            candles.push(
                Candle::new(
                    last_timestamp + i * self.interval as i64 * 1000,
                    0.,
                    0.,
                    0.,
                    0.,
                )
                .unwrap(),
            );
        }

        let chart_end_timestamp = state.cursor_timestamp.unwrap_or(last_timestamp);
        let chart_start_timestamp =
            chart_end_timestamp - self.interval as i64 * 1000 * (chart_width_usize as i64 - 1);
        let rendered_candles = candles
            .iter()
            .filter(|c| c.timestamp >= chart_start_timestamp && c.timestamp <= chart_end_timestamp)
            .collect_vec();

        state.set_info(CandleStikcChartInfo::new(
            candles[chart_width_usize - 1].timestamp,
            candles.last().unwrap().timestamp,
            self.interval,
            last_timestamp,
            rendered_candles.first().unwrap().timestamp < first_timestamp,
        ));

        let y_min = rendered_candles
            .iter()
            .filter(|c| c.timestamp >= first_timestamp && c.timestamp <= last_timestamp)
            .map(|c| c.low)
            .min()
            .unwrap();
        let y_max = rendered_candles
            .iter()
            .filter(|c| c.timestamp >= first_timestamp && c.timestamp <= last_timestamp)
            .map(|c| c.high)
            .max()
            .unwrap();

        let y_axis = YAxis::new(Numeric::default(), area.height - x_axis_height, y_min, y_max);
        if self.show_y_axis {
            let rendered_y_axis = y_axis.render();
            for (y, string) in rendered_y_axis.iter().enumerate() {
                buf.set_string(area.x, y as u16 + area.y, string, Style::default());
            }
        }

        let timestamp_min = rendered_candles.first().unwrap().timestamp;
        let timestamp_max = rendered_candles.last().unwrap().timestamp;

        if self.show_x_axis {
            let x_axis = XAxis::new(
                chart_width,
                timestamp_min,
                timestamp_max,
                self.interval,
                state.cursor_timestamp.is_none(),
            );
            let rendered_x_axis = x_axis.render(self.display_timezone);
            if self.show_y_axis {
                buf.set_string(area.x + y_axis_width - 2, area.y + area.height - 3, "└──", Style::default());
            }
            for (y, string) in rendered_x_axis.iter().enumerate() {
                buf.set_string(
                    area.x + y_axis_width,
                    area.y + area.height - 3 + y as u16,
                    string,
                    Style::default(),
                );
            }
        }

        // Calculate candle width and spacing distribution, or merge candles for squashing
        let (processed_candles, candle_width, extra_spaces, _) = match self.fit_mode {
            ChartFitMode::Fixed => {
                let data_candles: Vec<Candle> = rendered_candles.iter()
                    .filter(|c| c.timestamp >= first_timestamp && c.timestamp <= last_timestamp)
                    .map(|&c| c.clone())
                    .collect();
                (data_candles, 1u16, 0u16, 0usize)
            },
            ChartFitMode::Fit => {
                let data_candles: Vec<Candle> = rendered_candles.iter()
                    .filter(|c| c.timestamp >= first_timestamp && c.timestamp <= last_timestamp)
                    .map(|&c| c.clone())
                    .collect();
                    
                if data_candles.is_empty() {
                    (data_candles, 1u16, 0u16, 0usize)
                } else if data_candles.len() > chart_width as usize {
                    // Squashing: merge candles
                    let merge_ratio = (data_candles.len() + chart_width as usize - 1) / chart_width as usize; // Ceiling division
                    let mut merged_candles = Vec::new();
                    
                    for chunk in data_candles.chunks(merge_ratio) {
                        if !chunk.is_empty() {
                            // Create merged candle: first open, last close, min low, max high
                            let merged = Candle::new(
                                chunk[0].timestamp, // Use first timestamp
                                chunk[0].open.into(),
                                chunk.iter().map(|c| c.high).max().unwrap().into(),
                                chunk.iter().map(|c| c.low).min().unwrap().into(),
                                chunk[chunk.len() - 1].close.into()
                            ).unwrap();
                            merged_candles.push(merged);
                        }
                    }
                    
                    (merged_candles, 1u16, 0u16, 0usize)
                } else {
                    // Stretching: normal logic
                    let base_width = std::cmp::max(1, chart_width / data_candles.len() as u16);
                    let used_width = base_width * data_candles.len() as u16;
                    let extra_spaces = chart_width.saturating_sub(used_width);
                    
                    (data_candles, base_width, extra_spaces, 0)
                }
            }
        };
        
        let mut candle_index = 0;
        let mut current_x_offset = 0u16;
        
        // Pre-calculate where extra spaces should go for even distribution
        let mut space_positions = vec![false; processed_candles.len()];
        if extra_spaces > 0 && processed_candles.len() > 1 {
            let gaps = processed_candles.len() - 1;
            for i in 0..extra_spaces as usize {
                if i < gaps {
                    // Distribute spaces evenly across gaps using floating point for precision
                    let position = ((i as f64 + 0.5) * gaps as f64 / extra_spaces as f64) as usize;
                    if position < space_positions.len() - 1 {
                        space_positions[position] = true;
                    }
                }
            }
        }
        
        for candle in processed_candles.iter() {
            let (body_color, wick_color) = match if candle.open <= candle.close {
                CandleType::Bullish
            } else {
                CandleType::Bearish
            } {
                CandleType::Bearish => (self.bearish_color, self.bearish_wick_color),
                CandleType::Bullish => (self.bullish_color, self.bullish_wick_color),
            };

            if candle_width == 1 && extra_spaces == 0 {
                // Use normal rendering
                let (_, rendered) = candle.render(&y_axis);
                for (y, char) in rendered.iter().enumerate() {
                    let cell_x = candle_index as u16 + y_axis_width + area.x;
                    let cell_y = y as u16 + area.y;
                    if cell_x < area.x + area.width && let Some(cell) = buf.cell_mut((cell_x, cell_y)) {
                        // Determine if this character is a wick or body
                        let is_wick = matches!(*char, UNICODE_WICK | UNICODE_HALF_WICK_BOTTOM | UNICODE_HALF_WICK_TOP);
                        let color = if is_wick { wick_color } else { body_color };
                        
                        cell.set_symbol(char)
                            .set_style(Style::default().fg(color));
                    }
                }
            } else {
                // Use stretched rendering with pre-calculated spacing
                let (_, stretched_rendered) = if candle_width > 1 {
                    candle.render_stretched(&y_axis, candle_width)
                } else {
                    candle.render_stretched(&y_axis, 1)
                };
                
                for (y, row) in stretched_rendered.iter().enumerate() {
                    for (dx, char) in row.iter().enumerate() {
                        let cell_x = current_x_offset + dx as u16 + y_axis_width + area.x;
                        let cell_y = y as u16 + area.y;
                        if cell_x < area.x + area.width && let Some(cell) = buf.cell_mut((cell_x, cell_y)) {
                            // Determine if this character is a wick or body
                            let is_wick = matches!(*char, UNICODE_WICK | UNICODE_RIGHT_EIGHTH_BLOCK | UNICODE_LEFT_EIGHTH_BLOCK | UNICODE_HALF_WICK_BOTTOM | UNICODE_HALF_WICK_TOP);
                            let color = if is_wick { wick_color } else { body_color };
                            
                            cell.set_symbol(char)
                                .set_style(Style::default().fg(color));
                        }
                    }
                }
                
                // Move to next position
                current_x_offset += candle_width;
                
                // Add extra space if this position is marked for it
                if candle_index < space_positions.len() && space_positions[candle_index] {
                    current_x_offset += 1;
                }
            }
            candle_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        assert_buffer_eq,
        buffer::{Buffer, Cell},
        layout::Rect,
        style::{Style, Stylize},
        widgets::StatefulWidget,
    };

    use crate::{Candle, CandleStickChart, CandleStickChartState, Interval};

    fn render(widget: CandleStickChart, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let cell = Cell::new("x");
        let mut buffer = Buffer::filled(area, cell);
        widget.render(area, &mut buffer, &mut CandleStickChartState::default());
        buffer.set_style(area, Style::default().reset());
        buffer
    }

    #[test]
    fn empty_candle() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![]);
        let buffer = render(widget, 14, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
                "xxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle() {
        let widget = CandleStickChart::new(Interval::OneMinute)
            .candles(vec![Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 14, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     3.000 ├ │",
                "           │ │",
                "           │ ┃",
                "           │ │",
                "     0.600 ├ │",
                "xxxxxxxxxxx└──",
                "xxxxxxxxxxxxx ",
                "xxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute)
            .candles(vec![Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap()]);
        let buffer = render(widget, 30, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     3.000 ├ xxxxxxxxxxxxxxxx│",
                "           │ xxxxxxxxxxxxxxxx│",
                "           │ xxxxxxxxxxxxxxxx┃",
                "           │ xxxxxxxxxxxxxxxx│",
                "     0.600 ├ xxxxxxxxxxxxxxxx│",
                "xxxxxxxxxxx└─────────────────┴",
                "xxxxxxxxxxxxx*1970/01/01 00:00",
                "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(60000, 2.1, 4.2, 2.1, 3.9).unwrap(),
            Candle::new(120000, 3.9, 4.1, 2.0, 2.3).unwrap(),
        ]);
        let buffer = render(widget, 19, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     4.200 ├ xxx ╽┃",
                "           │ xxx│┃┃",
                "           │ xxx│╹╿",
                "           │ xxx│  ",
                "     0.840 ├ xxx│  ",
                "xxxxxxxxxxx└──────┴",
                "xxxxxxxxxxxxx*00:02",
                "xxxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_full_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(60000, 2.1, 4.2, 2.1, 3.9).unwrap(),
            Candle::new(120000, 3.9, 4.1, 2.0, 2.3).unwrap(),
            Candle::new(180000, 2.3, 3.9, 1.3, 2.0).unwrap(),
            Candle::new(240000, 2.0, 5.2, 0.9, 3.9).unwrap(),
        ]);
        let buffer = render(widget, 19, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     5.200 ├ x ╷  │",
                "           │ x ╽┃││",
                "           │ x│┃╿│┃",
                "           │ x┃ ╵││",
                "     1.040 ├ x│   ╵",
                "xxxxxxxxxxx└──────┴",
                "xxxxxxxxxxxxx*00:04",
                "xxxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_omitted_candles_with_x_label() {
        let widget = CandleStickChart::new(Interval::OneMinute).candles(vec![
            Candle::new(0, 0.9, 3.0, 0.0, 2.1).unwrap(),
            Candle::new(240000, 2.0, 5.2, 0.9, 3.9).unwrap(),
        ]);
        let buffer = render(widget, 19, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "     5.200 ├ x xxx│",
                "           │ x xxx│",
                "           │ x│xxx┃",
                "           │ x┃xxx│",
                "     1.040 ├ x│xxx╵",
                "xxxxxxxxxxx└──────┴",
                "xxxxxxxxxxxxx*00:04",
                "xxxxxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle_with_not_changing() {
        let widget = CandleStickChart::new(Interval::OneSecond).candles(vec![
            Candle::new(0, 0.0, 1000.0, 0.0, 50.0).unwrap(),
            Candle::new(1000, 50.0, 50.0, 50.0, 50.0).unwrap(),
            Candle::new(2000, 500.0, 500.0, 500.0, 500.0).unwrap(),
        ]);
        let buffer = render(widget, 16, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "  1000.000 ├ │  ",
                "           │ │  ",
                "           │ │ ╻",
                "           │ │  ",
                "   200.000 ├ │╻ ",
                "xxxxxxxxxxx└────",
                "xxxxxxxxxxxxx   ",
                "xxxxxxxxxxxxxxxx",
            ])
        );
    }

    #[test]
    fn simple_candle_with_small_candle() {
        let widget = CandleStickChart::new(Interval::OneSecond).candles(vec![
            Candle::new(0, 0.0, 1000.0, 0.0, 50.0).unwrap(),
            Candle::new(1000, 450.0, 580.0, 320.0, 450.0).unwrap(),
            Candle::new(2000, 580.0, 580.0, 320.0, 320.0).unwrap(),
        ]);
        let buffer = render(widget, 16, 8);
        assert_eq!(
            buffer,
            Buffer::with_lines(vec![
                "  1000.000 ├ │  ",
                "           │ │  ",
                "           │ │╽┃",
                "           │ │╵╹",
                "   200.000 ├ │  ",
                "xxxxxxxxxxx└────",
                "xxxxxxxxxxxxx   ",
                "xxxxxxxxxxxxxxxx",
            ])
        );
    }
}
