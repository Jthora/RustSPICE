use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_spice::*;

fn benchmark_state_vector_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_vector_calculations");
    
    // Test different targets
    let targets = vec!["MOON", "MARS", "VENUS", "JUPITER"];
    let et = calendar_to_et(2025, 7, 22, 12, 0, 0.0);
    
    for target in targets {
        group.bench_with_input(
            BenchmarkId::new("get_state_vector", target),
            &target,
            |b, &target| {
                b.iter(|| {
                    get_state_vector(
                        black_box(target),
                        black_box(et),
                        black_box("J2000"),
                        black_box("NONE"),
                        black_box("EARTH")
                    )
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_time_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_conversions");
    
    group.bench_function("calendar_to_et", |b| {
        b.iter(|| {
            calendar_to_et(
                black_box(2025),
                black_box(7),
                black_box(22),
                black_box(12),
                black_box(0),
                black_box(0.0)
            )
        })
    });
    
    group.bench_function("julian_date_to_et", |b| {
        b.iter(|| {
            julian_date_to_et(black_box(2451545.0))  // J2000 epoch
        })
    });
    
    group.finish();
}

fn benchmark_coordinate_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordinate_systems");
    
    let reference_frames = vec!["J2000", "ECLIPJ2000", "IAU_EARTH"];
    let et = calendar_to_et(2025, 7, 22, 12, 0, 0.0);
    
    for frame in reference_frames {
        group.bench_with_input(
            BenchmarkId::new("reference_frame", frame),
            &frame,
            |b, &frame| {
                b.iter(|| {
                    get_state_vector(
                        black_box("MOON"),
                        black_box(et),
                        black_box(frame),
                        black_box("NONE"),
                        black_box("EARTH")
                    )
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_aberration_corrections(c: &mut Criterion) {
    let mut group = c.benchmark_group("aberration_corrections");
    
    let corrections = vec!["NONE", "LT", "LT+S", "CN", "CN+S"];
    let et = calendar_to_et(2025, 7, 22, 12, 0, 0.0);
    
    for correction in corrections {
        group.bench_with_input(
            BenchmarkId::new("aberration", correction),
            &correction,
            |b, &correction| {
                b.iter(|| {
                    get_state_vector(
                        black_box("MOON"),
                        black_box(et),
                        black_box("J2000"),
                        black_box(correction),
                        black_box("EARTH")
                    )
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");
    
    // Simulate kernel loading
    group.bench_function("simulate_kernel_load", |b| {
        let dummy_data = vec![0u8; 1024 * 1024];  // 1MB of dummy kernel data
        b.iter(|| {
            // Simulate processing kernel data
            let _checksum: u32 = dummy_data.iter().map(|&x| x as u32).sum();
            black_box(_checksum)
        })
    });
    
    // State vector creation and manipulation
    group.bench_function("state_vector_creation", |b| {
        b.iter(|| {
            let state = StateVector::new(
                black_box(384400.0),  // Moon distance
                black_box(0.0),
                black_box(0.0),
                black_box(0.0),
                black_box(1.0),
                black_box(0.0),
                black_box(1.28)  // Light time to Moon
            );
            black_box(state)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_state_vector_calculation,
    benchmark_time_conversions,
    benchmark_coordinate_systems,
    benchmark_aberration_corrections,
    benchmark_memory_operations
);

criterion_main!(benches);
