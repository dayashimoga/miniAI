use miniai_vision::{
    ImageDimensions, ImageFormat, ImageMetadata, ImageTensor, VisionEngine, VisualQaQuery,
};

fn create_contrast_image_rgb(width: usize, height: usize) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        let is_band = (y / 15) % 2 == 0;
        let val = if is_band { 240 } else { 20 };
        for _ in 0..width {
            rgb.extend_from_slice(&[val, val, val]);
        }
    }
    rgb
}

#[test]
fn test_vision_engine_ocr_detection() {
    let engine = VisionEngine::default();
    let width = 120;
    let height = 120;
    let rgb = create_contrast_image_rgb(width, height);
    let dims = ImageDimensions::new(width as u32, height as u32);

    let ocr = engine
        .perform_ocr(&rgb, dims)
        .expect("OCR execution must succeed");
    assert!(
        !ocr.elements.is_empty(),
        "High-contrast bands must trigger OCR elements"
    );
    assert!(ocr.word_count > 0);

    for elem in &ocr.elements {
        assert!(elem.bounding_box.x_min >= 0.0 && elem.bounding_box.x_max <= 1.0);
        assert!(elem.bounding_box.y_min >= 0.0 && elem.bounding_box.y_max <= 1.0);
        assert!(elem.confidence >= 0.5);
    }
}

#[test]
fn test_vision_engine_scene_description() {
    let engine = VisionEngine::default();
    let width = 64;
    let height = 32;
    // Solid vibrant red
    let mut rgb = Vec::new();
    for _ in 0..(width * height) {
        rgb.extend_from_slice(&[240, 20, 20]);
    }

    let meta = ImageMetadata {
        format: ImageFormat::Png,
        dimensions: ImageDimensions::new(width as u32, height as u32),
        size_bytes: 1024,
        sha256: "dummy_sha256_for_test".to_string(),
        has_alpha: false,
    };

    let tensor = ImageTensor::from_rgb_bytes(&rgb, width, height, 32, 32)
        .expect("Tensor creation must succeed");

    let desc = engine.describe_scene(&meta, &tensor);
    assert_eq!(desc.aspect_ratio_label, "Landscape");
    assert!(desc.dominant_colors.contains(&"Vibrant Red".to_string()));
    assert!(desc.caption.contains("Landscape image"));
    assert!(!desc.detected_tags.is_empty());
}

#[test]
fn test_vision_engine_grounded_vqa_queries() {
    let engine = VisionEngine::default();
    let width = 100;
    let height = 100;
    let rgb = create_contrast_image_rgb(width, height);
    let dims = ImageDimensions::new(width as u32, height as u32);

    let meta = ImageMetadata {
        format: ImageFormat::Jpeg,
        dimensions: dims,
        size_bytes: 2048,
        sha256: "test_digest_123".to_string(),
        has_alpha: false,
    };

    let tensor = ImageTensor::from_rgb_bytes(&rgb, width, height, 32, 32).unwrap();
    let ocr = engine.perform_ocr(&rgb, dims).ok();

    // 1. Dimensions query
    let q_dim = VisualQaQuery {
        prompt: "What are the dimensions and size of this image?".to_string(),
    };
    let a_dim = engine.answer_visual_question(&q_dim, &meta, &tensor, ocr.as_ref());
    assert!(a_dim.answer.contains("100x100"));
    assert!(!a_dim.grounding_evidence.is_empty());
    assert!(a_dim.confidence >= 0.95);

    // 2. Color query
    let q_color = VisualQaQuery {
        prompt: "What are the dominant colors?".to_string(),
    };
    let a_color = engine.answer_visual_question(&q_color, &meta, &tensor, ocr.as_ref());
    assert!(a_color.confidence >= 0.85);

    // 3. OCR text query
    let q_text = VisualQaQuery {
        prompt: "Please read the text lines in this image".to_string(),
    };
    let a_text = engine.answer_visual_question(&q_text, &meta, &tensor, ocr.as_ref());
    assert!(a_text.answer.contains("detected text element"));

    // 4. Orientation query
    let q_ar = VisualQaQuery {
        prompt: "Is the image portrait or landscape?".to_string(),
    };
    let a_ar = engine.answer_visual_question(&q_ar, &meta, &tensor, ocr.as_ref());
    assert!(a_ar.answer.contains("Square (1:1)"));
}
