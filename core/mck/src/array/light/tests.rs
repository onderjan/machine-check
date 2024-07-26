use super::LightArray;

#[test]
fn map_inplace_indexed() {
    let mut a = LightArray::new_filled(0, 7);

    a.map_inplace_indexed(2, Some(4), |_| 0xCAFE);
    assert_eq!(a[0], 0);
    assert_eq!(a[1], 0);
    assert_eq!(a[2], 0xCAFE);
    assert_eq!(a[3], 0xCAFE);
    assert_eq!(a[4], 0xCAFE);
    assert_eq!(a[5], 0);
    assert_eq!(a[6], 0);

    a.map_inplace_indexed(3, Some(5), |v| v + 1);
    assert_eq!(a[0], 0);
    assert_eq!(a[1], 0);
    assert_eq!(a[2], 0xCAFE);
    assert_eq!(a[3], 0xCAFF);
    assert_eq!(a[4], 0xCAFF);
    assert_eq!(a[5], 1);
    assert_eq!(a[6], 0);
}

#[test]
fn map_write() {
    let mut a = LightArray::new_filled(0, 7);

    a.write(2, 0xCAFE);
    assert_eq!(a[0], 0);
    assert_eq!(a[1], 0);
    assert_eq!(a[2], 0xCAFE);
    assert_eq!(a[3], 0);
    assert_eq!(a[4], 0);
    assert_eq!(a[5], 0);
    assert_eq!(a[6], 0);

    a.write(3, 0xBEEF);
    assert_eq!(a[0], 0);
    assert_eq!(a[1], 0);
    assert_eq!(a[2], 0xCAFE);
    assert_eq!(a[3], 0xBEEF);
    assert_eq!(a[4], 0);
    assert_eq!(a[5], 0);
    assert_eq!(a[6], 0);
}
