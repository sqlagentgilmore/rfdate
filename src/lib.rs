use std::fmt::Display;
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub struct Date {
    year: Option<u16>,
    month: Option<u16>,
    day: Option<u16>,
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        for (a, b) in [self.year, self.month, self.day].iter().zip([other.year, other.month, other.day].iter()) {
            if a.is_none() && b.is_some() {
                return Some(std::cmp::Ordering::Less);
            } else if a.is_some() && b.is_none() {
                return Some(std::cmp::Ordering::Greater);
            } else if let (Some(a), Some(b)) = (a, b) {
                match a.cmp(&b) {
                    std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
                    std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
                    std::cmp::Ordering::Equal => continue,
                }
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}

fn is_separator(ch: &char) -> bool {
    matches!(ch, '-' | '/' | '_' | ' ' | '.')
}

pub fn find_dates(s: &str) -> Vec<Result<Date, DateError>> {
    let mut date_holders = DateHolders::new();
    let mut date_holder = DateHolder::new();
    let mut curr_part = Part::new();
    for letter in s.chars() {
        if is_separator(&letter) || letter.is_ascii_digit() {
            if letter.is_ascii_digit() {
                curr_part.push(letter);
            } else if !curr_part.is_empty() {
                date_holder.add_date_part(&mut curr_part);
            }
        } else if date_holder.len() >= 2 {
            date_holders.push(&mut date_holder);
        } else if !date_holder.is_empty() {
            date_holder.truncate();
            curr_part.truncate();
        }
    }
    if !date_holder.is_empty() {
        date_holder.add_date_part(&mut curr_part);
        date_holders.push(&mut date_holder);
    }
    date_holders.as_dates()
}

pub fn find_last_date(s: &str) -> Result<Date, DateError> {
    match find_dates(s).pop() {
        Some(date_result) => date_result,
        None => Err(DateError::NoDatesFound(s.to_string())),
    }
}

#[derive(Clone, Debug)]
struct Part(Vec<char>);

impl Part {
    fn new() -> Self {
        Self(vec![])
    }
    fn truncate(&mut self) {
        self.0.truncate(0);
    }
    fn push(&mut self, ch: char) {
        self.0.push(ch);
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn to_u16(&self) -> Result<u16, DateError> {
        Ok(self
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, ch)| {
                if i == 0 && *ch == '0' {
                    None
                } else {
                    Some(*ch)
                }
            })
            .collect::<String>()
            .parse::<u16>()?)
    }
}
impl Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ch in self.0.iter() {
            write!(f, "{ch}")?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
struct DateHolder {
    holding: Vec<Part>,
}
impl Display for DateHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for part in self.holding.iter() {
            str.push_str((part.to_string() + " ").as_str());
        }
        str.pop(); // remove last space
        write!(f, "{}", str)
    }
}
impl DateHolder {
    fn new() -> Self {
        Self { holding: vec![] }
    }
    fn add_date_part(&mut self, part: &mut Part) {
        self.holding.push(part.clone());
        part.truncate();
    }
    fn truncate(&mut self) {
        self.holding.truncate(0);
    }
    fn is_empty(&self) -> bool {
        self.holding.is_empty()
    }
    fn len(&self) -> usize {
        self.holding.len()
    }
    fn as_date(&self) -> Result<Date, DateError> {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        match self.holding.len() {
            2 => {
                let opt1 = self.holding[0].to_u16()?;
                let opt2 = self.holding[1].to_u16()?;
                if opt1 > 12 {
                    year = Some(opt1);
                    month = Some(opt2);
                } else if opt2 > 12 {
                    month = Some(opt1);
                    year = Some(opt2);
                } else {
                    return Err(DateError::UndecidedDate((Some(opt1), Some(opt2), None)));
                }
            }
            3 => {
                let opt1 = self.holding[0].to_u16()?;
                let opt2 = self.holding[1].to_u16()?;
                let opt3 = self.holding[2].to_u16()?;
                // if first date is greater than 12, it's year
                if opt1 > 12 {
                    year.replace(opt1);
                    month.replace(opt2);
                    day.replace(opt3);
                    // if last date is greater than 12, it's year
                } else if opt3 > 12 && opt1 > 12 {
                    day.replace(opt2);
                    month.replace(opt1);
                    year.replace(opt3);
                    // if middle date is greater than 12, it's day
                } else if opt2 > 12 {
                    month.replace(opt1);
                    day.replace(opt2);
                    year.replace(opt3);
                    // if all dates are equal it doesnt matter
                } else if opt1 == opt2 && opt2 == opt3 {
                    year.replace(opt1);
                    month.replace(opt2);
                    day.replace(opt3);
                    // otherwise undecided
                } else {
                    return Err(DateError::UndecidedDate((
                        Some(opt1),
                        Some(opt2),
                        Some(opt3),
                    )));
                }
            }
            _ => return Err(DateError::InvalidDateFormat(self.to_string())),
        }
        Ok(Date { year, month, day })
    }
}

#[derive(Debug)]
struct DateHolders(Vec<DateHolder>);
impl DateHolders {
    fn new() -> Self {
        Self(vec![])
    }
    fn push(&mut self, date_holder: &mut DateHolder) {
        self.0.push(date_holder.clone());
        date_holder.truncate();
    }

    fn as_dates(&self) -> Vec<Result<Date, DateError>> {
        let mut dates = vec![];
        for holder in self.0.iter() {
            dates.push(holder.as_date());
        }
        dates
    }
}

#[derive(Debug, PartialEq)]
pub enum DateError {
    NoDatesFound(String),
    UndecidedDate((Option<u16>, Option<u16>, Option<u16>)),
    InvalidDateFormat(String),
    ParseIntError(ParseIntError),
}

impl Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateError::NoDatesFound(msg) => write!(f, "No dates found from {}", msg),
            DateError::UndecidedDate(msg) => write!(
                f,
                "unable to determine date from values: {:?} {:?} {:?}",
                msg.0, msg.1, msg.2
            ),
            DateError::InvalidDateFormat(msg) => write!(f, "Invalid date format from {}", msg),
            DateError::ParseIntError(err) => write!(f, "{err}",),
        }
    }
}

impl From<ParseIntError> for DateError {
    fn from(err: ParseIntError) -> Self {
        DateError::InvalidDateFormat(err.to_string())
    }
}

impl std::error::Error for DateError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn part_to_u16_check_1() {
        let part = Part(vec!['2', '0', '2', '3']);
        let num = part.to_u16().unwrap();
        assert_eq!(num, 2023);
    }
    #[test]
    fn part_to_u16_check_2() {
        let part = Part(vec!['0', '5']);
        let num = part.to_u16().unwrap();
        assert_eq!(num, 5);
    }

    #[test]
    fn date_holders_check_1() {
        let dates = find_dates("2023-10-05");
        let expected = vec![Ok(Date {
            year: Some(2023),
            month: Some(10),
            day: Some(5),
        })];
        assert_eq!(dates, expected);
    }
    #[test]
    fn date_holders_check_2() {
        let dates = find_dates("2023-10-05 some other/random text 2021-11_21");
        let expected = vec![
            Ok(Date {
                year: Some(2023),
                month: Some(10),
                day: Some(5),
            }),
            Ok(Date {
                year: Some(2021),
                month: Some(11),
                day: Some(21),
            }),
        ];
        assert_eq!(dates, expected);
    }
    #[test]
    fn date_holders_check_3() {
        let dates = find_dates("100 some random text 2023-10-05 some other/random text 2021-11_21");
        let expected = vec![
            Ok(Date {
                year: Some(2023),
                month: Some(10),
                day: Some(5),
            }),
            Ok(Date {
                year: Some(2021),
                month: Some(11),
                day: Some(21),
            }),
        ];
        assert_eq!(dates, expected);
    }

    #[test]
    fn cast_date_holder_to_date_1() {
        let date = DateHolder {
            holding: vec![
                Part(vec!['2', '0', '2', '3']),
                Part(vec!['1', '0']),
                Part(vec!['0', '5']),
            ],
        }
        .as_date()
        .unwrap();
        assert_eq!(date, Date {
            year: Some(2023),
            month: Some(10),
            day: Some(5),
        });
    }
    #[test]
    fn cast_date_holder_to_date_2() {
        let date = DateHolder {
            holding: vec![
                Part(vec!['1', '2']),
                Part(vec!['1', '0']),
                Part(vec!['0', '5']),
            ],
        }
        .as_date();
        assert_eq!(
            date,
            Err(DateError::UndecidedDate((Some(12), Some(10), Some(5))))
        );
    }

    #[test]
    fn cmp_dates() {
        let date1 = Date {
            year: Some(2023),
            month: Some(10),
            day: Some(5),
        };
        let date2 = Date {
            year: Some(2022),
            month: Some(12),
            day: Some(31),
        };
        assert!(date1 > date2);
        let date3 = Date {
            year: Some(2023),
            month: None,
            day: None,
        };
        assert!(date3 < date1);
        let date4 = Date {
            year: Some(2023),
            month: Some(10),
            day: None,
        };
        assert!(date3 < date4);
    }
}
