//! Criterion benchmarks for ghurni mechanical sound synthesis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ghurni::prelude::*;

fn bench_gasoline_v8_1s(c: &mut Criterion) {
    c.bench_function("gasoline_v8_1s", |b| {
        let mut engine = Engine::new(EngineType::Gasoline, 8, 44100.0).unwrap();
        engine.set_firing_order(vec![0.0, 90.0, 270.0, 180.0, 540.0, 630.0, 450.0, 360.0]);
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

fn bench_transmission_500ms(c: &mut Criterion) {
    c.bench_function("transmission_5speed_500ms", |b| {
        let mut trans = Transmission::new(vec![3.5, 2.1, 1.4, 1.0, 0.8], 24, 44100.0).unwrap();
        b.iter(|| {
            let samples = trans.synthesize(3000.0, 0.5).unwrap();
            black_box(samples);
        });
    });
}

fn bench_turbocharger_1s(c: &mut Criterion) {
    c.bench_function("turbocharger_1s", |b| {
        let mut turbo = ForcedInduction::new(InductionType::Turbo, 2.5, 1.0, 44100.0).unwrap();
        b.iter(|| {
            let samples = turbo.synthesize(5000.0, 0.8, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_mixer_3channel_1s(c: &mut Criterion) {
    c.bench_function("mixer_3ch_1s", |b| {
        let mut mixer = MechanicalMixer::new();
        mixer.add_channel(
            "engine".into(),
            Box::new(Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap()),
        );
        mixer.add_channel(
            "gear".into(),
            Box::new(Gear::new(32, GearMaterial::Steel, 44100.0).unwrap()),
        );
        mixer.add_channel(
            "turbo".into(),
            Box::new(ForcedInduction::new(InductionType::Turbo, 2.5, 1.0, 44100.0).unwrap()),
        );
        mixer.set_rpm(3000.0);
        b.iter(|| {
            let mut output = vec![0.0f32; 44100];
            mixer.process_block(&mut output);
            black_box(output);
        });
    });
}

fn bench_process_block_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_sizes");
    for size in [64, 128, 256, 512, 1024, 4096] {
        group.bench_function(format!("engine_{size}"), |b| {
            let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
            engine.set_rpm(3000.0);
            engine.set_load(0.5);
            let mut buf = vec![0.0f32; size];
            b.iter(|| {
                engine.process_block(&mut buf);
                black_box(&buf);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_gasoline_v8_1s,
    bench_diesel_6cyl_1s,
    bench_gear_steel_500ms,
    bench_motor_brushless_1s,
    bench_turbine_1s,
    bench_wristwatch_1s,
    bench_transmission_500ms,
    bench_turbocharger_1s,
    bench_mixer_3channel_1s,
    bench_process_block_sizes,
);

criterion_main!(benches);
