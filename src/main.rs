
struct Print<'a> {
    fmt: &'a[u8],
    last: usize,
    curr: usize,
    ret: String,
}

impl<'a> Print<'a> {
    fn advance(&mut self) {
        let mut i = self.curr;
        loop {
            let mut c = if i < self.fmt.len() { self.fmt[i] } else { 0 as u8 };
            if c == '%' as u8 || c == 0 {
                break;
            } else {
                i += 1
            }
        }
        self.ret.push_str(&String::from_utf8_lossy(&self.fmt[self.curr..i]));
        self.curr = i;
    }
    fn fmt(s: &str)-> Print {
        let mut ret = Print{fmt: s.as_bytes(), last: 0, curr: 0, ret: String::with_capacity(256)};
        ret.advance();
        return ret;
    }
    fn out(&mut self) {
        print!("{}", self.ret);
    }
    fn arg(&mut self, v: &str)-> &mut Print<'a> {
        self.advance();
        let fmt = self.fmt;
        let mut i = self.curr;
        let mut c = if i < fmt.len() { fmt[i] } else { 0 as u8 };
        if c == '%' as u8  || c == 0 {
            loop {
                i += 1;
                if i < fmt.len() {
                    c = fmt[i];
                    if c >= 'a' as u8 && c <= 'z' as u8 {
                        i += 1;
                        self.ret.push_str(v);
                        self.last = i;
                        break;
                    }
                } else {
                    break;
                }
            }
        } else {
            i += 1;
        }
        self.curr = i;
        self.advance();
        return self;
    }
    fn s(&mut self, v: &str)-> &mut Print<'a> {
        return self.arg(v);
    }
    fn d(&mut self, v: i32)-> &mut Print<'a> {
        return self.arg(&v.to_string());
    }
}

fn format(fmt: &str)-> Print {
    return Print::fmt(fmt);
}

fn main() {
    format("Copyright (C) %d %s. All rights reserved.\n").d(2026).s("S.Fuchita").out();
}
