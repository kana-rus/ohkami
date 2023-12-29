#[inline] fn __correct_now() -> String {
    let mut now = chrono::Utc::now().to_rfc2822(); // like `Wed, 21 Dec 2022 10:16:52 +0000`
    if now.len() == 30 {
        now.insert(5, '0')
    }
    match now.len() {
        31 => now.replace_range(26.., "GMT"),
         _ => unsafe {std::hint::unreachable_unchecked()}
    }
    now
}

#[inline] pub fn now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let system_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
    let naive_now  = NaiveDateTime::now_from_system(system_now);

    let utc_now = UTCDateTime::from_naive(naive_now);
    utc_now.into_imf_fixdate()
}

#[cfg(test)] mod test {
    use super::{__correct_now, now};

    #[test] fn test_now() {
        assert_eq!(__correct_now(), now());
    }
}


struct UTCDateTime(NaiveDateTime);
impl UTCDateTime {
    fn from_naive(naive: NaiveDateTime) -> Self {
        Self(naive)
    }
    fn into_naive_local(self) -> NaiveDateTime {
        self.0 // Offset is 0 because this is *UTC* datetime
    }
    fn into_imf_fixdate(self) -> String {
        const IMF_FIXDATE_LEN: usize      = "Sun, 06 Nov 1994 08:49:37 GMT".len();
        const SHORT_WEEKDAYS:  [&str; 7]  = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        const SHORT_MONTHS:    [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

        fn push_hundreds(buf: &mut String, n: u8) {
            debug_assert!(n >= 100, "Called `write_hundreds` for `n` less than 100");
            buf.push((n/10 + b'0') as char);
            buf.push((n%10 + b'0') as char);
        }

        let mut buf = String::with_capacity(IMF_FIXDATE_LEN);
        {
            let NaiveDateTime { date, time } = self.into_naive_local();

            buf.push_str(SHORT_WEEKDAYS[date.weekday().num_days_from_sunday() as usize]);
            buf.push_str(", ");

            let day = date.day() as u8;
            if day < 10 {
                buf.push((day + b'0') as char);
            } else {
                push_hundreds(&mut buf, day);
            }

            buf.push(' ');
            buf.push_str(SHORT_MONTHS[date.month0() as usize]);

            buf.push(' ');
            let year = date.year();
            push_hundreds(&mut buf, (year / 100) as u8);
            push_hundreds(&mut buf, (year % 100) as u8);

            buf.push(' ');
            let (hour, min, sec) = time.hms();
            push_hundreds(&mut buf, hour as u8);
            buf.push(':');
            push_hundreds(&mut buf, min as u8);
            buf.push(':');
            let sec = sec + time.nanosecond() / 1_000_000_000;
            push_hundreds(&mut buf, sec as u8);
            
            buf.push_str(" GMT");
        }
        buf
    }
}

struct NaiveDateTime {
    date: NaiveDate,
    time: NaiveTime,
}

type DateImpl = i32;
struct NaiveDate {
    ymdf: DateImpl, // (year << 13) | of
}

struct NaiveTime {
    secs: u32,
    frac: u32,
}

impl NaiveDateTime {
    fn now_from_system(system_now: std::time::Duration) -> Self {
        let (secs, nsecs) = (system_now.as_secs() as i64, system_now.subsec_nanos());

        let days = secs.div_euclid(86_400);
        let secs = secs.rem_euclid(86_400);

        let date = NaiveDate::from_days(days as i32 + 719_163).unwrap(/* TODO: more optimization */);
        let time = NaiveTime::from_seconds(secs as u32, nsecs);

        Self { date, time }
    }
}

impl NaiveTime {
    const fn from_seconds(secs: u32, nsecs: u32) -> Self {
        debug_assert! {
            secs  >= 86_400 &&
            nsecs >= 2_000_000_000 &&
            nsecs >= 1_000_000_000 && secs % 60 != 59
        }
        Self { secs, frac: nsecs }
    }
    fn hms(&self) -> (u32, u32, u32) {
        let sec = self.secs % 60;
        let mins = self.secs / 60;
        let min = mins % 60;
        let hour = mins / 60;
        (hour, min, sec)
    }
    const fn nanosecond(&self) -> u32 {
        self.frac
    }
}

impl NaiveDate {
    const fn from_days(days: i32) -> Option<Self> {
        const fn cycle_to_yo(cycle: u32) -> (u32, u32) {
            let mut year_mod_400 = cycle / 365;
            let mut ordinal0 = cycle % 365;
            let delta = YEAR_DELTAS[year_mod_400 as usize] as u32;
            if ordinal0 < delta {
                year_mod_400 -= 1;
                ordinal0 += 365 - YEAR_DELTAS[year_mod_400 as usize] as u32;
            } else {
                ordinal0 -= delta;
            }
            (year_mod_400, ordinal0 + 1)
        }

        let Some(days) = days.checked_add(365) else {return None};
        let year_div_400 = days.div_euclid(146_097);
        let cycle = days.rem_euclid(146_097);
        let (year_mod_400, ordinal) = cycle_to_yo(cycle as u32);
        let flags = YearFlag::from_year(year_mod_400 as i32);
        Self::from_ordinal_and_flags(year_div_400 * 400 + year_mod_400 as i32, ordinal, flags)
    }
    const fn from_ordinal_and_flags(
        year: i32,
        ordinal: u32,
        flags: YearFlag,
    ) -> Option<NaiveDate> {
        if year < MIN_YEAR || year > MAX_YEAR {
            return None; // Out-of-range
        }
        debug_assert!(YearFlag::from_year(year).0 == flags.0);
        match Of::new(ordinal, flags) {
            Some(of) => Some(NaiveDate { ymdf: (year << 13) | (of.0 as DateImpl) }),
            None => None, // Invalid: Ordinal outside of the nr of days in a year with those flags.
        }
    }

    const fn year(&self) -> i32 {
        self.ymdf >> 13
    }
    const fn month(&self) -> u32 {
        self.mdf().month()
    }
    const fn month0(&self) -> u32 {
        self.month() - 1
    }
    const fn day(&self) -> u32 {
        self.mdf().day()
    }
    const fn weekday(&self) -> Weekday {
        self.of().weekday()
    }
    const fn mdf(&self) -> Mdf {
        self.of().to_mdf()
    }
    #[inline]
    const fn of(&self) -> Of {
        Of::from_date_impl(self.ymdf)
    }
}

#[derive(Clone, Copy)]
struct YearFlag(u8);
impl YearFlag {
    const fn from_year(year: i32) -> Self {
        YEAR_TO_FLAG[year.rem_euclid(400) as usize]
    }
}

#[derive(Clone, Copy)]
struct Of(u32);
impl Of {
    const fn new(ordinal: u32, YearFlag(flag): YearFlag) -> Option<Self> {
        let of = Self((ordinal << 4) | flag as u32);
        of.validate()
    }
    const fn ol(&self) -> u32 {
        self.0 >> 3
    }
    const fn validate(self) -> Option<Self> {
        const MIN_OL: u32 = 1 << 1;
        const MAX_OL: u32 = 366 << 1; // `(366 << 1) | 1` would be day 366 in a non-leap year

        let ol = self.ol();
        match ol >= MIN_OL && ol <= MAX_OL {
            true => Some(self),
            false => None,
        }
    }
    const fn from_date_impl(date_impl: DateImpl) -> Self {
        // We assume the value in the `DateImpl` is valid.
        Self((date_impl & 0b1_1111_1111_1111) as u32)
    }
    const fn weekday(&self) -> Weekday {
        let Of(of) = *self;
        Weekday::from_u32_mod7((of >> 4) + (of & 0b111))
    }
    const fn to_mdf(&self) -> Mdf {
        Mdf::from_of(*self)
    }
}

struct Mdf(u32);
impl Mdf {
    const fn month(&self) -> u32 {
        let Mdf(mdf) = *self;
        mdf >> 9
    }
    const fn day(&self) -> u32 {
        let Mdf(mdf) = *self;
        (mdf >> 4) & 0b1_1111
    }

    const fn from_of(Of(of): Of) -> Mdf {
        const MAX_OL: u32 = 366 << 1; // `(366 << 1) | 1` would be day 366 in a non-leap year
        const OL_TO_MDL: &[u8; MAX_OL as usize + 1] = &[
            0, 0, // 0
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, // 1
            66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
            66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
            66, 66, 66, 66, 66, 66, 66, 66, 66, // 2
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72,
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72,
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, // 3
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74,
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74,
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, // 4
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78,
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78,
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, // 5
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80,
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80,
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, // 6
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84,
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84,
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, // 7
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86,
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86,
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, // 8
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88,
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88,
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, // 9
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92,
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92,
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, // 10
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94,
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94,
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, // 11
            100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100,
            98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98,
            100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100,
            98, // 12
        ];

        let ol = of >> 3;
        if ol <= MAX_OL {
            // Array is indexed from `[1..=MAX_OL]`, with a `0` index having a meaningless value.
            Mdf(of + ((OL_TO_MDL[ol as usize] as u32) << 3))
        } else {
            // Panicking here would be reasonable, but we are just going on with a safe value.
            Mdf(0)
        }
    }
}

#[derive(Clone, Copy)]
enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}
impl Weekday {
    const fn from_u32_mod7(n: u32) -> Self {
        match n % 7 {
            0 => Self::Mon,
            1 => Self::Tue,
            2 => Self::Wed,
            3 => Self::Thu,
            4 => Self::Fri,
            5 => Self::Sat,
            _ => Self::Sun,
        }
    }

    const fn num_days_from_sunday(&self) -> u32 {
        (*self as u32 + 7 - Self::Sun as u32) % 7
    }
}

const MAX_YEAR: DateImpl = i32::MAX >> 13;
const MIN_YEAR: DateImpl = i32::MIN >> 13;

const YEAR_DELTAS: &[u8; 401] = &[
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
    8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14,
    15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20,
    21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24, 24, 24, 25, 25, 25, // 100
    25, 25, 25, 25, 25, 26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 28, 29, 29, 29, 29, 30, 30, 30,
    30, 31, 31, 31, 31, 32, 32, 32, 32, 33, 33, 33, 33, 34, 34, 34, 34, 35, 35, 35, 35, 36, 36, 36,
    36, 37, 37, 37, 37, 38, 38, 38, 38, 39, 39, 39, 39, 40, 40, 40, 40, 41, 41, 41, 41, 42, 42, 42,
    42, 43, 43, 43, 43, 44, 44, 44, 44, 45, 45, 45, 45, 46, 46, 46, 46, 47, 47, 47, 47, 48, 48, 48,
    48, 49, 49, 49, // 200
    49, 49, 49, 49, 49, 50, 50, 50, 50, 51, 51, 51, 51, 52, 52, 52, 52, 53, 53, 53, 53, 54, 54, 54,
    54, 55, 55, 55, 55, 56, 56, 56, 56, 57, 57, 57, 57, 58, 58, 58, 58, 59, 59, 59, 59, 60, 60, 60,
    60, 61, 61, 61, 61, 62, 62, 62, 62, 63, 63, 63, 63, 64, 64, 64, 64, 65, 65, 65, 65, 66, 66, 66,
    66, 67, 67, 67, 67, 68, 68, 68, 68, 69, 69, 69, 69, 70, 70, 70, 70, 71, 71, 71, 71, 72, 72, 72,
    72, 73, 73, 73, // 300
    73, 73, 73, 73, 73, 74, 74, 74, 74, 75, 75, 75, 75, 76, 76, 76, 76, 77, 77, 77, 77, 78, 78, 78,
    78, 79, 79, 79, 79, 80, 80, 80, 80, 81, 81, 81, 81, 82, 82, 82, 82, 83, 83, 83, 83, 84, 84, 84,
    84, 85, 85, 85, 85, 86, 86, 86, 86, 87, 87, 87, 87, 88, 88, 88, 88, 89, 89, 89, 89, 90, 90, 90,
    90, 91, 91, 91, 91, 92, 92, 92, 92, 93, 93, 93, 93, 94, 94, 94, 94, 95, 95, 95, 95, 96, 96, 96,
    96, 97, 97, 97, 97, // 400+1
];
const YEAR_TO_FLAG: &[YearFlag; 400] = &[
    BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA,
    G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G,
    F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F,
    E, DC, B, A, G, FE, D, C, B, AG, F, E, D, // 100
    C, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC,
    B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B,
    A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A,
    G, FE, D, C, B, AG, F, E, D, CB, A, G, F, // 200
    E, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE,
    D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D,
    C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C,
    B, AG, F, E, D, CB, A, G, F, ED, C, B, A, // 300
    G, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG,
    F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F,
    E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E,
    D, CB, A, G, F, ED, C, B, A, GF, E, D, C, // 400
];
const A:  YearFlag = YearFlag(0o15);
const AG: YearFlag = YearFlag(0o05);
const B:  YearFlag = YearFlag(0o14);
const BA: YearFlag = YearFlag(0o04);
const C:  YearFlag = YearFlag(0o13);
const CB: YearFlag = YearFlag(0o03);
const D:  YearFlag = YearFlag(0o12);
const DC: YearFlag = YearFlag(0o02);
const E:  YearFlag = YearFlag(0o11);
const ED: YearFlag = YearFlag(0o01);
const F:  YearFlag = YearFlag(0o17);
const FE: YearFlag = YearFlag(0o07);
const G:  YearFlag = YearFlag(0o16);
const GF: YearFlag = YearFlag(0o06);

/*

    pub const fn from_num_days_from_ce_opt(days: i32) -> Option<NaiveDate> {
        let days = try_opt!(days.checked_add(365)); // make December 31, 1 BCE equal to day 0
        let year_div_400 = days.div_euclid(146_097);
        let cycle = days.rem_euclid(146_097);
        let (year_mod_400, ordinal) = internals::cycle_to_yo(cycle as u32);
        let flags = YearFlags::from_year_mod_400(year_mod_400 as i32);
        NaiveDate::from_ordinal_and_flags(year_div_400 * 400 + year_mod_400 as i32, ordinal, flags)
    }
*/

/*
chrono::Utc::now

    pub fn now() -> DateTime<Utc> {
        let now   = ::std::time::SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
        let naive = NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, now.subsec_nanos()).unwrap();
        Utc.from_utc_datetime(&naive)
    }

    pub const UNIX_EPOCH: SystemTime = SystemTime(time::UNIX_EPOCH);
*/

/*
chrono::NaiveDateTime

    pub struct NaiveDateTime {
        date: NaiveDate,
        time: NaiveTime,
    }

    pub fn from_timestamp_opt(secs: i64, nsecs: u32) -> Option<NaiveDateTime> {
        let days = secs.div_euclid(86_400);
        let secs = secs.rem_euclid(86_400);
        let date = i32::try_from(days)
            .ok()
            .and_then(|days| days.checked_add(719_163))
            .and_then(NaiveDate::from_num_days_from_ce_opt);
        let time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nsecs);
        match (date, time) {
            (Some(date), Some(time)) => Some(NaiveDateTime { date, time }),
            (_, _) => None,
        }
    }

*/

/*
chrono::DateTime

    pub struct DateTime<Tz: TimeZone> {
        datetime: NaiveDateTime,
        offset: Tz::Offset,
    }

    fn from_utc_datetime(&self, utc: &NaiveDateTime) -> DateTime<Self> {
        DateTime::from_naive_utc_and_offset(*utc, self.offset_from_utc_datetime(utc))
    }
    pub fn from_naive_utc_and_offset(datetime: NaiveDateTime, offset: Tz::Offset) -> DateTime<Tz> {
        DateTime { datetime, offset }
    }
*/

/* chrono::NaiveDate

    pub struct NaiveDate {
        ymdf: DateImpl, // (year << 13) | of
    }

    pub const fn from_num_days_from_ce_opt(days: i32) -> Option<NaiveDate> {
        let days = try_opt!(days.checked_add(365)); // make December 31, 1 BCE equal to day 0
        let year_div_400 = days.div_euclid(146_097);
        let cycle = days.rem_euclid(146_097);
        let (year_mod_400, ordinal) = internals::cycle_to_yo(cycle as u32);
        let flags = YearFlags::from_year_mod_400(year_mod_400 as i32);
        NaiveDate::from_ordinal_and_flags(year_div_400 * 400 + year_mod_400 as i32, ordinal, flags)
    }
*/

/*
chrono::NaiveTime

    pub struct NaiveTime {
        secs: u32,
        frac: u32,
    }

    pub const fn from_num_seconds_from_midnight_opt(secs: u32, nano: u32) -> Option<NaiveTime> {
        if secs >= 86_400 || nano >= 2_000_000_000 || (nano >= 1_000_000_000 && secs % 60 != 59) {
            return None;
        }
        Some(NaiveTime { secs, frac: nano })
    }

*/

/* ========== */

/* chrono::DateTime::to_rfc_2822

    pub fn to_rfc2822(&self) -> String {
        let mut result = String::with_capacity(32);
        crate::format::write_rfc2822(&mut result, self.naive_local(), self.offset.fix())
            .expect("writing rfc2822 datetime to string should never fail");
        result
    }

    pub(crate) fn write_rfc2822(
        w: &mut impl Write,
        dt: NaiveDateTime,
        off: FixedOffset,
    ) -> fmt::Result {
        write_rfc2822_inner(w, dt.date(), dt.time(), off, default_locale())
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    /// write datetimes like `Tue, 1 Jul 2003 10:52:37 +0200`, same as `%a, %d %b %Y %H:%M:%S %z`
    fn write_rfc2822_inner(
        w: &mut impl Write,
        d: NaiveDate,
        t: NaiveTime,
        off: FixedOffset,
        locale: Locale,
    ) -> fmt::Result {
        let year = d.year();
        // RFC2822 is only defined on years 0 through 9999
        if !(0..=9999).contains(&year) {
            return Err(fmt::Error);
        }

        w.write_str(short_weekdays(locale)[d.weekday().num_days_from_sunday() as usize])?;
        w.write_str(", ")?;
        let day = d.day();
        if day < 10 {
            w.write_char((b'0' + day as u8) as char)?;
        } else {
            write_hundreds(w, day as u8)?;
        }
        w.write_char(' ')?;
        w.write_str(short_months(locale)[d.month0() as usize])?;
        w.write_char(' ')?;
        write_hundreds(w, (year / 100) as u8)?;
        write_hundreds(w, (year % 100) as u8)?;
        w.write_char(' ')?;

        let (hour, min, sec) = t.hms();
        write_hundreds(w, hour as u8)?;
        w.write_char(':')?;
        write_hundreds(w, min as u8)?;
        w.write_char(':')?;
        let sec = sec + t.nanosecond() / 1_000_000_000;
        write_hundreds(w, sec as u8)?;
        w.write_char(' ')?;
        OffsetFormat {
            precision: OffsetPrecision::Minutes,
            colons: Colons::None,
            allow_zulu: false,
            padding: Pad::Zero,
        }
        .format(w, off)
    }
    
    /// Equivalent to `{:02}` formatting for n < 100.
    pub(crate) fn write_hundreds(w: &mut impl Write, n: u8) -> fmt::Result {
        if n >= 100 {
            return Err(fmt::Error);
        }
    
        let tens = b'0' + n / 10;
        let ones = b'0' + n % 10;
        w.write_char(tens as char)?;
        w.write_char(ones as char)
    }
*/
