use miniai_video::buffer_pool::{FrameBufferPool, SharedFrameBufferPool};
use miniai_video::types::{BufferPoolConfig, VideoFrame};
use miniai_video::VideoError;

#[test]
fn test_buffer_pool_allocation_and_release() {
    let config = BufferPoolConfig {
        budget_bytes: 10 * 1024 * 1024, // 10 MB
        max_cached_frames: 4,
    };
    let mut pool = FrameBufferPool::new(config);

    // 100x100 RGB = 30,000 bytes
    let buf1 = pool.allocate_buffer(100, 100).expect("Allocate buf1");
    assert_eq!(buf1.len(), 30_000);

    let stats = pool.stats();
    assert_eq!(stats.allocated_bytes, 30_000);
    assert_eq!(stats.active_frames, 1);
    assert_eq!(stats.peak_bytes, 30_000);

    // Allocate second buffer
    let buf2 = pool.allocate_buffer(100, 100).expect("Allocate buf2");
    assert_eq!(pool.stats().allocated_bytes, 60_000);
    assert_eq!(pool.stats().active_frames, 2);

    // Release buf1
    pool.release_buffer(buf1);
    assert_eq!(pool.stats().allocated_bytes, 30_000);
    assert_eq!(pool.stats().active_frames, 1);

    // Releasing buf2
    pool.release_buffer(buf2);
    assert_eq!(pool.stats().allocated_bytes, 0);
    assert_eq!(pool.stats().active_frames, 0);

    // Next allocation should reuse cached buffer
    let buf3 = pool
        .allocate_buffer(100, 100)
        .expect("Allocate buf3 (reused)");
    assert_eq!(buf3.len(), 30_000);
    assert_eq!(pool.stats().allocated_bytes, 30_000);
}

#[test]
fn test_buffer_pool_budget_exhaustion() {
    let config = BufferPoolConfig {
        budget_bytes: 50_000, // Small budget (50 KB)
        max_cached_frames: 2,
    };
    let mut pool = FrameBufferPool::new(config);

    // 100x100 = 30,000 bytes (fits)
    let _buf1 = pool.allocate_buffer(100, 100).expect("Fits in budget");

    // Second 100x100 = 30,000 bytes -> total 60,000 bytes > 50,000 budget!
    let err = pool
        .allocate_buffer(100, 100)
        .expect_err("Should exceed budget");
    assert!(matches!(err, VideoError::BufferPoolExhausted { .. }));
}

#[test]
fn test_release_frame_helper() {
    let config = BufferPoolConfig::default();
    let mut pool = FrameBufferPool::new(config);

    let frame = VideoFrame {
        frame_index: 0,
        timestamp_ms: 0,
        width: 10,
        height: 10,
        rgb_data: pool.allocate_buffer(10, 10).expect("alloc"),
        is_keyframe: true,
        scene_id: 1,
        luminance_mean: 0.5,
        sha256: "dummy".to_string(),
    };

    assert_eq!(pool.stats().active_frames, 1);
    pool.release_frame(frame);
    assert_eq!(pool.stats().active_frames, 0);
}

#[test]
fn test_shrink_to_fit() {
    let config = BufferPoolConfig::default();
    let mut pool = FrameBufferPool::new(config);

    let buf = pool.allocate_buffer(50, 50).expect("alloc");
    pool.release_buffer(buf);
    pool.shrink_to_fit();
    // Verify pool is still completely functional
    let buf2 = pool.allocate_buffer(50, 50).expect("alloc after shrink");
    assert_eq!(buf2.len(), 7500);
}

#[test]
fn test_shared_frame_buffer_pool() {
    let shared = SharedFrameBufferPool::new(BufferPoolConfig::default());
    let buf = shared.allocate_buffer(64, 64).expect("Shared alloc");
    assert_eq!(buf.len(), 64 * 64 * 3);
    let stats = shared.stats().expect("Stats");
    assert_eq!(stats.active_frames, 1);
    shared.release_buffer(buf);
    assert_eq!(shared.stats().expect("Stats").active_frames, 0);
}
