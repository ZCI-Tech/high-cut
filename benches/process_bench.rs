use criterion::{black_box, criterion_group, criterion_main, Criterion};
use high_cut::ffmpeg::SilenceSegment;
use high_cut::Config;
use high_cut::Processor;

fn bench_heuristics(c: &mut Criterion) {
    let config = Config::default();
    let processor = Processor::new(config);

    // Generate 10,000 synthetic silence segments to stress the logic
    let mut silences = Vec::new();
    for i in 0..10000 {
        silences.push(SilenceSegment {
            start: (i * 10) as f64 + 5.0,
            end: (i * 10) as f64 + 8.0,
        });
    }
    let total_duration = 100000.0;

    c.bench_function("highlight_heuristics_10k_segments", |b| {
        b.iter(|| {
            processor.calculate_keep_segments(black_box(&silences), black_box(total_duration))
        })
    });
}

criterion_group!(benches, bench_heuristics);
criterion_main!(benches);
