#[macro_use] extern crate log;
extern crate svgparser;

use std::str;
use std::mem;

use svgparser::{svg, path, Stream};
use svgparser::path::SegmentData::{self, MoveTo, LineTo, HorizontalLineTo, VerticalLineTo};

pub type CoordinatePair = (f64, f64);
pub type Polyline = Vec<CoordinatePair>;


struct CurrentLine {
    line: Polyline,
}

impl CurrentLine {
    fn new() -> Self {
        CurrentLine { line: Polyline::new() }
    }

    /// Add a CoordinatePair to the internal polyline.
    fn add(&mut self, pair: CoordinatePair) {
        self.line.push(pair);
    }

    /// A polyline is only valid if it has more than 1 CoordinatePair.
    fn is_valid(&self) -> bool {
        self.line.len() > 1
    }

    /// Return the last x coordinate (if the line is not empty).
    fn last_x(&self) -> Option<f64> {
        self.line.last().map(|pair| pair.0)
    }
    
    /// Return the last y coordinate (if the line is not empty).
    fn last_y(&self) -> Option<f64> {
        self.line.last().map(|pair| pair.1)
    }

    /// Replace the internal polyline with a new instance and return the
    /// previously stored polyline.
    fn finish(&mut self) -> Polyline {
        let mut tmp = Polyline::new();
        mem::swap(&mut self.line, &mut tmp);
        tmp
    }
}

fn parse_segment_data(data: &SegmentData,
                      current_line: &mut CurrentLine,
                      lines: &mut Vec<Polyline>) -> Result<(), String> {
    match data {
        &MoveTo { x, y } => {
            if current_line.is_valid() {
                lines.push(current_line.finish());
            }
            current_line.add((x, y));
        },
        &LineTo { x, y } => {
            current_line.add((x, y));
        },
        &HorizontalLineTo { x } => {
            match current_line.last_y() {
                Some(y) => current_line.add((x, y)),
                None => return Err("Invalid state: HorizontalLineTo on emtpy CurrentLine".into()),
            }
        },
        &VerticalLineTo { y } => {
            match current_line.last_x() {
                Some(x) => current_line.add((x, y)),
                None => return Err("Invalid state: VerticalLineTo on emtpy CurrentLine".into()),
            }
        },
        d @ _ => {
            return Err(format!("Unsupported segment data: {:?}", d));
        }
    }
    Ok(())
}

fn parse_path(data: Stream) -> Vec<Polyline> {
    debug!("New path");

    let mut lines = Vec::new();

    let mut p = path::Tokenizer::new(data);
    let mut line = CurrentLine::new();
    loop {
        match p.parse_next() {
            Ok(segment_token) => {
                match segment_token {
                    path::SegmentToken::Segment(segment) => {
                        debug!("  Segment data: {:?}", segment.data);
                        parse_segment_data(&segment.data, &mut line, &mut lines).unwrap();
                    },
                    path::SegmentToken::EndOfStream => break,
                }
            },
            Err(e) => {
                warn!("Invalid path segment: {:?}", e);
                break;
            },
        }
    }

    lines
}

pub fn parse(svg: &str) -> Result<Vec<Polyline>, String> {
    let bytes = svg.as_bytes();

    let mut polylines = Vec::new();
    let mut tokenizer = svg::Tokenizer::new(&bytes);
    loop {
        match tokenizer.parse_next() {
            Ok(t) => {
                match t {
                    svg::Token::Attribute(name, value) => {
                        // Process only 'd' attributes
                        if name == b"d" {
                            polylines.extend(parse_path(value));
                        }
                    },
                    svg::Token::EndOfStream => break,
                    _ => {},
                }
            },
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(e.to_string());
            }
        }
    }

    Ok(polylines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_line() {
        let mut line = CurrentLine::new();
        assert_eq!(line.is_valid(), false);
        assert_eq!(line.last_x(), None);
        assert_eq!(line.last_y(), None);
        line.add((1.0, 2.0));
        assert_eq!(line.is_valid(), false);
        assert_eq!(line.last_x(), Some(1.0));
        assert_eq!(line.last_y(), Some(2.0));
        line.add((2.0, 3.0));
        assert_eq!(line.is_valid(), true);
        assert_eq!(line.last_x(), Some(2.0));
        assert_eq!(line.last_y(), Some(3.0));
        let finished = line.finish();
        assert_eq!(finished.len(), 2);
        assert_eq!(finished[0], (1.0, 2.0));
        assert_eq!(finished[1], (2.0, 3.0));
        assert_eq!(line.is_valid(), false);
    }

}
