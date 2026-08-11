use ndarray::{ArcArray, IxDyn, Slice, SliceInfo, SliceInfoElem, s};

fn main() {
    let mut arr = ArcArray::zeros(IxDyn(&[10, 10]));
    for i in 0..10 {
        for j in 0..10 {
            arr[[i, j]] = (i * 10 + j) as f64;
        }
    }

    // Dynamic slicing using SliceInfoElem
    let slice_info = vec![
        SliceInfoElem::Slice {
            start: 0,
            end: Some(5),
            step: 1,
        },
        SliceInfoElem::Slice {
            start: 2,
            end: Some(4),
            step: 1,
        },
    ];
    // Wait, SliceInfoElem requires conversion to SliceInfo, but SliceInfo is generic.
    // ndarray provides `arr.slice_axis()` for dynamic slicing, but for multiple axes,
    // it's easier to chain `slice_axis` or use `slice_each_axis`?

    let mut current_slice = arr.clone();
    current_slice = current_slice.slice_axis(ndarray::Axis(0), Slice::from(0..5));
    current_slice = current_slice.slice_axis(ndarray::Axis(1), Slice::from(2..4));

    println!("Dynamic slice shape: {:?}", current_slice.shape());
    println!(
        "Value at [0, 0] in dynamic slice: {}",
        current_slice[[0, 0]]
    );
}
