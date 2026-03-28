//! Criterion benchmarks for ghurni mechanical sound synthesis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ghurni::prelude::*;

fn bench_gasoline_v8_1s(c: &mut Criterion) {
    c.bench_function("gasoline_v8_1s", |b| {
        let mut engine = Engine::new(EngineType::Gasoline, 8, 44100.0).unwrap();
        b.iter(|| {
            let samples = engine.synthesize(4000.0, 0.6, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_diesel_6cyl_1s(c: &mut Criterion) {
    c.bench_function("diesel_6cyl_1s", |b| {
        let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
        b.iter(|| {
            let samples = engine.synthesize(2000.0, 0.7, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_gear_steel_500ms(c: &mut Criterion) {
    c.bench_function("gear_steel_500ms", |b| {
        let mut gear = Gear::new(32, GearMaterial::Steel, 44100.0).unwrap();
        b.iter(|| {
            let samples = gear.synthesize(3000.0, 0.5).unwrap();
            black_box(samples);
        });
    });
}

fn bench_motor_brushless_1s(c: &mut Criterion) {
    c.bench_function("motor_brushless_1s", |b| {
        let mut motor = Motor::new(MotorType::Brushless, 8, 44100.0).unwrap();
        b.iter(|| {
            let samples = motor.synthesize(10000.0, 0.5, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_turbine_1s(c: &mut Criterion) {
    c.bench_function("turbine_16blade_1s", |b| {
        let mut turbine = Turbine::new(16, 500.0, 44100.0).unwrap();
        b.iter(|| {
            let samples = turbine.synthesize(20000.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_wristwatch_1s(c: &mut Criterion) {
    c.bench_function("wristwatch_1s", |b| {
        let mut clock = Clock::new(ClockType::Wristwatch, 44100.0).unwrap();
        b.iter(|| {
            let samples = clock.synthesize(1.0).unwrap();
            black_box(samples);
        });
    });
}

criterion_group!(
    benches,
    bench_gasoline_v8_1s,
    bench_diesel_6cyl_1s,
    bench_gear_steel_500ms,
    bench_motor_brushless_1s,
    bench_turbine_1s,
    bench_wristwatch_1s,
);

criterion_main!(benches);
