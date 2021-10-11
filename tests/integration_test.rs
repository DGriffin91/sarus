#![feature(core_intrinsics)]

use serde::Deserialize;
use std::{
    alloc::{alloc, dealloc, Layout},
    collections::HashMap,
    f64::consts::*,
    ffi::CStr,
    mem,
};

use sarus::*;

#[test]
fn parentheses() -> anyhow::Result<()> {
    let code = r#"
fn main(a, b) -> (c) {
    c = a * (a - b) * (a * (2.0 + b))
}
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(a * (a - b) * (a * (2.0 + b)), func(a, b));
    Ok(())
}

#[test]
fn libc_math() -> anyhow::Result<()> {
    let code = r#"
fn main(a, b) -> (c) {
    c = b
    c = sin(c)
    c = cos(c)
    c = tan(c)
    c = asin(c)
    c = acos(c)
    c = atan(c)
    c = exp(c)
    c = log(c)
    c = log10(c)
    c = sqrt(c + 10.0)
    c = sinh(c)
    c = cosh(c)
    c = tanh(c * 0.00001)
    c = atan2(c, a)
    c = pow(c, a * 0.001)
    c *= nums()
}
fn nums() -> (r) {
    r = E + FRAC_1_PI + FRAC_1_SQRT_2 + FRAC_2_SQRT_PI + FRAC_PI_2 + FRAC_PI_3 + FRAC_PI_4 + FRAC_PI_6 + FRAC_PI_8 + LN_2 + LN_10 + LOG2_10 + LOG2_E + LOG10_2 + LOG10_E + PI + SQRT_2 + TAU
}
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut c = b;
    c = c.sin();
    c = c.cos();
    c = c.tan();
    c = c.asin();
    c = c.acos();
    c = c.atan();
    c = c.exp();
    c = c.log(E);
    c = c.log10();
    c = (c + 10.0).sqrt();
    c = c.sinh();
    c = c.cosh();
    c = (c * 0.00001).tanh();
    c = c.atan2(a);
    c = c.powf(a * 0.001);
    c *= E
        + FRAC_1_PI
        + FRAC_1_SQRT_2
        + FRAC_2_SQRT_PI
        + FRAC_PI_2
        + FRAC_PI_3
        + FRAC_PI_4
        + FRAC_PI_6
        + FRAC_PI_8
        + LN_2
        + LN_10
        + LOG2_10
        + LOG2_E
        + LOG10_2
        + LOG10_E
        + PI
        + SQRT_2
        + TAU;

    let epsilon = 0.00000000000001;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    let result = func(a, b);
    assert!(result >= c - epsilon && result <= c + epsilon);
    Ok(())
}

#[test]
fn rust_math() -> anyhow::Result<()> {
    let code = r#"
fn main(a, b) -> (c) {
    c = b
    c = c.sin()
    c = c.cos()
    c = c.tan()
    c = c.asin()
    c = c.acos()
    c = c.atan()
    c = c.exp()
    c = c.log(E)
    c = c.log10()
    c = (c + 10.0).sqrt()
    c = c.sinh()
    c = c.cosh()
    c = (c * 0.00001).tanh()
    c = c.atan2(a)
    c = c.powf(a * 0.001)
    c *= nums()
}
fn nums() -> (r) {
    r = E + FRAC_1_PI + FRAC_1_SQRT_2 + FRAC_2_SQRT_PI + FRAC_PI_2 + FRAC_PI_3 + FRAC_PI_4 + FRAC_PI_6 + FRAC_PI_8 + LN_2 + LN_10 + LOG2_10 + LOG2_E + LOG10_2 + LOG10_E + PI + SQRT_2 + TAU
}
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut c = b;
    c = c.sin();
    c = c.cos();
    c = c.tan();
    c = c.asin();
    c = c.acos();
    c = c.atan();
    c = c.exp();
    c = c.log(E);
    c = c.log10();
    c = (c + 10.0).sqrt();
    c = c.sinh();
    c = c.cosh();
    c = (c * 0.00001).tanh();
    c = c.atan2(a);
    c = c.powf(a * 0.001);
    c *= E
        + FRAC_1_PI
        + FRAC_1_SQRT_2
        + FRAC_2_SQRT_PI
        + FRAC_PI_2
        + FRAC_PI_3
        + FRAC_PI_4
        + FRAC_PI_6
        + FRAC_PI_8
        + LN_2
        + LN_10
        + LOG2_10
        + LOG2_E
        + LOG10_2
        + LOG10_E
        + PI
        + SQRT_2
        + TAU;

    let epsilon = 0.00000000000001;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    let result = func(a, b);
    assert!(result >= c - epsilon && result <= c + epsilon);
    Ok(())
}

#[test]
fn rounding() -> anyhow::Result<()> {
    let code = r#"
fn main(a, b) -> (c) {
    f = (1.5).floor()
    c = a.ceil() * b.floor() * a.trunc() * (a * b * -1.234).fract() * (1.5).round()
}
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(
        a.ceil() * b.floor() * a.trunc() * (a * b * -1.234).fract() * 1.5f64.round(),
        func(a, b)
    );
    Ok(())
}

#[test]
fn minmax() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (c) {
        c = a.min(b)
    }
    "#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(a, func(a, b));

    let code = r#"
    fn main(a, b) -> (c) {
        c = a.max(b)
    }
    "#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(b, func(a, b));

    Ok(())
}

#[test]
fn comments() -> anyhow::Result<()> {
    let code = r#"
//test
fn main(a, b) -> (c) {//test
//test
    //test
    d = foodd(a, b) + foodd(a, b) //test
//test


//test
    c = d + 1.0 //test
//test//test
}//test

//test
//test

fn maina(a, b) -> (c) {//test
    c = foodd(a, b) + 2.12312 + 1.1//test
    c = c + 10.0//test
}//test
//test
fn foodd(a, b) -> (c) {
    c = a + b//test
}//test

//fn foodd(a, b) -> (c) {
//    c = a + b//test
//}//test
    
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(601.0, func(a, b));
    Ok(())
}

#[test]
fn multiple_returns() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        c, d = stuff(a, b)
        c, d = d, c
        e, f = if a == b {
            stuff(b, a)
        } else {
            stuff(a, b)
        }
        if 1.0 == 1.0 {
            e = e * 100.0
        }
        e *= 2.0
        e /= 3.0
        e -= 1.0
        i = 0.0
        while i < 10.0 {
            e = e * 2.0
            i += 1.0
        }
    }
    
    fn stuff(a, b) -> (c, d) {
        c = a + 1.0
        d = c + b + 10.0
    }
    
    fn stuff2(a) -> (c) {
        c = a + 1.0
    }
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(6893909.333333333, func(a, b));
    Ok(())
}

#[test]
fn bools() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (c) {
        c = if true {
            a * b
        } else {
            0.0
        }
        if false {
            c = 999999999.0
        }
    }
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(20000.0, func(a, b));
    Ok(())
}

#[test]
fn ifelse_assign() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (c) {
        c = if a < b {
            a * b
        } else {
            0.0
        }
    }
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(20000.0, func(a, b));
    Ok(())
}

#[test]
fn order() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (c) {
        c = a
    }
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(100.0, func(a, b));
    Ok(())
}

#[test]
fn array_read_write() -> anyhow::Result<()> {
    let code = r#"
fn main(arr: &[f64], b) -> () {
    arr[0] = arr[0] * b
    arr[1] = arr[1] * b
    arr[2] = arr[2] * b
    arr[3] = arr[3] * b
}
"#;

    let mut arr = [1.0, 2.0, 3.0, 4.0];
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(*mut f64, f64)>(func_ptr) };
    func(arr.as_mut_ptr(), b);
    assert_eq!([200.0, 400.0, 600.0, 800.0], arr);
    Ok(())
}

#[test]
fn negative() -> anyhow::Result<()> {
    let code = r#"
    fn main(a) -> (c) {
        c = -1.0 + a
    }
"#;
    let a = -100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    assert_eq!(-101.0, func(a));
    Ok(())
}

//TODO the frontend doesn't have the syntax support for this yet
//#[test]
//fn if_return_types() -> anyhow::Result<()> {
//    let code = r#"
//    fn main(a) -> (c) {
//        f, g, h = if a < 0.0 {
//            1.0, 1, true
//        } else {
//            0.0, 0, false
//        }
//        f.assert_eq(1.0)
//        g.assert_eq(1)
//        h.assert_eq(true)
//    }
//"#;
//    let a = -100.0f64;
//    let mut jit = default_std_jit_from_code(&code)?;
//    let func_ptr = jit.get_func("main")?;
//    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
//    assert_eq!(-101.0, func(a));
//    Ok(())
//}

#[test]
fn compiled_graph() -> anyhow::Result<()> {
    let code = r#"
    fn add_node (a, b) -> (c) {
        c = a + b
    }
        
    fn sub_node (a, b) -> (c) {
        c = a - b
    }
        
    fn sin_node (a) -> (c) {
        c = sin(a)
    }
        
    fn graph (audio: &[f64]) -> () {
        i = 0
        while i <= 7 {
            vINPUT_0 = audio[i]
            vadd1_0 = add_node(vINPUT_0, 2.0000000000)
            vsin1_0 = sin_node(vadd1_0)
            vadd2_0 = add_node(vsin1_0, 4.0000000000)
            vsub1_0 = sub_node(vadd2_0, vadd1_0)
            vOUTPUT_0 = vsub1_0
            audio[i] = vOUTPUT_0
            i += 1
        }
    }
"#;

    let mut audio = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("graph")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [f64; 8])>(func_ptr) };
    dbg!(func(&mut audio));
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Metadata {
    description: Option<String>,
    inputs: HashMap<String, MetadataInput>,
}

#[derive(Deserialize, Debug)]
struct MetadataInput {
    default: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    description: Option<String>,
    label: Option<String>,
    unit: Option<String>,
    gradient: Option<String>,
}

#[test]
fn metadata() -> anyhow::Result<()> {
    let code = r#"    
    
    @ add_node node
        description = "add two numbers!"

        [inputs]
        a = {default = 0.0, description = "1st number"}
        b = {default = 0.0, description = "2nd number"}
    @
    fn add_node (a, b) -> (c) {
        c = a + b
    }
        
    fn sub_node (a, b) -> (c) {
        c = a - b
    }
        
    fn sin_node (a) -> (c) {
        c = sin(a)
    }
        
    fn graph (audio: &[f64]) -> () {
        i = 0
        while i <= 7 {
            vINPUT_0 = audio[i]
            vadd1_0 = add_node(vINPUT_0, 2.0000000000)
            vsin1_0 = sin_node(vadd1_0)
            vadd2_0 = add_node(vsin1_0, 4.0000000000)
            vsub1_0 = sub_node(vadd2_0, vadd1_0)
            vOUTPUT_0 = vsub1_0
            audio[i] = vOUTPUT_0
            i += 1
        }
    }
"#;
    let ast = parser::program(&code)?;
    let mut jit = default_std_jit_from_code(&code)?;

    let func_meta: Option<Metadata> = ast.iter().find_map(|d| match d {
        frontend::Declaration::Metadata(head, body) => {
            if let Some(head) = head.first() {
                if head == "add_node" {
                    Some(toml::from_str(&body).unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    });

    dbg!(&func_meta);

    let mut audio = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let func_ptr = jit.get_func("graph")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [f64; 8])>(func_ptr) };
    dbg!(func(&mut audio));
    //assert_eq!([200.0, 400.0, 600.0, 800.0], arr);
    Ok(())
}

#[test]
fn int_while_loop() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        e = 2.0
        i = 0
        while i < 10 {
            e = e * 2.0
            i += 1
        }
    }
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(2048.0, func(a, b));
    Ok(())
}

#[test]
fn int_to_float() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        i = 2
        e = i.f64() * a * b * (2).f64() * 2.0 * (2).f64()
    }
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(320000.0, func(a, b));
    Ok(())
}

#[test]
fn float_conversion() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        i_a = a.i64()
        e = if i_a < b.i64() {
            i_a.f64().i64().f64() //TODO chaining not working
        } else {
            2.0
        }
    }
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(100.0, func(a, b));
    Ok(())
}

#[test]
fn float_as_bool_error() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        i_a = a
        e_i = if true {
            1
        } else {
            2
        }
        e = e_i.f64()
    }
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(1.0, func(a, b));
    Ok(())
}

#[test]
fn array_return_from_if() -> anyhow::Result<()> {
    let code = r#"
fn main(arr1: &[f64], arr2: &[f64], b) -> () {
    arr3 = if b < 100.0 {
        arr1
    } else {
        arr2
    }
    arr3[0] = arr3[0] * 20.0
}
"#;

    let mut arr1 = [1.0, 2.0, 3.0, 4.0];
    let mut arr2 = [10.0, 20.0, 30.0, 40.0];
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(*mut f64, *mut f64, f64)>(func_ptr) };
    func(arr1.as_mut_ptr(), arr2.as_mut_ptr(), b);
    assert_eq!(200.0, arr2[0]);
    Ok(())
}

#[test]
fn var_type_consistency() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        n = 1
        n1 = n
        n2 = n1
        n3 = n2
        e = n3.f64()
    }
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(1.0, func(a, b));
    Ok(())
}

#[test]
fn three_inputs() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b, c) -> (e) {
        e = a + b + c
    }
"#;

    let a = 100.0f64;
    let b = 200.0f64;
    let c = 300.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64, f64) -> f64>(func_ptr) };
    assert_eq!(600.0, func(a, b, c));
    Ok(())
}

#[test]
fn manual_types() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f64, b: f64) -> (c: f64) {
    c = a * (a - b) * (a * (2.0 + b))
}
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(a * (a - b) * (a * (2.0 + b)), func(a, b));
    Ok(())
}

#[test]
fn i64_params() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f64, b: i64) -> (c: i64) {
    e = a * (a - b.f64()) * (a * (2.0 + b.f64()))
    c = e.i64()
}
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, i64) -> i64>(func_ptr) };
    assert_eq!((a * (a - b) * (a * (2.0 + b))) as i64, func(a, b as i64));
    Ok(())
}

#[test]
fn i64_params_multifunc() -> anyhow::Result<()> {
    //Not currently working, see BLOCKERs in jit.rs
    let code = r#"
fn main(a: f64, b: i64) -> (c: i64) {
    c = foo(a, b, 2)
}
fn foo(a: f64, b: i64, c: i64) -> (d: i64) {
    d = a.i64() + b + c
}
"#;
    let a = 100.0f64;
    let b = 200.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, i64) -> i64>(func_ptr) };
    assert_eq!(302, func(a, b as i64));
    Ok(())
}

#[test]
fn bool_params() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f64, b: bool) -> (c: f64) {
    c = if b {
        a
    } else {
        0.0-a
    }
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, bool) -> f64>(func_ptr) };
    assert_eq!(a, func(a, true));
    assert_eq!(-a, func(a, false));
    Ok(())
}

#[test]
fn logical_operators() -> anyhow::Result<()> {
    let code = r#"
fn and(a: bool, b: bool) -> (c: bool) {
    c = a && b
}
fn or(a: bool, b: bool) -> (c: bool) {
    c = a || b
}
fn gt(a: bool, b: bool) -> (c: bool) {
    c = a > b
}
fn ge(a: bool, b: bool) -> (c: bool) {
    c = a >= b
}
fn lt(a: bool, b: bool) -> (c: bool) {
    c = a < b
}
fn le(a: bool, b: bool) -> (c: bool) {
    c = a <= b
}
fn eq(a: bool, b: bool) -> (c: bool) {
    c = a == b
}
fn ne(a: bool, b: bool) -> (c: bool) {
    c = a != b
}
fn ifthen() -> (c: bool) {
    c = false
    if 1.0 < 2.0 && 2.0 < 3.0 {
        c = true
    }
}
fn ifthen2() -> (c: bool) {
    c = false
    if 1.0 < 2.0 || 2.0 < 1.0 {
        c = true
    }
}
fn ifthenparen() -> (c: bool) {
    c = false
    if (1.0 < 2.0) && (2.0 < 3.0) {
        c = true
    }
}
fn ifthennestedparen() -> (c: bool) {
    c = false
    if ((1.0 < 2.0) && (2.0 < 3.0) && true) {
        c = true
    }
}
fn parenassign() -> (c: bool) {
    c = ((1.0 < 2.0) && (2.0 < 3.0) && true)
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("and")?) };
    assert_eq!(true, f(true, true));
    assert_eq!(false, f(true, false));
    assert_eq!(false, f(false, true));
    assert_eq!(false, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("or")?) };
    assert_eq!(true, f(true, true));
    assert_eq!(true, f(true, false));
    assert_eq!(true, f(false, true));
    assert_eq!(false, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("gt")?) };
    assert_eq!(false, f(true, true));
    assert_eq!(true, f(true, false));
    assert_eq!(false, f(false, true));
    assert_eq!(false, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("ge")?) };
    assert_eq!(true, f(true, true));
    assert_eq!(true, f(true, false));
    assert_eq!(false, f(false, true));
    assert_eq!(true, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("lt")?) };
    assert_eq!(false, f(true, true));
    assert_eq!(false, f(true, false));
    assert_eq!(true, f(false, true));
    assert_eq!(false, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("le")?) };
    assert_eq!(true, f(true, true));
    assert_eq!(false, f(true, false));
    assert_eq!(true, f(false, true));
    assert_eq!(true, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("eq")?) };
    assert_eq!(true, f(true, true));
    assert_eq!(false, f(true, false));
    assert_eq!(false, f(false, true));
    assert_eq!(true, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool, bool) -> bool>(jit.get_func("ne")?) };
    assert_eq!(false, f(true, true));
    assert_eq!(true, f(true, false));
    assert_eq!(true, f(false, true));
    assert_eq!(false, f(false, false));
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthen")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthen2")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthenparen")?) };
    assert_eq!(true, f());
    let f =
        unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthennestedparen")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("parenassign")?) };
    assert_eq!(true, f());
    Ok(())
}

#[test]
fn unary_not() -> anyhow::Result<()> {
    let code = r#"
fn direct() -> (c: bool) {
    c = !true
}
fn direct2() -> (c: bool) {
    c = !false
}
fn direct3() -> (c: bool) {
    c = !(false)
}
fn not(a: bool) -> (c: bool) {
    c = !a
}
fn not2(a: bool) -> (c: bool) {
    c = !(a)
}
fn ifthen() -> (c: bool) {
    c = false
    if !(false) {
        c = true
    }
}
fn ifthen2() -> (c: bool) {
    c = false
    if !(!(false || !false)) {
        c = true
    }
}
fn ifthen3() -> (c: bool) {
    c = true
    if !(!(1.0 < 2.0) && !(2.0 < 3.0)) {
        c = false
    }
}
fn parenassign() -> (c: bool) {
    c = !((1.0 < 2.0) && (2.0 < 3.0) && true)
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("direct")?) };
    assert_eq!(false, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("direct2")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("direct3")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn(bool) -> bool>(jit.get_func("not2")?) };
    assert_eq!(false, f(true));
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthen")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthen2")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthen3")?) };
    assert_eq!(true, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("parenassign")?) };
    assert_eq!(false, f());
    Ok(())
}

extern "C" fn mult(a: f64, b: f64) -> f64 {
    a * b
}

extern "C" fn dbg(a: f64) {
    dbg!(a);
}

#[test]
fn extern_func() -> anyhow::Result<()> {
    let code = r#"
extern fn mult(a: f64, b: f64) -> (c: f64) {}
extern fn dbg(a: f64) -> () {}

fn main(a: f64, b: f64) -> (c: f64) {
    c = mult(a, b)
    dbg(a)
}
"#;
    let a = 100.0f64;
    let b = 100.0f64;
    let mut jit = default_std_jit_from_code_with_importer(&code, |_ast, jit_builder| {
        jit_builder.symbols([("mult", mult as *const u8), ("dbg", dbg as *const u8)]);
    })?;

    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    assert_eq!(mult(a, b), func(a, b));
    Ok(())
}

extern "C" fn prt2(s: *const i8) {
    unsafe {
        print!("{}", CStr::from_ptr(s).to_str().unwrap());
    }
}

#[test]
fn create_string() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f64, b: f64) -> (c: f64) {
    print("HELLO\n")
    print(["-"; 5])
    print("WORLD\n")
    c = a
}

extern fn print(s: &) -> () {}
"#;
    let a = 100.0f64;
    let b = 100.0f64;
    let mut jit = default_std_jit_from_code_with_importer(&code, |_ast, jit_builder| {
        jit_builder.symbols([("print", prt2 as *const u8)]);
    })?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
    func(a, b);

    Ok(())
}

#[test]
fn struct_access() -> anyhow::Result<()> {
    let code = r#"
struct Point {
    x: f64,
    y: f64,
    z: f64,
}
fn main(a: f64) -> (c: f64) {
    p = Point {
        x: a,
        y: 200.0,
        z: 300.0,
    }
    c = p.x + p.y + p.z
    (p.x).println()
    (p.y).println()
    (p.z).println()
    p.x.println()
    p.y.println()
    p.z.println()
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    //jit.print_clif(true);
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    dbg!(600.0, func(a));
    Ok(())
}

#[test]
fn struct_impl() -> anyhow::Result<()> {
    let code = r#"
struct Point {
    x: f64,
    y: f64,
    z: f64,
}
fn length(self: Point) -> (r: f64) {
    r = sqrt(pow(self.x, 2.0) + pow(self.y, 2.0) + pow(self.z, 2.0))
}
fn main(a: f64) -> (c: f64) {
    p = Point {
        x: a,
        y: 200.0,
        z: 300.0,
    }
    c = p.length()
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    assert_eq!(374.16573867739413, func(a));
    Ok(())
}

extern "C" fn dbgi(s: *const i8, a: i64) {
    unsafe {
        println!("{} {}", CStr::from_ptr(s).to_str().unwrap(), a);
    }
}
extern "C" fn dbgf(s: *const i8, a: f64) {
    unsafe {
        println!("{} {}", CStr::from_ptr(s).to_str().unwrap(), a);
    }
}

#[test]
fn struct_impl_nested() -> anyhow::Result<()> {
    let code = r#"
extern fn dbgf(s: &, a: f64) -> () {}
extern fn dbgi(s: &, a: i64) -> () {}
struct Line {
    a: Point,
    b: Point,
}

fn print(self: Line) -> () {
    "Line {".println()
    "a: ".print() self.a.print() ",".println()
    "b: ".print() self.b.print() ",".println()
    "}".println()
}

fn length(self: Line) -> (r: f64) {
    r = sqrt(pow(self.a.x - self.b.x, 2.0) + 
             pow(self.a.y - self.b.y, 2.0) + 
             pow(self.a.z - self.b.z, 2.0))
}

struct Point {
    x: f64,
    y: f64,
    z: f64,
}

fn print(self: Point) -> () {
    "Point {".println()
    "x: ".print() self.x.print() ",".println()
    "y: ".print() self.y.print() ",".println()
    "z: ".print() self.z.print() ",".println()
    "}".println()
}

fn length(self: Point) -> (r: f64) {
    r = sqrt(pow(self.x, 2.0) + pow(self.y, 2.0) + pow(self.z, 2.0))
}

fn main(n: f64) -> (c: f64) {
    p1 = Point {
        x: n,
        y: 200.0,
        z: 300.0,
    }
    p2 = Point {
        x: n * 4.0,
        y: 500.0,
        z: 600.0,
    }
    l1 = Line {
        a: p1,
        b: p2,
    }
    p1.x.print()

    p1.print()
    p2.print()
    l1.print()
    d = l1.a //struct is copied
    e = d.x + l1.a.x //f64's are copied
    d.print() //TODO This should work
    
    p1.y = e * d.z
    p1.y.assert_eq(e * d.z)

    c = l1.length()

    l1.a = l1.b
    l1.a.x.assert_eq(l1.b.x)
    l1.a.y.assert_eq(l1.b.y)
    l1.a.z.assert_eq(l1.b.z)

    l1.a.x, l1.a.y, l1.a.z, d = l1.a.x, l1.a.x, l1.a.x, l1.b
    l1.a.x.assert_eq(l1.b.x)
    l1.a.y.assert_eq(l1.b.x)
    l1.a.z.assert_eq(l1.b.x)
    
    d.x.assert_eq(l1.b.x)
    d.y.assert_eq(l1.b.y)
    d.z.assert_eq(l1.b.z)

    prev_x = l1.b.x
    prev_y = l1.b.y
    prev_z = l1.b.z
    prev_z2 = l1.a.z

    l1.b.x += 2.0
    l1.b.y -= 2.0
    l1.b.z *= 2.0
    l1.a.z /= 2.0

    l1.b.x.assert_eq(prev_x + 2.0)
    l1.b.y.assert_eq(prev_y - 2.0)
    l1.b.z.assert_eq(prev_z * 2.0)
    l1.a.z.assert_eq(prev_z2 / 2.0)

    //What about something like:?
    //a.l.f().e = a.b.c
    //I guess this isn't an issue while references can't be returned.
    
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code_with_importer(&code, |_ast, jit_builder| {
        jit_builder.symbols([("dbgf", dbgf as *const u8), ("dbgi", dbgi as *const u8)]);
    })?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(true);
    Ok(())
}

#[test]
fn struct_assign_op() -> anyhow::Result<()> {
    let code = r#"
struct Line {
    a: Point,
    b: Point,
}

struct Point {
    x: f64,
    y: f64,
    z: f64,
}

fn main(n: f64) -> (c: f64) {
    p1 = Point {
        x: n,
        y: 200.0,
        z: 300.0,
    }
    p2 = Point {
        x: n * 4.0,
        y: 500.0,
        z: 600.0,
    }
    l1 = Line {
        a: p1,
        b: p2,
    }


    l1.b.x = l1.b.x + 1.0
    l1.b.x += 1.0
    
}
"#;

    let ast = parser::program(&code)?;
    dbg!(&ast);
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(true);
    Ok(())
}

#[test]
fn struct_impl_nested_short() -> anyhow::Result<()> {
    let code = r#"
extern fn dbgf(s: &, a: f64) -> () {}
extern fn dbgi(s: &, a: i64) -> () {}
struct Foo {
    a: Bar,
    b: Bar,
}

fn print(self: Foo) -> () {
    "Foo {".println()
    "a: ".print() self.a.print() ",".println()
    "b: ".print() self.b.print() ",".println()
    "}".println()
}

struct Bar {
    x: f64,
}

fn print(self: Bar) -> () {
    "Bar {".println()
    "x: ".print() self.x.print() ",".println()
    "}".println()
}

fn main(n: f64) -> (c: f64) {
    pe = Bar {
        x: n,
    }
    pf = Bar {
        x: n * 4.0,
    }
    l1 = Foo {
        a: pe,
        b: pf,
    }
    pe.print()
    pf.print()
    "----------------".println()
    l1.print()
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code_with_importer(&code, |_ast, jit_builder| {
        jit_builder.symbols([("dbgf", dbgf as *const u8), ("dbgi", dbgi as *const u8)]);
    })?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(false);
    Ok(())
}
#[test]
fn type_impl() -> anyhow::Result<()> {
    let code = r#"
fn square(self: f64) -> (r: f64) {
    r = self * self
}
fn square(self: i64) -> (r: i64) {
    r = self * self
}
fn main(a: f64, b: i64) -> (c: f64) {
    c = a.square() + b.square().f64()
}
"#;
    let a = 100.0f64;
    let b = 100i64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, i64) -> f64>(func_ptr) };
    assert_eq!(20000.0, func(a, b));
    Ok(())
}

#[test]
fn stacked_paren() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f64) -> (c: bool) {
    d = a.i64().f64().i64().f64()
    e = ((((d).i64()).f64()).i64()).f64()
    c = d == e
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> bool>(func_ptr) };
    assert_eq!(true, func(a));
    Ok(())
}

//#[test]
//fn int_min_max() -> anyhow::Result<()> {
//    //Not currently working: Unsupported type for imin instruction: i64
//    //https://github.com/bytecodealliance/wasmtime/issues/3370
//    let code = r#"
//    fn main() -> (e) {
//        c = imin(1, 2)
//        //d = imax(3, 4)
//        //f = c * d
//        e = float(c)
//    }
//"#;
//    let a = 100.0f64;
//    let b = 100.0f64;
//    let mut jit = jit::JIT::new(&[("print", prt2 as *const u8)]);
//    let ast = parser::program(&code)?;
//    let ast = sarus_std_lib::append_std_funcs( ast);
//    jit.translate(ast.clone())?;
//    let func_ptr = jit.get_func("main")?;
//    let func = unsafe { mem::transmute::<_, extern "C" fn(f64, f64) -> f64>(func_ptr) };
//    func(a, b);
//    Ok(())
//}

#[test]
fn readme_example() -> anyhow::Result<()> {
    let code = r#"
struct Line {
    a: Point,
    b: Point,
}

fn length(self: Line) -> (r: f64) {
    r = ((self.a.x - self.b.x).powf(2.0) + 
         (self.a.y - self.b.y).powf(2.0) + 
         (self.a.z - self.b.z).powf(2.0)).sqrt()
}

struct Point {
    x: f64,
    y: f64,
    z: f64,
}

fn length(self: Point) -> (r: f64) {
    r = (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
}

fn main(n: f64) -> (c: f64) {
    p1 = Point {
        x: n,
        y: 200.0,
        z: 300.0,
    }
    p2 = Point {
        x: n * 4.0,
        y: 500.0,
        z: 600.0,
    }
    l1 = Line {
        a: p1,
        b: p2,
    }

    d = l1.a //struct is copied
    e = d.x + l1.a.x
    
    p1.y = e * d.z
    p1.y.assert_eq(e * d.z)

    c = l1.length()
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(true);
    Ok(())
}

#[test]
fn pass_by_ref() -> anyhow::Result<()> {
    let code = r#"
struct Line {
    a: Point,
    b: Point,
}

struct Point {
    x: f64,
    y: f64,
    z: f64,
}

//TODO set_to_0(point: &Point)
fn set_to_0(point: Point) -> () {
    point.x = 0.0
    point.y = 0.0
    point.z = 0.0
}

fn main(n: f64) -> (c: f64) {
    p1 = Point {
        x: n,
        y: 200.0,
        z: 300.0,
    }
    p2 = Point {
        x: n * 4.0,
        y: 500.0,
        z: 600.0,
    }
    l1 = Line {
        a: p1,
        b: p2,
    }
    p1a = p1 //by reference

    //TODO set_to_0(&p1)
    set_to_0(p1) //passed by reference

    p1.x.println()
    p1.y.println()
    p1.z.println()

    p1a.x.println()
    p1a.y.println()
    p1a.z.println()
}
"#;
    let a = 100.0f64;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> f64>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(true);
    Ok(())
}

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct Line {
    a: Point,
    b: Point,
}

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Line {
    fn length(self: Line) -> f64 {
        ((self.a.x - self.b.x).powf(2.0)
            + (self.a.y - self.b.y).powf(2.0)
            + (self.a.z - self.b.z).powf(2.0))
        .sqrt()
    }
}

#[test]
fn rust_struct() -> anyhow::Result<()> {
    let code = r#"
struct Line {
    a: Point,
    b: Point,
}

struct Point {
    x: f64,
    y: f64,
    z: f64,
}

fn length(self: Line) -> (r: f64) {
    r = ((self.a.x - self.b.x).powf(2.0) + 
         (self.a.y - self.b.y).powf(2.0) + 
         (self.a.z - self.b.z).powf(2.0)).sqrt()
}

fn main(l1: Line) -> (c: f64) {
    c = l1.length()
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(Line) -> f64>(func_ptr) };

    let p1 = Point {
        x: 100.0,
        y: 200.0,
        z: 300.0,
    };
    let p2 = Point {
        x: 100.0 * 4.0,
        y: 500.0,
        z: 600.0,
    };

    let l1 = Line { a: p1, b: p2 };

    assert_eq!(l1.length(), func(l1));
    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Misc {
    b1: bool,
    b2: bool,
    f1: f64,
    b3: bool,
    i1: i64,
    b4: bool,
    b5: bool,
}

#[test]
fn repr_alignment() -> anyhow::Result<()> {
    let code = r#"
struct Misc {
    b1: bool,
    b2: bool,
    f1: f64,
    b3: bool,
    i1: i64,
    b4: bool,
    b5: bool,
}

fn main(m: Misc) -> () {
    m.b1.println()
    m.b2.println()
    m.f1.println()
    m.b3.println()
    m.i1.println()
    m.b4.println()
    m.b5.println()

    m.b1.assert_eq(true)
    m.b2.assert_eq(false)
    m.f1.assert_eq(12345.0)
    m.b3.assert_eq(true)
    m.i1.assert_eq(6789)
    m.b4.assert_eq(false)
    m.b5.assert_eq(true)
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    //jit.print_clif(true);
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(Misc) -> ()>(func_ptr) };

    let m = Misc {
        b1: true,
        b2: false,
        f1: 12345.0,
        b3: true,
        i1: 6789,
        b4: false,
        b5: true,
    };
    func(m);
    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Misc2 {
    b1: bool,
    m: Misc,
    b2: bool,
    b3: bool,
}

#[test]
fn repr_alignment_nested() -> anyhow::Result<()> {
    let code = r#"
struct Misc {
    b1: bool,
    b2: bool,
    f1: f64,
    b3: bool,
    i1: i64,
    b4: bool,
    b5: bool,
}

struct Misc2 {
    b1: bool,
    m: Misc,
    b2: bool,
    b3: bool,
}

fn main(m2: Misc2) -> () {
    m2.b1.assert_eq(true)
    m2.m.b1.assert_eq(true)
    m2.m.b2.assert_eq(false)
    m2.m.f1.assert_eq(12345.0)
    m2.m.b3.assert_eq(true)
    m2.m.i1.assert_eq(6789)
    m2.m.b4.assert_eq(false)
    m2.m.b5.assert_eq(true)
    m2.b2.assert_eq(true)
    m2.b3.assert_eq(true)
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    //jit.print_clif(true);
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(Misc2) -> ()>(func_ptr) };

    let m = Misc {
        b1: true,
        b2: false,
        f1: 12345.0,
        b3: true,
        i1: 6789,
        b4: false,
        b5: true,
    };

    let m2 = Misc2 {
        b1: true,
        m,
        b2: true,
        b3: true,
    };

    func(m2);
    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Misc3 {
    b1: bool,
    m2: Misc2,
    f1: f64,
    b3: bool,
}

#[test]
fn repr_alignment_nested2() -> anyhow::Result<()> {
    let code = r#"
struct Misc {
    b1: bool,
    b2: bool,
    f1: f64,
    b3: bool,
    i1: i64,
    b4: bool,
    b5: bool,
}

struct Misc2 {
    b1: bool,
    m: Misc,
    b2: bool,
    b3: bool,
}

struct Misc3 {
    b1: bool,
    m2: Misc2,
    f1: f64,
    b3: bool,
}

fn main(m3: Misc3) -> () {
    m3.b1.assert_eq(true)
    m3.m2.b1.assert_eq(true)
    m3.m2.m.b1.assert_eq(true)
    m3.m2.m.b2.assert_eq(false)
    m3.m2.m.f1.assert_eq(12345.0)
    m3.m2.m.b3.assert_eq(true)
    m3.m2.m.i1.assert_eq(6789)
    m3.m2.m.b4.assert_eq(false)
    m3.m2.m.b5.assert_eq(true)
    m3.m2.b2.assert_eq(true)
    m3.m2.b3.assert_eq(true)
    m3.f1.assert_eq(54321.0)
    m3.b3.assert_eq(true)
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    //jit.print_clif(true);
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(Misc3) -> ()>(func_ptr) };

    let m = Misc {
        b1: true,
        b2: false,
        f1: 12345.0,
        b3: true,
        i1: 6789,
        b4: false,
        b5: true,
    };

    let m2 = Misc2 {
        b1: true,
        m,
        b2: true,
        b3: true,
    };

    let m3 = Misc3 {
        b1: true,
        m2,
        f1: 54321.0,
        b3: true,
    };

    func(m3);
    Ok(())
}

#[repr(C)]
#[derive(Debug, PartialEq)]
struct Stuff {
    w: bool,
    x: f64,
    y: f64,
    z: f64,
    i: i64,
}

extern "C" fn returns_a_stuff(a: f64) -> Stuff {
    Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    }
}

#[test]
fn return_struct() -> anyhow::Result<()> {
    let code = r#"
extern fn returns_a_stuff(a: f64) -> (c: Stuff) {}

struct Stuff {
    w: bool,
    x: f64,
    y: f64,
    z: f64,
    i: i64,
}
fn main(a: f64) -> (c: Stuff) {
    c = Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    }
}
fn main2(a: f64) -> (c: Stuff) {
    s = Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    }
    c = s
}
fn main3(a: f64) -> (c: Stuff) {
    s = main2(a)
    c = s
}
fn main4(a: f64) -> (c: Stuff) {
    s = returns_a_stuff(a)
    c = s
}
fn main5(a: f64) -> (c: Stuff) {
    s = returns_a_stuff(a)
    s.w = !s.w
    s.x *= s.x
    s.y *= s.y
    s.z *= s.z
    s.i *= s.i
    c = s
}
"#;
    let a = 100.0f64;
    let correct_stuff = Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    };

    let mut jit = default_std_jit_from_code_with_importer(&code, |_ast, jit_builder| {
        jit_builder.symbols([("returns_a_stuff", returns_a_stuff as *const u8)]);
    })?;

    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main2")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main3")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main4")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main5")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f64) -> Stuff>(func_ptr) };
    assert_eq!(
        Stuff {
            w: false,
            x: 10000.0,
            y: 40000.0,
            z: 90000.0,
            i: 15129,
        },
        func(a)
    );

    Ok(())
}

#[test]
fn struct_size() -> anyhow::Result<()> {
    let code = r#"
struct Misc {
    b1: bool,
    b2: bool,
    f1: f64,
    b3: bool,
    i1: i64,
    b4: bool,
    b5: bool,
}

struct Misc2 {
    b1: bool,
    m: Misc,
    b2: bool,
    b3: bool,
}

struct Misc3 {
    b1: bool,
    m2: Misc2,
    f1: f64,
    b3: bool,
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    //jit.print_clif(true);
    let (data_ptr, _size) = jit.get_data("Misc::size")?;
    let size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(mem::size_of::<Misc>(), *size as usize);
    let (data_ptr, _size) = jit.get_data("Misc2::size")?;
    let size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(mem::size_of::<Misc2>(), *size as usize);
    let (data_ptr, _size) = jit.get_data("Misc3::size")?;
    let size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(mem::size_of::<Misc3>(), *size as usize);
    let (data_ptr, _size) = jit.get_data("f64::size")?;
    let size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(mem::size_of::<f64>(), *size as usize);
    Ok(())
}

struct Heap {
    ptr: *mut u8,
    layout: Layout,
}
impl Drop for Heap {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) }
    }
}
impl Heap {
    pub fn new(size: usize) -> anyhow::Result<Self> {
        let layout = Layout::from_size_align(size, 8)?;
        let ptr = unsafe { alloc(layout) };
        Ok(Heap { ptr, layout })
    }
    pub fn get_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

#[test]
fn anonymous_struct_on_a_heap() -> anyhow::Result<()> {
    let code = r#"
struct Stuff {
    w: bool,
    x: f64,
    y: f64,
    z: f64,
    i: i64,
}
fn puts_a_stuff_there(there: Stuff) -> () {
    there = Stuff {
        w: true,
        x: 100.0,
        y: 200.0,
        z: 300.0,
        i: 123,
    }
}
fn takes_a_stuff(s: Stuff) -> () {
    s.w.assert_eq(true)
    s.x.assert_eq(100.0)
    s.y.assert_eq(200.0)
    s.z.assert_eq(300.0)
    s.i.assert_eq(123)
}
fn size_of_stuff() -> (size: i64) {
    size = Stuff::size
}
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("size_of_stuff")?;
    let size_of_stuff = unsafe { mem::transmute::<_, extern "C" fn() -> i64>(func_ptr) };

    let data = Heap::new(size_of_stuff() as usize)?;

    let func_ptr = jit.get_func("puts_a_stuff_there")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(*mut u8) -> ()>(func_ptr) };
    func(data.get_ptr());

    let func_ptr = jit.get_func("takes_a_stuff")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(*mut u8) -> ()>(func_ptr) };
    func(data.get_ptr());

    Ok(())
}

#[repr(C)]
struct AudioData {
    left: *const f64,
    right: *const f64,
    len: i64,
}

//TODO get dot accessing working with arrays
#[test]
fn struct_of_slices_of_numbers() -> anyhow::Result<()> {
    let code = r#"
struct AudioData {
    left: &[f64],
    right: &[f64],
    len: i64,
}

fn process(audio: AudioData) -> () {
    i = 0
    while i < audio.len {
        left = audio.left
        //left[i] = (audio.left[i] * 10.0).tanh() * 0.1
        //right[i] = (audio.right[i] * 10.0).tanh() * 0.1
        left[i] = i.f64() //TODO does not check type
        audio.right[i] = i.f64()  
        i += 1
    }
    audio.left[1].assert_eq(1.0)
    audio.left[2].assert_eq(2.0)
    (audio.left[3]).assert_eq(3.0)
    (audio.left[4]).assert_eq(4.0)
    (left[5]).assert_eq(5.0)
    left[6].assert_eq(6.0)
}
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioData) -> ()>(func_ptr) };

    let left = vec![0.0f64; 4096];
    let right = vec![0.0f64; 4096];

    let mut audio_data = AudioData {
        left: left.as_ptr(),
        right: right.as_ptr(),
        len: 64,
    };

    func(&mut audio_data);

    Ok(())
}

#[test]
fn struct_of_slices_of_numbers2() -> anyhow::Result<()> {
    let code = r#"
struct AudioData {
    left: &[f64],
    right: &[f64],
    len: i64,
}

fn process(audio: AudioData) -> () {
    i = 0
    while i < audio.len {
        audio.right[i] = i.f64()  
        i += 1
    }
    audio.right[1].assert_eq(1.0)
}
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioData) -> ()>(func_ptr) };

    let left = vec![0.0f64; 4096];
    let right = vec![0.0f64; 4096];

    let mut audio_data = AudioData {
        left: left.as_ptr(),
        right: right.as_ptr(),
        len: 64,
    };

    func(&mut audio_data);

    Ok(())
}

#[repr(C)]
struct BoolData {
    left: *const bool,
    right: *const bool,
    len: i64,
}

#[test]
fn struct_of_slices_of_bools() -> anyhow::Result<()> {
    let code = r#"
struct BoolData {
    left: &[bool],
    right: &[bool],
    len: i64,
}

fn process(audio: BoolData) -> () {
    i = 0
    while i < audio.len {
        audio.right[i] = (i.f64()).rem_euclid(2.0) == 1.0
        i += 1
    }
    (audio.right[0]).assert_eq(false)
    audio.right[1].assert_eq(true)
    audio.right[2].assert_eq(false)
    audio.right[3].assert_eq(true)
    audio.right[4].assert_eq(false)
    audio.right[5].assert_eq(true)
}
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut BoolData) -> ()>(func_ptr) };

    let left = vec![false; 4096];
    let right = vec![false; 4096];

    let mut audio_data = BoolData {
        left: left.as_ptr(),
        right: right.as_ptr(),
        len: 64,
    };

    func(&mut audio_data);

    Ok(())
}

#[derive(Clone, Copy)]
#[repr(C)]
struct AudioPair {
    left: f64,
    right: f64,
}

#[repr(C)]
struct AudioSamples {
    samples: *const AudioPair,
    len: i64,
}

#[test]
fn struct_of_slices_of_structs() -> anyhow::Result<()> {
    let code = r#"
struct AudioPair {
    left: f64,
    right: f64,
}

struct AudioSamples {
    samples: &[AudioPair],
    len: i64,
}

fn process(audio: AudioSamples) -> () {
    i = 0
    while i < audio.len {
        sample = audio.samples[i]
        sample.left = i.f64()
        sample.right = i.f64() + 0.5

        audio.samples[i].left = i.f64()  
        audio.samples[i].right = i.f64() + 0.5
        
        (audio.samples[i]).left = i.f64()
        (audio.samples[i]).right = i.f64() + 0.5
        
        i += 1
    }
    
    sample = audio.samples[1]
    sample.left.assert_eq(1.0)
    sample.right.assert_eq(1.5)
    sample = audio.samples[2]
    sample.left.assert_eq(2.0)
    sample.right.assert_eq(2.5)
    sample = audio.samples[3]
    sample.left.assert_eq(3.0)
    sample.right.assert_eq(3.5)

    s = audio.samples
    a = s[2]
    a.left.assert_eq(2.0)
    a.right.assert_eq(2.5)

    n = AudioPair {
        left: 1000.0,
        right: 1000.5,
    }

    s[5] = n

    d = audio.samples[5].left
    e = audio.samples[5].right

    d.assert_eq(1000.0)
    e.assert_eq(1000.5)



    (audio.samples[6].left).assert_eq(6.0)
    audio.samples[6].left.assert_eq(6.0)
    (audio.samples[6].right).assert_eq(6.5)
    audio.samples[6].right.assert_eq(6.5)

}
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioSamples) -> ()>(func_ptr) };

    let samples = vec![
        AudioPair {
            left: 0.0,
            right: 0.0,
        };
        4096
    ];

    let mut audio_samples = AudioSamples {
        samples: samples.as_ptr(),
        len: samples.len() as i64,
    };

    func(&mut audio_samples);

    Ok(())
}

//
// TODO
// This should return an error that a1 does not exist
// maybe the validator is not looking at struct assigns
/*

#[test]
fn lowshelf() -> anyhow::Result<()> {
    let code = r#"
struct AudioData {
    left: &[f64],
    right: &[f64],
    len: i64,
}

fn FilterParams::lowshelf(cutoff_hz, gain_db, q_value) -> (params: FilterParams) {
    cutoff_hz = cutoff_hz.min(48000.0 * 0.5)
    a = (10.0).powf(gain_db / 40.0)
    g = (PI * cutoff_hz / 48000.0).tan() / a.sqrt()
    k = 1.0 / q_value
    params = FilterParams {
        a1: 1.0 / (1.0 + g * (g + k)),
        a2: g * a1,
        a3: g * a2,
        m0: 1.0,
        m1: k * (a - 1.0),
        m2: a * a - 1.0,
    }
}

fn process(audio: AudioData) -> () {
    lowshelf = FilterParams::lowshelf(1000.0, -10.0, 2.0)
    i = 0
    while i < audio.len {
        left = audio.left
        right = audio.right
        left[i] = (left[i] * 10.0).tanh() * 0.1
        right[i] = (right[i] * 10.0).tanh() * 0.1
        i += 1
    }
}


struct FilterParams { a1, a2, a3, m0, m1, m2, }
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioData) -> ()>(func_ptr) };

    let left = vec![0.0f64; 4096];
    let right = vec![0.0f64; 4096];

    let mut audio_data = AudioData {
        left: left.as_ptr(),
        right: right.as_ptr(),
        len: 64,
    };

    func(&mut audio_data);

    Ok(())
}
*/

#[test]
fn inner_struct_manipulate() -> anyhow::Result<()> {
    let code = r#"
struct Filter {
    ic1eq,
    ic2eq,
}

struct ProcessState {
    filter_l: Filter,
    filter_r: Filter,
}


fn process(audio: AudioData) -> () {
    filter_l = Filter { ic1eq: 0.0, ic2eq: 0.0, }
    filter_r = Filter { ic1eq: 0.0, ic2eq: 0.0, }
    state = ProcessState {
        filter_l: filter_l,
        filter_r: filter_r,
    }
    state.filter_l.set_val()
    state.filter_r.set_val()

    state.filter_l.ic1eq.assert_eq(1.0)
    state.filter_l.ic2eq.assert_eq(2.0)
    state.filter_r.ic1eq.assert_eq(1.0)
    state.filter_r.ic2eq.assert_eq(2.0)
}

fn set_val(self: Filter) -> () {
    self.ic1eq = 1.0
    self.ic2eq = 2.0
}
"#;

    let mut jit = default_std_jit_from_code(&code)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn() -> ()>(func_ptr) };
    func();

    Ok(())
}

#[test]
fn src_line() -> anyhow::Result<()> {
    let code = r#"
fn main() -> () {
    src_line().assert_eq(2)
    src_line().assert_eq(3) src_line().assert_eq(3)
    //

    src_line().assert_eq(6)
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
    func();
    Ok(())
}

#[test]
fn const_size() -> anyhow::Result<()> {
    let code = r#"
fn main() -> () {
    f64::size.println()
}
"#;
    let mut jit = default_std_jit_from_code(&code)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
    func();
    let (data_ptr, _size) = jit.get_data("f64::size")?;
    let f64_size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(*f64_size, 8);

    Ok(())
}

//#[test]
//fn fixed_array_size() -> anyhow::Result<()> {
//    let code = r#"
//struct A {
//    a: f64,
//}
//
//fn main() -> () {
//    s = A {
//        a: 1.0,
//    }
//    n = [s; 10]
//    n[0].a.println()
//    n1 = [1; 10]
//    n2 = [1.0; 10]
//}
//"#;
//    let mut jit = default_std_jit_from_code(&code)?;
//    let func_ptr = jit.get_func("main")?;
//    let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
//    func();
//
//    Ok(())
//}
