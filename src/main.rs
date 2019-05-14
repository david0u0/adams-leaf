struct Flow {
    period: u32,
    size: u32,
    index: u32
}

enum AVBType { A, B }
struct AVBFlow {
    period: u32,
    size: u32,
    avb_type: AVBType
}
impl AVBFlow {
}
struct TTFlow {
    period: u32,
    size: u32,
}
impl TTFlow {
}

enum FlowEnum {
    AVB {
        period: i32
    },
    TT(TTFlow),
}

fn main() {
    let s;
    {
        let gg = String::from("ggg");
        let str_gg = &gg[..];
        println!("{}", str_gg);
        //s = test(str_gg);
        s = &gg[..];
        println!("{}", s);
    }
    println!("{}", s);
}

fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        return s1;
    } else {
        return s2;
    }
}

fn test<'a>(s: &'a str) -> &'a str {
    return "this is a t";
}