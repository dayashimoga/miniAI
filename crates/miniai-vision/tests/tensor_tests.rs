use miniai_vision::{ImageTensor, VisionError, IMAGENET_MEAN, IMAGENET_STD};

#[test]
fn test_image_tensor_construction_and_resampling() {
    // 2x2 solid white image
    let rgb = vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];

    let tensor =
        ImageTensor::from_rgb_bytes(&rgb, 2, 2, 4, 4).expect("Tensor creation must succeed");

    assert_eq!(tensor.channels, 3);
    assert_eq!(tensor.width, 4);
    assert_eq!(tensor.height, 4);
    assert_eq!(tensor.data.len(), 3 * 4 * 4);

    // Verify ImageNet normalization for 255 (value 1.0)
    let expected_r = (1.0 - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
    let expected_g = (1.0 - IMAGENET_MEAN[1]) / IMAGENET_STD[1];
    let expected_b = (1.0 - IMAGENET_MEAN[2]) / IMAGENET_STD[2];

    let sample_r = tensor.get(0, 0, 0).unwrap();
    let sample_g = tensor.get(1, 0, 0).unwrap();
    let sample_b = tensor.get(2, 0, 0).unwrap();

    assert!((sample_r - expected_r).abs() < 1e-4);
    assert!((sample_g - expected_g).abs() < 1e-4);
    assert!((sample_b - expected_b).abs() < 1e-4);

    assert!((tensor.luminance - 1.0).abs() < 1e-3);
    // Flat color has near zero edge energy
    assert!(tensor.edge_energy < 0.05);
}

#[test]
fn test_image_tensor_edge_energy_on_contrast() {
    // 4x4 high-contrast checkerboard
    let mut rgb = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            let val = if (x + y) % 2 == 0 { 255 } else { 0 };
            rgb.extend_from_slice(&[val, val, val]);
        }
    }

    let tensor =
        ImageTensor::from_rgb_bytes(&rgb, 4, 4, 8, 8).expect("Checkerboard tensor must succeed");

    assert!(
        tensor.edge_energy > 0.5,
        "Checkerboard must yield noticeable edge energy"
    );
}

#[test]
fn test_image_tensor_underflow_error() {
    let incomplete_rgb = vec![255, 255]; // too short for 2x2x3
    let err = ImageTensor::from_rgb_bytes(&incomplete_rgb, 2, 2, 4, 4).unwrap_err();
    assert!(matches!(err, VisionError::PreprocessingFailed { .. }));
}
