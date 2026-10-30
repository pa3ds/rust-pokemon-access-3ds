//! What the port needs of Python's own behaviour, where Rust's differs and the difference would
//! change what pa3ds says or reads.
//!
//! The port is checked against the Python runtime line for line (`tools/tape`), so a percentage
//! rounded the other way at .5, a negative cell divided toward zero instead of down, or a float
//! logged as `1` instead of `1.0` is a regression. Every module uses these instead of the Rust
//! operator that looks the same: `docs/porting.md` lists which is which.

use std::fmt;
use std::panic::Location;
use std::rc::Rc;

/// `log: Callable[[str], None]`: where a reader writes what the log file shows.
pub type Log = Rc<dyn Fn(&str)>;

/// A Python exception class the port raises or catches by name.
///
/// pa3ds catches by class (`except (RomError, struct.error)`), so the port keeps the classes and
/// their bases: [`Error::is`] is `isinstance`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Exception,
    RuntimeError,
    ValueError,
    UnicodeDecodeError,
    UnicodeEncodeError,
    LookupError,
    IndexError,
    KeyError,
    ArithmeticError,
    ZeroDivisionError,
    OverflowError,
    TypeError,
    /// `struct.error`: bytes too short for what was unpacked from them.
    StructError,
    OsError,
    TimeoutError,
    ConnectionResetError,
    /// `queue.Empty`.
    QueueEmpty,
    /// `tomllib.TOMLDecodeError`.
    TomlDecodeError,
    /// `pa3ds.core.rom.RomError`.
    RomError,
    /// `pa3ds.core.rpc.RpcError` and the classes under it.
    RpcError,
    RpcTimeoutError,
    RpcRefusedError,
    RpcLostError,
    /// `pa3ds.debug.gdb.GdbError` and `GdbTimeoutError`.
    GdbError,
    GdbTimeoutError,
}

impl Kind {
    /// The class this one derives from, as Python declares it.
    pub fn base(self) -> Option<Kind> {
        use Kind::*;
        match self {
            Exception => None,
            RuntimeError | ValueError | LookupError | ArithmeticError | TypeError | StructError
            | OsError | QueueEmpty | RomError => Some(Exception),
            ZeroDivisionError | OverflowError => Some(ArithmeticError),
            UnicodeDecodeError | UnicodeEncodeError | TomlDecodeError => Some(ValueError),
            IndexError | KeyError => Some(LookupError),
            TimeoutError | ConnectionResetError => Some(OsError),
            RpcError | GdbError => Some(RuntimeError),
            RpcTimeoutError => Some(RpcError),
            RpcRefusedError | RpcLostError => Some(RpcTimeoutError),
            GdbTimeoutError => Some(GdbError),
        }
    }

    /// The class's name as a traceback gives it.
    pub fn name(self) -> &'static str {
        use Kind::*;
        match self {
            Exception => "Exception",
            RuntimeError => "RuntimeError",
            ValueError => "ValueError",
            UnicodeDecodeError => "UnicodeDecodeError",
            UnicodeEncodeError => "UnicodeEncodeError",
            LookupError => "LookupError",
            IndexError => "IndexError",
            KeyError => "KeyError",
            ArithmeticError => "ArithmeticError",
            ZeroDivisionError => "ZeroDivisionError",
            OverflowError => "OverflowError",
            TypeError => "TypeError",
            StructError => "error",
            OsError => "OSError",
            TimeoutError => "TimeoutError",
            ConnectionResetError => "ConnectionResetError",
            QueueEmpty => "Empty",
            TomlDecodeError => "TOMLDecodeError",
            RomError => "RomError",
            RpcError => "RpcError",
            RpcTimeoutError => "RpcTimeoutError",
            RpcRefusedError => "RpcRefusedError",
            RpcLostError => "RpcLostError",
            GdbError => "GdbError",
            GdbTimeoutError => "GdbTimeoutError",
        }
    }
}

/// A raised exception: its class, `str(error)`, and the line that raised it, which the runtime
/// keys a fault by as Python keyed it by the traceback's last frame (`app.Faults`).
#[derive(Clone)]
pub struct Error {
    pub kind: Kind,
    pub message: String,
    pub location: &'static Location<'static>,
    /// For a short read, the line that called `core::binary`'s helper, which raised it: logged
    /// beside where it was raised, as Python's traceback names the reader, and never keyed by
    /// (#18).
    pub caller: Option<&'static Location<'static>>,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    #[track_caller]
    pub fn new(kind: Kind, message: impl Into<String>) -> Self {
        Error {
            kind,
            message: message.into(),
            location: Location::caller(),
            caller: None,
        }
    }

    /// This error, called for from `caller` unless an inner call already says where from.
    pub fn called_from(mut self, caller: &'static Location<'static>) -> Self {
        self.caller.get_or_insert(caller);
        self
    }

    /// `isinstance(error, kind)`.
    pub fn is(&self, kind: Kind) -> bool {
        let mut at = Some(self.kind);
        while let Some(k) = at {
            if k == kind {
                return true;
            }
            at = k.base();
        }
        false
    }

    #[track_caller]
    pub fn value(message: impl Into<String>) -> Self {
        Error::new(Kind::ValueError, message)
    }

    #[track_caller]
    pub fn index(message: impl Into<String>) -> Self {
        Error::new(Kind::IndexError, message)
    }

    #[track_caller]
    pub fn key(message: impl Into<String>) -> Self {
        Error::new(Kind::KeyError, message)
    }

    #[track_caller]
    pub fn unpack(message: impl Into<String>) -> Self {
        Error::new(Kind::StructError, message)
    }

    /// The fault this is, as one line: what was raised and where (`app._fault`).
    pub fn fault(&self) -> String {
        let file = self
            .location
            .file()
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("");
        format!("{} at {}:{}", self.kind.name(), file, self.location.line())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Debug for Error {
    /// `repr(error)`: `RpcError('page 0x08000000 unread')`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.kind.name(), repr(&self.message))
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        let kind = match error.kind() {
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => Kind::TimeoutError,
            std::io::ErrorKind::ConnectionReset => Kind::ConnectionResetError,
            _ => Kind::OsError,
        };
        Error::new(kind, error.to_string())
    }
}

/// `text.encode("ascii")`, raising the `UnicodeEncodeError` Python words for the first run of
/// characters over 0x7f: `character '\xe9' in position 3`, or `characters in position 3-5`. The
/// error is located at the caller, as the Python raised it there.
#[track_caller]
pub fn encode_ascii(text: &str) -> Result<Vec<u8>> {
    if text.is_ascii() {
        return Ok(text.as_bytes().to_vec());
    }
    let chars: Vec<char> = text.chars().collect();
    let start = chars.iter().position(|c| !c.is_ascii()).unwrap_or(0);
    let end = start + chars[start..].iter().take_while(|c| !c.is_ascii()).count();
    let what = if end - start == 1 {
        let escaped = match chars[start] as u32 {
            n if n < 0x100 => format!("\\x{n:02x}"),
            n if n < 0x10000 => format!("\\u{n:04x}"),
            n => format!("\\U{n:08x}"),
        };
        format!("character '{escaped}' in position {start}")
    } else {
        format!("characters in position {start}-{}", end - 1)
    };
    Err(Error::new(
        Kind::UnicodeEncodeError,
        format!("'ascii' codec can't encode {what}: ordinal not in range(128)"),
    ))
}

/// `time`: the clock pa3ds reads, which a tape replaces (`tools/tape`).
///
/// Every `time.monotonic()` and `time.sleep()` in pa3ds is [`monotonic`] and [`sleep`] here, so a
/// replay can answer each from the tape in the order the Python runtime asked. The RPC client's
/// own timing (`perf_counter`) is below the tape and always real.
pub mod time {
    use std::cell::RefCell;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    /// Where the time comes from while a tape is replayed.
    pub trait Clock {
        fn monotonic(&mut self) -> f64;
        /// `ends_tick`: the sleep is [`sleep_ending_tick`]'s.
        fn sleep(&mut self, seconds: f64, ends_tick: bool);
    }

    thread_local! {
        static CLOCK: RefCell<Option<Box<dyn Clock>>> = const { RefCell::new(None) };
    }

    fn epoch() -> Instant {
        static EPOCH: OnceLock<Instant> = OnceLock::new();
        *EPOCH.get_or_init(Instant::now)
    }

    /// Answer this thread's [`monotonic`] and [`sleep`] from `clock`, or from the real clock
    /// again with None. Returns the clock it replaces.
    pub fn set_clock(clock: Option<Box<dyn Clock>>) -> Option<Box<dyn Clock>> {
        CLOCK.with(|cell| std::mem::replace(&mut *cell.borrow_mut(), clock))
    }

    /// `time.monotonic()`: seconds since some fixed moment.
    pub fn monotonic() -> f64 {
        let taken = CLOCK.with(|cell| cell.borrow_mut().as_mut().map(|clock| clock.monotonic()));
        taken.unwrap_or_else(|| epoch().elapsed().as_secs_f64())
    }

    /// `time.sleep(seconds)`.
    pub fn sleep(seconds: f64) {
        sleep_as(seconds, false)
    }

    /// `time.sleep(seconds)` where it ends a pass of the poll loop, `app._session`'s: the same
    /// sleep, which a tape's replay tells from the rest to end its tick, as the Python's
    /// `spikes/tape_replay.py` tells it by the function that called it (`TICK_END`).
    pub fn sleep_ending_tick(seconds: f64) {
        sleep_as(seconds, true)
    }

    fn sleep_as(seconds: f64, ends_tick: bool) {
        let replayed = CLOCK.with(|cell| match cell.borrow_mut().as_mut() {
            Some(clock) => {
                clock.sleep(seconds, ends_tick);
                true
            }
            None => false,
        });
        if !replayed && seconds > 0.0 {
            std::thread::sleep(Duration::from_secs_f64(seconds));
        }
    }

    /// `time.perf_counter()`: the real clock, never a tape's.
    pub fn perf_counter() -> f64 {
        epoch().elapsed().as_secs_f64()
    }
}

/// `a // b` for ints: rounded down, not toward zero.
pub fn floordiv(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

/// `a % b` for ints: the sign of `b`, not of `a`.
pub fn modulo(a: i64, b: i64) -> i64 {
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        r + b
    } else {
        r
    }
}

/// CPython's `divmod` for floats (`_float_div_mod` in `Objects/floatobject.c`), which `//` and `%`
/// are made of: the quotient is not `(a / b).floor()` (`1 // 0.1` is 9.0, not 10.0), and a zero
/// remainder takes the divisor's sign. `b` must not be 0, where Python raises.
fn float_div_mod(a: f64, b: f64) -> (f64, f64) {
    let mut modulo = a % b;
    let mut div = (a - modulo) / b;
    if modulo != 0.0 {
        if (b < 0.0) != (modulo < 0.0) {
            modulo += b;
            div -= 1.0;
        }
    } else {
        modulo = 0.0_f64.copysign(b);
    }
    let floordiv = if div != 0.0 {
        let floor = div.floor();
        if div - floor > 0.5 {
            floor + 1.0
        } else {
            floor
        }
    } else {
        0.0_f64.copysign(a / b)
    };
    (floordiv, modulo)
}

/// `a // b` for floats.
pub fn floordiv_f(a: f64, b: f64) -> f64 {
    float_div_mod(a, b).0
}

/// `a % b` for floats: the sign of `b`.
pub fn modulo_f(a: f64, b: f64) -> f64 {
    float_div_mod(a, b).1
}

/// `math.hypot(x, y)` as CPython 3.14 computes it, which Rust's `f64::hypot` does not: that is
/// the C library's, and differs from Python's in the last bit on about one pair in eight, which
/// moves a distance compared against a reach across it (checked against CPython on a million
/// pairs by the walls' port).
pub fn hypot(x: f64, y: f64) -> f64 {
    let (x, y) = (x.abs(), y.abs());
    // Python's max(), which a NaN does not win.
    let max = [x, y]
        .into_iter()
        .fold(0.0, |best: f64, v| if v > best { v } else { best });
    vector_norm(&mut [x, y], max, x.is_nan() || y.is_nan())
}

/// CPython's `vector_norm` (Modules/mathmodule.c): the norm of the vector, accurate to a
/// fraction of the last bit, from the vector, its largest member and whether one is a NaN.
fn vector_norm(vec: &mut [f64], max: f64, found_nan: bool) -> f64 {
    /// `dl_mul`: the product of two floats as a high and a low part, exactly.
    fn dl_mul(x: f64, y: f64) -> (f64, f64) {
        let z = x * y;
        (z, x.mul_add(y, -z))
    }
    /// `dl_fast_sum`: the sum of two floats, the larger first, as a high and a low part.
    fn dl_fast_sum(a: f64, b: f64) -> (f64, f64) {
        let x = a + b;
        (x, (a - x) + b)
    }
    if max.is_infinite() {
        return max;
    }
    if found_nan {
        return f64::NAN;
    }
    if max == 0.0 || vec.len() <= 1 {
        return max;
    }
    let max_e = frexp_exponent(max);
    if max_e < -1023 {
        // When max_e < -1023, ldexp(1.0, -max_e) would overflow.
        for x in vec.iter_mut() {
            *x /= f64::MIN_POSITIVE;
        }
        return f64::MIN_POSITIVE * vector_norm(vec, max / f64::MIN_POSITIVE, found_nan);
    }
    let scale = ldexp_one(-max_e);
    let (mut csum, mut frac1, mut frac2) = (1.0, 0.0, 0.0);
    for &x in vec.iter() {
        let x = x * scale;
        let pr = dl_mul(x, x);
        let sm = dl_fast_sum(csum, pr.0);
        csum = sm.0;
        frac1 += pr.1;
        frac2 += sm.1;
    }
    let mut h = (csum - 1.0 + (frac1 + frac2)).sqrt();
    let pr = dl_mul(-h, h);
    let sm = dl_fast_sum(csum, pr.0);
    csum = sm.0;
    frac1 += pr.1;
    frac2 += sm.1;
    let x = csum - 1.0 + (frac1 + frac2);
    h += x / (2.0 * h);
    h / scale
}

/// The exponent `frexp` gives a positive finite float: 0.5 <= x / 2**e < 1.
fn frexp_exponent(x: f64) -> i32 {
    let exponent = ((x.to_bits() >> 52) & 0x7FF) as i32;
    if exponent == 0 {
        // A subnormal: scaled up into the normal range first.
        return frexp_exponent(x * 2f64.powi(54)) - 54;
    }
    exponent - 1022
}

/// `ldexp(1.0, exponent)`, for the exponents `vector_norm` asks for, where `powi` might round.
fn ldexp_one(exponent: i32) -> f64 {
    if exponent >= -1022 {
        f64::from_bits(((exponent + 1023) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (exponent + 1074))
    }
}

/// A float made an int: Python refuses a NaN and an infinity where Rust would quietly make them
/// 0 and the largest `i64`, and a float read out of freed memory is either often enough.
#[track_caller]
fn whole(x: f64) -> Result<i64> {
    if x.is_nan() {
        return Err(Error::value("cannot convert float NaN to integer"));
    }
    if x.is_infinite() {
        return Err(Error::new(
            Kind::OverflowError,
            "cannot convert float infinity to integer",
        ));
    }
    Ok(x as i64)
}

/// `round(x)`: to the nearest int, halves to the even one.
#[track_caller]
pub fn round(x: f64) -> Result<i64> {
    whole(x.round_ties_even())
}

/// `int(x)` of a float: toward zero.
#[track_caller]
pub fn int(x: f64) -> Result<i64> {
    whole(x.trunc())
}

/// `math.floor(x)`.
#[track_caller]
pub fn floor(x: f64) -> Result<i64> {
    whole(x.floor())
}

/// `math.ceil(x)`.
#[track_caller]
pub fn ceil(x: f64) -> Result<i64> {
    whole(x.ceil())
}

/// `round(x, digits)`: rounded as the decimal it prints as, which is how Python rounds.
pub fn round_to(x: f64, digits: usize) -> f64 {
    format!("{x:.digits$}").parse().unwrap_or(x)
}

/// `str(x)` and `repr(x)` for a float: `1.0`, `0.1`, `1e-05`, `1e+16`.
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0.0".into()
        } else {
            "0.0".into()
        };
    }
    // The fewest digits that read back as `x`, each count rounded halves-to-even as Python's
    // repr rounds them. Rust's own shortest form breaks a tie between two 17-digit decimals the
    // other way, and a float widened from an f32 out of the game's memory often sits on one.
    let scientific = (1..=17)
        .map(|digits| format!("{:.*e}", digits - 1, x))
        .find(|text| text.parse::<f64>() == Ok(x))
        .unwrap_or_else(|| format!("{x:.16e}"));
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific notation");
    let exponent: i32 = exponent.parse().expect("an exponent");
    let negative = mantissa.starts_with('-');
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let mut out = String::from(if negative { "-" } else { "" });
    if (-4..16).contains(&exponent) {
        if exponent >= 0 {
            let whole = exponent as usize + 1;
            if digits.len() <= whole {
                out.push_str(digits);
                out.push_str(&"0".repeat(whole - digits.len()));
                out.push_str(".0");
            } else {
                out.push_str(&digits[..whole]);
                out.push('.');
                out.push_str(&digits[whole..]);
            }
        } else {
            out.push_str("0.");
            out.push_str(&"0".repeat((-exponent - 1) as usize));
            out.push_str(digits);
        }
    } else {
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        let sign = if exponent < 0 { '-' } else { '+' };
        out.push_str(&format!("e{sign}{:02}", exponent.abs()));
    }
    out
}

/// `str.isspace()` for one character: wider than [`char::is_whitespace`], which leaves out
/// U+001C to U+001F, the separators Python splits on.
pub fn is_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0b' | '\x0c' | '\r' | '\x1c'..='\x1f' | ' ' | '\u{85}'
        | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}'
        | '\u{202f}' | '\u{205f}' | '\u{3000}')
}

/// `text.split()`: the words between runs of whitespace.
pub fn split(text: &str) -> Vec<&str> {
    text.split(is_space)
        .filter(|word| !word.is_empty())
        .collect()
}

/// `text.strip()`.
pub fn strip(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// `text.lstrip()`.
pub fn lstrip(text: &str) -> &str {
    text.trim_start_matches(is_space)
}

/// `text.rstrip()`.
pub fn rstrip(text: &str) -> &str {
    text.trim_end_matches(is_space)
}

/// `text.splitlines()`: Python ends a line at more than `\n`.
pub fn splitlines(text: &str) -> Vec<&str> {
    split_lines(text, false)
}

/// `text.splitlines(keepends=True)`.
pub fn splitlines_keepends(text: &str) -> Vec<&str> {
    split_lines(text, true)
}

/// Both forms of `splitlines`: a line ends at `\n`, `\r`, `\r\n` (one break), `\v`, `\f`, the
/// separators U+001C to U+001E, U+0085, U+2028 and U+2029, and a last line without an ending is
/// kept unless it is empty.
fn split_lines(text: &str, keepends: bool) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if matches!(
            c,
            '\n' | '\r' | '\x0b' | '\x0c' | '\x1c'..='\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}'
        ) {
            let mut end = i + c.len_utf8();
            if c == '\r' && chars.peek().map(|&(_, next)| next) == Some('\n') {
                chars.next();
                end += 1;
            }
            lines.push(&text[start..if keepends { end } else { i }]);
            start = end;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// `str.isdecimal()` for one character: the general category Nd, which is also what `\d` is in a
/// `re` pattern on a `str` and what `int()` reads as a digit.
pub fn is_decimal(c: char) -> bool {
    unicode_general_category::get_general_category(c)
        == unicode_general_category::GeneralCategory::DecimalNumber
}

/// `str.isdigit()` for one character: the decimal digits, and the 128 code points that Unicode
/// gives a digit value without making them decimal (superscripts, subscripts, circled and
/// parenthesized numbers), which `int()` cannot read.
pub fn is_digit(c: char) -> bool {
    // The code points with `isdigit` true and a general category other than Nd, as CPython 3.14
    // (Unicode 16) has them.
    const OTHER_DIGITS: [(u32, u32); 20] = [
        (178, 179),
        (185, 185),
        (4969, 4977),
        (6618, 6618),
        (8304, 8304),
        (8308, 8313),
        (8320, 8329),
        (9312, 9320),
        (9332, 9340),
        (9352, 9360),
        (9450, 9450),
        (9461, 9469),
        (9471, 9471),
        (10102, 10110),
        (10112, 10120),
        (10122, 10130),
        (68160, 68163),
        (69216, 69224),
        (69714, 69722),
        (127232, 127242),
    ];
    is_decimal(c)
        || OTHER_DIGITS
            .iter()
            .any(|(first, last)| (*first..=*last).contains(&(c as u32)))
}

/// `text.isdigit()`: some characters, all of them digits.
pub fn isdigit(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_digit)
}

/// `str.isalpha()` for one character: the letters, which is every general category starting
/// with L.
pub fn is_alpha(c: char) -> bool {
    use unicode_general_category::GeneralCategory::*;
    matches!(
        unicode_general_category::get_general_category(c),
        UppercaseLetter | LowercaseLetter | TitlecaseLetter | ModifierLetter | OtherLetter
    )
}

/// `str.isalnum()` for one character: `isalpha`, `isdecimal`, `isdigit` or `isnumeric`. Every
/// character that is numeric and no letter is in a number category (Nd, Nl, No); the numeric
/// ideographs are letters already.
pub fn is_alnum(c: char) -> bool {
    use unicode_general_category::GeneralCategory::*;
    is_alpha(c)
        || matches!(
            unicode_general_category::get_general_category(c),
            DecimalNumber | LetterNumber | OtherNumber
        )
}

/// `str.isprintable()` for one character, which decides how `repr` writes it.
fn is_printable(c: char) -> bool {
    use unicode_general_category::GeneralCategory::*;
    if c == ' ' {
        return true;
    }
    !matches!(
        unicode_general_category::get_general_category(c),
        Control
            | Format
            | Surrogate
            | PrivateUse
            | Unassigned
            | LineSeparator
            | ParagraphSeparator
            | SpaceSeparator
    )
}

/// `repr(text)`: quoted the way Python quotes it, which pa3ds's log lines show (`{name!r}`).
pub fn repr(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push(quote);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if is_printable(c) => out.push(c),
            c if (c as u32) < 0x100 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if (c as u32) < 0x10000 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push_str(&format!("\\U{:08x}", c as u32)),
        }
    }
    out.push(quote);
    out
}

/// `data[start:end]`, clamped as Python clamps a slice instead of panicking. A negative index
/// counts from the end.
pub fn slice<T>(data: &[T], start: i64, end: i64) -> &[T] {
    let len = data.len() as i64;
    let at = |i: i64| -> usize { (if i < 0 { (len + i).max(0) } else { i.min(len) }) as usize };
    let (a, b) = (at(start), at(end));
    if a >= b { &data[0..0] } else { &data[a..b] }
}

/// `data[start:]`, clamped.
pub fn slice_from<T>(data: &[T], start: i64) -> &[T] {
    slice(data, start, data.len() as i64)
}

/// `bytes.decode("utf-16-le", "replace")`: a lone surrogate, or an odd byte at the end, is
/// U+FFFD.
pub fn decode_utf16le(data: &[u8]) -> String {
    let units: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_le_bytes(*p))
        .collect();
    let mut out = String::from_utf16_lossy(&units);
    if data.len() % 2 == 1 {
        out.push('\u{fffd}');
    }
    out
}

/// `bytes.decode("utf-16-le")`, strict: the UnicodeDecodeError Python raises, with its message
/// (`'utf-16-le' codec can't decode byte 0x41 in position 6: truncated data`), which reaches a
/// log inside a RomError.
#[track_caller]
pub fn decode_utf16le_strict(data: &[u8]) -> Result<String> {
    #[track_caller]
    fn fail(data: &[u8], start: usize, end: usize, reason: &str) -> Result<String> {
        let codec = "'utf-16-le' codec can't decode";
        let message = if end == start + 1 {
            format!(
                "{codec} byte {:#04x} in position {start}: {reason}",
                data[start]
            )
        } else {
            format!("{codec} bytes in position {start}-{}: {reason}", end - 1)
        };
        Err(Error::new(Kind::UnicodeDecodeError, message))
    }
    let mut units: Vec<u16> = Vec::new();
    let mut at = 0;
    while data.len() - at >= 2 {
        let unit = u16::from_le_bytes([data[at], data[at + 1]]);
        if (0xD800..0xDC00).contains(&unit) {
            if data.len() - (at + 2) < 2 {
                return fail(data, at, data.len(), "unexpected end of data");
            }
            let next = u16::from_le_bytes([data[at + 2], data[at + 3]]);
            if !(0xDC00..0xE000).contains(&next) {
                return fail(data, at, at + 2, "illegal UTF-16 surrogate");
            }
            units.push(unit);
            units.push(next);
            at += 4;
        } else if (0xDC00..0xE000).contains(&unit) {
            return fail(data, at, at + 2, "illegal encoding");
        } else {
            units.push(unit);
            at += 2;
        }
    }
    if at < data.len() {
        return fail(data, at, data.len(), "truncated data");
    }
    Ok(String::from_utf16(&units).expect("surrogates were paired"))
}

/// `bytes.decode("ascii", "replace")`: every byte over 0x7f is U+FFFD.
pub fn decode_ascii(data: &[u8]) -> String {
    data.iter()
        .map(|&b| if b < 0x80 { b as char } else { '\u{fffd}' })
        .collect()
}

/// `bytes.decode("latin-1")`.
pub fn decode_latin1(data: &[u8]) -> String {
    data.iter().map(|&b| b as char).collect()
}

/// The value of a Unicode decimal digit, which is what `int()` and `float()` read any script's
/// digits as.
fn decimal_value(c: char) -> Option<u32> {
    if !is_decimal(c) {
        return None;
    }
    // Decimal digits come in runs of ten from a zero, so a digit's value is how far into its run
    // of neighbours it is.
    let mut before = 0u32;
    while let Some(previous) = char::from_u32((c as u32).wrapping_sub(before + 1)) {
        if !is_decimal(previous) {
            break;
        }
        before += 1;
    }
    Some(before % 10)
}

/// `_PyUnicode_TransformDecimalAndSpaceToASCII`, the first step of `int(str)` and `float(str)`:
/// every Unicode whitespace is a space, every decimal digit its ASCII digit, and every other
/// character beyond ASCII a `?` that no number contains.
fn to_ascii_number(text: &str) -> Vec<char> {
    text.chars()
        .map(|c| {
            // The separators U+001C to U+001F are `isspace` to `strip`, and are no whitespace to
            // the number parsers.
            if is_space(c) && !('\x1c'..='\x1f').contains(&c) {
                ' '
            } else if let Some(value) = decimal_value(c) {
                char::from_digit(value, 10).expect("a decimal digit")
            } else if c.is_ascii() {
                c
            } else {
                '?'
            }
        })
        .collect()
}

/// What `%.200R` makes of a string in an error message: its `repr`, cut at 200 characters.
fn repr_200(text: &str) -> String {
    repr(text).chars().take(200).collect()
}

/// `int(text, base)` for base 10 and 16, as the sign and the digits' values: what `int()`
/// accepts (outer whitespace of any script, a sign, digits of any script, single underscores
/// between digits, and for base 16 a `0x` and one underscore after it) and the ValueError it
/// raises for the rest.
#[track_caller]
fn int_digits(text: &str, base: u32) -> Result<(bool, Vec<u32>)> {
    let invalid = || {
        Error::value(format!(
            "invalid literal for int() with base {base}: {}",
            repr_200(text)
        ))
    };
    let all = to_ascii_number(text);
    // CPython reads the digits as a C string, so a NUL ends them: the text before it is read in
    // full, the digit limit applies to it, and only then is the NUL found to be a bad character.
    let nul = all.iter().position(|&c| c == '\0');
    let chars = &all[..nul.unwrap_or(all.len())];
    let blank = |c: char| matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r');
    let mut at = 0;
    while at < chars.len() && blank(chars[at]) {
        at += 1;
    }
    let negative = chars.get(at) == Some(&'-');
    if matches!(chars.get(at), Some('+' | '-')) {
        at += 1;
    }
    if base == 16 && chars.get(at) == Some(&'0') && matches!(chars.get(at + 1), Some('x' | 'X')) {
        at += 2;
        if chars.get(at) == Some(&'_') {
            at += 1;
        }
    }
    if chars.get(at) == Some(&'_') {
        return Err(invalid());
    }
    let mut digits = Vec::new();
    let mut previous = None;
    while let Some(&c) = chars.get(at) {
        if c == '_' {
            if previous == Some('_') {
                return Err(invalid());
            }
        } else if let Some(digit) = c.to_digit(base) {
            digits.push(digit);
        } else {
            break;
        }
        previous = Some(c);
        at += 1;
    }
    if previous == Some('_') || digits.is_empty() {
        return Err(invalid());
    }
    while at < chars.len() && blank(chars[at]) {
        at += 1;
    }
    if at < chars.len() {
        return Err(invalid());
    }
    // Only a base that is not a power of two has the limit.
    if base == 10 && digits.len() > 4300 {
        return Err(Error::value(format!(
            "Exceeds the limit (4300 digits) for integer string conversion: value has {} digits; \
             use sys.set_int_max_str_digits() to increase the limit",
            digits.len()
        )));
    }
    if nul.is_some() {
        return Err(invalid());
    }
    Ok((negative, digits))
}

/// `int(text)`. Python's integers have no end and these have: a number past `i64` stops at its
/// largest (or smallest).
#[track_caller]
pub fn int_base10(text: &str) -> Result<i64> {
    let (negative, digits) = int_digits(text, 10)?;
    let magnitude = digits.iter().fold(0u128, |value, &digit| {
        (value * 10 + u128::from(digit)).min(1 << 64)
    });
    Ok(if negative {
        i64::try_from(-(magnitude as i128)).unwrap_or(i64::MIN)
    } else {
        i64::try_from(magnitude).unwrap_or(i64::MAX)
    })
}

/// `int(text, 16)`. A number past 64 bits keeps its low 64 bits, read as an `i64`, which is how
/// a register or a thread id the stub writes is held.
#[track_caller]
pub fn int_base16(text: &str) -> Result<i64> {
    let (negative, digits) = int_digits(text, 16)?;
    let magnitude = digits
        .iter()
        .fold(0u64, |value, &digit| value << 4 | u64::from(digit)) as i64;
    Ok(if negative {
        magnitude.wrapping_neg()
    } else {
        magnitude
    })
}

/// `bytes.fromhex(text)`: pairs of hexadecimal digits, with ASCII whitespace allowed between
/// pairs, and the ValueError Python words for the rest (positions are in characters).
#[track_caller]
pub fn fromhex(text: &str) -> Result<Vec<u8>> {
    let chars: Vec<char> = text.chars().collect();
    // A string with a character beyond ASCII is refused at the first of them, whatever comes
    // before it.
    if let Some(at) = chars.iter().position(|c| !c.is_ascii()) {
        return Err(Error::value(format!(
            "non-hexadecimal number found in fromhex() arg at position {at}"
        )));
    }
    let digit = |at: usize| chars.get(at).and_then(|c| c.to_digit(16));
    let space = |at: usize| {
        matches!(
            chars.get(at),
            Some(' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r')
        )
    };
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        if space(at) {
            while space(at) {
                at += 1;
            }
            if at >= chars.len() {
                break;
            }
        }
        let Some(top) = digit(at) else {
            return Err(Error::value(format!(
                "non-hexadecimal number found in fromhex() arg at position {at}"
            )));
        };
        at += 1;
        let Some(bottom) = digit(at) else {
            return Err(if at >= chars.len() {
                Error::value("fromhex() arg must contain an even number of hexadecimal digits")
            } else {
                Error::value(format!(
                    "non-hexadecimal number found in fromhex() arg at position {at}"
                ))
            });
        };
        at += 1;
        out.push((top * 16 + bottom) as u8);
    }
    Ok(out)
}

/// `float(text)`: a ValueError for what is not a number, as Python words it.
#[track_caller]
pub fn float_of(text: &str) -> Result<f64> {
    let bad = || Error::value(format!("could not convert string to float: {}", repr(text)));
    let chars = to_ascii_number(text);
    // An underscore is allowed between two digits and nowhere else.
    let mut previous = ' ';
    let mut plain: Vec<char> = Vec::with_capacity(chars.len());
    for &c in &chars {
        if c == '_' {
            if !previous.is_ascii_digit() {
                return Err(bad());
            }
        } else {
            if previous == '_' && !c.is_ascii_digit() {
                return Err(bad());
            }
            plain.push(c);
        }
        previous = c;
    }
    if previous == '_' {
        return Err(bad());
    }
    let blank = |c: &char| matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r');
    let start = plain.iter().position(|c| !blank(c)).unwrap_or(plain.len());
    let end = plain
        .iter()
        .rposition(|c| !blank(c))
        .map_or(start, |at| at + 1);
    let number: String = plain[start..end].iter().collect();
    // What `float()` reads: a sign, then inf, infinity, nan, or digits with an optional point
    // and exponent. Rust reads these the same way, and reads more (hexadecimal, for one), so
    // the shape is checked here first.
    let unsigned = number.strip_prefix(['+', '-']).unwrap_or(&number);
    let lower = unsigned.to_ascii_lowercase();
    let special = matches!(lower.as_str(), "inf" | "infinity" | "nan");
    let bytes = unsigned.as_bytes();
    let count = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let whole = count(0);
    let mut at = whole;
    let mut fraction = 0;
    if bytes.get(at) == Some(&b'.') {
        fraction = count(at + 1);
        at += 1 + fraction;
    }
    let mut shaped = whole + fraction > 0;
    if shaped && matches!(bytes.get(at), Some(b'e' | b'E')) {
        let mut exponent = at + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        let digits = count(exponent);
        shaped = digits > 0;
        at = exponent + digits;
    }
    if !special && !(shaped && at == bytes.len()) {
        return Err(bad());
    }
    number.parse::<f64>().map_err(|_| bad())
}

/// `f"{x:.{digits}f}"`.
pub fn fixed(x: f64, digits: usize) -> String {
    if x.is_nan() {
        "nan".to_string()
    } else if x.is_infinite() {
        if x > 0.0 { "inf" } else { "-inf" }.to_string()
    } else {
        format!("{x:.digits$}")
    }
}

/// `f"{x:g}"`: six significant digits, the zeros after them dropped, and an exponent when the
/// number is small or large enough.
pub fn format_g(x: f64) -> String {
    if x.is_nan() || x.is_infinite() {
        return fixed(x, 0);
    }
    let trimmed = |text: &str| -> String {
        if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            text.to_string()
        }
    };
    let scientific = format!("{x:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    if !(-4..6).contains(&exponent) {
        let sign = if exponent < 0 { '-' } else { '+' };
        return format!("{}e{sign}{:02}", trimmed(mantissa), exponent.abs());
    }
    trimmed(&format!("{x:.*}", (5 - exponent) as usize))
}

/// `textwrap.wrap(text, width)` with its defaults: the lines the text breaks into, none longer
/// than `width` characters unless a word is. A width below 1 is the ValueError Python raises.
#[track_caller]
pub fn wrap(text: &str, width: i64) -> Result<Vec<String>> {
    let chunks = wrap_split(text);
    if width <= 0 {
        return Err(Error::value(format!("invalid width {width} (must be > 0)")));
    }
    Ok(wrap_chunks(chunks, width as usize))
}

/// `\w`: a letter, a number or an underscore.
fn is_word(c: char) -> bool {
    c == '_' || is_alnum(c)
}

/// `[^\d\W]`.
fn is_letter(c: char) -> bool {
    is_word(c) && !is_decimal(c)
}

/// `[\w!"'&.,?]`.
fn is_word_punct(c: char) -> bool {
    is_word(c) || matches!(c, '!' | '"' | '\'' | '&' | '.' | ',' | '?')
}

/// `textwrap._whitespace`: what `TextWrapper` turns into a space and splits chunks on.
fn is_wrap_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0b' | '\x0c' | '\r' | ' ')
}

/// Whether a run of two or more hyphens starts at `at` with a word character after it.
fn dashes_then_word(text: &[char], at: usize) -> bool {
    let run = text[at..].iter().take_while(|c| **c == '-').count();
    run >= 2 && text.get(at + run).is_some_and(|c| is_word(*c))
}

/// Where the chunk starting at `at` ends: `TextWrapper.wordsep_re` by hand, since the regex crate
/// has no look-around. A chunk is a run of whitespace, a run of hyphens between words, or a word
/// that ends at whitespace, after a hyphen inside it, or before a run of hyphens.
fn chunk_end(text: &[char], at: usize) -> usize {
    if is_wrap_space(text[at]) {
        return at + text[at..].iter().take_while(|c| is_wrap_space(**c)).count();
    }
    if at > 0 && is_word_punct(text[at - 1]) && dashes_then_word(text, at) {
        return at + text[at..].iter().take_while(|c| **c == '-').count();
    }
    let letter = |at: usize| text.get(at).is_some_and(|c| is_letter(*c));
    let mut end = at + 1;
    loop {
        if end >= text.len() || is_wrap_space(text[end]) {
            return end;
        }
        if text[end] == '-' {
            let behind = (end >= 2 && letter(end - 2) && letter(end - 1))
                || (end >= 3 && letter(end - 3) && text[end - 2] == '-' && letter(end - 1));
            let ahead = letter(end + 1)
                && (letter(end + 2) || (text.get(end + 2) == Some(&'-') && letter(end + 3)));
            if behind && ahead {
                return end + 1;
            }
        }
        if is_word_punct(text[end - 1]) && dashes_then_word(text, end) {
            return end;
        }
        end += 1;
    }
}

/// `_munge_whitespace` and `_split`: tabs expanded, every other whitespace character a space,
/// and the text cut into chunks.
fn wrap_split(text: &str) -> Vec<Vec<char>> {
    let mut munged: Vec<char> = Vec::new();
    let mut column = 0;
    for c in text.chars() {
        if c == '\t' {
            let spaces = 8 - column % 8;
            munged.extend(std::iter::repeat_n(' ', spaces));
            column += spaces;
        } else {
            munged.push(if is_wrap_space(c) { ' ' } else { c });
            column = if c == '\n' || c == '\r' {
                0
            } else {
                column + 1
            };
        }
    }
    let mut chunks = Vec::new();
    let mut at = 0;
    while at < munged.len() {
        let end = chunk_end(&munged, at);
        chunks.push(munged[at..end].to_vec());
        at = end;
    }
    chunks
}

fn blank(chunk: &[char]) -> bool {
    chunk.iter().all(|c| is_space(*c))
}

/// `_handle_long_word`: put as much of a word too long for any line as fits on this one, breaking
/// after its last hyphen when there is one.
fn handle_long_word(
    reversed_chunks: &mut Vec<Vec<char>>,
    cur_line: &mut Vec<Vec<char>>,
    cur_len: usize,
    width: usize,
) {
    // Figure out when indent is larger than the specified width, and make
    // sure at least one character is stripped off on every pass
    let space_left = if width < 1 { 1 } else { width - cur_len };
    // If we're allowed to break long words, then do so: put as much
    // of the next chunk onto the current line as will fit.
    if space_left > 0 {
        let mut end = space_left;
        let chunk = reversed_chunks[reversed_chunks.len() - 1].clone();
        if chunk.len() > space_left {
            // break after last hyphen, but only if there are
            // non-hyphens before it
            let hyphen = chunk[..space_left].iter().rposition(|c| *c == '-');
            if let Some(hyphen) = hyphen
                && hyphen > 0
                && chunk[..hyphen].iter().any(|c| *c != '-')
            {
                end = hyphen + 1;
            }
        }
        cur_line.push(chunk[..end].to_vec());
        let last = reversed_chunks.len() - 1;
        reversed_chunks[last] = chunk[end..].to_vec();
    } else if cur_line.is_empty() {
        // Otherwise, we have to preserve the long word intact.  Only add
        // it to the current line if there's nothing already there --
        // that minimizes how much we violate the width constraint.
        cur_line.extend(reversed_chunks.pop());
    }
}

/// `_wrap_chunks`: lines of `width` characters or fewer, whitespace dropped from both ends of each.
fn wrap_chunks(mut chunks: Vec<Vec<char>>, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    // Arrange in reverse order so items can be efficiently popped
    // from a stack of chucks.
    chunks.reverse();
    while !chunks.is_empty() {
        let mut cur_line: Vec<Vec<char>> = Vec::new();
        let mut cur_len = 0;
        // First chunk on line is whitespace -- drop it, unless this
        // is the very beginning of the text (ie. no lines started yet).
        if !lines.is_empty() && chunks.last().is_some_and(|chunk| blank(chunk)) {
            chunks.pop();
        }
        while let Some(chunk) = chunks.last() {
            let length = chunk.len();
            // Can at least squeeze this chunk onto the current line.
            if cur_len + length <= width {
                cur_line.extend(chunks.pop());
                cur_len += length;
            } else {
                // Nope, this line is full.
                break;
            }
        }
        // The current line is full, and the next chunk is too big to
        // fit on *any* line (not just this one).
        if chunks.last().is_some_and(|chunk| chunk.len() > width) {
            handle_long_word(&mut chunks, &mut cur_line, cur_len, width);
        }
        // If the last chunk on this line is all whitespace, drop it. Python also takes it off
        // `cur_len`, which only the `max_lines` branch reads, and that is not ported.
        if cur_line.last().is_some_and(|chunk| blank(chunk)) {
            cur_line.pop();
        }
        if !cur_line.is_empty() {
            lines.push(cur_line.concat().into_iter().collect());
        }
    }
    lines
}

/// `ntpath.splitroot(path)`: the drive, the root and the rest of a Windows path, which together
/// are the path. Either separator is read; the pieces keep what the path wrote.
pub fn splitroot(path: &str) -> (&str, &str, &str) {
    // `/` and `\` are one byte each, so a byte index found in the normalised text is a boundary
    // of the original.
    let separator = |c: char| c == '\\' || c == '/';
    let mut chars = path.chars();
    let (first, second, third) = (chars.next(), chars.next(), chars.next());
    if first.is_some_and(separator) {
        if second.is_some_and(separator) {
            // UNC drives, e.g. \\server\share or \\?\UNC\server\share, and device drives, e.g.
            // \\.\device or \\?\device.
            let prefix: String = path.chars().take(8).collect();
            let unc = prefix.replace('/', "\\").to_uppercase() == "\\\\?\\UNC\\";
            let start = if unc { 8 } else { 2 };
            // `start` counts characters; every character before it is ASCII when `unc` is true,
            // and is `\` or `/` when it is not.
            let find = |from: usize| path[from..].find(separator).map(|i| i + from);
            let Some(index) = find(start) else {
                return (path, "", "");
            };
            let Some(index2) = find(index + 1) else {
                return (path, "", "");
            };
            return (
                &path[..index2],
                &path[index2..index2 + 1],
                &path[index2 + 1..],
            );
        }
        return ("", &path[..1], &path[1..]);
    }
    if second == Some(':') {
        let drive = first.map_or(0, char::len_utf8) + 1;
        if third.is_some_and(separator) {
            return (&path[..drive], &path[drive..drive + 1], &path[drive + 1..]);
        }
        return (&path[..drive], "", &path[drive..]);
    }
    ("", "", path)
}

/// `str(Path(text))` on Windows: separators made backslashes, doubled ones and `.` parts gone, a
/// trailing one dropped, and an empty path `.`.
pub fn path_text(text: &str) -> String {
    let normal = text.replace('/', "\\");
    let (drive, mut root, rest) = splitroot(&normal);
    if root.is_empty() && drive.starts_with('\\') && !drive.ends_with('\\') {
        // `\\server\share` and `\\?\unc\server\share` are a root of their own.
        let parts = drive.split('\\').count();
        if (parts == 4 && !matches!(drive.split('\\').nth(2), Some("" | "?" | "."))) || parts == 6 {
            root = "\\";
        }
    }
    let mut tail: Vec<&str> = rest
        .split('\\')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();
    if drive.is_empty()
        && root.is_empty()
        && tail
            .first()
            .is_some_and(|part| !splitroot(part).0.is_empty())
    {
        tail.insert(0, ".");
    }
    let text = format!("{drive}{root}{}", tail.join("\\"));
    if text.is_empty() {
        ".".to_string()
    } else {
        text
    }
}

/// `str(path)` for a `pathlib.Path` on Windows.
pub fn path_str(path: &std::path::Path) -> String {
    path_text(&path.to_string_lossy())
}

/// `Path(text)` on Windows.
pub fn to_path(text: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(path_text(text))
}

/// `os.path.normcase(path)` on Windows: backslashes, and lower case as Windows itself lowers it
/// (`LCMapStringEx`, which Python calls), not as Unicode does: Windows leaves some 475 characters
/// that `str.lower()` lowers, from U+0130 to the Garay capitals, as they are.
pub fn normcase(path: &str) -> String {
    let path = path.replace('/', "\\");
    #[cfg(windows)]
    if !path.is_empty() {
        use windows_sys::Win32::Globalization::{LCMAP_LOWERCASE, LCMapStringEx};
        let wide: Vec<u16> = path.encode_utf16().collect();
        let mut lowered = vec![0u16; wide.len()];
        // LOCALE_NAME_INVARIANT is the empty name. Lower-casing never changes a string's length.
        let invariant = [0u16];
        let written = unsafe {
            LCMapStringEx(
                invariant.as_ptr(),
                LCMAP_LOWERCASE,
                wide.as_ptr(),
                wide.len() as i32,
                lowered.as_mut_ptr(),
                lowered.len() as i32,
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        if written > 0 {
            return String::from_utf16_lossy(&lowered[..written as usize]);
        }
    }
    path.to_lowercase()
}

/// `os.path.normpath(path)` on Windows.
pub fn normpath(path: &str) -> String {
    let path = path.replace('/', "\\");
    let (drive, root, rest) = splitroot(&path);
    let prefix = format!("{drive}{root}");
    let mut comps: Vec<&str> = rest.split('\\').collect();
    let mut i = 0;
    while i < comps.len() {
        if comps[i].is_empty() || comps[i] == "." {
            comps.remove(i);
        } else if comps[i] == ".." {
            if i > 0 && comps[i - 1] != ".." {
                comps.drain(i - 1..=i);
                i -= 1;
            } else if i == 0 && !root.is_empty() {
                comps.remove(i);
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    if prefix.is_empty() && comps.is_empty() {
        comps.push(".");
    }
    format!("{prefix}{}", comps.join("\\"))
}

/// The text helpers against what CPython 3.14 gave (and, over every code point and generated
/// inputs, against CPython itself while they were written).
#[cfg(test)]
mod text_checks {
    use super::*;

    fn message<T>(result: Result<T>) -> String {
        result.err().expect("an error").message
    }

    #[test]
    fn python_character_classes() {
        // A superscript is a digit and no decimal; `int()` reads only the decimal ones.
        assert!(is_digit('\u{b2}') && !is_decimal('\u{b2}'));
        assert!(is_digit('\u{663}') && is_decimal('\u{663}') && !is_digit('a'));
        assert!(isdigit("1\u{b2}") && !isdigit(""));
        assert!(is_alpha('\u{e9}') && !is_alpha('\u{b2}') && is_alnum('\u{b2}') && !is_alnum('-'));
    }

    #[test]
    fn python_splitlines() {
        let text = "a\r\nb\x1c\u{2028}c\n\r";
        assert_eq!(splitlines(text), ["a", "b", "", "c", ""]);
        assert_eq!(
            splitlines_keepends(text),
            ["a\r\n", "b\x1c", "\u{2028}", "c\n", "\r"]
        );
        assert!(splitlines("").is_empty());
    }

    #[test]
    fn python_strict_utf16_decode() {
        let codec = "'utf-16-le' codec can't decode";
        assert_eq!(
            decode_utf16le_strict(b"\x00\xd8\x00\xdc").unwrap(),
            "\u{10000}"
        );
        for (data, reason) in [
            (&b"A\x00B"[..], "byte 0x42 in position 2: truncated data"),
            (
                b"A\x00\x00\xd8",
                "bytes in position 2-3: unexpected end of data",
            ),
            (
                b"A\x00\x00\xd8A\x00",
                "bytes in position 2-3: illegal UTF-16 surrogate",
            ),
            (b"\x00\xdc", "bytes in position 0-1: illegal encoding"),
        ] {
            assert_eq!(
                message(decode_utf16le_strict(data)),
                format!("{codec} {reason}")
            );
        }
    }

    #[test]
    fn python_int_of_text() {
        assert_eq!(int_base10(" -1_000 ").unwrap(), -1000);
        assert_eq!(int_base10("\u{663}\u{664}").unwrap(), 34);
        assert_eq!(
            message(int_base10("\u{b2}")),
            "invalid literal for int() with base 10: '\u{b2}'"
        );
        assert!(int_base10("1__0").is_err() && int_base10("\x1c1").is_err());
        assert!(
            message(int_base10(&"9".repeat(4301))).starts_with("Exceeds the limit (4300 digits)")
        );
        assert_eq!(int_base10(&"9".repeat(30)).unwrap(), i64::MAX);
        assert_eq!(int_base16("0x_ff").unwrap(), 255);
        assert_eq!(int_base16("-8000000000000000").unwrap(), i64::MIN);
        assert_eq!(int_base16("\u{663}f").unwrap(), 63);
    }

    #[test]
    fn python_fromhex() {
        assert_eq!(fromhex("ab cd\n").unwrap(), [0xab, 0xcd]);
        assert_eq!(
            message(fromhex("a b")),
            "non-hexadecimal number found in fromhex() arg at position 1"
        );
        assert_eq!(
            message(fromhex("abc")),
            "fromhex() arg must contain an even number of hexadecimal digits"
        );
        // The first character beyond ASCII is named, not the bad one before it.
        assert_eq!(
            message(fromhex("zz\u{e9}")),
            "non-hexadecimal number found in fromhex() arg at position 2"
        );
    }

    #[test]
    fn python_float_of_text() {
        assert_eq!(float_of("1_0.5e1").unwrap(), 105.0);
        assert_eq!(float_of("\u{661}\u{662}").unwrap(), 12.0);
        assert_eq!(float_of(" 5. ").unwrap(), 5.0);
        assert_eq!(float_of("-INF").unwrap(), f64::NEG_INFINITY);
        assert!(float_of("nan").unwrap().is_nan());
        for bad in ["1e", "\x1c1", "0x10", "1__0"] {
            assert_eq!(
                message(float_of(bad)),
                format!("could not convert string to float: {}", repr(bad))
            );
        }
    }

    #[test]
    fn python_float_formats() {
        assert_eq!(fixed(2.5, 0), "2");
        assert_eq!(fixed(0.125, 2), "0.12");
        assert_eq!(fixed(-0.0, 1), "-0.0");
        assert_eq!(fixed(f64::NAN, 2), "nan");
        assert_eq!(fixed(f64::NEG_INFINITY, 2), "-inf");
        assert_eq!(format_g(0.5), "0.5");
        assert_eq!(format_g(1234565.0), "1.23456e+06");
        assert_eq!(format_g(1e-5), "1e-05");
        assert_eq!(format_g(100000.0), "100000");
        assert_eq!(format_g(999999.5), "1e+06");
        assert_eq!(format_g(-0.0), "-0");
    }

    #[test]
    fn python_textwrap() {
        assert_eq!(
            wrap("Look, goof-ball -- use the -b option!", 10).unwrap(),
            ["Look,", "goof-ball", "-- use the", "-b option!"]
        );
        assert_eq!(
            wrap("aaaaaaaaaaaa bc", 5).unwrap(),
            ["aaaaa", "aaaaa", "aa bc"]
        );
        assert_eq!(
            wrap("x\ty \u{b2}-\u{b2}-end", 6).unwrap(),
            ["x", "y \u{b2}-\u{b2}-", "end"]
        );
        assert_eq!(message(wrap("abc", 0)), "invalid width 0 (must be > 0)");
    }

    #[test]
    fn python_windows_paths() {
        assert_eq!(path_text("//server/share/x"), "\\\\server\\share\\x");
        assert_eq!(path_text("a/./b//c/"), "a\\b\\c");
        assert_eq!(path_text(""), ".");
        assert_eq!(path_text("./c:/x"), ".\\c:\\x");
        assert_eq!(normpath("C:/a/../b/./c//"), "C:\\b\\c");
        assert_eq!(normpath("\\..\\x"), "\\x");
        assert_eq!(normpath(""), ".");
        // Windows leaves U+0130 as it is, where `str.lower()` makes it two characters.
        assert_eq!(normcase("C:/Users/\u{130}X"), "c:\\users\\\u{130}x");
    }
}

/// How CPython lays a `set` of ints out, for the order Python iterates one in (hash order, not
/// insertion order).
///
/// A set's order reaches what pa3ds reads and says wherever one is iterated: `layout.track` reads
/// each work in the order Python iterates `{*works, *known}`, and an insertion-ordered set read
/// them in another order. The table is CPython 3.14's, slot for slot, written against
/// `Objects/setobject.c` (`set_add_entry`, `set_discard_entry`, `set_table_resize`, `set_merge`,
/// `set_update_internal`, `set_copy`, `set_difference`, `set_intersection`), so removing a key
/// leaves a dummy that later adds reuse and a resize purges, as it does there. Ints hash as
/// CPython hashes them (modulo 2**61 - 1). Checked against CPython in `set_checks` below.
#[derive(Debug, Clone)]
pub struct PySet {
    table: Vec<Slot>,
    /// Slots that hold a key or a dummy, which is what a resize is triggered by: a discarded key
    /// makes no room until the table is rebuilt.
    fill: usize,
    used: usize,
}

/// One slot of a set's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Free,
    /// Where a key was discarded: a search goes on past it, an add may reuse it.
    Dummy,
    Key(i64),
}

impl Default for PySet {
    fn default() -> Self {
        PySet::new()
    }
}

impl PySet {
    const LINEAR_PROBES: usize = 9;
    const PERTURB_SHIFT: u32 = 5;
    const MIN_SIZE: usize = 8;

    /// `set()`.
    pub fn new() -> Self {
        PySet {
            table: vec![Slot::Free; Self::MIN_SIZE],
            fill: 0,
            used: 0,
        }
    }

    /// `hash(key)`.
    fn hash(key: i64) -> i64 {
        const MODULUS: u64 = (1 << 61) - 1;
        let magnitude = (key.unsigned_abs() % MODULUS) as i64;
        let hash = if key < 0 { -magnitude } else { magnitude };
        if hash == -1 { -2 } else { hash }
    }

    /// The slots a probe of a table of `mask + 1` slots visits from slot `i`: ten in a row, or
    /// one where the row would run off the end.
    fn row(i: usize, mask: usize) -> std::ops::RangeInclusive<usize> {
        i..=i + if i + Self::LINEAR_PROBES <= mask {
            Self::LINEAR_PROBES
        } else {
            0
        }
    }

    /// `set_lookkey`: the slot holding `key`, or else the free slot that ends the search.
    fn lookkey(&self, key: i64) -> usize {
        let mask = self.table.len() - 1;
        let hash = Self::hash(key);
        let mut perturb = hash as u64;
        let mut i = (hash as u64 as usize) & mask;
        loop {
            for at in Self::row(i, mask) {
                match self.table[at] {
                    Slot::Free => return at,
                    Slot::Key(held) if held == key => return at,
                    _ => {}
                }
            }
            perturb >>= Self::PERTURB_SHIFT;
            i = i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize)
                & mask;
        }
    }

    /// `set_insert_clean`: `key`, known to be absent, into the first free slot of its probe, in a
    /// table with no dummies.
    fn insert_clean(table: &mut [Slot], key: i64) {
        let mask = table.len() - 1;
        let hash = Self::hash(key);
        let mut perturb = hash as u64;
        let mut i = (hash as u64 as usize) & mask;
        loop {
            for at in Self::row(i, mask) {
                if table[at] == Slot::Free {
                    table[at] = Slot::Key(key);
                    return;
                }
            }
            perturb >>= Self::PERTURB_SHIFT;
            i = i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize)
                & mask;
        }
    }

    /// `set.add(key)`.
    pub fn add(&mut self, key: i64) {
        let mask = self.table.len() - 1;
        let hash = Self::hash(key);
        let mut perturb = hash as u64;
        let mut i = (hash as u64 as usize) & mask;
        // The last dummy met on the way, which is where the key goes if it is not found.
        let mut freeslot = None;
        let free = 'probing: loop {
            for at in Self::row(i, mask) {
                match self.table[at] {
                    Slot::Free => break 'probing at,
                    Slot::Key(held) if held == key => return,
                    Slot::Key(_) => {}
                    Slot::Dummy => freeslot = Some(at),
                }
            }
            perturb >>= Self::PERTURB_SHIFT;
            i = i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize)
                & mask;
        };
        if let Some(at) = freeslot {
            self.table[at] = Slot::Key(key);
            self.used += 1;
            return;
        }
        self.table[free] = Slot::Key(key);
        self.fill += 1;
        self.used += 1;
        if self.fill * 5 < mask * 3 {
            return;
        }
        self.resize(self.grown());
    }

    /// What a set that needs room is resized to: four times its keys, and twice from 50000.
    fn grown(&self) -> usize {
        if self.used > 50000 {
            self.used * 2
        } else {
            self.used * 4
        }
    }

    /// `set_table_resize`: a table of the least power of two slots above `minused`, every key put
    /// in again in the order the old table holds them, and the dummies gone. A table that is
    /// already the smallest and has no dummy is left as it is.
    fn resize(&mut self, minused: usize) {
        let mut size = Self::MIN_SIZE;
        while size <= minused {
            size <<= 1;
        }
        if size == Self::MIN_SIZE && self.table.len() == Self::MIN_SIZE && self.fill == self.used {
            return;
        }
        let old = std::mem::replace(&mut self.table, vec![Slot::Free; size]);
        for slot in old {
            if let Slot::Key(key) = slot {
                Self::insert_clean(&mut self.table, key);
            }
        }
        self.fill = self.used;
    }

    /// `set_merge`: every key of `other` added, sized for them all at once, and where this set is
    /// empty and its table the size of the other's, slot for slot as the other has them.
    fn merge(&mut self, other: &PySet) {
        if other.used == 0 {
            return;
        }
        let mask = self.table.len() - 1;
        if (self.fill + other.used) * 5 >= mask * 3 {
            self.resize((self.used + other.used) * 2);
        }
        if self.fill == 0 && self.table.len() == other.table.len() && other.fill == other.used {
            for (at, slot) in other.table.iter().enumerate() {
                if let Slot::Key(_) = slot {
                    self.table[at] = *slot;
                }
            }
            self.fill = other.fill;
            self.used = other.used;
            return;
        }
        if self.fill == 0 {
            self.fill = other.used;
            self.used = other.used;
            for slot in &other.table {
                if let Slot::Key(key) = slot {
                    Self::insert_clean(&mut self.table, *key);
                }
            }
            return;
        }
        for key in other.iter() {
            self.add(key);
        }
    }

    /// `set.update(some_dict)`, which makes room for every key before adding any. `keys` are the
    /// dict's, in its order.
    pub fn update_dict(&mut self, keys: &[i64]) {
        let mask = self.table.len() - 1;
        if (self.fill + keys.len()) * 5 >= mask * 3 {
            self.resize((self.used + keys.len()) * 2);
        }
        for &key in keys {
            self.add(key);
        }
    }

    /// `set.update(other_set)`.
    pub fn update(&mut self, other: &PySet) {
        self.merge(other);
    }

    /// `set(iterable)` of an iterator, which adds one key at a time.
    pub fn of(keys: impl IntoIterator<Item = i64>) -> Self {
        let mut set = PySet::new();
        for key in keys {
            set.add(key);
        }
        set
    }

    /// `set(some_dict)`.
    pub fn of_dict(keys: &[i64]) -> Self {
        let mut set = PySet::new();
        set.update_dict(keys);
        set
    }

    /// `set.copy()` and `set(some_set)`: a new table sized for the keys, where a table of the
    /// same size keeps every key in its slot and any other puts them in again in the order
    /// this one holds them.
    pub fn copy(&self) -> Self {
        let mut copy = PySet::new();
        copy.merge(self);
        copy
    }

    /// `set.clear()`: back to the smallest table.
    pub fn clear(&mut self) {
        *self = PySet::new();
    }

    /// `set.discard(key)`: whether it was there. The slot becomes a dummy.
    pub fn discard(&mut self, key: i64) -> bool {
        let at = self.lookkey(key);
        if self.table[at] == Slot::Free {
            return false;
        }
        self.table[at] = Slot::Dummy;
        self.used -= 1;
        true
    }

    /// `set.remove(key)`: `KeyError(key)` where it is not there.
    #[track_caller]
    pub fn remove(&mut self, key: i64) -> Result<()> {
        if self.discard(key) {
            Ok(())
        } else {
            Err(Error::key(key.to_string()))
        }
    }

    /// `key in set`.
    pub fn contains(&self, key: i64) -> bool {
        self.table[self.lookkey(key)] != Slot::Free
    }

    /// `len(set)`.
    pub fn len(&self) -> usize {
        self.used
    }

    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// The keys in the order Python iterates them.
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        self.table.iter().filter_map(|slot| match slot {
            Slot::Key(key) => Some(*key),
            _ => None,
        })
    }

    /// `set - other` (`set_difference`): where this set has more than four times the keys of
    /// `other` it is copied and `other`'s keys discarded from the copy, and otherwise built again,
    /// key by key in this set's order, from those `other` lacks.
    pub fn difference(&self, other: &PySet) -> PySet {
        if (self.used >> 2) > other.used {
            let mut result = self.copy();
            result.difference_update(other);
            return result;
        }
        let mut result = PySet::new();
        for key in self.iter() {
            if !other.contains(key) {
                result.add(key);
            }
        }
        result
    }

    /// `set -= other` (`set_difference_update_internal`): a table more than a quarter dummies is
    /// rebuilt after.
    pub fn difference_update(&mut self, other: &PySet) {
        // Where `other` is more than eight times this set, only the keys they share are discarded.
        let shared;
        let other = if (other.used >> 3) > self.used {
            shared = self.intersection(other);
            &shared
        } else {
            other
        };
        for key in other.iter() {
            self.discard(key);
        }
        if self.fill - self.used <= (self.table.len() - 1) / 4 {
            return;
        }
        self.resize(self.grown());
    }

    /// `set | other` (`set_or`): a copy of this set, then every key of `other` added.
    pub fn union(&self, other: &PySet) -> PySet {
        let mut result = self.copy();
        result.merge(other);
        result
    }

    /// `set & other` (`set_intersection`): the keys of the smaller set that the larger holds,
    /// added in the smaller one's order.
    pub fn intersection(&self, other: &PySet) -> PySet {
        let (larger, smaller) = if other.used > self.used {
            (other, self)
        } else {
            (self, other)
        };
        let mut result = PySet::new();
        for key in smaller.iter() {
            if larger.contains(key) {
                result.add(key);
            }
        }
        result
    }

    /// `set.intersection_update(other)`: this set's table is replaced by the intersection's.
    pub fn intersection_update(&mut self, other: &PySet) {
        *self = self.intersection(other);
    }
}

/// `sorted(items, key=...)` of a key that only has `<`, in CPython's order.
///
/// Rust's `sort_by` gives the same order only while the keys are totally ordered. Once a key is
/// NaN, `<` is false both ways and the result depends on which pairs timsort happens to compare,
/// so this follows CPython's steps (`listsort` in `Objects/listobject.c`) rather than its
/// contract. See `sort_by_lt`.
///
/// Returns the order as a permutation: `order[i]` is the index in `items` of what ends up at
/// position `i`.
fn timsort_order<T>(items: &[T], mut lt: impl FnMut(&T, &T) -> bool) -> Vec<usize> {
    let len = items.len();
    let mut sorter = Timsort {
        lt: |x: usize, y: usize| lt(&items[x], &items[y]),
        a: (0..len).collect(),
        pending: Vec::new(),
        min_gallop: MIN_GALLOP,
    };
    if len < 2 {
        return sorter.a;
    }
    // Natural runs are found left to right, and a short one is extended to `minrun` by binary
    // insertion before it is pushed.
    let minrun = compute_minrun(len);
    let (mut lo, mut remaining) = (0, len);
    while remaining > 0 {
        let mut n = sorter.count_run(lo, remaining);
        if n < minrun {
            let force = remaining.min(minrun);
            sorter.binarysort(lo, force, n);
            n = force;
        }
        sorter.found_new_run(n);
        sorter.pending.push(Run {
            base: lo,
            len: n,
            power: 0,
        });
        lo += n;
        remaining -= n;
    }
    sorter.force_collapse();
    sorter.a
}

/// `items.sort(key=...)` where `lt(a, b)` means `key(a) < key(b)`: stable and in place, in the
/// order CPython 3.14 gives, comparing the same pairs in the same sequence.
///
/// The sequence matters because a NaN key makes `<` inconsistent (not a total order), and then
/// only the exact algorithm, `listsort` and what it calls (`count_run`, `binarysort`,
/// `merge_compute_minrun`, `found_new_run` with `powerloop`, `merge_force_collapse`, `merge_at`,
/// `merge_lo`, `merge_hi`, `gallop_left`, `gallop_right`), reproduces its output. Only `<` is
/// ever asked of the keys, as in CPython, which infers equality from two false answers.
///
/// What this does not follow is CPython's choice of comparison function for the keys: float keys
/// compare as `<` on the floats, which is what `lt` should do, but a tuple key is compared with
/// `==` on its elements first (`unsafe_tuple_compare`), and `lt` has to do the same for the
/// result to match.
pub fn sort_by_lt<T>(items: &mut [T], lt: impl FnMut(&T, &T) -> bool) {
    let mut order = timsort_order(items, lt);
    // `order[at]` is where the item that belongs at `at` is now. Follow each cycle once, carrying
    // the item that started it along in the swaps, so `T` needs no `Clone`.
    for start in 0..order.len() {
        let mut at = start;
        while order[at] != start {
            let from = order[at];
            order[at] = at;
            items.swap(at, from);
            at = from;
        }
        order[at] = at;
    }
}

/// `sorted(items, key=...)` where `lt(a, b)` means `key(a) < key(b)`. See `sort_by_lt`.
pub fn sorted_by_lt<T>(mut items: Vec<T>, lt: impl FnMut(&T, &T) -> bool) -> Vec<T> {
    sort_by_lt(&mut items, lt);
    items
}

/// Galloping starts after this many wins in a row, to begin with.
const MIN_GALLOP: usize = 7;

/// The longest `minrun`, and so the longest stretch `binarysort` is given.
const MAX_MINRUN: usize = 64;

/// `merge_compute_minrun`: the length short natural runs are extended to, chosen so that the
/// number of runs is close to, and below, a power of two.
fn compute_minrun(mut n: usize) -> usize {
    let mut r = 0;
    while n >= MAX_MINRUN {
        r |= n & 1;
        n >>= 1;
    }
    n + r
}

/// A run of `len` items starting at `base` in the array being sorted, and its depth in
/// powersort's merge tree (CPython's `struct s_slice`).
struct Run {
    base: usize,
    len: usize,
    power: u32,
}

/// CPython's `MergeState`. It sorts item indices, never the items, so the array `a` is what
/// CPython calls the list of pointers and a move of an index is a move of a pointer.
struct Timsort<F> {
    lt: F,
    a: Vec<usize>,
    pending: Vec<Run>,
    /// CPython's `ms->min_gallop`. Merges read it once, work on a local copy, and write it back
    /// only where CPython does, which is inside galloping mode.
    min_gallop: usize,
}

impl<F: FnMut(usize, usize) -> bool> Timsort<F> {
    /// `a[x] < a[y]` for two positions in `self.a`.
    fn lt_at(&mut self, x: usize, y: usize) -> bool {
        (self.lt)(self.a[x], self.a[y])
    }

    /// `binarysort`: `a[lo..lo + n]` with `a[lo..lo + ok]` already in order.
    fn binarysort(&mut self, lo: usize, n: usize, mut ok: usize) {
        if ok == 0 {
            ok += 1;
        }
        while ok < n {
            let pivot = self.a[lo + ok];
            let (mut l, mut r) = (0, ok);
            loop {
                let m = (l + r) >> 1;
                if (self.lt)(pivot, self.a[lo + m]) {
                    r = m;
                } else {
                    l = m + 1;
                }
                if l >= r {
                    break;
                }
            }
            self.a[lo + l..=lo + ok].rotate_right(1);
            ok += 1;
        }
    }

    /// `count_run`: the length of the monotone run at `lo`, made ascending in place. Written for
    /// 3.14, which reverses a descending run's equal stretches as it goes so that the whole-run
    /// reversal at the end leaves them in their original order.
    fn count_run(&mut self, lo: usize, remaining: usize) -> usize {
        let mut n = 1;
        while n < remaining {
            if self.lt_at(lo + n, lo + n - 1) {
                break;
            }
            n += 1;
        }
        if n == remaining {
            return n;
        }
        if n > 1 {
            if self.lt_at(lo, lo + n - 1) {
                return n;
            }
            self.a[lo..lo + n].reverse();
        }
        n += 1;

        // The last `neq + 1` items are equal; reverse them and start counting again.
        let mut neq = 0;
        while n < remaining {
            if self.lt_at(lo + n, lo + n - 1) {
                if neq > 0 {
                    neq += 1;
                    self.a[lo + n - neq..lo + n].reverse();
                    neq = 0;
                }
            } else if self.lt_at(lo + n - 1, lo + n) {
                break;
            } else {
                neq += 1;
            }
            n += 1;
        }
        if neq > 0 {
            neq += 1;
            self.a[lo + n - neq..lo + n].reverse();
        }
        self.a[lo..lo + n].reverse();

        while n < remaining {
            if self.lt_at(lo + n, lo + n - 1) {
                break;
            }
            n += 1;
        }
        n
    }

    /// `gallop_left`: where `key` goes in the sorted `a[..n]`, left of any equal items, starting
    /// the search at `hint`. Positions are signed because the search steps below zero before it
    /// clamps.
    fn gallop_left(lt: &mut F, key: usize, a: &[usize], n: isize, hint: isize) -> isize {
        let at = |i: isize| a[i as usize];
        let mut lastofs: isize = 0;
        let mut ofs: isize = 1;
        if lt(at(hint), key) {
            // a[hint] < key: gallop right until a[hint + lastofs] < key <= a[hint + ofs].
            let maxofs = n - hint;
            while ofs < maxofs {
                if lt(at(hint + ofs), key) {
                    lastofs = ofs;
                    ofs = (ofs << 1) + 1;
                } else {
                    break;
                }
            }
            ofs = ofs.min(maxofs);
            lastofs += hint;
            ofs += hint;
        } else {
            // key <= a[hint]: gallop left until a[hint - ofs] < key <= a[hint - lastofs].
            let maxofs = hint + 1;
            while ofs < maxofs {
                if lt(at(hint - ofs), key) {
                    break;
                }
                lastofs = ofs;
                ofs = (ofs << 1) + 1;
            }
            ofs = ofs.min(maxofs);
            let k = lastofs;
            lastofs = hint - ofs;
            ofs = hint - k;
        }
        // a[lastofs] < key <= a[ofs]: binary search between them.
        lastofs += 1;
        while lastofs < ofs {
            let m = lastofs + ((ofs - lastofs) >> 1);
            if lt(at(m), key) {
                lastofs = m + 1;
            } else {
                ofs = m;
            }
        }
        ofs
    }

    /// `gallop_right`: as `gallop_left`, but right of any equal items.
    fn gallop_right(lt: &mut F, key: usize, a: &[usize], n: isize, hint: isize) -> isize {
        let at = |i: isize| a[i as usize];
        let mut lastofs: isize = 0;
        let mut ofs: isize = 1;
        if lt(key, at(hint)) {
            // key < a[hint]: gallop left until a[hint - ofs] <= key < a[hint - lastofs].
            let maxofs = hint + 1;
            while ofs < maxofs {
                if lt(key, at(hint - ofs)) {
                    lastofs = ofs;
                    ofs = (ofs << 1) + 1;
                } else {
                    break;
                }
            }
            ofs = ofs.min(maxofs);
            let k = lastofs;
            lastofs = hint - ofs;
            ofs = hint - k;
        } else {
            // a[hint] <= key: gallop right until a[hint + lastofs] <= key < a[hint + ofs].
            let maxofs = n - hint;
            while ofs < maxofs {
                if lt(key, at(hint + ofs)) {
                    break;
                }
                lastofs = ofs;
                ofs = (ofs << 1) + 1;
            }
            ofs = ofs.min(maxofs);
            lastofs += hint;
            ofs += hint;
        }
        // a[lastofs] <= key < a[ofs]: binary search between them.
        lastofs += 1;
        while lastofs < ofs {
            let m = lastofs + ((ofs - lastofs) >> 1);
            if lt(key, at(m)) {
                ofs = m;
            } else {
                lastofs = m + 1;
            }
        }
        ofs
    }

    /// `merge_lo`: merge the `na` items at `ssa` with the `nb` at `ssb = ssa + na`, copying the
    /// `a` run out to a temporary array. The last of `a` is known to belong at the end, and
    /// `na <= nb`.
    fn merge_lo(&mut self, ssa: usize, mut na: usize, ssb: usize, mut nb: usize) {
        enum End {
            Succeed,
            CopyB,
        }
        let tmp: Vec<usize> = self.a[ssa..ssa + na].to_vec();
        let mut dest = ssa;
        let mut pa = 0; // into `tmp`
        let mut pb = ssb;

        self.a[dest] = self.a[pb];
        dest += 1;
        pb += 1;
        nb -= 1;

        let end = 'merge: {
            if nb == 0 {
                break 'merge End::Succeed;
            }
            if na == 1 {
                break 'merge End::CopyB;
            }
            let mut min_gallop = self.min_gallop;
            loop {
                let mut acount = 0;
                let mut bcount = 0;
                // The straightforward merge, until one run seems to win consistently.
                loop {
                    if (self.lt)(self.a[pb], tmp[pa]) {
                        self.a[dest] = self.a[pb];
                        dest += 1;
                        pb += 1;
                        bcount += 1;
                        acount = 0;
                        nb -= 1;
                        if nb == 0 {
                            break 'merge End::Succeed;
                        }
                        if bcount >= min_gallop {
                            break;
                        }
                    } else {
                        self.a[dest] = tmp[pa];
                        dest += 1;
                        pa += 1;
                        acount += 1;
                        bcount = 0;
                        na -= 1;
                        if na == 1 {
                            break 'merge End::CopyB;
                        }
                        if acount >= min_gallop {
                            break;
                        }
                    }
                }

                min_gallop += 1;
                loop {
                    min_gallop -= usize::from(min_gallop > 1);
                    self.min_gallop = min_gallop;
                    let k = Self::gallop_right(&mut self.lt, self.a[pb], &tmp[pa..], na as isize, 0)
                        as usize;
                    acount = k;
                    if k > 0 {
                        self.a[dest..dest + k].copy_from_slice(&tmp[pa..pa + k]);
                        dest += k;
                        pa += k;
                        na -= k;
                        if na == 1 {
                            break 'merge End::CopyB;
                        }
                        // Zero is impossible for a consistent `<`, which a NaN key is not.
                        if na == 0 {
                            break 'merge End::Succeed;
                        }
                    }
                    self.a[dest] = self.a[pb];
                    dest += 1;
                    pb += 1;
                    nb -= 1;
                    if nb == 0 {
                        break 'merge End::Succeed;
                    }

                    let k = Self::gallop_left(&mut self.lt, tmp[pa], &self.a[pb..], nb as isize, 0)
                        as usize;
                    bcount = k;
                    if k > 0 {
                        self.a.copy_within(pb..pb + k, dest);
                        dest += k;
                        pb += k;
                        nb -= k;
                        if nb == 0 {
                            break 'merge End::Succeed;
                        }
                    }
                    self.a[dest] = tmp[pa];
                    dest += 1;
                    pa += 1;
                    na -= 1;
                    if na == 1 {
                        break 'merge End::CopyB;
                    }
                    if acount < MIN_GALLOP && bcount < MIN_GALLOP {
                        break;
                    }
                }
                min_gallop += 1; // penalize it for leaving galloping mode
                self.min_gallop = min_gallop;
            }
        };
        match end {
            End::Succeed => {
                if na > 0 {
                    self.a[dest..dest + na].copy_from_slice(&tmp[pa..pa + na]);
                }
            }
            End::CopyB => {
                // The last of `a` belongs at the end of the merge.
                self.a.copy_within(pb..pb + nb, dest);
                self.a[dest + nb] = tmp[pa];
            }
        }
    }

    /// `merge_hi`: as `merge_lo`, from the right, copying the `b` run out. `na >= nb`.
    fn merge_hi(&mut self, ssa: usize, mut na: usize, ssb: usize, mut nb: usize) {
        enum End {
            Succeed,
            CopyA,
        }
        // Positions that walk down may end one before the array's start, so they are signed.
        let tmp: Vec<usize> = self.a[ssb..ssb + nb].to_vec();
        let mut dest = (ssb + nb - 1) as isize;
        let mut pa = (ssa + na - 1) as isize; // into `a`
        let mut pb = nb as isize - 1; // into `tmp`

        self.a[dest as usize] = self.a[pa as usize];
        dest -= 1;
        pa -= 1;
        na -= 1;

        let end = 'merge: {
            if na == 0 {
                break 'merge End::Succeed;
            }
            if nb == 1 {
                break 'merge End::CopyA;
            }
            let mut min_gallop = self.min_gallop;
            loop {
                let mut acount = 0;
                let mut bcount = 0;
                loop {
                    if (self.lt)(tmp[pb as usize], self.a[pa as usize]) {
                        self.a[dest as usize] = self.a[pa as usize];
                        dest -= 1;
                        pa -= 1;
                        acount += 1;
                        bcount = 0;
                        na -= 1;
                        if na == 0 {
                            break 'merge End::Succeed;
                        }
                        if acount >= min_gallop {
                            break;
                        }
                    } else {
                        self.a[dest as usize] = tmp[pb as usize];
                        dest -= 1;
                        pb -= 1;
                        bcount += 1;
                        acount = 0;
                        nb -= 1;
                        if nb == 1 {
                            break 'merge End::CopyA;
                        }
                        if bcount >= min_gallop {
                            break;
                        }
                    }
                }

                min_gallop += 1;
                loop {
                    min_gallop -= usize::from(min_gallop > 1);
                    self.min_gallop = min_gallop;
                    let k = Self::gallop_right(
                        &mut self.lt,
                        tmp[pb as usize],
                        &self.a[ssa..],
                        na as isize,
                        na as isize - 1,
                    );
                    let k = na - k as usize;
                    acount = k;
                    if k > 0 {
                        dest -= k as isize;
                        pa -= k as isize;
                        let (d, s) = ((dest + 1) as usize, (pa + 1) as usize);
                        self.a.copy_within(s..s + k, d);
                        na -= k;
                        if na == 0 {
                            break 'merge End::Succeed;
                        }
                    }
                    self.a[dest as usize] = tmp[pb as usize];
                    dest -= 1;
                    pb -= 1;
                    nb -= 1;
                    if nb == 1 {
                        break 'merge End::CopyA;
                    }

                    let k = Self::gallop_left(
                        &mut self.lt,
                        self.a[pa as usize],
                        &tmp,
                        nb as isize,
                        nb as isize - 1,
                    );
                    let k = nb - k as usize;
                    bcount = k;
                    if k > 0 {
                        dest -= k as isize;
                        pb -= k as isize;
                        let (d, s) = ((dest + 1) as usize, (pb + 1) as usize);
                        self.a[d..d + k].copy_from_slice(&tmp[s..s + k]);
                        nb -= k;
                        if nb == 1 {
                            break 'merge End::CopyA;
                        }
                        // Zero is impossible for a consistent `<`, which a NaN key is not.
                        if nb == 0 {
                            break 'merge End::Succeed;
                        }
                    }
                    self.a[dest as usize] = self.a[pa as usize];
                    dest -= 1;
                    pa -= 1;
                    na -= 1;
                    if na == 0 {
                        break 'merge End::Succeed;
                    }
                    if acount < MIN_GALLOP && bcount < MIN_GALLOP {
                        break;
                    }
                }
                min_gallop += 1; // penalize it for leaving galloping mode
                self.min_gallop = min_gallop;
            }
        };
        match end {
            End::Succeed => {
                if nb > 0 {
                    let start = (dest + 1) as usize - nb;
                    self.a[start..start + nb].copy_from_slice(&tmp[..nb]);
                }
            }
            End::CopyA => {
                // The first of `b` belongs at the front of the merge.
                let (d, s) = ((dest + 1) as usize - na, (pa + 1) as usize - na);
                self.a.copy_within(s..s + na, d);
                dest -= na as isize;
                self.a[dest as usize] = tmp[pb as usize];
            }
        }
    }

    /// `merge_at`: merge pending runs `i` and `i + 1`.
    fn merge_at(&mut self, i: usize) {
        let (mut ssa, mut na) = (self.pending[i].base, self.pending[i].len);
        let (ssb, mut nb) = (self.pending[i + 1].base, self.pending[i + 1].len);
        self.pending[i].len = na + nb;
        self.pending.remove(i + 1);

        // Where b starts in a; what comes before is already in place.
        let k =
            Self::gallop_right(&mut self.lt, self.a[ssb], &self.a[ssa..], na as isize, 0) as usize;
        ssa += k;
        na -= k;
        if na == 0 {
            return;
        }

        // Where a ends in b; what comes after is already in place.
        nb = Self::gallop_left(
            &mut self.lt,
            self.a[ssa + na - 1],
            &self.a[ssb..],
            nb as isize,
            nb as isize - 1,
        ) as usize;
        if nb == 0 {
            return;
        }
        if na <= nb {
            self.merge_lo(ssa, na, ssb, nb);
        } else {
            self.merge_hi(ssa, na, ssb, nb);
        }
    }

    /// `powerloop`: the depth in the merge tree of the boundary between the run of `n1` items at
    /// `s1` and the `n2` after it, in a list of `n`. It walks the binary expansions of the two
    /// runs' midpoints (doubled, to stay in integers) until a bit differs.
    fn powerloop(s1: usize, n1: usize, n2: usize, n: usize) -> u32 {
        let mut result = 0;
        let mut a = 2 * s1 + n1;
        let mut b = a + n1 + n2;
        loop {
            result += 1;
            if a >= n {
                a -= n;
                b -= n;
            } else if b >= n {
                break;
            }
            a <<= 1;
            b <<= 1;
        }
        result
    }

    /// `found_new_run`: a run of `n2` items was found after the top pending run; merge the
    /// pending runs of greater power. The caller pushes the new run.
    fn found_new_run(&mut self, n2: usize) {
        let Some(top) = self.pending.last() else {
            return;
        };
        let power = Self::powerloop(top.base, top.len, n2, self.a.len());
        while self.pending.len() > 1 && self.pending[self.pending.len() - 2].power > power {
            self.merge_at(self.pending.len() - 2);
        }
        let last = self.pending.len() - 1;
        self.pending[last].power = power;
    }

    /// `merge_force_collapse`: merge every pending run into one.
    fn force_collapse(&mut self) {
        while self.pending.len() > 1 {
            let mut n = self.pending.len() - 2;
            if n > 0 && self.pending[n - 1].len < self.pending[n + 1].len {
                n -= 1;
            }
            self.merge_at(n);
        }
    }
}

/// `sort_by_lt` against orders CPython 3.14 gave for float keys with NaNs in them, where the
/// order is only what timsort's steps make of it.
#[cfg(test)]
mod sort_checks {
    use super::sort_by_lt;

    fn order(keys: &[f64]) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..keys.len()).collect();
        sort_by_lt(&mut indices, |a, b| keys[*a] < keys[*b]);
        indices
    }

    #[test]
    fn sort_by_lt_orders_nan_as_cpython_does() {
        let nan = f64::NAN;
        assert_eq!(
            order(&[2.0, nan, 1.0, 0.0, 3.0, nan, -1.0]),
            [3, 0, 1, 2, 4, 5, 6]
        );
        assert_eq!(
            order(&[1.0, nan, 0.0, 2.0, -1.0, nan, 3.0, nan, -2.0]),
            [0, 1, 8, 4, 2, 3, 5, 6, 7]
        );

        // 100 items: past 64, so binary insertion runs, then merges with galloping.
        let scattered: Vec<f64> = (0..100)
            .map(|i| {
                if i % 7 == 3 {
                    nan
                } else {
                    ((i * 37) % 101) as f64
                }
            })
            .collect();
        assert_eq!(
            order(&scattered),
            [
                0, 41, 11, 33, 44, 14, 36, 1, 2, 3, 22, 25, 6, 47, 28, 39, 9, 20, 42, 12, 23, 34,
                4, 15, 26, 37, 7, 48, 18, 29, 40, 21, 32, 43, 13, 35, 5, 46, 16, 8, 10, 17, 19, 24,
                27, 30, 31, 38, 45, 71, 82, 93, 63, 74, 85, 50, 51, 52, 55, 96, 77, 88, 58, 99, 69,
                91, 61, 72, 83, 53, 64, 75, 86, 56, 97, 78, 89, 81, 92, 62, 84, 54, 57, 59, 67, 70,
                95, 65, 76, 98, 68, 79, 49, 90, 60, 66, 73, 80, 87, 94
            ]
        );

        // Three ascending runs of 40 with one NaN in the middle one.
        let runs: Vec<f64> = (0..120)
            .map(|i| {
                if i == 57 {
                    nan
                } else {
                    (i % 40) as f64 + (i / 40) as f64 * 0.5
                }
            })
            .collect();
        assert_eq!(
            order(&runs),
            [
                0, 40, 1, 41, 2, 42, 3, 43, 4, 44, 5, 45, 6, 46, 7, 47, 8, 48, 9, 49, 10, 50, 11,
                51, 12, 52, 13, 53, 14, 54, 15, 55, 16, 56, 17, 18, 58, 19, 59, 20, 21, 22, 23, 24,
                25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 57, 80, 81, 82, 83, 84,
                85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 60, 100, 61, 101, 62,
                102, 63, 103, 64, 104, 65, 105, 66, 106, 67, 107, 68, 108, 69, 109, 70, 110, 71,
                111, 72, 112, 73, 113, 74, 114, 75, 115, 76, 116, 77, 117, 78, 118, 79, 119
            ]
        );
    }
}

/// `PySet` against orders CPython 3.14 gave, over works like the field's (0x104 apart) and the
/// screens' (0x10000 apart), which collide in a small table.
#[cfg(test)]
mod set_checks {
    use super::{Kind, PySet};

    #[test]
    fn pyset_iterates_as_cpython_does() {
        let mut every = PySet::of([0x083002a4, 0x14B00000, 0x14B10000]);
        every.update_dict(&[0x14B00000, 0x14C00000, 0x14B20000]);
        assert_eq!(
            every.iter().collect::<Vec<_>>(),
            [0x14B00000, 0x14B10000, 0x14C00000, 0x14B20000, 0x083002a4]
        );
        let field: Vec<i64> = (0..12).map(|index| 0x08300000 + index * 0x104).collect();
        assert_eq!(
            PySet::of_dict(&field).iter().collect::<Vec<_>>(),
            [
                0x8300000, 0x8300820, 0x8300104, 0x8300924, 0x8300208, 0x8300a28, 0x830030c,
                0x8300b2c, 0x8300410, 0x8300514, 0x8300618, 0x830071c
            ]
        );
        assert_eq!(
            PySet::of([0x14B30600, 0x14B10600, 0x14B20600])
                .iter()
                .collect::<Vec<_>>(),
            [0x14B30600, 0x14B10600, 0x14B20600]
        );
        let mut small = PySet::of([5, 3, 1]);
        assert!(small.contains(3) && !small.contains(4) && small.len() == 3);
        small.update_dict(&[9, 2, 8]);
        assert_eq!(small.iter().collect::<Vec<_>>(), [1, 2, 3, 5, 8, 9]);
    }

    #[test]
    fn pyset_operations_iterate_as_cpython_does() {
        let works: Vec<i64> = (0..12).map(|index| 0x08300000 + index * 0x104).collect();
        assert_eq!(
            PySet::of_dict(&works)
                .difference(&PySet::of([0x08300208]))
                .iter()
                .collect::<Vec<_>>(),
            [
                0x8300000, 0x8300820, 0x8300104, 0x8300924, 0x8300a28, 0x830030c, 0x8300b2c,
                0x8300410, 0x8300514, 0x8300618, 0x830071c
            ]
        );
        assert_eq!(
            PySet::of([0x14B30600, 0x14B10600, 0x14B20600])
                .difference(&PySet::of([0x14B10600]))
                .iter()
                .collect::<Vec<_>>(),
            [0x14B30600, 0x14B20600]
        );
        // A discarded slot is taken by the next add that probes through it.
        let mut reused = PySet::of((0..40).step_by(3));
        for key in [3, 6, 9, 0] {
            assert!(reused.discard(key));
        }
        assert!(!reused.discard(3));
        for key in [100, 17, 8, 33] {
            reused.add(key);
        }
        assert_eq!(
            reused.iter().collect::<Vec<_>>(),
            [33, 36, 100, 39, 8, 12, 15, 17, 18, 21, 24, 27, 30]
        );
        // A copy is sized for its keys, so seven of them are laid out again.
        let seven = PySet::of([5, 3, 1, 12, 20, 33, 41]);
        assert_eq!(seven.iter().collect::<Vec<_>>(), [1, 33, 3, 5, 41, 12, 20]);
        assert_eq!(
            seven.copy().iter().collect::<Vec<_>>(),
            [1, 33, 3, 20, 5, 41, 12]
        );
        assert_eq!(
            seven
                .union(&PySet::of([2, 9, 64]))
                .iter()
                .collect::<Vec<_>>(),
            [64, 1, 33, 3, 2, 5, 41, 9, 12, 20]
        );
        assert_eq!(
            seven
                .intersection(&PySet::of([41, 3, 7, 12, 99]))
                .iter()
                .collect::<Vec<_>>(),
            [41, 3, 12]
        );
        let mut kept = PySet::of([1, 9, 17, 25]);
        kept.intersection_update(&PySet::of([25, 1, 7]));
        assert_eq!(kept.iter().collect::<Vec<_>>(), [25, 1]);
        let error = PySet::of([8, 16, 24]).remove(5).unwrap_err();
        assert!(error.is(Kind::KeyError) && error.message == "5");
    }
}

#[cfg(test)]
mod checks {
    use super::*;

    #[test]
    fn python_arithmetic() -> Result<()> {
        assert_eq!((floordiv(-7, 2), modulo(-7, 2)), (-4, 1));
        assert_eq!((floordiv(7, -2), modulo(7, -2)), (-4, -1));
        assert_eq!(
            (round(0.5)?, round(1.5)?, round(2.5)?, round(-0.5)?),
            (0, 2, 2, 0)
        );
        assert!(round(f64::NAN).unwrap_err().is(Kind::ValueError));
        assert!(int(f64::INFINITY).unwrap_err().is(Kind::OverflowError));
        assert_eq!((int(-2.7)?, floor(-2.5)?, ceil(-2.5)?), (-2, -3, -2));
        assert_eq!(round_to(2.675, 2), 2.67);
        assert_eq!(modulo_f(-1.0, 360.0), 359.0);
        assert_eq!(
            (floordiv_f(1.0, 0.1), modulo_f(1.0, 0.1)),
            (9.0, 0.09999999999999995)
        );
        assert!(modulo_f(-0.0, 360.0).is_sign_positive() && modulo_f(5.0, -5.0).is_sign_negative());
        assert_eq!((floordiv_f(-7.5, 2.0), modulo_f(-7.5, 2.0)), (-4.0, 0.5));
        Ok(())
    }

    #[test]
    fn python_text() {
        assert_eq!(float_repr(1.0), "1.0");
        assert_eq!(float_repr(0.1), "0.1");
        assert_eq!(float_repr(1e-5), "1e-05");
        assert_eq!(float_repr(1e16), "1e+16");
        assert_eq!(float_repr(-2.5), "-2.5");
        assert_eq!(
            float_repr(f64::from_bits(0x4062911c40000000)),
            "148.53469848632812"
        );
        assert_eq!(float_repr(1.5e16), "1.5e+16");
        assert_eq!(float_repr(-0.0), "-0.0");
        assert_eq!(float_repr(123.0), "123.0");
        assert_eq!(repr("it's"), "\"it's\"");
        assert_eq!(repr("a\nb\u{e08e}"), "'a\\nb\\ue08e'");
        assert_eq!(split(" a\u{a0}b\x1cc "), vec!["a", "b", "c"]);
        assert_eq!(slice(b"abc", 1, 10), b"bc");
        assert_eq!(slice(b"abc", -2, 3), b"bc");
        assert_eq!(
            decode_utf16le(&[0x41, 0, 0x00, 0xd8, 0x42]),
            "A\u{fffd}\u{fffd}"
        );
    }

    #[test]
    fn exception_classes() {
        let lost = Error::new(Kind::RpcLostError, "gone");
        assert!(lost.is(Kind::RpcTimeoutError) && lost.is(Kind::RpcError));
        assert!(lost.is(Kind::Exception) && !lost.is(Kind::OsError));
        assert!(lost.fault().starts_with("RpcLostError at py.rs:"));
        assert_eq!(format!("{lost:?}"), "RpcLostError('gone')");
    }

    #[test]
    fn encode_ascii_raises_what_cpython_raises() {
        assert_eq!(encode_ascii("a\0~").unwrap(), b"a\0~");
        let one = encode_ascii("ab\u{e9}").unwrap_err();
        assert!(one.is(Kind::UnicodeEncodeError) && one.is(Kind::ValueError));
        assert_eq!(
            one.message,
            "'ascii' codec can't encode character '\\xe9' in position 2: ordinal not in range(128)"
        );
        assert_eq!(
            encode_ascii("\u{3042}\u{1f600}a").unwrap_err().message,
            "'ascii' codec can't encode characters in position 0-1: ordinal not in range(128)"
        );
    }
}
