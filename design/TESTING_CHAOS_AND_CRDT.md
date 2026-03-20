# Adversarial CRDT Chaos Testing

In a multiplayer, Human+AI environment utilizing Event Sourcing and Last-Writer-Wins (LWW-Element-Set) CRDTs, unit testing the "happy path" is insufficient. 

We must prove that no matter how violently an AI agent and a human user clash over the same architecture diagram, the state always converges deterministically and never corrupts. We achieve this via **Adversarial Chaos Testing**.

## The Threat Model
1. **Human** drags "Database Node" 100 pixels to the right.
2. **AI Agent** executes a 15-step `seshat validate patch.json` refactoring that routes 3 arrows into the "Database Node".
3. **Network Skew**: The Human's events arrive at the SQLite WAL out-of-order or with delayed HLC (Hybrid Logical Clock) timestamps.

## The Chaos Harness
We use a custom Rust test harness built on `tokio` to spawn multiple asynchronous actor threads. 

1. **Actor A (Simulated Human)**: Rapidly fires position update events into the SQLite WAL.
2. **Actor B (Simulated AI)**: Fires complex edge-binding events into the same WAL.
3. **The Chaos Proxy**: We intentionally inject jitter, delay, and re-ordering into the event delivery mechanism.

```rust
#[tokio::test]
async fn test_crdt_convergence_under_heavy_load() {
    let db = setup_in_memory_sqlite().await;
    
    // Spawn 10 AI writers and 10 Human writers
    let mut handles = vec![];
    for i in 0..20 {
        let pool = db.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..100 {
                // Generate random CRDT operations
                let op = generate_random_mutation(i, j);
                // Introduce random artificial jitter (0-50ms)
                tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 50)).await;
                append_event(&pool, op).await.unwrap();
            }
        }));
    }
    
    futures::future::join_all(handles).await;
    
    // THE ASSERTION
    // No matter what order the 2,000 events arrived in the SQLite WAL,
    // if we spin up two independent read-replicas and replay the events,
    // their final hashes MUST match perfectly.
    let final_state_a = replay_events(&db).await;
    let final_state_b = replay_events_in_crdt_order(&db).await;
    
    assert_eq!(final_state_a.hash(), final_state_b.hash(), "CRDT Failed to converge!");
}
```

## Why This Matters
If this test fails, it means our CRDT implementation is flawed and users will experience "Zombie Nodes" or infinitely disconnecting arrows. 

By pushing this logic into the Pure Calc tier and fuzzing it with Chaos, AI agents writing patches can be 100% confident that if they dispatch a valid JSON spec, it will safely merge with human work without locking the UI.