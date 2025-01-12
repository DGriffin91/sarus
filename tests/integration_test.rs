#![feature(core_intrinsics)]
#![feature(try_blocks)]

use cranelift_jit::JITBuilder;
use serde::Deserialize;
use std::{
    alloc::{alloc, dealloc, Layout},
    collections::HashMap,
    env,
    f32::consts::*,
    ffi::CStr,
    mem,
    path::PathBuf,
};
use tracing::info;

#[allow(unused_imports)]
use sarus::logging::setup_logging;

use sarus::*;

fn only_run_func(code: &str) -> anyhow::Result<()> {
    info!("test with deep stack");
    {
        let use_deep_stack = true;
        let mut jit = default_std_jit_from_code(code, use_deep_stack)?;
        let func_ptr = jit.get_func("main")?;
        let funcc = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
        funcc();
    }
    info!("test without deep stack");
    {
        let use_deep_stack = false;
        let mut jit = default_std_jit_from_code(code, use_deep_stack)?;
        let func_ptr = jit.get_func("main")?;
        let funcc = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
        funcc();
    }
    Ok(())
}

fn only_run_func_with_importer(
    ast: Vec<Declaration>,
    file_index_table: Option<Vec<PathBuf>>,
    importer: impl FnOnce(&mut Vec<Declaration>, &mut JITBuilder),
) -> anyhow::Result<()> {
    let mut jit = default_std_jit_from_code_with_importer(
        ast.clone(),
        file_index_table.clone(),
        importer,
        true,
    )?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
    func();

    Ok(())
}

#[test]
fn parentheses() -> anyhow::Result<()> {
    let code = r#"
fn main(a, b) -> (c) {
    c = a * (a - b) * (a * (2.0 + b))
}
"#;

    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(a * (a - b) * (a * (2.0 + b)), func(a, b));
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

    let a = 100.0f32;
    let b = 200.0f32;
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

    let epsilon = 0.00001;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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

    let a = 100.1f32;
    let b = 200.2f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(
        a.ceil() * b.floor() * a.trunc() * (a * b * -1.234).fract() * 1.5f32.round(),
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
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(a, func(a, b));

    let code = r#"
    fn main(a, b) -> (c) {
        c = a.max(b)
    }
    "#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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

    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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

    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(6893909.333333333, func(a, b));
    Ok(())
}

#[test]
fn multiple_returns_simple() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        c, d = stuff(a, b)
        e = c+ d
    }
    
    fn stuff(a, b) -> (c, d) {
        c = a + 1.0
        d = c + b + 10.0
    }
"#;

    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    dbg!(func(a, b));
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
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(100.0, func(a, b));
    Ok(())
}

#[test]
fn array_read_write() -> anyhow::Result<()> {
    let code = r#"
fn main(arr: &[f32], b) -> () {
    arr[0] = arr[0] * b
    arr[1] = arr[1] * b
    arr[2] = arr[2] * b
    arr[3] = arr[3] * b
}
"#;

    let mut arr = [1.0, 2.0, 3.0, 4.0];
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(*mut f32, f32)>(func_ptr) };
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
    let a = -100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> f32>(func_ptr) };
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
//    let a = -100.0f32;
//    let mut jit = default_std_jit_from_code(code, true)?;
//    let func_ptr = jit.get_func("main")?;
//    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> f32>(func_ptr) };
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
        c = a.sin()
    }
        
    fn graph (audio: &[f32]) -> () {
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
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("graph")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [f32; 8])>(func_ptr) };
    dbg!(func(&mut audio));
    Ok(())
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Metadata {
    description: Option<String>,
    inputs: HashMap<String, MetadataInput>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MetadataInput {
    default: Option<f32>,
    min: Option<f32>,
    max: Option<f32>,
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
        c = a.sin()
    }
        
    fn graph (audio: &[f32]) -> () {
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
    let ast = parse(code)?;
    let mut jit = default_std_jit_from_code(code, true)?;

    let func_meta: Option<Metadata> = ast.iter().find_map(|d| match d {
        frontend::Declaration::Metadata(head, body) => {
            if let Some(head) = head.first() {
                if head == "add_node" {
                    Some(toml::from_str(body).unwrap())
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
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [f32; 8])>(func_ptr) };
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

    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(2048.0, func(a, b));
    Ok(())
}

#[test]
fn int_to_float() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        i = 2
        e = i.f32() * a * b * (2).f32() * 2.0 * (2).f32()
    }
"#;

    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(320000.0, func(a, b));
    Ok(())
}

#[test]
fn float_conversion() -> anyhow::Result<()> {
    let code = r#"
    fn main(a, b) -> (e) {
        i_a = a.i64()
        e = if i_a < b.i64() {
            i_a.f32().i64().f32()
        } else {
            2.0
        }
    }
"#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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
        e = e_i.f32()
    }
"#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(1.0, func(a, b));
    Ok(())
}

#[test]
fn if_else_multi() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {   
    a = 0
    b = 0
    c = 0
    d = 0
    if true {
        a = 1
        b = 2
    } else {
        c = 3
        d = 4
    }
    a.assert_eq(1)
    b.assert_eq(2)
    c.assert_eq(0)
    d.assert_eq(0)
    if false {
        a = 5
        b = 6
    } else {
        c = 7
        d = 8
    }
    a.assert_eq(1)
    b.assert_eq(2)
    c.assert_eq(7)
    d.assert_eq(8)
}
"#;
    only_run_func(code)
}

#[test]
fn array_return_from_if() -> anyhow::Result<()> {
    let code = r#"
fn main(arr1: &[f32], arr2: &[f32], b) -> () {
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
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(*mut f32, *mut f32, f32)>(func_ptr) };
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
        e = n3.f32()
    }
"#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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

    let a = 100.0f32;
    let b = 200.0f32;
    let c = 300.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32, f32) -> f32>(func_ptr) };
    assert_eq!(600.0, func(a, b, c));
    Ok(())
}

#[test]
fn manual_types() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f32, b: f32) -> (c: f32) {
    c = a * (a - b) * (a * (2.0 + b))
}
"#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(a * (a - b) * (a * (2.0 + b)), func(a, b));
    Ok(())
}

#[test]
fn i64_params() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f32, b: i64) -> (c: i64) {
    e = a * (a - b.f32()) * (a * (2.0 + b.f32()))
    c = e.i64()
}
"#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, i64) -> i64>(func_ptr) };
    assert_eq!((a * (a - b) * (a * (2.0 + b))) as i64, func(a, b as i64));
    Ok(())
}

#[test]
fn i64_params_multifunc() -> anyhow::Result<()> {
    //Not currently working, see BLOCKERs in jit.rs
    let code = r#"
fn main(a: f32, b: i64) -> (c: i64) {
    c = foo(a, b, 2)
}
fn foo(a: f32, b: i64, c: i64) -> (d: i64) {
    d = a.i64() + b + c
}
"#;
    let a = 100.0f32;
    let b = 200.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, i64) -> i64>(func_ptr) };
    assert_eq!(302, func(a, b as i64));
    Ok(())
}

#[test]
fn bool_params() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f32, b: bool) -> (c: f32) {
    c = if b {
        a
    } else {
        0.0-a
    }
}
"#;
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, bool) -> f32>(func_ptr) };
    assert_eq!(a, func(a, true));
    assert_eq!(-a, func(a, false));
    Ok(())
}

#[test]
fn if_else_return_tuple_assignment() -> anyhow::Result<()> {
    let code = r#"
fn main() -> () {  
    a, b = if true {1.0} else {-1.0}, if true {-1.0} else {1.0}
    a.assert_eq(1.0)
    b.assert_eq(-1.0)
}
"#;
    only_run_func(code)
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
    let mut jit = default_std_jit_from_code(code, true)?;
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
fn nested() -> (c: bool) {
    c = !(!(1.0 < 2.0) && !(2.0 < 3.0))
}
fn parenassign() -> (c: bool) {
    c = !((1.0 < 2.0) && (2.0 < 3.0) && true)
}
"#;
    let mut jit = default_std_jit_from_code(code, true)?;
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
    assert_eq!(if !(!(false || !false)) { true } else { false }, f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("ifthen3")?) };
    assert_eq!(
        if !(!(1.0 < 2.0) && !(2.0 < 3.0)) {
            false
        } else {
            true
        },
        f()
    );
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("nested")?) };
    assert_eq!(!(!(1.0 < 2.0) && !(2.0 < 3.0)), f());
    let f = unsafe { mem::transmute::<_, extern "C" fn() -> bool>(jit.get_func("parenassign")?) };
    assert_eq!(!((1.0 < 2.0) && (2.0 < 3.0) && true), f());
    Ok(())
}

extern "C" fn mult(a: f32, b: f32) -> f32 {
    a * b
}

extern "C" fn dbg(a: f32) {
    dbg!(a);
}

#[test]
fn extern_func() -> anyhow::Result<()> {
    let code = r#"
extern fn mult(a: f32, b: f32) -> (c: f32) {}
extern fn dbg(a: f32) -> () {}

fn main(a: f32, b: f32) -> (c: f32) {
    c = mult(a, b)
    dbg(a)
}
"#;
    let a = 100.0f32;
    let b = 100.0f32;
    let ast = parse(code)?;
    let mut jit = default_std_jit_from_code_with_importer(
        ast,
        None,
        |_ast, jit_builder| {
            jit_builder.symbols([("mult", mult as *const u8), ("dbg", dbg as *const u8)]);
        },
        true,
    )?;

    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    assert_eq!(mult(a, b), func(a, b));
    Ok(())
}

extern "C" fn prt2(s: sarus_std_lib::SarusSlice<u8>) {
    print!("{}", std::str::from_utf8(s.slice()).unwrap());
}

#[test]
fn create_string() -> anyhow::Result<()> {
    let code = "
fn main(a: f32, b: f32) -> (c: f32) {
    print(\"HELLO\n\")
    print(\"WORLD\n\")
    c = a
}

extern fn print(s: [u8]) -> () {}
";
    let a = 100.0f32;
    let b = 100.0f32;
    let ast = parse(code)?;
    let mut jit = default_std_jit_from_code_with_importer(
        ast,
        None,
        |_ast, jit_builder| {
            jit_builder.symbols([("print", prt2 as *const u8)]);
        },
        true,
    )?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
    func(a, b);

    Ok(())
}

#[test]
fn struct_access() -> anyhow::Result<()> {
    let code = r#"
struct Point {
    x: f32,
    y: f32,
    z: f32,
}
fn main(a: f32) -> (c: f32) {
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
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    //jit.print_clif(true);
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> f32>(func_ptr) };
    dbg!(600.0, func(a));
    Ok(())
}

#[test]
fn struct_impl() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
struct Point {
    x: f32,
    y: f32,
    z: f32,
}
fn length(self: Point) -> (r: f32) {
    r = (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
}
fn main(a: f32) -> (c: f32) {
    p = Point {
        x: a,
        y: 200.0,
        z: 300.0,
    }
    c = p.length()
}
"#;
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> f32>(func_ptr) };
    assert_eq!(374.16573867739413, func(a));
    Ok(())
}

extern "C" fn dbgi(s: *const i8, a: i64) {
    unsafe {
        println!("{} {}", CStr::from_ptr(s).to_str().unwrap(), a);
    }
}
extern "C" fn dbgf(s: *const i8, a: f32) {
    unsafe {
        println!("{} {}", CStr::from_ptr(s).to_str().unwrap(), a);
    }
}

#[test]
fn struct_impl_nested() -> anyhow::Result<()> {
    let code = r#"
extern fn dbgf(s: &, a: f32) -> () {}
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

fn length(self: Line) -> (r: f32) {
    r = ((self.a.x - self.b.x).powf(2.0) + 
         (self.a.y - self.b.y).powf(2.0) + 
         (self.a.z - self.b.z).powf(2.0)).sqrt()
}

struct Point {
    x: f32,
    y: f32,
    z: f32,
}

fn print(self: Point) -> () {
    "Point {".println()
    "x: ".print() self.x.print() ",".println()
    "y: ".print() self.y.print() ",".println()
    "z: ".print() self.z.print() ",".println()
    "}".println()
}

fn length(self: Point) -> (r: f32) {
    r = (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
}

fn main(n: f32) -> (c: f32) {
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
    d = l1.a
    e = d.x + l1.a.x
    d.print()
    
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
    let a = 100.0f32;
    let ast = parse(code)?;
    let mut jit = default_std_jit_from_code_with_importer(
        ast,
        None,
        |_ast, jit_builder| {
            jit_builder.symbols([("dbgf", dbgf as *const u8), ("dbgi", dbgi as *const u8)]);
        },
        true,
    )?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> f32>(func_ptr) };
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
    x: f32,
    y: f32,
    z: f32,
}

fn main(n: f32) -> () {
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

    //let ast = parse(code)?;
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32)>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(true);
    Ok(())
}

#[test]
fn struct_impl_nested_short() -> anyhow::Result<()> {
    let code = r#"
extern fn dbgf(s: &, a: f32) -> () {}
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
    x: f32,
}

fn print(self: Bar) -> () {
    "Bar {".println()
    "x: ".print() self.x.print() ",".println()
    "}".println()
}

fn main(n: f32) -> () {
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
    let a = 100.0f32;
    let ast = parse(code)?;
    let mut jit = default_std_jit_from_code_with_importer(
        ast,
        None,
        |_ast, jit_builder| {
            jit_builder.symbols([("dbgf", dbgf as *const u8), ("dbgi", dbgi as *const u8)]);
        },
        true,
    )?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32)>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(false);
    Ok(())
}
#[test]
fn type_impl() -> anyhow::Result<()> {
    let code = r#"
fn square(self: f32) -> (r: f32) {
    r = self * self
}
fn square(self: i64) -> (r: i64) {
    r = self * self
}
fn main(a: f32, b: i64) -> (c: f32) {
    c = a.square() + b.square().f32()
}
"#;
    let a = 100.0f32;
    let b = 100i64;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, i64) -> f32>(func_ptr) };
    assert_eq!(20000.0, func(a, b));
    Ok(())
}

#[test]
fn stacked_paren() -> anyhow::Result<()> {
    let code = r#"
fn main(a: f32) -> (c: bool) {
    d = a.i64().f32().i64().f32()
    e = ((((d).i64()).f32()).i64()).f32()
    c = d == e
}
"#;
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> bool>(func_ptr) };
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
//    let a = 100.0f32;
//    let b = 100.0f32;
//    let mut jit = jit::JIT::new(&[("print", prt2 as *const u8)]);
//    let ast = parse(code)?;
//    let ast = sarus_std_lib::append_std_funcs( ast);
//    jit.translate(ast.clone())?;
//    let func_ptr = jit.get_func("main")?;
//    let func = unsafe { mem::transmute::<_, extern "C" fn(f32, f32) -> f32>(func_ptr) };
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

fn length(self: Line) -> (r: f32) {
    r = ((self.a.x - self.b.x).powf(2.0) + 
         (self.a.y - self.b.y).powf(2.0) + 
         (self.a.z - self.b.z).powf(2.0)).sqrt()
}

struct Point {
    x: f32,
    y: f32,
    z: f32,
}

fn length(self: Point) -> (r: f32) {
    r = (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
}

fn main(n: f32) -> (c: f32) {
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

    d = l1.a
    e = d.x + l1.a.x
    
    p1.y = e * d.z
    p1.y.assert_eq(e * d.z)

    c = l1.length()
}
"#;
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> f32>(func_ptr) };
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
    x: f32,
    y: f32,
    z: f32,
}

//TODO set_to_0(point: &Point)
fn set_to_0(point: Point) -> () {
    point.x = 0.0
    point.y = 0.0
    point.z = 0.0
}

fn main(n: f32) -> () {
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
    let a = 100.0f32;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32)>(func_ptr) };
    dbg!(func(a));
    //assert_eq!(200.0, func(a));
    //jit.print_clif(true);
    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Line {
    a: Point,
    b: Point,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
}

impl Line {
    fn length(self: Line) -> f32 {
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
    x: f32,
    y: f32,
    z: f32,
}

fn length(self: Line) -> (r: f32) {
    r = ((self.a.x - self.b.x).powf(2.0) + 
         (self.a.y - self.b.y).powf(2.0) + 
         (self.a.z - self.b.z).powf(2.0)).sqrt()
}

fn main(l1: Line) -> (c: f32) {
    c = l1.length()
}
"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(Line) -> f32>(func_ptr) };

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
    f1: f32,
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
    f1: f32,
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
    let mut jit = default_std_jit_from_code(code, true)?;
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
    f1: f32,
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
    let mut jit = default_std_jit_from_code(code, true)?;
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
    f1: f32,
    b3: bool,
}

#[test]
fn repr_alignment_nested2() -> anyhow::Result<()> {
    let code = r#"
struct Misc {
    b1: bool,
    b2: bool,
    f1: f32,
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
    f1: f32,
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
    let mut jit = default_std_jit_from_code(code, true)?;
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
    x: f32,
    y: f32,
    z: f32,
    i: i64,
}

extern "C" fn returns_a_stuff(a: f32) -> Stuff {
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
extern fn returns_a_stuff(a: f32) -> (c: Stuff) {}

struct Stuff {
    w: bool,
    x: f32,
    y: f32,
    z: f32,
    i: i64,
}
fn main(a: f32) -> (c: Stuff) {
    c = Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    }
}
fn main2(a: f32) -> (c: Stuff) {
    s = Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    }
    c = s
}
fn main3(a: f32) -> (c: Stuff) {
    s = main2(a)
    c = s
}
fn main4(a: f32) -> (c: Stuff) {
    s = returns_a_stuff(a)
    c = s
}
fn main5(a: f32) -> (c: Stuff) {
    s = returns_a_stuff(a)
    //TODO !s.w is not supported because unary ! and - only work with unary exprs as operand
    //This was required to get the order of operations to be correct for things like two + -four + two
    s.w = !(s.w) 
    s.x *= s.x
    s.y *= s.y
    s.z *= s.z
    s.i *= s.i
    c = s
}
"#;
    let a = 100.0f32;
    let correct_stuff = Stuff {
        w: true,
        x: a,
        y: 200.0,
        z: 300.0,
        i: 123,
    };

    let ast = parse(code)?;
    let mut jit = default_std_jit_from_code_with_importer(
        ast,
        None,
        |_ast, jit_builder| {
            jit_builder.symbols([("returns_a_stuff", returns_a_stuff as *const u8)]);
        },
        true,
    )?;

    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main2")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main3")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main4")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> Stuff>(func_ptr) };
    assert_eq!(correct_stuff, func(a));
    let func_ptr = jit.get_func("main5")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(f32) -> Stuff>(func_ptr) };
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
    f1: f32,
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
    f1: f32,
    b3: bool,
}
"#;
    let mut jit = default_std_jit_from_code(code, true)?;
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
    let (data_ptr, _size) = jit.get_data("f32::size")?;
    let size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(mem::size_of::<f32>(), *size as usize);
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
    x: f32,
    y: f32,
    z: f32,
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

    let mut jit = default_std_jit_from_code(code, true)?;

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
    left: *const f32,
    right: *const f32,
    len: i64,
}

#[test]
fn struct_of_slices_of_numbers() -> anyhow::Result<()> {
    let code = r#"
struct AudioData {
    left: &[f32],
    right: &[f32],
    len: i64,
}

fn process(audio: AudioData) -> () {
    i = 0
    left = audio.left
    while i < audio.len {
        left[i] = i.f32()
        audio.right[i] = i.f32()  
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

    let mut jit = default_std_jit_from_code(code, true)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioData) -> ()>(func_ptr) };

    let left = vec![0.0f32; 4096];
    let right = vec![0.0f32; 4096];

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
    left: &[f32],
    right: &[f32],
    len: i64,
}

fn process(audio: AudioData) -> () {
    i = 0
    while i < audio.len {
        audio.right[i] = i.f32()  
        i += 1
    }
    audio.right[1].assert_eq(1.0)
}
"#;

    let mut jit = default_std_jit_from_code(code, true)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioData) -> ()>(func_ptr) };

    let left = vec![0.0f32; 4096];
    let right = vec![0.0f32; 4096];

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
        audio.right[i] = (i.f32()).rem_euclid(2.0) == 1.0
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

    let mut jit = default_std_jit_from_code(code, true)?;

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
    left: f32,
    right: f32,
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
    left: f32,
    right: f32,
}

struct AudioSamples {
    samples: &[AudioPair],
    len: i64,
}

fn process(audio: AudioSamples) -> () {
    i = 0
    while i < audio.len {
        sample = audio.samples[i]
        sample.left = i.f32()
        sample.right = i.f32() + 0.5

        audio.samples[i].left = i.f32()  
        audio.samples[i].right = i.f32() + 0.5
        
        (audio.samples[i]).left = i.f32()
        (audio.samples[i]).right = i.f32() + 0.5
        
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

    let mut jit = default_std_jit_from_code(code, true)?;

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
    left: &[f32],
    right: &[f32],
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

    let mut jit = default_std_jit_from_code(code, true)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut AudioData) -> ()>(func_ptr) };

    let left = vec![0.0f32; 4096];
    let right = vec![0.0f32; 4096];

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
    //setup_logging();
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

    let mut jit = default_std_jit_from_code(code, true)?;

    let func_ptr = jit.get_func("process")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn() -> ()>(func_ptr) };
    func();

    Ok(())
}

#[test]
fn src_line() -> anyhow::Result<()> {
    let code = r#"
fn main() -> () {
    src_line().assert_eq(3)
    src_line().assert_eq(4) src_line().assert_eq(4)
    //

    src_line().assert_eq(7)
}
"#;
    only_run_func(code)
}

#[test]
fn const_size() -> anyhow::Result<()> {
    let code = r#"
fn main() -> () {
    f32::size.println()
}
"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
    func();
    let (data_ptr, _size) = jit.get_data("f32::size")?;
    let f32_size: &i64 = unsafe { mem::transmute(data_ptr) };
    assert_eq!(*f32_size, 4);

    Ok(())
}

#[test]
fn fixed_arrays() -> anyhow::Result<()> {
    let code = r#"
struct A {
    a: f32,
    b: f32,
    c: bool,
    d: i64,
}

fn main() -> () {
    s = A {
        a: 1.0,
        b: 2.0,
        c: true,
        d: 3,
    }
    n = [s; 10]
    n[0].a.assert_eq(1.0)
    n[0].b.assert_eq(2.0)
    n[0].c.assert_eq(true)
    n[0].d.assert_eq(3)
    n[9].a.assert_eq(1.0)
    n[9].b.assert_eq(2.0)
    n[9].c.assert_eq(true)
    n[9].d.assert_eq(3)
    n1 = [1; 3]
    n2 = [1.0; 3]
    n1[0].assert_eq(1)
    n2[0].assert_eq(1.0)
    n1[1].assert_eq(1)
    n2[1].assert_eq(1.0)

    i = 0
    while i < 10 {
        n[i] = A {
            a: i.f32(),
            b: i.f32() + 0.5,
            c: i.f32().rem_euclid(2.0) == 0.0,
            d: i * i,
        }
        i += 1
    }
    i = 0
    while i < 10 {
        n[i].a.assert_eq(i.f32())
        n[i].b.assert_eq(i.f32() + 0.5)
        n[i].c.assert_eq(i.f32().rem_euclid(2.0) == 0.0)
        n[i].d.assert_eq(i * i)
        i += 1
    }
    //TODO n.len()
}
"#;
    only_run_func(code)
}

#[test]
fn fixed_arrays_func() -> anyhow::Result<()> {
    let code = r#"
fn takes_an_array(n: [A; 10]) -> () {
    i = 0
    while i < 10 {
        n[i].a.assert_eq(i.f32())
        n[i].b.assert_eq(i.f32() + 0.5)
        n[i].c.assert_eq(i.f32().rem_euclid(2.0) == 0.0)
        n[i].d.assert_eq(i * i)
        i += 1
    }
}

struct A {
    a: f32,
    b: f32,
    c: bool,
    d: i64,
}

fn main() -> () {
    i = 0
    s = A {
        a: 1.0,
        b: 2.0,
        c: true,
        d: 3,
    }
    n = [s; 10]
    while i < 10 {
        n[i] = A {
            a: i.f32(),
            b: i.f32() + 0.5,
            c: i.f32().rem_euclid(2.0) == 0.0,
            d: i * i,
        }
        i += 1
    }
    takes_an_array(n)
}
"#;
    only_run_func(code)
}

#[cfg(test)]
mod returns_a_fixed_array_in_a_struct {

    use super::*;

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct A {
        a: f32,
        b: f32,
        c: bool,
        d: i64,
    }

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct B {
        i: i64,
        a: bool,
        arr: [A; 10],
        b: bool,
        f: f32,
    }

    #[test]
    fn returns_a_fixed_array_in_a_struct() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
struct B {
    i: i64,
    a: bool,
    arr: [A; 10],
    b: bool,
    f: f32,
}

fn returns_a_fixed_array_in_a_struct() -> (arr: B) {    
    i = 0
    s = A {
        a: 1.0,
        b: 2.0,
        c: true,
        d: 3,
    }
    n = [s; 10]
    while i < 10 {
        n[i] = A {
            a: i.f32(),
            b: i.f32() + 0.5,
            c: i.f32().rem_euclid(2.0) == 0.0,
            d: i * i,
        }
        i += 1
    }
    arr = B {
        i: 123,
        a: true,
        arr: n,
        b: true,
        f: 123.123,
    }
}

struct A {
    a: f32,
    b: f32,
    c: bool,
    d: i64,
}


fn main() -> () {
    n = returns_a_fixed_array_in_a_struct().arr
    n[0].a.println()
    i = 0
    while i < 10 {
        n[i].a.assert_eq(i.f32())
        n[i].b.assert_eq(i.f32() + 0.5)
        n[i].c.assert_eq(i.f32().rem_euclid(2.0) == 0.0)
        n[i].d.assert_eq(i * i)
        i += 1
    }
    returns_a_fixed_array_in_a_struct().i.assert_eq(123)
    returns_a_fixed_array_in_a_struct().a.assert_eq(true)
    returns_a_fixed_array_in_a_struct().b.assert_eq(true)
    returns_a_fixed_array_in_a_struct().f.assert_eq(123.123)
    B::size.println()
    A::size.println()

    //Checking for over/under shoot on mem copy size
    f = returns_a_fixed_array_in_a_struct()
    f.arr[6].a.assert_eq(6.0)
    f.arr[5] = A {
        a: 0.1,
        b: 0.2,
        c: true,
        d: 55,
    }
    f.arr[5].a.assert_eq(0.1)
    f.arr[5].b.assert_eq(0.2)
    f.arr[5].c.assert_eq(true)
    f.arr[5].d.assert_eq(55)
    f.arr[5].a += 1.0
    f.arr[6].a.assert_eq(6.0)
    f.arr = [A {
                a: 0.1,
                b: 0.2,
                c: true,
                d: 55,
            }; 10]
    i = 0
    while i < 10 {
        f.arr[i].a.assert_eq(0.1)
        f.arr[i].b.assert_eq(0.2)
        f.arr[i].c.assert_eq(true)
        f.arr[i].d.assert_eq(55)
        i += 1
    }
}
"#;
        let mut jit = default_std_jit_from_code(code, true)?;
        //let func_ptr = jit.get_func("main")?;
        //let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
        //func();
        let func_ptr = jit.get_func("returns_a_fixed_array_in_a_struct")?;
        let func = unsafe { mem::transmute::<_, extern "C" fn() -> B>(func_ptr) };
        let b = B {
            i: 123,
            a: true,
            arr: [
                A {
                    a: 0.0,
                    b: 0.5,
                    c: true,
                    d: 0,
                },
                A {
                    a: 1.0,
                    b: 1.5,
                    c: false,
                    d: 1,
                },
                A {
                    a: 2.0,
                    b: 2.5,
                    c: true,
                    d: 4,
                },
                A {
                    a: 3.0,
                    b: 3.5,
                    c: false,
                    d: 9,
                },
                A {
                    a: 4.0,
                    b: 4.5,
                    c: true,
                    d: 16,
                },
                A {
                    a: 5.0,
                    b: 5.5,
                    c: false,
                    d: 25,
                },
                A {
                    a: 6.0,
                    b: 6.5,
                    c: true,
                    d: 36,
                },
                A {
                    a: 7.0,
                    b: 7.5,
                    c: false,
                    d: 49,
                },
                A {
                    a: 8.0,
                    b: 8.5,
                    c: true,
                    d: 64,
                },
                A {
                    a: 9.0,
                    b: 9.5,
                    c: false,
                    d: 81,
                },
            ],
            b: true,
            f: 123.123,
        };
        assert_eq!(func(), b);
        Ok(())
    }

    #[test]
    fn returns_a_fixed_array_in_a_struct_inline_issue() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
struct B {
    i: i64,
    a: bool,
    arr: [A; 10],
    b: bool,
    f: f32,
}

fn returns_a_fixed_array_in_a_struct() -> (arr: B) {    
    i = 0
    s = A {
        a: 1.0,
        b: 2.0,
        c: true,
        d: 3,
    }
    n = [s; 10]
    while i < 10 {
        n[i] = A {
            a: i.f32(),
            b: i.f32() + 0.5,
            c: i.f32().rem_euclid(2.0) == 0.0,
            d: i * i,
        }
        i += 1
    }
    arr = B {
        i: 123,
        a: true,
        arr: n,
        b: true,
        f: 123.123,
    }
}

struct A {
    a: f32,
    b: f32,
    c: bool,
    d: i64,
}

fn main() -> () {
    returns_a_fixed_array_in_a_struct()
    returns_a_fixed_array_in_a_struct()
    returns_a_fixed_array_in_a_struct()
    returns_a_fixed_array_in_a_struct()
    returns_a_fixed_array_in_a_struct()
}
"#;
        default_std_jit_from_code(code, true)?;

        Ok(())
    }

    #[repr(C)]
    #[derive(Debug)]
    struct C {
        arr: [i64; 10],
    }

    #[test]
    fn returns_a_fixed_array_in_a_struct_basic() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
struct C {
    arr: [i64; 10],
}

fn returns_a_fixed_array_in_a_struct() -> (struct_of_arr: C) {    
    i = 0
    n = [1; 10]
    while i < 10 {
        n[i] = i
        i += 1
    }
    struct_of_arr = C {
        arr: n,
    }
    i = 0
    while i < 10 {
        struct_of_arr.arr[i].assert_eq(i)
        i += 1
    }
}

fn main() -> () {
    struct_of_arr = returns_a_fixed_array_in_a_struct()
    struct_of_arr.arr[0].println()
    i = 0
    while i < 10 {
        struct_of_arr.arr[i].assert_eq(i)
        struct_of_arr.arr[i] += 1
        i += 1
    }
}

"#;
        let mut jit = default_std_jit_from_code(code, true)?;
        let func_ptr = jit.get_func("returns_a_fixed_array_in_a_struct")?;
        let func = unsafe { mem::transmute::<_, extern "C" fn() -> C>(func_ptr) };
        let b = func();
        assert_eq!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9], b.arr);
        let func_ptr = jit.get_func("main")?;
        let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
        func();
        Ok(())
    }
}

#[test]
fn modify_fixed_array_arg() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn modifies_an_array(arr: [i64; 10]) -> () {    
    i = 0
    while i < 10 {
        arr[i] = i
        i += 1
    }
}

fn modifies_a_unbounded_array(arr: &[i64]) -> () {    
    i = 0
    while i < 10 {
        arr[i] = i
        i += 1
    }
}

fn main() -> () {
    n = [1; 10]
    modifies_an_array(n)
    i = 0
    while i < 10 {
        n[i].assert_eq(i)
        n[i] += 1
        i += 1
    }
}

"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let mut arr = [1i64; 10];
    let func_ptr = jit.get_func("modifies_an_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [i64; 10])>(func_ptr) };
    func(&mut arr);
    assert_eq!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9], arr);
    let mut arr = [1i64; 10];
    let func_ptr = jit.get_func("modifies_a_unbounded_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [i64; 10])>(func_ptr) };
    func(&mut arr);
    assert_eq!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9], arr);
    let func_ptr = jit.get_func("main")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn()>(func_ptr) };
    func();
    Ok(())
}

#[test]
fn modify_fixed_array_assign_to_arg() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn modifies_an_array(arr: [i64; 10]) -> () {    
    arr = [2; 10]
}

"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let mut arr = [1i64; 10];
    let func_ptr = jit.get_func("modifies_an_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [i64; 10])>(func_ptr) };
    func(&mut arr);
    assert_eq!([2i64; 10], arr);
    Ok(())
}

#[test]
fn nested_fixed_array() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn modifies_an_array(arr: [[i64; 10]; 10]) -> () {    
    arr = [[2; 10]; 10]
    a = arr[0]
    a[0] = 5
    //TODO normal multidimensional array access: arr[0][0] and (arr[0])[0]
}

"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let mut arr = [[1i64; 10]; 10];
    let func_ptr = jit.get_func("modifies_an_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [[i64; 10]; 10])>(func_ptr) };
    func(&mut arr);
    let mut cmp_array = [[2i64; 10]; 10];
    cmp_array[0][0] = 5;
    assert_eq!(cmp_array, arr);
    Ok(())
}

#[repr(C)]
#[derive(Debug)]
struct B {
    arr: [i64; 10],
}

#[test]
fn struct_macro_2() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
struct B {
    arr: [i64; 10],
}
fn make_b(a: [i64; 10]) -> (r: B) {
    r = B {
        arr: a,
    }
}
fn modifies_an_array(b: B) -> () {   
    b.arr[0] = 5
    c = make_b(b.arr)
    c.arr[0] += 1 * 2
    a = b.arr
    a[1] += 1
}
"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let mut arr = [1i64; 10];
    arr[5] = 5;
    let mut b = B { arr };
    let func_ptr = jit.get_func("modifies_an_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut B)>(func_ptr) };
    func(&mut b);
    assert_eq!(b.arr, [5, 2, 1, 1, 1, 5, 1, 1, 1, 1,]);
    Ok(())
}

#[test]
fn inline_function() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

inline fn add(x, y) -> (z) {
    f = x * y
    z = x + y * f
}

fn main() -> () {    
    a = 5.0
    b = 6.0
    c = add(a, b)
    c.println()
}

"#;
    only_run_func(code)
}

///////////////////////////////////////////////////////////////////////////////
// TODO come up with a way to run all the tests both with and without inlining
///////////////////////////////////////////////////////////////////////////////

#[test]
fn editor_like_example() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

struct ProcessState {
    filter_l: Filter,
    filter_r: Filter,
}

fn init_process_state(state: ProcessState) -> () {
    filter_l = Filter { ic1eq: 0.0, ic2eq: 0.0, }
    filter_r = Filter { ic1eq: 0.0, ic2eq: 0.0, }
    state = ProcessState {
        filter_l: filter_l,
        filter_r: filter_r,
    }
}

fn process(params: SarusDSPModelParams, audio: AudioData, 
           state: ProcessState, dbg: Debugger) -> () {
    i = 0
    left = audio.in_left
    right = audio.in_right

    while i < audio.len {
        highshelf = Coefficients::highshelf(
            audio.sample_rate,
            1.0,
            1.0,
            1.0
        )
        left[i] = state.filter_l.process(left[i], highshelf)
        right[i] = state.filter_r.process(right[i], highshelf)
        audio.out_left[i] = left[i]
        audio.out_right[i] = right[i]
        i += 1
    }
}

struct Filter {
    ic1eq,
    ic2eq,
}

inline fn process(self: Filter, audio, c: Coefficients) -> (audio_out) {
    v3 = audio - self.ic2eq
    v1 = c.a1 * self.ic1eq + c.a2 * v3
    v2 = self.ic2eq + c.a2 * self.ic1eq + c.a3 * v3
    self.ic1eq = 2.0 * v1 - self.ic1eq
    self.ic2eq = 2.0 * v2 - self.ic2eq
    audio_out = c.m0 * audio + c.m1 * v1 + c.m2 * v2
}

struct Coefficients { a1, a2, a3, m0, m1, m2, }

inline fn Coefficients::highshelf(cutoff_hz, gain_db, q_value, sample_rate) -> (coeffs: Coefficients) {
    cutoff_hz = cutoff_hz.min(sample_rate * 0.5)
    a = (10.0).powf(gain_db / 40.0)
    g = (PI * cutoff_hz / sample_rate).tan() * a.sqrt()
    k = 1.0 / q_value
    a1 = 1.0 / (1.0 + g * (g + k))
    a2 = g * a1
    a3 = g * a2
    m0 = a * a
    m1 = k * (1.0 - a) * a
    m2 = 1.0 - a * a
    coeffs = Coefficients { a1: a1, a2: a2, a3: a3, m0: m0, m1: m1, m2: m2, }
}

struct AudioData { in_left: &[f32], in_right: &[f32], out_left: &[f32], out_right: &[f32], len: i64, sample_rate: f32, }
struct Ui { ui: &, }
struct Debugger {}
struct SarusUIModelParams { p1: f32, p2: f32, p3: f32, p4: f32, p5: f32, p6: f32, p7: f32, p8: f32, }
struct SarusDSPModelParams { p1: &[f32], p2: &[f32], p3: &[f32], p4: &[f32], p5: &[f32], p6: &[f32], p7: &[f32], p8: &[f32], }

"#;
    default_std_jit_from_code(code, true)?;
    Ok(())
}

#[test]
fn if_then_else_if() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn float_to_int(a: f32) -> (c: i64) {
    c = 0
    n = 0
    if a == 1.0 {
        n = 1
        c = 1
    } else if a == 2.0 {
        n = 2
        c = 2
    } else if a == 3.0 {
        n = 3
        c = 3
    } else if a == 4.0 {
        n = 4
        c = 4
    } else if a == 5.0 {
        n = 5
        c = 5
    }
    c.assert_eq(n)
}
fn main(a: f32) -> () {
    float_to_int(0.0).assert_eq(0)
    float_to_int(1.0).assert_eq(1)
    float_to_int(2.0).assert_eq(2)
    float_to_int(3.0).assert_eq(3)
    float_to_int(4.0).assert_eq(4)
    float_to_int(5.0).assert_eq(5)
    float_to_int(6.0).assert_eq(0)
}
"#;
    only_run_func(code)
}

#[test]
fn if_then_else_if_else() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn float_to_int(a: f32) -> (c: i64) {
    c = if a == 1.0 {
        1
    } else if a == 2.0 {
        2
    } else if a == 3.0 {
        3
    } else if a == 4.0 {
        4
    } else if a == 5.0 {
        5
    } else { 
        0
    }
}
fn float_to_int2(a: f32) -> (c: i64) {
    n = 0
    c = if a == 1.0 {
        n = 1
        1
    } else if a == 2.0 {
        n = 2
        2
    } else if a == 3.0 {
        n = 3
        3
    } else if a == 4.0 {
        n = 4
        4
    } else if a == 5.0 {
        n = 5
        5
    } else { 
        n = 0
        0
    }
    c.assert_eq(n)
}
fn main(a: f32) -> () {
    float_to_int(0.0).assert_eq(0)
    float_to_int(1.0).assert_eq(1)
    float_to_int(2.0).assert_eq(2)
    float_to_int(3.0).assert_eq(3)
    float_to_int(4.0).assert_eq(4)
    float_to_int(5.0).assert_eq(5)
    float_to_int(6.0).assert_eq(0)
    float_to_int2(0.0).assert_eq(0)
    float_to_int2(1.0).assert_eq(1)
    float_to_int2(2.0).assert_eq(2)
    float_to_int2(3.0).assert_eq(3)
    float_to_int2(4.0).assert_eq(4)
    float_to_int2(5.0).assert_eq(5)
    float_to_int2(6.0).assert_eq(0)
}
"#;
    only_run_func(code)
}

fn get_test_dir() -> PathBuf {
    env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
}

#[test]
fn test_include() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
include "./resources/include_test.sarus"

fn main() -> () {    
    a = 5.0
    b = 6.0
    c = add(a, b)
    c.println()
}

"#;

    let (ast, file_index_table) = parse_with_context(code, &get_test_dir().join("test.sarus"))?;
    only_run_func_with_importer(ast, Some(file_index_table), |_, _| {})
}

#[test]
fn test_include_redundant() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
include "./resources/include_test.sarus"
include "./resources/include_test2.sarus" //Should be skipped

fn main() -> () {    
    a = 5.0
    b = 6.0
    c = add(a, b)
    c.println()
}

"#;
    let (ast, file_index_table) = parse_with_context(code, &get_test_dir().join("test.sarus"))?;
    only_run_func_with_importer(ast, Some(file_index_table), |_, _| {})
}

//#[cfg(test)]
mod inline_closures {

    use super::*;

    #[test]
    fn basic() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
fn main() -> () {
    add|| -> () {
        c = c * 2.0
    }
    stuff|e| -> (f) {
        c = c * 2.0 * e
        f = c * e
    }
    a = 5.0
    b = 6.0
    c = 5.0 + 6.0
    c.assert_eq(11.0)
    add()
    c.assert_eq(22.0)
    j = stuff(3.0)
    c.assert_eq(132.0)
    j.assert_eq(396.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn nested() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
inline fn stuff1(a, b) -> (c, k) {
    c = a + b
    stuff2|d| -> (f) { 
        c = c * d
        f = c + 5.0
    }
    k = stuff2(3.0)
}

fn main() -> () {    
    a = 5.0
    b = 6.0
    h, i = stuff1(a, b)
    h.assert_eq(33.0)
    i.assert_eq(38.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn nested2() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
fn main() -> () {
    j = 0.0
    add|| -> () {
        c = c * 2.0
        stuff|e| -> (f) { 
            c.assert_eq(22.0)
            c = c * 2.0 * e
            c.assert_eq(132.0)
            f = c * e
        }
        j = stuff(3.0)
    }
    a = 5.0
    b = 6.0
    c = 5.0 + 6.0
    c.assert_eq(11.0)
    add()
    c.assert_eq(132.0)
    j.assert_eq(396.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn nested3() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
inline fn stuff1(a, b) -> (c, k) {
    c = a + b
    stuff2|d| -> (f) { 
        c = c * d
        f = c + 5.0
    }
    k = stuff2(3.0)
}

fn main() -> () {
    j = 0.0
    add|| -> () {
        c = c * 2.0
        stuff|e| -> (f) { 
            c.assert_eq(22.0)
            c = c * 2.0 * e
            c.assert_eq(132.0)
            f = c * e
            u, l = stuff1(c, e)
            u.assert_eq(405.0)
            l.assert_eq(410.0)
        }
        j = stuff(3.0)
    }
    a = 5.0
    b = 6.0
    c = 5.0 + 6.0
    c.assert_eq(11.0)
    add()
    c.assert_eq(132.0)
    j.assert_eq(396.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn passing() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e| -> ()) -> () {
    some_closure(n * 5.0)
}
fn main() -> () {
    stuff|e| -> () {
        c *= e
    }
    c = 5.0 + 6.0
    run_some_closure(2.0, stuff)
    c.assert_eq(110.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn passing_with_return() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e| -> (f)) -> (f) {
    f = some_closure(n * 5.0)
}
fn main() -> () {
    stuff|e| -> (f) {
        c *= e
        f = c * 2.0
    }
    c = 5.0 + 6.0
    j = run_some_closure(2.0, stuff)
    c.assert_eq(110.0)
    j.assert_eq(220.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn passing_no_param() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(some_closure: || -> ()) -> () {
    some_closure()
}
fn main() -> () {
    stuff|| -> () {
        c *= 2.0
    }
    c = 5.0 + 6.0
    run_some_closure(stuff)
    c.assert_eq(22.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn anonymous_passing() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |f32| -> ()) -> () {
    some_closure(n * 5.0)
}
fn main() -> () {
    c = 5.0 + 6.0
    run_some_closure(2.0, |e| -> () {c *= e})
    c.assert_eq(110.0)
}
"#;
        only_run_func(code)
    }

    #[test]
    fn anonymous_passing_with_return() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e| -> (f)) -> (f) {
    f = some_closure(n * 5.0)
}
fn main() -> () {
    c = 5.0 + 6.0
    j = run_some_closure(2.0, |e| -> (f) {
        c *= e
        f = c * 2.0
    })
    c.assert_eq(110.0)
    j.assert_eq(220.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn anonymous_passing_with_return_calls_inline() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
inline fn mult(a, b) -> (c) {
    c = a * b
}

inline fn multj(a, b) -> (c) {
    j = 6.0
    c = mult(a, b) + j
    c -= j
}

always_inline fn run_some_closure(n, some_closure: |e| -> (f)) -> (f) {
    f = some_closure(n * 5.0)
}

fn main() -> () {
    c = 5.0 + 6.0
    j = run_some_closure(2.0, |e| -> (f) {
        c *= e
        f = multj(c, 2.0)
    })
    c.assert_eq(110.0)
    j.assert_eq(220.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn anonymous_passing2() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e| -> ()) -> () {
    some_closure(n * 5.0)
}

always_inline fn stuff(n, some_closure2: |e| -> ()) -> () {
    run_some_closure(n, some_closure2)
}

fn main() -> () {
    c = 5.0 + 6.0
    stuff(2.0, |e| -> () {c *= e})
    c.assert_eq(110.0)
}
"#;
        only_run_func(code)
    }

    #[test]
    fn inline_func_has_anonymous_passing() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e| -> ()) -> () {
    some_closure(n * 5.0)
}

inline fn stuff() -> () {
    c = 5.0 + 6.0
    run_some_closure(2.0, |e| -> () {c *= e})
    c.assert_eq(110.0)
}

fn main() -> () {
    stuff()
}
"#;

        only_run_func(code)
    }

    #[test]
    fn inline_func_has_anonymous_passing_with_return_calls_inline() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
inline fn mult(a, b) -> (c) {
    c = a * b
}

inline fn multj(a, b) -> (c) {
    j = 6.0
    c = mult(a, b) + j
    c -= j
}

always_inline fn run_some_closure(n, some_closure: |e| -> (f)) -> (f) {
    f = some_closure(n * 5.0)
}

fn stuff() -> () {
    c = 5.0 + 6.0
    j = run_some_closure(2.0, |e| -> (f) {
        c *= e
        f = multj(c, 2.0)
    })
    c.assert_eq(110.0)
    j.assert_eq(220.0)
}

fn main() -> () {
    stuff()
}
"#;

        only_run_func(code)
    }

    #[test]
    fn closure_in_closure() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
inline fn mult(a, b) -> (c) {
    c = a * b
}

always_inline fn run_some_closure2(n, some_closure: |e| -> ()) -> () {
    some_closure(mult(n, 5.0))
}

always_inline fn run_some_closure(n, some_closure: |e| -> ()) -> () {
    some_closure(n * 5.0)
}

inline fn stuff() -> () {
    c = 5.0 + 6.0
    run_some_closure(2.0, |e| -> () {
        run_some_closure2(3.0, |f| -> () {c *= f})
        c *= e
    })
    c.assert_eq(1650.0)
}

fn main() -> () {
    stuff()
}
"#;

        only_run_func(code)
    }

    #[test]
    fn closure_in_closure_struct() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
struct FloatVal {
    val: f32,
}

inline fn mult(a: FloatVal, b: FloatVal) -> (c: FloatVal) {
    c = FloatVal{ val: a.val * b.val, }
}

always_inline fn run_some_closure2(n: FloatVal, some_closure: |e: FloatVal| -> ()) -> () {
    some_closure( mult(n, FloatVal{ val: 5.0, }) )
}

always_inline fn run_some_closure(n: FloatVal, some_closure: |e: FloatVal| -> ()) -> () {
    n.val *= 5.0
    some_closure(n)
}

inline fn stuff() -> () {
    c = FloatVal { val: 5.0 + 6.0, }
    c.val.assert_eq(11.0)
    run_some_closure(FloatVal{ val: 2.0, }, |e: FloatVal| -> () {
        c.val.assert_eq(11.0)
        run_some_closure2(FloatVal{ val: 3.0, }, |f: FloatVal| -> () {
            c.val.assert_eq(11.0) //"cl2 c ".print() 
            e.val.assert_eq(10.0) //"cl2 e ".print() 
            f.val.assert_eq(15.0) //"cl2 f ".print() 
            c.val *= f.val
        })
        c.val.assert_eq(165.0)
        c.val *= e.val
        c.val.assert_eq(1650.0)
    })
    c.val.assert_eq(1650.0)
}

fn main() -> () {
    stuff()
}
"#;

        only_run_func(code)
    }

    #[test]
    fn closure_takes_closure() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e, cl: |f| -> (j)| -> ()) -> () {
    a = 1.0
    some_closure(n * 5.0, |f| -> (j) {j = f + a})
}

inline fn stuff() -> () {
    c = 5.0 + 6.0
    run_some_closure(2.0, |e, cl: |f| -> (j)| -> () {
        cl(3.0).assert_eq(4.0)
        e.assert_eq(10.0)
        c.assert_eq(11.0)
        c *= e + cl(3.0)
    })
    c.assert_eq(154.0)
}

fn main() -> () {
    stuff()
}
"#;

        only_run_func(code)
    }

    #[test]
    fn use_closure_from_parent_closure_scope() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
always_inline fn run_some_closure(n, some_closure: |e| -> ()) -> () {
    some_closure(n * 5.0)
}

fn main() -> () {
    c = 5.0 + 6.0
    add1|| -> () {c += 1.0}
    run_some_closure(2.0, |e| -> () {c *= e add1()})
    c.assert_eq(111.0)
}
"#;

        only_run_func(code)
    }

    #[test]
    fn use_closure_from_parent_closure_scope2() -> anyhow::Result<()> {
        //setup_logging();
        let code = r#"
fn main() -> () {
    a = 1.0
    single_proc|| -> () {
        a += 1.0
    }

    while_proc | get_coeffs: ||->(c: f32) |->() {
        coeffs = get_coeffs()
        single_proc()        
    }
    
    while_proc(||->(c: f32) {
        c = 5.0
    })
}
"#;

        only_run_func(code)
    }
}

#[test]
fn var_define_checking() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn main() -> () {  
    a = 0.0
    if false {
        a = 5.0
    } else {
        a = 4.0
    }
    a.assert_eq(4.0)
    b = 0.0
    if false {
        b = 5.0
    } else if (true) {
        b = 4.0
    } else {
        b = 3.0
    }
    b.assert_eq(4.0)
}

"#;
    only_run_func(code)
}

#[test]
fn dot_access_conditionals() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {  
    (if true {1.0} else {-1.0}).assert_eq(1.0)
    (if false {1.0} else {-1.0}).assert_eq(-1.0)
    (if false {1.0} else if true {-1.0} else {0.0}).assert_eq(-1.0)
}
"#;
    only_run_func(code)
}

#[test]
fn recursion() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
inline fn fib(n: i64) -> (r: i64) {
    r = if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
fn main() -> () {  
    i = 0
    while i < 10 {
        fib(i).println()
        i += 1
    }
}
"#;
    only_run_func(code)
}

#[test]
fn unary_negative() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn number() -> (y) {
    y = 2.0
}
fn main() -> () {
    a = 5
    b = -a
    b.assert_eq(-5)
    (-b).assert_eq(5)
    c = 5.0
    d = -c
    d.assert_eq(-5.0)
    (-d).assert_eq(5.0)
    e = -number()
    e.assert_eq(-2.0)
    (-number()).assert_eq(-2.0)
    (-(number())).assert_eq(-2.0)
    (4 + -4).assert_eq(0)
    (2 + -4 + 2).assert_eq(0)
    ((2 + -4) + 2).assert_eq(0)
    (2 + (-4 + 2)).assert_eq(0)
    four = 4
    two = 2
    (four + -four).assert_eq(0)
    (two + -four + two).assert_eq(0)
    ((two + -four) + two).assert_eq(0)
    (two + (-four + two)).assert_eq(0)
}
"#;
    only_run_func(code)
}

#[test]
fn expr_array_access() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn arr(n) -> (a: [f32; 10]) {
    a = [n; 10]
}

fn main() -> () { 
    a = [1.0; 10]
    b = (a)[1]
    b.assert_eq(1.0)
    ([2.0; 10])[1].assert_eq(2.0)
    [3.0; 10][1].assert_eq(3.0)
    arr(4.0)[1].assert_eq(4.0)
}
"#;
    only_run_func(code)
}

#[test]
fn declare_var_in_if() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn main() -> () { 
    if true {
        a = 5
        a.assert_eq(5)
    }
}
"#;
    only_run_func(code)
}

#[test]
fn init_process_state() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
struct ProcessState {
    delay_l: [f32; 10000],
    delay_r: [f32; 10000],
}
fn main() -> () {
    state = ProcessState {
        delay_l: [1.234; 10000],
        delay_r: [1.234; 10000],
    }
    state.delay_r[0].assert_eq(1.234)
    state.delay_r[1].assert_eq(1.234)
    state.delay_r[9999].assert_eq(1.234)
    state.delay_l[0].assert_eq(1.234)
    state.delay_l[1].assert_eq(1.234)
    state.delay_l[9999].assert_eq(1.234)

    a = [1.234; 15] //Test array init that does not use loop
    a[0].assert_eq(1.234)
    a[14].assert_eq(1.234)
}

"#;
    only_run_func(code)
}

#[test]
fn assign_to_param_address_without_extra_alloc() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn takes_arr(n: [f32; 5]) -> () {
    n = [1.0; 5]
}

fn main() -> () { 
    a = [0.0; 5]
    a[1].assert_eq(0.0)
    takes_arr(a)
    a[1].assert_eq(1.0)
}
"#;
    only_run_func(code)
}

#[test]
fn basic_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () { 
    a = [0.0; 100]
    a[0] = 0.0
    a[1] = 1.0
    a[2] = 2.0
    a[99] = 99.0
    slice_of_a = a[..]
    sub_slice_of_a1 = a[0..3]
    sub_slice_of_a2 = a[..3]
    sub_slice_of_a3 = a[2..]
    sub_slice_of_a4 = a[1..3]

    slice_of_a[0].assert_eq(0.0)
    slice_of_a[1].assert_eq(1.0)
    slice_of_a[2].assert_eq(2.0)
    slice_of_a[99].assert_eq(99.0)

    sub_slice_of_a1[0].assert_eq(0.0)
    sub_slice_of_a1[1].assert_eq(1.0)
    sub_slice_of_a1[2].assert_eq(2.0)

    sub_slice_of_a2[0].assert_eq(0.0)
    sub_slice_of_a2[1].assert_eq(1.0)
    sub_slice_of_a2[2].assert_eq(2.0)

    sub_slice_of_a3[0].assert_eq(2.0)
    sub_slice_of_a3[99-2].assert_eq(99.0)

    sub_slice_of_a4[0].assert_eq(1.0)
    sub_slice_of_a4[1].assert_eq(2.0)

    a[1] = 10.0
    slice_of_a[1].assert_eq(10.0)

    slice_of_a[2] = 20.0
    a[2].assert_eq(20.0)

    sub_slice_of_a4[0] = 5.0
    sub_slice_of_a4[1] = 6.0

    a[1].assert_eq(5.0)
    a[2].assert_eq(6.0)
}
"#;
    only_run_func(code)
}

#[test]
fn slice_contains_struct() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
struct Point {
    x, y, z,
}

fn main() -> () { 
    a = [Point {
        x:0.0, 
        y:0.0, 
        z:0.0,
    }; 100]
    a[0].y = 0.0
    a[1].y = 1.0
    a[2].y = 2.0
    a[99].y = 99.0
    slice_of_a = a[..]
    a[1].y.assert_eq(1.0)
    sub_slice_of_a1 = a[0..3]
    sub_slice_of_a2 = a[..3]
    sub_slice_of_a3 = a[2..]
    sub_slice_of_a4 = a[1..3]

    slice_of_a[0].y.assert_eq(0.0)
    slice_of_a[1].y.assert_eq(1.0)
    slice_of_a[2].y.assert_eq(2.0)
    slice_of_a[99].y.assert_eq(99.0)

    sub_slice_of_a1[0].y.assert_eq(0.0)
    sub_slice_of_a1[1].y.assert_eq(1.0)
    sub_slice_of_a1[2].y.assert_eq(2.0)

    sub_slice_of_a2[0].y.assert_eq(0.0)
    sub_slice_of_a2[1].y.assert_eq(1.0)
    sub_slice_of_a2[2].y.assert_eq(2.0)

    sub_slice_of_a3[0].y.assert_eq(2.0)
    sub_slice_of_a3[99-2].y.assert_eq(99.0)

    sub_slice_of_a4[0].y.assert_eq(1.0)
    sub_slice_of_a4[1].y.assert_eq(2.0)

    a[1].y = 10.0
    slice_of_a[1].y.assert_eq(10.0)

    slice_of_a[2].y = 20.0
    a[2].y.assert_eq(20.0)

    sub_slice_of_a4[0].y = 5.0
    sub_slice_of_a4[1].y = 6.0

    a[1].y.assert_eq(5.0)
    a[2].y.assert_eq(6.0)
}
"#;
    only_run_func(code)
}

#[test]
fn pass_slice_to_func() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
struct Point {
    x, y, z,
}

fn takes_slices(a: [Point], b: [Point]) -> () {
    a[0].y.assert_eq(0.0)
    a[1].y.assert_eq(1.0)
    a[2].y.assert_eq(2.0)
    a[99].y.assert_eq(99.0)

    b[0].y.assert_eq(1.0)
    b[1].y.assert_eq(2.0)

    a[3].y = 20.0
    b[1].y = 10.0
}

fn main() -> () { 
    a = [Point {
        x:0.0, 
        y:0.0, 
        z:0.0,
    }; 100]
    a[0].y = 0.0
    a[1].y = 1.0
    a[2].y = 2.0
    a[99].y = 99.0
    slice_of_a = a[..]
    sub_slice_of_a1 = a[1..3]

    takes_slices(slice_of_a, sub_slice_of_a1)

    a[3].y.assert_eq(20.0)
    a[2].y.assert_eq(10.0)
}
"#;
    only_run_func(code)
}

#[test]
fn return_slice_from_func() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
inline fn takes_slices(in: [f32]) -> (r: [f32]) {
    r = in
}

fn main() -> () { 
    sl1 = [1.0; 100][..]
    sl = [1.0; 200][..]
    sl1 = takes_slices(sl)
    sl1[5].assert_eq(1.0)
}
"#;
    only_run_func(code)
}

#[test]
fn unsized_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn modifies_an_array(arr: &[i64], len: i64) -> () {   
    arr_slice = arr[..10]
    a = arr_slice[0]
    arr_slice[0] = 5
    arr_slice[9] = 5
}

fn modifies_a_fixed_array(arr: [i64; 10], len: i64) -> () {   
    arr_slice = arr[..]
    a = arr_slice[0]
    arr_slice[0] = 5
    arr_slice[9] = 5
}
"#;
    let mut jit = default_std_jit_from_code(code, true)?;
    let mut arr = [1i64; 10];
    let func_ptr = jit.get_func("modifies_an_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [i64; 10], i64)>(func_ptr) };
    let len = arr.len() as i64;
    func(&mut arr, len);
    assert_eq!(arr, [5, 1, 1, 1, 1, 1, 1, 1, 1, 5,]);
    let func_ptr = jit.get_func("modifies_a_fixed_array")?;
    let func = unsafe { mem::transmute::<_, extern "C" fn(&mut [i64; 10], i64)>(func_ptr) };
    let len = arr.len() as i64;
    func(&mut arr, len);
    assert_eq!(arr, [5, 1, 1, 1, 1, 1, 1, 1, 1, 5,]);
    Ok(())
}

#[test]
fn slice_of_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn main() -> () { 
    sl = [0.0; 100][..]
    sl[1] = 1.0
    sl[2] = 2.0
    sl[3] = 3.0
    sl2 = sl[..]
    sl2[1].assert_eq(1.0)
    sl2[2].assert_eq(2.0)
    sl2[3].assert_eq(3.0)
    sl3 = sl2[1..]
    sl3[1].assert_eq(2.0)
    sl3[2].assert_eq(3.0)
    sl3[3].assert_eq(0.0)
    sl4 = sl3[1..]
    sl4[1].assert_eq(3.0)
    sl4[2].assert_eq(0.0)
    sl4[3].assert_eq(0.0)
    (sl4[0..1])[0].assert_eq(2.0)
    (sl4[1..2])[0].assert_eq(3.0)

    sl5 = sl[1..2]
    sl5[0].assert_eq(1.0)
    sl6 = sl[1..5]
    sl6[0].assert_eq(1.0)
    sl6[1].assert_eq(2.0)
    sl6[2].assert_eq(3.0)

    a = [0, 1, 2, 3, 4, 5]
    b = a[0..2]
    b.len().assert_eq(2)
    b.cap().assert_eq(6)
    
    c = b[..6]
    c.len().assert_eq(6)
    c.cap().assert_eq(6)
    c[4].assert_eq(4)
    c[5].assert_eq(5)
}
"#;
    only_run_func(code)
}

#[test]
fn direct_array_literal() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {
    arr = [0.0, 0.5+0.5, 2.0, 3.0, 4.0]
    arr[0].assert_eq(0.0)
    arr[1].assert_eq(1.0)
    arr[2].assert_eq(2.0)
    arr[3].assert_eq(3.0)
    arr[4].assert_eq(4.0)
}
"#;
    only_run_func(code)
}

#[test]
fn push_onto_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"

fn main() -> () {
    arr = [0.0; 100]
    sl = arr[0..0]
    sl.cap().assert_eq(arr.len())
    sl.len().assert_eq(0)
    sl.push(1.0)
    sl.len().assert_eq(1)
    sl.push(2.0)
    sl.len().assert_eq(2)
    sl.pop().assert_eq(2.0)
    sl.len().assert_eq(1)
    sl.pop().assert_eq(1.0)
    sl.len().assert_eq(0)
    (sl.unsized())[0].assert_eq(arr[0])
    (sl.unsized())[1].assert_eq(arr[1])
    (sl.unsized())[2].assert_eq(arr[2])
}
"#;
    only_run_func(code)
}

#[test]
fn append_to_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {
    arr = [2.0; 100]
    sl = arr[0..3]
    sl.append([1.0;3])
    sl.len().assert_eq(6)
    sl[2].assert_eq(2.0)
    sl[3].assert_eq(1.0)
    sl[4].assert_eq(1.0)
    sl[5].assert_eq(1.0)
    (sl[0..sl.len()+1])[6].assert_eq(2.0)
    sl.append([6.0,7.0,8.0][..])
    sl.len().assert_eq(9)
    sl[5].assert_eq(1.0)
    sl[6].assert_eq(6.0)
    sl[7].assert_eq(7.0)
    sl[8].assert_eq(8.0)
    (sl[0..sl.len()+1])[9].assert_eq(2.0)
}
"#;
    only_run_func(code)
}

#[test]
fn append_slice_of_structs_to_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
struct Point { x, y, z, }

fn main() -> () {
    arr = [Point { x: 0.0, y: 0.0, z: 0.0, }; 100]
    sl = arr[0..0]
    sl.len().assert_eq(0)
    to_append = [
        Point { x: 1.0, y: 2.0, z: 3.0, },
        Point { x: 4.0, y: 5.0, z: 6.0, },
        Point { x: 7.0, y: 8.0, z: 9.0, }
    ]
    sl.append(to_append)
    sl.len().assert_eq(3)
    sl[0].x.assert_eq(1.0)
    sl[0].y.assert_eq(2.0)
    sl[0].z.assert_eq(3.0)
    sl[1].x.assert_eq(4.0)
    sl[1].y.assert_eq(5.0)
    sl[1].z.assert_eq(6.0)
    sl[2].x.assert_eq(7.0)
    sl[2].y.assert_eq(8.0)
    sl[2].z.assert_eq(9.0)
}
"#;
    only_run_func(code)
}

#[test]
fn returns_a_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
inline fn a_slice(a) -> (b: [f32]) {
    b = [a; 100][..]
}
fn main() -> () {
    b = a_slice(5.0)
    b[20].assert_eq(5.0)
}
"#;
    only_run_func(code)
}

#[test]
fn scoped_vars() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {
    if true {
        a = 1.0 //does not live outside if statement
    }
    a = true
}
"#;
    only_run_func(code)
}

#[test]
fn u8_math() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {
    a = 0
    b = 255
    c = 100
    d = 5
    ua = (0).u8()
    ub = (255).u8()
    uc = (100).u8()
    ud = (5).u8()
    (ua+ub).assert_eq((a+b).u8())
    (ua-uc).assert_eq((a-c).u8())
    (ua/uc).assert_eq((a/c).u8())
    (ua*uc).assert_eq((a*c).u8())
    (ud*ud).assert_eq((d*d).u8())
    (ua+ub).i64().assert_eq(a+b)
    (ua/uc).i64().assert_eq(a/c)
    (ua*uc).i64().assert_eq(a*c)
    (ud*ud).i64().assert_eq(d*d)
    ua.assert_eq(0u8)
    ub.assert_eq(255u8)
    uc.assert_eq(100u8)
    ud.assert_eq(5u8)
    (0u8+255u8).assert_eq((0+255).u8())
    (0u8-100u8).assert_eq((0-100).u8())
    (0u8/100u8).assert_eq((0/100).u8())
    (0u8*100u8).assert_eq((0*100).u8())
    (5u8*5u8).assert_eq((5*5).u8())
}
"#;
    only_run_func(code)
}

extern "C" fn rust_check_slice(a: sarus_std_lib::SarusSlice<u8>) {
    assert_eq!(a.len(), 43);
    assert_eq!(a.cap(), 1000);
    assert_eq!(
        std::str::from_utf8(a.slice()).unwrap(),
        "Hello World !\"#$%&'()*+,-./0123456789:;<=>?"
    );
}

#[test]
fn extern_func_slice() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
    extern fn rust_check_slice(a: [u8]) -> () {}

    fn main() -> () {
        a = [0u8;1000][0..0]
        a.append("Hello")
        a.append(" ")
        a.append("World")
        i = 0u8
        while i < 32u8 {
            a.push(i + 32u8)
            i += 1u8
        }
        rust_check_slice(a)
    }
"#;
    let ast = parse(code)?;
    only_run_func_with_importer(ast, None, |_ast, jit_builder| {
        jit_builder.symbols([("rust_check_slice", rust_check_slice as *const u8)]);
    })
}

#[test]
fn while_iter_block() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {

    arr = [0; 10]
    
    i = 0 while i < 10 { i += 1 }:{arr[i] = i}

    i = 0 while i < 10 
    { i += 1 } : {
        arr[i].assert_eq(i)
    }

    i = 0 while i < 10 { i += 1 } : {
        arr[i].assert_eq(i)
    }

    a = 5

    i = 0 while i < 10 { 
        a = i + 1
        i += 1 
    } : {
        arr[i].assert_eq(i)
    }
    a.assert_eq(10)

    i = 0 while i < 10 {
        arr[i].assert_eq(i)
        i += 1
    }
}
"#;
    only_run_func(code)
}

#[test]
fn while_break() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {

    arr = [0; 10]
    a = 5
    
    i = 0 while i < 10 {
        j = 0 
        while j < 10 {
            if i > 5 && j > 5 {
                "!!!!".println()
                a = j
                break
            }
            j += 1
        }
        if i > 8 {
            break
        }
        i += 1
    }

    a.assert_eq(6)
    i.assert_eq(9)

    i = 0 while true {i+=1 if i > 3 {break}} : {
        a = 2
    }
    i.assert_eq(4)

}
"#;
    only_run_func(code)
}

#[test]
fn while_continue() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn main() -> () {
    arr = [0; 10]

    a = 0
    i = 0 while i < 10 {i += 1} : {
        if i > 5 {
            continue
        }
        a = i
    }
    i.assert_eq(10)
    a.assert_eq(5)

    a = 0
    i = 0 while i < 10 {i += 1} : {
        if i > 5 {
            continue
        } else {
            continue
        }
        a = i
    }
    i.assert_eq(10)
    a.assert_eq(0)

    a = 0
    i = 0 while i < 10 {i += 1} : {
        if i > 5 {
            j = 0 while j < 10 {j += 1} : {
                if j > 5 {
                    a += 1
                    break
                } else {
                    a += 2
                    continue
                }
            }
        } else {
            continue
        }
        a += i
    }
    i.assert_eq(10)
    a.assert_eq(82)

}
"#;
    only_run_func(code)
}

#[test]
fn early_return() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn other2(a: i64) -> (b: i64) {
    b = 0
    if a > 5 {
        b = 3
        return
    } else {
        b = 4
        return
    }
    b = a
}

fn other(a: i64) -> (b: i64) {
    b = 0
    if a > 5 {
        return
    }
    b = a
}

fn main() -> () {
    other(6).assert_eq(0)
    other(4).assert_eq(4)
    other2(1).assert_eq(4)
    other2(6).assert_eq(3)
}
"#;
    only_run_func(code)
}

#[cfg(test)]
mod string_tests {

    use super::*;
    #[test]
    fn find() -> anyhow::Result<()> {
        only_run_func(
            r#"
fn main() -> () {
    "hello".find("l").assert_eq(2)
    "hello".find("x").assert_eq(-1)
    "ประเทศไทย中华Việt Nam".find("华").assert_eq(30)
}
        "#,
        )
    }

    #[test]
    fn rfind() -> anyhow::Result<()> {
        only_run_func(
            r#"
fn main() -> () {
    "hello".rfind("l").assert_eq(3)
    "hello".rfind("x").assert_eq(-1)
    "ประเทศไทย中华Việt Nam".rfind("华").assert_eq(30)
}
        "#,
        )
    }

    #[test]
    fn starts_with() -> anyhow::Result<()> {
        only_run_func(
            r#"
fn main() -> () {
    "".starts_with("").assert_eq(true)
    "abc".starts_with("").assert_eq(true)
    "abc".starts_with("a").assert_eq(true)
    "a".starts_with("abc").assert_eq(false)
    "".starts_with("abc").assert_eq(false)
    "ödd".starts_with("-").assert_eq(false)
    "ödd".starts_with("öd").assert_eq(true)
}
        "#,
        )
    }
    #[test]
    fn ends_with() -> anyhow::Result<()> {
        only_run_func(
            r#"
fn main() -> () {
    "".ends_with("").assert_eq(true)
    "abc".ends_with("").assert_eq(true)
    "abc".ends_with("c").assert_eq(true)
    "a".ends_with("abc").assert_eq(false)
    "".ends_with("abc").assert_eq(false)
    "ddö".ends_with("-").assert_eq(false)
    "ddö".ends_with("dö").assert_eq(true)
}
        "#,
        )
    }
}
#[cfg(test)]
mod enum_tests {

    use super::*;
    #[test]
    fn enums_num() -> anyhow::Result<()> {
        //setup_logging();
        only_run_func(
            r#"
enum Num {
    int: i64,
    float: f32,
    byte: u8,
}

fn f32(self: Num) -> (n: f32) {
    n = 0.0
    if self.type == Num::int {
        n = self.int.f32()
    } else if self.type == Num::float {
        n = self.float
    } else {
        n = self.byte.f32()
    }
}

fn println(self: Num) -> () {
    if self.type == Num::int {
        self.int.println()
    } else if self.type == Num::float {
        self.float.println()
    } else {
        self.byte.println()
    }
}

fn assert_eq(self: Num, other: Num) -> () {
    self.unify(other)
    if self.type == Num::int {
        self.int.assert_eq(other.int)
    } else if self.type == Num::float {
        self.float.assert_eq(other.float)
    } else {
        self.byte.assert_eq(other.byte)
    }
}

fn unify(self: Num, other: Num) -> () {
    if self.type == other.type {
        return
    } else {
        self = Num::float(self.f32())
        other = Num::float(other.f32())
    }
}

fn add(self: Num, other: Num) -> (z: Num) {
    self.unify(other)
    z = Num::int(100)
    if self.type == Num::int {
        z = Num::int(self.int + other.int)
    } else if self.type == Num::float {
        z = Num::float(self.float + other.float)
    } else {
        z = Num::byte(self.byte + other.byte)
    }
}

fn main() -> () {
    a = Num::int(5)
    b = Num::float(5.0)
    Num::int(5).add(Num::int(5)).assert_eq(Num::int(10))
    Num::float(5.0).add(Num::float(5.0)).assert_eq(Num::float(10.0))
    Num::int(5).add(Num::float(5.0)).assert_eq(Num::float(10.0))
    Num::byte(5u8).add(Num::byte(5u8)).assert_eq(Num::byte(10u8))
    Num::byte(5u8).add(Num::float(5.0)).assert_eq(Num::float(10.0))
}
    "#,
        )
    }

    #[test]
    fn enums_traditional() -> anyhow::Result<()> {
        only_run_func(
            r#"
enum Num {
    int,
    float,
    byte,
}

fn main() -> () {
    Num::int().type.assert_eq(Num::int)
    Num::float().type.assert_eq(Num::float)
    Num::byte().type.assert_eq(Num::byte)
    a = Num::int()
    b = Num::float()
    c = Num::byte()
    a.type.assert_eq(0)
    b.type.assert_eq(1)
    c.type.assert_eq(2)
}
    "#,
        )
    }

    #[test]
    fn enums_mixed() -> anyhow::Result<()> {
        only_run_func(
            r#"
enum Num {
    int,
    float,
    something: f32,
    byte,
    something_else: i64,
}

fn main() -> () {
    Num::int().type.assert_eq(Num::int)
    Num::float().type.assert_eq(Num::float)
    Num::something(100.0).type.assert_eq(Num::something)
    Num::byte().type.assert_eq(Num::byte)
    Num::something_else(200).type.assert_eq(Num::something_else)
    a = Num::int()
    b = Num::float()
    c = Num::something(100.0)
    d = Num::byte()
    e = Num::something_else(200)

    a.type.assert_eq(0)
    b.type.assert_eq(1)

    c.type.assert_eq(2)
    c.something.assert_eq(100.0)

    d.type.assert_eq(3)

    e.type.assert_eq(4)
    e.something_else.assert_eq(200)
    
}
    "#,
        )
    }

    #[test]
    fn rust_by_example_enum() -> anyhow::Result<()> {
        //adapted from: https://doc.rust-lang.org/rust-by-example/custom_types/enum.html
        only_run_func(
            r#"
// Create an `enum` to classify a web event. Note how both
// names and type information together specify the variant:
// `page_load != page_unload` and `key_press: [u8] != paste: [u8]`.
// Each is different and independent.
struct Click {
    x: i64, 
    y: i64,
}

enum WebEvent {
    // An `enum` may either be `unit-like`,
    page_load,
    page_unload,
    // or have associated types
    key_press: [u8],
    paste: [u8],
    click: Click,
}

// A function which takes a `WebEvent` enum as an argument and
// returns nothing.
fn inspect(event: WebEvent) -> () {
    if event.type == WebEvent::page_load {
        "page loaded".println()
    } else if event.type == WebEvent::page_unload {
        "page unloaded".println()
    } else if event.type == WebEvent::key_press {
        "pressed ".print() event.key_press.println()
    } else if event.type == WebEvent::paste {
        "pasted ".print() event.paste.println()
    } else if event.type == WebEvent::click {
        "clicked at ".print() 
        "x= ".print() event.click.x.print()
        ", y= ".print() event.click.x.print()
        ".".println()
    } 
}

fn main() -> () {
    pressed = WebEvent::key_press("x"[..])
    pasted = WebEvent::paste("my text"[..])
    click = WebEvent::click(Click{ x: 20, y: 80, }) 
    load = WebEvent::page_load()
    unload = WebEvent::page_unload()

    inspect(pressed)
    inspect(pasted)
    inspect(click)
    inspect(load)
    inspect(unload)

    pressed.type.assert_eq(WebEvent::key_press)
    pasted.type.assert_eq(WebEvent::paste)
    click.type.assert_eq(WebEvent::click)
    load.type.assert_eq(WebEvent::page_load)
    unload.type.assert_eq(WebEvent::page_unload)
}
    "#,
        )
    }
}

#[test]
fn loop_lifetime() -> anyhow::Result<()> {
    only_run_func(
        r#"
fn main() -> () {
    a = [0;10000][..]
    i = 0 while i < 5 {i+=1} : {
        c = [1.0;10000]
        if i == 0 {
            b = [2;10000] //(while)inner, copy
            a = b[..] //TODO compiler error
        }
        d = [1.0;10000]
    }
    e = [1.0;10000]
    //a[5].println()
}
"#,
    )
}

#[test]
fn loop_lifetime_simple() -> anyhow::Result<()> {
    //setup_logging();
    only_run_func(
        r#"
fn main() -> () {
    //a = [0;100][..]
    i = 0 while i < 1 {i+=1} : {
        d = [1.0;100]
    }
    //(1.0).println()
    //a = 1.0
    //i = 0 while i < 100000000 {i+=1} : {
    //    a *= (1.0).sin()
    //}
    //(2.0).println()

}
"#,
    )
}

#[test]
fn deep_stack_while_loop2() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn blank() -> () {
}

fn other() -> () {
    a = [1234; 10000]
}

fn first() -> () {
    
    blank()
    blank()
    other()
    a = [1234; 10000]
    other()
}

fn main() -> () { 
    i = 0 while i <= 10 {
        if i == 0 {
        }
        a = [1234; 10000]
        b = [1234; 10000]
        i += 1
    }
    if true {
        
    }
    first()
}
"#;
    only_run_func(code)
}

#[test]
fn deep_stack_while_loop3() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn blank() -> () {
}

fn other() -> () {
    a = [1234; 10000]
}

fn first() -> () {
    
    blank()
    blank()
    other()
    a = [1234; 10000]
    other()
}

fn main() -> () { 
     i = 0 while i <= 10 {
        first()
        i += 1
    }
    
}
"#;
    only_run_func(code)
}

#[test]
fn deep_stack_while_loop_early_return() -> anyhow::Result<()> {
    //setup_logging();
    let code = r#"
fn other(return_early: bool) -> () {
    a = [1234; 10000]
    i = 0 while i <= 5 {i+=1}:{
        b = [1234; 10000]
        if return_early {
            return
        }
    }
}

fn main() -> () { 
    other(true)
    other(false)
}
"#;
    only_run_func(code)
}
