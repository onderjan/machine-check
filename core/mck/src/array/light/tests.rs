use crate::{concr::UnsignedBitvector, misc::CBound};

use super::LightArray;

// easily create index
macro_rules! i {
    ($a:tt) => {
        UnsignedBitvector::new($a, CBound)
    };
}

#[test]
fn map_inplace_indexed() {
    let mut a: LightArray<UnsignedBitvector<CBound<3>>, i32> = LightArray::new_filled(CBound, 0);

    a.map_inplace_indexed(i![2], Some(i![4]), |_| 0xCAFE);
    assert_eq!(a[i![0]], 0);
    assert_eq!(a[i![1]], 0);
    assert_eq!(a[i![2]], 0xCAFE);
    assert_eq!(a[i![3]], 0xCAFE);
    assert_eq!(a[i![4]], 0xCAFE);
    assert_eq!(a[i![5]], 0);
    assert_eq!(a[i![6]], 0);

    a.map_inplace_indexed(i![3], Some(i![5]), |v| v + 1);
    assert_eq!(a[i![0]], 0);
    assert_eq!(a[i![1]], 0);
    assert_eq!(a[i![2]], 0xCAFE);
    assert_eq!(a[i![3]], 0xCAFF);
    assert_eq!(a[i![4]], 0xCAFF);
    assert_eq!(a[i![5]], 1);
    assert_eq!(a[i![6]], 0);
}

#[test]
fn map_write() {
    let mut a: LightArray<UnsignedBitvector<CBound<3>>, i32> = LightArray::new_filled(CBound, 0);

    a.write(i![2], 0xCAFE);
    assert_eq!(a[i![0]], 0);
    assert_eq!(a[i![1]], 0);
    assert_eq!(a[i![2]], 0xCAFE);
    assert_eq!(a[i![3]], 0);
    assert_eq!(a[i![4]], 0);
    assert_eq!(a[i![5]], 0);
    assert_eq!(a[i![6]], 0);

    a.write(i![3], 0xBEEF);
    assert_eq!(a[i![0]], 0);
    assert_eq!(a[i![1]], 0);
    assert_eq!(a[i![2]], 0xCAFE);
    assert_eq!(a[i![3]], 0xBEEF);
    assert_eq!(a[i![4]], 0);
    assert_eq!(a[i![5]], 0);
    assert_eq!(a[i![6]], 0);
}
