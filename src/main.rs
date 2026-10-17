/// arg() に渡される値。s() / d() / u() などのどれから呼ばれたかを表す
enum Value<'a> {
    Str(&'a str),
    Int(i64),
    Uint(u64),
    Float(f64),
}

/// `%[フラグ][幅]変換文字` の解析結果
#[derive(Default)]
struct Spec {
    left: bool,  // '-' 左寄せ
    plus: bool,  // '+' 非負数にも '+' を付ける
    space: bool, // ' ' 非負数の前に空白を付ける
    zero: bool,  // '0' 零埋め
    width: usize,
    prec: Option<usize>, // '.' 精度(%f のみ意味を持つ。省略時は 6)
    long: bool, // 'l' 64 ビット整数として扱う(符号なし変換のみ意味を持つ)
    conv: char,
}

/// '%' の直後の文字列を解析し、(Spec, 変換指定の後ろの残り) を返す。
/// 変換文字(小文字)が見つからない場合は None
fn parse_spec(s: &str) -> Option<(Spec, &str)> {
    let mut sp = Spec::default();
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        match c {
            '-' => sp.left = true,
            '+' => sp.plus = true,
            ' ' => sp.space = true,
            '0' => sp.zero = true,
            _ => break,
        }
        rest = &rest[1..];
    }
    let n = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    sp.width = rest[..n].parse().unwrap_or(0);
    rest = &rest[n..];

    // 精度 '.' に続く数字(数字がなければ 0)
    if let Some(r) = rest.strip_prefix('.') {
        let n = r.find(|c: char| !c.is_ascii_digit()).unwrap_or(r.len());
        sp.prec = Some(r[..n].parse().unwrap_or(0));
        rest = &r[n..];
    }

    // 長さ修飾子 'l'(複数あっても 1 つとして扱う)
    let after = rest.trim_start_matches('l');
    sp.long = after.len() != rest.len();
    rest = after;

    let pos = rest.find(|c: char| c.is_ascii_alphabetic())?;
    sp.conv = rest.as_bytes()[pos] as char;
    Some((sp, &rest[pos + 1..]))
}

/// %d の符号部分('-' / '+' / ' ' / なし)
fn sign_of(negative: bool, sp: &Spec) -> &'static str {
    if negative {
        "-"
    } else if sp.plus {
        "+"
    } else if sp.space {
        " "
    } else {
        ""
    }
}

/// 符号なし変換(%u %o %x %X)の本体。
/// C と同様に、l なしは u32、l 付きは u64 として解釈する
fn unsigned_body(bits: u64, sp: &Spec) -> String {
    let u = if sp.long { bits } else { bits as u32 as u64 };
    match sp.conv {
        'u' => u.to_string(),
        'o' => format!("{u:o}"),
        'x' => format!("{u:x}"),
        _ => format!("{u:X}"),
    }
}

/// 値を指定に従って整形する。意味を持たないフラグは無視する
fn render(sp: &Spec, v: Value) -> String {
    // 符号と零埋めが有効なのは「%d / %u / %o / %x / %X に整数」の場合だけ
    // (符号なし変換では + と空白は無視)
    let (sign, body, numeric) = match (v, sp.conv) {
        (Value::Int(n), 'd') => (sign_of(n < 0, sp), n.unsigned_abs().to_string(), true),
        (Value::Uint(u), 'd') => (sign_of(false, sp), u.to_string(), true),
        (Value::Int(n), 'u' | 'o' | 'x' | 'X') => ("", unsigned_body(n as u64, sp), true),
        (Value::Uint(u), 'u' | 'o' | 'x' | 'X') => ("", unsigned_body(u, sp), true),
        (Value::Float(x), 'f') => {
            let sign = sign_of(x.is_sign_negative() && !x.is_nan(), sp);
            if x.is_finite() {
                (sign, format!("{:.*}", sp.prec.unwrap_or(6), x.abs()), true)
            } else {
                // inf / nan には零埋めを適用しない
                (sign, if x.is_nan() { "nan" } else { "inf" }.to_string(), false)
            }
        }
        (Value::Float(x), _) => ("", x.to_string(), false),
        (Value::Int(n), _) => ("", n.to_string(), false),
        (Value::Uint(u), _) => ("", u.to_string(), false),
        (Value::Str(s), _) => ("", s.to_string(), false),
    };
    let len = sign.len() + body.chars().count();
    let pad = sp.width.saturating_sub(len);
    if sp.left {
        format!("{sign}{body}{}", " ".repeat(pad))
    } else if numeric && sp.zero {
        format!("{sign}{}{body}", "0".repeat(pad))
    } else {
        format!("{}{sign}{body}", " ".repeat(pad))
    }
}

struct Print<'a> {
    rest: &'a str, // 未処理のフォーマット文字列
    ret: String,
}

impl<'a> Print<'a> {
    /// 次の '%' の手前までをそのまま出力する
    fn advance(&mut self) {
        let end = self.rest.find('%').unwrap_or(self.rest.len());
        let (lit, rest) = self.rest.split_at(end);
        self.ret.push_str(lit);
        self.rest = rest;
    }
    fn fmt(s: &str) -> Print<'_> {
        let mut p = Print { rest: s, ret: String::with_capacity(256) };
        p.advance();
        p
    }
    fn out(&mut self) {
        print!("{}", self.ret);
    }
    fn err(&mut self) {
        eprint!("{}", self.ret);
    }
    fn str(&mut self) -> String {
        self.ret.clone()
    }
    fn arg(&mut self, v: Value) -> &mut Print<'a> {
        if let Some(spec) = self.rest.strip_prefix('%') {
            match parse_spec(spec) {
                Some((sp, rest)) => {
                    self.ret.push_str(&render(&sp, v));
                    self.rest = rest;
                }
                None => self.rest = "",
            }
        }
        self.advance();
        self
    }
    fn s(&mut self, v: &str) -> &mut Print<'a> {
        self.arg(Value::Str(v))
    }
    fn d(&mut self, v: i32) -> &mut Print<'a> {
        self.arg(Value::Int(v as i64))
    }
    fn ld(&mut self, v: i64) -> &mut Print<'a> {
        self.arg(Value::Int(v))
    }
    fn f(&mut self, v: f64) -> &mut Print<'a> {
        self.arg(Value::Float(v))
    }
    fn u(&mut self, v: u32) -> &mut Print<'a> {
        self.arg(Value::Uint(v as u64))
    }
    fn lu(&mut self, v: u64) -> &mut Print<'a> {
        self.arg(Value::Uint(v))
    }
}

fn format(fmt: &str) -> Print<'_> {
    Print::fmt(fmt)
}

fn main() {
    format("Copyright (C) %d %s. All rights reserved.\n").d(2026).s("S.Fuchita").out();
    format("[%5d][%-5d][%05d][%+d][% d][%+05d]\n")
        .d(42).d(42).d(-42).d(42).d(42).d(42).out();
    format("[%8s][%-8s][%08s][%+s]\n")
        .s("abc").s("abc").s("abc").s("abc").out();
    format("[%u][%o][%x][%X][%08x][%-6X][%+u][%u]\n")
        .d(42).d(42).d(255).d(255).d(255).d(255).d(42).d(-1).out();
    format("[%x][%5o]\n").s("abc").s("xyz").out();
    format("[%ld][%lu][%lx][%lX][%020lu][%-22ld][%lx][%x][%ls]\n")
        .ld(i64::MIN).ld(-1).ld(-1).ld(0x1234_5678_9abc).ld(-1).ld(i64::MAX).d(-1).ld(-1).s("abc").out();
    format("[%u][%lu][%x][%lx][%lx][%x][%d][%+d][%08u][%-6u|][%+u][%s]\n")
        .u(u32::MAX).lu(u64::MAX).u(0xdead_beef).lu(0xdead_beef_cafe).lu(u64::MAX)
        .lu(u64::MAX).u(u32::MAX).u(7).u(42).u(42).u(42).lu(u64::MAX).out();
    format("[%f][%.2f][%.0f][%8.3f][%-8.1f|][%08.2f][%+.1f][% .1f][%+08.2f]\n")
        .f(3.14159265).f(3.14159265).f(2.5).f(-3.14159265).f(2.25).f(-3.14159).f(1.0).f(1.0).f(1.5).out();
    format("[%f][%08f][%f][%5.1f][%.2f]\n")
        .f(f64::INFINITY).f(f64::NEG_INFINITY).f(f64::NAN).f(-0.0).f(1e10).out();
    format("[%f][%f][%d][%s][%08.2f]\n")
        .d(42).s("abc").f(1.5).f(2.5).s("xy").out();
    format("error: code=%d (%s)\n").d(2).s("stderr").err();
    let s = format("%s-%05d").s("id").d(42).str();
    println!("[{s}] len={}", s.len());
    // 不一致: %s に整数、%d に文字列(無意味なフラグは無視)
    format("[%+5s][%05s][%-6d][%+d]\n")
        .d(42).d(-7).s("abc").s("xyz").out();
}
