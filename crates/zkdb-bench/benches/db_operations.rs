use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use zkdb_lib::{Database, DatabaseType};
use zkdb_store::file::FileStore;

// Helper function to set up a clean database for each benchmark
async fn setup_db() -> (Database, Arc<FileStore>, TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a subdirectory for the database files
    let db_path = temp_dir.path().join("db");

    // Create store first, which will handle directory creation
    let store = Arc::new(FileStore::new(&db_path).await.unwrap());

    // Then create database
    let db = Database::new(DatabaseType::Merkle, store.clone(), None)
        .await
        .unwrap();

    (db, store, temp_dir)
}

fn create_benchmark_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

// Benchmark single put operation
fn bench_put(c: &mut Criterion) {
    let rt = create_benchmark_runtime();

    let mut group = c.benchmark_group("put_operations");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(5));

    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let value = vec![0u8; *size];
            b.to_async(&rt).iter_batched(
                || setup_db(),
                |setup_future| async {
                    let (mut db, _, _) = setup_future.await;
                    db.put("test_key", &value, false).await.unwrap();
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// Benchmark single get operation
fn bench_get(c: &mut Criterion) {
    let rt = create_benchmark_runtime();

    let mut group = c.benchmark_group("get_operations");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(5));

    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let value = vec![0u8; *size];
            b.to_async(&rt).iter_batched(
                || async {
                    let (mut db, _, _) = setup_db().await;
                    db.put("test_key", &value, false).await.unwrap();
                    db
                },
                |db| async move {
                    // Now we await the setup future first
                    let db = db.await;
                    db.get("test_key", false).await.unwrap()
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// Benchmark proof generation
fn bench_proof_generation(c: &mut Criterion) {
    let rt = create_benchmark_runtime();

    let mut group = c.benchmark_group("proof_generation");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(5));

    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            b.to_async(&rt).iter_batched(
                || async {
                    let (mut db, _, _) = setup_db().await;
                    for i in 0..*size {
                        let key = format!("key_{}", i);
                        let value = vec![i as u8; 100];
                        db.put(&key, &value, false).await.unwrap();
                    }
                    let key = format!("key_{}", size - 1);
                    (db, key)
                },
                |future_result| async move {
                    // Await the setup future to get the db and key
                    let (db, key) = future_result.await;
                    db.get(&key, true).await.unwrap()
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// Benchmark batch operations
fn bench_batch_operations(c: &mut Criterion) {
    let rt = create_benchmark_runtime();

    let mut group = c.benchmark_group("batch_operations");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(5));

    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            b.to_async(&rt).iter_batched(
                || setup_db(),
                |setup_future| async {
                    let (mut db, _, _) = setup_future.await;
                    for i in 0..*size {
                        let key = format!("key_{}", i);
                        let value = vec![i as u8; 100];
                        db.put(&key, &value, false).await.unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_put,
    bench_get,
    bench_proof_generation,
    bench_batch_operations
);
criterion_main!(benches);
