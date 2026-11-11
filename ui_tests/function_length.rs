fn long_function() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    let k = 11;
    let l = 12;
    let m = 13;
    let n = 14;
    let o = 15;
    let p = 16;
    let q = 17;
    let r = 18;
    let s = 19;
    let t = 20;
    let u = 21;
    let v = 22;
    let w = 23;
    let x = 24;
    let y = 25;
    let z = 26;
    let aa = 27;
    let bb = 28;
    let cc = 29;
    let dd = 30;
    let ee = 31;
    let ff = 32;
    let gg = 33;
    let hh = 34;
    let ii = 35;
    let jj = 36;
    let kk = 37;
    let ll = 38;
    let mm = 39;
    let nn = 40;
    let oo = 41;
    let pp = 42;
    let qq = 43;
    let rr = 44;
    let ss = 45;
    let tt = 46;
    let uu = 47;
    let vv = 48;
    let ww = 49;
    let xx = 50;
    let yy = 51;
    let zz = 52;
    let aaa = 53;
    let bbb = 54;
    let ccc = 55;
    let ddd = 56;
    let eee = 57;
    let fff = 58;
    let ggg = 59;
    let hhh = 60;
    let iii = 61;
    let jjj = 62;
    let kkk = 63;
    let lll = 64;
    let mmm = 65;
    let nnn = 66;
    let ooo = 67;
    let ppp = 68;
    let qqq = 69;
    let rrr = 70;
    let sss = 71;
    71
}

// Should NOT warn: 70 body lines, at the limit; the braces are not body lines
fn at_limit_function() -> u32 {
    let v1 = 1;
    let v2 = 2;
    let v3 = 3;
    let v4 = 4;
    let v5 = 5;
    let v6 = 6;
    let v7 = 7;
    let v8 = 8;
    let v9 = 9;
    let v10 = 10;
    let v11 = 11;
    let v12 = 12;
    let v13 = 13;
    let v14 = 14;
    let v15 = 15;
    let v16 = 16;
    let v17 = 17;
    let v18 = 18;
    let v19 = 19;
    let v20 = 20;
    let v21 = 21;
    let v22 = 22;
    let v23 = 23;
    let v24 = 24;
    let v25 = 25;
    let v26 = 26;
    let v27 = 27;
    let v28 = 28;
    let v29 = 29;
    let v30 = 30;
    let v31 = 31;
    let v32 = 32;
    let v33 = 33;
    let v34 = 34;
    let v35 = 35;
    let v36 = 36;
    let v37 = 37;
    let v38 = 38;
    let v39 = 39;
    let v40 = 40;
    let v41 = 41;
    let v42 = 42;
    let v43 = 43;
    let v44 = 44;
    let v45 = 45;
    let v46 = 46;
    let v47 = 47;
    let v48 = 48;
    let v49 = 49;
    let v50 = 50;
    let v51 = 51;
    let v52 = 52;
    let v53 = 53;
    let v54 = 54;
    let v55 = 55;
    let v56 = 56;
    let v57 = 57;
    let v58 = 58;
    let v59 = 59;
    let v60 = 60;
    let v61 = 61;
    let v62 = 62;
    let v63 = 63;
    let v64 = 64;
    let v65 = 65;
    let v66 = 66;
    let v67 = 67;
    let v68 = 68;
    let v69 = 69;
    v69
}

fn short_function() -> u32 {
    let a = 1;
    let b = 2;
    a + b
}

fn main() {
    long_function();
    short_function();
    at_limit_function();
}

#[test]
fn long_test_function() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    let k = 11;
    let l = 12;
    let m = 13;
    let n = 14;
    let o = 15;
    let p = 16;
    let q = 17;
    let r = 18;
    let s = 19;
    let t = 20;
    let u = 21;
    let v = 22;
    let w = 23;
    let x = 24;
    let y = 25;
    let z = 26;
    let aa = 27;
    let bb = 28;
    let cc = 29;
    let dd = 30;
    let ee = 31;
    let ff = 32;
    let gg = 33;
    let hh = 34;
    let ii = 35;
    let jj = 36;
    let kk = 37;
    let ll = 38;
    let mm = 39;
    let nn = 40;
    let oo = 41;
    let pp = 42;
    let qq = 43;
    let rr = 44;
    let ss = 45;
    let tt = 46;
    let uu = 47;
    let vv = 48;
    let ww = 49;
    let xx = 50;
    let yy = 51;
    let zz = 52;
    let aaa = 53;
    let bbb = 54;
    let ccc = 55;
    let ddd = 56;
    let eee = 57;
    let fff = 58;
    let ggg = 59;
    let hhh = 60;
    let iii = 61;
    let jjj = 62;
    let kkk = 63;
    let lll = 64;
    let mmm = 65;
    let nnn = 66;
    let ooo = 67;
    let ppp = 68;
    let qqq = 69;
    let rrr = 70;
    let sss = 71;
    71
}

struct Widget;

impl Widget {
    // Should warn: inherent methods follow the same limit as free functions
    fn long_method(&self) -> u32 {
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
        let h = 8;
        let i = 9;
        let j = 10;
        let k = 11;
        let l = 12;
        let m = 13;
        let n = 14;
        let o = 15;
        let p = 16;
        let q = 17;
        let r = 18;
        let s = 19;
        let t = 20;
        let u = 21;
        let v = 22;
        let w = 23;
        let x = 24;
        let y = 25;
        let z = 26;
        let aa = 27;
        let bb = 28;
        let cc = 29;
        let dd = 30;
        let ee = 31;
        let ff = 32;
        let gg = 33;
        let hh = 34;
        let ii = 35;
        let jj = 36;
        let kk = 37;
        let ll = 38;
        let mm = 39;
        let nn = 40;
        let oo = 41;
        let pp = 42;
        let qq = 43;
        let rr = 44;
        let ss = 45;
        let tt = 46;
        let uu = 47;
        let vv = 48;
        let ww = 49;
        let xx = 50;
        let yy = 51;
        let zz = 52;
        let aaa = 53;
        let bbb = 54;
        let ccc = 55;
        let ddd = 56;
        let eee = 57;
        let fff = 58;
        let ggg = 59;
        let hhh = 60;
        let iii = 61;
        let jjj = 62;
        let kkk = 63;
        let lll = 64;
        let mmm = 65;
        let nnn = 66;
        let ooo = 67;
        let ppp = 68;
        let qqq = 69;
        let rrr = 70;
        let sss = 71;
        71
    }

    fn short_method(&self) -> u32 {
        1
    }
}

trait LongDefault {
    // Should NOT warn: required trait methods have no body
    fn required_method(&self) -> u32;

    // Should warn: provided trait methods have a body
    fn long_default_method(&self) -> u32 {
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
        let h = 8;
        let i = 9;
        let j = 10;
        let k = 11;
        let l = 12;
        let m = 13;
        let n = 14;
        let o = 15;
        let p = 16;
        let q = 17;
        let r = 18;
        let s = 19;
        let t = 20;
        let u = 21;
        let v = 22;
        let w = 23;
        let x = 24;
        let y = 25;
        let z = 26;
        let aa = 27;
        let bb = 28;
        let cc = 29;
        let dd = 30;
        let ee = 31;
        let ff = 32;
        let gg = 33;
        let hh = 34;
        let ii = 35;
        let jj = 36;
        let kk = 37;
        let ll = 38;
        let mm = 39;
        let nn = 40;
        let oo = 41;
        let pp = 42;
        let qq = 43;
        let rr = 44;
        let ss = 45;
        let tt = 46;
        let uu = 47;
        let vv = 48;
        let ww = 49;
        let xx = 50;
        let yy = 51;
        let zz = 52;
        let aaa = 53;
        let bbb = 54;
        let ccc = 55;
        let ddd = 56;
        let eee = 57;
        let fff = 58;
        let ggg = 59;
        let hhh = 60;
        let iii = 61;
        let jjj = 62;
        let kkk = 63;
        let lll = 64;
        let mmm = 65;
        let nnn = 66;
        let ooo = 67;
        let ppp = 68;
        let qqq = 69;
        let rrr = 70;
        let sss = 71;
        71
    }
}
